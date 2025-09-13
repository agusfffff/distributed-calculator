use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener},
    str::FromStr,
    sync::Arc,
    thread,
};

mod calculator;
mod operation;
mod server_error;

use crate::{operation::Operation, server_error::ServerError};
use calculator::Calculator;
use distributed_calculator::protocol::Protocol;

fn main() -> Result<(), ServerError> {
    let addr: SocketAddr = parse_arguments(std::env::args())?;
    run_server(addr)?;
    Ok(())
}

fn parse_arguments<I: IntoIterator<Item=String>>(inputs: I) -> Result<SocketAddr, ServerError>{
    let mut iter = inputs.into_iter();
    iter.next();
    let ip_str = iter.next().ok_or(ServerError::MissingArgument)?;
    let addr = SocketAddr::from_str(&ip_str).map_err(|_| ServerError::InvalidArgument)?;
    Ok(addr)
}

fn run_server(address: SocketAddr) -> Result<(), ServerError> {
    // let (sender, receiver) = std::sync::mpsc::channel::<String>();
    // let logger = thread::spawn(funcion_del_logger(receiver));

    let listener: TcpListener =
        TcpListener::bind(address).map_err(|_| ServerError::BindFailed)?;

    let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let calculator_pointer = Arc::clone(&calculator);
                thread::spawn( move || 
                        if let Err(e) = handle_connection(stream, calculator_pointer) {
                        eprintln!("{}", e);
                        }
                    );
            }
            Err(_) => {
                eprintln!("{}", ServerError::FailedConnection);
                continue;
            }
        }
    }

    // caso donde se cierre el listener 

    Ok(())
}

/// handle client connection

fn handle_connection<RW: Read + Write>(mut stream: RW, calculator: Arc<std::sync::Mutex<Calculator>>)-> Result<(), ServerError> {
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
            _ => send_protocol(Protocol::SynthaxError, reader.get_mut()),
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
        Err(_) => {
            return send_protocol(Protocol::ErrorOperation, stream); 
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

fn get_value(calculator: &Arc<std::sync::Mutex<Calculator>>) -> Result<i32, ServerError> {
    match calculator.lock() {
        Ok(calc) => { 
            Ok(calc._accumulation())
        }, 
        Err(_) => Err(ServerError::PoisonError)
    }
}



 #[cfg(test)] 
mod tests {
    use std::net::TcpListener;

    use crate::{parse_arguments, run_server, server_error::ServerError};

    #[test] 
    fn parse_arguments_fails_with_missing_arguments(){ 
        //falta el ip:puerto 
        let args = vec!["program_name".to_string()]; 
        let result = parse_arguments(args);
        assert!(matches!(result, Err(ServerError::MissingArgument)));
    }

    #[test]
    fn parse_arguments_fails_with_invalid_argument(){ 
        //ip no valida
        let args = vec!["program_name".to_string(), "not_an_ip".to_string()]; 
        let result = parse_arguments(args);
        assert!(matches!(result, Err(ServerError::InvalidArgument)));
    }

    #[test]
    fn server_bind_fails(){ 
        let addr = "127.0.0.1:54321".parse().unwrap(); 
        let _listener = TcpListener::bind(addr).unwrap();

        let result =run_server(addr); 
        assert!(matches!(result, Err(ServerError::BindFailed)));
    }

}