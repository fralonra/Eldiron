
use std::error::Error;

pub mod asset;
pub mod gamedata;
pub mod draw2d;
pub mod script_types;

pub fn really_complicated_code(a: u8, b: u8) -> Result<u8, Box<dyn Error>> {
    Ok(a + b)
}