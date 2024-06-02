//this file generates the interface
use super::super::*;

#[cfg(feature = "java_wrapper")]
use crate::ffi::jni_c_header::*;

//-------------------------------SwordFish Port Finder-----------------------------------
foreign_class!(
    class SwordFishPortFinder {
        fn find_probable_swordfish_port() -> String {
            let result = match swordfish_comm::find_probable_swordfish_port() {
                Some(port_name) => port_name.to_string(),
                None => "".to_string(),
            };
            result
        }
        fn get_serial_ports() -> String {
            let result = match swordfish_comm::get_serial_ports() {
                Some(ports) => ports,
                None => "".to_string(),
            };
            result
        }
    }
);

//-------------------------------SwordFish Comm-----------------------------------
use swordfish_comm::SwordFishComm as SwordFishComm;
foreign_class!(
    class SwordFishComm {
        self_type SwordFishComm;
        constructor SwordFishComm::new(port_name: &str) -> SwordFishComm {
            let swordfish_comm = SwordFishComm::new(port_name)
                .expect("Failed to create SwordFishComm");
            swordfish_comm
        }
        // fn SwordFishComm::change_message_rx_callback(&self, opcode: u8, callback: Box<dyn Fn(SwordFishConcentratedMessage) + Send>);
        fn SwordFishComm::send_msg(&self, msg: SwordFishConcentratedMessage) -> Option<SwordFishConcentratedMessage>;
        fn SwordFishComm::get_tx_counter(&self) -> usize;
        fn SwordFishComm::get_rx_counter(&self) -> usize;
    }

);

//-------------------------------SwordFish ConceneratedMessages-----------------------
use swordfish_concentrated_message::SwordFishConcentratedMessage as SwordFishConcentratedMessage;
impl SwordFishConcentratedMessage {
    pub fn print(&self) {
        println!("{:?}", self);
    }
}

foreign_class!(
    class SwordFishConcentratedMessage {
        self_type SwordFishConcentratedMessage;
        constructor SwordFishConcentratedMessage::new(counter: u16, opcode: u8, payload: &[u8]) -> SwordFishConcentratedMessage;
        fn SwordFishConcentratedMessage::print(&self);
    }
);

//-------------------------------SwordFish Messages----------------------------------
//--------------Ping------------------//
use swordfish_messages::Ping as PingMessage;

foreign_class!(
    class PingMessage {
        self_type PingMessage;
        constructor new() -> PingMessage {PingMessage::default()}
        fn PingMessage::print(&self);
        fn PingMessage::to_concentrated(&self, counter: u16) -> SwordFishConcentratedMessage;
        fn PingMessage::from_concentrated(concenrated_msg: &SwordFishConcentratedMessage) -> Option<PingMessage> {
            match PingMessage::from_concentrated(concenrated_msg) {
                Ok(msg) => Some(msg),
                Err(_) => None,
            }
        }
    }
);

//----------------VersionData----------------//
use swordfish_messages::VersionData as VersionDataMessage;
impl VersionDataMessage {
    pub fn new(version : u8, subversion : u8, mcu_type : u32, uuid : &[u8]) -> Self {
        VersionDataMessage {
            version,
            subversion,
            mcu_type,
            uuid: {
                let mut uuid_arr = [0; 8];
                let len = std::cmp::min(uuid.len(), 8);
                uuid_arr[..len].copy_from_slice(&uuid[..len]);
                uuid_arr
            }
        }
    }
    pub fn get_version(&self) -> u8 {self.version}
    pub fn get_subversion(&self) -> u8 {self.subversion}
    pub fn get_mcu_type(&self) -> u32 {self.mcu_type}
    pub fn get_uuid(&self) -> Vec<u8> {self.uuid.to_vec()}
    pub fn set_version(&mut self, version: u8) {self.version = version}
    pub fn set_subversion(&mut self, subversion: u8) {self.subversion = subversion}
    pub fn set_mcu_type(&mut self, mcu_type: u32) {self.mcu_type = mcu_type}
    pub fn set_uuid(&mut self, uuid: &[u8]) {
        self.uuid = [0; 8];
        let len = std::cmp::min(uuid.len(), 8);
        self.uuid[..len].copy_from_slice(&uuid[..len]);
    }

}


foreign_class!(
    class VersionDataMessage
    {
        self_type VersionDataMessage;
        constructor VersionDataMessage::new(version : u8, subversion : u8, mcu_type : u32, uuid : &[u8]) -> VersionDataMessage;
        fn make_empty() -> VersionDataMessage {VersionDataMessage::default()}
        fn VersionDataMessage::print(&self);
        fn VersionDataMessage::to_concentrated(&self, counter: u16) -> SwordFishConcentratedMessage;
        fn VersionDataMessage::from_concentrated(concenrated_msg: &SwordFishConcentratedMessage) -> Option<VersionDataMessage> {
            match VersionDataMessage::from_concentrated(concenrated_msg) {
                Ok(msg) => Some(msg),
                Err(_) => None,
            }
        }
        //specific getters
        fn VersionDataMessage::get_version(&self) -> u8;
        fn VersionDataMessage::get_subversion(&self) -> u8;
        fn VersionDataMessage::get_mcu_type(&self) -> u32;
        fn VersionDataMessage::get_uuid(&self) -> Vec<u8>;
        //specific setters
        fn VersionDataMessage::set_version(&mut self, version: u8);
        fn VersionDataMessage::set_subversion(&mut self, subversion: u8);
        fn VersionDataMessage::set_mcu_type(&mut self, mcu_type: u32);
        fn VersionDataMessage::set_uuid(&mut self, uuid: &[u8]);
    }
);

