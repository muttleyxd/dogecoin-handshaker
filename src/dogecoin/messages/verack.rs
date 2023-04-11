use super::super::*;

// verack -> version ack

pub fn build_verack_message(
    network_type: NetworkType,
) -> Result<Vec<u8>, header::HeaderBuildError> {
    header::build_header(network_type, "verack", &[])
}

#[cfg(test)]
mod tests {
    use super::build_verack_message;
    use super::NetworkType;

    #[test]
    fn test_build_verack_message() {
        let bytes = build_verack_message(NetworkType::Test);
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
