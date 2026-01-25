pub use makepad_widgets;
use makepad_widgets::*;

pub mod app;
pub mod theme;

pub mod editor;
pub mod file_explorer;
pub mod panel;
pub mod top_bar;

pub fn live_design(cx: &mut Cx) {
    makepad_widgets::live_design(cx);
    makepad_code_editor::live_design(cx);

    theme::live_design(cx);
    editor::live_design(cx);
    file_explorer::live_design(cx);
    panel::live_design(cx);
    top_bar::live_design(cx);
}
