use std::net::{UdpSocket, ToSocketAddrs, SocketAddr, Ipv4Addr, IpAddr, TcpStream};
use std::{mem, thread, time};
use std::env;
use std::fs;



#[macro_use]
//For serializing/deserializing binary
extern crate serde_derive;
extern crate bincode;

//For random numbers
//Parsing command line arguments
use structopt::StructOpt;
//For random number generation since I don't have enough AWS RAM to compile the standard rust
//random number library...
use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{serialize, deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
//use std::convert::TryInto;
use std::ptr::null;
use std::time::{Duration, Instant};

#[derive(StructOpt)]
struct Cli {
    /// Number of operations
    #[structopt(default_value = "1000", short, long, help = "Pass `-h` and you'll see me!")]
    operations: f64,
    #[structopt(default_value = "1000", short, long, help = "Pass `-h` and you'll see me!")]
    max_key: u32,
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Packet<'a> {
    operation: bool,
    //put = false, get = true
    is_int: bool,
    key: i32,
    val: &'a [u8],
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Hash, Clone, Copy)]
enum Val<'a> {
    String(&'a str),
    Integer(i32),
}

fn send_put_packet(address: &str, packet: &Packet) -> i32 {
//    println!("Destination Address {:#?}", address);
//    println!("Packet to Send: {:#?} \n", packet);
    let mut socket = UdpSocket::bind("0.0.0.0:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
//    println!("{}", address);
    socket.connect(address).expect("Error connecting");
    socket.send(&bytes_to_send).expect("couldn't send message");
    let mut buf = [0; 256];
    let (amt, src) = socket.recv_from(&mut buf).expect("No response");
    let filled_buf = &mut buf[..amt];
//    println!("{:#?}",filled_buf);
    if filled_buf == [0] {
//        println!("false");
        return 0;
    } else {
//        println!("true");
        return 1;
    }
}

fn send_get_packet(address: &str, packet: &Packet) -> i32 {
//    println!("Destination Address {:#?}", address);
//    println!("Packet to Send: {:#?} \n", packet);
    let mut socket = UdpSocket::bind("0.0.0.0:34255").expect("Failed to bind to UDP socket.");
    let bytes_to_send = serialize(&packet).unwrap();
    socket.connect(address).expect("Error connecting");
    socket.send(&bytes_to_send).expect("couldn't send message");
    let mut buf = [0; 256];
    let (amt, src) = socket.recv_from(&mut buf).expect("No response");
    let filled_buf = &mut buf[..amt];
//    println!("{:#?}",filled_buf);
    if filled_buf == [0] {
//        println!("null");
        return 0;
    } else {
        let value: Val = bincode::deserialize(filled_buf).unwrap();
//        println!("{:#?}", value);
        return 1;
    }
}



fn main() -> std::io::Result<()> {
    {
        let args = Cli::from_args();
        let operations: f64 = args.operations;
        println!("Operations: {:#?}", operations);


        //Retrieve list of ips
        let ip_list_string = fs::read_to_string("iplist").expect("Could not read in node ip list");
        let ip_list_vec = ip_list_string.split("\n").collect::<Vec<&str>>();
        let tcp_vec : Vec<TcpStream> = vec![];
        let mut iter = ip_list_vec.iter().enumerate();
        iter.next(); //We want to skip the first one
        for (i, ip) in iter {
            println!("In position {} we have value {}", i, ip);
            if let Ok(stream) = TcpStream::connect(ip) {
                println!("Connected to {}", ip);
            } else {
                println!("Couldn't connect to {}", ip);
            }
        }
        let num_nodes: u64 = ip_list_vec.len() as u64 - 1;
        if num_nodes < 1 {
            eprintln!("Error: Less than two nodes found in iplist, format should be this node's AWS Elastic IP, and the elastic node ips [1..n]");
            std::process::exit(1);
        }
        let this_ip = ip_list_vec[0];
//        println!("{:#?} {}", ip_list_vec, num_nodes);
//        println!("This Node's IP:{}",this_ip);

        let max_key: u32 = args.max_key;
        //Workaround for not having a proper randomization function
        let mut random_num = get_random_key(max_key);
        println!("epoch {}", random_num);
        println!("nanos: {}", random_num % max_key);

        let mut putNum : i32 = (0.4 * operations).round() as i32;
        let putTotal = putNum.clone();
        println!("Puts {}",putNum);
        let mut getNum : i32 = (operations - (putNum as f64)) as i32;
        let getTotal = getNum.clone();

        println!("Gets {}",getNum);

        //Prepopulate hash tables
        let prepopulated_keys = (max_key / 2);
        println!("Prepopulated Keys: {}",prepopulated_keys);
        for i in 0..prepopulated_keys {
            random_num = get_random_key(max_key);
            let mut dest_ip = ip_list_vec[((calculate_hash(&random_num) % num_nodes) + 1) as usize];
            let packet = Packet {operation: false, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
            send_put_packet(dest_ip, &packet);
        }

//        let mut successfulGet : i32 = 0;
//        let mut successfulPut : i32 = 0;

        let start = Instant::now();
//        let start_time: i32 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i32;
//        println!("Start Time: {}", start_time);
        while getNum > 0 {
//            println!("Get Key");
            random_num = get_random_key(max_key);
//            println!("{}",random_num);
            let packet = Packet { operation: true, key: random_num as i32, is_int: true, val: &[0] };
            let mut dest_ip = ip_list_vec[((calculate_hash(&random_num) % num_nodes) + 1) as usize];
//            println!("{}",dest_ip);

//            successfulGet +=
            send_get_packet(dest_ip, &packet);
            getNum = getNum-1;
//            println!("{}", getNum);
            if putNum > 0 {
                random_num = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() % max_key;
                let mut dest_ip = ip_list_vec[((calculate_hash(&random_num) % num_nodes) + 1) as usize];
                let packet = Packet {operation: false, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
//                successfulPut +=
                send_put_packet(dest_ip, &packet);
                putNum = putNum-1;
            }
        }
        let end = start.elapsed();
//        let end_time : i32 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i32;
//        println!("End Time: {}", end_time);
//        let operation_time : i32= (end_time - start_time) as i32;
//        let operation_time_ms : i32 = (operation_time / 1000000);
        println!("Total Time(milliseconds): {}", end.as_millis());
//        println!("Successful Puts: {}/{}", successfulPut, putTotal);
//        println!("Successful Gets: {}/{}", successfulGet, getTotal);

//
    } // the socket is closed here
    Ok(())
}

fn get_random_key(max_key: u32) -> u32 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % max_key;
}