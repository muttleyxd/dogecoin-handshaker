pub mod header;
pub mod messages;
pub mod serializer;

#[allow(dead_code)] // Main and RegressionTest are not used outside of unit tests
pub enum NetworkType {
    Main,
    Test,
    RegressionTest,
}

use bitcoin_hashes::Hash;
fn calculate_message_hash(message: &[u8]) -> [u8; 4] {
    let hash = bitcoin_hashes::sha256d::Hash::hash(message);
    [hash[0], hash[1], hash[2], hash[3]]
}

#[derive(Debug, PartialEq)]
pub struct IntegerParsingFailure;

impl std::fmt::Display for IntegerParsingFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Integer parsing failure")
    }
}

impl std::error::Error for IntegerParsingFailure {}

fn string_to_ip(ip_address: &str) -> Option<[u8; 4]> {
    let splits = ip_address.split('.').collect::<Vec<&str>>();
    if splits.len() != 4 {
        return None;
    }

    let octets = (
        splits[0].parse::<u8>().ok()?,
        splits[1].parse::<u8>().ok()?,
        splits[2].parse::<u8>().ok()?,
        splits[3].parse::<u8>().ok()?,
    );

    Some([octets.0, octets.1, octets.2, octets.3])
}

#[cfg(test)]
mod tests {
    use super::calculate_message_hash;

    pub const VERSION_MESSAGE: [u8; 105] = [
        0x7f, 0x11, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x66, 0x34,
        0x64, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x34, 0x4d, 0xe7, 0x29, 0xae,
        0x0c, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0xdf, 0x74,
        0xaf, 0xc8, 0x94, 0x63, 0xf8, 0x13, 0x2f, 0x53, 0x68, 0x69, 0x62, 0x65, 0x74, 0x6f, 0x73,
        0x68, 0x69, 0x3a, 0x31, 0x2e, 0x31, 0x34, 0x2e, 0x36, 0x2f, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];

    #[test]
    fn test_calculate_message_hash() {
        const EMPTY_MESSAGE: [u8; 0] = [];
        const EXPECTED_FOR_EMPTY_MESSAGE: [u8; 4] = [0x5D, 0xF6, 0xE0, 0xE2];
        assert_eq!(
            EXPECTED_FOR_EMPTY_MESSAGE,
            calculate_message_hash(&EMPTY_MESSAGE)
        );

        const EXPECTED_FOR_VERSION_MESSAGE: [u8; 4] = [0xA2, 0xBB, 0x58, 0x1C];
        assert_eq!(
            EXPECTED_FOR_VERSION_MESSAGE,
            calculate_message_hash(&VERSION_MESSAGE)
        );
    }
}
