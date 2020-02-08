use std::net::{UdpSocket, ToSocketAddrs, SocketAddr, Ipv4Addr, IpAddr};
use std::mem;
use std::env;
use std::fs;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::convert::TryInto;
use std::ptr::null;

//TODO have enum so you can specify the type of data being saved in val, instead of assuming string

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Packet<'a> {
    operation: bool,
    //get = false, put = true
    is_int: bool,
    key: i32,
    val: &'a [u8],
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn send_put_packet(address: &str, packet: &Packet) {
    println!("Destination Address {:#?}", address);
    println!("Packet to Send: {:#?} \n", packet);
    let mut socket = UdpSocket::bind("0.0.0.0:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
    socket.connect(address).expect("Error connecting");
    socket.send(&bytes_to_send).expect("couldn't send message");
    let mut buf = [0; 256];
    let (amt, src) = socket.recv_from(&mut buf).expect("No response");
    let filled_buf = &mut buf[..amt];
//    println!("{:#?}",filled_buf);
    if filled_buf == [0] {
        println!("false");
    }
    else {
        println!("true");
    }
}

fn send_get_packet(address: &str, packet: &Packet) {
    println!("Destination Address {:#?}", address);
    println!("Packet to Send: {:#?} \n", packet);
    let mut socket = UdpSocket::bind("0.0.0.0:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
    socket.connect(address).expect("Error connecting");
    socket.send(&bytes_to_send).expect("couldn't send message");
    let mut buf = [0; 256];
    let (amt, src) = socket.recv_from(&mut buf).expect("No response");
    let filled_buf = &mut buf[..amt];
//    println!("{:#?}",filled_buf);
    if filled_buf == [0] {
        println!("null");
    }
    else {
        println!("{:#?}", filled_buf);
    }
}
//fn self_send_packet(packet: &Packet) {
//    println!("{:#?}", packet);
//    let mut socket = UdpSocket::bind("127.0.0.1:34255").expect("Failed to bind to UDP socket.");
//    let bytes_to_send = serialize(&packet).unwrap();
//    let result = socket.send_to(&bytes_to_send, "0.0.0.0:34254").expect("Failed to send packet");
//}


fn main() -> std::io::Result<()> {
    {
        //Retrieve list of ips
        let ip_list_string = fs::read_to_string("iplist").expect("Could not read in node ip list");
        let ip_list_vec = ip_list_string.split("\n").collect::<Vec<&str>>();
        let num_nodes: u64 = ip_list_vec.len() as u64 - 1;
        if num_nodes < 1 {
            eprintln!("Error: Less than two nodes found in iplist, format should be this node's AWS Elastic IP, and the elastic node ips [1..n]");
            std::process::exit(1);
        }
        let this_ip = ip_list_vec[0];
        println!("{:#?} {}", ip_list_vec, num_nodes);
        println!("This Node's IP:{}",this_ip);
        //Entry to be sent
        let key = 42;
        let value = 5_i32;
        //Fill in packet
        let packet = Packet { operation: false, key, is_int: true, val: &value.to_ne_bytes() };

        //Calculate hash value to figure out where the value should go
//        let hash = calculate_hash(&key);
//        let dest_node = hash % num_nodes;
//        println!("Hash: {}", hash);
//        println!("Destination Node: {}", dest_node);
        let mut dest_ip = ip_list_vec[((calculate_hash(&key) % num_nodes) + 1) as usize];
        send_put_packet(&dest_ip, &packet);


        let packet = Packet { operation: true, key, is_int: true, val: &[0] };
        dest_ip = ip_list_vec[((calculate_hash(&key) % num_nodes) + 1) as usize];
        send_get_packet(&dest_ip, &packet);


    } // the socket is closed here
    Ok(())
}