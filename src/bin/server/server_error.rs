//! Representa los distintos errores que pueden ocurrir en el programa.
//!
/// Cada variante del enum representa un caso de especifico de error que puede
/// ocurrir durante la ejecución.

#[derive(Debug)]

pub enum ServerError {
    ///Error por falta de un argumento
    MissingArgument,
    ///Error por argumento invalido   
    InvalidArgument,
    ///Error al conectar con el cliente
    FailedConnection,
    ///Error al conectar el socket
    BindFailed,
    ///Error al escribir
    WriteFailed,
    ///Error de lock envenenando
    PoisonError,
    ///Error de lectura
    ReadFailed,
}

impl ServerError {
    /// Devuelve un mensaje de error descriptivo para cada variante del ServerError Enum.
    pub fn message(&self) -> &str {
        match self {
            ServerError::MissingArgument => "A required argument is missing.",
            ServerError::InvalidArgument => "An argument provided is invalid.",
            ServerError::FailedConnection => "Incoming connection failed.",
            ServerError::BindFailed => "Failed to bind to the specified address.",
            ServerError::WriteFailed => "Failed to write to the stream.",
            ServerError::PoisonError => "Failed to acquire lock on the calculator -> poisoned.",
            ServerError::ReadFailed => "Failed to read from the stream.",
        }
    }
}

impl std::fmt::Display for ServerError {
    /// Imprime el error en un formato legible.
    /// Ejemplo: Error: A required argument is missing.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ERROR \"{}\"", self.message())
    }
}
