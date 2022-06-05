use server::asset::{ Asset };
use server::asset::tileset::TileUsage;
use server::gamedata::game_screen::{GameScreen, GameScreenWidget};
use server::gamedata::nodes_game::screen;

use crate::widget::context::ScreenContext;
use crate::editor::{ TileSelectorWidget };
use server::gamedata::behavior::{ BehaviorType };

use crate::editor::{ EditorOptions, EditorContent };
use crate::editor::regionoptions::RegionEditorMode;

use crate::editor::ToolBar;

use super::screeneditor_options::{ScreenEditorMode, ScreenEditorAction};

pub struct ScreenEditor {
    pub rect                : (usize, usize, usize, usize),
    pub region_id           : usize,

    grid_size               : usize,

    offset                  : (isize, isize),
    screen_offset           : (usize, usize),

    pub tile_selector       : TileSelectorWidget,

    mouse_wheel_delta       : (isize, isize),

    mouse_hover_pos         : (usize, usize),
    pub clicked             : Option<(isize, isize)>,

    widget_start            : Option<(isize, isize)>,
    widget_end              : Option<(isize, isize)>,

    game_screen             : GameScreen,

    selector_size           : usize,

    node_behavior_id        : (usize, usize, String),
    node_behavior_value     : (f64, f64, f64, f64, String),

    screen_width            : usize,
    screen_height           : usize,
    screen_tile_size        : usize,
}

impl EditorContent for ScreenEditor {

    fn new(_text: Vec<String>, rect: (usize, usize, usize, usize), _behavior_type: BehaviorType, asset: &Asset, context: &ScreenContext) -> Self {

        let bottom_size = 250;

        // Tile Selector
        let mut tile_selector = TileSelectorWidget::new(vec!(), (rect.0, rect.1 + rect.3 - bottom_size, rect.2, bottom_size), asset, &context);
        tile_selector.set_tile_type(vec![TileUsage::Environment], None, None, &asset);


        Self {
            rect,
            region_id               : 0,
            grid_size               : 32,

            offset                  : (0, 0),
            screen_offset           : (0, 0),

            tile_selector,

            mouse_wheel_delta       : (0, 0),
            mouse_hover_pos         : (0, 0),
            clicked                 : None,

            widget_start            : None,
            widget_end              : None,

            game_screen             : GameScreen::new(),

            selector_size           : 250,

            node_behavior_id        : (0, 0, "".to_string()),
            node_behavior_value     : (0.0, 0.0, 0.0, 0.0, "".to_string()),

            screen_width            : 0,
            screen_height           : 0,
            screen_tile_size        : 0,
        }
    }

    fn resize(&mut self, width: usize, height: usize, _context: &ScreenContext) {
        self.rect.2 = width;
        self.rect.3 = height;

        self.tile_selector.rect = (self.rect.0, self.rect.1 + self.rect.3 - self.selector_size, width, self.selector_size);
        self.tile_selector.resize(width, self.selector_size);
    }

    fn draw(&mut self, frame: &mut [u8], anim_counter: usize, _asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>) {
        context.draw2d.draw_rect(frame, &self.rect, context.width, &context.color_black);

        if let Some(_options) = options {

            let grid_size = self.grid_size;
            let rect = self.rect.clone();

            let left_offset = (rect.2 % grid_size) / 2;
            let top_offset = (rect.3 % grid_size) / 2;

            self.screen_offset = (left_offset, top_offset);

            let x_tiles = (rect.2 / grid_size) as isize;
            let y_tiles = (rect.3 / grid_size) as isize;

            context.data.game_anim_counter = anim_counter;
            self.game_screen.draw(self.node_behavior_id.1, &mut context.data);

            context.draw2d.blend_slice_safe(frame, &mut context.data.game_frame[..], &(rect.0 as isize + left_offset as isize + self.offset.0 * grid_size as isize, rect.1 as isize + top_offset as isize + self.offset.1 * grid_size as isize, self.screen_width, self.screen_height), context.width, &rect);

            for y in 0..y_tiles {
                for x in 0..x_tiles {

                    let cx = x - self.offset.0;
                    let cy = y - self.offset.1;

                    if let Some(widget_start) = self.widget_start {
                        if let Some(widget_end) = self.widget_end {

                            if  cy >= widget_start.1 && cx >= widget_start.0 { // >=
                                if  cy <= widget_end.1 && cx <= widget_end.0 { // <=
                                    let pos = (rect.0 + left_offset + (x as usize) * grid_size, rect.1 + top_offset + (y as usize) * grid_size);

                                    context.draw2d.draw_rect(frame, &(pos.0, pos.1, grid_size, grid_size), context.width, &context.color_white);
                                }
                            }
                        }
                    }
                }
            }

            /*
            let editor_mode = options.get_editor_mode();

            let mut rect = self.rect.clone();
            if editor_mode != RegionEditorMode::Areas {
                rect.3 -= 250;
            }

            if let Some(region) = context.data.regions.get(&self.region_id) {

                if context.is_running == false {
                    let x_tiles = (rect.2 / grid_size) as isize;
                    let y_tiles = (rect.3 / grid_size) as isize;

                    for y in 0..y_tiles {
                        for x in 0..x_tiles {
                            let values = region.get_value((x - self.offset.0, y - self.offset.1));

                            if values.is_empty() == false {
                                let pos = (rect.0 + left_offset + (x as usize) * grid_size, rect.1 + top_offset + (y as usize) * grid_size);
                                for value in values {
                                    let map = asset.get_map_of_id(value.0);
                                    context.draw2d.draw_animated_tile(frame, &pos, map,context.width,&(value.1, value.2), anim_counter, grid_size);
                                }
                            }
                        }
                    }
                } else {
                    context.draw2d.draw_region_with_instances(frame, region, &rect, &(-self.offset.0, -self.offset.1), context.width, grid_size, anim_counter, asset, context);
                }
            }

            if editor_mode == RegionEditorMode::Tiles {
                self.tile_selector.draw(frame, context.width, anim_counter, asset, context);
            } else
            if editor_mode == RegionEditorMode::Areas || editor_mode == RegionEditorMode::Behavior {
                if let Some(region) = context.data.regions.get(&self.region_id) {

                    let x_tiles = (rect.2 / grid_size) as isize;
                    let y_tiles = (rect.3 / grid_size) as isize;

                    let curr_area_index = context.curr_region_area_index;

                    for y in 0..y_tiles {
                        for x in 0..x_tiles {

                            let rx = x - self.offset.0;
                            let ry = y - self.offset.1;

                            for area_index in 0..region.data.areas.len() {

                                if region.data.areas[area_index].area.contains(&(rx, ry)) {
                                    let pos = (rect.0 + left_offset + (x as usize) * grid_size, rect.1 + top_offset + (y as usize) * grid_size);

                                    let mut c = context.color_white.clone();
                                    if curr_area_index == area_index {
                                        c[3] = 100;
                                    } else {
                                        if editor_mode == RegionEditorMode::Areas {
                                            continue;
                                        }
                                        c[3] = 50;
                                    }
                                    context.draw2d.blend_rect(frame, &(pos.0, pos.1, grid_size, grid_size), context.width, &c);
                                }
                            }
                        }
                    }
                }
            }*/

            if self.mouse_hover_pos != (0,0) {
                if let Some(id) = self.get_tile_id(self.mouse_hover_pos) {
                    let pos = (rect.0 + left_offset + ((id.0 + self.offset.0) as usize) * grid_size, rect.1 + top_offset + ((id.1 + self.offset.1) as usize) * grid_size);

                    if  pos.0 + grid_size < rect.0 + rect.2 && pos.1 + grid_size < rect.1 + rect.3 {
                        context.draw2d.draw_rect_outline(frame, &(pos.0, pos.1, grid_size, grid_size), context.width, context.color_light_white);
                        context.draw2d.draw_rect_outline(frame, &(pos.0 + 1, pos.1 + 1, grid_size - 2, grid_size - 2), context.width, context.color_black);
                    }
                }
            }
        }
    }

    fn mouse_down(&mut self, pos: (usize, usize), _asset: &mut Asset, _context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        let consumed = false;

        self.widget_start = None;
        self.widget_end = None;

        if let Some(options) = options {

            let mode = options.get_screen_editor_mode();

            if mode.0 == ScreenEditorMode::Widgets {
                if mode.1 == ScreenEditorAction::Add {
                    if let Some(id) = self.get_tile_id(pos) {
                        //println!("{:?}", id);
                        self.widget_start = Some(id);
                        self.widget_end = Some(id);
                    }
                }
            }
        }

        /*
        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Tiles {
                if self.tile_selector.mouse_down(pos, asset, context) {
                    consumed = true;
                    if let Some(selected) = &self.tile_selector.selected {
                        context.curr_region_tile = Some(selected.clone());
                    } else {
                        context.curr_region_tile = None;
                    }
                }
            }

            if consumed == false && context.contains_pos_for(pos, self.rect) {
                if let Some(id) = self.get_tile_id(pos) {
                    self.clicked = Some(id);
                    let editor_mode = options.get_editor_mode();

                    if editor_mode == RegionEditorMode::Tiles {
                        if let Some(selected) = &self.tile_selector.selected {
                            if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                region.set_value(options.get_layer(), id, selected.clone());
                                region.save_data();
                            }
                        }
                    } else
                    if editor_mode == RegionEditorMode::Areas {
                        if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                            if region.data.areas.len() > 0 {
                                let area = &mut region.data.areas[context.curr_region_area_index];
                                if area.area.contains(&id) == false {
                                    area.area.push(id);
                                }
                                region.save_data();
                            }
                        }
                    }
                }
                consumed = true;
            }
        }*/
        consumed
    }

    fn mouse_up(&mut self, _pos: (usize, usize), _asset: &mut Asset, _context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {
        self.clicked = None;

        let consumed = false;

        if let Some(options) = options {
            //let editor_mode = options.get_editor_mode();

            let mode = options.get_screen_editor_mode();

            if mode.0 == ScreenEditorMode::Widgets {
                if mode.1 == ScreenEditorAction::Add {

                    if let Some(widget_start) = self.widget_start {
                        if let Some(widget_end) = self.widget_end {

                            let widget = GameScreenWidget { name: "Widget".to_string(), widget_type: server::gamedata::game_screen::GameScreenWidgetType::Game, top_left: widget_start, bottom_right: widget_end };
                            self.game_screen.widgets.push(widget);
                            self.game_screen.curr_widget_index = self.game_screen.widgets.len() - 1;
                        }
                    }
                }
            }
        }

        self.widget_start = None;
        self.widget_end = None;

        consumed
    }

    fn mouse_hover(&mut self, pos: (usize, usize), _asset: &mut Asset, _context: &mut ScreenContext, _options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {
        self.mouse_hover_pos = pos.clone();
        true
    }

    fn mouse_dragged(&mut self, pos: (usize, usize), _asset: &mut Asset, _context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        let mut consumed = false;

        if let Some(options) = options {
            //let editor_mode = options.get_editor_mode();

            let mode = options.get_screen_editor_mode();

            if mode.0 == ScreenEditorMode::Widgets {
                if mode.1 == ScreenEditorAction::Add {

                    if let Some(id) = self.get_tile_id(pos) {

                        if let Some(widget_start) = self.widget_start {

                            if id.0 >= widget_start.0 && id.1 >= widget_start.1 {
                                self.widget_end = Some(id);
                                consumed = true;
                            }
                        }
                    }
                }
            }

            /*
            if consumed == false && context.contains_pos_for(pos, self.rect) {
                if let Some(id) = self.get_tile_id(pos) {
                    if self.clicked != Some(id) {

                        self.clicked = Some(id);
                        let editor_mode = options.get_editor_mode();

                        if editor_mode == RegionEditorMode::Tiles {
                            if let Some(selected) = &self.tile_selector.selected {
                                if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                    region.set_value(options.get_layer(), id, selected.clone());
                                    region.save_data();
                                }
                            }
                        }
                    }
                }*/


        }
        consumed
    }

    fn mouse_wheel(&mut self, delta: (isize, isize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        let mut consumed = false;
        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Tiles {
                if context.contains_pos_for(self.mouse_hover_pos, self.tile_selector.rect) && self.tile_selector.mouse_wheel(delta, asset, context) {
                    consumed = true;
                }
            }

            if consumed == false {
                self.mouse_wheel_delta.0 += delta.0;
                self.mouse_wheel_delta.1 += delta.1;

                self.offset.0 -= self.mouse_wheel_delta.0 / self.grid_size as isize;
                self.offset.1 += self.mouse_wheel_delta.1 / self.grid_size as isize;

                self.mouse_wheel_delta.0 -= (self.mouse_wheel_delta.0 / self.grid_size as isize) * self.grid_size as isize;
                self.mouse_wheel_delta.1 -= (self.mouse_wheel_delta.1 / self.grid_size as isize) * self.grid_size as isize;
            }
        }
        true
    }

    /// Sets a region id
    fn set_region_id(&mut self, id: usize, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>) {
        self.region_id = id;

        if let Some(region) = context.data.regions.get_mut(&self.region_id) {
            if let Some(options) = options {
                options.set_area_names(region.get_area_names());
                context.curr_region_area_index = 0;
            }
        }
    }

    /// Get the tile id
    fn get_tile_id(&self, pos: (usize, usize)) -> Option<(isize, isize)> {
        let grid_size = self.grid_size;
        if pos.0 > self.rect.0 + self.screen_offset.0 && pos.1 > self.rect.1 + self.screen_offset.1
        && pos.0 < self.rect.0 + self.rect.2 - self.screen_offset.0  && pos.1 < self.rect.1 + self.rect.3 - self.screen_offset.1 //} - self.selector_size
        {
            let x = ((pos.0 - self.rect.0 - self.screen_offset.0) / grid_size) as isize - self.offset.0;
            let y = ((pos.1 - self.rect.1 - self.screen_offset.0) / grid_size) as isize - self.offset.1;
            return Some((x, y));
        }
        None
    }

    /// Returns the selected tile
    fn get_selected_tile(&self) -> Option<(usize, usize, usize, TileUsage)> {
        self.tile_selector.selected.clone()
    }

    /// Return the tile_selector
    fn get_tile_selector(&mut self) -> Option<&mut TileSelectorWidget> {
        Some(&mut self.tile_selector)
    }

    /// Returns the region_id
    fn get_region_id(&self) -> usize {
        self.region_id
    }

    /// Screen is opening
    fn opening(&mut self, _asset: &mut Asset, context: &mut ScreenContext, _options: &mut Option<Box<dyn EditorOptions>>) {

        self.node_behavior_id = context.dialog_node_behavior_id.clone();
        self.node_behavior_value = context.dialog_node_behavior_value.clone();

        if context.is_running == false {
            context.data.startup_client();

            // Execute the screen node script to get the screen dimensions and tile_size
            screen(context.data.game_instance_index.unwrap(), (self.node_behavior_id.0, self.node_behavior_id.1), &mut context.data, BehaviorType::GameLogic);
        }

        self.screen_width = context.data.game_screen_width;
        self.screen_height = context.data.game_screen_height;
        self.screen_tile_size = context.data.game_screen_tile_size;
        context.data.game_frame = vec![0; self.screen_width * self.screen_height * 4];

        self.game_screen = serde_json::from_str(&self.node_behavior_value.4.clone())
            .unwrap_or(GameScreen::new() );

        //if let Some(options) = options {
        //    options.update_area_ui(context, &mut Some(Box::new(*self)));
        //}
    }

    /// Screen is closing
    fn closing(&mut self, _asset: &mut Asset, context: &mut ScreenContext, _options: &mut Option<Box<dyn EditorOptions>>) {

        if let Some(json) = serde_json::to_string(&self.game_screen).ok() {
            self.node_behavior_value.4 = json;
            context.dialog_node_behavior_value = self.node_behavior_value.clone();
            context.dialog_node_behavior_id = self.node_behavior_id.clone();

            context.data.set_behavior_id_value(context.dialog_node_behavior_id.clone(), context.dialog_node_behavior_value.clone(), BehaviorType::GameLogic);
        }

        if context.is_running == false {
            context.data.shutdown_client();
        }
    }

    /// Returns the game screen
    fn get_game_screen(&mut self) -> Option<&mut GameScreen> {
        Some(&mut self.game_screen)
    }

}