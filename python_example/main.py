'''
pip install the swordfish_com whl file to import the swordfish_com module
'''

import swordfish_com

if __name__ == "__main__":
    print(dir(swordfish_com))

    print(swordfish_com.get_serial_ports())

    print(swordfish_com.find_probable_swordfish_port())

    counter = 0
    opcode = 2
    payload = [1,2,3,5,6]
    tmp = swordfish_com.SwordFishConcentratedMessage(counter, opcode, payload)
    tmp.print()
    
    ping_msg = swordfish_com.PingMessage()
    ping_msg.print()
    
    ping_concentrated = ping_msg.to_concentrated(0)
    ping_unconcentrated = swordfish_com.PingMessage.from_concentrated(ping_concentrated)
    ping_unconcentrated.print()

    comm = swordfish_com.SwordFishComm(swordfish_com.find_probable_swordfish_port())
    print(f"Tx counter: {comm.get_tx_counter()}")
    print(f"Rx counter: {comm.get_rx_counter()}")
    answer = comm.send_msg(ping_concentrated)
    print(f"Tx counter: {comm.get_tx_counter()}")
    print(f"Rx counter: {comm.get_rx_counter()}")
    answer.print()

