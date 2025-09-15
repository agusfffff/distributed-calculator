//! Modulo de Logger
//! Este módulo proporciona un logger simple basado en hilos que escribe eventos de log en un archivo.
//! Soporta eventos de tipo `Info`, `Error` y `CloseConnection`, y corre en un hilo dedicado.
use std::{fs::OpenOptions, io::Write, sync::mpsc, thread, time::SystemTime};

/// Representa un evento de log que puede ser enviado al hilo del logger.
pub enum LogEvent{ 
    /// Mensaje informativo
    Info(String), 
    /// Mensaje de error    
    Error(String), 
    /// Señal para cerrar el hilo del logger de manera segura    
    CloseConnection
}

/// Inicia un hilo de logger que escucha eventos `LogEvent` y los escribe en un archivo.
/// Recibe: `file_path` - Ruta del archivo de log. El archivo se borra al iniciar.
/// Recibe: `receiver` - Canal MPSC desde el que se recibirán los eventos de log.
///
/// Devuelve un `JoinHandle` del hilo del logger. Se puede llamar a `.join()` para esperar a que termine.
///
/// #Comportamiento
///
/// Borra el contenido del archivo de log al inicio.
/// Añade nuevas entradas a medida que llegan eventos.
/// Termina cuando recibe `LogEvent::CloseConnection`.
pub fn start_logger(file_path: &str, reciever: mpsc::Receiver<LogEvent>) -> thread::JoinHandle<()> { 
    let path = file_path.to_string(); 
    
    thread::spawn(move || { 

        if let Err(e) = OpenOptions::new().write(true).truncate(true).create(true).open(&path) {
            eprintln!("Failed to clear log file: {}", e);
            return;
        }

        let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
                return;
            }
        };

        for event in reciever {
            match event { 
                LogEvent::Info(msg) => { 
                    let line = format!("[{:?}] INFO: {}\n", SystemTime::now(), msg); 
                    let _ = file.write_all(line.as_bytes());
                    let _ = file.flush();
                },
                LogEvent::Error(msg) => { 
                    let line = format!("[{:?}] ERROR: {}\n", SystemTime::now(), msg); 
                    let _ = file.write_all(line.as_bytes());
                    let _ = file.flush();
                }
                LogEvent::CloseConnection => break,
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, sync::mpsc};

    use crate::logger::{start_logger, LogEvent};
    #[test]
    fn test_logger_receives_events() {
        let log_path = "logs/server_test_.log";

        let _ = fs::remove_file(log_path);

        let (sender, receiver) = mpsc::channel();
        let handle = start_logger(log_path, receiver);

        sender.send(LogEvent::Info("Test info".to_string())).unwrap();
        sender.send(LogEvent::Error("Test error".to_string())).unwrap();
        sender.send(LogEvent::CloseConnection).unwrap(); 

        handle.join().unwrap();

        let content = fs::read_to_string(log_path).unwrap();
        assert!(content.contains("INFO: Test info"));
        assert!(content.contains("ERROR: Test error"));
        let _ = fs::remove_file(log_path);
    }

}