//use crate::prelude::*;
use theframework::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ModelFXTerminalRole {
    Face,
    UV,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum ModelFXColor {
    Color(TheColor),
}

impl ModelFXColor {
    pub fn create(index: u8) -> Self {
        Self::Color(match index {
            0 => TheColor::from_hex("#cf0000"),
            1 => TheColor::from_hex("#eefb1c"),
            2 => TheColor::from_hex("#2c34d6"),
            3 => TheColor::from_hex("#0af505"),
            4 => TheColor::from_hex("#7bc4f5"),
            _ => TheColor::from_hex("#d1d1d1"),
        })
    }
    pub fn color(&self) -> &TheColor {
        match self {
            Self::Color(color) => color,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct ModelFXTerminal {
    pub role: ModelFXTerminalRole,
    pub color: ModelFXColor,
}

impl ModelFXTerminal {
    pub fn new(role: ModelFXTerminalRole, index: u8) -> Self {
        Self {
            role,
            color: ModelFXColor::create(index),
        }
    }
}
