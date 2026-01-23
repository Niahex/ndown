pub mod widget;
pub mod cursor;
pub mod events;
pub mod history;
pub mod types;
pub mod formatting;
pub mod text_mapping;
pub mod actions;
pub mod inline;

pub use widget::RichTextInput;
pub use types::*;
pub use actions::RichTextInputAction;

use makepad_widgets::*;

pub fn live_design(cx: &mut Cx) {
    widget::live_design(cx);
}

