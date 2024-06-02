use super::super::*;
use pyo3::prelude::*;
use swordfish_concentrated_message::SwordFishConcentratedMessage as RustSwordFishConcentratedMessage;

#[pyfunction]
fn get_serial_ports() -> String {
    let result = match swordfish_comm::get_serial_ports() {
        Some(ports) => ports,
        None => "".to_string(),
    };
    result
}
#[pyfunction]
fn find_probable_swordfish_port() -> String {
    let result = match swordfish_comm::find_probable_swordfish_port() {
        Some(port_name) => port_name.to_string(),
        None => "".to_string(),
    };
    result
}

#[pyclass]
pub struct SwordFishConcentratedMessage(RustSwordFishConcentratedMessage);
#[pymethods]
impl SwordFishConcentratedMessage {
    #[new]
    fn new(counter: u16, opcode: u8, payload: Vec<u8>) -> Self {
        SwordFishConcentratedMessage(RustSwordFishConcentratedMessage::new(
            counter, opcode, &payload,
        ))
    }
    fn print(&self) {
        println!("{:?}", self.0);
    }
}

use swordfish_messages::Ping as RustPingMessage;
#[pyclass]
pub struct PingMessage(RustPingMessage);
#[pymethods]
impl PingMessage {
    #[new]
    fn new() -> Self {
        PingMessage(RustPingMessage::default())
    }
    fn to_concentrated(&self, counter : u16) -> SwordFishConcentratedMessage {
        let tmp : RustSwordFishConcentratedMessage = self.0.to_concentrated(counter);
        SwordFishConcentratedMessage(tmp)
    }
    #[staticmethod]
    fn from_concentrated(msg: &SwordFishConcentratedMessage) -> Option<PingMessage> {
        match RustPingMessage::from_concentrated(&msg.0) {
            Ok(ping_message) => Some(PingMessage(ping_message)),
            Err(_) => None,
        }
    }
    fn print(&self) {
        self.0.print();
    }
}

use swordfish_comm::SwordFishComm as RustSwordFishComm;
#[pyclass]
pub struct SwordFishComm(RustSwordFishComm);
#[pymethods]
impl SwordFishComm {
    #[new]
    fn new(port_name: &str) -> Self {
        SwordFishComm(RustSwordFishComm::new(port_name).expect("Failed to create SwordFishComm"))
    }
    fn send_msg(&self, msg: &SwordFishConcentratedMessage) -> Option<SwordFishConcentratedMessage> {
        match self.0.send_msg(msg.0) {
            Some(msg) => Some(SwordFishConcentratedMessage(msg)),
            None => None,
        }
    }
    fn get_tx_counter(&self) -> usize {
        self.0.get_tx_counter()
    }
    fn get_rx_counter(&self) -> usize {
        self.0.get_rx_counter()
    }
}

#[pymodule]
fn swordfish_com(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_serial_ports, m)?)?;
    m.add_function(wrap_pyfunction!(find_probable_swordfish_port, m)?)?;
    m.add_class::<SwordFishComm>()?;
    m.add_class::<SwordFishConcentratedMessage>()?;
    m.add_class::<PingMessage>()?;
    Ok(())
}
