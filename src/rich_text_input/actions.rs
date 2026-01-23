use makepad_widgets::*;

#[derive(Clone, Debug, DefaultNone)]
pub enum RichTextInputAction {
    None,
    KeyFocus,
    KeyFocusLost,
    Changed(String),
    KeyDownUnhandled(KeyEvent),
}
