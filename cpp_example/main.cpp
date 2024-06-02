#include "SwordFishPortFinder.hpp"
#include "c_SwordFishConcentratedMessage.h"
#include "SwordFishConcentratedMessage.hpp"
#include "rust_slice_tmpl.hpp"
#include "CRustSliceu8.h"
#include "PingMessage.hpp"
#include "VersionDataMessage.hpp"
#include "SwordFishComm.hpp"
#include <iostream>
#include <string_view>

int main() {
    //print swordfish ports
    std::string serial_ports = swordfish_com::SwordFishPortFinder::get_serial_ports().to_std_string();
    std::cout << "Found ports:" << std::endl;
    std::cout << serial_ports << std::endl;
    std::string probable_swordfish_port = swordfish_com::SwordFishPortFinder::find_probable_swordfish_port().to_std_string();
    std::cout << "Probable SwordFish port: " << probable_swordfish_port << std::endl;

    //create a swordfish concentrated message - c verison
    uint16_t counter = 1;
    uint8_t opcode = 2;
    uint8_t payload[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    CRustSliceu8 payload_rust_slice = {payload, 10};
    SwordFishConcentratedMessageOpaque *concentrated_message_c = SwordFishConcentratedMessage_new(counter, opcode, payload_rust_slice);
    SwordFishConcentratedMessage_print(concentrated_message_c);

    //create a swordfish concentrated message, c++ version
    //we need to move the payload to conform to rust borrow checker (I guess)
    swordfish_com::RustSlice<const uint8_t> payload_cpp = swordfish_com::RustSlice<const uint8_t>(payload, 10);
    swordfish_com::SwordFishConcentratedMessage concentrated_message_cpp = swordfish_com::SwordFishConcentratedMessage(counter, opcode, std::move(payload_cpp));

    //create a ping message
    swordfish_com::PingMessage ping_message = swordfish_com::PingMessage();
    ping_message.print();

    //create versiondata message
    swordfish_com::VersionDataMessage version_data_message = swordfish_com::VersionDataMessage::make_empty();
    version_data_message.print();
    version_data_message.set_version(1);
    version_data_message.set_subversion(2);
    version_data_message.set_mcu_type(3);
    uint8_t uuid[] = {1, 2, 3, 4, 5, 6, 7, 8};
    swordfish_com::RustSlice<const uint8_t> uuid_rust_slice = swordfish_com::RustSlice<const uint8_t>(uuid, 8);
    version_data_message.set_uuid(std::move(uuid_rust_slice));
    version_data_message.print();

    //create swordfish comm
    swordfish_com::SwordFishComm swordfish_comm = swordfish_com::SwordFishComm(probable_swordfish_port);
    std::cout << "Tx counter: " << swordfish_comm.get_tx_counter() << std::endl;
    std::cout << "Rx counter: " << swordfish_comm.get_rx_counter() << std::endl;
    swordfish_com::SwordFishConcentratedMessage concentrated_send = ping_message.to_concentrated(0);
    std::optional<swordfish_com::SwordFishConcentratedMessage> concentrated_answer = swordfish_comm.send_msg(std::move(concentrated_send));
    if (concentrated_answer.has_value()) {
        std::optional<swordfish_com::PingMessage> ping_answer = swordfish_com::PingMessage::from_concentrated(concentrated_answer.value());
        if (ping_answer.has_value()) {
            ping_answer.value().print();
        }
    }
    std::cout << "Rx counter: " << swordfish_comm.get_rx_counter() << std::endl;
    std::cout << "Tx counter: " << swordfish_comm.get_tx_counter() << std::endl;

    return 0;
}