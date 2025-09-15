//! Representa los distintos errores que pueden ocurrir en el programa.
//!
/// Cada variante del enum representa un caso de especifico de error que puede
/// ocurrir durante la ejecución.

#[derive(Debug)]

pub enum ClientError {
    ///Error por falta de un argumento
    MissingArgument,
    ///Error por argumento invalido
    InvalidArgument,
    ///Error al conectar con el servidor
    FailedConnection,
    ///Error al leer
    FailToReadLine,
    ///Error al escribir
    FailedWrite,
    ///Error al recibir un mensaje incorrectamente del servidor
    ErrorMessage,
    ///Mensaje de error recibido del servidor
    ServerErrorMessage(String)
}

impl ClientError {
    /// Devuelve un mensaje de error descriptivo para cada variante del ClientError Enum.
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
    /// Imprime el error en un formato legible.
    /// Ejemplo: Error: A required argument is missing.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR \"{}\"", self.message())
    }
}
