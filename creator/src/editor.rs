use crate::prelude::*;
use crate::self_update::SelfUpdateEvent;
use crate::self_update::SelfUpdater;
use crate::Embedded;
use rusterix::{PlayerCamera, Rusterix, Texture, Value, ValueContainer};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use std::thread;

pub static TILEPICKER: LazyLock<RwLock<TilePicker>> =
    LazyLock::new(|| RwLock::new(TilePicker::new("Main Tile Picker".to_string())));
pub static MATERIALPICKER: LazyLock<RwLock<MaterialPicker>> =
    LazyLock::new(|| RwLock::new(MaterialPicker::new("Main Material Picker".to_string())));
pub static EFFECTPICKER: LazyLock<RwLock<EffectPicker>> =
    LazyLock::new(|| RwLock::new(EffectPicker::new("Main Effect Picker".to_string())));
pub static TILEMAPEDITOR: LazyLock<RwLock<TilemapEditor>> =
    LazyLock::new(|| RwLock::new(TilemapEditor::new()));
pub static SIDEBARMODE: LazyLock<RwLock<SidebarMode>> =
    LazyLock::new(|| RwLock::new(SidebarMode::Region));
pub static UNDOMANAGER: LazyLock<RwLock<UndoManager>> =
    LazyLock::new(|| RwLock::new(UndoManager::default()));
pub static TOOLLIST: LazyLock<RwLock<ToolList>> =
    LazyLock::new(|| RwLock::new(ToolList::default()));
pub static PANELS: LazyLock<RwLock<Panels>> = LazyLock::new(|| RwLock::new(Panels::new()));
pub static TEXTEDITOR: LazyLock<RwLock<TextEditor>> =
    LazyLock::new(|| RwLock::new(TextEditor::new()));
pub static PALETTE: LazyLock<RwLock<ThePalette>> =
    LazyLock::new(|| RwLock::new(ThePalette::default()));
pub static RUSTERIX: LazyLock<RwLock<Rusterix>> =
    LazyLock::new(|| RwLock::new(Rusterix::default()));
pub static SETTINGSPICKER: LazyLock<RwLock<SettingsPicker>> =
    LazyLock::new(|| RwLock::new(SettingsPicker::new("Main Settings Picker".to_string())));
pub static PREVIEWVIEW: LazyLock<RwLock<PreviewView>> =
    LazyLock::new(|| RwLock::new(PreviewView::new()));

pub struct Editor {
    project: Project,
    project_path: Option<PathBuf>,

    sidebar: Sidebar,
    mapeditor: MapEditor,

    server_ctx: ServerContext,

    update_tracker: UpdateTracker,
    event_receiver: Option<Receiver<TheEvent>>,

    self_update_rx: Receiver<SelfUpdateEvent>,
    self_update_tx: Sender<SelfUpdateEvent>,
    self_updater: Arc<Mutex<SelfUpdater>>,

    first_update: bool,

    build_values: ValueContainer,
}

impl TheTrait for Editor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let (self_update_tx, self_update_rx) = channel();

        Self {
            project: Project::new(),
            project_path: None,

            sidebar: Sidebar::new(),
            mapeditor: MapEditor::new(),

            server_ctx: ServerContext::default(),

            update_tracker: UpdateTracker::new(),
            event_receiver: None,

            self_update_rx,
            self_update_tx,
            self_updater: Arc::new(Mutex::new(SelfUpdater::new(
                "markusmoenig",
                "Eldiron",
                "eldiron",
            ))),

            first_update: true,

            build_values: ValueContainer::default(),
        }
    }

    fn init(&mut self, _ctx: &mut TheContext) {
        let updater = Arc::clone(&self.self_updater);
        let tx = self.self_update_tx.clone();

        thread::spawn(move || {
            let mut updater = updater.lock().unwrap();

            if let Err(err) = updater.fetch_release_list() {
                tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                    .unwrap();
            };
        });
    }

    fn window_title(&self) -> String {
        "Eldiron Creator".to_string()
    }

    fn default_window_size(&self) -> (usize, usize) {
        (1200, 720)
    }

    fn window_icon(&self) -> Option<(Vec<u8>, u32, u32)> {
        if let Some(file) = Embedded::get("window_logo.png") {
            let data = std::io::Cursor::new(file.data);

            let decoder = png::Decoder::new(data);
            if let Ok(mut reader) = decoder.read_info() {
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                let bytes = &buf[..info.buffer_size()];

                Some((bytes.to_vec(), info.width, info.height))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn init_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) {
        // Embedded Icons
        for file in Embedded::iter() {
            let name = file.as_ref();

            if name.ends_with(".png") {
                if let Some(file) = Embedded::get(name) {
                    let data = std::io::Cursor::new(file.data);

                    let decoder = png::Decoder::new(data);
                    if let Ok(mut reader) = decoder.read_info() {
                        let mut buf = vec![0; reader.output_buffer_size()];
                        let info = reader.next_frame(&mut buf).unwrap();
                        let bytes = &buf[..info.buffer_size()];

                        let mut cut_name = name.replace("icons/", "");
                        cut_name = cut_name.replace(".png", "");

                        ctx.ui.add_icon(
                            cut_name.to_string(),
                            TheRGBABuffer::from(bytes.to_vec(), info.width, info.height),
                        );
                    }
                }
            }
        }

        // ---

        ui.set_statusbar_name("Statusbar".to_string());

        // Menu

        let mut menu_canvas = TheCanvas::new();
        let mut menu = TheMenu::new(TheId::named("Menu"));

        let mut file_menu = TheContextMenu::named(str!("File"));
        file_menu.add(TheContextMenuItem::new(
            str!("Open..."),
            TheId::named("Open"),
        ));
        file_menu.add(TheContextMenuItem::new(str!("Save"), TheId::named("Save")));
        file_menu.add(TheContextMenuItem::new(
            str!("Save As ..."),
            TheId::named("Save As"),
        ));
        let mut edit_menu = TheContextMenu::named(str!("Edit"));
        edit_menu.add(TheContextMenuItem::new_with_accel(
            str!("Undo"),
            TheId::named("Undo"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'z'),
        ));
        edit_menu.add(TheContextMenuItem::new_with_accel(
            str!("Redo"),
            TheId::named("Redo"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD | TheAcceleratorKey::SHIFT, 'z'),
        ));
        edit_menu.add_separator();
        edit_menu.add(TheContextMenuItem::new_with_accel(
            str!("Cut"),
            TheId::named("Cut"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'x'),
        ));
        edit_menu.add(TheContextMenuItem::new_with_accel(
            str!("Copy"),
            TheId::named("Copy"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'c'),
        ));
        edit_menu.add(TheContextMenuItem::new_with_accel(
            str!("Paste"),
            TheId::named("Paste"),
            TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'v'),
        ));
        // let mut view_menu = TheContextMenu::named(str!("View"));
        // view_menu.add(TheContextMenuItem::new_with_accel(
        //     str!("2D Map"),
        //     TheId::named("2DMap"),
        //     TheAccelerator::new(TheAcceleratorKey::CTRLCMD, '2'),
        // ));
        // view_menu.add(TheContextMenuItem::new_with_accel(
        //     str!("3D Map"),
        //     TheId::named("3DMap"),
        //     TheAccelerator::new(TheAcceleratorKey::CTRLCMD, '3'),
        // ));
        // view_menu.add(TheContextMenuItem::new_with_accel(
        //     str!("Shared Map"),
        //     TheId::named("2D3DMap"),
        //     TheAccelerator::new(TheAcceleratorKey::CTRLCMD, '0'),
        // ));
        // let mut tools_menu = TheContextMenu::named(str!("Tools"));
        // tools_menu.add(TheContextMenuItem::new_with_accel(
        //     str!("Rerender 3D Map"),
        //     TheId::named("Rerender"),
        //     TheAccelerator::new(TheAcceleratorKey::CTRLCMD, 'r'),
        // ));

        edit_menu.register_accel(ctx);
        // view_menu.register_accel(ctx);
        // tools_menu.register_accel(ctx);

        menu.add_context_menu(file_menu);
        menu.add_context_menu(edit_menu);
        menu_canvas.set_widget(menu);

        // Menubar
        let mut top_canvas = TheCanvas::new();

        let mut menubar = TheMenubar::new(TheId::named("Menubar"));
        menubar.limiter_mut().set_max_height(43 + 22);

        let mut logo_button = TheMenubarButton::new(TheId::named("Logo"));
        logo_button.set_icon_name("logo".to_string());
        logo_button.set_status_text("Open the Eldiron Website ...");

        let mut open_button = TheMenubarButton::new(TheId::named("Open"));
        open_button.set_icon_name("icon_role_load".to_string());
        open_button.set_status_text("Open an existing Eldiron project...");

        let mut save_button = TheMenubarButton::new(TheId::named("Save"));
        save_button.set_status_text("Save the current project.");
        save_button.set_icon_name("icon_role_save".to_string());

        let mut save_as_button = TheMenubarButton::new(TheId::named("Save As"));
        save_as_button.set_icon_name("icon_role_save_as".to_string());
        save_as_button.set_status_text("Save the current project to a new file.");
        save_as_button.set_icon_offset(Vec2::new(2, -5));

        let mut undo_button = TheMenubarButton::new(TheId::named("Undo"));
        undo_button.set_status_text("Undo the last action.");
        undo_button.set_icon_name("icon_role_undo".to_string());

        let mut redo_button = TheMenubarButton::new(TheId::named("Redo"));
        redo_button.set_status_text("Redo the last action.");
        redo_button.set_icon_name("icon_role_redo".to_string());

        let mut play_button = TheMenubarButton::new(TheId::named("Play"));
        play_button.set_status_text("Start the server for live editing and debugging.");
        play_button.set_icon_name("play".to_string());
        //play_button.set_fixed_size(vec2i(28, 28));

        let mut pause_button = TheMenubarButton::new(TheId::named("Pause"));
        pause_button.set_status_text("Pause. Click for single stepping the server.");
        pause_button.set_icon_name("play-pause".to_string());

        let mut stop_button = TheMenubarButton::new(TheId::named("Stop"));
        stop_button.set_status_text("Stop the server.");
        stop_button.set_icon_name("stop-fill".to_string());

        let mut time_slider = TheTimeSlider::new(TheId::named("Server Time Slider"));
        time_slider.set_continuous(true);
        time_slider.limiter_mut().set_max_width(400);
        time_slider.set_value(TheValue::Time(TheTime::default()));

        let mut update_button = TheMenubarButton::new(TheId::named("Update"));
        update_button.set_status_text("Update application.");
        update_button.set_icon_name("arrows-clockwise".to_string());

        let mut patreon_button = TheMenubarButton::new(TheId::named("Patreon"));
        patreon_button.set_status_text("Visit my Patreon page.");
        patreon_button.set_icon_name("patreon".to_string());
        // patreon_button.set_fixed_size(vec2i(36, 36));
        patreon_button.set_icon_offset(Vec2::new(-4, -2));

        let mut hlayout = TheHLayout::new(TheId::named("Menu Layout"));
        hlayout.set_background_color(None);
        hlayout.set_margin(Vec4::new(10, 2, 10, 1));
        hlayout.add_widget(Box::new(logo_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(open_button));
        hlayout.add_widget(Box::new(save_button));
        hlayout.add_widget(Box::new(save_as_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(undo_button));
        hlayout.add_widget(Box::new(redo_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(play_button));
        hlayout.add_widget(Box::new(pause_button));
        hlayout.add_widget(Box::new(stop_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(time_slider));
        //hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));

        hlayout.add_widget(Box::new(update_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(patreon_button));

        hlayout.set_reverse_index(Some(3));

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);
        top_canvas.set_top(menu_canvas);
        ui.canvas.set_top(top_canvas);

        ctx.ui.send(TheEvent::Custom(
            TheId::named("Set Game Tool"),
            TheValue::Text(str!("Tile Drawer Tool")),
        ));

        // Sidebar
        self.sidebar.init_ui(ui, ctx, &mut self.project);

        // Panels
        let bottom_panels =
            PANELS
                .write()
                .unwrap()
                .init_ui(ui, ctx, &mut self.project, &mut self.server_ctx);

        // Editor
        //let mut tab_canvas: TheCanvas = TheCanvas::new();
        //let mut tab_layout = TheTabLayout::new(TheId::named("Editor Tab"));

        let poly_canvas = self.mapeditor.init_ui(ui, ctx, &mut self.project);

        //tab_layout.add_canvas(str!("Game View"), game_canvas);

        // let model_canvas: TheCanvas =
        //     MODELEDITOR
        //         .lock()
        //         .unwrap()
        //         .init_ui(ui, ctx, &mut self.project);
        // tab_layout.add_canvas(str!("Model View"), model_canvas);

        /*
        let material_canvas = self.materialeditor.init_ui(ui, ctx, &mut self.project);
        tab_layout.add_canvas(str!("Material View"), material_canvas);

        let screen_canvas = self.screeneditor.init_ui(ui, ctx, &mut self.project);
        tab_layout.add_canvas(str!("Screen View"), screen_canvas);

        tab_canvas.set_layout(tab_layout);
        */
        let mut vsplitlayout = TheSharedVLayout::new(TheId::named("Shared VLayout"));
        vsplitlayout.add_canvas(poly_canvas);
        vsplitlayout.add_canvas(bottom_panels);
        vsplitlayout.set_shared_ratio(crate::DEFAULT_VLAYOUT_RATIO);
        vsplitlayout.set_mode(TheSharedVLayoutMode::Shared);

        let mut shared_canvas = TheCanvas::new();
        shared_canvas.set_layout(vsplitlayout);

        // Tool List
        let mut tool_list_canvas: TheCanvas = TheCanvas::new();

        let mut tool_list_bar_canvas = TheCanvas::new();
        tool_list_bar_canvas.set_widget(TheToolListBar::new(TheId::empty()));
        tool_list_canvas.set_top(tool_list_bar_canvas);

        let mut v_tool_list_layout = TheVLayout::new(TheId::named("Tool List Layout"));
        v_tool_list_layout.limiter_mut().set_max_width(51);
        v_tool_list_layout.set_margin(Vec4::new(2, 2, 2, 2));
        v_tool_list_layout.set_padding(1);

        TOOLLIST
            .write()
            .unwrap()
            .set_active_editor(&mut v_tool_list_layout, ctx);

        tool_list_canvas.set_layout(v_tool_list_layout);

        let mut tool_list_border_canvas = TheCanvas::new();
        let mut border_widget = TheIconView::new(TheId::empty());
        border_widget.set_border_color(Some([82, 82, 82, 255]));
        border_widget.limiter_mut().set_max_width(1);
        border_widget.limiter_mut().set_max_height(i32::MAX);
        tool_list_border_canvas.set_widget(border_widget);

        tool_list_canvas.set_right(tool_list_border_canvas);
        shared_canvas.set_left(tool_list_canvas);

        ui.canvas.set_center(shared_canvas);

        let mut status_canvas = TheCanvas::new();
        let mut statusbar = TheStatusbar::new(TheId::named("Statusbar"));
        statusbar.set_text(
            "Welcome to Eldiron! Visit Eldiron.com for information and example projects."
                .to_string(),
        );
        status_canvas.set_widget(statusbar);

        ui.canvas.set_bottom(status_canvas);

        // -

        // ctx.ui.set_disabled("Save");
        // ctx.ui.set_disabled("Save As");
        ctx.ui.set_disabled("Undo");
        ctx.ui.set_disabled("Redo");

        // Init Rusterix

        if let Some(icon) = ctx.ui.icon("light_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values.set("light_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("light_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values.set("light_off", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("character_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("character_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("character_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("character_off", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("treasure_on") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("treasure_on", Value::Texture(texture));
        }
        if let Some(icon) = ctx.ui.icon("treasure_off") {
            let texture = Texture::from_rgbabuffer(icon);
            self.build_values
                .set("treasure_off", Value::Texture(texture));
        }

        RUSTERIX.write().unwrap().set_d2();

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));
    }

    /// Set the command line arguments
    fn set_cmd_line_args(&mut self, args: Vec<String>, ctx: &mut TheContext) {
        if args.len() > 1 {
            #[allow(irrefutable_let_patterns)]
            if let Ok(path) = PathBuf::from_str(&args[1]) {
                ctx.ui.send(TheEvent::FileRequesterResult(
                    TheId::named("Open"),
                    vec![path],
                ));
            }
        }
    }

    /// Handle UI events and UI state
    fn update_ui(&mut self, ui: &mut TheUI, ctx: &mut TheContext) -> bool {
        let mut redraw = false;
        let mut update_server_icons = false;

        if self.first_update {
            let mut toollist = TOOLLIST.write().unwrap();
            let id = toollist.get_current_tool().id().uuid;

            toollist.set_tool(id, ui, ctx, &mut self.project, &mut self.server_ctx);
            self.first_update = false;
        }

        let (redraw_update, tick_update) = self.update_tracker.update(
            (1000 / self.project.settings.get_i32_value("renderFPS", 30)) as u64,
            self.project.settings.get_i32_value("projectTickMs", 250) as u64,
        );

        if tick_update {
            RUSTERIX.write().unwrap().client.inc_animation_frame();

            // Update the widgets which have animations
            if let Some(icon_view) = ui.get_widget("Tilemap Selection Preview") {
                if let Some(icon_view) = icon_view.as_icon_view() {
                    icon_view.step();
                    redraw = true;
                }
            }
        }

        /*
        if tick_update {
            // Update the widgets which have anims (if they are visible)
            if let Some(icon_view) = ui.get_widget("Global Icon Preview") {
                if let Some(icon_view) = icon_view.as_icon_view() {
                    icon_view.step();
                    redraw = true;
                }
            }
            if let Some(icon_view) = ui.get_widget("Icon Preview") {
                if let Some(icon_view) = icon_view.as_icon_view() {
                    icon_view.step();
                    redraw = true;
                }
            }
            if let Some(icon_view) = ui.get_widget("Tilemap Selection Preview") {
                if let Some(icon_view) = icon_view.as_icon_view() {
                    icon_view.step();
                    redraw = true;
                }
            }

            if self.server.state == ServerState::Running {
                // Ticks
                self.client
                    .tick(*ACTIVEEDITOR.lock().unwrap() == ActiveEditor::GameEditor);
                let debug = self.server.tick();
                if !debug.is_empty() {
                    self.sidebar.add_debug_messages(debug, ui, ctx);
                }

                // Get the messages for the client from the server.
                let client_messages = self.server.get_client_messages();
                for cmd in client_messages {
                    self.client.process_server_message(&cmd);
                }

                // Get the messages for the server from the client.
                let server_messages = self.client.get_server_messages();
                for cmd in server_messages {
                    self.server.execute_client_cmd(self.client.id, cmd);
                }

                let interactions = self.server.get_interactions();
                self.server_ctx.add_interactions(interactions);
                PANELS.lock().unwrap().update_code_object(
                    ui,
                    ctx,
                    &mut self.server,
                    &mut self.server_ctx,
                );
                if let Some(update) = self
                    .server
                    .get_region_update_json(self.server_ctx.curr_region)
                {
                    self.client.set_region_update(update);
                }

                if let Some(widget) = ui.get_widget("Server Time Slider") {
                    widget.set_value(TheValue::Time(self.server.world.time));
                    TOOLLIST.lock().unwrap().server_time = self.server.world.time;
                }
            }

            // Set Debug Data

            let mut debug_entity: Option<Uuid> = None;
            if let Some(id) = self.server_ctx.curr_character_instance {
                debug_entity = Some(id);
            } else if let Some(id) = self.server_ctx.curr_area {
                debug_entity = Some(id);
            }

            if let Some(debug_entity) = debug_entity {
                let mut debug_has_set = false;

                if let Some(debug) = self
                    .server
                    .get_entity_debug_data(self.server_ctx.curr_region, debug_entity)
                {
                    let editor_codegrid_id = CODEEDITOR.lock().unwrap().get_codegrid_id(ui);
                    for debug in debug.values() {
                        if debug.codegrid_id == editor_codegrid_id {
                            CODEEDITOR
                                .lock()
                                .unwrap()
                                .set_debug_module(debug.clone(), ui);
                            debug_has_set = true;
                            break;
                        }
                    }
                }

                if !debug_has_set {
                    CODEEDITOR
                        .lock()
                        .unwrap()
                        .set_debug_module(TheDebugModule::default(), ui);
                }
            }
        }*/

        /*
        // Get prerendered results
        {
            let mut renderer = RENDERER.lock().unwrap();

            let mut counter = 0;
            while let Some(rendered) = PRERENDERTHREAD.lock().unwrap().receive() {
                match rendered {
                    PreRenderResult::MaterialPreviewRendered(_, buffer) => {
                        ui.set_node_overlay("MaterialFX NodeCanvas", Some(buffer));
                    }
                    PreRenderResult::ClearRegionTile(id, tile) => {
                        if let Some(region) = self.project.get_region_mut(&id) {
                            region.prerendered.clear_tile_albedo(&tile);
                            self.server.clear_prerendered_tile(id, &tile);
                        }
                    }
                    PreRenderResult::RenderedRegionTile(id, tile, sample, tile_data) => {
                        if let Some(region) = self.project.get_region_mut(&id) {
                            region.prerendered.merge_tile_data(
                                region.tile_size,
                                &tile,
                                sample,
                                &tile_data,
                            );
                            self.server
                                .set_prerendered_tile(id, &tile, sample, &tile_data);
                        }
                        redraw = true;
                    }
                    PreRenderResult::Clear(id) => {
                        if let Some(region) = self.project.get_region_mut(&id) {
                            self.server.clear_prerendered(region.id);
                            region.prerendered.clear();
                            renderer.canvas.canvas.fill([0, 0, 0]);
                            renderer.canvas.distance_canvas.clear();
                            renderer.canvas.lights_canvas.clear();
                        }
                    }
                    PreRenderResult::UpdateMiniMap => {
                        if let Some(region) =
                            self.project.get_region_mut(&self.server_ctx.curr_region)
                        {
                            if let Some(render_view) = ui.get_render_view("MiniMap") {
                                let dim = *render_view.dim();
                                let buffer = render_view.render_buffer_mut();
                                buffer.resize(dim.width, dim.height);
                                draw_minimap(region, buffer);
                            }
                        }
                    }
                    PreRenderResult::Progress(id) => {
                        if let Some(region) = self.project.get_region_mut(&id) {
                            renderer.render_canvas(region);
                        }
                    }
                    PreRenderResult::Paused => {
                        TOOLLIST.lock().unwrap().render_button_text = "Paused".to_string();
                        ui.set_widget_value("Render Button", ctx, TheValue::Text(str!("Paused")));
                    }
                    PreRenderResult::Finished => {
                        TOOLLIST.lock().unwrap().render_button_text = "Finished".to_string();
                        ui.set_widget_value("Render Button", ctx, TheValue::Text(str!("Finished")));
                    }
                    _ => {}
                }
                counter += 1;
                if counter > 50 {
                    break;
                }
            }
        }*/

        if redraw_update && !self.project.regions.is_empty() {
            // let render_mode = *RENDERMODE.lock().unwrap();

            let sample_mode = self.project.settings.get_i32_value("renderSampleMode", 0);
            match sample_mode {
                0 => {
                    self.build_values.set(
                        "sample_mode",
                        Value::SampleMode(rusterix::SampleMode::Nearest),
                    );
                }
                1 => {
                    self.build_values.set(
                        "sample_mode",
                        Value::SampleMode(rusterix::SampleMode::Linear),
                    );
                }
                _ => {}
            }

            // Update entities when the server is running
            {
                let rusterix = &mut RUSTERIX.write().unwrap();
                if rusterix.server.state == rusterix::ServerState::Running {
                    rusterix.server.update();
                    if rusterix.server.log_changed {
                        ui.set_widget_value(
                            "LogEdit",
                            ctx,
                            TheValue::Text(rusterix.server.get_log()),
                        );
                    }
                    for r in &mut self.project.regions {
                        let (entities, items) = rusterix.server.get_entities_items(&r.map.id);
                        if let Some(entities) = entities {
                            r.map.entities = entities.clone();
                        }
                        if let Some(items) = items {
                            r.map.items = items.clone();
                        }

                        if r.id == self.server_ctx.curr_region {
                            if let Some(time) = rusterix.server.get_time(&r.map.id) {
                                rusterix.client.server_time = time;
                                if let Some(widget) = ui.get_widget("Server Time Slider") {
                                    widget.set_value(TheValue::Time(rusterix.client.server_time));
                                }
                            }
                        }
                    }
                }
                rusterix.set_dirty();
            }

            // Draw Region

            if let Some(render_view) = ui.get_render_view("PolyView") {
                let dim = *render_view.dim();

                let buffer = render_view.render_buffer_mut();
                buffer.resize(dim.width, dim.height);

                {
                    let rusterix = &mut RUSTERIX.write().unwrap();
                    let is_running = rusterix.server.state == rusterix::ServerState::Running;
                    let b = &mut rusterix.client.builder_d2;

                    // Draw the region map
                    if self.server_ctx.curr_map_context == MapContext::Region {
                        if let Some(region) = self.project.get_region(&self.server_ctx.curr_region)
                        {
                            if region.map.camera == MapCamera::TwoD {
                                b.set_material_mode(false);
                                b.set_map_tool_type(self.server_ctx.curr_map_tool_type);
                                if let Some(hover_cursor) = self.server_ctx.hover_cursor {
                                    b.set_map_hover_info(
                                        self.server_ctx.hover,
                                        Some(vek::Vec2::new(hover_cursor.x, hover_cursor.y)),
                                    );
                                } else {
                                    b.set_map_hover_info(self.server_ctx.hover, None);
                                }

                                if let Some(camera_pos) = region.map.camera_xz {
                                    b.set_camera_info(
                                        Some(Vec3::new(camera_pos.x, 0.0, camera_pos.y)),
                                        None,
                                    );
                                }
                            } else if region.map.camera == MapCamera::ThreeDIso {
                                if !is_running || !self.server_ctx.game_mode {
                                    let p = vek::Vec3::new(
                                        region.editing_position_3d.x,
                                        0.0,
                                        region.editing_position_3d.z,
                                    );

                                    rusterix.client.camera_d3.set_parameter_vec3("center", p);
                                    rusterix.client.camera_d3.set_parameter_vec3(
                                        "position",
                                        p + vek::Vec3::new(-10.0, 10.0, 10.0),
                                    );
                                }
                            } else if region.map.camera == MapCamera::ThreeDFirstPerson {
                                #[allow(clippy::collapsible_if)]
                                if !is_running || !self.server_ctx.game_mode {
                                    let p = vek::Vec3::new(
                                        region.editing_position_3d.x,
                                        1.5,
                                        region.editing_position_3d.z,
                                    );
                                    rusterix.client.camera_d3.set_parameter_vec3("position", p);
                                    rusterix.client.camera_d3.set_parameter_vec3(
                                        "center",
                                        p + vek::Vec3::new(0.0, 0.0, -1.0),
                                    );
                                }
                            }

                            rusterix.build_scene(
                                Vec2::new(dim.width as f32, dim.height as f32),
                                &region.map,
                                &self.build_values,
                                self.server_ctx.game_mode,
                            );

                            if let Some(map) = self.project.get_map(&self.server_ctx) {
                                let assets = rusterix.assets.clone();
                                rusterix.apply_entities_items(
                                    &map.entities,
                                    &map.items,
                                    map,
                                    &assets,
                                    &self.build_values,
                                );
                            }

                            rusterix.draw_scene(
                                render_view.render_buffer_mut().pixels_mut(),
                                dim.width as usize,
                                dim.height as usize,
                            );
                        }
                    } else
                    // Draw the material map
                    if self.server_ctx.curr_map_context == MapContext::Material {
                        b.set_map_tool_type(self.server_ctx.curr_map_tool_type);
                        if let Some(material) = self.project.get_map_mut(&self.server_ctx) {
                            if let Some(hover_cursor) = self.server_ctx.hover_cursor {
                                b.set_map_hover_info(
                                    self.server_ctx.hover,
                                    Some(vek::Vec2::new(hover_cursor.x, hover_cursor.y)),
                                );
                            } else {
                                b.set_map_hover_info(self.server_ctx.hover, None);
                            }
                            b.set_material_mode(true);
                            rusterix.build_scene(
                                Vec2::new(dim.width as f32, dim.height as f32),
                                material,
                                &self.build_values,
                                self.server_ctx.game_mode,
                            );
                            rusterix.draw_scene(
                                render_view.render_buffer_mut().pixels_mut(),
                                dim.width as usize,
                                dim.height as usize,
                            );
                        }
                    }
                }

                if let Some(map) = self.project.get_map_mut(&self.server_ctx) {
                    TOOLLIST.write().unwrap().draw_hud(
                        render_view.render_buffer_mut(),
                        map,
                        ctx,
                        &mut self.server_ctx,
                    );
                }
            }

            // Draw the 3D Preview if active.
            if self.server_ctx.curr_map_tool_helper == MapToolHelper::Preview {
                if let Some(region) = self.project.get_region_ctx(&self.server_ctx) {
                    PREVIEWVIEW.write().unwrap().draw(
                        region,
                        ui,
                        ctx,
                        &mut self.server_ctx,
                        &self.project.settings,
                    );
                }
            }

            redraw = true;
        }

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                redraw = self.sidebar.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                );
                if TOOLLIST.write().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if PANELS.write().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if self.mapeditor.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if TILEMAPEDITOR.write().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                match event {
                    TheEvent::DialogValueOnClose(role, name, uuid, _value) => {
                        if name == "Delete Character Instance ?" {
                            if role == TheDialogButtonRole::Delete {
                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    let character_id = uuid;
                                    if region.characters.shift_remove(&character_id).is_some() {
                                        self.server_ctx.curr_region_content =
                                            ContentContext::Unknown;
                                        region.map.selected_entity_item = None;
                                        redraw = true;

                                        // Remove from the content list
                                        if let Some(list) =
                                            ui.get_list_layout("Region Content List")
                                        {
                                            list.remove(TheId::named_with_id(
                                                "Region Content List Item",
                                                character_id,
                                            ));
                                            ui.select_first_list_item("Region Content List", ctx);
                                            ctx.ui.relayout = true;
                                        }
                                        insert_content_into_maps(&mut self.project);
                                        RUSTERIX.write().unwrap().set_dirty();
                                    }
                                }
                            }
                        } else if name == "Delete Item Instance ?" {
                            if role == TheDialogButtonRole::Delete {
                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    let item_id = uuid;
                                    if region.items.shift_remove(&item_id).is_some() {
                                        self.server_ctx.curr_region_content =
                                            ContentContext::Unknown;
                                        redraw = true;

                                        // Remove from the content list
                                        if let Some(list) =
                                            ui.get_list_layout("Region Content List")
                                        {
                                            list.remove(TheId::named_with_id(
                                                "Region Content List Item",
                                                item_id,
                                            ));
                                            ui.select_first_list_item("Region Content List", ctx);
                                            ctx.ui.relayout = true;
                                        }
                                    }
                                }
                            }
                        } else if name == "Update Eldiron" && role == TheDialogButtonRole::Accept {
                            let updater = self.self_updater.lock().unwrap();

                            if updater.has_newer_release() {
                                let release = updater.latest_release().cloned().unwrap();

                                let updater = Arc::clone(&self.self_updater);
                                let tx = self.self_update_tx.clone();

                                self.self_update_tx
                                    .send(SelfUpdateEvent::UpdateStart(release.clone()))
                                    .unwrap();

                                thread::spawn(move || {
                                    match updater.lock().unwrap().update_latest() {
                                        Ok(status) => match status {
                                            self_update::Status::UpToDate(_) => {
                                                tx.send(SelfUpdateEvent::AlreadyUpToDate).unwrap();
                                            }
                                            self_update::Status::Updated(_) => {
                                                tx.send(SelfUpdateEvent::UpdateCompleted(release))
                                                    .unwrap();
                                            }
                                        },
                                        Err(err) => {
                                            tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                                                .unwrap();
                                        }
                                    }
                                });
                            } else {
                                self.self_update_tx
                                    .send(SelfUpdateEvent::AlreadyUpToDate)
                                    .unwrap();
                            }
                        }
                    }
                    TheEvent::RenderViewDrop(_id, location, drop) => {
                        let mut grid_pos = Vec2::zero();

                        if let Some(map) = self.project.get_map(&self.server_ctx) {
                            if let Some(render_view) = ui.get_render_view("PolyView") {
                                let dim = *render_view.dim();
                                grid_pos = self.server_ctx.local_to_map_grid(
                                    Vec2::new(dim.width as f32, dim.height as f32),
                                    Vec2::new(location.x as f32, location.y as f32),
                                    map,
                                    map.subdivisions,
                                );
                            }
                        }

                        if drop.id.name.starts_with("Character") {
                            let mut instance = Character {
                                character_id: drop.id.uuid,
                                position: Vec3::new(grid_pos.x, 1.5, grid_pos.y),
                                ..Default::default()
                            };

                            if let Some(bytes) = crate::Embedded::get("python/instcharacter.py") {
                                if let Ok(source) = std::str::from_utf8(bytes.data.as_ref()) {
                                    instance.source = source.to_string();
                                }
                            }

                            let mut name = "Character".to_string();
                            if let Some(character) = self.project.characters.get(&drop.id.uuid) {
                                name.clone_from(&character.name);
                            }

                            if let Some(list) = ui.get_list_layout("Region Content List") {
                                let mut item = TheListItem::new(TheId::named_with_id(
                                    "Region Content List Item",
                                    instance.id,
                                ));
                                item.set_text(name);
                                item.set_state(TheWidgetState::Selected);
                                item.add_value_column(100, TheValue::Text("Character".to_string()));

                                list.deselect_all();
                                item.set_context_menu(Some(TheContextMenu {
                                    items: vec![TheContextMenuItem::new(
                                        "Delete Character...".to_string(),
                                        TheId::named("Sidebar Delete Character Instance"),
                                    )],
                                    ..Default::default()
                                }));
                                list.add_item(item, ctx);
                                list.select_item(instance.id, ctx, true);
                            }

                            // Add the character instance to the project
                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                self.server_ctx.curr_region_content =
                                    ContentContext::CharacterInstance(instance.id);
                                region.characters.insert(instance.id, instance.clone());
                                insert_content_into_maps(&mut self.project);
                                RUSTERIX.write().unwrap().set_dirty();
                            }
                        } else if drop.id.name.starts_with("Item") {
                            let mut instance = Item {
                                item_id: drop.id.uuid,
                                position: Vec3::new(grid_pos.x, 1.5, grid_pos.y),
                                ..Default::default()
                            };

                            if let Some(bytes) = crate::Embedded::get("python/institem.py") {
                                if let Ok(source) = std::str::from_utf8(bytes.data.as_ref()) {
                                    instance.source = source.to_string();
                                }
                            }

                            let mut name = "Item".to_string();
                            if let Some(character) = self.project.characters.get(&drop.id.uuid) {
                                name.clone_from(&character.name);
                            }

                            if let Some(list) = ui.get_list_layout("Region Content List") {
                                let mut item = TheListItem::new(TheId::named_with_id(
                                    "Region Content List Item",
                                    instance.id,
                                ));
                                item.set_text(name);
                                item.set_state(TheWidgetState::Selected);
                                item.add_value_column(100, TheValue::Text("Item".to_string()));

                                list.deselect_all();
                                item.set_context_menu(Some(TheContextMenu {
                                    items: vec![TheContextMenuItem::new(
                                        "Delete Item...".to_string(),
                                        TheId::named("Sidebar Delete Item Instance"),
                                    )],
                                    ..Default::default()
                                }));
                                list.add_item(item, ctx);
                                list.select_item(instance.id, ctx, true);
                            }

                            // Add the character instance to the project
                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                self.server_ctx.curr_region_content =
                                    ContentContext::ItemInstance(instance.id);
                                region.items.insert(instance.id, instance.clone());
                                insert_content_into_maps(&mut self.project);
                                RUSTERIX.write().unwrap().set_dirty();
                            }
                        }
                    }
                    /*
                    TheEvent::TileEditorDrop(_id, location, drop) => {
                        if drop.id.name.starts_with("Character") {
                            let mut instance = TheCodeBundle::new();

                            let mut init = TheCodeGrid {
                                name: "init".into(),
                                ..Default::default()
                            };
                            init.insert_atom(
                                (0, 0),
                                TheCodeAtom::Set(
                                    "@self.position".to_string(),
                                    TheValueAssignment::Assign,
                                ),
                            );
                            init.insert_atom(
                                (1, 0),
                                TheCodeAtom::Assignment(TheValueAssignment::Assign),
                            );
                            init.insert_atom(
                                (2, 0),
                                TheCodeAtom::Value(TheValue::Position(Vec3::new(
                                    location.x as f32,
                                    0.0,
                                    location.y as f32,
                                ))),
                            );
                            instance.insert_grid(init);

                            // Set the character instance bundle, disabled for now

                            // self.sidebar.code_editor.set_bundle(
                            //     instance.clone(),
                            //     ctx,
                            //     self.sidebar.width,
                            // );

                            let character = Character {
                                id: instance.id,
                                character_id: drop.id.uuid,
                                instance,
                            };

                            // Add the character instance to the region content list

                            let mut name = "Character".to_string();
                            if let Some(character) = self.project.characters.get(&drop.id.uuid) {
                                name.clone_from(&character.name);
                            }

                            if let Some(list) = ui.get_list_layout("Region Content List") {
                                let mut item = TheListItem::new(TheId::named_with_id(
                                    "Region Content List Item",
                                    character.id,
                                ));
                                item.set_text(name);
                                item.set_state(TheWidgetState::Selected);
                                item.add_value_column(100, TheValue::Text("Character".to_string()));

                                list.deselect_all();
                                item.set_context_menu(Some(TheContextMenu {
                                    items: vec![TheContextMenuItem::new(
                                        "Delete Character...".to_string(),
                                        TheId::named("Sidebar Delete Character Instance"),
                                    )],
                                    ..Default::default()
                                }));
                                list.add_item(item, ctx);
                                list.select_item(character.id, ctx, true);
                            }

                            // Add the character instance to the project

                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                region.characters.insert(character.id, character.clone());
                            }

                            // Add the character instance to the server

                            self.server_ctx.curr_character = Some(character.character_id);
                            self.server_ctx.curr_character_instance = Some(character.id);
                            self.server_ctx.curr_area = None;
                            //self.sidebar.deselect_all("Character List", ui);

                            self.server_ctx.curr_grid_id =
                                self.server.add_character_instance_to_region(
                                    self.server_ctx.curr_region,
                                    character,
                                    None,
                                );

                            // Set the character instance debug info, disabled for now

                            // if let Some(curr_grid_id) = self.server_ctx.curr_grid_id {
                            //     let debug_module = self.server.get_region_debug_module(
                            //         self.server_ctx.curr_region,
                            //         curr_grid_id,
                            //     );

                            //     self.sidebar.code_editor.set_debug_module(debug_module, ui);
                            // }
                        } else if drop.id.name.starts_with("Item") {
                            let mut instance = TheCodeBundle::new();

                            let mut init = TheCodeGrid {
                                name: "init".into(),
                                ..Default::default()
                            };
                            init.insert_atom(
                                (0, 0),
                                TheCodeAtom::Set(
                                    "@self.position".to_string(),
                                    TheValueAssignment::Assign,
                                ),
                            );
                            init.insert_atom(
                                (1, 0),
                                TheCodeAtom::Assignment(TheValueAssignment::Assign),
                            );
                            init.insert_atom(
                                (2, 0),
                                TheCodeAtom::Value(TheValue::Position(Vec3::new(
                                    location.x as f32,
                                    0.0,
                                    location.y as f32,
                                ))),
                            );
                            instance.insert_grid(init);

                            // Set the character instance bundle, disabled for now

                            // self.sidebar.code_editor.set_bundle(
                            //     instance.clone(),
                            //     ctx,
                            //     self.sidebar.width,
                            // );

                            let item = Item {
                                id: instance.id,
                                item_id: drop.id.uuid,
                                instance,
                            };

                            // Add the item instance to the region content list

                            let mut name = "Item".to_string();
                            if let Some(item) = self.project.items.get(&drop.id.uuid) {
                                name.clone_from(&item.name);
                            }

                            if let Some(list) = ui.get_list_layout("Region Content List") {
                                let mut list_item = TheListItem::new(TheId::named_with_id(
                                    "Region Content List Item",
                                    item.id,
                                ));
                                list_item.set_text(name);
                                list_item.set_state(TheWidgetState::Selected);
                                list_item.add_value_column(100, TheValue::Text("Item".to_string()));

                                list.deselect_all();
                                list.add_item(list_item, ctx);
                                list.select_item(item.id, ctx, true);
                            }

                            // Add the item instance to the project

                            if let Some(region) =
                                self.project.get_region_mut(&self.server_ctx.curr_region)
                            {
                                region.items.insert(item.id, item.clone());
                            }

                            // Add the character instance to the server

                            self.server_ctx.curr_character = None;
                            self.server_ctx.curr_character_instance = None;
                            self.server_ctx.curr_item = Some(item.item_id);
                            self.server_ctx.curr_item_instance = Some(item.id);
                            self.server_ctx.curr_area = None;

                            self.server_ctx.curr_grid_id = self
                                .server
                                .add_item_instance_to_region(self.server_ctx.curr_region, item);

                            // Set the character instance debug info, disabled for now

                            // if let Some(curr_grid_id) = self.server_ctx.curr_grid_id {
                            //     let debug_module = self.server.get_region_debug_module(
                            //         self.server_ctx.curr_region,
                            //         curr_grid_id,
                            //     );

                            //     self.sidebar.code_editor.set_debug_module(debug_module, ui);
                            // }
                        }
                    }*/
                    TheEvent::FileRequesterResult(id, paths) => {
                        // Load a palette from a file
                        if id.name == "Palette Import" {
                            for p in paths {
                                let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
                                let prev = self.project.palette.clone();
                                self.project.palette.load_from_txt(contents);
                                if let Some(palette_picker) =
                                    ui.get_palette_picker("Palette Picker")
                                {
                                    let index = palette_picker.index();

                                    palette_picker.set_palette(self.project.palette.clone());
                                    if let Some(widget) = ui.get_widget("Palette Color Picker") {
                                        if let Some(color) = &self.project.palette[index] {
                                            widget.set_value(TheValue::ColorObject(color.clone()));
                                        }
                                    }
                                    if let Some(widget) = ui.get_widget("Palette Hex Edit") {
                                        if let Some(color) = &self.project.palette[index] {
                                            widget.set_value(TheValue::Text(color.to_hex()));
                                        }
                                    }
                                }
                                redraw = true;

                                let undo =
                                    PaletteUndoAtom::Edit(prev, self.project.palette.clone());
                                UNDOMANAGER.write().unwrap().add_palette_undo(undo, ctx);
                            }
                        } else
                        // Open
                        if id.name == "Open" {
                            for p in paths {
                                self.project_path = Some(p.clone());
                                // let contents =
                                //     std::fs::read_to_string(p.clone()).unwrap_or("".to_string());
                                // if let Ok(contents) = std::fs::read(p) {
                                // let pr: Result<Project, serde_json::Error> =
                                //     serde_json::from_str(&contents);
                                // println!("{:?}", pr.err());
                                if let Ok(contents) = std::fs::read_to_string(p) {
                                    //if let Ok(project) =
                                    //    postcard::from_bytes::<Project>(contents.deref())
                                    // let pr: Result<Project, serde_json::Error> =
                                    //     serde_json::from_str(&contents);
                                    // println!("{:?}", pr.err());

                                    if let Ok(project) = serde_json::from_str(&contents) {
                                        self.project = project;
                                        insert_content_into_maps(&mut self.project);

                                        self.project.settings =
                                            shared::settingscontainer::SettingsContainer::default();

                                        if let Some(widget) = ui.get_widget("Server Time Slider") {
                                            widget.set_value(TheValue::Time(self.project.time));
                                        }
                                        // self.server.set_time(self.project.time);

                                        self.sidebar.load_from_project(
                                            ui,
                                            ctx,
                                            &mut self.server_ctx,
                                            &self.project,
                                        );
                                        self.mapeditor.load_from_project(ui, ctx, &self.project);
                                        update_server_icons = true;
                                        redraw = true;
                                        self.server_ctx.clear();

                                        // Set palette and textures
                                        *PALETTE.write().unwrap() = self.project.palette.clone();

                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project loaded successfully.".to_string(),
                                        ));
                                    }
                                }
                            }
                        } else if id.name == "Save As" {
                            for p in paths {
                                let json = serde_json::to_string(&self.project);
                                if let Ok(json) = json {
                                    if std::fs::write(p.clone(), json).is_ok() {
                                        self.project_path = Some(p);
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project saved successfully.".to_string(),
                                        ))
                                    } else {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Unable to save project!".to_string(),
                                        ))
                                    }
                                }
                            }
                        }
                    }
                    TheEvent::StateChanged(id, _state) => {
                        // if id.name == "Square" {
                        //     if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                        //         if layout.mode() == TheSharedVLayoutMode::Top {
                        //             layout.set_mode(TheSharedVLayoutMode::Bottom);
                        //             ctx.ui.relayout = true;
                        //         } else {
                        //             layout.set_mode(TheSharedVLayoutMode::Top);
                        //             ctx.ui.relayout = true;
                        //         }
                        //         redraw = true;
                        //     }
                        // } else if id.name == "Square Half" {
                        //     if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                        //         layout.set_mode(TheSharedVLayoutMode::Shared);
                        //         ctx.ui.relayout = true;
                        //         redraw = true;
                        //     }
                        // } else
                        if id.name == "Logo" {
                            _ = open::that("https://eldiron.com");
                            ctx.ui
                                .set_widget_state("Logo".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Patreon" {
                            _ = open::that("https://www.patreon.com/eldiron");
                            ctx.ui
                                .set_widget_state("Patreon".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Update" {
                            let updater = self.self_updater.lock().unwrap();

                            if updater.has_newer_release() {
                                self.self_update_tx
                                    .send(SelfUpdateEvent::UpdateConfirm(
                                        updater.latest_release().cloned().unwrap(),
                                    ))
                                    .unwrap();
                            } else {
                                if let Some(statusbar) = ui.get_widget("Statusbar") {
                                    statusbar
                                        .as_statusbar()
                                        .unwrap()
                                        .set_text("Checking updates...".to_string());
                                }

                                let updater = Arc::clone(&self.self_updater);
                                let tx = self.self_update_tx.clone();

                                thread::spawn(move || {
                                    let mut updater = updater.lock().unwrap();

                                    match updater.fetch_release_list() {
                                        Ok(_) => {
                                            if updater.has_newer_release() {
                                                tx.send(SelfUpdateEvent::UpdateConfirm(
                                                    updater.latest_release().cloned().unwrap(),
                                                ))
                                                .unwrap();
                                            } else {
                                                tx.send(SelfUpdateEvent::AlreadyUpToDate).unwrap();
                                            }
                                        }
                                        Err(err) => {
                                            tx.send(SelfUpdateEvent::UpdateError(err.to_string()))
                                                .unwrap();
                                        }
                                    }
                                });
                            }

                            ctx.ui
                                .set_widget_state("Update".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Open" {
                            ctx.ui.open_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Open".into(),
                                TheFileExtension::new(
                                    "Eldiron".into(),
                                    vec!["eldiron".to_string()],
                                ),
                            );
                            ctx.ui
                                .set_widget_state("Open".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        } else if id.name == "Save" {
                            if let Some(path) = &self.project_path {
                                let mut success = false;
                                // if let Ok(output) = postcard::to_allocvec(&self.project) {
                                if let Ok(output) = serde_json::to_string(&self.project) {
                                    if std::fs::write(path, output).is_ok() {
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project saved successfully.".to_string(),
                                        ));
                                        success = true;
                                    }
                                }

                                if !success {
                                    ctx.ui.send(TheEvent::SetStatusText(
                                        TheId::empty(),
                                        "Unable to save project!".to_string(),
                                    ))
                                }
                            } else {
                                ctx.ui.send(TheEvent::StateChanged(
                                    TheId::named("Save As"),
                                    TheWidgetState::Clicked,
                                ));
                                ctx.ui
                                    .set_widget_state("Save".to_string(), TheWidgetState::None);
                            }
                        } else if id.name == "Save As" {
                            ctx.ui.save_file_requester(
                                TheId::named_with_id(id.name.as_str(), Uuid::new_v4()),
                                "Save".into(),
                                TheFileExtension::new(
                                    "Eldiron".into(),
                                    vec!["eldiron".to_string()],
                                ),
                            );
                            ctx.ui
                                .set_widget_state("Save As".to_string(), TheWidgetState::None);
                            ctx.ui.clear_hover();
                            redraw = true;
                        }
                        // Server
                        else if id.name == "Play" {
                            let state = RUSTERIX.read().unwrap().server.state;
                            if state == rusterix::ServerState::Off {
                                start_server(&mut RUSTERIX.write().unwrap(), &mut self.project);
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Server has been started.".to_string(),
                                ));
                                RUSTERIX.write().unwrap().player_camera = PlayerCamera::D2;
                            }
                            /*
                            self.server.start();
                            self.client.reset();
                            self.client.set_project(self.project.clone());
                            self.server_ctx.clear_interactions();
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                "Server has been started.".to_string(),
                            ));
                            self.sidebar.clear_debug_messages(ui, ctx);
                            */
                            update_server_icons = true;
                        } else if id.name == "Pause" {
                            /*
                            if self.server.state == ServerState::Running {
                                self.server.state = ServerState::Paused;
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Server has been paused.".to_string(),
                                ));
                                update_server_icons = true;
                            } else if self.server.state == ServerState::Paused {
                                self.client.tick(
                                    *ACTIVEEDITOR.lock().unwrap() == ActiveEditor::GameEditor,
                                );
                                let debug = self.server.tick();
                                if !debug.is_empty() {
                                    self.sidebar.add_debug_messages(debug, ui, ctx);
                                }
                                let interactions = self.server.get_interactions();
                                self.server_ctx.add_interactions(interactions);
                            }*/
                        } else if id.name == "Stop" {
                            RUSTERIX.write().unwrap().server.stop();
                            RUSTERIX.write().unwrap().player_camera = PlayerCamera::D2;

                            /*
                            _ = self.server.set_project(self.project.clone());
                            self.server.stop();*/
                            insert_content_into_maps(&mut self.project);
                            update_server_icons = true;
                        } else if id.name == "Undo" || id.name == "Redo" {
                            if ui.focus_widget_supports_undo_redo(ctx) {
                                if id.name == "Undo" {
                                    ui.undo(ctx);
                                } else {
                                    ui.redo(ctx);
                                }
                            } else {
                                let mut manager = UNDOMANAGER.write().unwrap();

                                if manager.context == UndoManagerContext::Region {
                                    if id.name == "Undo" {
                                        manager.undo(
                                            self.server_ctx.curr_region,
                                            &mut self.server_ctx,
                                            &mut self.project,
                                            ui,
                                            ctx,
                                        );
                                    } else {
                                        manager.redo(
                                            self.server_ctx.curr_region,
                                            &mut self.server_ctx,
                                            &mut self.project,
                                            ui,
                                            ctx,
                                        );
                                    }
                                    ctx.ui.send(TheEvent::Custom(
                                        TheId::named("Update Minimap"),
                                        TheValue::Empty,
                                    ));
                                } else if manager.context == UndoManagerContext::Material
                                    || manager.context == UndoManagerContext::Palette
                                {
                                    if id.name == "Undo" {
                                        manager.undo(
                                            Uuid::nil(),
                                            &mut self.server_ctx,
                                            &mut self.project,
                                            ui,
                                            ctx,
                                        );
                                    } else {
                                        manager.redo(
                                            Uuid::nil(),
                                            &mut self.server_ctx,
                                            &mut self.project,
                                            ui,
                                            ctx,
                                        );
                                    }
                                }
                            }
                        } else if id.name == "Cut" {
                            if ui.focus_widget_supports_clipboard(ctx) {
                                // Widget specific
                                ui.cut(ctx);
                            } else {
                                // Global
                                ctx.ui.send(TheEvent::Cut);
                            }
                        } else if id.name == "Copy" {
                            if ui.focus_widget_supports_clipboard(ctx) {
                                // Widget specific
                                ui.copy(ctx);
                            } else {
                                // Global
                                ctx.ui.send(TheEvent::Copy);
                            }
                        } else if id.name == "Paste" {
                            if ui.focus_widget_supports_clipboard(ctx) {
                                // Widget specific
                                ui.paste(ctx);
                            } else {
                                // Global
                                if let Some(value) = &ctx.ui.clipboard {
                                    ctx.ui.send(TheEvent::Paste(
                                        value.clone(),
                                        ctx.ui.clipboard_app_type.clone(),
                                    ));
                                }
                            }
                        }
                    }
                    TheEvent::ValueChanged(id, value) => {
                        if id.name == "Server Time Slider" {
                            if let TheValue::Time(time) = value {
                                self.project.time = time;
                                let mut rusterix = RUSTERIX.write().unwrap();
                                rusterix.client.server_time = time;
                                if rusterix.server.state == rusterix::ServerState::Running {
                                    if let Some(map) = self.project.get_map(&self.server_ctx) {
                                        rusterix.server.set_time(&map.id, time);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        while let Ok(event) = self.self_update_rx.try_recv() {
            match event {
                SelfUpdateEvent::AlreadyUpToDate => {
                    let text = str!("Eldiron is already up-to-date.");
                    let uuid = Uuid::new_v4();

                    let width = 300;
                    let height = 100;

                    let mut canvas = TheCanvas::new();
                    canvas.limiter_mut().set_max_size(Vec2::new(width, height));

                    let mut hlayout: TheHLayout = TheHLayout::new(TheId::empty());
                    hlayout.limiter_mut().set_max_width(width);

                    let mut text_widget = TheText::new(TheId::named_with_id("Dialog Value", uuid));
                    text_widget.set_text(text.to_string());
                    text_widget.limiter_mut().set_max_width(200);
                    hlayout.add_widget(Box::new(text_widget));

                    canvas.set_layout(hlayout);

                    ui.show_dialog(
                        "Eldiron Up-to-Date",
                        canvas,
                        vec![TheDialogButtonRole::Accept],
                        ctx,
                    );
                }
                SelfUpdateEvent::UpdateCompleted(release) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar.as_statusbar().unwrap().set_text(format!(
                            "Updated to version {}. Please restart the application to enjoy the new features.",
                            release.version
                        ));
                    }
                }
                SelfUpdateEvent::UpdateConfirm(release) => {
                    let text = &format!("Update to version {}?", release.version);
                    let uuid = Uuid::new_v4();

                    let width = 300;
                    let height = 100;

                    let mut canvas = TheCanvas::new();
                    canvas.limiter_mut().set_max_size(Vec2::new(width, height));

                    let mut hlayout: TheHLayout = TheHLayout::new(TheId::empty());
                    hlayout.limiter_mut().set_max_width(width);

                    let mut text_widget = TheText::new(TheId::named_with_id("Dialog Value", uuid));
                    text_widget.set_text(text.to_string());
                    text_widget.limiter_mut().set_max_width(200);
                    hlayout.add_widget(Box::new(text_widget));

                    canvas.set_layout(hlayout);

                    ui.show_dialog(
                        "Update Eldiron",
                        canvas,
                        vec![TheDialogButtonRole::Accept, TheDialogButtonRole::Reject],
                        ctx,
                    );
                }
                SelfUpdateEvent::UpdateError(err) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar
                            .as_statusbar()
                            .unwrap()
                            .set_text(format!("Failed to update Eldiron: {}", err));
                    }
                }
                SelfUpdateEvent::UpdateStart(release) => {
                    if let Some(statusbar) = ui.get_widget("Statusbar") {
                        statusbar
                            .as_statusbar()
                            .unwrap()
                            .set_text(format!("Updating to version {}...", release.version));
                    }
                }
            }
        }

        if update_server_icons {
            self.update_server_state_icons(ui);
            redraw = true;
        }
        redraw
    }
}

pub trait EldironEditor {
    fn update_server_state_icons(&mut self, ui: &mut TheUI);
}

impl EldironEditor for Editor {
    fn update_server_state_icons(&mut self, ui: &mut TheUI) {
        let rusterix = RUSTERIX.read().unwrap();
        if rusterix.server.state == rusterix::ServerState::Running {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-fill".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop".to_string());
                }
            }
        } else if rusterix.server.state == rusterix::ServerState::Paused {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause-fill".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop".to_string());
                }
            }
        } else if rusterix.server.state == rusterix::ServerState::Off {
            if let Some(button) = ui.get_widget("Play") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Pause") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("play-pause".to_string());
                }
            }
            if let Some(button) = ui.get_widget("Stop") {
                if let Some(button) = button.as_menubar_button() {
                    button.set_icon_name("stop-fill".to_string());
                }
            }
        }
    }
}
