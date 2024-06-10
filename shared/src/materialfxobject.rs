use crate::prelude::*;
use rayon::prelude::*;
use theframework::prelude::*;

/// A character instance.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct MaterialFXObject {
    pub id: Uuid,

    pub name: String,

    /// The nodes which make up the material.
    pub nodes: Vec<MaterialFXNode>,

    /// The node connections: Source node index, source terminal, dest node index, dest terminal
    pub connections: Vec<(u16, u8, u16, u8)>,

    #[serde(skip)]
    pub node_previews: Vec<Option<TheRGBABuffer>>,

    pub zoom: f32,
    pub selected_node: Option<usize>,

    pub preview: TheRGBABuffer,

    #[serde(default = "Vec2i::zero")]
    pub scroll_offset: Vec2i,
}

impl Default for MaterialFXObject {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialFXObject {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),

            name: "New Material".to_string(),

            nodes: Vec::new(),
            connections: Vec::new(),

            node_previews: Vec::new(),
            zoom: 1.0,
            selected_node: None,

            preview: TheRGBABuffer::empty(),

            scroll_offset: Vec2i::zero(),
        }
    }

    /// Computes the material
    pub fn compute(&self, hit: &mut Hit, palette: &ThePalette) {
        for (i, node) in self.nodes.iter().enumerate() {
            if node.role == MaterialFXNodeRole::Geometry {
                self.follow_trail(i, 0, hit, palette);
                break;
            }
        }
    }

    /// Computes the displacement if any
    pub fn displacement(&self, hit: &mut Hit) {
        for (i, node) in self.nodes.iter().enumerate() {
            if node.role == MaterialFXNodeRole::Geometry {
                if let Some((node, _)) = self.find_connected_input_node(i, 2) {
                    self.nodes[node as usize].displacement(hit);
                }
                break;
            }
        }
    }

    /// Returns true if the material supports displacement
    pub fn has_displacement(&self) -> bool {
        for (i, node) in self.nodes.iter().enumerate() {
            if node.role == MaterialFXNodeRole::Geometry {
                if let Some((_, _)) = self.find_connected_input_node(i, 2) {
                    return true;
                }
            }
        }
        false
    }

    /// Returns the connected input node and terminal for the given output node and terminal.
    pub fn find_connected_input_node(
        &self,
        node: usize,
        terminal_index: usize,
    ) -> Option<(u16, u8)> {
        for (o, ot, i, it) in &self.connections {
            if *o == node as u16 && *ot == terminal_index as u8 {
                return Some((*i, *it));
            }
        }
        None
    }

    /// Returns the connected output node for the given input node and terminal.
    pub fn find_connected_output_node(&self, node: usize, terminal_index: usize) -> Option<usize> {
        for (o, _, i, it) in &self.connections {
            if *i == node as u16 && *it == terminal_index as u8 {
                return Some(*o as usize);
            }
        }
        None
    }

    /// After exiting a geometry node follow the trail of material nodes to calculate the final color.
    pub fn follow_trail(
        &self,
        node: usize,
        terminal_index: usize,
        hit: &mut Hit,
        palette: &ThePalette,
    ) {
        let mut connections = vec![];

        for (o, ot, i, it) in &self.connections {
            if *o == node as u16 && *ot == terminal_index as u8 {
                connections.push((*i, *it));
            }
        }

        match connections.len() {
            0 => {}
            1 => {
                let o = connections[0].0 as usize;

                /*
                let mut noise = 0.0;
                if let Some(noise_index) = self.find_connected_output_node(o, 1) {
                    if let ModelFXNode::Noise3D(_coll) = &self.nodes[noise_index] {
                        noise = self.nodes[noise_index].noise(hit);
                        hit.uv += 7.23;
                        let noise2 = self.nodes[noise_index].noise(hit);
                        let wobble = vec2f(noise, noise2);
                        hit.uv -= 7.23;
                        hit.uv += wobble * 0.5;
                    }
                }

                */
                if let Some(ot) = self.nodes[o].compute(hit, palette) {
                    self.follow_trail(o, ot as usize, hit, palette);
                }
            }
            _ => {
                let index = (hit.hash * connections.len() as f32).floor() as usize;
                if let Some(random_connection) = connections.get(index) {
                    let o = random_connection.0 as usize;
                    /*
                    let mut noise = 0.0;
                    if let Some(noise_index) = self.find_connected_output_node(o, 1) {
                        if let ModelFXNode::Noise3D(_coll) = &self.nodes[noise_index] {
                            noise = self.nodes[noise_index].noise(hit);
                        }
                        }*/
                    if let Some(ot) = self.nodes[o].compute(hit, palette) {
                        self.follow_trail(o, ot as usize, hit, palette);
                    }
                }
            }
        }
    }

    /// Convert the model to a node canvas.
    pub fn to_canvas(&mut self, _palette: &ThePalette) -> TheNodeCanvas {
        let mut canvas = TheNodeCanvas {
            node_width: 95,
            selected_node: self.selected_node,
            ..Default::default()
        };

        let preview_size = (40.0 * self.zoom) as i32;

        for (i, node) in self.nodes.iter().enumerate() {
            if i >= self.node_previews.len() {
                self.node_previews.resize(i + 1, None);
            }

            // Remove preview buffer if size has changed
            if let Some(preview_buffer) = &self.node_previews[i] {
                if preview_buffer.dim().width != preview_size
                    && preview_buffer.dim().height != preview_size
                {
                    self.node_previews[i] = None;
                }
            }

            // Create preview if it doesn't exist
            if self.node_previews[i].is_none() {
                let preview_buffer = TheRGBABuffer::new(TheDim::sized(preview_size, preview_size));
                //self.render_node_preview(&mut preview_buffer, i, palette);
                self.node_previews[i] = Some(preview_buffer);
            }

            let n = TheNode {
                name: node.name(),
                position: node.position,
                inputs: node.inputs(),
                outputs: node.outputs(),
                preview: self.node_previews[i].clone().unwrap(),
            };
            canvas.nodes.push(n);
        }
        canvas.connections.clone_from(&self.connections);
        canvas.zoom = self.zoom;
        canvas.offset = self.scroll_offset;
        canvas.selected_node = self.selected_node;

        canvas
    }

    pub fn render_preview(&mut self, palette: &ThePalette) {
        let size: usize = 48;
        let mut buffer = TheRGBABuffer::new(TheDim::sized(size as i32, size as i32));

        fn distance(p: Vec3f) -> f32 {
            length(p) - 1.8
        }

        pub fn normal(p: Vec3f) -> Vec3f {
            let scale = 0.5773 * 0.0005;
            let e = vec2f(1.0 * scale, -1.0 * scale);

            // IQs normal function

            let e1 = vec3f(e.x, e.y, e.y);
            let e2 = vec3f(e.y, e.y, e.x);
            let e3 = vec3f(e.y, e.x, e.y);
            let e4 = vec3f(e.x, e.x, e.x);

            let n = e1 * distance(p + e1)
                + e2 * distance(p + e2)
                + e3 * distance(p + e3)
                + e4 * distance(p + e4);
            normalize(n)
        }

        let ro = vec3f(2.0, 2.0, 2.0);
        let rd = vec3f(0.0, 0.0, 0.0);

        let aa = 2;
        let aa_f = aa as f32;

        let camera = Camera::new(ro, rd, 80.0);
        let bgc = 74.0 / 255.0;

        buffer
            .pixels_mut()
            .par_rchunks_exact_mut(size * 4)
            .enumerate()
            .for_each(|(j, line)| {
                let mut hit = Hit::default();

                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * size + i;

                    let xx = (i % size) as f32;
                    let yy = (i / size) as f32;

                    let mut total = Vec4f::zero();

                    for m in 0..aa {
                        for n in 0..aa {
                            let camera_offset =
                                vec2f(m as f32 / aa_f, n as f32 / aa_f) - vec2f(0.5, 0.5);

                            let mut color = vec4f(bgc, bgc, bgc, 1.0);

                            let ray = camera.create_ray(
                                vec2f(xx / size as f32, 1.0 - yy / size as f32),
                                vec2f(size as f32, size as f32),
                                camera_offset,
                            );

                            let mut t = 0.001;

                            for _ in 0..20 {
                                let p = ray.at(t);
                                let d = distance(p);
                                if d.abs() < 0.001 {
                                    hit.hit_point = p;
                                    hit.normal = normal(p);

                                    for (i, node) in self.nodes.iter().enumerate() {
                                        if node.role == MaterialFXNodeRole::Geometry {
                                            self.follow_trail(i, 0, &mut hit, palette);

                                            color.x += hit.albedo.x;
                                            color.y += hit.albedo.y;
                                            color.z += hit.albedo.z;
                                            break;
                                        }
                                    }

                                    break;
                                }
                                t += d;
                            }
                            total += color;
                        }
                    }

                    let aa_aa = aa_f * aa_f;
                    total[0] /= aa_aa;
                    total[1] /= aa_aa;
                    total[2] /= aa_aa;
                    total[3] /= aa_aa;

                    pixel.copy_from_slice(&TheColor::from_vec4f(total).to_u8_array());
                }
            });
        self.preview = buffer;
    }

    /// Load a model from a JSON string.
    pub fn from_json(json: &str) -> Self {
        let mut material: MaterialFXObject = serde_json::from_str(json).unwrap_or_default();
        let cnt = material.nodes.len();
        for _ in 0..cnt {
            material.node_previews.push(None);
        }
        material
    }

    /// Convert the model to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
