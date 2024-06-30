use crate::prelude::*;
use rayon::prelude::*;
use theframework::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum MaterialFXNodeRole {
    Geometry,
    MaterialMixer,
    Material,
    Noise2D,
    Noise3D,
    Brick,
    UVSplitter,
    Subdivide,
    BoxSubdivision,
    Tiles,
    Distance,
}

use MaterialFXNodeRole::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MaterialFXNode {
    pub id: Uuid,
    pub role: MaterialFXNodeRole,
    pub timeline: TheTimeline,

    pub position: Vec2i,

    pub supports_preview: bool,
    pub preview_is_open: bool,

    pub preview: TheRGBABuffer,

    pub resolve_branches: bool,

    pub texture_id: Option<Uuid>,
}

impl MaterialFXNode {
    pub fn new(role: MaterialFXNodeRole) -> Self {
        let mut coll = TheCollection::named(str!("Props"));
        let mut supports_preview = false;
        let mut preview_is_open = false;
        let mut resolve_branches = false;

        match role {
            Geometry => {
                supports_preview = true;
                preview_is_open = true;
            }
            MaterialMixer => {
                resolve_branches = true;
            }
            Material => {
                coll.set("Color", TheValue::PaletteIndex(0));
                coll.set("Reflectance", TheValue::FloatRange(0.5, 0.0..=1.0));
                coll.set("Roughness", TheValue::FloatRange(0.5, 0.0..=1.0));
                coll.set("Metallic", TheValue::FloatRange(0.0, 0.0..=1.0));
                //coll.set("Emission", TheValue::FloatRange(0.0, 0.0..=1.0));
                coll.set("IOR", TheValue::FloatRange(0.0, 0.0..=2.0));
                coll.set("Texture", TheValue::Text(str!("")));
            }
            UVSplitter => {
                coll.set("Map", TheValue::TextList(0, vec![str!("Cylinder")]));
            }
            Noise2D => {
                coll.set("UV Scale X", TheValue::FloatRange(1.0, 0.0..=10.0));
                coll.set("UV Scale Y", TheValue::FloatRange(1.0, 0.0..=10.0));
                coll.set("Out Scale", TheValue::FloatRange(1.0, 0.0..=1.0));
                coll.set("Octaves", TheValue::IntRange(5, 0..=5));
                supports_preview = true;
                preview_is_open = true;
            }
            Noise3D => {
                coll.set("UV Scale X", TheValue::FloatRange(1.0, 0.0..=10.0));
                coll.set("UV Scale Y", TheValue::FloatRange(1.0, 0.0..=10.0));
                coll.set("UV Scale Z", TheValue::FloatRange(1.0, 0.0..=10.0));
                coll.set("Out Scale", TheValue::FloatRange(1.0, 0.0..=1.0));
                coll.set("Octaves", TheValue::IntRange(5, 0..=5));
                supports_preview = true;
                preview_is_open = true;
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
            Subdivide => {
                coll.set(
                    "Mode",
                    TheValue::TextList(0, vec![str!("Horizontal"), str!("Vertical")]),
                );
                coll.set("Offset", TheValue::FloatRange(0.5, 0.0..=1.0));
            }
            Distance => {
                coll.set("From", TheValue::FloatRange(0.0, 0.0..=1.0));
                coll.set("To", TheValue::FloatRange(0.2, 0.0..=1.0));
                resolve_branches = true;
            }
            BoxSubdivision => {
                coll.set("Scale", TheValue::FloatRange(1.0, 0.0..=2.0));
                coll.set("Gap", TheValue::FloatRange(1.0, 0.0..=2.0));
                coll.set("Rotation", TheValue::FloatRange(0.15, 0.0..=2.0));
                coll.set("Rounding", TheValue::FloatRange(0.15, 0.0..=1.0));
            }
            Tiles => {
                coll.set("Subdivisions", TheValue::IntRange(2, 1..=8));
                coll.set("Size", TheValue::FloatRange(0.8, 0.0..=1.0));
                coll.set("Rotation", TheValue::FloatRange(0.15, 0.0..=2.0));
                coll.set("Rounding", TheValue::FloatRange(0.15, 0.0..=1.0));
            }
        }

        let timeline = TheTimeline::collection(coll);

        Self {
            id: Uuid::new_v4(),
            role,
            timeline,
            position: Vec2i::new(10, 5),
            supports_preview,
            preview_is_open,
            preview: TheRGBABuffer::empty(),
            resolve_branches,
            texture_id: None,
        }
    }

    pub fn name(&self) -> String {
        match self.role {
            Geometry => str!("Geometry"),
            MaterialMixer => str!("Material Mixer"),
            Material => str!("Material"),
            Noise2D => str!("Noise2D"),
            Noise3D => str!("Noise3D"),
            Brick => str!("Bricks"),
            UVSplitter => str!("UV Splitter"),
            Subdivide => str!("Subdivide"),
            Distance => str!("Distance"),
            BoxSubdivision => str!("Box Subdivision"),
            Tiles => str!("Tiles"),
        }
    }

    pub fn nodes() -> Vec<Self> {
        vec![
            Self::new(MaterialFXNodeRole::Geometry),
            Self::new(MaterialFXNodeRole::MaterialMixer),
            Self::new(MaterialFXNodeRole::Material),
            Self::new(MaterialFXNodeRole::Noise2D),
            Self::new(MaterialFXNodeRole::Noise3D),
            Self::new(MaterialFXNodeRole::Brick),
            Self::new(MaterialFXNodeRole::UVSplitter),
            Self::new(MaterialFXNodeRole::Subdivide),
            Self::new(MaterialFXNodeRole::Distance),
            Self::new(MaterialFXNodeRole::BoxSubdivision),
            Self::new(MaterialFXNodeRole::Tiles),
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
                        name: str!("displace"),
                        role: str!("Displace"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            Noise3D | Noise2D | UVSplitter | Subdivide | Distance => {
                vec![TheNodeTerminal {
                    name: str!("in"),
                    role: str!("Input"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            Material | MaterialMixer => {
                vec![
                    TheNodeTerminal {
                        name: str!("in"),
                        role: str!("Input"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("noise"),
                        role: str!("Noise"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            BoxSubdivision => {
                vec![TheNodeTerminal {
                    name: str!("geo"),
                    role: str!("Geometry"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            Tiles => {
                vec![
                    TheNodeTerminal {
                        name: str!("geo"),
                        role: str!("Geometry"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("noise"),
                        role: str!("Noise"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            _ => vec![],
        }
    }

    pub fn outputs(&self) -> Vec<TheNodeTerminal> {
        match self.role {
            Geometry => {
                vec![
                    TheNodeTerminal {
                        name: str!("mat"),
                        role: str!("Material"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("geo"),
                        role: str!("Geometry"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            MaterialMixer => {
                vec![
                    TheNodeTerminal {
                        name: str!("mat1"),
                        role: str!("Material1"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("mat2"),
                        role: str!("Material2"),
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
            Material | Noise3D | Noise2D | Distance => {
                vec![TheNodeTerminal {
                    name: str!("out"),
                    role: str!("Output"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            UVSplitter => {
                vec![
                    TheNodeTerminal {
                        name: str!("top"),
                        role: str!("Top"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("side"),
                        role: str!("Side"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("front"),
                        role: str!("Front"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("mapped"),
                        role: str!("Mapped"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            Subdivide => {
                vec![
                    TheNodeTerminal {
                        name: str!("mat1"),
                        role: str!("Material1"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("mat2"),
                        role: str!("Material2"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            _ => vec![],
        }
    }

    /// Computes the node.
    pub fn compute(
        &self,
        hit: &mut Hit,
        palette: &ThePalette,
        textures: &FxHashMap<Uuid, TheRGBATile>,
        resolved: Vec<Hit>,
    ) -> Option<u8> {
        match self.role {
            Material => {
                let collection = self.collection();

                let mut used_texture = false;

                if let Some(texture_id) = &self.texture_id {
                    if let Some(texture) = textures.get(texture_id) {
                        if let Some(color) = texture.buffer[0].at_f_vec4f(vec2f(hit.uv.x, hit.uv.y))
                        {
                            hit.albedo.x = color.x;
                            hit.albedo.y = color.y;
                            hit.albedo.z = color.z;
                            used_texture = true;
                        }
                    }
                }

                if !used_texture {
                    if let Some(TheValue::PaletteIndex(index)) = collection.get("Color") {
                        if let Some(color) = &palette.colors[*index as usize] {
                            hit.albedo.x = color.r;
                            hit.albedo.y = color.g;
                            hit.albedo.z = color.b;
                            if let Some(noise) = hit.noise {
                                let noise = (noise * 2.0 - 1.0) * hit.noise_scale;
                                hit.albedo.x += noise;
                                hit.albedo.y += noise;
                                hit.albedo.z += noise;
                            }
                            hit.roughness = collection.get_f32_default("Roughness", 0.5);
                            hit.metallic = collection.get_f32_default("Metallic", 0.0);
                        }
                    }
                }

                Some(0)
            }
            MaterialMixer => {
                if resolved.len() == 1 {
                    *hit = resolved[0].clone();
                } else if resolved.len() >= 2 {
                    if let Some(noise) = hit.noise {
                        let noise = noise * hit.noise_scale;
                        hit.albedo = lerp(resolved[0].albedo, resolved[1].albedo, noise);
                        hit.roughness = lerp(resolved[0].roughness, resolved[1].roughness, noise);
                        hit.metallic = lerp(resolved[0].metallic, resolved[1].metallic, noise);
                    } else {
                        hit.albedo = lerp(resolved[0].albedo, resolved[1].albedo, hit.value);
                        hit.roughness =
                            lerp(resolved[0].roughness, resolved[1].roughness, hit.value);
                        hit.metallic = lerp(resolved[0].metallic, resolved[1].metallic, hit.value);
                    }
                }
                None
            }
            Noise2D => {
                let collection = self.collection();
                hit.noise_scale = collection.get_f32_default("Out Scale", 1.0);
                hit.noise = Some(noise2d(&collection, &hit.uv));
                hit.albedo = vec3f(hit.value, hit.value, hit.value);
                Some(0)
            }
            Noise3D => {
                let collection = self.collection();
                hit.noise_scale = collection.get_f32_default("Out Scale", 1.0);
                hit.noise = Some(noise3d(&collection, &hit.hit_point));
                hit.albedo = vec3f(hit.value, hit.value, hit.value);
                Some(0)
            }
            Brick => {
                let collection = self.collection();
                let (_, terminal) = bricks(&collection, hit.uv, hit);
                Some(terminal)
            }
            UVSplitter => {
                if hit.two_d {
                    // In 2D mode, we akways return the top face, UV is already set
                    return Some(0);
                }
                let normal = hit.normal;
                let hp = hit.hit_point;
                // if abs(normal.y) > abs(normal.x) && abs(normal.y) > abs(normal.z) {
                if abs(normal.y) > 0.9 && abs(normal.x) < 0.1 && abs(normal.z) < 0.1 {
                    // Top (or bottom) face
                    hit.uv = Vec2f::new(frac(hp.x), frac(hp.z));
                    Some(0)
                // } else if abs(normal.x) > abs(normal.y) && abs(normal.x) > abs(normal.z) {
                } else if abs(normal.x) > 0.9 && abs(normal.y) < 0.1 && abs(normal.z) < 0.1 {
                    // Side face (left or right)
                    hit.uv = Vec2f::new(frac(hp.z), 1.0 - frac(hp.y));
                    Some(1)
                // } else if abs(normal.z) > abs(normal.y) && abs(normal.z) > abs(normal.x) {
                } else if abs(normal.z) > 0.9 && abs(normal.y) < 0.1 && abs(normal.x) < 0.1 {
                    // Front (or back) face
                    hit.uv = Vec2f::new(frac(hp.x), 1.0 - frac(hp.y));
                    Some(2)
                } else {
                    let collection = self.collection();
                    let map = collection.get_i32_default("Map", 0);

                    if map == 0 {
                        // Cylindrical mapping

                        let u = atan2(hp.z, hp.x) / (2.0 * f32::pi()) + 0.5; // Map the angle to [0, 1]
                        let v = hp.y;

                        hit.uv = Vec2f::new(u, v);
                    }

                    Some(3)
                }
            }
            Subdivide => {
                let collection = self.collection();
                Some(subdivide(&collection, hit.uv, hit))
            }
            Distance => {
                let collection = self.collection();
                let from = collection.get_f32_default("From", 0.0);
                let to = collection.get_f32_default("To", 0.2);

                if hit.interior_distance > PATTERN2D_DISTANCE_BORDER {
                    return None;
                }

                let value = smoothstep(from, to, -hit.interior_distance);

                if resolved.len() == 1 {
                    hit.albedo = lerp(resolved[0].albedo, hit.albedo, value);
                }

                Some(0)
            }
            _ => None,
        }
    }

    pub fn geometry(&self, hit: &mut Hit) -> Option<u8> {
        #[allow(clippy::single_match)]
        match self.role {
            BoxSubdivision => {
                let collection = self.collection();
                let scale = collection.get_f32_default("Scale", 1.0);
                let gap = collection.get_f32_default("Gap", 1.0);
                let rotation = collection.get_f32_default("Rotation", 0.15);
                let rounding = collection.get_f32_default("Rounding", 0.15);

                let p = hit.pattern_pos / (5.0 * scale);
                let rc = box_divide(p, gap, rotation, rounding);
                hit.interior_distance_mortar = Some(hit.interior_distance);
                hit.interior_distance = rc.0;
                hit.hash = rc.1;
            }
            Tiles => {
                let collection = self.collection();
                let size = collection.get_f32_default("Size", 0.8);
                let subdivisions = collection.get_i32_default("Subdivisions", 2);
                let _rotation = collection.get_f32_default("Rotation", 0.15);
                let rounding = collection.get_f32_default("Rounding", 0.15);

                let p = hit.pattern_pos; // / (5.0);

                let x = p.x.floor();
                let y = p.y.floor();

                let mut d = f32::INFINITY;

                let grid_size = subdivisions;
                let box_size = 1.0 / grid_size as f32;
                let half_box_size = box_size * 0.5;

                let rounding = rounding * half_box_size;

                // Check distance to each box in the grid
                for by in 0..grid_size {
                    for bx in 0..grid_size {
                        let center = Vec2f::new(
                            x + (bx as f32 + 0.5) * box_size,
                            y + (by as f32 + 0.5) * box_size,
                        );
                        let dist = sdf_box2d(
                            p,
                            center,
                            half_box_size * size - rounding / 1.0,
                            half_box_size * size - rounding / 1.0,
                        ) - rounding;
                        d = d.min(dist);
                    }
                }

                hit.interior_distance_mortar = Some(hit.interior_distance);
                hit.interior_distance = d;
                //hit.hash = rc.1;
            }
            _ => {}
        }
        None
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

    pub fn render_preview(&mut self, _palette: &ThePalette) {
        let width = 111;
        let height = 104;

        let mut buffer = TheRGBABuffer::new(TheDim::sized(width as i32, height));
        let collection = self.collection();

        buffer
            .pixels_mut()
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * width + i;

                    let xx = (i % width) as f32;
                    let yy = (i / width) as f32;

                    let mut color = Vec4f::zero();

                    match &self.role {
                        Noise2D => {
                            let uv = Vec2f::new(xx / width as f32, yy / height as f32);

                            let value = noise2d(&collection, &uv);
                            color = Vec4f::new(value, value, value, 1.0);
                        }
                        Noise3D => {
                            let hit_point = Vec3f::new(xx / width as f32, 0.0, yy / height as f32);

                            let value = noise3d(&collection, &hit_point);
                            color = Vec4f::new(value, value, value, 1.0);
                        }
                        _ => {}
                    }

                    pixel.copy_from_slice(&TheColor::from_vec4f(color).to_u8_array());
                }
            });

        self.preview = buffer;
    }
}
