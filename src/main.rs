extern crate config;

use structopt::StructOpt;
use std::collections::HashMap;
use config::Value;
use std::string::ToString;
use std::net::UdpSocket;

//use std::sync::{Mutex, RwLock, Arc};

#[derive(StructOpt)]
struct Cli {
    /// The pattern to look for
    #[structopt(default_value = "foobar", long)]
    operation: String,
}

#[derive(Debug)]
enum Val<'a> {
    String(&'a str),
    Integer(i32),
}


fn main() {
        let args = Cli::from_args();
        println!("operation {}", &args.operation);

//  Load Config
        let mut settings = config::Config::default();
        settings
            // Add in `./Settings.toml`
            .merge(config::File::with_name("Settings")).unwrap();
        // Print out our settings (as a HashMap)
        let settings_map = settings.try_into::<HashMap<String, Vec<Value>>>().expect("Error reading Settings.toml");
        let config_map = settings_map;
//    println!("{:?}",
//             config_map);
        let ip_array = config_map.get_key_value("ips").expect("Error reading list of node ips");
        println!("{:#?}", ip_array.1);

    {
        let mut socket = UdpSocket::bind("127.0.0.1:34254").expect("Error creating socket on 127.0.0.1:34254");

        // Thread Safe version
        //let mut cc = Arc::new(RwLock::new(HashMap::new()));
        let mut cc = HashMap::new();

        cc.insert(1, Val::Integer(5));
        cc.insert(2, Val::String("five"));

        for v in cc.values() {
            println!("{:?}", v);
        }

        loop {
            let mut buf = [0; 10];
            let (number_of_bytes, src_addr) = socket.recv_from(&mut buf)
                .expect("Didn't receive data");
            let filled_buf = &mut buf[..number_of_bytes];
            println!("Recieved: {:#?}",&buf);
        }
    }
}

