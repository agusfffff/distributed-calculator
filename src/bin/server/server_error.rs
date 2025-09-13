#[derive(Debug)]

pub enum ServerError {
    MissingArgument,
    InvalidArgument,
    FailedConnection,
    BindFailed,
    WriteFailed, 
    PoisonError,
    ReadFailed,
}

impl ServerError {
    pub fn message(&self) -> &str {
        match self {
            ServerError::MissingArgument => "A required argument is missing.",
            ServerError::InvalidArgument => "An argument provided is invalid.",
            ServerError::FailedConnection => "Incoming connection failed.",
            ServerError::BindFailed => "Failed to bind to the specified address.",
            ServerError::WriteFailed => "Failed to write to the stream.",
            ServerError::PoisonError => "Failed to acquire lock on the calculator.",
            ServerError::ReadFailed => "Failed to read from the stream.",
        }
    }
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR \"{}\"", self.message())
    }
}
