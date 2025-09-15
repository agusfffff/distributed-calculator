use std::{fs::File, io::BufReader};

use crate::{client_error::ClientError, utils::parse_address,utils::process_files};

mod utils; 
mod client_error;

fn main() -> Result<(), ClientError> {
    let addr = parse_address(std::env::args())?;
    let file_path = std::env::args().nth(2).ok_or(ClientError::MissingArgument)?;
    let file = File::open(file_path).map_err(|_| ClientError::InvalidArgument)?; 
    let reader = BufReader::new(file);
    process_files(addr, reader)?;
    Ok(())
}
