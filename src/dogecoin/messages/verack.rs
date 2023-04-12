use super::super::*;
use header::Header;
use header::HEADER_SIZE;

// verack -> version ack

pub struct Verack {
    header: Header,
}

impl Verack {
    pub fn new(network_type: NetworkType) -> Self {
        Verack {
            header: Header {
                network_type,
                command: "verack".to_string(),
                message_size: 0,
                hash: [0; 4],
            },
        }
    }
}

impl NetworkSerializable<Verack> for Verack {
    fn from_network_bytes(bytes: &[u8]) -> Result<Verack, NetworkSerializationError> {
        if bytes.len() < HEADER_SIZE {
            return Err(NetworkSerializationError::UnknownBytes);
        }
        Ok(Verack {
            header: Header::from_network_bytes(&bytes[0..24])?,
        })
    }

    fn to_network_bytes(&self) -> Result<Vec<u8>, NetworkSerializationError> {
        Ok(self.header.to_network_bytes(&[])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_verack_message() {
        let message = Verack::new(NetworkType::Test);
        let bytes = message.to_network_bytes();
        assert_eq!(
            &[
                0xFC, 0xC1, 0xB7, 0xDC, 'v' as u8, 'e' as u8, 'r' as u8, 'a' as u8, 'c' as u8,
                'k' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5D, 0xF6,
                0xE0, 0xE2
            ],
            bytes.unwrap().as_slice()
        );
    }
}
