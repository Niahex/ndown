pub use makepad_widgets;
use makepad_widgets::*;

pub mod app;

pub mod editor;
pub mod file_explorer;
pub mod outline_panel;
pub mod top_bar;

pub fn live_design(cx: &mut Cx) {
    makepad_widgets::live_design(cx);
    makepad_code_editor::live_design(cx);
    
    editor::live_design(cx);
    file_explorer::live_design(cx);
    outline_panel::live_design(cx);
    top_bar::tabs::live_design(cx);
    top_bar::live_design(cx);
}
