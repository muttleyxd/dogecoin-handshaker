use rand::rngs::ThreadRng;
use rand::Rng;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::str::FromStr;
use std::time::{SystemTime, SystemTimeError};

use super::*;
use errors::*;
use header::Header;
use messages::{verack::Verack, version::Version};

pub struct NodeConnectionAgent {
    ip: String,
    network_type: NetworkType,
    port: u16,
    random_number_generator: ThreadRng,
    stream: TcpStream,
}

impl NodeConnectionAgent {
    pub fn new(
        network_type: NetworkType,
        ip: &str,
        port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let address = SocketAddrV4::new(Ipv4Addr::from_str(ip)?, port);

        Ok(Self {
            ip: ip.to_string(),
            network_type,
            port,
            random_number_generator: rand::thread_rng(),
            stream: TcpStream::connect(address)?,
        })
    }
    
    pub fn read_version_ack(&mut self) -> Result<(), NodeConnectionAgentError> {
        let mut header_buffer: [u8; 24] = [0; 24];
        let read_size = self.stream.read(&mut header_buffer)?;
        if read_size != header_buffer.len() {
            return Err(NodeConnectionAgentError::IncorrectNumberOfBytesReceived);
        }
        
        let verack_message = Verack::new(self.network_type.clone());
        let verack_bytes = verack_message.to_network_bytes()?;
    
        if header_buffer != verack_bytes.as_slice() {
            return Err(NodeConnectionAgentError::IncorrectResponse);
        }
        
        Ok(())
    }

    pub fn receive_version(&mut self) -> Result<Version, NodeConnectionAgentError> {
        let mut header_buffer: [u8; 24] = [0; 24];
        let read_size = self.stream.read(&mut header_buffer)?;
        if read_size != header_buffer.len() {
            return Err(NodeConnectionAgentError::IncorrectNumberOfBytesReceived);
        }

        let header = Header::from_network_bytes(&header_buffer)?;
        if header.command != "version" {
            return Err(NodeConnectionAgentError::UnexpectedCommand(
                "version".to_string(),
                header.command,
            ));
        }

        let mut data_buffer: Vec<u8> = Vec::new();
        data_buffer.resize(header.message_size, 0);
        let read_size = self.stream.read(data_buffer.as_mut_slice())?;
        if read_size != header.message_size {
            return Err(NodeConnectionAgentError::IncorrectNumberOfBytesReceived);
        }

        let mut version_bytes = Vec::new();
        version_bytes.extend(header_buffer);
        version_bytes.extend(data_buffer);

        Ok(Version::from_network_bytes(version_bytes.as_slice())?)
    }
    
    pub fn send_version_ack(&mut self) -> Result<(), NodeConnectionAgentError> {
        let verack_message = Verack::new(self.network_type.clone());
        let verack_bytes = verack_message.to_network_bytes()?;
        
        let write_size = self.stream.write(verack_bytes.as_slice())?;
        if write_size != verack_bytes.len() {
            return Err(NodeConnectionAgentError::IncorrectNumberOfBytesSent);
        }
        
        Ok(())
    }

    pub fn send_version(&mut self) -> Result<(), NodeConnectionAgentError> {
        const CLIENT_NAME: &str = "/Shibetoshi:1.14.6/";
        let version_message = Version::new(
            self.network_type.clone(),
            Self::get_unix_timestamp()?,
            self.ip.as_str(),
            self.port,
            self.random_number_generator.gen(),
            CLIENT_NAME,
        )?;
        let version_message_bytes = version_message.to_network_bytes()?;

        let write_result = self.stream.write(version_message_bytes.as_slice())?;
        if write_result != version_message_bytes.len() {
            return Err(NodeConnectionAgentError::IncorrectNumberOfBytesSent);
        }

        Ok(())
    }

    fn get_unix_timestamp() -> Result<u64, SystemTimeError> {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

        Ok(duration.as_secs())
    }
}
