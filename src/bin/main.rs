extern crate config;
extern crate disthashtable;

//use std::collections::HashMap;
use config::Value;
use std::net::{TcpListener, TcpStream};
use std::str;

use std::sync::{Mutex, RwLock, Arc};

//The thread pool I wrote, its in a different crate for modularity
//and saving time and RAM at compile time
use disthashtable::ThreadPool;
mod hash_map {
    pub mod hash_map;
}

#[macro_use]
extern crate serde_derive;
extern crate bincode;

//Used for settings only, not the one used in the RHT
use std::collections::HashMap;
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
    operation: i32,
    //put = 0, get = 1, multiput = 2
    is_int: bool,
    key: i32,
    val: &'a [u8],
}

//Protocol for sending information over socket
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MultiPacket<'a> {
    has_next: bool, //if this node has 3 keys to update
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
    // Thread Safe version
    let mut cc : Arc<RwLock<hash_map::hash_map::hash_map<i32, Val>>> = Arc::new(RwLock::new(hash_map::hash_map::hash_map::new()));

    let listener = TcpListener::bind("0.0.0.0:34254").unwrap();

    let pool = ThreadPool::new(6);


    for stream in listener.incoming() {
        match stream {
            Err(_) => println!("Listener error"),
            Ok(mut stream) => {
                //We want a valid reference to the hashmap
                let map_clone = cc.clone();
//                println!("{:?}", stream);
                let mut buf = [0; 10];
                pool.execute(move || {
                    handle_packet(&mut stream, map_clone);
                });
            }
        }
    }
}


fn handle_packet(stream: &mut TcpStream, cc: Arc<RwLock<hash_map::hash_map::hash_map<i32, Val>>>) -> () {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    let mut rec_packet: Packet = bincode::deserialize(&buffer).expect("Malformed Packet");
    //Put request
    if rec_packet.operation == 0 {
        loop {
            let key = rec_packet.key.clone();
            let value;
            // println!("{:#?}",rec_packet.key);
            value = Val::Integer(i32::from_ne_bytes(rec_packet.val.try_into().expect("slice with incorrect length")));
            let lock = cc.try_write();
             match lock {
                //We've acquired the lock successfully
                Ok(mut map) => {
                    //Send "True", as a byte
                    stream.write(&[1]).expect("Error writing True to the stream");
                    //Wait for coordinator to tell us decision
                    let mut decision = [0; 1];
                    stream.read(&mut decision);
                    //If answer is yes, commit the put
                    if decision[0] == 1 {
                        map.entry(key).or_insert(value);
                        stream.write(&[1]).expect("Error writing True to the stream");
                    }
                    break;
                },
                Err(e) => {
                    //Send false as a byte
                    stream.write(&[0]).expect("Error writing False to the stream");
                    continue;
                }
            };
        }
    }
    //Multiput request
    else if rec_packet.operation == 2 {
        loop {
            //Get Second Key/Value pair as multipacket
            let mut buffer2 = [0; 512];
            stream.read(&mut buffer2).unwrap();
            let key = rec_packet.key.clone();
            let mut rec_packet2: MultiPacket = bincode::deserialize(&buffer2).expect("Malformed Packet");
            let key2 = rec_packet2.key.clone();
            // println!("{:#?}",rec_packet.key);
            let value = Val::Integer(i32::from_ne_bytes(rec_packet.val.try_into().expect("slice with incorrect length")));
            let value2 = Val::Integer(i32::from_ne_bytes(rec_packet2.val.try_into().expect("slice with incorrect length")));

            //Attempt to get a write lock on the value, blocking current thread until that happens
            let lock = cc.try_write();
            match lock {
                //We've acquired the lock successfully
                Ok(mut map) => {
                    //Send "True", as a byte
                    stream.write(&[1]).expect("Error writing True to the stream");
                    //Wait for coordinator to tell us decision
                    let mut decision = [0; 1];
                    stream.read(&mut decision);
                    //If answer is yes, commit the put
                    if decision[0] == 1 {
                        map.entry(key).or_insert(value);
                        map.entry(key2).or_insert(value2);
                        if rec_packet2.has_next {
                            //Get Third Key/Value pair as multipacket
                            let mut buffer3 = [0; 512];
                            stream.read(&mut buffer3).unwrap();
                            let mut rec_packet3: MultiPacket = bincode::deserialize(&buffer3).expect("Malformed Packet");
                            let key3 = rec_packet3.key.clone();
                            let value3 = Val::Integer(i32::from_ne_bytes(rec_packet3.val.try_into().expect("slice with incorrect length")));
                            map.entry(key3).or_insert(value3);
                        }
                        stream.write(&[1]).expect("Error writing True to the stream");
                    }
                    break;
                },
                Err(e) => {
                    //Send false as a byte
                    stream.write(&[0]).expect("Error writing False to the stream");
                    continue;
                }
            };
        }
    }

        //Get request
    else {
        let key: i32 = rec_packet.key.clone();
        let map = cc.read().expect("RwLock poisoned");
        let value = map.get(&key);
        match value {
            Some(x) => {
                let packet = bincode::serialize(x).expect("invalid value");
                stream.write(&packet).expect("Error writing packet to the stream");
            }
            None => {
                stream.write(&[0]).expect("Error writing False (get) to the stream");
            }
        }
    }
}


