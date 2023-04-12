use std::time::SystemTimeError;

#[derive(Debug, PartialEq)]
pub enum CalculateSizeOfSerializedStringAndLengthBytesError {
    StringTooLong,
}

impl std::fmt::Display for CalculateSizeOfSerializedStringAndLengthBytesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[derive(Clone, Debug, PartialEq)]
pub enum HeaderBuildError {
    CommandIsEmpty,
    CommandTooLong,
    MessageSizeParseFailure,
    MessageTooLong(usize),
    TooShort,
    UnknownNetworkType,
}

impl std::fmt::Display for HeaderBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderBuildError::CommandIsEmpty => write!(f, "Command is empty"),
            HeaderBuildError::CommandTooLong => write!(f, "Command too long"),
            HeaderBuildError::MessageTooLong(value) => {
                write!(f, "Value too big for u32: {}", value)
            }
            HeaderBuildError::UnknownNetworkType => write!(f, "Unknown network type"),
            HeaderBuildError::MessageSizeParseFailure => write!(f, "Message size parse failure"),
            HeaderBuildError::TooShort => {
                write!(f, "Passed header is too short (24 bytes are required)")
            }
        }
    }
}

impl From<HeaderBuildError> for NetworkSerializationError {
    fn from(value: HeaderBuildError) -> Self {
        NetworkSerializationError::HeaderParseError(value)
    }
}

impl From<NetworkSerializationError> for HeaderBuildError {
    fn from(value: NetworkSerializationError) -> Self {
        match value {
            NetworkSerializationError::BufferTooShort => HeaderBuildError::TooShort,
            NetworkSerializationError::UnknownBytes => HeaderBuildError::MessageSizeParseFailure,
            NetworkSerializationError::HeaderParseError(e) => e,
            NetworkSerializationError::StringParseError => HeaderBuildError::CommandTooLong,
        }
    }
}

impl std::error::Error for HeaderBuildError {}

#[derive(Clone, Debug)]
pub enum NetworkSerializationError {
    BufferTooShort,
    UnknownBytes,
    HeaderParseError(HeaderBuildError),
    StringParseError,
}

impl std::fmt::Display for NetworkSerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkSerializationError::BufferTooShort => write!(f, "Buffer too short"),
            NetworkSerializationError::UnknownBytes => write!(f, "Unknown bytes"),
            NetworkSerializationError::HeaderParseError(e) => {
                write!(f, "Header parse error: {}", e)
            }
            NetworkSerializationError::StringParseError => write!(f, "String parse error"),
        }
    }
}

impl std::error::Error for NetworkSerializationError {}

#[derive(Debug, PartialEq)]
pub struct IntegerParsingFailure;

impl std::fmt::Display for IntegerParsingFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Integer parsing failure")
    }
}

impl std::error::Error for IntegerParsingFailure {}

impl From<IntegerParsingFailure> for NetworkSerializationError {
    fn from(_: IntegerParsingFailure) -> Self {
        NetworkSerializationError::StringParseError
    }
}

#[derive(Debug)]
pub enum NodeConnectionAgentError {
    FailedCreatingUnixTimestamp(SystemTimeError),
    HeaderBuildFailure(HeaderBuildError),
    IncorrectNumberOfBytesReceived,
    IncorrectNumberOfBytesSent,
    IncorrectResponse,
    IntegerParsingFailure,
    IoError(std::io::Error),
    NetworkSerializationFailure(NetworkSerializationError),
    UnexpectedCommand(String, String),
}

impl std::fmt::Display for NodeConnectionAgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeConnectionAgentError::FailedCreatingUnixTimestamp(e) => {
                write!(f, "Failed creating unix timestamp: {}", e)
            }
            NodeConnectionAgentError::HeaderBuildFailure(e) => {
                write!(f, "Header build error: {}", e)
            }
            NodeConnectionAgentError::IncorrectNumberOfBytesReceived => {
                write!(f, "Incorrect number of bytes received")
            }
            NodeConnectionAgentError::IncorrectNumberOfBytesSent => {
                write!(f, "Incorrect number of bytes sent")
            }
            NodeConnectionAgentError::IncorrectResponse => write!(f, "Incorrect response"),
            NodeConnectionAgentError::IntegerParsingFailure => write!(f, "Integer parsing failure"),
            NodeConnectionAgentError::IoError(e) => write!(f, "I/O error: {}", e),
            NodeConnectionAgentError::NetworkSerializationFailure(e) => {
                write!(f, "Network serialization failure: {}", e)
            }
            NodeConnectionAgentError::UnexpectedCommand(expected, actual) => write!(
                f,
                "Unexpected command, expected: '{}', actual: '{}",
                expected, actual
            ),
        }
    }
}

impl From<SystemTimeError> for NodeConnectionAgentError {
    fn from(value: SystemTimeError) -> Self {
        NodeConnectionAgentError::FailedCreatingUnixTimestamp(value)
    }
}

impl From<IntegerParsingFailure> for NodeConnectionAgentError {
    fn from(_: IntegerParsingFailure) -> Self {
        NodeConnectionAgentError::IntegerParsingFailure
    }
}

impl From<NetworkSerializationError> for NodeConnectionAgentError {
    fn from(value: NetworkSerializationError) -> Self {
        NodeConnectionAgentError::NetworkSerializationFailure(value)
    }
}

impl From<std::io::Error> for NodeConnectionAgentError {
    fn from(value: std::io::Error) -> Self {
        NodeConnectionAgentError::IoError(value)
    }
}

impl From<HeaderBuildError> for NodeConnectionAgentError {
    fn from(value: HeaderBuildError) -> Self {
        NodeConnectionAgentError::HeaderBuildFailure(value)
    }
}

impl std::error::Error for NodeConnectionAgentError {}
