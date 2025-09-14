use std::{fs::File, io::{BufRead, BufReader, BufWriter, Write}, net::{SocketAddr, TcpStream}, str::FromStr};

use crate::client_error::ClientError;
use distributed_calculator::protocol::Protocol;

mod client_error;

fn main() -> Result<(), ClientError> {
    let (addr, file ) = parse_arguments(std::env::args())?;
    let reader = BufReader::new(file);
    process_files(addr, reader)?;
    Ok(())
}

fn parse_arguments<I: IntoIterator<Item=String>>(inputs: I) -> Result<(SocketAddr,File), ClientError>{
    let mut iter = inputs.into_iter();
    iter.next();
    let ip_str = iter.next().ok_or(ClientError::MissingArgument)?;
    let addr = SocketAddr::from_str(&ip_str).map_err(|_| ClientError::InvalidArgument)?;
    let file_path = iter.next().ok_or(ClientError::MissingArgument)?;
    let file: File = File::open(file_path).map_err(|_| ClientError::InvalidArgument)?;
    Ok((addr, file))
}


fn process_files<R: BufRead>(addr : SocketAddr, mut file_reader : R) -> Result<(), ClientError> {
    let stream = TcpStream::connect(addr).map_err(|_| ClientError::FailedConnection)?;
    let mut writer = BufWriter::new(&stream);
    let mut line_buf = String::new();
    let mut reader = BufReader::new(&stream);
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

            write_to_addr(&mut writer, bytes)?;
            receive_response(&mut reader, &mut server_buf)?;

            server_buf.clear();

    }
    write_to_addr(&mut writer, &Protocol::Get.to_bytes())?;
    last_value_of_calculator(&mut reader, &mut server_buf)?;

    Ok(())
}


fn receive_response (reader : &mut BufReader<&TcpStream>, server_buf : &mut String ) -> Result<(), ClientError> {
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
    match protocol {
        Protocol::ErrorOperation(message) => {
            eprintln!("{}", ClientError::ServerErrorMessage(message));
        },
        _ => {}
    }
    Ok(())
}

fn write_to_addr(writer : &mut BufWriter<&TcpStream>, bytes : &[u8]) -> Result<(), ClientError> { 
    writer.write_all(bytes).map_err(|_| ClientError::FailedWrite)?;
    writer.flush().map_err(|_| ClientError::FailedWrite)?;
    Ok(())
}

fn last_value_of_calculator(reader : &mut BufReader<&TcpStream>, server_buf : &mut String) -> Result<(), ClientError> { 
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
        _ => {return Err(ClientError::ErrorMessage);}
    }
    Ok(())
}

