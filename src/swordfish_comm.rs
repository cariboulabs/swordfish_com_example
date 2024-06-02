use crate::swordfish_concentrated_message::{
    SwordFishConcentratedMessage, SwordFishConcentratedMessageBufferBuilder,
};
use crate::swordfish_messages::create_swordfish_messages_hashmap;
use crate::{SwordFishMessageBucket, SwordFishMessageCategory, CONCENTRATED_MESSAGE_TOTAL_SIZE};
use inline_colorization::{color_red, color_reset};
use log;
use serialport::{DataBits, Parity, SerialPort, StopBits};
use std::collections::HashMap;
use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{spawn, JoinHandle};
use std::time::Duration;

pub fn get_serial_ports() -> Option<String> {
    match serialport::available_ports() {
        Ok(ports) => {
            let mut ports_string = String::new();
            ports_string.push_str(&format!("Num of devices: {}\n", ports.len()));
            for (i, port_info) in ports.iter().enumerate() {
                let port_name = &port_info.port_name;
                match &port_info.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        ports_string.push_str(&format!("{} : USB Port : {}\n", i, port_name));
                        ports_string.push_str(&format!("  - VID: 0x{:04x}\n", info.vid));
                        ports_string.push_str(&format!("  - PID: 0x{:04x}\n", info.pid));
                        ports_string.push_str(&format!("  - Serial Number: {}\n", info.serial_number.as_deref().unwrap_or("None")));
                        ports_string.push_str(&format!("  - Manufacturer: {}\n", info.manufacturer.as_deref().unwrap_or("None")));
                        ports_string.push_str(&format!("  - Product: {}\n", info.product.as_deref().unwrap_or("None")));
                    }
                    serialport::SerialPortType::BluetoothPort => {
                        ports_string.push_str(&format!("{} : Bluetooth Port : {}\n", i, port_name));
                    }
                    serialport::SerialPortType::PciPort => {
                        ports_string.push_str(&format!("{} : PCI Port : {}\n", i, port_name));
                    }
                    serialport::SerialPortType::Unknown => {
                        ports_string.push_str(&format!("{} : Unknown Port : {}\n", i, port_name));
                    }
                }
            }
            return Some(ports_string);
        }
        Err(e) => {
            log::error!("Error listing serial ports: {:?}", e);
            return None;
        }
    }
}

pub fn find_probable_swordfish_port() -> Option<String> {
    match serialport::available_ports() {
        Ok(ports) => {
            for port_info in ports {
                if let serialport::SerialPortType::UsbPort(info) = port_info.port_type {
                    if info.manufacturer.as_deref() == Some("Silicon Labs")
                        && info.vid == 0x10C4
                        && info.pid == 0xEA60
                    {
                        return Some(port_info.port_name);
                    } else if info.manufacturer.as_deref() == Some("FTDI")
                        && info.vid == 0x0403
                        && info.pid == 0x6015
                    {
                        return Some(port_info.port_name);
                    }
                }
            }
            None
        }
        Err(e) => {
            log::error!(
                "{}-{} : Error listing serial ports: {:?}",
                file!(),
                line!(),
                e
            );
            None
        }
    }
}

static INSTANCE_COUNTER: AtomicUsize = AtomicUsize::new(0);
pub struct SwordFishComm {
    //thread to run read operations
    thread_handle: Option<JoinHandle<()>>,
    thread_alive: Arc<AtomicBool>,
    transmitter: Sender<SwordFishConcentratedMessage>,
    tx_counter: Arc<AtomicUsize>,
    rx_counter: Arc<AtomicUsize>,
    messages_hashmap: Arc<RwLock<HashMap<u8, SwordFishMessageBucket>>>,
}

impl SwordFishComm {
    pub fn new(portpath: &str) -> Result<SwordFishComm, serialport::Error> {
        if INSTANCE_COUNTER.load(Ordering::SeqCst) > 0 {
            panic!("{color_red}Only one instance of SwordFishComm is allowed{color_reset}");
        } else {
            INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed);
        }

        let port_builder = serialport::new(portpath, 115200)
            .stop_bits(StopBits::One)
            .parity(Parity::None)
            .data_bits(DataBits::Eight)
            .timeout(Duration::from_millis(0)); //non-blocking

        let swordfish_messages_hashmap = Arc::new(RwLock::new(create_swordfish_messages_hashmap()));

        let (master_transmitter, slave_receiver) = mpsc::channel::<SwordFishConcentratedMessage>();

        let mut port: Box<dyn SerialPort> = port_builder.open()?;
        let thread_alive = Arc::new(AtomicBool::new(true));
        let rx_counter = Arc::new(AtomicUsize::new(0));
        let tx_counter = Arc::new(AtomicUsize::new(0));

        let swordfish_messages_hashmap_clone = swordfish_messages_hashmap.clone();
        let thread_alive_clone = thread_alive.clone();
        let rx_counter_clone = Arc::clone(&rx_counter);
        let tx_counter_clone = Arc::clone(&tx_counter);
        let thread_handle = spawn(move || {
            let mut read_buffer = [0; CONCENTRATED_MESSAGE_TOTAL_SIZE];
            let mut concentrated_messsage_builder: SwordFishConcentratedMessageBufferBuilder =
                SwordFishConcentratedMessageBufferBuilder::new();
            while thread_alive_clone.load(Ordering::Relaxed) {
                //check if there is anything to write
                if let Ok(msg) = slave_receiver.try_recv() {
                    let buffer = msg.into_bytes();
                    match port.write(&buffer) {
                        Ok(_n_bytes_written) => match port.flush() {
                            Ok(_) => {
                                tx_counter_clone.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(e) => log::error!("{}-{} : {:?}", file!(), line!(), e),
                        },
                        Err(e) => {
                            //write error
                            log::error!("{}-{} : {:?}", file!(), line!(), e);
                            if e.kind() == std::io::ErrorKind::BrokenPipe {
                                thread_alive_clone.store(false, Ordering::Relaxed);
                                continue;
                            }
                        }
                    }
                }

                //check if there is anything to read
                match port.read(&mut read_buffer) {
                    Ok(n_bytes_read) => {
                        if let Some(msg) = concentrated_messsage_builder
                            .append_buffer(&read_buffer[0..n_bytes_read])
                        {
                            rx_counter_clone.fetch_add(1, Ordering::Relaxed);
                            let gaurd = swordfish_messages_hashmap_clone
                                .read()
                                .expect("we are only reading, this should work");
                            let bucket = gaurd
                                .get(&msg.opcode)
                                .expect("Opcode not found in hashmap, this should never happen");

                            //if message has an rx callback, do it
                            if let Some(rx_callback) = bucket
                                .on_rx_callback
                                .lock()
                                .expect("Another thread holding the mutex panicked")
                                .as_mut()
                            {
                                rx_callback(msg);
                            }

                            //if message category is bounce or param, place the message in the bucket and notify the waiting thread
                            match bucket.catagory {
                                SwordFishMessageCategory::Bounce | SwordFishMessageCategory::Param => {
                                    let mut bucket_msg = bucket
                                        .message
                                        .lock()
                                        .expect("Another thread holding the mutex panicked");
                                    *bucket_msg = Some(msg);
                                    bucket.condvar.notify_one();
                                }
                                //if message category is operation, place the message in the response bucket and notify the waiting thread
                                SwordFishMessageCategory::Operation(Some(response_opcode)) => {
                                    let gaurd = swordfish_messages_hashmap_clone
                                        .read()
                                        .expect("we are only reading, this should work");
                                    let response_bucket = gaurd.get(&response_opcode).expect(
                                        "Opcode not found in hashmap, this should never happen",
                                    );
                                    let mut bucket_msg = response_bucket
                                        .message
                                        .lock()
                                        .expect("Another thread holding the mutex panicked");
                                    *bucket_msg = Some(msg);
                                    response_bucket.condvar.notify_one();
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::TimedOut {
                            continue;
                        } else if e.kind() == std::io::ErrorKind::BrokenPipe {
                            thread_alive_clone.store(false, Ordering::Relaxed);
                        } else {
                            log::error!("{}-{} : {:?}", file!(), line!(), e);
                        }
                    }
                }
            }
        });

        return Ok(SwordFishComm {
            thread_handle: Some(thread_handle),
            thread_alive: thread_alive,
            transmitter: master_transmitter,
            tx_counter: tx_counter,
            rx_counter: rx_counter,
            messages_hashmap: swordfish_messages_hashmap,
        });
    }

    pub fn get_tx_counter(&self) -> usize {
        self.tx_counter.load(Ordering::SeqCst)
    }

    pub fn get_rx_counter(&self) -> usize {
        self.rx_counter.load(Ordering::SeqCst)
    }

    pub fn send_msg(&self, msg: SwordFishConcentratedMessage) -> Option<SwordFishConcentratedMessage> {
        let gaurd = self
            .messages_hashmap
            .read()
            .expect("we are only reading, this should work");
        let bucket = gaurd
            .get(&msg.opcode)
            .expect("Opcode not found in hashmap, this should never happen");

        self.transmitter.send(msg).expect("Failed to send message");

        match bucket.catagory {
            SwordFishMessageCategory::Bounce | SwordFishMessageCategory::Param => {
                if let Ok(mut optional_response_msg) = bucket.condvar.wait_timeout(
                    bucket
                        .message
                        .lock()
                        .expect("Another thread holding the mutex panicked"),
                    Duration::from_millis(200),
                ) {
                    if let Some(response_msg) = optional_response_msg.0.take() {
                        return Some(response_msg);
                    } else {
                        println!("No response message");
                    }
                }
                return None;
            }
            SwordFishMessageCategory::Operation(Some(response_opcode)) => {
                let gaurd = self
                    .messages_hashmap
                    .read()
                    .expect("we are only reading, this should work");
                let response_bucket = gaurd
                    .get(&response_opcode)
                    .expect("Opcode not found in hashmap, this should never happen");
                if let Ok(mut optional_response_msg) = bucket.condvar.wait_timeout(
                    response_bucket
                        .message
                        .lock()
                        .expect("Another thread holding the mutex panicked"),
                    Duration::from_millis(200),
                ) {
                    if let Some(response_msg) = optional_response_msg.0.take() {
                        return Some(response_msg);
                    }
                }
                return None;
            }
            _ => return None,
        }
    }

    pub fn change_message_rx_callback(
        &self,
        opcode: u8,
        new_rx_callback: Box<dyn FnMut(SwordFishConcentratedMessage) + Send>,
    ) {
        let mut gaurd = self
            .messages_hashmap
            .write()
            .expect("we are the only writers, this should work");
        let bucket = gaurd
            .get_mut(&opcode)
            .expect("Opcode not found in hashmap, this should never happen");
        bucket.on_rx_callback = Mutex::new(Some(new_rx_callback));
    }
}

impl Drop for SwordFishComm {
    fn drop(&mut self) {
        self.thread_alive.store(false, Ordering::Relaxed);
        //wait for the thread to finish
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .expect("The thread that handles reads could not be joined");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_swordfish_comm() {
        let swordfish_port: String = find_probable_swordfish_port().expect("No swordfish port found");
        let swordfish_comm = SwordFishComm::new(&swordfish_port.as_str())
            .expect(&format!("failed to connect to {}", swordfish_port));
        drop(swordfish_comm);
    }
}
