use crc32fast::Hasher;
use pnet::packet::ethernet::EthernetPacket;
use pnet::packet::Packet;
use pnet_datalink::Channel;

fn main() {
    // List interfaces, iterate over them, and find the one
    // named "en4"
    let interface = pnet_datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == "en4")
        .unwrap();
    
    // Define the three components of an Ethernet frame header:
    // 1) Destination MAC address
    // 2) Source MAC address (which we get from the interface)
    // 3) EtherType 
    let dest_mac = [0x14, 0xb3, 0x1f, 0x23, 0x8c, 0xc6];
    let src_mac: [u8; 6] = interface.mac.unwrap().into();
    let ether_type: [u8; 2] = [0x88, 0xb5];

    // Define a payload
    let payload: &[u8] = &b"cat"[..];

    // Create an empty buffer of 1518 bytes
    // We'll fill it in the expected order:
    //  1) Destination MAC 
    //  2) Source MAC
    //  3) EtherType
    //  4) Payload
    //  5) CRC (which will be computed from the first 4 elements)
    let mut ethernet_buffer = Vec::with_capacity(1518);
        ethernet_buffer.extend_from_slice(&dest_mac);
        ethernet_buffer.extend_from_slice(&src_mac);
        ethernet_buffer.extend_from_slice(&ether_type);


        println!("\nEthernet header: {:x?}", ethernet_buffer);
        println!("\t\t |---------DST----------||--------SRC---------||--ET--|\n");
        ethernet_buffer.extend_from_slice(&payload);

    // Add padding if needed (when data length < 46)
    if payload.len() < 46 {
        ethernet_buffer.extend_from_slice(&[0u8; 46][..(46 - payload.len())]);
    }

    // No jumbo bois
    if payload.len() > 1500 {
        panic!("This is too big!")
    }

    // Create a CRC hasher, hash the current frame, and append the bytes 
    let mut hasher = Hasher::new();                                         // New hasher
    hasher.update(&ethernet_buffer);                                        // Hash current frame
    let crc = hasher.finalize();                                            // Finish up
    ethernet_buffer.extend_from_slice(&crc.to_le_bytes());                  // Add hash to frame Little-endian bytes, specifically.
    let ethernet_frame = EthernetPacket::new(&ethernet_buffer).unwrap();    // Map existing raw bytes to an Ethernet frame data structure
                                                                            // Aside: ethernet "packet" is not the correct terminology. 'Tis a frame.

    // Create a transmitting and recieving channel (the reciever is not used)
    let (mut tx, mut _rx) = match pnet_datalink::channel(&interface, Default::default()) {
        Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };

    // Transmit the frame
    tx.send_to(&ethernet_frame.packet(), None).unwrap().unwrap();
}
