use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

fn main() {
    let endpoint = SocketAddr::from(([127, 0, 0, 1], 1433));
    if TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).is_err() {
        std::process::exit(1);
    }
}
