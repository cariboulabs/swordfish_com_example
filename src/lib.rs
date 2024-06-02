pub mod swordfish_comm;
mod swordfish_concentrated_message;
pub mod swordfish_messages;
pub use swordfish_concentrated_message::SwordFishConcentratedMessage;
pub use swordfish_concentrated_message::TOTAL_MESSAGE_SIZE as CONCENTRATED_MESSAGE_TOTAL_SIZE;
mod ffi;

//---------------------Buckets and Catagories---------------------
use std::sync::{Condvar, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwordFishMessageCategory {
    Bounce,                //message is sent to swordfish and swordfish sends it back
    Param,                 //message is sent to swordfish to set/get parameters
    Operation(Option<u8>), //message is sent to swordfish to perform an operation, the u8 is the opcode of the response msg
    Response,              //message that is sent from swordfish as a response to an operation,
}

pub struct SwordFishMessageBucket {
    pub message: Mutex<Option<SwordFishConcentratedMessage>>,
    pub on_rx_callback: Mutex<Option<Box<dyn FnMut(SwordFishConcentratedMessage) + Send>>>,
    pub catagory: SwordFishMessageCategory,
    pub condvar: Condvar,
}

impl SwordFishMessageBucket {
    pub fn new(catagory: SwordFishMessageCategory) -> Self {
        SwordFishMessageBucket {
            message: Mutex::new(None),
            on_rx_callback: Mutex::new(None),
            catagory,
            condvar: Condvar::new(),
        }
    }
}

//---------------------SwordFishMessageTrait---------------------
use crate::swordfish_concentrated_message::MAX_PAYLOAD_SIZE;
use anyhow::{anyhow, Result};
use inline_colorization::{color_red, color_reset};
use std::mem::{self, MaybeUninit};

pub trait SwordFishMessageTrait
where
    Self: Sized + Default + std::fmt::Debug,
{
    const OPCODE: u8;
    const CATEGORY: SwordFishMessageCategory;

    fn print(&self) {
        println!("  Opcode: {}", Self::OPCODE);
        println!("  Category: {:?}", Self::CATEGORY);
        println!("      {:?}", self);
    }

    fn get_payload_length() -> usize {
        return std::mem::size_of::<Self>();
    }

    fn to_concentrated(&self, counter: u16) -> SwordFishConcentratedMessage {
        let length = Self::get_payload_length();
        if length > MAX_PAYLOAD_SIZE {
            panic!("{color_red}Payload too large, this should never happen{color_reset}");
        }
        let payload =
            unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, length) };
        SwordFishConcentratedMessage::new(counter, Self::OPCODE, &payload)
    }
    fn from_concentrated(concenrated_msg: &SwordFishConcentratedMessage) -> Result<Self> {
        if Self::OPCODE != concenrated_msg.opcode {
            return Err(anyhow!(
                "Wrong opcode, expected {}, got {}",
                Self::OPCODE,
                concenrated_msg.opcode
            ));
        } else if concenrated_msg.length as usize != std::mem::size_of::<Self>() {
            return Err(anyhow!(
                "Wrong length, expected {}, got {}",
                std::mem::size_of::<Self>(),
                concenrated_msg.length
            ));
        } else {
            let mut uninit = MaybeUninit::<Self>::uninit();
            let ptr = uninit.as_mut_ptr() as *mut u8;
            unsafe {
                ptr.copy_from_nonoverlapping(
                    concenrated_msg.payload.as_ptr(),
                    mem::size_of::<Self>(),
                );
                Ok(uninit.assume_init())
            }
        }
    }
}
