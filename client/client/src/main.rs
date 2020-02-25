use std::net::{UdpSocket, ToSocketAddrs, SocketAddr, Ipv4Addr, IpAddr, TcpStream};
use std::{mem, thread, time, process};
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
use std::io::{Write, Read};

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

fn send_put_packet(dest: &mut TcpStream, packet: &Packet) -> i32 {
    let bytes_to_send = serialize(&packet).unwrap();
    println!("{:?}", dest.write(&bytes_to_send));
    let mut buf = [0; 256];
    dest.read(&mut buf);
    println!("{:#?}", buf[0]);
    if buf[0] != 0 {
//        println!("true");
        return 1;
    } else {
//        println!("false");
        return 0;
    }
}

fn send_get_packet(mut dest: TcpStream, packet: &Packet) -> i32 {
    let bytes_to_send = serialize(&packet).unwrap();
    dest.write(&bytes_to_send);
    let mut buf = [0; 256];
    dest.read(&mut buf);
    if (buf[0] & 0) == 0 {
//        println!("null");
        return 0;
    } else {
        let value: Val = bincode::deserialize(&buf).unwrap();
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
        let num_nodes: u64 = ip_list_vec.len() as u64 - 1;
        if num_nodes < 1 {
            eprintln!("Error: Less than two nodes found in iplist, format should be this node's AWS Elastic IP, and the elastic node ips [1..n]");
            std::process::exit(1);
        }
        let this_ip = ip_list_vec[0];

//        let mut tcp_vec = vec![];
//        let mut iter = ip_list_vec.iter().enumerate();
//        iter.next(); //We want to skip the first one, since its the local ip, and would appear twice
//        for (i, ip) in iter {
//            println!("In position {} we have value {}", i, ip);
//            let remote: SocketAddr = ip.parse().unwrap();
//            if let Ok(stream) = TcpStream::connect_timeout(&remote,Duration::from_secs(3)) {
//                stream.set_read_timeout(Some(Duration::from_secs(3)));
//                println!("Connected to {}", ip);
//                tcp_vec.push(stream);
//            } else {
//                println!("Couldn't connect to {}", ip);
//                process::exit(0x0100); //Quit
//            }
//        }


        let max_key: u32 = args.max_key;
        //Workaround for not having a proper randomization function
        let mut random_num = get_random_key(max_key);
        println!("epoch {}", random_num);
        println!("nanos: {}", random_num % max_key);

        let mut putNum: i32 = (0.4 * operations).round() as i32;
        let putTotal = putNum.clone();
        println!("Puts {}", putNum);
        let mut getNum: i32 = (operations - (putNum as f64)) as i32;
        let getTotal = getNum.clone();

        println!("Gets {}", getNum);

        //Prepopulate hash tables
        let prepopulated_keys = (max_key / 2);
        println!("Prepopulated Keys: {}", prepopulated_keys);
        for i in 0..prepopulated_keys {
            random_num = get_random_key(max_key);
            let mut destination_node = &mut TcpStream::connect(ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize]).unwrap();
            let packet = Packet { operation: false, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
            println!("{:?}", send_put_packet(destination_node, &packet));
        }


        let start = Instant::now();
        while getNum > 0 {
            random_num = get_random_key(max_key);
            let packet = Packet { operation: true, key: random_num as i32, is_int: true, val: &[0] };
            let mut destination_node = TcpStream::connect(ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize]).unwrap();

            send_get_packet(destination_node, &packet);
            getNum = getNum - 1;
            if putNum > 0 {
                random_num = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() % max_key;
                let mut destination_node = &mut TcpStream::connect(ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize]).unwrap();
                let packet = Packet { operation: false, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
                send_put_packet(destination_node, &packet);
                putNum = putNum - 1;
            }
        }
        let end = start.elapsed();
        println!("Total Time(milliseconds): {}", end.as_millis());
    } // the socket is closed here
    Ok(())
}

fn get_random_key(max_key: u32) -> u32 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() % max_key;
}