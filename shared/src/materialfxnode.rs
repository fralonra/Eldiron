use crate::prelude::*;
use theframework::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MaterialFXNodeRole {
    Geometry,
    Material,
    Brick,
}

use MaterialFXNodeRole::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MaterialFXNode {
    pub id: Uuid,
    pub role: MaterialFXNodeRole,
    pub timeline: TheTimeline,

    pub position: Vec2i,
}

impl MaterialFXNode {
    pub fn new(role: MaterialFXNodeRole) -> Self {
        let mut coll = TheCollection::named(str!("Props"));

        match role {
            Geometry => {}
            Material => {
                coll.set("Color", TheValue::PaletteIndex(0));
                coll.set("Roughness", TheValue::FloatRange(0.5, 0.0..=1.0));
                coll.set("Metallic", TheValue::FloatRange(0.0, 0.0..=1.0));
            }
            Brick => {
                coll.set("Ratio", TheValue::FloatRange(2.0, 1.0..=10.0));
                coll.set("Rounding", TheValue::FloatRange(0.0, 0.0..=0.5));
                //coll.set("Bevel", TheValue::FloatRange(0.0, 0.0..=0.5));
                coll.set("Gap", TheValue::FloatRange(0.1, 0.0..=0.5));
                coll.set("Cell", TheValue::FloatRange(6.0, 0.0..=15.0));
                coll.set(
                    "Mode",
                    TheValue::TextList(0, vec![str!("Bricks"), str!("Tiles")]),
                );
                coll.set("Displace", TheValue::FloatRange(0.0, 0.0..=1.0));
            }
        }

        let timeline = TheTimeline::collection(coll);

        Self {
            id: Uuid::new_v4(),
            role,
            timeline,
            position: Vec2i::new(20, 20),
        }
    }

    pub fn name(&self) -> String {
        match self.role {
            Geometry => str!("Geometry"),
            Material => str!("Material"),
            Brick => str!("Bricks"),
        }
    }

    pub fn nodes() -> Vec<Self> {
        vec![
            Self::new(MaterialFXNodeRole::Geometry),
            Self::new(MaterialFXNodeRole::Material),
            Self::new(MaterialFXNodeRole::Brick),
        ]
    }

    pub fn inputs(&self) -> Vec<TheNodeTerminal> {
        match self.role {
            Brick => {
                vec![
                    TheNodeTerminal {
                        name: str!("in"),
                        role: str!("Input"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("dis"),
                        role: str!("Displacement"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            Material => {
                vec![TheNodeTerminal {
                    name: str!("in"),
                    role: str!("Input"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            _ => vec![],
        }
    }

    pub fn outputs(&self) -> Vec<TheNodeTerminal> {
        match self.role {
            Geometry => {
                vec![
                    TheNodeTerminal {
                        name: str!("3D"),
                        role: str!("3D"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("2D"),
                        role: str!("2D"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("dis"),
                        role: str!("Displacement"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            Brick => {
                vec![
                    TheNodeTerminal {
                        name: str!("brick"),
                        role: str!("Brick"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("mortar"),
                        role: str!("Mortar"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            _ => vec![],
        }
    }

    /// Computes the node.
    pub fn compute(&self, hit: &mut Hit, palette: &ThePalette) -> Option<u8> {
        match self.role {
            Material => {
                let collection = self.collection();

                if let Some(TheValue::PaletteIndex(index)) = collection.get("Color") {
                    if let Some(color) = &palette.colors[*index as usize] {
                        hit.albedo.x = color.r;
                        hit.albedo.y = color.g;
                        hit.albedo.z = color.b;
                        hit.roughness = collection.get_f32_default("Roughness", 0.5);
                        hit.metallic = collection.get_f32_default("Metallic", 0.0);
                    }
                }

                None
            }
            Brick => {
                let collection = self.collection();
                let (_, terminal) = bricks(&collection, hit.uv, hit);
                Some(terminal)
            }
            _ => None,
        }
    }

    /// Computes the displacement of the node.
    pub fn displacement(&self, hit: &mut Hit) {
        match self.role {
            Brick => {
                let collection = self.collection();
                let (_, terminal) = bricks(&collection, hit.uv, hit);
                if terminal == 1 {
                    hit.displacement = collection.get_f32_default("Displace", 0.0);
                } else {
                    hit.displacement = 0.0;
                }
            }
            _ => {}
        }
    }

    /// Creates a new node from a name.
    pub fn new_from_name(name: String) -> Self {
        let nodes = MaterialFXNode::nodes();
        for n in nodes {
            if n.name() == name {
                return n;
            }
        }
        MaterialFXNode::new(Geometry)
    }

    pub fn collection(&self) -> TheCollection {
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Props"))
        {
            return coll;
        }

        TheCollection::default()
    }

    pub fn get(&self, key: &str) -> Option<TheValue> {
        self.timeline.get(
            "Props".to_string(),
            key.to_string(),
            &TheTime::default(),
            TheInterpolation::Linear,
        )
    }

    pub fn set(&mut self, key: &str, value: TheValue) {
        self.timeline.set(&TheTime::default(), key, "Props", value);
    }

    pub fn preview(&self, _buffer: &mut TheRGBABuffer) {}
}

/*#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GeoFXNode {
    Disc(Uuid, TheTimeline),
}

impl GeoFXNode {
    pub fn new_disc() -> Self {
        let mut coll = TheCollection::named(str!("Geo"));
        coll.set("Radius", TheValue::FloatRange(0.4, 0.001..=5.0));
        Self::Disc(Uuid::new_v4(), TheTimeline::collection(coll))
    }

    pub fn nodes() -> Vec<Self> {
        vec![Self::new_disc()]
    }

    pub fn distance(&self, time: &TheTime, p: Vec2f, scale: f32) -> f32 {
        match self {
            Self::Disc(_, timeline) => {
                if let Some(value) =
                    timeline.get(str!("Geo"), str!("Radius"), time, TheInterpolation::Linear)
                {
                    if let Some(radius) = value.to_f32() {
                        return length(p) - radius * scale;
                    }
                }
            }
        }

        f32::INFINITY
    }

    pub fn collection(&self) -> TheCollection {
        match self {
            Self::Disc(_, timeline) => {
                if let Some(coll) = timeline.get_collection_at(&TheTime::default(), str!("Geo")) {
                    return coll.clone();
                }
            }
        }

        TheCollection::default()
    }

    pub fn set_id(&mut self, id: Uuid) {
        match self {
            Self::Disc(ref mut node_id, _) => {
                *node_id = id;
            }
        }
    }

    pub fn set(&mut self, key: &str, value: TheValue) {
        match self {
            Self::Disc(_, timeline) => {
                timeline.set(&TheTime::default(), key, "Geo", value);
            }
        }
    }

    pub fn preview(&self, buffer: &mut TheRGBABuffer) {
        fn mix_color(a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
            [
                (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[3] as f32 / 255.0) + b[3] as f32 / 255.0 * v) * 255.0) as u8,
            ]
        }

        let width = buffer.dim().width;
        let height = buffer.dim().height;

        for y in 0..height {
            for x in 0..width {
                let p = vec2f(
                    x as f32 / width as f32 - 0.5,
                    y as f32 / height as f32 - 0.5,
                );
                let d = self.distance(&TheTime::default(), p, 1.0);
                let t = smoothstep(-0.04, 0.0, d);
                buffer.set_pixel(x, y, &mix_color(&WHITE, &BLACK, t));
            }
        }
    }
} */
