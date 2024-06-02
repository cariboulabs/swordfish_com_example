use inline_colorization::{color_red, color_reset};
pub const MAX_PAYLOAD_SIZE: usize = 245;
pub const TOTAL_MESSAGE_SIZE: usize = 255;
pub const HEADER_SIZE: usize = 9;
const SYNC_WORD_TO_SWORDFISH_U32: u32 = 0xefbeadde;
pub const SYNC_WORD_FROM_SWORDFISH: [u8; 4] = [0xde, 0xad, 0xbe, 0xef];
pub const _SYNC_WORD_TO_SWORDFISH: [u8; 4] = [0xef, 0xbe, 0xad, 0xde];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwordFishConcentratedMessage {
    pub sync_word: u32,
    pub counter: u16,
    pub opcode: u8,
    pub length: u16,
    pub payload: [u8; MAX_PAYLOAD_SIZE],
    pub checksum: u8,
}

impl SwordFishConcentratedMessage {
    pub fn new(counter: u16, opcode: u8, payload: &[u8]) -> Self {
        let length = payload.len() as u16;
        SwordFishConcentratedMessage {
            counter: counter,
            sync_word: SYNC_WORD_TO_SWORDFISH_U32,
            opcode: opcode,
            length: length,
            payload: if length > 0 {
                let mut p = [0; MAX_PAYLOAD_SIZE];
                p[..payload.len()].copy_from_slice(payload);
                p
            } else {
                [0; MAX_PAYLOAD_SIZE]
            },
            checksum: SwordFishConcentratedMessage::calculate_checksum(
                SYNC_WORD_TO_SWORDFISH_U32,
                counter,
                opcode,
                length,
                &payload,
            ),
        }
    }

    fn calculate_checksum(
        sync_word: u32,
        counter: u16,
        opcode: u8,
        length: u16,
        payload: &[u8],
    ) -> u8 {
        let counter_low_byte = counter & 0xff;
        let counter_high_byte = (counter >> 8) & 0xff;
        let length_low_byte = length & 0xff;
        let length_high_byte = (length >> 8) & 0xff;
        let mut checksum: u8 = opcode
            .wrapping_add(length_low_byte as u8)
            .wrapping_add(length_high_byte as u8)
            .wrapping_add(counter_low_byte as u8)
            .wrapping_add(counter_high_byte as u8);

        for byte in sync_word.to_ne_bytes().iter() {
            checksum = checksum.wrapping_add(*byte);
        }
        if length > 0 {
            for byte in payload.iter() {
                checksum = checksum.wrapping_add(*byte);
            }
        }
        checksum
    }

    pub fn into_bytes(self) -> Box<[u8]> {
        //future: fix this by building correct size of buffer to begin wtth
        let mut buffer = [0; TOTAL_MESSAGE_SIZE];
        buffer[..4].copy_from_slice(&self.sync_word.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.counter.to_le_bytes());
        buffer[6] = self.opcode;
        buffer[7..HEADER_SIZE].copy_from_slice(&self.length.to_le_bytes());
        buffer[HEADER_SIZE..HEADER_SIZE + self.length as usize]
            .copy_from_slice(&self.payload[..self.length as usize]);
        buffer[HEADER_SIZE + self.length as usize] = self.checksum; //last index is 254

        let cut_buffer = &buffer[..HEADER_SIZE + self.length as usize + 1];
        cut_buffer.to_vec().into_boxed_slice()
    }
}

impl Default for SwordFishConcentratedMessage {
    fn default() -> Self {
        SwordFishConcentratedMessage {
            sync_word: 0,
            counter: 0,
            opcode: 0,
            length: 0,
            payload: [0; MAX_PAYLOAD_SIZE],
            checksum: 0,
        }
    }
}

pub struct SwordFishConcentratedMessageBufferBuilder {
    accumulated_buffer: [u8; TOTAL_MESSAGE_SIZE * 3],
    n_accum_bytes: usize,
}

impl SwordFishConcentratedMessageBufferBuilder {
    pub fn new() -> Self {
        SwordFishConcentratedMessageBufferBuilder {
            accumulated_buffer: [0; TOTAL_MESSAGE_SIZE * 3],
            n_accum_bytes: 0,
        }
    }

    pub fn append_buffer(&mut self, buffer: &[u8]) -> Option<SwordFishConcentratedMessage> {
        //copy buffer into accumulated buffer
        if self.n_accum_bytes + buffer.len() > self.accumulated_buffer.len() {
            panic!(
                "{color_red}Accumulated buffer overflow, buffer size: {}, required size: {}{color_reset}",
                self.accumulated_buffer.len(),
                self.n_accum_bytes + buffer.len()
            );
        }

        //copy buffer into accumulated buffer and update n_accum_bytes
        self.accumulated_buffer[self.n_accum_bytes..self.n_accum_bytes + buffer.len()]
            .copy_from_slice(buffer);
        self.n_accum_bytes += buffer.len();

        if self.accumulated_buffer.len() < HEADER_SIZE {
            //nothing to look for
            return None;
        }

        let sync_word_window = self
            .accumulated_buffer
            .windows(4)
            .position(|window| window == SYNC_WORD_FROM_SWORDFISH);
        if let Some(start_pos) = sync_word_window {
            let payload_length = u16::from_le_bytes([buffer[start_pos + 7], buffer[start_pos + 8]]);
            if payload_length > MAX_PAYLOAD_SIZE as u16 {
                //bad message, remove it
                return None;
            }
            let msg_length = HEADER_SIZE + payload_length as usize + 1; //+1 for checksum
            let msg_end = start_pos + msg_length;
            let msg_range = start_pos..msg_end;

            let msg_buffer = &self.accumulated_buffer[msg_range.clone()];
            let sync_word =
                u32::from_le_bytes([msg_buffer[0], msg_buffer[1], msg_buffer[2], msg_buffer[3]]);
            let counter = u16::from_le_bytes([msg_buffer[4], msg_buffer[5]]);
            let opcode = msg_buffer[6];

            let payload = {
                let mut p = [0; MAX_PAYLOAD_SIZE];
                let payload_start = HEADER_SIZE;
                let payload_end = HEADER_SIZE + payload_length as usize;
                p[..payload_length as usize]
                    .copy_from_slice(&msg_buffer[payload_start..payload_end]);
                p
            };

            let checksum = msg_buffer[msg_buffer.len() - 1];
            let calc_checksum = SwordFishConcentratedMessage::calculate_checksum(
                sync_word,
                counter,
                opcode,
                payload_length,
                &payload,
            );
            if calc_checksum == checksum {
                //remove msg_range.end bytes from accumulated_buffer by copying the rest of the buffer to the beginning
                self.accumulated_buffer.copy_within(msg_range.end.., 0);
                self.n_accum_bytes -= msg_range.end;
                Some(SwordFishConcentratedMessage {
                    sync_word: sync_word,
                    counter: counter,
                    opcode: opcode,
                    length: payload_length,
                    payload: payload,
                    checksum: checksum,
                })
            } else {
                //bad message, wrong checksum, remove it
                return None;
            }
        } else {
            //couldnt find sync word
            if self.n_accum_bytes >= 2 * TOTAL_MESSAGE_SIZE {
                //remove TOTAL_MESSAGE_SIZE bytes from buffer by shifting the rest of the buffer to the beginning
                //this prevent accumulation of bad messages that will cause buffer overflow
                self.accumulated_buffer.copy_within(TOTAL_MESSAGE_SIZE.., 0);
                self.n_accum_bytes -= TOTAL_MESSAGE_SIZE;
            }
            return None;
        }
    }
}
