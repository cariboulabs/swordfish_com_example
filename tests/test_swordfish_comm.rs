use swordfish_com::swordfish_comm::{find_probable_swordfish_port, SwordFishComm};
use swordfish_com::swordfish_messages::VersionData;
use swordfish_com::{SwordFishConcentratedMessage, SwordFishMessageTrait};
use std::sync::{Arc,RwLock};

#[cfg(not(feature = "all_wrappers"))]
use simple_logger;

#[test]
fn with_swordfish_opcode2() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    //open comm
    let swordfish_port = find_probable_swordfish_port().expect("Failed to find probable SwordFish port");

    let swordfish_comm =
        SwordFishComm::new(swordfish_port.as_str()).expect("Safe to unwrap because the port was found");

    let rx_vec: Vec<u64> =vec![0];
    let arc_rx_vec = Arc::new(RwLock::new(rx_vec));
    let arc_rx_vec_clone = arc_rx_vec.clone();

    let new_rx_callback = move |msg: SwordFishConcentratedMessage| {
        if let Ok(msg) = VersionData::from_concentrated(&msg) {
            let last_value = arc_rx_vec_clone.read().unwrap().last().unwrap().clone();
            let mut arc_rx_vec_clone = arc_rx_vec_clone.write().unwrap();
            arc_rx_vec_clone.push(last_value + 1);

            println!("Received version data: {:?}, and rx_vec len is {}", msg, arc_rx_vec_clone.len());
        }
    };
    swordfish_comm.change_message_rx_callback(VersionData::OPCODE, Box::new(new_rx_callback));

    let time0 = std::time::Instant::now();
    let mut rx_counter = 0;
    while rx_counter < 10 {
        //send request for version data
        let request_concentrated_msg =
            VersionData::default().to_concentrated(swordfish_comm.get_tx_counter() as u16);

        let answer = swordfish_comm.send_msg(request_concentrated_msg);
        println!("sent the {} message", swordfish_comm.get_tx_counter());
        match answer {
            Some(answer) => {
                if let Ok(_) = VersionData::from_concentrated(&answer) {
                    rx_counter += 1;
                }
            }
            None => (),
        }

        if std::time::Instant::now().duration_since(time0).as_secs() > 5 {
            //0.5 seconds for a msg is enough
            break;
        }
    }
    println!(
        "sent {} messages, and received {} messages",
        swordfish_comm.get_tx_counter(),
        rx_counter
    );
    println!("rx_vec: {:?}", arc_rx_vec.read().unwrap());
    assert_eq!(rx_counter, 10);
}
