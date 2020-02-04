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

//TODO have enum so you can specify the type of data being saved in val, instead of assuming string

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet<'a> {
    operation: bool,
    //get = false, put = true
    key: i32,
    val_len: i32,
    val: &'a [u8],
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn send_packet(address: &str, packet: &Packet) {
//    let mut addr_mut = address.to_owned();
//    let port : &str = ":34254";
//    let dest_addr
//    let dest_addr: String = format!("{}{}", address, port);
    println!("Destination Address {:#?}", address);
    println!("{:#?}", packet);
    let mut socket = UdpSocket::bind("0.0.0.0:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
    socket.connect(address).expect("Error connecting");
//  let mut socket_addr_iter : Vec<_> = address.to_socket_addrs().expect("Unable to parse IP address").collect();
//    let mut socket_addr = *socket_addr_iter.get(0).expect("Unable to parse IP address");
//    socket_addr.set_port(34254);
//    let socket_addr : SocketAddr = address.parse().unwrap();
//    println!("{:#?}", socket_addr);
//    socket.send_to(&bytes_to_send, &socket_addr).expect("Failed to send packet");
//    let result = socket.send_to(&bytes_to_send, address).expect("Failed to send packet");
//    println!("{:#?}", result);
    socket.send(&bytes_to_send).expect("couldn't send message");
}

fn self_send_packet(packet: &Packet) {
    println!("{:#?}", packet);
    let mut socket = UdpSocket::bind("127.0.0.1:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
    let result = socket.send_to(&bytes_to_send, "0.0.0.0:34254").expect("Failed to send packet");
}


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
        let value = "Value to be stored";
        //Fill in packet
        let packet = Packet { operation: false, key, val_len: mem::size_of_val(value) as i32, val: value.as_bytes() };

        //Calculate hash value to figure out where the value should go
        let hash = calculate_hash(&key);
        let dest_node = hash % num_nodes;
        println!("Hash: {}", hash);
        println!("Destination Node: {}", dest_node);
        if dest_node == 0 {
            self_send_packet(&packet);
        } else {
            let dest_ip = ip_list_vec[(dest_node + 1) as usize];
            send_packet(&dest_ip, &packet);
        }
    } // the socket is closed here
    Ok(())
}