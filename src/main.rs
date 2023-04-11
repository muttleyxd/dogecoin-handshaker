use rand::Rng;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::{SystemTime, SystemTimeError};

mod dogecoin;

use dogecoin::{
    messages::{
        verack::build_verack_message,
        version::{build_version_message, fill_version_message_data},
    },
    NetworkType,
};

fn get_unix_timestamp() -> Result<u64, SystemTimeError> {
    let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

    Ok(duration.as_secs())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!(
            "Usage: {} -- <IP> <port>\nExample: cargo run -- 52.77.231.41 44556",
            args[0]
        );
        return Err("Incorrect arguments".into());
    }

    let ip = &args[1];
    let port_str = &args[2];
    let port = port_str.parse::<u16>()?;

    let mut random_number_generator = rand::thread_rng();

    eprintln!("Constructing version & version ack messages");
    let version_message = build_version_message(
        NetworkType::Test,
        fill_version_message_data(
            get_unix_timestamp()?,
            ip.as_str(),
            port,
            random_number_generator.gen(),
            "/Shibetoshi:1.14.6/",
        )?,
    )?;

    let expected_response = build_verack_message(NetworkType::Test)?;
    
    eprintln!("Constructed messages, connecting to {}:{}", ip, port);

    let address = SocketAddrV4::new(Ipv4Addr::from_str(ip.as_str())?, port);
    let mut stream = TcpStream::connect(address)?;
    
    eprintln!("Connection successful!\nSending version packet...");
    let write_result = stream.write(version_message.as_slice())?;
    if write_result != version_message.len() {
        return Err("Incorrect number of bytes sent".into());
    }
    eprintln!("Sending version packet success! Trying to receive version ack packet");

    let mut response_buffer: [u8; 24] = [0; 24];
    let read_size = stream.read(&mut response_buffer)?;
    if read_size != expected_response.len() {
        return Err("Incorrect number of bytes received".into());
    }
    eprintln!("Received packet, verifying if response is correct...");

    if response_buffer != expected_response.as_slice() {
        eprintln!(
            "received: {:?}\nexpected: {:?}",
            response_buffer, expected_response
        );
        return Err("Incorrect version ack response received".into());
    }
    eprintln!("Correct version ack (verack) packet received, closing...");

    Ok(())
}
