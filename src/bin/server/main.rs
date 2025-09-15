use std::{
    net::{SocketAddr, TcpListener},
    str::FromStr,
    sync::{mpsc::{self, Sender}, Arc},
    thread,
};

mod calculator;
mod handle_client;
mod operation;
mod server_error;
mod logger;
use crate::{handle_client::handle_connection, logger::LogEvent, server_error::ServerError};
use calculator::Calculator;
use logger::start_logger;

fn main() -> Result<(), ServerError> {
    let addr: SocketAddr = parse_arguments(std::env::args())?;
    let log_path = "./logs/server.log";
    run_server(addr, log_path)?;
    Ok(())
}

fn parse_arguments<I: IntoIterator<Item = String>>(inputs: I) -> Result<SocketAddr, ServerError> {
    let mut iter = inputs.into_iter();
    iter.next();
    let ip_str = iter.next().ok_or(ServerError::MissingArgument)?;
    let addr = SocketAddr::from_str(&ip_str).map_err(|_| ServerError::InvalidArgument)?;
    Ok(addr)
}

fn run_server(address: SocketAddr, log_file: &str) -> Result<(), ServerError> {
    let (sender, receiver) = mpsc::channel::<LogEvent>();
    let logger_handle = start_logger(log_file, receiver);
    
    let listener: TcpListener = TcpListener::bind(address).map_err(|_| ServerError::BindFailed)?;

    run_server_with_listener(listener,  sender.clone())?;
    
    let _ = sender.send(LogEvent::CloseConnection);
    match logger_handle.join()  {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open log file: [{:?}] ", e);
            }
    };

    Ok(())

}

fn run_server_with_listener(listener: TcpListener, sender : Sender<LogEvent> ) -> Result<(), ServerError> {
    let calculator = Arc::new(std::sync::Mutex::new(Calculator::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let calculator_pointer = Arc::clone(&calculator);
                let sender_clone = sender.clone();
                let peer_addr = stream.peer_addr().map_or("unknown".to_string(), |p| p.to_string());
                let _ = sender_clone.send(LogEvent::Info(format!("New connection from {}", peer_addr)));

                thread::spawn(move || {
                    if let Err(e) = handle_connection(stream, calculator_pointer, sender_clone.clone()) {
                        eprintln!("{}", e);
                        let _ = sender_clone.send(LogEvent::Error(format!("Error: {}", e)));
                    }

                    let _ = sender_clone.send(LogEvent::Info(format!("Connection from {} closed", peer_addr)));
                });
            }
            Err(_) => {
                eprintln!("{}", ServerError::FailedConnection);
                let _ = sender.send(LogEvent::Error(format!("{}", ServerError::FailedConnection)));
                continue;
            }
        }
    }

    Ok(())
}

#[cfg(test)]

mod tests {
    use std::net::TcpListener;

    use crate::{parse_arguments, run_server, server_error::ServerError};

    #[test]
    fn parse_arguments_fails_with_missing_arguments() {
        let args = vec!["program_name".to_string()];
        let result = parse_arguments(args);
        assert!(matches!(result, Err(ServerError::MissingArgument)));
    }

    #[test]
    fn parse_arguments_fails_with_invalid_argument() {
        let args = vec!["program_name".to_string(), "not_an_ip".to_string()];
        let result = parse_arguments(args);
        assert!(matches!(result, Err(ServerError::InvalidArgument)));
    }

    #[test]
    fn server_bind_fails() {
        let addr = "127.0.0.1:54321".parse().unwrap();
        let _listener = TcpListener::bind(addr).unwrap();
        let log_path = "./logs/server.log";
        let result = run_server(addr,log_path);
        assert!(matches!(result, Err(ServerError::BindFailed)));
    }
}
