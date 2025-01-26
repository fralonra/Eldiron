use crate::prelude::*;

use crate::editor::UNDOMANAGER;

pub struct ToolList {
    pub server_time: TheTime,
    pub render_button_text: String,

    pub game_tools: Vec<Box<dyn Tool>>,
    pub curr_game_tool: usize,

    pub char_click_selected: bool,
    pub char_click_pos: Vec2<f32>,

    drag_changed: bool,
    undo_map: Map,
}

impl Default for ToolList {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolList {
    pub fn new() -> Self {
        let game_tools: Vec<Box<dyn Tool>> = vec![
            Box::new(SelectionTool::new()),
            //Box::new(PickerTool::new()),
            Box::new(VertexTool::new()),
            Box::new(LinedefTool::new()),
            Box::new(SectorTool::new()),
            // Box::new(DrawTool::new()),
            // Box::new(MapObjectsTool::new()),
            Box::new(FXTool::new()),
            Box::new(CodeTool::new()),
            // Box::new(EraserTool::new()),
            Box::new(TilemapTool::new()),
            // Box::new(ZoomTool::new()),
            //Box::new(ResizeTool::new()),
            Box::new(GameTool::new()),
        ];
        Self {
            server_time: TheTime::default(),
            render_button_text: "Finished".to_string(),

            game_tools,
            curr_game_tool: 2,

            char_click_selected: false,
            char_click_pos: Vec2::zero(),

            drag_changed: false,
            undo_map: Map::default(),
        }
    }

    /// Build the UI
    pub fn set_active_editor(&mut self, list: &mut dyn TheVLayoutTrait, ctx: &mut TheContext) {
        list.clear();
        ctx.ui.relayout = true;

        for (index, tool) in self.game_tools.iter().enumerate() {
            let mut b = TheToolListButton::new(tool.id());

            b.set_icon_name(tool.icon_name());
            b.set_status_text(&tool.info());
            if index == self.curr_game_tool {
                b.set_state(TheWidgetState::Selected);
            }
            list.add_widget(Box::new(b));
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// If the map has been changed, update its context and add an undo.
    fn update_map_context(
        &mut self,
        _ui: &mut TheUI,
        ctx: &mut TheContext,
        _project: &mut Project,
        server_ctx: &mut ServerContext,
        undo_atom: Option<RegionUndoAtom>,
    ) {
        if server_ctx.curr_map_context == MapContext::Region {
            if let Some(undo_atom) = undo_atom {
                UNDOMANAGER.lock().unwrap().add_region_undo(
                    &server_ctx.curr_region,
                    undo_atom,
                    ctx,
                );
                crate::editor::RUSTERIX.lock().unwrap().set_dirty();
            }
        } else if server_ctx.curr_map_context == MapContext::Material {
            if let Some(undo_atom) = undo_atom {
                if let Some(material_undo_atom) = undo_atom.to_material_atom() {
                    UNDOMANAGER
                        .lock()
                        .unwrap()
                        .add_material_undo(material_undo_atom, ctx);
                    crate::editor::RUSTERIX.lock().unwrap().set_dirty();
                    ctx.ui.send(TheEvent::Custom(
                        TheId::named("Update Materialpicker"),
                        TheValue::Empty,
                    ));
                }
            }
        }
    }

    pub fn draw_hud(
        &mut self,
        buffer: &mut TheRGBABuffer,
        map: &mut Map,
        ctx: &mut TheContext,
        server_ctx: &mut ServerContext,
    ) {
        self.game_tools[self.curr_game_tool].draw_hud(buffer, map, ctx, server_ctx);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn handle_event(
        &mut self,
        event: &TheEvent,
        ui: &mut TheUI,
        ctx: &mut TheContext,
        project: &mut Project,
        server_ctx: &mut ServerContext,
    ) -> bool {
        let mut redraw = false;
        match event {
            TheEvent::KeyDown(TheValue::Char(c)) => {
                if let Some(id) = &ctx.ui.focus {
                    if id.name == "PolyView" {
                        if let Some(map) = project.get_map_mut(server_ctx) {
                            let undo_atom = self.get_current_tool().map_event(
                                MapEvent::MapKey(*c),
                                ui,
                                ctx,
                                map,
                                server_ctx,
                            );
                            self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                        }
                    }
                }

                // let mut acc = !ui.focus_widget_supports_text_input(ctx);

                // if self.get_current_tool().id().name == "Game Tool" {
                //     acc = false;
                // }

                /*
                if acc {
                    if (*c == '-' || *c == '=' || *c == '+') && (ui.ctrl || ui.logo) {
                        // Global Zoom In / Zoom Out
                        if let Some(region) = project.get_region_mut(&server_ctx.curr_region) {
                            if *c == '=' || *c == '+' {
                                region.zoom += 0.2;
                            } else {
                                region.zoom -= 0.2;
                            }
                            region.zoom = region.zoom.clamp(1.0, 5.0);
                            if let Some(layout) = ui.get_rgba_layout("Region Editor") {
                                layout.set_zoom(region.zoom);
                                layout.relayout(ctx);
                            }
                            if let Some(edit) = ui.get_text_line_edit("Editor Zoom") {
                                edit.set_value(TheValue::Float(region.zoom));
                            }
                            return true;
                        }
                    }

                    let mut tool_uuid = None;
                    for tool in self.game_tools.iter() {
                        if tool.accel() == Some(*c) {
                            tool_uuid = Some(tool.id().uuid);
                            ctx.ui.set_widget_state(
                                self.game_tools[self.curr_game_tool].id().name,
                                TheWidgetState::None,
                            );
                            ctx.ui
                                .set_widget_state(tool.id().name, TheWidgetState::Selected);
                        }
                    }
                    if let Some(uuid) = tool_uuid {
                        self.set_tool(uuid, ui, ctx, project, server_ctx);
                    }
                }*/
            }
            TheEvent::StateChanged(id, state) => {
                if id.name.contains("Tool") && *state == TheWidgetState::Selected {
                    redraw = self.set_tool(id.uuid, ui, ctx, project, server_ctx);
                }
            }
            TheEvent::KeyCodeDown(TheValue::KeyCode(code)) => {
                if let Some(id) = &ctx.ui.focus {
                    if id.name == "PolyView" {
                        if *code == TheKeyCode::Escape {
                            if let Some(map) = project.get_map_mut(server_ctx) {
                                let undo_atom = self.get_current_tool().map_event(
                                    MapEvent::MapEscape,
                                    ui,
                                    ctx,
                                    map,
                                    server_ctx,
                                );
                                self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                            }
                        } else if *code == TheKeyCode::Delete {
                            if let Some(map) = project.get_map_mut(server_ctx) {
                                if server_ctx.curr_map_context == MapContext::Region
                                    && server_ctx.curr_map_tool_type != MapToolType::Effects
                                    && map.selected_entity.is_some()
                                {
                                    ctx.ui.send(TheEvent::ContextMenuSelected(
                                        TheId::empty(),
                                        TheId::named("Sidebar Delete Character Instance"),
                                    ));
                                    return false;
                                }
                                let undo_atom = self.get_current_tool().map_event(
                                    MapEvent::MapDelete,
                                    ui,
                                    ctx,
                                    map,
                                    server_ctx,
                                );
                                self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                            }
                        }
                    }
                }
            }
            TheEvent::RenderViewClicked(id, coord) => {
                self.char_click_selected = false;

                if let Some(map) = project.get_map_mut(server_ctx) {
                    // Test for character click
                    if let Some(render_view) = ui.get_render_view("PolyView") {
                        let dim = *render_view.dim();

                        let grid_pos = server_ctx.local_to_map_grid(
                            Vec2::new(dim.width as f32, dim.height as f32),
                            Vec2::new(coord.x as f32, coord.y as f32),
                            map,
                            map.subdivisions,
                        );

                        if server_ctx.curr_map_context == MapContext::Region
                            && server_ctx.curr_map_tool_type != MapToolType::Effects
                            && id.name == "PolyView"
                        {
                            self.char_click_pos = grid_pos;

                            for entity in map.entities.iter().cloned() {
                                let ep = entity.position;
                                let ep = Vec2::new(ep.x, ep.z);
                                let d = ep.distance(grid_pos);
                                if d < 1.0 {
                                    let prev = map.clone();
                                    self.undo_map = map.clone();
                                    self.char_click_selected = true;
                                    self.drag_changed = false;
                                    if map.selected_entity != Some(entity.creator_id) {
                                        map.clear_selection();
                                        map.selected_entity = Some(entity.creator_id);
                                        let undo_atom = RegionUndoAtom::MapEdit(
                                            Box::new(prev),
                                            Box::new(map.clone()),
                                        );
                                        UNDOMANAGER.lock().unwrap().add_region_undo(
                                            &server_ctx.curr_region,
                                            undo_atom,
                                            ctx,
                                        );
                                        if let Some(layout) =
                                            ui.get_list_layout("Region Content List")
                                        {
                                            layout.select_item(entity.creator_id, ctx, true);
                                        }
                                        ctx.ui.send(TheEvent::Custom(
                                            TheId::named("Map Selection Changed"),
                                            TheValue::Empty,
                                        ));
                                        crate::editor::RUSTERIX.lock().unwrap().set_dirty();
                                    }
                                    return true;
                                }
                            }
                        }

                        let undo_atom = self.get_current_tool().map_event(
                            MapEvent::MapClicked(*coord),
                            ui,
                            ctx,
                            map,
                            server_ctx,
                        );
                        self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                    }
                    redraw = true;
                }
            }
            TheEvent::RenderViewDragged(id, coord) => {
                if id.name == "PolyView" {
                    if let Some(map) = project.get_map_mut(server_ctx) {
                        if self.char_click_selected {
                            // Dragging selected lines
                            if let Some(render_view) = ui.get_render_view("PolyView") {
                                let dim = *render_view.dim();

                                let drag_pos = server_ctx.local_to_map_grid(
                                    Vec2::new(dim.width as f32, dim.height as f32),
                                    Vec2::new(coord.x as f32, coord.y as f32),
                                    map,
                                    map.subdivisions,
                                );

                                let drag_delta = self.char_click_pos - drag_pos;

                                for entity in map.entities.iter_mut() {
                                    if Some(entity.creator_id) == map.selected_entity {
                                        let new_pos = Vec2::new(
                                            self.char_click_pos.x - drag_delta.x,
                                            self.char_click_pos.y - drag_delta.y,
                                        );
                                        entity.position.x = new_pos.x;
                                        entity.position.z = new_pos.y;

                                        self.drag_changed = self.char_click_pos.x != new_pos.x
                                            || self.char_click_pos.y != new_pos.y;
                                    }
                                }

                                crate::editor::RUSTERIX.lock().unwrap().set_dirty();
                                return true;
                            }
                        }

                        let undo_atom = self.get_current_tool().map_event(
                            MapEvent::MapDragged(*coord),
                            ui,
                            ctx,
                            map,
                            server_ctx,
                        );
                        self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                    }
                    redraw = true;
                }
            }
            TheEvent::RenderViewUp(id, coord) => {
                if id.name == "PolyView" {
                    if let Some(map) = project.get_map_mut(server_ctx) {
                        if self.char_click_selected && self.drag_changed {
                            let undo_atom = RegionUndoAtom::MapEdit(
                                Box::new(self.undo_map.clone()),
                                Box::new(map.clone()),
                            );
                            UNDOMANAGER.lock().unwrap().add_region_undo(
                                &server_ctx.curr_region,
                                undo_atom,
                                ctx,
                            );

                            self.char_click_selected = false;
                            return true;
                        }

                        let undo_atom = self.get_current_tool().map_event(
                            MapEvent::MapUp(*coord),
                            ui,
                            ctx,
                            map,
                            server_ctx,
                        );
                        self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                    }
                    redraw = true;
                }
            }
            TheEvent::RenderViewHoverChanged(id, coord) => {
                if id.name == "PolyView" {
                    if let Some(map) = project.get_map_mut(server_ctx) {
                        let undo_atom = self.get_current_tool().map_event(
                            MapEvent::MapHover(*coord),
                            ui,
                            ctx,
                            map,
                            server_ctx,
                        );
                        self.update_map_context(ui, ctx, project, server_ctx, undo_atom);
                    }
                    redraw = true;
                }
            }
            /*
            TheEvent::TileEditorClicked(id, coord) => {
                if id.name == "Region Editor View"
                    || id.name == "Screen Editor View"
                    || id.name == "TerrainMap View"
                {
                    let mut coord_f = Vec2f::from(*coord);
                    if id.name == "Region Editor View" {
                        if let Some(editor) = ui.get_rgba_layout("Region Editor") {
                            if let Some(rgba_view) = editor.rgba_view_mut().as_rgba_view() {
                                coord_f = rgba_view.float_pos();
                            }
                        }
                    }

                    self.get_current_tool().tool_event(
                        ToolEvent::TileDown(*coord, coord_f),
                        ToolContext::TwoD,
                        ui,
                        ctx,
                        project,
                        server,
                        client,
                        server_ctx,
                    );
                }
            }
            TheEvent::TileEditorDragged(id, coord) => {
                if id.name == "Region Editor View"
                    || id.name == "Screen Editor View"
                    || id.name == "TerrainMap View"
                {
                    let mut coord_f = Vec2f::from(*coord);
                    if id.name == "Region Editor View" {
                        if let Some(editor) = ui.get_rgba_layout("Region Editor") {
                            if let Some(rgba_view) = editor.rgba_view_mut().as_rgba_view() {
                                coord_f = rgba_view.float_pos();
                            }
                        }
                    }

                    self.get_current_tool().tool_event(
                        ToolEvent::TileDrag(*coord, coord_f),
                        ToolContext::TwoD,
                        ui,
                        ctx,
                        project,
                        server,
                        client,
                        server_ctx,
                    );
                }
            }
            TheEvent::TileEditorUp(id) => {
                if id.name == "Region Editor View"
                    || id.name == "Screen Editor View"
                    || id.name == "TerrainMap View"
                {
                    self.get_current_tool().tool_event(
                        ToolEvent::TileUp,
                        ToolContext::TwoD,
                        ui,
                        ctx,
                        project,
                        server,
                        client,
                        server_ctx,
                    );
                }
            }
            TheEvent::RenderViewClicked(id, coord) => {
                if id.name == "PolyView" {
                    // if let Some(render_view) = ui.get_render_view("PolyView") {
                    // let dim = render_view.dim();
                    // if let Some(region) = project.get_region_mut(&server_ctx.curr_region) {
                    // let pos = RENDERER.lock().unwrap().get_hit_position_at(
                    //     *coord,
                    //     region,
                    //     &mut server.get_instance_draw_settings(server_ctx.curr_region),
                    //     dim.width as usize,
                    //     dim.height as usize,
                    // );
                    //
                    let pos = Some((*coord, *coord));

                    if let Some((pos, _)) = pos {
                        redraw = self.get_current_tool().tool_event(
                            ToolEvent::TileDown(
                                vec2i(pos.x, pos.y),
                                vec2f(pos.x as f32, pos.y as f32),
                            ),
                            ToolContext::ThreeD,
                            ui,
                            ctx,
                            project,
                            server,
                            client,
                            server_ctx,
                        );
                    }
                    // }
                    // }
                }
            }
            TheEvent::RenderViewDragged(id, coord) => {
                if id.name == "PolyView" {
                    //if let Some(render_view) = ui.get_render_view("RenderView") {
                    //let dim = render_view.dim();
                    //if let Some(region) = project.get_region_mut(&server_ctx.curr_region) {
                    // let pos = RENDERER.lock().unwrap().get_hit_position_at(
                    //     *coord,
                    //     region,
                    //     &mut server.get_instance_draw_settings(server_ctx.curr_region),
                    //     dim.width as usize,
                    //     dim.height as usize,
                    // );

                    let pos = Some((*coord, *coord));

                    if let Some((pos, _)) = pos {
                        redraw = self.get_current_tool().tool_event(
                            ToolEvent::TileDrag(
                                vec2i(pos.x, pos.y),
                                vec2f(pos.x as f32, pos.y as f32),
                            ),
                            ToolContext::ThreeD,
                            ui,
                            ctx,
                            project,
                            server,
                            client,
                            server_ctx,
                        );
                    }
                    //}
                    //}
                }
            }*/
            // TheEvent::ContextMenuSelected(widget_id, item_id) => {
            //     if widget_id.name == "Render Button" {
            //         if let Some(region) = project.get_region_mut(&server_ctx.curr_region) {
            //             if item_id.name == "Start Renderer" {
            //                 PRERENDERTHREAD.lock().unwrap().set_paused(false);
            //             } else if item_id.name == "Pause Renderer" {
            //                 PRERENDERTHREAD.lock().unwrap().set_paused(true);
            //             } else if item_id.name == "Restart Renderer" {
            //                 PRERENDERTHREAD.lock().unwrap().set_paused(false);
            //                 PRERENDERTHREAD
            //                     .lock()
            //                     .unwrap()
            //                     .render_region(region.clone(), None);
            //             }
            //             redraw = true;
            //         }
            //     }
            // }
            TheEvent::Custom(id, value) => {
                if id.name == "Set Game Tool" {
                    if let TheValue::Text(name) = value {
                        if let Some(tool_id) = self.get_game_tool_uuid_of_name(name) {
                            self.set_tool(tool_id, ui, ctx, project, server_ctx);
                        }
                    }
                }
            }
            _ => {}
        }

        if !redraw {
            redraw = self
                .get_current_tool()
                .handle_event(event, ui, ctx, project, server_ctx);
        }

        redraw
    }

    /// Returns the curently active tool.
    pub fn get_current_tool(&mut self) -> &mut Box<dyn Tool> {
        &mut self.game_tools[self.curr_game_tool]
    }

    #[allow(clippy::too_many_arguments)]
    pub fn deactivte_tool(
        &mut self,
        ui: &mut TheUI,
        ctx: &mut TheContext,
        project: &mut Project,
        server_ctx: &mut ServerContext,
    ) {
        self.game_tools[self.curr_game_tool].tool_event(
            ToolEvent::DeActivate,
            ToolContext::TwoD,
            ui,
            ctx,
            project,
            server_ctx,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_tool(
        &mut self,
        tool_id: Uuid,
        ui: &mut TheUI,
        ctx: &mut TheContext,
        project: &mut Project,
        server_ctx: &mut ServerContext,
    ) -> bool {
        let mut redraw = false;
        let mut switched_tool = false;
        let layout_name = "Game Tool Params";
        let mut old_tool_index = 0;
        for (index, tool) in self.game_tools.iter().enumerate() {
            if tool.id().uuid == tool_id && index != self.curr_game_tool {
                switched_tool = true;
                old_tool_index = self.curr_game_tool;
                self.curr_game_tool = index;
                redraw = true;
            }
        }
        if switched_tool {
            for tool in self.game_tools.iter() {
                if tool.id().uuid != tool_id {
                    ctx.ui
                        .set_widget_state(tool.id().name.clone(), TheWidgetState::None);
                }
            }
            self.game_tools[old_tool_index].tool_event(
                ToolEvent::DeActivate,
                ToolContext::TwoD,
                ui,
                ctx,
                project,
                server_ctx,
            );
        }

        if let Some(layout) = ui.get_hlayout(layout_name) {
            layout.clear();
            layout.set_reverse_index(None);
            ctx.ui.redraw_all = true;
        }

        self.get_current_tool().tool_event(
            ToolEvent::Activate,
            ToolContext::TwoD,
            ui,
            ctx,
            project,
            server_ctx,
        );

        crate::editor::RUSTERIX.lock().unwrap().set_dirty();

        /*
        if let Some(layout) = ui.get_hlayout(layout_name) {
            if layout.widgets().is_empty() {
                // Add default widgets

                // let mut gb = TheGroupButton::new(TheId::named("2D3D Group"));
                // gb.add_text("2D Map".to_string());
                // gb.add_text("Mixed".to_string());
                // gb.add_text("3D Map".to_string());

                // match *RENDERMODE.lock().unwrap() {
                //     EditorDrawMode::Draw2D => gb.set_index(0),
                //     EditorDrawMode::DrawMixed => gb.set_index(1),
                //     EditorDrawMode::Draw3D => gb.set_index(2),
                // }

                // let mut time_slider = TheTimeSlider::new(TheId::named("Server Time Slider"));
                // time_slider.set_continuous(true);
                // time_slider.limiter_mut().set_max_width(400);
                // time_slider.set_value(TheValue::Time(self.server_time));

                let mut spacer = TheSpacer::new(TheId::empty());
                spacer.limiter_mut().set_max_width(30);

                let mut render_button = TheTraybarButton::new(TheId::named("Render Button"));
                render_button.set_text(self.render_button_text.clone());
                render_button.set_status_text("Controls the 3D background renderer. During rendering it displays how many tiles are left to render.");
                render_button.set_fixed_size(true);
                render_button.limiter_mut().set_max_width(80);

                render_button.set_context_menu(Some(TheContextMenu {
                    items: vec![
                        TheContextMenuItem::new(
                            "Start Renderer".to_string(),
                            TheId::named("Start Renderer"),
                        ),
                        TheContextMenuItem::new(
                            "Pause".to_string(),
                            TheId::named("Pause Renderer"),
                        ),
                        TheContextMenuItem::new(
                            "Restart".to_string(),
                            TheId::named("Restart Renderer"),
                        ),
                    ],
                    ..Default::default()
                }));

                //layout.add_widget(Box::new(gb));
                layout.add_widget(Box::new(spacer));
                //layout.add_widget(Box::new(time_slider));
                layout.add_widget(Box::new(render_button));
                layout.set_reverse_index(Some(1));
            }
        }*/

        ctx.ui.relayout = true;

        redraw
    }

    // Return the uuid given game tool.
    pub fn get_game_tool_uuid_of_name(&self, name: &str) -> Option<Uuid> {
        for tool in self.game_tools.iter() {
            if tool.id().name == name {
                return Some(tool.id().uuid);
            }
        }
        None
    }
}
