

use std::net::UdpSocket;
use std::mem;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

use bincode::{serialize, deserialize};

//TODO have enum so you can specify the type of data being saved in val, instead of assuming string

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Packet<'a> {
operation: bool, //get = false, put = true
key: i32,
val_len: i32,
val: &'a [u8],
}



fn main() -> std::io::Result<()> {
    {
        let mut socket = UdpSocket::bind("127.0.0.1:34255")?;

        let key = 5;
        let value = "Value to be stored";
        let packet = Packet{ operation: false, key, val_len: mem::size_of_val(value) as i32, val: value.as_bytes()};
        let bytes_to_send = serialize(&packet).unwrap();
        socket.send_to(&bytes_to_send, "127.0.0.1:34254")?;
    } // the socket is closed here
    Ok(())
}