use super::super::{header::HEADER_SIZE, *};
use serializer::{calculate_size_of_serialized_string_and_length_bytes, SerializeString};
use std::borrow::Borrow;
use std::mem::size_of;

pub type IpAddress = [u8; 16];

#[derive(Debug, PartialEq)]
pub struct IpData {
    pub node_services: u64,
    pub ip_address: IpAddress,
    pub port: u16,
}

#[derive(Debug, PartialEq)]
pub struct VersionMessageData {
    pub protocol_version: u32,
    pub local_node_services: u64,
    pub unix_timestamp: u64,
    pub node_ip_data: IpData,
    pub our_ip_data: IpData,
    pub nonce: u64,
    pub client_name: String,
    pub node_starting_height: u32,
    pub relay_transactions: bool,
}

pub fn fill_version_message_data(
    unix_timestamp: u64,
    target_ip_address: &str,
    port: u16,
    nonce: u64,
    client_name: &str,
) -> Result<VersionMessageData, IntegerParsingFailure> {
    const PROTOCOL_VERSION: u32 = 70015;

    const NODE_NETWORK: u64 = 1;
    const NODE_BLOOM: u64 = 4;
    const LOCAL_NODE_SERVICES: u64 = NODE_NETWORK | NODE_BLOOM;

    let octets = string_to_ip(target_ip_address);
    if octets.is_none() {
        return Err(IntegerParsingFailure);
    }
    let octets = octets.unwrap();

    Ok(VersionMessageData {
        protocol_version: PROTOCOL_VERSION,
        local_node_services: LOCAL_NODE_SERVICES,
        unix_timestamp,
        node_ip_data: IpData {
            node_services: NODE_NETWORK,
            ip_address: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, octets[0], octets[1], octets[2],
                octets[3],
            ],
            port,
        },
        our_ip_data: IpData {
            node_services: 5,
            ip_address: [0; 16],
            port: 0,
        },
        nonce,
        client_name: client_name.to_string(),
        node_starting_height: 0,
        relay_transactions: true,
    })
}

pub fn build_version_message(
    network_type: NetworkType,
    data: VersionMessageData,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const DATA_SIZE_WITHOUT_CLIENT_NAME: usize = size_of::<u32>()
        + size_of::<u64>()
        + size_of::<u64>()
        + size_of::<IpData>()
        + size_of::<IpData>()
        + size_of::<u64>()
        + size_of::<u32>()
        + size_of::<bool>();

    let serialized_string_and_length_bytes_size =
        calculate_size_of_serialized_string_and_length_bytes(data.client_name.len());

    let message_size = DATA_SIZE_WITHOUT_CLIENT_NAME + serialized_string_and_length_bytes_size?;
    let total_size = HEADER_SIZE + message_size;

    let mut buffer = Vec::new();
    buffer.reserve(total_size);

    buffer.resize(HEADER_SIZE, 0);

    buffer.extend_from_slice(&data.protocol_version.to_le_bytes());
    buffer.extend_from_slice(&data.local_node_services.to_le_bytes());
    buffer.extend_from_slice(&data.unix_timestamp.to_le_bytes());

    buffer.extend_from_slice(&data.node_ip_data.node_services.to_le_bytes());
    buffer.extend_from_slice(data.node_ip_data.ip_address.as_slice());
    buffer.extend_from_slice(&data.node_ip_data.port.to_be_bytes());

    buffer.extend_from_slice(&data.our_ip_data.node_services.to_le_bytes());
    buffer.extend_from_slice(data.our_ip_data.ip_address.as_slice());
    buffer.extend_from_slice(&data.our_ip_data.port.to_be_bytes());

    buffer.extend_from_slice(&data.nonce.to_le_bytes());
    buffer.extend_from_slice(data.client_name.to_dogecoin_bytes()?.as_ref());
    buffer.extend_from_slice(&data.node_starting_height.to_le_bytes());
    buffer.push(data.relay_transactions.into());

    // Copy header into buffer
    let header = header::build_header(
        network_type,
        "version",
        &buffer.as_slice()[HEADER_SIZE..buffer.len()],
    )?;
    buffer.as_mut_slice()[0..HEADER_SIZE]
        .clone_from_slice(header.as_slice()[0..HEADER_SIZE].borrow());

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_version_message_data() {
        let data = fill_version_message_data(0, "invalid ip", 0, 0, "");
        assert!(data.is_err());
        assert_eq!(data.unwrap_err(), IntegerParsingFailure);

        let data = fill_version_message_data(
            1681155665,
            "52.77.231.41",
            44556,
            17898312933758525253,
            "/Shibetoshi:1.14.6/",
        );
        assert!(data.is_ok());
        assert_eq!(
            VersionMessageData {
                protocol_version: 70015,
                local_node_services: 5,
                unix_timestamp: 1681155665,
                node_ip_data: IpData {
                    node_services: 1,
                    ip_address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0xFF, 0xFF, 52, 77, 231, 41],
                    port: 44556,
                },
                our_ip_data: IpData {
                    node_services: 5,
                    ip_address: [0; 16],
                    port: 0,
                },
                nonce: 17898312933758525253,
                client_name: "/Shibetoshi:1.14.6/".to_string(),
                node_starting_height: 0,
                relay_transactions: true,
            },
            data.unwrap()
        );
    }

    #[test]
    fn test_build_verack_message() {
        let data = fill_version_message_data(
            1681155665,
            "52.77.231.41",
            44556,
            17898312933758525253,
            "/Shibetoshi:1.14.6/",
        );
        assert!(data.is_ok());
        let data = data.unwrap();

        let bytes = build_version_message(NetworkType::Test, data);
        assert_eq!(
            &[
                0xfc, 0xc1, 0xb7, 0xdc, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x69, 0x00, 0x00, 0x00, 0xa2, 0xbb, 0x58, 0x1c, 0x7f, 0x11, 0x01, 0x00,
                0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x66, 0x34, 0x64, 0x00, 0x00,
                0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x34, 0x4d, 0xe7, 0x29, 0xae, 0x0c,
                0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0xdf,
                0x74, 0xaf, 0xc8, 0x94, 0x63, 0xf8, 0x13, 0x2f, 0x53, 0x68, 0x69, 0x62, 0x65, 0x74,
                0x6f, 0x73, 0x68, 0x69, 0x3a, 0x31, 0x2e, 0x31, 0x34, 0x2e, 0x36, 0x2f, 0x00, 0x00,
                0x00, 0x00, 0x01
            ],
            bytes.unwrap().as_slice()
        );
    }
}
