use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Value {
    // Empty
    Empty(),
    // Number
    Float(f32),
    // Number, can be both float or integer
    Integer(i32),
    // Uuid of region, and 2D position
    Position(Position),
    // Uuid of region, and Uuid of area
    Area(Uuid, Uuid),
    // Uuid of tilemap and 2D offset
    Tile(Uuid, u16, u16),
    // Text (or script)
    String(String),
    // Tile
    TileId(TileId),
    //
    TileData(TileData),
    PropertySink(PropertySink),
    Bool(bool),
    USize(usize),
    Date(Date),
}

impl Value {
    pub fn to_float(&self) -> Option<f32> {
        match self {
            Value::Integer(value) => return Some(*value as f32),
            Value::Float(value) => return Some(*value),
            _ => None,
        }
    }

    pub fn to_integer(&self) -> Option<i32> {
        match self {
            Value::Float(value) => return Some(*value as i32),
            Value::Integer(value) => return Some(*value),
            _ => None,
        }
    }

    pub fn to_string(&self) -> Option<String> {
        match self {
            Value::String(value) => return Some(value.clone()),
            _ => None,
        }
    }

    pub fn to_currency(&self) -> Option<Currency> {
        match self {
            Value::String(s) => {
                use std::str::FromStr;

                let gold_regex = regex::Regex::new(r"(\d+)g").unwrap();
                let silver_regex = regex::Regex::new(r"(\d+)s").unwrap();

                let gold = gold_regex
                    .find(s)
                    .map(|mat| i32::from_str(mat.as_str().trim_end_matches('g')).unwrap());
                let silver = silver_regex
                    .find(s)
                    .map(|mat| i32::from_str(mat.as_str().trim_end_matches('s')).unwrap());

                if gold.is_some() || silver.is_some() {
                    let mut g = 0;
                    let mut s = 0;

                    if let Some(gold) = gold {
                        g = gold;
                    }
                    if let Some(silver) = silver {
                        s = silver;
                    }

                    Some(Currency::new(g, s))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn to_string_value(&self) -> String {
        match self {
            Value::Float(value) => format!("{:?}", value),
            Value::Integer(value) => format!("{:?}", value),
            Value::String(value) => value.clone(),
            _ => "".to_string(),
        }
    }

    pub fn to_position(&self) -> Option<Position> {
        match self {
            Value::Position(value) => return Some(value.clone()),
            _ => None,
        }
    }

    pub fn to_tile_data(&self) -> Option<TileData> {
        match self {
            Value::TileData(value) => return Some(value.clone()),
            _ => None,
        }
    }

    pub fn to_tile_id(&self) -> Option<TileId> {
        match self {
            Value::Tile(value, x, y) => Some(TileId::new(*value, *x, *y)),
            Value::TileId(value) => Some(value.clone()),
            Value::TileData(value) => Some(TileId::new(value.tilemap, value.x_off, value.y_off)),
            _ => None,
        }
    }

    pub fn to_date(&self) -> Option<Date> {
        match self {
            Value::Date(value) => return Some(value.clone()),
            _ => None,
        }
    }
}
