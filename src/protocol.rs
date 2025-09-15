//! Representa el protocolo de comunicación con el que cumplen el servidor y el cliente 
//!

use std::fmt;

#[derive(Debug)]

pub enum Protocol {
    ///Operación aritmetica
    Operation(String),
    ///Pide el valor actual 
    Get,
    ///Ejecución correcta
    Ok,
    ///Se envio un algo erroneo 
    ErrorOperation(String),
    ///Valor actual 
    Value(String),
    ///Se usa para catalogar los mensajes que no son validos
    SynthaxError(String),
}

impl Protocol {
    /// Crea un `Protocol` a partir de un slice de bytes.
    ///
    /// Intenta interpretar los bytes como UTF-8.  
    /// - Si es válido, se parsea el string según las reglas del protocolo (`OP`, `GET`, `OK`, `ERROR`, `VALUE`).  
    /// - Si no es válido UTF-8, se devuelve `Protocol::SynthaxError` con el mensaje de error.
    ///
    /// # Ejemplo
    /// 
    /// let proto = Protocol::from_bytes(b"GET\n");
    /// assert!(matches!(proto, Protocol::Get));
    /// 
    pub fn from_bytes(bytes: &[u8]) -> Protocol {
        match std::str::from_utf8(bytes) {
            Ok(message) => {
                let vector: Vec<&str> = message.split_whitespace().collect();
                Protocol::from_str(vector)
            }
            Err(message) => Protocol::SynthaxError(message.to_string()),
        }
    }

    /// Parser interno: convierte un vector de tokens (`Vec<&str>`) en la variante correspondiente.
    ///
    /// - `["OP", arg1, arg2]` → `Protocol::Operation` con `"arg1 arg2"`.  
    /// - `["GET"]` → `Protocol::Get`
    /// - `["OK"]` → `Protocol::Ok`
    /// - `["ERROR", ...]` → `Protocol::ErrorOperation` con los argumentos concatenados.  
    /// - `["VALUE", val]` → `Protocol::Value` con el valor.  
    /// - Otro caso → `Protocol::SynthaxError` con el string original.
    ///
    /// Este método está marcado como `fn` porque se usa solo desde [`from_bytes`].    
    fn from_str(message: Vec<&str>) -> Protocol {
        match message.as_slice() {
            ["OP", rest @ ..] if rest.len() == 2 => {
                let args = rest.join(" ");
                Protocol::Operation(args)
            }
            ["GET"] => Protocol::Get,
            ["OK"] => Protocol::Ok,
            ["ERROR", rest @ ..] => {
                let args = rest.join(" ");
                Protocol::ErrorOperation(args)
            }
            ["VALUE", only] => Protocol::Value((*only).to_string()),
            _ => Protocol::SynthaxError(message.join(" ")),
        }
    }

    /// Convierte un `Protocol` en bytes para ser enviados por red.
    ///
    /// Devuelve la representación en texto plano del mensaje, finalizada con `\n`
    /// (excepto en el caso de `Protocol::SynthaxError`, que devuelve el mensaje sin alterarlo).
    ///
    /// # Ejemplo
    /// let proto = Protocol::Operation("ADD 5".to_string());
    /// assert_eq!(proto.to_bytes(), b"OP ADD 5\n".to_vec());
    /// 
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Protocol::Operation(args) => format!("OP {}\n", args).into_bytes(),
            Protocol::Get => b"GET\n".to_vec(),
            Protocol::Ok => b"OK\n".to_vec(),
            Protocol::ErrorOperation(args) => format!("ERROR \"{}\"\n", args).into_bytes(),
            Protocol::Value(val) => format!("VALUE {}\n", val).into_bytes(),
            Protocol::SynthaxError(val) => val.as_bytes().to_vec(),
        }
    }
}

impl fmt::Display for Protocol {
    /// Convierte el `Protocol` en su representación textual.
    ///
    /// Útil para tests y logs.  
    /// Internamente se mantiene consistente con [`to_bytes`].
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Protocol::Operation(args) => format!("OP {}\n", args),
            Protocol::Get => "GET\n".to_string(),
            Protocol::Ok => "OK\n".to_string(),
            Protocol::ErrorOperation(args) => format!("ERROR \"{}\"\n", args),
            Protocol::Value(val) => format!("VALUE {}\n", val),
            Protocol::SynthaxError(args) => args.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]

mod tests {
    use crate::protocol::Protocol;
 
    #[test]
    fn from_bytes_operation() {
        let proto = Protocol::from_bytes(b"OP ADD 5\n");
        match proto {
            Protocol::Operation(args) => assert_eq!(args, "ADD 5"),
            _ =>  assert_eq!(proto.to_string(), "OP ADD 5\n")
        }
    }

    #[test]
    fn test_from_bytes_invalid_utf8() {
        let proto = Protocol::from_bytes(&[0xFF, 0xFF, 0xFF]); // bytes no válidos UTF-8
        match proto {
            Protocol::SynthaxError(msg) => assert!(msg.contains("invalid utf-8")),
            _ => assert_eq!(proto.to_string(), "invalid utf-8"),
        }
    }    

    #[test]
    fn test_operation_to_bytes_and_display() {
        let proto = Protocol::Operation("ADD 5".to_string());
        assert_eq!(proto.to_string(), "OP ADD 5\n");
        assert_eq!(proto.to_bytes(), b"OP ADD 5\n".to_vec());
}
}
