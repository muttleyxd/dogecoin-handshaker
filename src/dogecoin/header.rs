use super::*;
use errors::*;
use serializer::{slice_to_string, slice_to_u32};

#[derive(Debug, PartialEq)]
pub struct Header {
    pub network_type: NetworkType,
    pub command: String,
    pub message_size: usize,
    pub hash: [u8; 4],
}

impl Header {
    pub fn from_network_bytes(bytes: &[u8]) -> Result<Self, HeaderBuildError> {
        if bytes.len() < HEADER_SIZE {
            return Err(HeaderBuildError::TooShort);
        }

        let network_type = NetworkType::from_network_bytes(&bytes[0..4]);
        if let Err(e) = network_type {
            return match e {
                NetworkSerializationError::BufferTooShort => Err(HeaderBuildError::TooShort),
                NetworkSerializationError::UnknownBytes => {
                    Err(HeaderBuildError::UnknownNetworkType)
                }
                NetworkSerializationError::HeaderParseError(e) => Err(e),
                NetworkSerializationError::StringParseError => {
                    Err(HeaderBuildError::CommandTooLong)
                }
            };
        }

        let command = slice_to_string(&bytes[4..16]);
        if command.is_empty() {
            return Err(HeaderBuildError::CommandIsEmpty);
        }

        let message_size =
            slice_to_u32(&bytes[16..20]).ok_or(HeaderBuildError::MessageSizeParseFailure)?;
        let message_size = message_size as usize;

        Ok(Self {
            network_type: network_type.unwrap(),
            command,
            message_size,
            hash: [bytes[20], bytes[21], bytes[22], bytes[23]],
        })
    }

    pub fn to_network_bytes(&self, message: &[u8]) -> Result<Vec<u8>, HeaderBuildError> {
        if self.command.len() > COMMAND_SIZE {
            return Err(HeaderBuildError::CommandTooLong);
        }

        let length: Option<u32> = message.len().try_into().ok();
        if length.is_none() {
            return Err(HeaderBuildError::MessageTooLong(message.len()));
        }
        let length = length.unwrap();

        let mut buffer = Vec::new();
        buffer.reserve(HEADER_SIZE);

        buffer.extend_from_slice(self.network_type.to_network_bytes()?.as_slice());
        buffer.extend_from_slice(self.command.as_bytes());

        let padding_size = COMMAND_SIZE - self.command.len();
        buffer.resize(buffer.len() + padding_size, 0);

        buffer.extend_from_slice(length.to_le_bytes().as_slice());
        buffer.extend_from_slice(&super::calculate_message_hash(message));

        Ok(buffer)
    }
}

const NETWORK_TYPE_HEADER_SIZE: usize = 4;
const COMMAND_SIZE: usize = 12;
const MESSAGE_SIZE_SIZE: usize = 4;
const HASH_SIZE: usize = 4;

pub const HEADER_SIZE: usize =
    NETWORK_TYPE_HEADER_SIZE + COMMAND_SIZE + MESSAGE_SIZE_SIZE + HASH_SIZE;

#[cfg(test)]
mod tests {
    use super::super::NetworkSerializable;
    use super::*;

    const VERACK_HEADER: [u8; 24] = [
        0xFC, 0xC1, 0xB7, 0xDC, 'v' as u8, 'e' as u8, 'r' as u8, 'a' as u8, 'c' as u8, 'k' as u8,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5D, 0xF6, 0xE0, 0xE2,
    ];
    const VERSION_HEADER: [u8; 24] = [
        0xFC, 0xC1, 0xB7, 0xDC, 'v' as u8, 'e' as u8, 'r' as u8, 's' as u8, 'i' as u8, 'o' as u8,
        'n' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x69, 0x00, 0x00, 0x00, 0xA2, 0xBB, 0x58, 0x1C,
    ];

    #[test]
    fn test_build_network_type_header() {
        let bytes = NetworkType::Main.to_network_bytes();
        assert!(bytes.is_ok());
        assert_eq!(&[0xC0, 0xC0, 0xC0, 0xC0], bytes.unwrap().as_slice());

        let bytes = NetworkType::Test.to_network_bytes();
        assert!(bytes.is_ok());
        assert_eq!(&[0xFC, 0xC1, 0xB7, 0xDC], bytes.unwrap().as_slice());

        let bytes = NetworkType::RegressionTest.to_network_bytes();
        assert!(bytes.is_ok());
        assert_eq!(&[0xFA, 0xBF, 0xB5, 0xDA], bytes.unwrap().as_slice());
    }

    #[test]
    fn test_build_header_command_too_long() {
        let message = Vec::new();
        let result = Header {
            network_type: NetworkType::Test,
            command: "verylongcommandname".to_string(),
            message_size: 0,
            hash: [0; 4],
        }
        .to_network_bytes(&message);
        assert_eq!(Err(HeaderBuildError::CommandTooLong), result);
    }

    #[test]
    fn test_build_header_verack() {
        let message = Vec::new();
        let bytes = Header {
            network_type: NetworkType::Test,
            command: "verack".to_string(),
            message_size: 0,
            hash: [0; 4],
        }
        .to_network_bytes(&message);
        assert_eq!(&VERACK_HEADER, bytes.unwrap().as_slice());
    }

    #[test]
    fn test_build_header_version() {
        let bytes = Header {
            network_type: NetworkType::Test,
            command: "version".to_string(),
            message_size: crate::dogecoin::tests::VERSION_MESSAGE.len(),
            hash: [0; 4],
        }
        .to_network_bytes(&crate::dogecoin::tests::VERSION_MESSAGE);
        assert_eq!(&VERSION_HEADER, bytes.unwrap().as_slice());
    }

    #[test]
    fn test_parse_header_verack() {
        let header = Header::from_network_bytes(&VERACK_HEADER);
        assert!(header.is_ok());

        let header = header.unwrap();
        assert_eq!(NetworkType::Test, header.network_type);
        assert_eq!("verack", header.command);
        assert_eq!(0, header.message_size);
        assert_eq!([0x5D, 0xF6, 0xE0, 0xE2], header.hash)
    }
}
