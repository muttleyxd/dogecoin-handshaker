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

pub trait SerializeString {
    fn to_dogecoin_bytes(
        &self,
    ) -> Result<Vec<u8>, CalculateSizeOfSerializedStringAndLengthBytesError>;
}

impl SerializeString for String {
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
}
