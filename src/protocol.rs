use std::fmt;

#[derive(Debug)]

pub enum Protocol {
    Operation(String),
    Get,
    Ok,
    ErrorOperation(String),
    Value(String), 
    SynthaxError(String)
}

impl Protocol {
    pub fn from_bytes(bytes: &[u8]) -> Protocol {
        match std::str::from_utf8(bytes) {
            Ok(message) => {
                let vector: Vec<&str> = message.split_whitespace().collect();
                Protocol::from_str(vector)
            }
            Err(message) => Protocol::SynthaxError(message.to_string()),
        }
    }

    fn from_str(message: Vec<&str>) -> Protocol {
        match message.as_slice() {
            ["OPERATION", rest @ ..] if rest.len() == 2 => {
                let args = rest.join(" ");
                Protocol::Operation(args)
            }
            ["GET"] => Protocol::Get,
            ["OK"] => Protocol::Ok,
            ["ERROR" , rest @ ..] => { 
                let args = rest.join(" ");
                Protocol::ErrorOperation(args)
            },
            ["VALUE", only] => Protocol::Value((*only).to_string()),
            _ => Protocol::SynthaxError(message.join(" ")),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Protocol::Operation(args) => format!("OPERATION {}\n", args).into_bytes(),
            Protocol::Get => b"GET\n".to_vec(),
            Protocol::Ok => b"OK\n".to_vec(),
            Protocol::ErrorOperation(args) => format!("ERROR \"{}\"\n", args).into_bytes(),
            Protocol::Value(val) => format!("VALUE {}\n", val).into_bytes(),
            Protocol::SynthaxError(val) => val.as_bytes().to_vec(),
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Protocol::Operation(args) => format!("OPERATION {}\n", args),
            Protocol::Get => "GET\n".to_string(),
            Protocol::Ok => "OK\n".to_string(),
            Protocol::ErrorOperation(args) => format!("ERROR \"{}\"\n", args),
            Protocol::Value(val) => format!("VALUE {}\n", val),
            Protocol::SynthaxError(args) => args.to_string(),
        };
        write!(f, "{}", s)
    }
}