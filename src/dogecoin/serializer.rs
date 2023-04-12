use crate::dogecoin::messages::version::IpAddress;
use crate::dogecoin::{IntegerParsingFailure, NetworkSerializationError};
use std::fmt::{Display, Formatter};
use std::mem::size_of;

#[derive(Debug, PartialEq)]
pub enum CalculateSizeOfSerializedStringAndLengthBytesError {
    StringTooLong,
}

impl Display for CalculateSizeOfSerializedStringAndLengthBytesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalculateSizeOfSerializedStringAndLengthBytesError::StringTooLong => {
                write!(f, "String too long to be serialized")
            }
        }
    }
}

impl std::error::Error for CalculateSizeOfSerializedStringAndLengthBytesError {}

impl From<CalculateSizeOfSerializedStringAndLengthBytesError> for NetworkSerializationError {
    fn from(value: CalculateSizeOfSerializedStringAndLengthBytesError) -> Self {
        match value {
            CalculateSizeOfSerializedStringAndLengthBytesError::StringTooLong => {
                NetworkSerializationError::StringParseError
            }
        }
    }
}

pub fn slice_to_ip_address(slice: &[u8]) -> Option<IpAddress> {
    slice.try_into().ok()
}

pub fn be_slice_to_u16(slice: &[u8]) -> Option<u16> {
    Some(u16::from_be_bytes(slice.try_into().ok()?))
}

pub fn slice_to_u16(slice: &[u8]) -> Option<u16> {
    Some(u16::from_le_bytes(slice.try_into().ok()?))
}

pub fn slice_to_u32(slice: &[u8]) -> Option<u32> {
    Some(u32::from_le_bytes(slice.try_into().ok()?))
}

pub fn slice_to_u64(slice: &[u8]) -> Option<u64> {
    Some(u64::from_le_bytes(slice.try_into().ok()?))
}

pub fn slice_to_string(slice: &[u8]) -> String {
    slice
        .iter()
        .filter_map(|byte| match *byte {
            0 => None,
            _ => Some(*byte as char),
        })
        .collect()
}

pub fn calculate_size_of_serialized_string_and_length_bytes(
    length: usize,
) -> Result<usize, CalculateSizeOfSerializedStringAndLengthBytesError> {
    if length < u8::MAX as usize {
        Ok(length + size_of::<u8>())
    } else if length < u16::MAX as usize {
        Ok(length + size_of::<u16>() + 1)
    } else if length < u32::MAX as usize {
        Ok(length + size_of::<u32>() + 1)
    } else if length < (u64::MAX - 1) as usize {
        Ok(length + size_of::<u64>() + 1)
    } else {
        Err(CalculateSizeOfSerializedStringAndLengthBytesError::StringTooLong)
    }
}

#[derive(PartialEq)]
pub struct SerializedStringResult {
    pub bytes_read: usize,
    pub value: String,
}

pub trait SerializeString {
    fn from_dogecoin_bytes(slice: &[u8]) -> Result<SerializedStringResult, IntegerParsingFailure>;

    fn to_dogecoin_bytes(
        &self,
    ) -> Result<Vec<u8>, CalculateSizeOfSerializedStringAndLengthBytesError>;
}

impl SerializeString for String {
    fn from_dogecoin_bytes(slice: &[u8]) -> Result<SerializedStringResult, IntegerParsingFailure> {
        let mut offset: usize = 0;
        let length: usize;

        let first_byte = slice[0];
        if first_byte == 0 {
            return Ok(SerializedStringResult {
                bytes_read: offset,
                value: "".to_string(),
            });
        } else if first_byte < 253 {
            offset = 1;
            length = first_byte as usize;
        } else if first_byte == 253 {
            length = slice_to_u16(&slice[1..3]).ok_or(IntegerParsingFailure)? as usize;
            offset = 3;
        } else if first_byte == 254 {
            length = slice_to_u32(&slice[1..5]).ok_or(IntegerParsingFailure)? as usize;
            offset = 5;
        } else {
            length = slice_to_u64(&slice[1..9]).ok_or(IntegerParsingFailure)? as usize;
            offset = 9;
        }

        Ok(SerializedStringResult {
            bytes_read: offset + length,
            value: slice_to_string(&slice[offset..offset + length]),
        })
    }

    fn to_dogecoin_bytes(
        &self,
    ) -> Result<Vec<u8>, CalculateSizeOfSerializedStringAndLengthBytesError> {
        let mut buffer = Vec::new();

        let length = self.len();

        buffer.reserve(calculate_size_of_serialized_string_and_length_bytes(
            length,
        )?);

        if length < u8::MAX as usize {
            let length = length as u8;
            buffer.extend_from_slice(&length.to_le_bytes());
            if length > 0 {
                buffer.extend_from_slice(self.as_bytes());
            }
        } else if length < u16::MAX as usize {
            let length = length as u16;
            buffer.push(253);
            buffer.extend_from_slice(&length.to_le_bytes());
            buffer.extend_from_slice(self.as_bytes());
        } else if length < u32::MAX as usize {
            let length = length as u32;
            buffer.extend_from_slice(&length.to_le_bytes());
            buffer.extend_from_slice(self.as_bytes());
            buffer.push(254);
        } else if length < (u64::MAX - 1) as usize {
            let length = length as u64;
            buffer.push(255);
            buffer.extend_from_slice(&length.to_le_bytes());
            buffer.extend_from_slice(self.as_bytes());
        } else {
            return Err(CalculateSizeOfSerializedStringAndLengthBytesError::StringTooLong);
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_size_of_serialized_string() {
        assert_eq!(
            Ok(1),
            calculate_size_of_serialized_string_and_length_bytes(0)
        );
        assert_eq!(
            Ok(17),
            calculate_size_of_serialized_string_and_length_bytes(16)
        );

        assert_eq!(
            Ok(258),
            calculate_size_of_serialized_string_and_length_bytes(u8::MAX as usize)
        );
        assert_eq!(
            Ok(65540),
            calculate_size_of_serialized_string_and_length_bytes(u16::MAX as usize)
        );
        assert_eq!(
            Ok(4294967304),
            calculate_size_of_serialized_string_and_length_bytes(u32::MAX as usize)
        );
        assert_eq!(
            Err(CalculateSizeOfSerializedStringAndLengthBytesError::StringTooLong),
            calculate_size_of_serialized_string_and_length_bytes(u64::MAX as usize)
        );
    }

    #[test]
    fn test_slice_to_string() {
        let bytes: [u8; 4] = [0, 0, 0, 0];
        assert_eq!("", slice_to_string(&bytes));

        let bytes: [u8; 4] = [b'a', 0, 0, 0];
        assert_eq!("a", slice_to_string(&bytes));

        let bytes: [u8; 4] = [b'a', b'b', b'c', b'd'];
        assert_eq!("abcd", slice_to_string(&bytes));
    }

    #[test]
    fn test_serialize_string() {
        assert_eq!(
            &[5, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8],
            "Hello".to_string().to_dogecoin_bytes().unwrap().as_slice()
        );

        assert_eq!(
            &[
                253, 10, 1, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 10, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 10, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 10, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48,
                48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48, 48
            ],
            r#"00000000000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000
0000000000000000000000000000000000000000000000000000000000000000"#
                .to_string()
                .to_dogecoin_bytes()
                .unwrap()
                .as_slice()
        );
    }

    #[test]
    fn test_deserialize_string() {
        assert_eq!(
            String::from_dogecoin_bytes(&[
                5, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8
            ])
            .unwrap()
            .value,
            "Hello"
        );
    }
}
