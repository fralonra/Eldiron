
use std::error::Error;

pub mod gamedata;

pub fn really_complicated_code(a: u8, b: u8) -> Result<u8, Box<dyn Error>> {
    Ok(a + b)
}