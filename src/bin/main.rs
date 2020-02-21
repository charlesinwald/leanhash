extern crate config;
extern crate disthashtable;

use structopt::StructOpt;
use std::collections::HashMap;
use config::Value;
use std::string::ToString;
use std::net::{UdpSocket, TcpListener, TcpStream};
use std::str;

use std::sync::{Mutex, RwLock, Arc, mpsc};

//The thread pool I wrote, its in a different crate for modularity
//and saving time and RAM at compile time
use disthashtable::ThreadPool;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use std::{thread, time::Duration};
use std::collections::hash_map::RandomState;
use std::net::SocketAddr;
use bincode::{serialize, deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::borrow::{BorrowMut, Borrow};
use std::convert::TryInto;
use std::sync::mpsc::{Sender, Receiver, SyncSender};
use std::io::{Read, Write};


//Protocol for sending information over socket
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet<'a> {
    operation: bool,
    //put = false, get = true
    is_int: bool,
    key: i32,
    val: &'a [u8],
}

//To make the hashmap generic
#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Clone, Copy)]
enum Val<'a> {
    String(&'a str),
    Integer(i32),
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {

    //  Load Config
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings")).unwrap();
    let settings_map = settings.try_into::<HashMap<String, Vec<Value>>>().expect("Error reading iplist");
    let config_map = settings_map;
    let ip_array = config_map.get_key_value("ips").expect("Error reading list of node ips");


    // Thread Safe version
    let mut cc: Arc<RwLock<HashMap<i32, Mutex<Val>>>> = Arc::new(RwLock::new(HashMap::new()));

    let listener = TcpListener::bind("127.0.0.1:34254").unwrap();

    let pool = ThreadPool::new(6);

    for stream in listener.incoming() {
        //We want a valid reference to the hashmap
        let map_clone = cc.clone();
        let stream = stream.unwrap();
        pool.execute(move || {
            handle_packet(stream, map_clone);
        })
    }
//    loop {
//        let sock = socket.try_clone().expect("Failed to clone socket");
//        //creates another pointer to the hash map and increases the atomic reference counter
//        let map_clone = cc.clone();
//        match socket.recv_from(&mut buf) {
//            Ok((amt, src)) => {
//                pool.execute(move || {
//                    let filled_buf = &mut buf[..amt];
//                    let mut rec_packet: Packet = bincode::deserialize(&filled_buf).expect("Malformed Packet, unable to deserialize");
//                    handlePacket(&sock, map_clone, src, rec_packet);
//                });
//            }
//            Err(e) => {
//                eprintln!("Couldn't recieve a datagram: {}", e);
//            }
//        }
//    }
}

fn handle_packet(mut stream: TcpStream, cc: Arc<RwLock<HashMap<i32, Mutex<Val>, RandomState>>>) -> () {
    println!("{:#?}", stream.peer_addr().unwrap());
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let mut rec_packet: Packet = bincode::deserialize(&buffer).expect("Malformed Packet");

    //Put request
    if rec_packet.operation == false {
        loop {
            let key = rec_packet.key.clone();
            let value;
            value = Val::Integer(i32::from_ne_bytes(rec_packet.val.try_into().expect("slice with incorrect length")));
            let map = cc.read().expect("RwLock poisoned");
            //Key exists
            if let Some(element) = map.get(&key) {
                drop(map); //Let go of lock
                //Send "False", as a byte
                stream.write(&[0]);
                break;
            }
            //Key doesn't exist
            else {
                //Drop read lock...
                drop(map);
                //...in favor of a write lock
                let mut map = cc.write().expect("RwLock poisoned");
                map.insert(key, Mutex::new(value));
                //Send "True" as a byte
                stream.write(&[1]);
                break;
            }
        }
    }
//    //Get request
    else {
        let key: i32 = rec_packet.key.clone();
        let map = cc.read().expect("RwLock poisoned");
        let value = map.get(&key);
        match value {
            Some(x) => {
                let packet = bincode::serialize(x).expect("invalid value");
                stream.write(&packet);
            }
            None => {
                stream.write(&[0]);
            }
        }
    }
    stream.flush().unwrap();
}


