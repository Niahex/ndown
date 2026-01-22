use makepad_widgets::*;
use crate::markdown::detect_heading_level;

pub fn get_template_for_block(content: &str, is_active: bool) -> LiveId {
    match detect_heading_level(content) {
        Some(1) => if is_active { live_id!(BlockInputH1) } else { live_id!(BlockInputInactiveH1) },
        Some(2) => if is_active { live_id!(BlockInputH2) } else { live_id!(BlockInputInactiveH2) },
        Some(3) => if is_active { live_id!(BlockInputH3) } else { live_id!(BlockInputInactiveH3) },
        _ => if is_active { live_id!(BlockInput) } else { live_id!(BlockInputInactive) },
    }
}
