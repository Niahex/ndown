pub use makepad_widgets;
use makepad_widgets::*;

pub mod ui;
pub mod app;
pub mod model;

pub fn live_design(cx: &mut Cx) {
    makepad_widgets::live_design(cx);
    makepad_code_editor::live_design(cx);
    ui::editor::live_design(cx);
    ui::file_explorer::live_design(cx);
    ui::outline_panel::live_design(cx);
    ui::top_bar::tabs::live_design(cx);
    ui::top_bar::live_design(cx);
}
