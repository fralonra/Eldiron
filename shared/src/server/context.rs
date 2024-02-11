use crate::prelude::*;
use theframework::prelude::*;

/// This gives context to the server of the editing state for live highlighting.
pub struct ServerContext {
    /// The currently selected region in the editor.
    pub curr_region: Uuid,

    /// The currently selected character in the editor.
    pub curr_character: Option<Uuid>,

    /// The currently selected character instance in the editor.
    pub curr_character_instance: Option<Uuid>,

    /// The currently selected item in the editor.
    pub curr_item: Option<Uuid>,

    /// The currently selected item instance in the editor.
    pub curr_item_instance: Option<Uuid>,

    /// The currently selected area in the editor.
    pub curr_area: Option<Uuid>,

    /// The currently selected codegrid in the code editor.
    pub curr_grid_id: Option<Uuid>,

    /// If the user selects a tile area.
    pub tile_selection: Option<TileArea>,

    /// The currently selected screen.
    pub curr_screen: Uuid,
}

impl Default for ServerContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerContext {
    pub fn new() -> Self {
        Self {
            curr_region: Uuid::nil(),

            curr_character: None,
            curr_character_instance: None,

            curr_item: None,
            curr_item_instance: None,

            curr_area: None,

            curr_grid_id: None,

            tile_selection: None,

            curr_screen: Uuid::nil(),
        }
    }

    pub fn clear(&mut self) {
        self.curr_region = Uuid::nil();
        self.curr_character = None;
        self.curr_character_instance = None;
        self.curr_item = None;
        self.curr_item_instance = None;
        self.curr_area = None;
        self.curr_grid_id = None;
        self.tile_selection = None;
        self.curr_screen = Uuid::nil();
    }
}
