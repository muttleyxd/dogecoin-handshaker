use super::NetworkType;

#[derive(Debug, PartialEq)]
pub enum HeaderBuildError {
    CommandTooLong,
    MessageTooLong(usize),
}

impl std::fmt::Display for HeaderBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderBuildError::CommandTooLong => write!(f, "Command too long"),
            HeaderBuildError::MessageTooLong(value) => {
                write!(f, "Value too big for u32: {}", value)
            }
        }
    }
}

impl std::error::Error for HeaderBuildError {}

fn build_network_type_header(network_type: NetworkType) -> &'static [u8; 4] {
    match network_type {
        NetworkType::Main => &[0xC0, 0xC0, 0xC0, 0xC0],
        NetworkType::Test => &[0xFC, 0xC1, 0xB7, 0xDC],
        NetworkType::RegressionTest => &[0xFA, 0xBF, 0xB5, 0xDA],
    }
}

const NETWORK_TYPE_HEADER_SIZE: usize = 4;
const COMMAND_SIZE: usize = 12;
const MESSAGE_SIZE_SIZE: usize = 4;
const HASH_SIZE: usize = 4;

pub const HEADER_SIZE: usize =
    NETWORK_TYPE_HEADER_SIZE + COMMAND_SIZE + MESSAGE_SIZE_SIZE + HASH_SIZE;

pub fn build_header(
    network_type: NetworkType,
    command: &str,
    message: &[u8],
) -> Result<Vec<u8>, HeaderBuildError> {
    if command.len() > COMMAND_SIZE {
        return Err(HeaderBuildError::CommandTooLong);
    }

    let length: Option<u32> = message.len().try_into().ok();
    if length.is_none() {
        return Err(HeaderBuildError::MessageTooLong(message.len()));
    }

    let mut buffer = Vec::new();
    buffer.reserve(HEADER_SIZE);

    buffer.extend_from_slice(build_network_type_header(network_type));
    buffer.extend_from_slice(command.as_bytes());

    let padding_size = COMMAND_SIZE - command.len();
    buffer.resize(buffer.len() + padding_size, 0);

    buffer.extend_from_slice(length.unwrap().to_le_bytes().as_slice());
    buffer.extend_from_slice(&super::calculate_message_hash(message));

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::NetworkType;
    use super::{build_header, build_network_type_header};
    use crate::dogecoin::header::HeaderBuildError;

    #[test]
    fn test_build_network_type_header() {
        let bytes = build_network_type_header(NetworkType::Main);
        assert_eq!(&[0xC0, 0xC0, 0xC0, 0xC0], bytes);

        let bytes = build_network_type_header(NetworkType::Test);
        assert_eq!(&[0xFC, 0xC1, 0xB7, 0xDC], bytes);

        let bytes = build_network_type_header(NetworkType::RegressionTest);
        assert_eq!(&[0xFA, 0xBF, 0xB5, 0xDA], bytes);
    }

    #[test]
    fn test_build_header_command_too_long() {
        let message = Vec::new();
        let result = build_header(NetworkType::Test, "verylongcommandname", &message);
        assert_eq!(Err(HeaderBuildError::CommandTooLong), result);
    }

    #[test]
    fn test_build_header_verack() {
        let message = Vec::new();
        let bytes = build_header(NetworkType::Test, "verack", &message);
        assert_eq!(
            &[
                0xFC, 0xC1, 0xB7, 0xDC, 'v' as u8, 'e' as u8, 'r' as u8, 'a' as u8, 'c' as u8,
                'k' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5D, 0xF6,
                0xE0, 0xE2
            ],
            bytes.unwrap().as_slice()
        );
    }

    #[test]
    fn test_build_header_version() {
        let bytes = build_header(
            NetworkType::Test,
            "version",
            &crate::dogecoin::tests::VERSION_MESSAGE,
        );
        assert_eq!(
            &[
                0xFC, 0xC1, 0xB7, 0xDC, 'v' as u8, 'e' as u8, 'r' as u8, 's' as u8, 'i' as u8,
                'o' as u8, 'n' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x69, 0x00, 0x00, 0x00, 0xA2,
                0xBB, 0x58, 0x1C
            ],
            bytes.unwrap().as_slice()
        );
    }
}
