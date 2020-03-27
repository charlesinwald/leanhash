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
    operation: i32,
    //put = 0, get = 1, commit request = 2
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

fn send_put_packet(dest: &mut TcpStream, dest2: &mut TcpStream, dest3: &mut TcpStream, packet: &Packet) -> i32 {
    let bytes_to_send = serialize(&packet).unwrap();
    //Phase 1, first node, ask to put
    dest.write(&bytes_to_send);
    let (mut ack, mut ack2, mut ack3) = ([0; 1], [0; 1], [0; 1]);
    dest.read(&mut ack);
    //If first node says yes keep going
    if ack[0] == 1 {
        dest2.write(&bytes_to_send);
        dest2.read(&mut ack2);
        //If second node says yes keep going
        if ack2[0] == 1 {
            dest3.write(&bytes_to_send);
            dest3.read(&mut ack3);
            if ack3[0] == 1 {
                //All nodes are okay with decision, reply to proceed
                dest.write(&[1]).expect("Error writing True to the stream");
                dest2.write(&[1]).expect("Error writing True to the stream");
                dest3.write(&[1]).expect("Error writing True to the stream");
                return 1;
            }
            else {
                return 0;
            }
        } else {
            return 0;
        }
    } else {
        return 0;
    }
    return 0;
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
            let destip = ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize];
            println!("Node {}", destip);
            let mut destination_node = &mut TcpStream::connect(destip).unwrap();
            //First replica
            let destip2 = (ip_list_vec[((calculate_hash(&random_num)) + 1) as usize % num_nodes as usize]);
            println!("  Replica 1 {}", destip2);
            let mut destination_node2 = &mut TcpStream::connect(destip2).unwrap();
            //Second Replica
            let destip3 = (ip_list_vec[((calculate_hash(&random_num)) + 2) as usize % num_nodes as usize]);
            println!("  Replica 2 {}", destip3);
            let mut destination_node3 = &mut TcpStream::connect(destip3).unwrap();

            let packet = Packet { operation: 0, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
            send_put_packet(destination_node, destination_node2, destination_node3, &packet);
        }


        let start = Instant::now();
        while getNum > 0 {
            random_num = get_random_key(max_key);
            let packet = Packet { operation: 1, key: random_num as i32, is_int: true, val: &[0] };
            let mut destination_node = TcpStream::connect(ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize]).unwrap();

            send_get_packet(destination_node, &packet);
            getNum = getNum - 1;
            if putNum > 0 {
                random_num = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() % max_key;
                let mut destination_node = &mut TcpStream::connect(ip_list_vec[(calculate_hash(&random_num) % num_nodes) as usize]).unwrap();
                let mut destination_node2 = &mut TcpStream::connect((ip_list_vec[((calculate_hash(&random_num)) + 1) as usize % num_nodes as usize])).unwrap();
                let mut destination_node3 = &mut TcpStream::connect((ip_list_vec[((calculate_hash(&random_num)) + 2) as usize % num_nodes as usize])).unwrap();
                let packet = Packet { operation: 0, key: random_num as i32, is_int: true, val: &random_num.to_ne_bytes() };
                send_put_packet(destination_node, destination_node2, destination_node3, &packet);
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