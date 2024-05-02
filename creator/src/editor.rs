use crate::prelude::*;
use crate::self_update::SelfUpdateEvent;
use crate::self_update::SelfUpdater;
use crate::Embedded;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

lazy_static! {
    pub static ref CODEEDITOR: Mutex<TheCodeEditor> = Mutex::new(TheCodeEditor::new());
    pub static ref TILEPICKER: Mutex<TilePicker> =
        Mutex::new(TilePicker::new("Main Tile Picker".to_string()));
    pub static ref TILEMAPEDITOR: Mutex<TilemapEditor> = Mutex::new(TilemapEditor::new());
    pub static ref SIDEBARMODE: Mutex<SidebarMode> = Mutex::new(SidebarMode::Region);
    pub static ref TILEDRAWER: Mutex<TileDrawer> = Mutex::new(TileDrawer::new());
    pub static ref RENDERER: Mutex<Renderer> = Mutex::new(Renderer::new());
    pub static ref RENDERMODE: Mutex<EditorDrawMode> = Mutex::new(EditorDrawMode::Draw2D);
    pub static ref TILEFXEDITOR: Mutex<TileFXEditor> = Mutex::new(TileFXEditor::new());
    pub static ref MODELFXEDITOR: Mutex<ModelFXEditor> = Mutex::new(ModelFXEditor::new());
    pub static ref REGIONFXEDITOR: Mutex<RegionFXEditor> = Mutex::new(RegionFXEditor::new());
    pub static ref VOXELTHREAD: Mutex<VoxelThread> = Mutex::new(VoxelThread::default());
    pub static ref UNDOMANAGER: Mutex<UndoManager> = Mutex::new(UndoManager::default());
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum ActiveEditor {
    TileEditor,
    ScreenEditor,
}

pub struct Editor {
    project: Project,
    project_path: Option<PathBuf>,

    active_editor: ActiveEditor,

    sidebar: Sidebar,
    panels: Panels,
    tileeditor: TileEditor,
    screeneditor: ScreenEditor,

    server: Server,
    client: Client,
    server_ctx: ServerContext,

    update_tracker: UpdateTracker,
    event_receiver: Option<Receiver<TheEvent>>,

    self_update_rx: Receiver<SelfUpdateEvent>,
    self_update_tx: Sender<SelfUpdateEvent>,
    self_updater: Arc<Mutex<SelfUpdater>>,
}

impl TheTrait for Editor {
    fn new() -> Self
    where
        Self: Sized,
    {
        let mut server = Server::new();
        server.debug_mode = true;

        let client = Client::new();

        let (self_update_tx, self_update_rx) = channel();

        Self {
            project: Project::new(),
            project_path: None,

            active_editor: ActiveEditor::TileEditor,

            sidebar: Sidebar::new(),
            panels: Panels::new(),
            tileeditor: TileEditor::new(),
            screeneditor: ScreenEditor::new(),

            server_ctx: ServerContext::default(),
            server,
            client,

            update_tracker: UpdateTracker::new(),
            event_receiver: None,

            self_update_rx,
            self_update_tx,
            self_updater: Arc::new(Mutex::new(SelfUpdater::new(
                "markusmoenig",
                "Eldiron",
                "eldiron",
            ))),
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
        set_server_externals();

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
        edit_menu.add(TheContextMenuItem::new(str!("Undo"), TheId::named("Undo")));
        edit_menu.add(TheContextMenuItem::new(str!("Redo"), TheId::named("Redo")));
        edit_menu.add_separator();
        edit_menu.add(TheContextMenuItem::new(str!("Cut"), TheId::named("Cut")));
        edit_menu.add(TheContextMenuItem::new(str!("Copy"), TheId::named("Copy")));
        edit_menu.add(TheContextMenuItem::new(
            str!("Paste"),
            TheId::named("Paste"),
        ));

        menu.add_context_menu(file_menu);
        menu.add_context_menu(edit_menu);

        let code_menu = create_code_menu(ctx);

        menu.add_context_menu(code_menu);
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
        save_as_button.set_icon_offset(vec2i(2, -5));

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

        let mut square_button = TheMenubarButton::new(TheId::named("Square"));
        square_button.set_status_text("Display full content.");
        square_button.set_icon_name("frame_corners".to_string());
        square_button.set_icon_offset(vec2i(-1, -1));

        let mut square_half_button = TheMenubarButton::new(TheId::named("Square Half"));
        square_half_button.set_status_text("Display content 60/40.");
        square_half_button.set_icon_name("square_half_bottom".to_string());
        square_half_button.set_icon_offset(vec2i(-1, -1));

        let mut update_button = TheMenubarButton::new(TheId::named("Update"));
        update_button.set_status_text("Update application.");
        update_button.set_icon_name("arrows-clockwise".to_string());

        let mut patreon_button = TheMenubarButton::new(TheId::named("Patreon"));
        patreon_button.set_status_text("Visit my Patreon page.");
        patreon_button.set_icon_name("patreon".to_string());
        // patreon_button.set_fixed_size(vec2i(36, 36));
        patreon_button.set_icon_offset(vec2i(-4, -2));

        let mut hlayout = TheHLayout::new(TheId::named("Menu Layout"));
        hlayout.set_background_color(None);
        hlayout.set_margin(vec4i(10, 2, 10, 1));
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
        hlayout.add_widget(Box::new(square_button));
        hlayout.add_widget(Box::new(square_half_button));

        hlayout.add_widget(Box::new(update_button));
        hlayout.add_widget(Box::new(TheMenubarSeparator::new(TheId::empty())));
        hlayout.add_widget(Box::new(patreon_button));

        hlayout.set_reverse_index(Some(3));

        top_canvas.set_widget(menubar);
        top_canvas.set_layout(hlayout);
        top_canvas.set_top(menu_canvas);
        ui.canvas.set_top(top_canvas);

        // Sidebar
        self.sidebar
            .init_ui(ui, ctx, &mut self.project, &mut self.server);

        // Panels
        let bottom_panels = self.panels.init_ui(ui, ctx, &mut self.project);

        // Editor
        let mut tab_canvas: TheCanvas = TheCanvas::new();
        let mut tab_layout = TheTabLayout::new(TheId::named("Editor Tab"));

        let game_canvas = self.tileeditor.init_ui(ui, ctx, &mut self.project);
        tab_layout.add_canvas(str!("Game"), game_canvas);

        let screen_canvas = self.screeneditor.init_ui(ui, ctx, &mut self.project);
        tab_layout.add_canvas(str!("Screen"), screen_canvas);

        tab_canvas.set_layout(tab_layout);

        let mut vsplitlayout = TheSharedVLayout::new(TheId::named("Shared VLayout"));
        vsplitlayout.add_canvas(tab_canvas);
        vsplitlayout.add_canvas(bottom_panels);
        vsplitlayout.set_shared_ratio(0.62);
        vsplitlayout.set_mode(TheSharedVLayoutMode::Shared);

        let mut shared_canvas = TheCanvas::new();
        shared_canvas.set_layout(vsplitlayout);

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

        self.event_receiver = Some(ui.add_state_listener("Main Receiver".into()));

        // Startup the voxel render thread.
        VOXELTHREAD.lock().unwrap().startup();
    }

    /// Set the command line arguments
    fn set_cmd_line_args(&mut self, args: Vec<String>, ctx: &mut TheContext) {
        if args.len() > 1 {
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

        let (redraw_update, tick_update) = self.update_tracker.update(
            (1000 / self.project.target_fps) as u64,
            self.project.tick_ms as u64,
        );

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
                self.client.tick();
                let debug = self.server.tick();
                if !debug.is_empty() {
                    self.sidebar.add_debug_messages(debug, ui, ctx);
                }
                let interactions = self.server.get_interactions();
                self.server_ctx.add_interactions(interactions);
                self.panels
                    .update_code_object(ui, ctx, &mut self.server, &mut self.server_ctx);
                if let Some(update) = self
                    .server
                    .get_region_update_json(self.server_ctx.curr_region)
                {
                    self.client.set_region_update(update);
                }

                if let Some(widget) = ui.get_widget("Server Time Slider") {
                    widget.set_value(TheValue::Time(self.server.world.time));
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
        }

        while let Some(VoxelRenderResult::VoxelizedModel(id, key, model)) =
            VOXELTHREAD.lock().unwrap().receive()
        {
            if let Some(region) = self.project.get_region_mut(&id) {
                region.models.insert((key.x, key.y, key.z), model.clone());
            }
            self.server.set_voxelized_model(id, key, model);
            redraw = true;
        }

        if self.active_editor == ActiveEditor::TileEditor
            && redraw_update
            && !self.project.regions.is_empty()
        {
            let render_mode = *RENDERMODE.lock().unwrap();
            if render_mode != EditorDrawMode::Draw3D {
                self.tileeditor
                    .redraw_region(ui, &mut self.server, ctx, &self.server_ctx, true);
            }
            if render_mode != EditorDrawMode::Draw2D {
                self.tileeditor.rerender_region(
                    ui,
                    &mut self.server,
                    ctx,
                    &self.server_ctx,
                    &self.project,
                    render_mode == EditorDrawMode::Draw3D,
                );
            }
            redraw = true;
        } else if self.active_editor == ActiveEditor::ScreenEditor && redraw_update {
            self.screeneditor.redraw_screen(
                ui,
                &mut self.client,
                ctx,
                &self.server_ctx,
                &self.project,
            );
            redraw = true;
        }

        if let Some(receiver) = &mut self.event_receiver {
            while let Ok(event) = receiver.try_recv() {
                redraw = self.sidebar.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.client,
                    &mut self.server_ctx,
                );
                if self.panels.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if self.tileeditor.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if self.screeneditor.handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.client,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if TILEMAPEDITOR.lock().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if TILEFXEDITOR.lock().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if MODELFXEDITOR.lock().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                if REGIONFXEDITOR.lock().unwrap().handle_event(
                    &event,
                    ui,
                    ctx,
                    &mut self.project,
                    &mut self.server,
                    &mut self.server_ctx,
                ) {
                    redraw = true;
                }
                match event {
                    TheEvent::Custom(id, _) => {
                        if id.name == "Update Code Menu" {
                            let codemenu = create_code_menu(ctx);
                            if let Some(menu) = ui.get_menu("Menu") {
                                menu.replace_context_menu(codemenu);
                            }
                        }
                    }
                    TheEvent::ContextMenuSelected(_, action) => {
                        if action.name.starts_with("Code") {
                            CODEEDITOR
                                .lock()
                                .unwrap()
                                .insert_context_menu_id(action, ui, ctx);
                        }
                    }
                    TheEvent::IndexChanged(id, index) => {
                        if id.name == "Editor Tab Tabbar" {
                            if index == 0 {
                                self.active_editor = ActiveEditor::TileEditor;
                            } else if index == 1 {
                                self.active_editor = ActiveEditor::ScreenEditor;
                                self.client.set_project(self.project.clone());
                            }
                            redraw = true;
                        }
                    }
                    TheEvent::DialogValueOnClose(role, name, uuid, value) => {
                        //println!("Dialog Value On Close: {} -> {:?}", name, value);

                        if name == "Delete Character Instance ?" {
                            if role == TheDialogButtonRole::Delete {
                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    let character_id = uuid;
                                    if region.characters.remove(&character_id).is_some() {
                                        self.server
                                            .remove_character_instance(region.id, character_id);
                                        self.server_ctx.curr_character_instance = None;
                                        self.server_ctx.curr_character = None;
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
                                        }
                                    }
                                }
                            }
                        } else if name == "Delete Item Instance ?" {
                            if role == TheDialogButtonRole::Delete {
                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    let item_id = uuid;
                                    if region.items.remove(&item_id).is_some() {
                                        self.server.remove_character_instance(region.id, item_id);
                                        self.server_ctx.curr_item_instance = None;
                                        self.server_ctx.curr_item = None;
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
                                        }
                                    }
                                }
                            }
                        } else if name == "Delete Area ?" {
                            if role == TheDialogButtonRole::Delete {
                                let area_id = uuid;

                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    if region.areas.remove(&area_id).is_some() {
                                        self.server.remove_area(region.id, area_id);
                                        self.server_ctx.curr_area = None;
                                        redraw = true;

                                        // Remove from the content list
                                        if let Some(list) =
                                            ui.get_list_layout("Region Content List")
                                        {
                                            list.remove(TheId::named_with_id(
                                                "Region Content List Item",
                                                area_id,
                                            ));
                                            ui.select_first_list_item("Region Content List", ctx);
                                        }
                                    }
                                }
                            }
                        } else if name == "Delete Widget ?" {
                            if role == TheDialogButtonRole::Delete {
                                let widget_id = uuid;

                                if let Some(screen) =
                                    self.project.screens.get_mut(&self.server_ctx.curr_screen)
                                {
                                    screen.remove_widget(&widget_id);

                                    // Remove from the content list
                                    if let Some(list) = ui.get_list_layout("Screen Content List") {
                                        list.remove(TheId::named_with_id(
                                            "Screen Content List Item",
                                            widget_id,
                                        ));
                                        ui.select_first_list_item("Screen Content List", ctx);
                                    }

                                    self.client.update_screen(screen);
                                    self.sidebar.apply_screen(ui, ctx, Some(screen));
                                }
                            }
                        } else if name == "New Area Name" {
                            // Create a new area

                            if let Some(tiles) = &self.server_ctx.tile_selection {
                                let mut area = Area {
                                    area: tiles.tiles(),
                                    name: value.describe(),
                                    ..Default::default()
                                };

                                let main = TheCodeGrid {
                                    name: "main".into(),
                                    ..Default::default()
                                };

                                area.bundle.insert_grid(main);

                                if let Some(list) = ui.get_list_layout("Region Content List") {
                                    let mut item = TheListItem::new(TheId::named_with_id(
                                        "Region Content List Item",
                                        area.id,
                                    ));
                                    item.set_text(area.name.clone());
                                    item.set_state(TheWidgetState::Selected);
                                    item.add_value_column(100, TheValue::Text("Area".to_string()));

                                    list.deselect_all();
                                    list.add_item(item, ctx);
                                    list.select_item(area.id, ctx, true);
                                }

                                self.server_ctx.curr_area = Some(area.id);
                                self.server_ctx.curr_character_instance = None;
                                self.server_ctx.curr_character = None;

                                if let Some(region) =
                                    self.project.get_region_mut(&self.server_ctx.curr_region)
                                {
                                    region.areas.insert(area.id, area);
                                }
                            }
                            self.server_ctx.tile_selection = None;
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
                                    ":self.position".to_string(),
                                    TheValueAssignment::Assign,
                                ),
                            );
                            init.insert_atom(
                                (1, 0),
                                TheCodeAtom::Assignment(TheValueAssignment::Assign),
                            );
                            init.insert_atom(
                                (2, 0),
                                TheCodeAtom::Value(TheValue::Position(vec3f(
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
                                name = character.name.clone();
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
                                    ":self.position".to_string(),
                                    TheValueAssignment::Assign,
                                ),
                            );
                            init.insert_atom(
                                (1, 0),
                                TheCodeAtom::Assignment(TheValueAssignment::Assign),
                            );
                            init.insert_atom(
                                (2, 0),
                                TheCodeAtom::Value(TheValue::Position(vec3f(
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
                                name = item.name.clone();
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
                    }
                    TheEvent::FileRequesterResult(id, paths) => {
                        // Load a palette from a file
                        if id.name == "Palette Import" {
                            for p in paths {
                                let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
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
                                self.server.set_palette(&self.project.palette);
                                redraw = true;
                            }
                        } else if id.name == "Open" {
                            for p in paths {
                                self.project_path = Some(p.clone());
                                //let contents = std::fs::read_to_string(p).unwrap_or("".to_string());
                                // if let Ok(contents) = std::fs::read(p) {
                                if let Ok(contents) = std::fs::read_to_string(p) {
                                    //if let Ok(project) =
                                    //    postcard::from_bytes::<Project>(contents.deref())
                                    if let Ok(project) = serde_json::from_str(&contents) {
                                        self.project = project;

                                        for r in &mut self.project.regions {
                                            for m in r.models.values_mut() {
                                                m.floor.addjust_nodes();
                                                m.wall.addjust_nodes();
                                                m.ceiling.addjust_nodes();
                                            }
                                        }

                                        for region in &mut self.project.regions {
                                            VOXELTHREAD.lock().unwrap().voxelize_region_models(
                                                region.clone(),
                                                self.project.palette.clone(),
                                            );
                                        }

                                        if let Some(widget) = ui.get_widget("Server Time Slider") {
                                            widget.set_value(TheValue::Time(self.project.time));
                                        }
                                        self.server.set_time(self.project.time);

                                        if let Some(widget) = ui.get_group_button("2D3D Group") {
                                            widget.set_index(self.project.map_mode as i32);
                                        }

                                        if let Some(shared) = ui.get_sharedhlayout("Editor Shared")
                                        {
                                            match self.project.map_mode {
                                                MapMode::TwoD => {
                                                    *RENDERMODE.lock().unwrap() =
                                                        EditorDrawMode::Draw2D;
                                                    shared.set_mode(TheSharedHLayoutMode::Left);
                                                }
                                                MapMode::Mixed => {
                                                    *RENDERMODE.lock().unwrap() =
                                                        EditorDrawMode::DrawMixed;
                                                    shared.set_mode(TheSharedHLayoutMode::Shared);
                                                }
                                                MapMode::ThreeD => {
                                                    *RENDERMODE.lock().unwrap() =
                                                        EditorDrawMode::Draw3D;
                                                    shared.set_mode(TheSharedHLayoutMode::Right);
                                                }
                                            }
                                        }

                                        self.sidebar.load_from_project(ui, ctx, &self.project);
                                        self.tileeditor.load_from_project(ui, ctx, &self.project);
                                        let packages =
                                            self.server.set_project(self.project.clone());
                                        self.client.set_project(self.project.clone());
                                        CODEEDITOR.lock().unwrap().set_packages(packages);
                                        self.server.state = ServerState::Stopped;
                                        update_server_icons = true;
                                        redraw = true;
                                        self.server_ctx.clear();
                                        ctx.ui.send(TheEvent::SetStatusText(
                                            TheId::empty(),
                                            "Project loaded successfully.".to_string(),
                                        ))
                                    }
                                }
                            }
                        } else if id.name == "Save As" {
                            for p in paths {
                                let json = serde_json::to_string(&self.project);
                                if let Ok(json) = json {
                                    if std::fs::write(p, json).is_ok() {
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
                        // Open / Save Project

                        if id.name == "Square" {
                            if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                                if layout.mode() == TheSharedVLayoutMode::Top {
                                    layout.set_mode(TheSharedVLayoutMode::Bottom);
                                    ctx.ui.relayout = true;
                                } else {
                                    layout.set_mode(TheSharedVLayoutMode::Top);
                                    ctx.ui.relayout = true;
                                }
                                redraw = true;
                            }
                        } else if id.name == "Square Half" {
                            if let Some(layout) = ui.get_sharedvlayout("Shared VLayout") {
                                layout.set_mode(TheSharedVLayoutMode::Shared);
                                ctx.ui.relayout = true;
                                redraw = true;
                            }
                        } else if id.name == "Logo" {
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
                            self.server.start();
                            self.server_ctx.clear_interactions();
                            ctx.ui.send(TheEvent::SetStatusText(
                                TheId::empty(),
                                "Server has been started.".to_string(),
                            ));
                            self.sidebar.clear_debug_messages(ui, ctx);
                            update_server_icons = true;
                        } else if id.name == "Pause" {
                            if self.server.state == ServerState::Running {
                                self.server.state = ServerState::Paused;
                                ctx.ui.send(TheEvent::SetStatusText(
                                    TheId::empty(),
                                    "Server has been paused.".to_string(),
                                ));
                                update_server_icons = true;
                            } else if self.server.state == ServerState::Paused {
                                self.client.tick();
                                let debug = self.server.tick();
                                if !debug.is_empty() {
                                    self.sidebar.add_debug_messages(debug, ui, ctx);
                                }
                                let interactions = self.server.get_interactions();
                                self.server_ctx.add_interactions(interactions);
                            }
                        } else if id.name == "Stop" {
                            _ = self.server.set_project(self.project.clone());
                            self.server.stop();
                            update_server_icons = true;
                        } else {
                            // TODO: UNDO

                            if id.name == "Undo" || id.name == "Redo" {
                                let mut manager = UNDOMANAGER.lock().unwrap();

                                if manager.context == UndoManagerContext::Region {
                                    if id.name == "Undo" {
                                        manager.undo(
                                            self.server_ctx.curr_region,
                                            &mut self.project,
                                            ctx,
                                        );
                                    } else {
                                        manager.redo(
                                            self.server_ctx.curr_region,
                                            &mut self.project,
                                            ctx,
                                        );
                                    }
                                    if let Some(region) =
                                        self.project.get_region(&self.server_ctx.curr_region)
                                    {
                                        self.server.update_region(region);
                                        RENDERER.lock().unwrap().set_region(region);
                                    }
                                } else if manager.context == UndoManagerContext::ModelFX {
                                    if id.name == "Undo" {
                                        manager.undo(Uuid::nil(), &mut self.project, ctx);
                                    } else {
                                        manager.redo(Uuid::nil(), &mut self.project, ctx);
                                    }
                                    let mut model_editor = MODELFXEDITOR.lock().unwrap();
                                    model_editor.modelfx.draw(ui, ctx, &self.project.palette);
                                    model_editor.set_selected_node_ui(
                                        ui,
                                        ctx,
                                        &self.project.palette,
                                    );
                                    model_editor.render_preview(ui, &self.project.palette);
                                }
                            }

                            /*
                            let mut data: Option<(TheId, String)> = None;
                            if id.name == "Undo" && ctx.ui.undo_stack.has_undo() {
                                data = Some(ctx.ui.undo_stack.undo());
                            } else if id.name == "Redo" && ctx.ui.undo_stack.has_redo() {
                                data = Some(ctx.ui.undo_stack.redo());
                            }

                            if let Some((id, json)) = data {
                                #[allow(clippy::single_match)]
                                match id.name.as_str() {
                                    "RegionChanged" => {
                                        let region = Region::from_json(json.as_str());
                                        for (index, r) in self.project.regions.iter().enumerate() {
                                            if r.id == region.id {
                                                self.server.update_region(&region);
                                                RENDERER.lock().unwrap().set_region(&region);
                                                self.project.regions[index] = region;
                                                break;
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                                redraw = true;
                                }*/
                        }
                    }
                    TheEvent::ImageDecodeResult(id, name, buffer) => {
                        if id.name == "Add Image" {
                            // Add a new tilemap to the project
                            let asset = Asset {
                                name,
                                id: id.uuid,
                                buffer: AssetBuffer::Image(buffer),
                            };

                            self.project.add_asset(asset);
                            self.client.set_assets(self.project.assets.clone());
                        } else if id.name == "Tilemap Add" {
                            // Add a new tilemap to the project
                            let mut tilemap = Tilemap::new();
                            tilemap.name = name;
                            tilemap.id = id.uuid;
                            tilemap.buffer = buffer;

                            self.project.add_tilemap(tilemap);
                        }
                    }
                    TheEvent::ValueChanged(id, value) => {
                        if id.name == "Server Time Slider" {
                            if let TheValue::Time(time) = value {
                                self.server.set_time(time);
                                self.project.time = time;
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
                    canvas.limiter_mut().set_max_size(vec2i(width, height));

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
                    canvas.limiter_mut().set_max_size(vec2i(width, height));

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
        if self.server.state == ServerState::Running {
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
        } else if self.server.state == ServerState::Paused {
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
        } else if self.server.state == ServerState::Stopped {
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
