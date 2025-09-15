use std::{io::{BufRead, BufReader, Read, Write}, net::{SocketAddr, TcpStream}, str::FromStr};

use distributed_calculator::protocol::Protocol;

use crate::client_error::ClientError;

pub fn parse_address<I: IntoIterator<Item=String>>(inputs: I) -> Result<SocketAddr, ClientError>{
    let mut iter = inputs.into_iter();
    iter.next();
    let ip_str = iter.next().ok_or(ClientError::MissingArgument)?;
    let addr = SocketAddr::from_str(&ip_str).map_err(|_| ClientError::InvalidArgument)?;
    Ok(addr)
}

pub fn process_files<R: BufRead>(addr : SocketAddr, file_reader : R) -> Result<(), ClientError> {
    let stream = TcpStream::connect(addr).map_err(|_| ClientError::FailedConnection)?;
    process_files_with_stream(file_reader, stream)
}
    
fn process_files_with_stream<R: BufRead, W: Write + Read>(mut file_reader: R, stream:  W) -> Result<(), ClientError> {
    let mut reader = BufReader::new(stream);  
    let mut line_buf = String::new();
    let mut server_buf = String::new();
    
    loop {
            line_buf.clear();  
            let bytes_read_result: Result<usize, std::io::Error> = file_reader.read_line(&mut line_buf);
            match bytes_read_result {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                }, 
                Err(_) => {
                    eprintln!("{}", ClientError::FailToReadLine);
                    continue;
                }
            };
    
            let bytes = line_buf.as_bytes();
    
            write_to_addr(reader.get_mut(), bytes)?;
            receive_response(&mut reader, &mut server_buf)?;
    
            server_buf.clear();
    
    }
    write_to_addr(reader.get_mut(), &Protocol::Get.to_bytes())?;
    last_value_of_calculator(&mut reader, &mut server_buf)?;
    
    Ok(())
    }


fn receive_response<R: BufRead>(reader : &mut R, server_buf : &mut String ) -> Result<(), ClientError> {
    let response_bytes_result = reader.read_line(server_buf);
    match response_bytes_result {
        Ok(n) => {
            if n == 0 {
                return Err(ClientError::FailedConnection);
            }
        }, 
        Err(_) => {
            return Err(ClientError::FailedConnection);
        }
    };

    let protocol: Protocol = Protocol::from_bytes(server_buf.trim_end().as_bytes());
    if let Protocol::ErrorOperation(message) = protocol {
        eprintln!("{}", ClientError::ServerErrorMessage(message));
    }
    Ok(())
}

fn write_to_addr<W: Write>(writer : &mut W, bytes : &[u8]) -> Result<(), ClientError> { 
    writer.write_all(bytes).map_err(|_| ClientError::FailedWrite)?;
    writer.flush().map_err(|_| ClientError::FailedWrite)?;
    Ok(())
}

fn last_value_of_calculator<R: BufRead>(reader : &mut R, server_buf : &mut String) -> Result<(), ClientError> { 
    let response_bytes_result = reader.read_line(server_buf);
    match response_bytes_result {
        Ok(n) => {
           if n == 0 {
                return Err(ClientError::FailedConnection);
            }
        }, 
        Err(_) => {
            return Err(ClientError::FailedConnection);
        }
    };

    let protocol = Protocol::from_bytes(server_buf.trim_end().as_bytes());
    
    match protocol {
        Protocol::Value(val) => {
            println!("{}", val);
        },
        Protocol::ErrorOperation(message) => {
            eprintln!("{}", ClientError::ServerErrorMessage(message));
        }, 
        _ => {return Err(ClientError::ErrorMessage);}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{io::{BufReader, BufWriter, Cursor, Write}, net::SocketAddr};

    use distributed_calculator::protocol::Protocol;

    use crate::{client_error::ClientError, utils::{last_value_of_calculator, parse_address, receive_response, write_to_addr}};
     
    #[test]
    fn parsing_address_successfully() {
        let args = vec![
            "program".to_string(),
            "127.0.0.1:8080".to_string()
        ];

        let addr = parse_address(args).unwrap();
        assert_eq!(addr, "127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    }

    #[test]
    fn parse_fails_missing_argument() {
        let args = vec!["program".to_string()]; // sin IP

        let err = parse_address(args).unwrap_err();
        matches!(err, ClientError::MissingArgument);
    }

    #[test]
    fn parse_fails_invalid_address() {
        let args = vec![
            "program".to_string(), 
            "not_an_ip".to_string()
        ]; 

        let err = parse_address(args).unwrap_err();
        matches!(err, ClientError::InvalidArgument);
    }

    #[test]
    fn last_value_of_calculator_is_received() { 
        let server_response = Protocol::Value("42".to_string()).to_bytes();
        let cursor = Cursor::new(&server_response);

        let mut reader = BufReader::new(cursor);
        let mut buf = String::new();

        let result= last_value_of_calculator(&mut reader, &mut buf);
        
        assert!(result.is_ok());
        assert_eq!(buf.as_bytes(), server_response.as_slice());

    }

    #[test]
    fn last_value_of_calculator_recieves_wrong_message() { 
        let server_response = Protocol::Ok.to_bytes();
        let cursor = Cursor::new(&server_response);

        let mut reader = BufReader::new(cursor);
        let mut buf = String::new();

        let result= last_value_of_calculator(&mut reader, &mut buf).unwrap_err();
        
        matches!(result, ClientError::ErrorMessage);
    }

    #[test]
    fn last_value_of_calculator_receives_nothing() { 
        use std::io::Cursor;

        let empty_cursor = Cursor::new(Vec::new()); // nada que leer
        let mut reader = BufReader::new(empty_cursor);
        let mut buf = String::new();

        let result = last_value_of_calculator(&mut reader, &mut buf).unwrap_err();
        
        assert!(matches!(result, ClientError::FailedConnection));
    }

    #[test]
    fn write_to_addr_success() {
        let mut buffer = Cursor::new(Vec::new());
        let data = Protocol::Operation("+ 1".to_string()).to_bytes();
        { 
            let mut writer = BufWriter::new(&mut buffer);
            write_to_addr(&mut writer, &data).unwrap();
            writer.flush().unwrap();
        }

        assert_eq!(buffer.get_ref().as_slice(), data);
    }

    #[test]
    fn receive_response_ok() {
        let data = Protocol::Ok.to_bytes();  
        let cursor = Cursor::new(&data);
        let mut reader = BufReader::new(cursor);
        let mut buf = String::new();

        let result = receive_response(&mut reader, &mut buf);
        assert!(result.is_ok());
        let data_str = String::from_utf8(data).unwrap();
        assert_eq!(buf, data_str);  
    }    

    #[test]
    fn recieve_0_bytes_fails() { 
        let cursor = Cursor::new(Vec::new());
        let mut reader = BufReader::new(cursor);
        let mut buf = String::new();
        let result = receive_response(&mut reader, &mut buf).unwrap_err();
        assert!(matches!(result, ClientError::FailedConnection));
    }

}
