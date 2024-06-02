use crate::{SwordFishMessageBucket, SwordFishMessageCategory, SwordFishMessageTrait};
use std::collections::HashMap;

//--------------Ping------------------//
#[repr(C, packed(1))]
#[derive(Debug, Default)]
pub struct Ping {}
impl SwordFishMessageTrait for Ping {
    const OPCODE: u8 = 0;
    const CATEGORY: SwordFishMessageCategory = SwordFishMessageCategory::Bounce;
}

//----------------VersionData----------------//
#[repr(C, packed(1))]
#[derive(Debug, PartialEq, Eq, Default)] //partial Eq and Eq are needed for the tests
pub struct VersionData {
    pub version: u8,
    pub subversion: u8,
    pub mcu_type: u32,
    pub uuid: [u8; 8],
}
impl SwordFishMessageTrait for VersionData {
    const OPCODE: u8 = 2;
    const CATEGORY: SwordFishMessageCategory = SwordFishMessageCategory::Bounce;
}

pub fn create_swordfish_messages_hashmap() -> HashMap<u8, SwordFishMessageBucket> {
    let mut map = HashMap::new();
    map.insert(Ping::OPCODE, SwordFishMessageBucket::new(Ping::CATEGORY));
    map.insert(
        VersionData::OPCODE,
        SwordFishMessageBucket::new(VersionData::CATEGORY),
    );
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swordfish_concentrated_message::SwordFishConcentratedMessageBufferBuilder;

    #[test]
    fn to_from() {
        let input_version_data = VersionData::default();
        let input_concentrated_data = input_version_data.to_concentrated(0);
        let bytes = input_concentrated_data.into_bytes();
        let mut concentrated_message_builder = SwordFishConcentratedMessageBufferBuilder::new();
        let output_concenrated_msg = concentrated_message_builder.append_buffer(&bytes).unwrap();
        assert_eq!(input_concentrated_data, output_concenrated_msg);
        let output_version_data = VersionData::from_concentrated(&output_concenrated_msg).unwrap();
        assert_eq!(input_version_data, output_version_data);
    }
}