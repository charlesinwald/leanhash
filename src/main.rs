extern crate config;

use structopt::StructOpt;
use std::collections::HashMap;
use config::Value;
use std::string::ToString;
use std::net::UdpSocket;
use std::str;

use std::sync::{Mutex, RwLock, Arc};

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher, Hash};
use std::borrow::{BorrowMut, Borrow};
use std::convert::TryInto;

#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(default_value = "foobar", long)]
    operation: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet<'a> {
    operation: bool,
    //get = false, put = true
    is_int: bool,
    key: i32,
    val: &'a [u8],
}

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
    let args = Cli::from_args();
    println!("operation {}", &args.operation);

    //  Load Config
    let mut settings = config::Config::default();
    settings
        // Add in `./iplist`
        .merge(config::File::with_name("Settings")).unwrap();
    // Print out our settings (as a HashMap)
    let settings_map = settings.try_into::<HashMap<String, Vec<Value>>>().expect("Error reading iplist");
    let config_map = settings_map;
//    println!("{:?}",
//             config_map);
    let ip_array = config_map.get_key_value("ips").expect("Error reading list of node ips");
    println!("{:#?}", ip_array.1);

    {
        let socket = UdpSocket::bind("0.0.0.0:34254").expect("Error creating socket on 127.0.0.1:34254");

        // Thread Safe version
        let mut cc = Arc::new(RwLock::new(HashMap::new()));
//        let mut cc: HashMap<i32, Val> = HashMap::new();

        loop {
            //Make a buffer, and receive UDP data over the socket
            let mut buf = [0; 256];
            let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                .expect("Didn't receive data");
            //Remove any excess unused bytes

            let filled_buf = &mut buf[..number_of_bytes];

            let mut rec_packet: Packet = bincode::deserialize(filled_buf).expect("Malformed Packet, unable to deserialize");

            //Put request
            if rec_packet.operation == false {
                loop {
                    let key = rec_packet.key.clone();
                    let value;
//                if !rec_packet.is_int {
//                    value = Val::String(str::from_utf8(rec_packet.val).unwrap());
//                } else {
                    value = Val::Integer(i32::from_ne_bytes(rec_packet.val.try_into().expect("slice with incorrect length")));
                    let map = cc.read().expect("RwLock poisoned");
                    //Key exists
                    if let Some(element) = map.get(&key) {
                        println!("Exists");
                        drop(map); //Let go of lock
                        //Send "False", as a byte
                        socket.send_to(&[0], src_addr);
                        break;
                    }
                    //Key doesn't exist
                    else {
                        println!("Doesn't exist");
                        //Drop read lock...
                        drop(map);
                        //...in favor of a write lock
                        let mut map = cc.write().expect("RwLock poisoned");
                        map.insert(key, Mutex::new(value));
                        //Send "True" as a byte
                        socket.send_to(&[1], src_addr);
                    }
                }
                //let result = map.insert(key, value);
//                match result {
//                    Some(x) => {
//
//                    }
//                    None => {  }
//                }
//                for v in cc.values() {
//                    println!("{:?}", v);
//                }
            }
            //Get request
            else {
                let key: i32 = rec_packet.key.clone();
                let map = cc.read().expect("RwLock poisoned");
                let value = map.get(&key);
                match value {
                    Some(x) => {
                        let packet = bincode::serialize(x).expect("invalid value");
                        socket.send_to(&packet, src_addr);
                    }
                    None => {
                        println!("Value not found");
                        socket.send_to(&[0], src_addr);
                    }
                }
            }
        }
    }
}

