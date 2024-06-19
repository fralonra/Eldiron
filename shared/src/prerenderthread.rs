use crate::prelude::*;
use rayon::ThreadPoolBuilder;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use theframework::prelude::*;
pub enum PreRenderCmd {
    SetTextures(FxHashMap<Uuid, TheRGBATile>),
    SetMaterials(IndexMap<Uuid, MaterialFXObject>),
    SetPalette(ThePalette),
    MaterialChanged(MaterialFXObject),
    RenderRegionCoordTree(Region),
    RenderRegion(Region, Option<Vec<Vec2i>>),
    Quit,
}

pub enum PreRenderResult {
    RenderedRegion(Uuid, PreRendered),
    RenderedRegionTile(
        Uuid,
        Vec2i,
        Vec2i,
        TheRGBBuffer,
        TheRGBBuffer,
        TheFlattenedMap<f32>,
        TheFlattenedMap<Vec<PreRenderedLight>>,
    ),
    RenderedRTree(Uuid, RTree<PreRenderedData>),
    Quit,
}

#[derive(Debug)]
pub struct PreRenderThread {
    pub tx: Option<mpsc::Sender<PreRenderCmd>>,

    pub rx: Option<mpsc::Receiver<PreRenderResult>>,

    renderer_thread: Option<JoinHandle<()>>,
}

impl Default for PreRenderThread {
    fn default() -> Self {
        Self::new()
    }
}

impl PreRenderThread {
    pub fn new() -> Self {
        Self {
            tx: None,
            rx: None,
            renderer_thread: None,
        }
    }

    /// Check for a renderer result
    pub fn receive(&self) -> Option<PreRenderResult> {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                return Some(result);
            }
        }

        None
    }

    /// Send a cmd to the renderer.
    pub fn send(&self, cmd: PreRenderCmd) {
        if let Some(tx) = &self.tx {
            tx.send(cmd).unwrap();
        }
    }

    pub fn set_textures(&self, textures: FxHashMap<Uuid, TheRGBATile>) {
        self.send(PreRenderCmd::SetTextures(textures));
    }

    pub fn set_materials(&self, materials: IndexMap<Uuid, MaterialFXObject>) {
        self.send(PreRenderCmd::SetMaterials(materials));
    }

    pub fn set_palette(&self, palette: ThePalette) {
        self.send(PreRenderCmd::SetPalette(palette));
    }

    pub fn material_changed(&self, material: MaterialFXObject) {
        self.send(PreRenderCmd::MaterialChanged(material));
    }

    pub fn render_region(&self, region: Region, tiles: Option<Vec<Vec2i>>) {
        self.send(PreRenderCmd::RenderRegion(region, tiles));
    }

    pub fn render_region_coord_tree(&self, region: Region) {
        self.send(PreRenderCmd::RenderRegionCoordTree(region));
    }

    pub fn startup(&mut self) {
        let (tx, rx) = mpsc::channel::<PreRenderCmd>();

        self.tx = Some(tx);

        let (result_tx, result_rx) = mpsc::channel::<PreRenderResult>();

        self.rx = Some(result_rx);

        let renderer_thread = thread::spawn(move || {
            // We allocate half of the available cpus to the background pool
            let cpus = num_cpus::get();
            let background_pool = ThreadPoolBuilder::new()
                .num_threads(cpus / 2)
                .build()
                .unwrap();

            let mut renderer = Renderer::new();
            let mut palette = ThePalette::default();

            let mut draw_settings = RegionDrawSettings::new();
            draw_settings.daylight = vec3f(1.0, 1.0, 1.0);

            let mut prerendered_regions: FxHashMap<Uuid, PreRendered> = FxHashMap::default();

            loop {
                if let Ok(cmd) = rx.try_recv() {
                    match cmd {
                        PreRenderCmd::SetTextures(new_textures) => {
                            println!("PreRenderCmd::SetTextures");
                            renderer.set_textures(new_textures.clone());
                        }
                        PreRenderCmd::SetMaterials(new_materials) => {
                            println!("PreRenderCmd::SetMaterials");
                            renderer.materials.clone_from(&new_materials);
                        }
                        PreRenderCmd::SetPalette(new_palette) => {
                            println!("PreRenderCmd::SetPalette");
                            palette = new_palette;
                        }
                        PreRenderCmd::MaterialChanged(changed_material) => {
                            println!("PreRenderCmd::MaterialChanged");
                            renderer
                                .materials
                                .insert(changed_material.id, changed_material);
                        }
                        PreRenderCmd::RenderRegionCoordTree(region) => {
                            println!("PreRenderCmd::RenderRegionCoordTree");

                            let w = (region.width as f32 * region.grid_size as f32) as i32;
                            let h = (region.height as f32 * region.grid_size as f32) as i32;

                            renderer.set_region(&region);
                            renderer.position =
                                vec3f(region.width as f32 / 2.0, 0.0, region.height as f32 / 2.0);

                            let mut prerendered = PreRendered::new(
                                TheRGBBuffer::new(TheDim::sized(w, h)),
                                TheRGBBuffer::new(TheDim::sized(w, h)),
                            );
                            prerendered.add_all_tiles(region.grid_size);

                            background_pool.install(|| {
                                renderer.prerender_rtree(
                                    &mut prerendered,
                                    &region,
                                    &mut draw_settings,
                                );
                            });

                            prerendered_regions.insert(region.id, prerendered.clone());

                            result_tx
                                .send(PreRenderResult::RenderedRTree(region.id, prerendered.tree))
                                .unwrap();
                            println!("finished");
                        }
                        PreRenderCmd::RenderRegion(region, tiles) => {
                            println!("PreRenderCmd::RenderRegion");

                            let w = (region.width as f32 * region.grid_size as f32) as i32;
                            let h = (region.height as f32 * region.grid_size as f32) as i32;

                            renderer.set_region(&region);
                            renderer.position =
                                vec3f(region.width as f32 / 2.0, 0.0, region.height as f32 / 2.0);

                            let mut reset = false;

                            if region.prerendered.albedo.dim().width != w
                                || region.prerendered.albedo.dim().height != h
                                || tiles.is_none()
                            {
                                reset = true;
                            }

                            let mut prerendered = if reset {
                                let mut prerendered = PreRendered::new(
                                    TheRGBBuffer::new(TheDim::sized(w, h)),
                                    TheRGBBuffer::new(TheDim::sized(w, h)),
                                );
                                if let Some(pre) = prerendered_regions.get(&region.id) {
                                    prerendered.tree = pre.tree.clone();
                                }

                                // Flush the queue
                                while rx.try_recv().is_ok() {
                                    println!("skipped");
                                }

                                prerendered.add_all_tiles(region.grid_size);
                                prerendered
                            } else {
                                let mut prerendered =
                                    if let Some(pre) = prerendered_regions.get(&region.id) {
                                        pre.clone()
                                    } else {
                                        region.prerendered.clone()
                                    };
                                if let Some(tiles) = tiles {
                                    prerendered.add_tiles(tiles, region.grid_size);
                                }
                                prerendered
                            };

                            println!(
                                "tiles_to_render: {:?}, size {}",
                                prerendered.tiles_to_render.len(),
                                vec2i(w, h)
                            );

                            if !prerendered.tiles_to_render.is_empty() {
                                background_pool.install(|| {
                                    renderer.prerender(
                                        &mut prerendered,
                                        &region,
                                        &mut draw_settings,
                                        &palette,
                                        result_tx.clone(),
                                    );
                                });

                                prerendered.tiles_to_render.clear();

                                prerendered_regions.insert(region.id, prerendered.clone());

                                // result_tx
                                //     .send(PreRenderResult::RenderedRegion(region.id, prerendered))
                                //     .unwrap();
                                println!("finished");
                            }
                        }
                        PreRenderCmd::Quit => {
                            println!("PreRenderCmd::Quit");
                            break;
                        }
                    }
                }
            }

            println!("Renderer thread exiting")
        });
        self.renderer_thread = Some(renderer_thread);
    }
}
