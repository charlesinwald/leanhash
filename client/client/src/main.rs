use std::net::UdpSocket;
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
operation: bool, //get = false, put = true
key: i32,
val_len: i32,
val: &'a [u8],
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() -> std::io::Result<()> {
    {
        //Retrieve list of ips
        let ip_list_string = fs::read_to_string("iplist").expect("Could not read in node ip list");
        let ip_list_vec = ip_list_string.split("\n").collect::<Vec<&str>>();
        let num_nodes : u64 = ip_list_vec.len() as u64;
        if num_nodes < 1 {
            eprintln!("Error: No nodes found in iplist");
            std::process::exit(1);
        }
        println!("{:#?} {}", ip_list_vec, num_nodes);

        //Entry to be sent
        let key = 5;
        let value = "Value to be stored";
        //Calculate hash value to figure out where the value should go
        let hash = calculate_hash(&key);
        let dest_node = hash % num_nodes;
        println!("Hash: {}", hash);
        println!("Destination Node: {}", dest_node);




        let mut socket = UdpSocket::bind("127.0.0.1:34255")?;


        let packet = Packet{ operation: false, key, val_len: mem::size_of_val(value) as i32, val: value.as_bytes()};
        let bytes_to_send = serialize(&packet).unwrap();
        socket.send_to(&bytes_to_send, "127.0.0.1:34254")?;
    } // the socket is closed here
    Ok(())
}