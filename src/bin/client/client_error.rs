#[derive(Debug)]

pub enum ClientError {
    MissingArgument,
    InvalidArgument,
    FailedConnection,
    FailToReadLine,
    FailedWrite,
    ErrorMessage,
    ServerErrorMessage(String)
}

impl ClientError {
    pub fn message(&self) -> &str {
        match self {
            ClientError::MissingArgument => "A required argument is missing.",
            ClientError::InvalidArgument => "An argument provided is invalid.",
            ClientError::FailedConnection => "Incoming connection failed.",
            ClientError::FailToReadLine => "Failed to read a line from the input.",
            ClientError::FailedWrite => "Failed to write to the server.",
            ClientError::ErrorMessage => "Received a message incorrectly from the server.",
            ClientError::ServerErrorMessage(msg) => msg,
        }
    }
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR \"{}\"", self.message())
    }
}
