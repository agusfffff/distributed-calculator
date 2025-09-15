use std::{
    net::{SocketAddr, TcpListener},
    str::FromStr,
    sync::Arc,
    thread,
};

mod handle_client;
mod calculator;
mod operation;
mod server_error;

use crate::{handle_client::handle_connection, server_error::ServerError};
use calculator::Calculator;

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

    run_server_with_listener(listener)
}
    
fn run_server_with_listener(listener: TcpListener) -> Result<(), ServerError> {
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


 #[cfg(test)] 
mod tests {
    use std::{net::{TcpListener}};

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