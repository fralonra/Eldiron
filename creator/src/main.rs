#![windows_subsystem = "windows"]

use theframework::*;

pub mod editor;
pub mod effectpicker;
pub mod hud;
pub mod mapeditor;
pub mod materialpicker;
pub mod minimap;
pub mod misc;
pub mod panels;
pub mod previewview;
pub mod self_update;
pub mod settingspicker;
pub mod sidebar;
pub mod texteditor;
pub mod tilemapeditor;
pub mod tilepicker;
pub mod toollist;
pub mod tools;
pub mod undo;
pub mod utils;

use rust_embed::RustEmbed;
#[derive(RustEmbed)]
#[folder = "embedded/"]
#[exclude = "*.txt"]
#[exclude = "*.DS_Store"]
pub struct Embedded;

const DEFAULT_VLAYOUT_RATIO: f32 = 0.62;

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    pub use ::serde::{Deserialize, Serialize};

    pub use shared::prelude::*;
    pub use std::sync::{LazyLock, RwLock};
    pub use theframework::prelude::*;

    pub use crate::effectpicker::*;
    pub use crate::mapeditor::*;
    pub use crate::materialpicker::*;
    pub use crate::misc::*;
    pub use crate::panels::*;
    pub use crate::previewview::*;
    pub use crate::sidebar::*;
    pub use crate::texteditor::*;
    pub use crate::tilemapeditor::*;
    pub use crate::tilepicker::*;
    pub use crate::toollist::*;
    pub use crate::undo::material_undo::*;
    pub use crate::undo::palette_undo::*;
    pub use crate::undo::region_undo::*;
    pub use crate::undo::*;
    pub use crate::utils::*;

    pub use crate::tools::code::CodeTool;
    pub use crate::tools::fx::FXTool;
    pub use crate::tools::game::GameTool;
    pub use crate::tools::linedef::LinedefTool;
    pub use crate::tools::sector::SectorTool;
    pub use crate::tools::selection::SelectionTool;
    pub use crate::tools::tilemap::TilemapTool;
    pub use crate::tools::vertex::VertexTool;

    pub use crate::tools::*;

    pub use crate::settingspicker::SettingsPicker;
}

use crate::editor::Editor;

fn main() {
    let args: Vec<_> = std::env::args().collect();

    std::env::set_var("RUST_BACKTRACE", "1");

    let editor = Editor::new();
    let mut app = TheApp::new();
    app.set_cmd_line_args(args);

    let () = app.run(Box::new(editor));
}
