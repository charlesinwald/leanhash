use std::net::{UdpSocket, ToSocketAddrs, SocketAddr, Ipv4Addr, IpAddr, TcpStream};
use std::{mem, thread, time, process};
use std::env;
use std::fs;

use client::ThreadPool;


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
    #[structopt(default_value = "1000", short, long)]
    operations: f64,
    #[structopt(default_value = "1000", short, long)]
    max_key: u32,
    #[structopt(default_value = "2", short, long, help = "Replication degree 1 or 2")]
    replication: i32, //1 or 2
}


#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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



fn send_put_packet(dest: &mut TcpStream, dest2: &mut TcpStream, dest3: &mut TcpStream, packet: &Packet, thirdreplica: bool) -> i32 {
    let bytes_to_send = serialize(&packet).unwrap();
    //Since we want to retry if we fail for concurrency, we loop,
        // Successful puts return
        // NACKs continue
    // loop {
        //Phase 1, first node, ask to put
        let (mut ack, mut ack2, mut ack3) = ([0; 1], [0; 1], [0; 1]);
        commit_request(dest, &bytes_to_send, &mut ack);
        //If first node says yes keep going
        if ack[0] == 1 {
            commit_request(dest2, &bytes_to_send, &mut ack2);
            //If second node says yes keep going
            if ack2[0] == 1 {
                if thirdreplica {
                    commit_request(dest3, &bytes_to_send, &mut ack3);
                    if ack3[0] == 1 {
                        //All nodes are okay with decision, reply to proceed
                        yes(dest, dest2, dest3);
                        return 1;
                    }
                }
                else {
                    yes2(dest, dest2);
                    return 1;
                }
            }
            else {
                if thirdreplica {
                    println!("NACK");
                    abort(dest, dest2, dest3);
                }
                else {
                    println!("NACK");
                    abort2(dest, dest2);
                }
            }
        }
        else {
            if thirdreplica {
                println!("NACK");
                abort(dest, dest2, dest3);
            }
            else {
                println!("NACK");
                abort2(dest, dest2);
            }
        }
        //         else {
        //             // println!("NACK");
        //             continue;
        //         }
        //     } else {
        //         // println!("NACK");
        //         continue;
        //     }
        // } else {
        //     // println!("NACK");
        //     continue;
        // }
    // }
    return 0;
}

fn send_multiput_packet(dest: &mut TcpStream, dest2: &mut TcpStream, dest3: &mut TcpStream, packet: &Packet, packet2: &Packet, packet3: &Packet) -> i32 {
    let bytes_to_send = serialize(&packet).unwrap();
    let bytes_to_send2 = serialize(&packet2).unwrap();
    let bytes_to_send3 = serialize(&packet3).unwrap();

    //Since we want to retry if we fail for concurrency, we loop,
    // Successful puts return
    // NACKs continue
    loop {
        let (mut ack, mut ack2, mut ack3) = ([0; 1], [0; 1], [0; 1]);
        let (mut ack4, mut ack5, mut ack6) = ([0; 1], [0; 1], [0; 1]);
        let (mut ack7, mut ack8, mut ack9) = ([0; 1], [0; 1], [0; 1]);

        //Phase 1, first node, ask to put
        commit_request(dest, &bytes_to_send, &mut ack);
        //If first node says yes keep going
        if (ack[0] & ack2[0] & ack3[0]) == 1 {
            commit_request(dest2, &bytes_to_send, &mut ack4);

            //If second node says yes keep going
            if (ack4[0] & ack5[0] & ack6[0]) == 1 {
                commit_request(dest3, &bytes_to_send, &mut ack7);

                if (ack7[0] & ack8[0] & ack9[0]) == 1 {
                    //All nodes are okay with decision, reply to proceed
                    yes(dest, dest2, dest3);
                    return 1;
                } else {
                    abort(dest, dest2, dest3);
                    continue;
                }
            } else {
                abort(dest, dest2, dest3);
                continue;
            }
        } else {
            abort(dest, dest2, dest3);
            continue;
        }
    }
    return 0;
}


fn yes(dest: &mut TcpStream, dest2: &mut TcpStream, dest3: &mut TcpStream) {
    dest.write(&[1]).expect("Error writing True to the stream");
    dest2.write(&[1]).expect("Error writing True to the stream");
    dest3.write(&[1]).expect("Error writing True to the stream");
}

fn abort(dest: &mut TcpStream, dest2: &mut TcpStream, dest3: &mut TcpStream) {
    dest.write(&[0]).expect("Error writing True to the stream");
    dest2.write(&[0]).expect("Error writing True to the stream");
    dest3.write(&[0]).expect("Error writing True to the stream");
}
fn abort2(dest: &mut TcpStream, dest2: &mut TcpStream) {
    dest.write(&[0]).expect("Error writing True to the stream");
    dest2.write(&[0]).expect("Error writing True to the stream");
}
fn yes2(dest: &mut TcpStream, dest2: &mut TcpStream) {
    dest.write(&[1]).expect("Error writing True to the stream");
    dest2.write(&[1]).expect("Error writing True to the stream");
}

fn commit_request(dest: &mut TcpStream, bytes_to_send: &[u8], mut ack: &mut [u8; 1]) {
    dest.write(&bytes_to_send);
    dest.read( ack);
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
        let degree: i32 = args.replication;
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

        //Create threadpool with 8 threads
        let pool = ThreadPool::new(8);


        println!("Gets {}", getNum);

        let start = Instant::now();
        while getNum > 0 {
            ip_list_vec.clone();
            // pool.execute(move || {
                //Retrieve list of ips
                let ip_list_string = fs::read_to_string("iplist").expect("Could not read in node ip list");
                let ip_list_vec = ip_list_string.split("\n").collect::<Vec<&str>>();
                let num_nodes: u64 = ip_list_vec.len() as u64 - 1;
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
                        // send_put_packet(destination_node, destination_node2, destination_node3, &packet);
                    if degree == 2 {
                        send_put_packet(destination_node, destination_node2, destination_node3, &packet, true);
                    }
                    else {
                        send_put_packet(destination_node, destination_node2, destination_node3, &packet, false);
                    }
                    putNum = putNum - 1;
            }
            // });
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