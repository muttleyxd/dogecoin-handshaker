use super::super::*;

use header::{Header, HEADER_SIZE};
use serializer::{
    be_slice_to_u16, calculate_size_of_serialized_string_and_length_bytes, slice_to_ip_address,
    slice_to_u32, slice_to_u64, SerializeString,
};
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

#[derive(Debug, PartialEq)]
pub struct Version {
    pub header: Header,
    pub data: VersionMessageData,
}

const IP_DATA_SIZE: usize = 26;

const DATA_SIZE_WITHOUT_CLIENT_NAME: usize = size_of::<u32>()
    + size_of::<u64>()
    + size_of::<u64>()
    + 2 * IP_DATA_SIZE
    + size_of::<u64>()
    + size_of::<u32>()
    + size_of::<bool>();

impl Version {
    pub fn new(
        network_type: NetworkType,
        unix_timestamp: u64,
        target_ip_address: &str,
        port: u16,
        nonce: u64,
        client_name: &str,
    ) -> Result<Self, IntegerParsingFailure> {
        const PROTOCOL_VERSION: u32 = 70015;

        const NODE_NETWORK: u64 = 1;
        const NODE_BLOOM: u64 = 4;
        const LOCAL_NODE_SERVICES: u64 = NODE_NETWORK | NODE_BLOOM;

        let octets = string_to_ip(target_ip_address);
        if octets.is_none() {
            return Err(IntegerParsingFailure);
        }
        let octets = octets.unwrap();

        Ok(Version {
            header: Header {
                network_type,
                command: "version".to_string(),
                message_size: 0,
                hash: [0; 4],
            },
            data: VersionMessageData {
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
            },
        })
    }
}

impl NetworkSerializable<IpData> for IpData {
    fn from_network_bytes(bytes: &[u8]) -> Result<IpData, NetworkSerializationError> {
        if bytes.len() < IP_DATA_SIZE {
            return Err(NetworkSerializationError::BufferTooShort);
        }

        let node_services =
            slice_to_u64(&bytes[0..8]).ok_or(NetworkSerializationError::UnknownBytes)?;
        let ip_address =
            slice_to_ip_address(&bytes[8..24]).ok_or(NetworkSerializationError::UnknownBytes)?;
        let port =
            be_slice_to_u16(&bytes[24..26]).ok_or(NetworkSerializationError::UnknownBytes)?;

        Ok(IpData {
            node_services,
            ip_address,
            port,
        })
    }

    fn to_network_bytes(&self) -> Result<Vec<u8>, NetworkSerializationError> {
        let mut buffer = Vec::new();

        buffer.reserve(size_of::<IpData>());

        buffer.extend_from_slice(&self.node_services.to_le_bytes());
        buffer.extend_from_slice(self.ip_address.as_slice());
        buffer.extend_from_slice(&self.port.to_be_bytes());

        Ok(buffer)
    }
}

impl NetworkSerializable<VersionMessageData> for VersionMessageData {
    fn from_network_bytes(bytes: &[u8]) -> Result<VersionMessageData, NetworkSerializationError> {
        if bytes.len() < DATA_SIZE_WITHOUT_CLIENT_NAME + 1 {
            return Err(NetworkSerializationError::BufferTooShort);
        }

        let protocol_version =
            slice_to_u32(&bytes[0..4]).ok_or(NetworkSerializationError::UnknownBytes)?;
        let local_node_services =
            slice_to_u64(&bytes[4..12]).ok_or(NetworkSerializationError::UnknownBytes)?;
        let unix_timestamp =
            slice_to_u64(&bytes[12..20]).ok_or(NetworkSerializationError::UnknownBytes)?;
        let node_ip_data = IpData::from_network_bytes(&bytes[20..46])?;
        let our_ip_data = IpData::from_network_bytes(&bytes[46..72])?;
        let nonce = slice_to_u64(&bytes[72..80]).ok_or(NetworkSerializationError::UnknownBytes)?;

        let result = String::from_dogecoin_bytes(&bytes[80..bytes.len()])?;
        let client_name = result.value;

        let offset = 80 + result.bytes_read;
        let node_starting_height = slice_to_u32(&bytes[offset..offset + 4])
            .ok_or(NetworkSerializationError::UnknownBytes)?;
        let relay_transactions = bytes[offset + 4] != 0;

        Ok(VersionMessageData {
            protocol_version,
            local_node_services,
            unix_timestamp,
            node_ip_data,
            our_ip_data,
            nonce,
            client_name,
            node_starting_height,
            relay_transactions,
        })
    }

    fn to_network_bytes(&self) -> Result<Vec<u8>, NetworkSerializationError> {
        let mut buffer = Vec::new();

        let serialized_string_and_length_bytes_size =
            calculate_size_of_serialized_string_and_length_bytes(self.client_name.len())?;

        let message_size = DATA_SIZE_WITHOUT_CLIENT_NAME + serialized_string_and_length_bytes_size;
        buffer.reserve(message_size);

        buffer.extend_from_slice(&self.protocol_version.to_le_bytes());
        buffer.extend_from_slice(&self.local_node_services.to_le_bytes());
        buffer.extend_from_slice(&self.unix_timestamp.to_le_bytes());
        buffer.extend_from_slice(&self.node_ip_data.to_network_bytes()?);
        buffer.extend_from_slice(&self.our_ip_data.to_network_bytes()?);
        buffer.extend_from_slice(&self.nonce.to_le_bytes());
        buffer.extend_from_slice(self.client_name.to_dogecoin_bytes()?.as_ref());
        buffer.extend_from_slice(&self.node_starting_height.to_le_bytes());
        buffer.push(self.relay_transactions.into());

        Ok(buffer)
    }
}

impl NetworkSerializable<Version> for Version {
    fn from_network_bytes(bytes: &[u8]) -> Result<Version, NetworkSerializationError> {
        if bytes.len() < HEADER_SIZE + DATA_SIZE_WITHOUT_CLIENT_NAME {
            return Err(NetworkSerializationError::UnknownBytes);
        }

        let header = Header::from_network_bytes(&bytes[0..HEADER_SIZE])?;
        let data = VersionMessageData::from_network_bytes(&bytes[HEADER_SIZE..bytes.len()])?;

        Ok(Version { header, data })
    }

    fn to_network_bytes(&self) -> Result<Vec<u8>, NetworkSerializationError> {
        let data_bytes = self.data.to_network_bytes()?;
        let mut bytes = self.header.to_network_bytes(&data_bytes)?;
        bytes.extend(data_bytes);

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_VERSION_MESSAGE: [u8; 129] = [
        0xfc, 0xc1, 0xb7, 0xdc, 0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x69, 0x00, 0x00, 0x00, 0xa2, 0xbb, 0x58, 0x1c, 0x7f, 0x11, 0x01, 0x00, 0x05, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x51, 0x66, 0x34, 0x64, 0x00, 0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xff, 0xff, 0x34, 0x4d, 0xe7, 0x29, 0xae, 0x0c, 0x05, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x45, 0xdf, 0x74, 0xaf, 0xc8, 0x94, 0x63, 0xf8, 0x13,
        0x2f, 0x53, 0x68, 0x69, 0x62, 0x65, 0x74, 0x6f, 0x73, 0x68, 0x69, 0x3a, 0x31, 0x2e, 0x31,
        0x34, 0x2e, 0x36, 0x2f, 0x00, 0x00, 0x00, 0x00, 0x01,
    ];

    #[test]
    fn test_fill_version_message_data() {
        let data = Version::new(NetworkType::Test, 0, "invalid ip", 0, 0, "");
        assert!(data.is_err());
        assert_eq!(data.unwrap_err(), IntegerParsingFailure);

        let data = Version::new(
            NetworkType::Test,
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
            data.unwrap().data
        );
    }

    #[test]
    fn test_build_version_message() {
        let data = Version::new(
            NetworkType::Test,
            1681155665,
            "52.77.231.41",
            44556,
            17898312933758525253,
            "/Shibetoshi:1.14.6/",
        );
        assert!(data.is_ok());
        let data = data.unwrap();

        let bytes = data.to_network_bytes();
        assert!(bytes.is_ok());
        assert_eq!(&FULL_VERSION_MESSAGE, bytes.unwrap().as_slice());
    }

    #[test]
    fn test_parse_version_message() {
        let data = Version::from_network_bytes(&FULL_VERSION_MESSAGE).unwrap();

        let header = data.header;
        let data = data.data;

        assert_eq!(header.network_type, NetworkType::Test);
        assert_eq!(header.command, "version");
        assert_eq!(header.message_size, 105);
        assert_eq!(header.hash, [0xa2, 0xbb, 0x58, 0x1c]);

        assert_eq!(data.protocol_version, 70015);
        assert_eq!(data.local_node_services, 5);
        assert_eq!(data.unix_timestamp, 1681155665);

        assert_eq!(data.node_ip_data.node_services, 1);
        assert_eq!(
            data.node_ip_data.ip_address,
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 52, 77, 231, 41]
        );
        assert_eq!(data.node_ip_data.port, 44556);

        assert_eq!(data.our_ip_data.node_services, 5);
        assert_eq!(data.our_ip_data.ip_address, [0; 16]);
        assert_eq!(data.our_ip_data.port, 0);

        assert_eq!(data.nonce, 17898312933758525253);
        assert_eq!(data.client_name, "/Shibetoshi:1.14.6/");
        assert_eq!(data.node_starting_height, 0);
        assert_eq!(data.relay_transactions, true);
    }
}
