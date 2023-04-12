use rand::Rng;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::{SystemTime, SystemTimeError};

mod dogecoin;

use crate::dogecoin::header::Header;
use crate::dogecoin::messages::verack::Verack;
use crate::dogecoin::messages::version::Version;
use crate::dogecoin::NetworkSerializable;
use dogecoin::NetworkType;

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

    const NETWORK_TYPE: NetworkType = NetworkType::Test;

    let version_message = Version::new(
        NETWORK_TYPE,
        get_unix_timestamp()?,
        ip.as_str(),
        port,
        random_number_generator.gen(),
        "/Shibetoshi:1.14.6/",
    )?;
    let version_message_bytes = version_message.to_network_bytes()?;

    eprintln!("Connecting to {}:{}", ip, port);
    let address = SocketAddrV4::new(Ipv4Addr::from_str(ip.as_str())?, port);
    let mut stream = TcpStream::connect(address)?;
    eprintln!("Connection successful!\nSending version packet...");

    let write_result = stream.write(version_message_bytes.as_slice())?;
    if write_result != version_message_bytes.len() {
        return Err("Incorrect number of bytes sent".into());
    }
    eprintln!("Sending version packet success! Trying to receive node version packet");

    let mut header_buffer: [u8; 24] = [0; 24];
    let read_size = stream.read(&mut header_buffer)?;
    if read_size != header_buffer.len() {
        return Err("Incorrect number of bytes received".into());
    }
    eprintln!("Received packet, verifying if response is correct...");

    let header = Header::from_network_bytes(&header_buffer)?;
    if header.command != "version" {
        return Err(format!(
            "Incorrect command received, expected: 'version', actual: '{}'",
            header.command
        )
        .into());
    }

    let mut data_buffer: Vec<u8> = Vec::new();
    data_buffer.resize(header.message_size, 0);
    let read_size = stream.read(data_buffer.as_mut_slice())?;
    if read_size != header.message_size {
        return Err("Incorrect message size".into());
    }

    let mut version_bytes = Vec::new();
    version_bytes.extend(header_buffer);
    version_bytes.extend(data_buffer);

    let version_data = Version::from_network_bytes(version_bytes.as_slice())?;
    eprintln!(
        "Received version data from remote node: {:?}\n\nSending version ack packet:",
        version_data
    );

    let verack_message = Verack::new(NETWORK_TYPE);
    let verack_bytes = verack_message.to_network_bytes()?;

    let write_size = stream.write(verack_bytes.as_slice())?;
    if write_size != verack_bytes.len() {
        return Err("Error sending version ack, incorrect write size".into());
    }
    eprintln!("Sent version ack packet! Receiving version ack...");

    let read_size = stream.read(&mut header_buffer)?;
    if read_size != header_buffer.len() {
        return Err("Incorrect number of bytes received".into());
    }

    if header_buffer != verack_bytes.as_slice() {
        return Err("Incorrect version ack packet received".into());
    }
    eprintln!("Correct version ack (verack) packet received, closing...");

    Ok(())
}
