mod header;
mod result;
mod packet;
mod query;
mod util;
use std::net::UdpSocket;
use util::handle_query;


fn main() {
    let socket = UdpSocket::bind(("0.0.0.0", 2053)).unwrap();

    loop {
        match handle_query(&socket) {
            Ok(_) => {}
            Err(e) => eprintln!("An error occurred: {}", e),
        }
    }
}
