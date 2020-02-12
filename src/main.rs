extern crate config;

use structopt::StructOpt;
use std::collections::HashMap;
use config::Value;
use std::string::ToString;
use std::net::UdpSocket;
use std::str;

use std::sync::{Mutex, RwLock, Arc, mpsc};

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

//#[derive(StructOpt)]
//struct Cli {
//    /// The pattern to look for
//    #[structopt(default_value = "foobar", long)]
//    operation: String,
//}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet<'a> {
    operation: bool,
    //put = false, get = true
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
//    let args = Cli::from_args();
//    println!("operation {}", &args.operation);

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


//    let mut joins = Vec::new();
//    let (sender, receiver): (Sender<Packet>, Receiver<Packet>) = mpsc::channel();


    // Thread Safe version
    let mut cc: Arc<RwLock<HashMap<i32, Mutex<Val>>>> = Arc::new(RwLock::new(HashMap::new()));
//        let mut cc: HashMap<i32, Val> = HashMap::new();

    let socket = UdpSocket::bind("0.0.0.0:34254").expect("Error creating socket on 127.0.0.1:34254");
    let mut buf = [0; 256];

    loop {
        let sock = socket.try_clone().expect("Failed to clone socket");
        //creates another pointer to the hash map and increases the atomic reference counter
        let map_clone = cc.clone();
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                thread::spawn(move || {
//                    println!("Handling connection from {}, {} bytes", src, amt);
                    let filled_buf = &mut buf[..amt];
                    let mut rec_packet: Packet = bincode::deserialize(&filled_buf).expect("Malformed Packet, unable to deserialize");
//                    let map = map_clone.read().expect("RwLock poisoned");
                    handlePacket(&sock, map_clone, src, rec_packet);
                });
            }
            Err(e) => {
                eprintln!("Couldn't recieve a datagram: {}", e);
            }
        }


//        let mut buf = [0; 256];
//        //Make a buffer, and receive UDP data over the socket
//        let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
//            .expect("Didn't receive data");
//        //Remove any excess unused bytes
////        let filled_buf = &mut buf[..number_of_bytes];
//        let mut filled_buf = [0;256];
//        filled_buf[..number_of_bytes].clone_from_slice(&buf);
//        let mut rec_packet: Packet = bincode::deserialize(&filled_buf).expect("Malformed Packet, unable to deserialize");
//        thread::spawn(move || {
//            handlePacket(&socket, &mut cc, src_addr, rec_packet);
//        });
    }
}

fn handlePacket(socket: &UdpSocket, cc: Arc<RwLock<HashMap<i32, Mutex<Val>, RandomState>>>, src_addr: SocketAddr, mut rec_packet: Packet) -> () {
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
//                println!("Exists");
                drop(map); //Let go of lock
                //Send "False", as a byte
                socket.send_to(&[0], src_addr);
                break;
            }
            //Key doesn't exist
            else {
//                println!("Doesn't exist");
                //Drop read lock...
                drop(map);
                //...in favor of a write lock
                let mut map = cc.write().expect("RwLock poisoned");
                map.insert(key, Mutex::new(value));
                //Send "True" as a byte
                socket.send_to(&[1], src_addr);
                break;
            }
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
                socket.send_to(&packet, src_addr);
            }
            None => {
//                println!("Value not found");
                socket.send_to(&[0], src_addr);
            }
        }
    }
}


