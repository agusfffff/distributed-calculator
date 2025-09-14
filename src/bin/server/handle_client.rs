use std::{io::{BufRead, BufReader, Read, Write}, sync::Arc, str::FromStr};

use distributed_calculator::protocol::Protocol;

use crate::{calculator::Calculator, operation::Operation, server_error::ServerError};


pub fn handle_connection<RW: Read + Write>(mut stream: RW, calculator: Arc<std::sync::Mutex<Calculator>>)-> Result<(), ServerError> {
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
            Protocol::Operation(args) => handle_operation_message(&calculator, reader.get_mut(), args),
            Protocol::Get => handle_get_message(&calculator, reader.get_mut()),
            _ => send_protocol(Protocol::ErrorOperation(format!("unexpected message: {}", protocol.to_string()).to_string()), reader.get_mut()),
        }?;

    }
}


fn send_protocol<RW: Read + Write>(protocol: Protocol, stream: &mut RW) -> Result<(), ServerError> {
    let response = protocol.to_bytes();
    stream.write_all(&response).map_err(|_| ServerError::WriteFailed)?;
    Ok(())
}

fn handle_operation_message<RW: Read + Write>(calculator: &Arc<std::sync::Mutex<Calculator>>, stream : &mut RW, args: String) -> Result<(), ServerError> {
    let op = match Operation::from_str(&args) {
        Ok(op) => op,
        Err(e) => {
            return send_protocol(Protocol::ErrorOperation(e.to_string()), stream); 
        }
    };
    apply_operation(&calculator, op)?;
    send_protocol(Protocol::Ok, stream)?;
    Ok(())
}

fn apply_operation(calculator: &Arc<std::sync::Mutex<Calculator>>, operation: Operation) -> Result<(), ServerError> {
    match calculator.lock() {
        Ok(mut calc) => {
            calc.apply(operation);
            Ok(())
        },
        Err(_) => Err(ServerError::PoisonError)
    }
}


fn handle_get_message<RW: Read + Write>(calculator: &Arc<std::sync::Mutex<Calculator>>, stream : &mut RW) -> Result<(), ServerError> {
    let value = get_value(&calculator)?;
    send_protocol(Protocol::Value(value.to_string()), stream)?; 
    Ok(())
}

fn get_value(calculator: &Arc<std::sync::Mutex<Calculator>>) -> Result<u8, ServerError> {
    match calculator.lock() {
        Ok(calc) => { 
            Ok(calc.accumulation())
        }, 
        Err(_) => Err(ServerError::PoisonError)
    }
}

#[cfg(test)]
mod tests {
    use std::{io::{Cursor, Read}, sync::Arc};

    use distributed_calculator::protocol::Protocol;

    use crate::{calculator::Calculator, handle_client::{get_value, handle_get_message}};

    #[test]
    fn get_value_of_calculator() {        
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let value = get_value(&calculator).unwrap();
      
        assert_eq!(value, 0);
    }

    #[test]
    fn send_get_message() { 
        let response = Protocol::Value("0".to_string());
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let mut cursor = Cursor::new(Vec::new()); 
        let mut output = String::new();
        
        handle_get_message( &calculator, &mut cursor).unwrap(); 
        cursor.set_position(0);
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response.to_string());
    }

    #[test]
    fn apply_operation_success(){ 
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        
        let mut calc = calculator.lock().unwrap();
        calc.apply(crate::operation::Operation::Add(5));
        let value = calc.accumulation();
        
        assert_eq!(value, 5);
    }

    #[test]
    fn handle_operation_message_ok() { 
        let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));
        let mut cursor = Cursor::new(Vec::new()); 
        let args = "+ 5".to_string();
        let response = Protocol::Ok;

        crate::handle_client::handle_operation_message(&calculator, &mut cursor, args).unwrap(); 
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
        let response = Protocol::ErrorOperation(("parsing error: unknown operation: %").to_string());

        crate::handle_client::handle_operation_message(&calculator, &mut cursor, args).unwrap(); 
        cursor.set_position(0);
        let mut output = String::new();
        cursor.read_to_string(&mut output).unwrap();

        assert_eq!(output, response.to_string());
    }
}