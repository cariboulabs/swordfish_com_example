import com.swordfish.SwordFishPortFinder;
import com.swordfish.SwordFishConcentratedMessage;
import com.swordfish.PingMessage;
import com.swordfish.VersionDataMessage;
import com.swordfish.SwordFishComm;

public class Main {
    static {
        String currentDir = System.getProperty("user.dir");
        System.load(currentDir + "/com/swordfish/target/libswordfish_com.so"); // Load the library with the absolute path
    }

    public static void main(String[] args) {
        //print swordfish ports
        String serial_ports = SwordFishPortFinder.get_serial_ports();
        System.out.println("Serial ports: " + serial_ports);
        String probable_swordfish_port = SwordFishPortFinder.find_probable_swordfish_port();
        System.out.println("Probable SwordFish port: " + probable_swordfish_port);
        

        //create a swordfish concentrated message
        SwordFishConcentratedMessage msg = new SwordFishConcentratedMessage(1, (short) 2, new byte[] { 1, 2, 3 });
        msg.print();

        //create a ping message
        PingMessage ping_message = new PingMessage();
        ping_message.print();

        //create version data message
        VersionDataMessage version_data_message = VersionDataMessage.make_empty();
        version_data_message.print();
        version_data_message.set_mcu_type(1);
        version_data_message.set_subversion((short)2);
        version_data_message.set_mcu_type(3);
        byte[] uuid = new byte[] {1, 2, 3, 4, 5, 6, 7, 8};
        version_data_message.set_uuid(uuid);
        version_data_message.print();

        //create a swordfish comm object
        SwordFishComm swordfish_comm = new SwordFishComm(probable_swordfish_port);
        System.out.println("Tx counter: " + swordfish_comm.get_tx_counter());
        System.out.println("Rx counter: " + swordfish_comm.get_rx_counter());
        java.util.Optional<SwordFishConcentratedMessage> concentrated_answer = swordfish_comm.send_msg(ping_message.to_concentrated(0));
        if (concentrated_answer.isPresent()) {
            concentrated_answer.get().print();
        }
        System.out.println("Tx counter: " + swordfish_comm.get_tx_counter());
        System.out.println("Rx counter: " + swordfish_comm.get_rx_counter());        
    }
}
