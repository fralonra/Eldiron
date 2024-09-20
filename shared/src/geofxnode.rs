use crate::prelude::*;
use rayon::prelude::*;
use theframework::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GeoFXNodeExtrusion {
    None,
    X,
    Y,
    Z,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GeoFXNodeFacing {
    NorthSouth,
    WestEast,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum GeoFXNodeRole {
    AddHeight,
    RemoveHeight,
    SetHeight,
    Column,

    LeftWall,
    TopWall,
    RightWall,
    BottomWall,
    MiddleWallH,
    MiddleWallV,

    Gate,

    Box,
    Disc,
    Bricks,
    Group,

    Material,

    Repeat,
    Stack,

    MetaMaterial,
    MetaDelete,
}

use GeoFXNodeRole::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GeoFXNode {
    pub id: Uuid,
    pub role: GeoFXNodeRole,
    pub timeline: TheTimeline,

    pub position: Vec2i,

    pub supports_preview: bool,
    pub preview_is_open: bool,

    pub preview: TheRGBABuffer,
}

impl GeoFXNode {
    pub fn is_shape(&self) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match &self.role {
            Box => true,
            Disc => true,
            _ => false,
        }
    }

    pub fn new(role: GeoFXNodeRole) -> Self {
        let mut coll = TheCollection::named(str!("Geo"));
        let supports_preview = false;
        let preview_is_open = false;

        match role {
            AddHeight => {
                coll.set("Add Height", TheValue::Float(0.2));
            }
            RemoveHeight => {
                coll.set("Remove Height", TheValue::Float(0.2));
            }
            SetHeight => {
                coll.set("Height", TheValue::Float(0.0));
            }
            Box => {
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set("Rounding", TheValue::Text(str!("0.0")));
                coll.set("Rotation", TheValue::Text(str!("0.0")));
                coll.set("Annular", TheValue::Text(str!("0.0")));
                coll.set("Extrusion", TheValue::Text(str!("thickness")));
            }
            Disc => {
                coll.set("Radius", TheValue::FloatRange(0.5, 0.0..=1.0));
                coll.set("Annular", TheValue::Text(str!("0.0")));
                coll.set("Extrusion", TheValue::Text(str!("thickness")));
            }
            LeftWall | TopWall | RightWall | BottomWall | MiddleWallH | MiddleWallV => {
                coll.set("Pos X", TheValue::Float(0.1));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set("Thickness", TheValue::FloatRange(0.1, 0.001..=1.0));
                // coll.set(
                //     "2D Mode",
                //     TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                // );
            }
            /*
            TopWall => {
                // coll.set("Pos X", TheValue::Float(0.5));
                // coll.set("Pos Y", TheValue::Float(0.1));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                // coll.set(
                //     "2D Mode",
                //     TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                // );
            }
            RightWall => {
                // coll.set("Pos X", TheValue::Float(0.9));
                // coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                // coll.set(
                //     "2D Mode",
                //     TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                // );
            }
            BottomWall => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.9));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set(
                    "2D Mode",
                    TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                );
            }
            MiddleWallH => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set(
                    "2D Mode",
                    TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                );
            }
            MiddleWallV => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set(
                    "2D Mode",
                    TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
                );
            }
            BendWallNW | BendWallNE | BendWallSW | BendWallSE => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Thickness", TheValue::FloatRange(0.2, 0.001..=1.0));
                coll.set("Rounding", TheValue::FloatRange(0.3, 0.0..=1.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
            }*/
            Column => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set("Radius", TheValue::FloatRange(0.4, 0.001..=2.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
                coll.set("Hole", TheValue::FloatRange(0.0, 0.0..=1.0));
            }
            Gate => {
                coll.set("Pos X", TheValue::Float(0.5));
                coll.set("Pos Y", TheValue::Float(0.5));
                coll.set(
                    "Align",
                    TheValue::TextList(0, vec![str!("North/South"), str!("West/East")]),
                );
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=3.0));
            }
            Bricks => {
                coll.set("Ratio", TheValue::FloatRange(2.0, 1.0..=10.0));
                coll.set("Rounding", TheValue::FloatRange(0.0, 0.0..=0.5));
                coll.set("Rotation", TheValue::FloatRange(1.0, 0.0..=5.0));
                coll.set("Gap", TheValue::FloatRange(1.0, 0.0..=5.0));
                coll.set("Cell", TheValue::FloatRange(3.0, 0.0..=15.0));
            }
            Material => {
                coll.set("Color", TheValue::PaletteIndex(0));
                coll.set("Modifier", TheValue::Text(str!("0.0")));
                coll.set("Roughness", TheValue::Text(str!("0.5")));
                coll.set("Metallic", TheValue::Text(str!("0.0")));
                coll.set("Anisotropic", TheValue::Text(str!("0.0")));
                coll.set("Subsurface", TheValue::Text(str!("0.0")));
                coll.set("Specular Tint", TheValue::Text(str!("0.0")));
                coll.set("Sheen", TheValue::Text(str!("0.0")));
                coll.set("Sheen Tint", TheValue::Text(str!("0.0")));
                coll.set("Clearcoat", TheValue::Text(str!("0.0")));
                coll.set("Clearcoat Gloss", TheValue::Text(str!("0.0")));
                coll.set("Transmission", TheValue::Text(str!("0.0")));
                coll.set("Emission", TheValue::Text(str!("0.0")));
                coll.set("IOR", TheValue::Text(str!("1.5")));
                coll.set("Texture", TheValue::Text(str!("")));
            }
            Repeat => {
                coll.set("Spacing", TheValue::FloatRange(0.01, 0.0..=1.0));
                coll.set("Offset", TheValue::FloatRange(0.0, 0.0..=1.0));
            }
            Stack => {
                coll.set("Spacing", TheValue::FloatRange(0.01, 0.0..=1.0));
                coll.set("Offset", TheValue::FloatRange(0.0, 0.0..=1.0));
            }
            Group => {
                coll.set("X", TheValue::FloatRange(0.0, 0.0..=10.0));
                coll.set("Y", TheValue::FloatRange(0.0, 0.0..=10.0));
                coll.set("Length", TheValue::FloatRange(1.0, 0.001..=10.0));
                coll.set("Height", TheValue::FloatRange(1.0, 0.001..=10.0));
            }
            MetaMaterial => {
                coll.set("Meta", TheValue::Text(str!("")));
            }
            MetaDelete => {
                coll.set("Meta", TheValue::Text(str!("")));
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
        }
    }

    pub fn nodes() -> Vec<Self> {
        vec![
            Self::new(GeoFXNodeRole::AddHeight),
            Self::new(GeoFXNodeRole::RemoveHeight),
            Self::new(GeoFXNodeRole::SetHeight),
            Self::new(GeoFXNodeRole::LeftWall),
            Self::new(GeoFXNodeRole::TopWall),
            Self::new(GeoFXNodeRole::RightWall),
            Self::new(GeoFXNodeRole::BottomWall),
            Self::new(GeoFXNodeRole::MiddleWallH),
            Self::new(GeoFXNodeRole::MiddleWallV),
            // Self::new(GeoFXNodeRole::BendWallNW),
            // Self::new(GeoFXNodeRole::BendWallNE),
            // Self::new(GeoFXNodeRole::BendWallSW),
            // Self::new(GeoFXNodeRole::BendWallSE),
            Self::new(GeoFXNodeRole::Column),
            Self::new(GeoFXNodeRole::Gate),
            Self::new(GeoFXNodeRole::Box),
            Self::new(GeoFXNodeRole::Disc),
            Self::new(GeoFXNodeRole::Bricks),
            Self::new(GeoFXNodeRole::Material),
            Self::new(GeoFXNodeRole::Repeat),
            Self::new(GeoFXNodeRole::Stack),
            Self::new(GeoFXNodeRole::Group),
            Self::new(GeoFXNodeRole::MetaMaterial),
            Self::new(GeoFXNodeRole::MetaDelete),
        ]
    }

    pub fn name(&self) -> String {
        match &self.role {
            AddHeight => str!("Add height to the ground tile (height map)."),
            RemoveHeight => str!("Remove height from the ground tile (height map)."),
            SetHeight => str!("Set the height of the ground tile (height map)."),
            LeftWall => str!("Left Wall"),
            TopWall => str!("Top Wall"),
            RightWall => str!("Right Wall"),
            BottomWall => str!("Bottom Wall"),
            MiddleWallH => str!("Middle Wall X"),
            MiddleWallV => str!("Niddle Wall Y"),
            Column => str!("A column (disc) with an optional profile."),
            Gate => str!("A gate."),
            Bricks => "Bricks".to_string(),
            Box => "Box".to_string(),
            Disc => "Disc".to_string(),
            Material => "Material".to_string(),
            Repeat => "Repeat".to_string(),
            Stack => "Stack".to_string(),
            Group => "Group".to_string(),
            MetaMaterial => "Meta Material".to_string(),
            MetaDelete => "Meta Delete".to_string(),
        }
    }

    /// Creates a new node from a name.
    pub fn new_from_name(name: String) -> Self {
        let nodes = GeoFXNode::nodes();
        for n in nodes {
            if n.name() == name {
                return n;
            }
        }
        GeoFXNode::new(MiddleWallH)
    }

    pub fn description(&self) -> String {
        match &self.role {
            AddHeight => str!("Add height to the ground tile (height map)."),
            RemoveHeight => str!("Remove height from the ground tile (height map)."),
            SetHeight => str!("Set the height of the ground tile (height map)."),
            LeftWall => str!("A wall on the left side of the tile."),
            TopWall => str!("A wall on the top side of the tile."),
            RightWall => str!("A wall on the right side of the tile."),
            BottomWall => str!("A wall on the bottom side of the tile."),
            MiddleWallH => str!("A horizontal wall in the middle of the tile."),
            MiddleWallV => str!("A vertical wall in the middle of the tile."),
            // BendWallNW => str!("A rounded wall from the left to the top side of the tile."),
            // BendWallNE => str!("A rounded wall from the top to the right side of the tile."),
            // BendWallSE => str!("A rounded wall from the right to the bottom side of the tile."),
            // BendWallSW => str!("A rounded wall from the bottom to the left side of the tile."),
            Column => str!("A column (disc) with an optional profile."),
            Gate => str!("A gate."),
            _ => "".to_string(),
        }
    }

    /// Returns the layer role (RemoveHeightBrush, Wall etc) for this node.
    pub fn get_layer_role(&self) -> Layer2DRole {
        match self.role {
            GeoFXNodeRole::AddHeight | GeoFXNodeRole::RemoveHeight | GeoFXNodeRole::SetHeight => {
                Layer2DRole::Ground
            }
            _ => Layer2DRole::Wall,
        }
    }

    /// Gives the node a chance to update its parameters in case things changed.
    pub fn update_parameters(&mut self) {
        // match self.role {
        //     LeftWall | TopWall | RightWall | BottomWall => {
        //         if let Some(coll) = self
        //             .timeline
        //             .get_collection_at(&TheTime::default(), str!("Geo"))
        //         {
        //             // if coll.get_f32_default("Height", 0.01) == 0.1 {
        //             //     self.set("Height", TheValue::FloatRange(0.01, 0.001..=1.0));
        //             // }
        //             self.set(
        //                 "2D Mode",
        //                 TheValue::TextList(0, vec![str!("Normal"), str!("Full Thickness")]),
        //             );
        //         }
        //     }
        //     _ => {}
        // }
    }

    pub fn build(
        &self,
        palette: &ThePalette,
        _textures: &FxHashMap<Uuid, TheRGBATile>,
        ctx: &mut FTBuilderContext,
    ) {
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            let mut shape_params = str!("");

            if self.is_shape() {
                if let Some(value) = coll
                    .get_default("Extrusion", TheValue::Text(str!("thickness")))
                    .to_string()
                {
                    if value != "thickness" {
                        shape_params += &format!(", extrusion = {}", value);
                    }
                }
                if let Some(value) = coll
                    .get_default("Rotation", TheValue::Text(str!("0.0")))
                    .to_string()
                {
                    if value != "0.0" {
                        shape_params += &format!(", rotation = {}", value);
                    }
                }
                if let Some(value) = coll
                    .get_default("Annular", TheValue::Text(str!("0.0")))
                    .to_string()
                {
                    if value != "0.0" {
                        shape_params += &format!(", annular = {}", value);
                    }
                }
            }

            match self.role {
                Box => {
                    let geo = format!(
                        "let box_{id_counter} = Shape<Box>: material = {material}, length = {length}, height = {height}, rounding = {rounding}{shape_params};\n",
                        id_counter = { ctx.id_counter },
                        material = { if ctx.material_id.is_some() { ctx.material_id.clone().unwrap()} else {str!("none") }},
                        length = coll.get_f32_default("Length", 1.0),
                        height = coll.get_f32_default("Height", 1.0),
                        rounding = coll.get_f32_default("Rounding", 0.0) / 2.0,
                    );
                    ctx.geometry.push(format!("box_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Disc => {
                    let geo = format!(
                        "let disc_{id_counter} = Shape<Disc>: material = {material}, radius = {radius}{shape_params};\n",
                        id_counter = { ctx.id_counter },
                        material = { if ctx.material_id.is_some() { ctx.material_id.clone().unwrap()} else {str!("none") }},
                        radius = coll.get_f32_default("Radius", 0.5),
                    );
                    ctx.geometry.push(format!("disc_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Bricks => {
                    let geo = format!(
                        "let pattern_{id_counter} = Pattern<Bricks>: material = {material}, ratio = {ratio}, rounding = {rounding}, rotation = {rotation}, gap = {gap}, cell = {cell};\n",
                        id_counter = { ctx.id_counter },
                        material = { if ctx.material_id.is_some() { ctx.material_id.clone().unwrap()} else {str!("none") }},
                        ratio = coll.get_f32_default("Ratio", 3.0),
                        rounding = coll.get_f32_default("Rounding", 0.0),
                        rotation = coll.get_f32_default("Rotation", 1.0),
                        gap = coll.get_f32_default("Gap", 1.0),
                        cell = coll.get_f32_default("Cell", 3.0),
                    );
                    ctx.geometry.push(format!("pattern_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Repeat => {
                    let geometry = ctx.geometry.join(",");
                    let geo = format!(
                        "let pattern_{id_counter} = Pattern<Repeat>: content = [{geometry}], spacing = {spacing}, offset = {offset};\n",
                        id_counter = { ctx.id_counter },
                        spacing = coll.get_f32_default("Spacing", 0.01),
                        offset = coll.get_f32_default("Offset", 0.0),
                        geometry = geometry
                    );
                    ctx.geometry.clear();
                    ctx.geometry.push(format!("pattern_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Stack => {
                    let geometry = ctx.geometry.join(",");
                    let geo = format!(
                        "let pattern_{id_counter} = Pattern<Stack>: content = [{geometry}], spacing = {spacing}, offset = {offset};\n",
                        id_counter = { ctx.id_counter },
                        spacing = coll.get_f32_default("Spacing", 0.01),
                        offset = coll.get_f32_default("Offset", 0.0),
                        geometry = geometry
                    );
                    ctx.geometry.clear();
                    ctx.geometry.push(format!("pattern_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Group => {
                    let geometry = ctx.geometry.join(",");
                    let geo = format!(
                        "let pattern_{id_counter} = Pattern<Group>: content = [{geometry}], x = {x}, y = {y}, length = {length}, height = {height};\n",
                        id_counter = { ctx.id_counter },
                        x = coll.get_f32_default("X", 0.0),
                        y = coll.get_f32_default("Y", 0.0),
                        length = coll.get_f32_default("Length", 1.0),
                        height = coll.get_f32_default("Height", 0.0),
                        geometry = geometry
                    );
                    ctx.geometry.clear();
                    ctx.geometry.push(format!("pattern_{}", ctx.id_counter));
                    ctx.out += &geo;
                    ctx.id_counter += 1;
                    ctx.material_id = None;
                }
                Material => {
                    let mut hex = "000000".to_string();
                    let color_index = coll.get_i32_default("Color", 0);
                    if let Some(color) = &palette.colors[color_index as usize] {
                        hex = color.to_hex();
                        hex.remove(0);
                    }

                    let mut parameters = "".to_string();

                    // Modifier
                    if let Some(modifier) = coll
                        .get_default("Modifier", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if modifier != "0.0" {
                            parameters += &format!(", modifier = {}", modifier);
                        }
                    }
                    // Roughness
                    if let Some(value) = coll
                        .get_default("Roughness", TheValue::Text(str!("0.5")))
                        .to_string()
                    {
                        if value != "0.5" {
                            parameters += &format!(", roughness = {}", value);
                        }
                    }
                    // Metallic
                    if let Some(value) = coll
                        .get_default("Metallic", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", metallic = {}", value);
                        }
                    }
                    // Anisotropic
                    if let Some(value) = coll
                        .get_default("Anisotropic", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", anisotropic = {}", value);
                        }
                    }
                    // Subsurface
                    if let Some(value) = coll
                        .get_default("Subsurface", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", subsurface = {}", value);
                        }
                    }
                    // Specular Tint
                    if let Some(value) = coll
                        .get_default("Specular Tint", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", specular_tint = {}", value);
                        }
                    }
                    // Sheen
                    if let Some(value) = coll
                        .get_default("Sheen", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", sheen = {}", value);
                        }
                    }
                    // Sheen Tint
                    if let Some(value) = coll
                        .get_default("Sheen Tint", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", sheen_tint = {}", value);
                        }
                    }
                    // Clearcoat
                    if let Some(value) = coll
                        .get_default("Clearcoat", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", clearcoat = {}", value);
                        }
                    }
                    // Clearcoat Gloss
                    if let Some(value) = coll
                        .get_default("Clearcoat Gloss", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", clearcoat_gloss = {}", value);
                        }
                    }
                    // Transmission
                    if let Some(value) = coll
                        .get_default("Transmission", TheValue::Text(str!("0.0")))
                        .to_string()
                    {
                        if value != "0.0" {
                            parameters += &format!(", transmission = {}", value);
                        }
                    }
                    // IOR
                    if let Some(value) = coll
                        .get_default("IOR", TheValue::Text(str!("1.5")))
                        .to_string()
                    {
                        if value != "1.5" {
                            parameters += &format!(", ior = {}", value);
                        }
                    }

                    // println!("parameters {}", parameters);

                    let mat = format!(
                        "let material_{id_counter} = Material<BSDF>: color = #{hex}{parameters};\n",
                        id_counter = { ctx.id_counter },
                        hex = { hex },
                        parameters = { parameters }
                    );
                    ctx.out += &mat;
                    ctx.material_id = Some(format!("material_{}", ctx.id_counter));
                    ctx.id_counter += 1;
                }
                LeftWall | MiddleWallV | RightWall | TopWall | MiddleWallH | BottomWall => {
                    let face_type = match &self.role {
                        LeftWall => "Left",
                        MiddleWallV => "MiddleY",
                        RightWall => "Right",
                        TopWall => "Top",
                        MiddleWallH => "MiddleX",
                        BottomWall => "Bottom",
                        _ => "",
                    };

                    let geometry = ctx.geometry.join(",");
                    let geo = format!(
                        "let face = Face<{face_type}> : length = {length}, height = {height}, thickness = {thickness}, content = [{geometry}];\n",
                        face_type = face_type,
                        length = coll.get_f32_default("Length", 1.0),
                        height = coll.get_f32_default("Height", 1.0),
                        thickness = coll.get_f32_default("Thickness", 0.2),
                        geometry = geometry
                    );
                    ctx.out += &geo;
                }
                MetaMaterial => {
                    if let Some(value) = coll
                        .get_default("Meta", TheValue::Text(str!("")))
                        .to_string()
                    {
                        if !value.is_empty() {
                            let geo = format!(
                                "let meta_{id_counter}  = Meta<Material> : material = {material}, content = [{meta}];\n",
                                id_counter = { ctx.id_counter },
                                material = { if ctx.material_id.is_some() { ctx.material_id.clone().unwrap()} else {str!("none") }},
                                meta = { value }
                            );
                            ctx.id_counter += 1;
                            ctx.out += &geo;
                            ctx.material_id = None;
                        }
                    }
                }
                MetaDelete => {
                    if let Some(value) = coll
                        .get_default("Meta", TheValue::Text(str!("")))
                        .to_string()
                    {
                        if !value.is_empty() {
                            let geo = format!(
                                "let meta_{id_counter}  = Meta<Delete> : content = [{meta}];\n",
                                id_counter = { ctx.id_counter },
                                meta = { value }
                            );
                            ctx.id_counter += 1;
                            ctx.out += &geo;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// The 2D distance from the node to a point.
    pub fn distance(
        &self,
        _time: &TheTime,
        p: Vec2f,
        scale: f32,
        hit: &mut Option<&mut Hit>,
    ) -> f32 {
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            match self.role {
                LeftWall => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = (t / 2.0) * scale + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let mut pos = self.position(&coll);
                    pos.x = pos.x.floor() + t / 2.0;
                    pos *= scale;

                    return sdf_box2d(p, pos, thick, len);
                }
                TopWall => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = (t / 2.0) * scale + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let mut pos = self.position(&coll);
                    pos.y = pos.y.floor() + t / 2.0;
                    pos *= scale;

                    return sdf_box2d(p, pos, len, thick);
                }
                RightWall => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = (t / 2.0) * scale + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let mut pos = self.position(&coll);
                    pos.x = pos.x.floor() + 1.0 - t / 2.0;
                    pos *= scale;

                    return sdf_box2d(p, pos, thick, len);
                }
                BottomWall => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = (t / 2.0) * scale + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let mut pos = self.position(&coll);
                    pos.y = pos.y.floor() + 1.0 - t / 2.0;
                    pos *= scale;

                    return sdf_box2d(p, pos, len, thick);
                }
                MiddleWallV => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = t * scale / 2.0 + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let pos = self.position(&coll) * scale;

                    return sdf_box2d(p, pos, thick, len);
                }
                MiddleWallH => {
                    let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                        1.0
                    } else {
                        coll.get_f32_default("Thickness", 0.2)
                    };

                    let thick = t * scale / 2.0 + 0.1;
                    let len = coll.get_f32_default("Length", 1.0) * scale / 2.0 + 0.1;

                    let pos = self.position(&coll) * scale;

                    return sdf_box2d(p, pos, len, thick);
                }
                // BendWallNW => {
                //     let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                //         1.0
                //     } else {
                //         coll.get_f32_default("Thickness", 0.2)
                //     };

                //     let thick = t * scale / 2.0 + 0.1;
                //     let round = coll.get_f32_default("Rounding", 0.3) * scale;

                //     let mut pos = self.position(&coll);
                //     pos += 1.0;
                //     pos *= scale;

                //     let rounding = (round, round, round, round);

                //     let p = p - pos;
                //     let d = sdf_rounded_box2d(p, 1.5 * scale, thick, rounding);

                //     return d.abs() - thick;
                // }
                // BendWallNE => {
                //     let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                //         1.0
                //     } else {
                //         coll.get_f32_default("Thickness", 0.2)
                //     };

                //     let thick = t * scale / 2.0 + 0.1;
                //     let round = coll.get_f32_default("Rounding", 0.3) * scale;

                //     let mut pos = self.position(&coll);
                //     pos += vec2f(-1.0, 1.0);
                //     pos *= scale;

                //     let rounding = (round, round, round, round);

                //     let p = p - pos;
                //     let d = sdf_rounded_box2d(p, 1.5 * scale, thick, rounding);

                //     return d.abs() - thick;
                // }
                // BendWallSW => {
                //     let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                //         1.0
                //     } else {
                //         coll.get_f32_default("Thickness", 0.2)
                //     };

                //     let thick = t * scale / 2.0 + 0.1;
                //     let round = coll.get_f32_default("Rounding", 0.3) * scale;

                //     let mut pos = self.position(&coll);
                //     pos += vec2f(1.0, -1.0);
                //     pos *= scale;

                //     let rounding = (round, round, round, round);

                //     let p = p - pos;
                //     let d = sdf_rounded_box2d(p, 1.5 * scale, thick, rounding);

                //     return d.abs() - thick;
                // }
                // BendWallSE => {
                //     let t = if coll.get_i32_default("2D Mode", 0) == 1 {
                //         1.0
                //     } else {
                //         coll.get_f32_default("Thickness", 0.2)
                //     };

                //     let thick = t * scale / 2.0 + 0.1;
                //     let round = coll.get_f32_default("Rounding", 0.3) * scale;

                //     let mut pos = self.position(&coll);
                //     pos += vec2f(-1.0, -1.0);
                //     pos *= scale;

                //     let rounding = (round, round, round, round);

                //     let p = p - pos;
                //     let d = sdf_rounded_box2d(p, 1.5 * scale, thick, rounding);

                //     return d.abs() - thick;
                // }
                Column => {
                    let radius = coll.get_f32_default("Radius", 0.4);

                    // let waveAmplitude = 0.05;
                    // let waveFrequency = 12.0 * 4.0; // Higher frequency for more fluting patterns

                    // let angle = atan2(p.y + 0.5, p.x + 0.5);

                    // // Modulate the radius with a sine wave to create fluting
                    // let wave = waveAmplitude * sin(waveFrequency * angle);

                    // // Calculate the modified radius
                    // let modifiedRadius = radius + wave; // * 0.05;

                    let hole = coll.get_f32_default("Annular", 0.0) * scale;

                    let mut d = length(p - self.position(&coll) * scale) - radius * scale + hole;
                    if hole > 0.0 {
                        d = d.abs() - hole;
                    }

                    return d;
                }
                Gate => {
                    let mut pos = self.position(&coll) * scale;
                    let h = coll.get_f32_default("Height", 1.0);
                    let height = h * scale;
                    pos.y -= (height - 1.0 * scale) / 2.0;

                    let r = op_rep_lim(p - pos, 0.32 * scale, vec2f(-1., 0.), vec2f(1., 0.));
                    let d = sdf_box2d(r, Vec2f::zero(), 0.06 * scale, height / 2.0);

                    if let Some(hit) = hit {
                        if hit.two_d {
                            hit.extrusion_length = h;
                        }
                    }

                    return d;
                }
                _ => {}
            }
        }

        f32::INFINITY
    }

    /// The 3D distance from the node to a point.
    pub fn distance_3d(
        &self,
        _time: &TheTime,
        p: Vec3f,
        hit: &mut Option<&mut Hit>,
        params: &[f32],
    ) -> f32 {
        // float opExtrusion( in vec3 p, in sdf2d primitive, in float h )
        // {
        //     float d = primitive(p.xy)
        //     vec2 w = vec2( d, abs(p.z) - h );
        //     return min(max(w.x,w.y),0.0) + length(max(w,0.0));
        // }

        match self.role {
            LeftWall => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.z, p.y),
                    vec2f(pos.y, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.z, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::X;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(pos.x.floor() + thick.fract(), 0.0, 0.0);
                }

                d
            }
            TopWall => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.x, p.y),
                    vec2f(pos.x, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.x, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::Z;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point =
                        p - vec3f(0.0, 0.0, pos.y.floor() + hit.extrusion_length.fract());
                }

                d
            }
            RightWall => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.z, p.y),
                    vec2f(pos.y, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.z, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::X;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(pos.x.floor() + 1.0 - thick.fract(), 0.0, 0.0);
                }

                d
            }
            BottomWall => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.x, p.y),
                    vec2f(pos.x, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.x, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::Z;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point =
                        p - vec3f(0.0, 0.0, pos.y.floor() + 1.0 - hit.extrusion_length.fract());
                }

                d
            }
            MiddleWallV => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.z, p.y),
                    vec2f(pos.y, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.z, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::X;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(pos.x, 0.0, 0.0);
                }

                d
            }
            MiddleWallH => {
                let thick = params[2] / 2.0;
                let len = params[3];
                let mut height = params[4];

                if let Some(hit) = hit {
                    if let Some(noise) = hit.noise {
                        height += ((noise * 2.) - 1.0) * hit.noise_scale;
                    }
                }

                let pos = vec2f(params[0], params[1]);
                let d = sdf_box2d(
                    vec2f(p.x, p.y),
                    vec2f(pos.x, height / 2.0),
                    len,
                    height / 2.0,
                );

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.x, p.y);
                    hit.extrusion = GeoFXNodeExtrusion::Z;
                    hit.extrusion_length = thick;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(0.0, 0.0, pos.y);
                }

                d
            }
            // BendWallNW => {
            //     let thick = params[2] / 2.0;
            //     let round = params[3];
            //     let height = params[4];

            //     let pos = vec2f(params[0], params[1]) + 1.0;
            //     let rounding = (round, round, round, round);

            //     let pp = vec2f(p.x, p.z) - pos;

            //     let size = 1.0;
            //     let mut d = sdf_rounded_box2d(pp, size, thick, rounding);

            //     d = d.abs() - thick;

            //     if let Some(hit) = hit {
            //         hit.pattern_pos = vec2f(p.x, p.z);
            //         hit.extrusion = GeoFXNodeExtrusion::Y;
            //         hit.extrusion_length = height;
            //         hit.interior_distance = d;
            //         hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
            //     }

            //     op_extrusion_y(p, d, height)
            //     // let plane = dot(p, vec3f(0.0, 1.0, 0.0));
            //     // max(-plane, d)
            // }
            // BendWallNE => {
            //     let thick = params[2] / 2.0;
            //     let round = params[3];
            //     let height = params[4];

            //     let pos = vec2f(params[0], params[1]) + vec2f(0.0, 1.0);
            //     let rounding = (round, round, round, round);

            //     let pp = vec2f(p.x, p.z) - pos;

            //     let size = 1.0;
            //     let mut d = sdf_rounded_box2d(pp, size, thick, rounding);

            //     d = d.abs() - thick;

            //     if let Some(hit) = hit {
            //         hit.pattern_pos = vec2f(p.x, p.z);
            //         hit.extrusion = GeoFXNodeExtrusion::Y;
            //         hit.extrusion_length = height;
            //         hit.interior_distance = d;
            //         hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
            //     }

            //     op_extrusion_y(p, d, height)
            //     // let plane = dot(p, vec3f(0.0, 1.0, 0.0));
            //     // max(-plane, d)
            // }
            // BendWallSW => {
            //     let thick = params[2] / 2.0;
            //     let round = params[3];
            //     let height = params[4];

            //     let pos = vec2f(params[0], params[1]) + vec2f(1.0, 0.0);
            //     let rounding = (round, round, round, round);

            //     let pp = vec2f(p.x, p.z) - pos;

            //     let size = 1.0;
            //     let mut d = sdf_rounded_box2d(pp, size, thick, rounding);

            //     d = d.abs() - thick;

            //     if let Some(hit) = hit {
            //         hit.pattern_pos = vec2f(p.x, p.z);
            //         hit.extrusion = GeoFXNodeExtrusion::Y;
            //         hit.extrusion_length = height;
            //         hit.interior_distance = d;
            //         hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
            //     }

            //     op_extrusion_y(p, d, height)
            //     // let plane = dot(p, vec3f(0.0, 1.0, 0.0));
            //     // max(-plane, d)
            // }
            // BendWallSE => {
            //     let thick = params[2] / 2.0;
            //     let round = params[3];
            //     let height = params[4];

            //     let pos = vec2f(params[0], params[1]);
            //     let rounding = (round, round, round, round);

            //     let pp = vec2f(p.x, p.z) - pos;

            //     let size = 1.0;
            //     let mut d = sdf_rounded_box2d(pp, size, thick, rounding);

            //     d = d.abs() - thick;

            //     if let Some(hit) = hit {
            //         hit.pattern_pos = vec2f(p.x, p.z);
            //         hit.extrusion = GeoFXNodeExtrusion::Y;
            //         hit.extrusion_length = height;
            //         hit.interior_distance = d;
            //         hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
            //     }

            //     op_extrusion_y(p, d, height)
            //     //let plane = dot(p, vec3f(0.0, 1.0, 0.0));
            //     //max(-plane, d)
            // }
            Column => {
                let radius = params[2];
                let height = params[3];
                let hole = params[4];

                let pos = vec2f(params[0], params[1]);
                let mut d = length(vec2f(p.x, p.z) - pos) - radius + hole;
                if hole > 0.0 {
                    d = d.abs() - hole;
                }

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.x, p.z);
                    hit.extrusion = GeoFXNodeExtrusion::Y;
                    hit.extrusion_length = height;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
                }

                d
            }
            Gate => {
                let pos = vec2f(params[0], params[1]);
                let align = params[2] as i32;
                let height = params[3];

                let r = if align == 0 {
                    op_rep_lim(vec2f(p.x, p.z) - pos, 0.32, vec2f(0., -1.), vec2f(0., 1.))
                } else {
                    op_rep_lim(vec2f(p.x, p.z) - pos, 0.32, vec2f(-1., 0.), vec2f(1., 0.))
                };
                let d = sdf_box2d(r, Vec2f::zero(), 0.06, 0.06);

                if let Some(hit) = hit {
                    hit.pattern_pos = vec2f(p.x, p.z);
                    hit.extrusion = GeoFXNodeExtrusion::Y;
                    hit.extrusion_length = height;
                    hit.interior_distance = d;
                    hit.hit_point = p - vec3f(pos.x.floor(), 0.0, pos.y.floor());
                }

                d
            }
            _ => f32::MAX,
        }
    }

    /// For ground nodes which edit the heightmap.
    pub fn heightmap_edit(&self, pos: &Vec2i, heightmap: &mut Heightmap) {
        match &self.role {
            GeoFXNodeRole::AddHeight => {
                if let Some(coll) = self
                    .timeline
                    .get_collection_at(&TheTime::default(), str!("Geo"))
                {
                    let add = coll.get_f32_default("Add Height", 0.2);
                    let height = heightmap.get_height(pos.x as f32, pos.y as f32);
                    heightmap.set_height(pos.x as f32, pos.y as f32, height + add);
                }
            }
            GeoFXNodeRole::RemoveHeight => {
                if let Some(coll) = self
                    .timeline
                    .get_collection_at(&TheTime::default(), str!("Geo"))
                {
                    let add = coll.get_f32_default("Remove Height", 0.2);
                    let height = heightmap.get_height(pos.x as f32, pos.y as f32);
                    heightmap.set_height(pos.x as f32, pos.y as f32, height - add);
                }
            }
            GeoFXNodeRole::SetHeight => {
                if let Some(coll) = self
                    .timeline
                    .get_collection_at(&TheTime::default(), str!("Geo"))
                {
                    let value = coll.get_f32_default("Height", 0.0);
                    heightmap.set_height(pos.x as f32, pos.y as f32, value);
                }
            }
            _ => {}
        }
    }

    /// Returns all tiles which are touched by this geometry.
    pub fn area(&self) -> Vec<Vec2i> {
        let mut area = Vec::new();
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            match self.role {
                Column => {
                    let radius = coll.get_f32_default("Radius", 0.4);

                    let center = self.position(&coll);
                    let min_x = (center.x - radius).floor() as i32;
                    let max_x = (center.x + radius).ceil() as i32;
                    let min_y = (center.y - radius).floor() as i32;
                    let max_y = (center.y + radius).ceil() as i32;

                    fn tile_intersects_disc(center: Vec2f, radius: f32, x: i32, y: i32) -> bool {
                        let closest_x = if center.x < x as f32 {
                            x as f32
                        } else if center.x > (x + 1) as f32 {
                            (x + 1) as f32
                        } else {
                            center.x
                        };
                        let closest_y = if center.y < y as f32 {
                            y as f32
                        } else if center.y > (y + 1) as f32 {
                            (y + 1) as f32
                        } else {
                            center.y
                        };

                        let dist_x = center.x - closest_x;
                        let dist_y = center.y - closest_y;

                        dist_x * dist_x + dist_y * dist_y <= radius * radius
                    }

                    for x in min_x..=max_x {
                        for y in min_y..=max_y {
                            if tile_intersects_disc(center, radius, x, y) {
                                area.push(Vec2i::new(x, y));
                            }
                        }
                    }
                }
                LeftWall | RightWall | MiddleWallV => {
                    let pos = Vec2i::from(self.position(&coll));
                    let length = self.length().ceil() as i32;
                    for i in 0..length {
                        area.push(Vec2i::new(pos.x, pos.y + i));
                    }
                }
                TopWall | BottomWall | MiddleWallH => {
                    let pos = Vec2i::from(self.position(&coll));
                    let length = self.length().ceil() as i32;
                    for i in 0..length {
                        area.push(Vec2i::new(pos.x + i, pos.y));
                    }
                }
                _ => {
                    area.push(Vec2i::from(self.position(&coll)));
                }
            }
        }
        area
    }

    /// Returns the length of the geometry.
    pub fn length(&self) -> f32 {
        let mut length = 1.0;
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            if let Some(h) = coll.get("Length") {
                if let Some(h) = h.to_f32() {
                    length = h;
                }
            }
        }
        length
    }

    /// Returns the height of the geometry.
    pub fn height(&self) -> f32 {
        let mut height = 1.0;
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            if let Some(h) = coll.get("Height") {
                if let Some(h) = h.to_f32() {
                    height = h;
                }
            }
        }
        height
    }

    /// Returns the thickness of the geometry.
    pub fn thickness(&self) -> f32 {
        let mut thickness = 0.2;
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            if let Some(h) = coll.get("Thickness") {
                if let Some(h) = h.to_f32() {
                    thickness = h;
                }
            }
        }
        thickness
    }

    #[inline(always)]
    pub fn position(&self, coll: &TheCollection) -> Vec2f {
        let x = coll.get_f32_default("Pos X", 0.0);
        let y = coll.get_f32_default("Pos Y", 0.0);
        vec2f(x, y)
    }

    /// Set the position
    pub fn set_position(&mut self, pos: Vec2f) {
        self.set("Pos X", TheValue::Float(pos.x));
        self.set("Pos Y", TheValue::Float(pos.y));
    }

    pub fn set_default_position(&mut self, p: Vec2i) {
        let pf = vec2f(p.x as f32, p.y as f32);
        /*
        match self.role {
            LeftWall => {
                pf.x += 0.1;
                pf.y += 0.5;
            }
            TopWall => {
                pf.x += 0.5;
                pf.y += 0.1;
            }
            RightWall => {
                pf.x += 0.9;
                pf.y += 0.5;
            }
            BottomWall => {
                pf.x += 0.5;
                pf.y += 0.9;
            }
            _ => {
                pf.x += 0.5;
                pf.y += 0.5;
            }
        }*/
        self.set("Pos X", TheValue::Float(pf.x));
        self.set("Pos Y", TheValue::Float(pf.y));
    }

    pub fn collection(&self) -> TheCollection {
        if let Some(coll) = self
            .timeline
            .get_collection_at(&TheTime::default(), str!("Geo"))
        {
            return coll;
        }

        TheCollection::default()
    }

    pub fn set(&mut self, key: &str, value: TheValue) {
        self.timeline.set(&TheTime::default(), key, "Geo", value);
    }

    pub fn is_blocking(&self) -> bool {
        // match self.role {
        //     RemoveHeightBrush => false,
        //     AddHeightBrush => {
        //         if let Some(coll) = self
        //             .timeline
        //             .get_collection_at(&TheTime::default(), str!("Geo"))
        //         {
        //             let height = coll.get_f32_default("Height", 0.01);
        //             height > 0.3
        //         } else {
        //             false
        //         }
        //     }
        //     _ => true,
        // }
        true
    }

    pub fn get_icon_description(&self) -> Option<&str> {
        match self.role {
            GeoFXNodeRole::AddHeight => Some("+"),
            GeoFXNodeRole::RemoveHeight => Some("-"),
            GeoFXNodeRole::SetHeight => Some("="),
            _ => None,
        }
    }

    pub fn inputs(&self) -> Vec<TheNodeTerminal> {
        match self.role {
            LeftWall | TopWall | RightWall | BottomWall | MiddleWallH | MiddleWallV => {
                vec![]
            }
            Bricks | Box | Disc | Material | Repeat | Stack | Group => {
                vec![TheNodeTerminal {
                    name: str!("in"),
                    role: str!("In"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            _ => {
                vec![]
            }
        }
    }

    pub fn outputs(
        &self,
        index: &usize,
        connections: &[(u16, u8, u16, u8)],
    ) -> Vec<TheNodeTerminal> {
        let mut highest_output_terminal: i32 = 0;
        #[allow(clippy::collapsible_if)]
        for (s, st, _, _) in connections {
            if *s as usize == *index {
                if *st as i32 + 1 > highest_output_terminal {
                    highest_output_terminal = *st as i32 + 1;
                }
            }
        }
        highest_output_terminal += 1;

        match self.role {
            LeftWall | TopWall | RightWall | BottomWall | MiddleWallH | MiddleWallV => {
                let mut terminals = vec![];
                for i in 1..=highest_output_terminal {
                    terminals.push(TheNodeTerminal {
                        name: format!("layer #{}", i),
                        role: format!("Layer #{}", i),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    });
                }
                terminals
            }
            Bricks | Box | Disc => {
                vec![
                    TheNodeTerminal {
                        name: str!("out"),
                        role: str!("Out"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                    TheNodeTerminal {
                        name: str!("mat"),
                        role: str!("Mat"),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    },
                ]
            }
            Repeat | Group => {
                let mut terminals = vec![];
                for i in 1..=highest_output_terminal {
                    terminals.push(TheNodeTerminal {
                        name: format!("shape #{}", i),
                        role: format!("Shape #{}", i),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    });
                }
                terminals
            }
            Stack => {
                let mut terminals = vec![];
                for i in 1..=highest_output_terminal {
                    terminals.push(TheNodeTerminal {
                        name: format!("row #{}", i),
                        role: format!("Row #{}", i),
                        color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                    });
                }
                terminals
            }
            MetaMaterial => {
                vec![TheNodeTerminal {
                    name: str!("mat"),
                    role: str!("Mat"),
                    color: TheColor::new(0.5, 0.5, 0.5, 1.0),
                }]
            }
            _ => vec![],
        }
    }

    /// Palette index has been changed. If we are a material, adjust the color.
    pub fn set_palette_index(&mut self, index: u16) -> bool {
        if self.role == GeoFXNodeRole::Material {
            self.set("Color", TheValue::PaletteIndex(index));
            true
        } else {
            false
        }
    }

    pub fn preview(
        &self,
        buffer: &mut TheRGBABuffer,
        material: Option<&MaterialFXObject>,
        palette: &ThePalette,
        tiles: &FxHashMap<Uuid, TheRGBATile>,
        coord: Vec2f,
        ctx: &mut TheContext,
    ) {
        if let Some(text) = self.get_icon_description() {
            if let Some(font) = &ctx.ui.font {
                buffer.fill([0, 0, 0, 0]);
                buffer.draw_text(
                    vec2i(0, 0),
                    font,
                    text,
                    25.0,
                    WHITE,
                    TheHorizontalAlign::Center,
                    TheVerticalAlign::Center,
                )
            }
            return;
        }

        fn mix_color(a: &[u8; 4], b: &[u8; 4], v: f32) -> [u8; 4] {
            [
                (((1.0 - v) * (a[0] as f32 / 255.0) + b[0] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[1] as f32 / 255.0) + b[1] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[2] as f32 / 255.0) + b[2] as f32 / 255.0 * v) * 255.0) as u8,
                (((1.0 - v) * (a[3] as f32 / 255.0) + b[3] as f32 / 255.0 * v) * 255.0) as u8,
            ]
        }

        let width = buffer.dim().width as usize;
        let height = buffer.dim().height;

        let time = TheTime::default();

        let mut mat_obj_params: Vec<Vec<f32>> = vec![];

        if let Some(material) = material {
            mat_obj_params = material.load_parameters(&time);
        }

        buffer
            .pixels_mut()
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * width + i;

                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    let mut hit = Hit {
                        two_d: true,
                        ..Default::default()
                    };

                    let p = vec2f(x / width as f32, 1.0 - y / height as f32);
                    let p_coord = p + coord;
                    hit.uv = p;
                    hit.global_uv = p_coord;
                    hit.pattern_pos = p_coord;
                    hit.hit_point = vec3f(p.x + coord.x, 0.0, p.y + coord.y);
                    hit.normal = vec3f(0.0, 1.0, 0.0);
                    let d = self.distance(&time, p_coord, 1.0, &mut Some(&mut hit));
                    hit.distance = d;

                    if let Some(material) = material {
                        material.follow_geo_trail(&TheTime::default(), &mut hit, &mat_obj_params);
                        if hit.interior_distance <= 0.01 {
                            hit.value = 0.0;
                        } else {
                            hit.value = 1.0;
                        }
                        material.compute(&mut hit, palette, tiles, &mat_obj_params);
                    };

                    let t = smoothstep(-0.04, 0.0, d);

                    let color = if material.is_some() {
                        TheColor::from_vec3f(hit.mat.base_color).to_u8_array()
                    } else {
                        [209, 209, 209, 255]
                    };
                    pixel.copy_from_slice(&mix_color(&color, &[81, 81, 81, 255], t));
                }
            });
    }
}
