use crate::editor::MODELFXEDITOR;
use crate::prelude::*;

pub struct MaterialNodeEditTool {
    id: TheId,
}

impl Tool for MaterialNodeEditTool {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            id: TheId::named("Edit Tool (E)."),
        }
    }

    fn id(&self) -> TheId {
        self.id.clone()
    }
    fn info(&self) -> String {
        str!("Edit Tool (E). Edit the nodes of the Material.")
    }
    fn icon_name(&self) -> String {
        str!("picker")
    }
    fn accel(&self) -> Option<char> {
        Some('e')
    }

    fn tool_event(
        &mut self,
        tool_event: ToolEvent,
        _tool_context: ToolContext,
        ui: &mut TheUI,
        _ctx: &mut TheContext,
        project: &mut Project,
        _server: &mut Server,
        _client: &mut Client,
        server_ctx: &mut ServerContext,
    ) -> bool {
        if let ToolEvent::Activate = tool_event {
            MODELFXEDITOR.lock().unwrap().set_geometry_mode(false);

            // Set the current material
            if let Some(material_id) = server_ctx.curr_material_object {
                if let Some(material) = project.materials.get_mut(&material_id) {
                    let node_canvas = material.to_canvas(&project.palette);
                    ui.set_node_canvas("MaterialFX NodeCanvas", node_canvas);
                }
            }

            if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                layout.set_mode(TheSharedVLayoutMode::Top);
            }

            if let Some(layout) = ui.get_hlayout("Material Tool Params") {
                layout.clear();
                let mut material_button = TheTraybarButton::new(TheId::named("MaterialFX Nodes"));
                material_button.set_text(str!("Material Nodes"));
                material_button.set_status_text(
                    "Nodes which model walls and components like windows, doors and decoration.",
                );

                material_button.set_context_menu(Some(TheContextMenu {
                    items: vec![
                        TheContextMenuItem::new_submenu(
                            "Patterns".to_string(),
                            TheId::named("Extrusion Patterns"),
                            TheContextMenu {
                                items: vec![
                                    TheContextMenuItem::new(
                                        "Box Subdivision".to_string(),
                                        TheId::named("Box Subdivision"),
                                    ),
                                    TheContextMenuItem::new(
                                        "Bricks & Tiles".to_string(),
                                        TheId::named("Bricks"),
                                    ),
                                    //TheContextMenuItem::new("Tiles".to_string(), TheId::named("Tiles")),
                                ],
                                ..Default::default()
                            },
                        ),
                        TheContextMenuItem::new_submenu(
                            "Utility".to_string(),
                            TheId::named("MaterialFX Nodes Patterns"),
                            TheContextMenu {
                                items: vec![TheContextMenuItem::new(
                                    "Distance".to_string(),
                                    TheId::named("Distance"),
                                )],
                                ..Default::default()
                            },
                        ),
                        TheContextMenuItem::new(
                            "UV Splitter".to_string(),
                            TheId::named("UV Splitter"),
                        ),
                        TheContextMenuItem::new("Noise2D".to_string(), TheId::named("Noise2D")),
                        TheContextMenuItem::new("Noise3D".to_string(), TheId::named("Noise3D")),
                        //TheContextMenuItem::new("Geometry".to_string(), TheId::named("Geometry")),
                        TheContextMenuItem::new(
                            "Material Mixer".to_string(),
                            TheId::named("Material Mixer"),
                        ),
                        TheContextMenuItem::new("Material".to_string(), TheId::named("Material")),
                    ],
                    ..Default::default()
                }));

                layout.add_widget(Box::new(material_button));
                layout.set_reverse_index(Some(1));
            }
        } else if let ToolEvent::DeActivate = tool_event {
            if let Some(layout) = ui.get_hlayout("Material Tool Params") {
                layout.clear();
                layout.set_reverse_index(None);
            }
            if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                layout.set_mode(TheSharedVLayoutMode::Shared);
            }
            MODELFXEDITOR.lock().unwrap().set_geometry_mode(true);
        }
        false
    }

    /*
    #[allow(clippy::too_many_arguments)]
    fn handle_event(
        &mut self,
        event: &TheEvent,
        _ui: &mut TheUI,
        _ctx: &mut TheContext,
        _project: &mut Project,
        _server: &mut Server,
        _client: &mut Client,
        _server_ctx: &mut ServerContext,
    ) -> bool {
        let redraw = false;
        match event {
            // TheEvent::StateChanged(id, TheWidgetState::Selected) => {
            //     if id.name ==
            // }
            //
            _ => {}
        }

        redraw
    }*/
}
