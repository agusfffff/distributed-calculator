//! Modulo de manejo de clientes conectados al servidor.
use std::{
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
    sync::Arc,
};

use distributed_calculator::protocol::Protocol;

use crate::{calculator::Calculator, operation::Operation, server_error::ServerError};

/// Maneja la conexión con un cliente.
/// Lee mensajes del cliente, los procesa y envía respuestas.
/// Recibe un stream de lectura/escritura y una referencia al calculadora compartida.
/// Devuelve un resultado indicando éxito o error.
///
/// # Errores
/// - `ServerError::ReadFailed`: Si falla la lectura del stream.
pub fn handle_connection<RW: Read + Write>(
    mut stream: RW,
    calculator: Arc<std::sync::Mutex<Calculator>>,
) -> Result<(), ServerError> {
    let mut buf = String::new();
    let mut reader = BufReader::new(&mut stream);

    loop {
        buf.clear();

        let bytes_read_result = reader.read_line(&mut buf);

        match bytes_read_result {
            Ok(n) => {
                if n == 0 {
                    // acá se podría logear
                    return Ok(());
                }
            }
            Err(_) => {
                return Err(ServerError::ReadFailed);
            }
        };

        let protocol = Protocol::from_bytes(buf.trim_end().as_bytes());

        match protocol {
            Protocol::Operation(args) => {
                handle_operation_message(&calculator, reader.get_mut(), args)
            }
            Protocol::Get => handle_get_message(&calculator, reader.get_mut()),
            _ => send_protocol(
                Protocol::ErrorOperation(format!("unexpected message: {}", protocol).to_string()),
                reader.get_mut(),
            ),
        }?;
    }
}

/// Envía un mensaje de protocolo al cliente a través del stream.
/// Recibe el protocolo y el stream.
/// Devuelve un resultado indicando éxito o error.
///
/// # Errores
/// - `ServerError::WriteFailed`: Si falla la escritura en el stream.
fn send_protocol<RW: Read + Write>(protocol: Protocol, stream: &mut RW) -> Result<(), ServerError> {
    let response = protocol.to_bytes();
    stream
        .write_all(&response)
        .map_err(|_| ServerError::WriteFailed)?;
    Ok(())
}

/// Maneja un mensaje de operación recibido del cliente.
/// Parsea la operación, la aplica a la calculadora y envía una respuesta.
/// Recibe la calculadora compartida, el stream y los argumentos de la operación.
/// Devuelve un resultado indicando éxito o error.
///
/// #Errores
/// Asociados a el parseo de la Operacion o a la aplicación de la Operación.
fn handle_operation_message<RW: Read + Write>(
    calculator: &Arc<std::sync::Mutex<Calculator>>,
    stream: &mut RW,
    args: String,
) -> Result<(), ServerError> {
    let op = match Operation::from_str(&args) {
        Ok(op) => op,
        Err(e) => {
            return send_protocol(Protocol::ErrorOperation(e.to_string()), stream);
        }
    };
    apply_operation(calculator, op)?;
    send_protocol(Protocol::Ok, stream)?;
    Ok(())
}

/// Aplica operación a una calculadora.
/// Recibe la calculadra y la operación.
/// Devuelve un resultado indicando éxito o error.
///
/// #Errores
/// `Error::PosionError` - En el caso de que se envenene el lock, se termina la conexión.
fn apply_operation(
    calculator: &Arc<std::sync::Mutex<Calculator>>,
    operation: Operation,
) -> Result<(), ServerError> {
    match calculator.lock() {
        Ok(mut calc) => {
            calc.apply(operation);
            Ok(())
        }
        Err(_) => Err(ServerError::PoisonError),
    }
}

/// Calcula el valor actual de la calculadora y envia el protocolo de get al cliente .
/// Recibe la calculadora y el stream.
/// Devuelve un resultado indicando éxito o error.
///
/// #Errores
/// Asociados a la aplicación de las funciones.
fn handle_get_message<RW: Read + Write>(
    calculator: &Arc<std::sync::Mutex<Calculator>>,
    stream: &mut RW,
) -> Result<(), ServerError> {
    let value = get_value(calculator)?;
    send_protocol(Protocol::Value(value.to_string()), stream)?;
    Ok(())
}

///Aplica la operación de pedirle la acumulación a la calculadora
/// Recibe la calculadora y la lockea para poder acceder a sus datos.
/// Devuelve un resultado indicando éxito o error.
///
/// #Errores
/// `Error::PosionError` - En el caso de que se envenene el lock, se termina la conexión.
fn get_value(calculator: &Arc<std::sync::Mutex<Calculator>>) -> Result<u8, ServerError> {
    match calculator.lock() {
        Ok(calc) => Ok(calc.accumulation()),
        Err(_) => Err(ServerError::PoisonError),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Cursor, Read, Write},
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
        thread,
    };

    use distributed_calculator::protocol::Protocol;

    use crate::{
        calculator::Calculator,
        handle_client::{
            apply_operation, get_value, handle_connection, handle_get_message,
            handle_operation_message, send_protocol,
        },
    };

    #[test]
    fn get_value_of_calculator() {
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let value = get_value(&calculator).unwrap();

        assert_eq!(value, 0);
    }

    #[test]
    fn send_get_message() {
        let response = Protocol::Value("0".to_string()).to_string();
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let mut cursor = Cursor::new(Vec::new());
        let mut output = String::new();

        handle_get_message(&calculator, &mut cursor).unwrap();
        cursor.set_position(0);
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response);
    }

    #[test]
    fn apply_operation_success() {
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let op = crate::operation::Operation::Add(5);

        apply_operation(&calculator, op).unwrap();

        assert_eq!(calculator.lock().unwrap().accumulation(), 5);
    }

    #[test]
    fn handle_operation_message_ok() {
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let mut cursor = Cursor::new(Vec::new());
        let args = "+ 5".to_string();
        let response = Protocol::Ok;

        handle_operation_message(&calculator, &mut cursor, args).unwrap();
        cursor.set_position(0);
        let mut output = String::new();
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response.to_string());
        assert_eq!(calculator.lock().unwrap().accumulation(), 5);
    }

    #[test]
    fn handle_operation_message_error() {
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let mut cursor = Cursor::new(Vec::new());
        let args = "% 5".to_string();
        let response =
            Protocol::ErrorOperation(("parsing error: unknown operation: %").to_string())
                .to_string();

        handle_operation_message(&calculator, &mut cursor, args).unwrap();
        cursor.set_position(0);
        let mut output = String::new();
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response);
    }

    #[test]
    fn test_send_protocol() {
        let mut cursor = Cursor::new(Vec::new());
        let protocol = Protocol::Ok;
        let response = Protocol::Ok.to_string();

        send_protocol(protocol, &mut cursor).unwrap();
        cursor.set_position(0);
        let mut output = String::new();
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response);
    }

    #[test]
    fn integration_test_handle_connection() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"OPERATION + 1\nGET\n").unwrap();
        client.flush().unwrap();

        let mut reader = BufReader::new(client);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("OK"));

        buf.clear();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("VALUE 1"));
    }

    #[test]
    fn integration_test_handle_connection_unexpected_message() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"hola\nGET\n").unwrap();
        client.flush().unwrap();

        let mut reader = BufReader::new(client);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("ERROR \"unexpected message: hola\""));

        buf.clear();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("VALUE 0"));
    }

    #[test]
    fn integration_test_handle_connection_unknown_operation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"OPERATION 8 8\nGET\n").unwrap();
        client.flush().unwrap();

        let mut reader = BufReader::new(client);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        println!("buf: {}", buf);

        assert!(buf.contains("ERROR \"parsing error: unknown operation: 8\""));

        buf.clear();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("VALUE 0"));
    }

    #[test]
    fn integration_test_handle_connection_too_large_integer_operation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"OPERATION + 300\nGET\n").unwrap();
        client.flush().unwrap();

        let mut reader = BufReader::new(client);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        println!("buf: {}", buf);

        assert!(buf.contains(
            "ERROR \"parsing error: invalid integer: number too large to fit in target type\""
        ));

        buf.clear();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("VALUE 0"));
    }

    #[test]
    fn integration_test_handle_connection_invalid_digit_operation() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator).unwrap();
        });

        let mut client = TcpStream::connect(addr).unwrap();
        client.write_all(b"OPERATION + cinco\nGET\n").unwrap();
        client.flush().unwrap();

        let mut reader = BufReader::new(client);
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        println!("buf: {}", buf);

        assert!(
            buf.contains("ERROR \"parsing error: invalid integer: invalid digit found in string\"")
        );

        buf.clear();
        reader.read_line(&mut buf).unwrap();

        assert!(buf.contains("VALUE 0"));
    }

    #[test]
    fn test_client_disconnects() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let calculator = Arc::new(Mutex::new(Calculator::new()));
        let handle = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle_connection(stream, calculator)
        });

        let client = TcpStream::connect(addr).unwrap();
        drop(client);

        let result = handle.join().unwrap();
        assert!(matches!(result, Ok(())));
    }
}
