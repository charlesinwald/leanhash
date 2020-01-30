use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let mut socket = UdpSocket::bind("127.0.0.1:34255")?;

        socket.send_to(b"Test", "127.0.0.1:34254")?;
    } // the socket is closed here
    Ok(())
}