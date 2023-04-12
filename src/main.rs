use std::error::Error;

mod dogecoin;

use dogecoin::NetworkType;

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

    eprintln!("Connecting to {}:{}", ip, port);
    let mut agent = dogecoin::node_connection_agent::NodeConnectionAgent::new(
        NetworkType::Test,
        ip.as_str(),
        port,
    )?;
    eprintln!("Connection successful!\nSending version packet...");

    agent.send_version()?;
    eprintln!("Sending version packet success! Trying to receive node version packet");

    let node_version = agent.receive_version()?;
    eprintln!("Received version data from remote node: {:?}", node_version);
    
    agent.send_version_ack()?;
    eprintln!("Sent version ack packet! Receiving version ack...");
    
    agent.read_version_ack()?;
    eprintln!("Received version ack, success! Closing...");

    Ok(())
}
