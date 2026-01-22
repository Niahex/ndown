use makepad_widgets::*;
use crate::markdown::parser::{detect_heading_level, detect_list_item, ListType};

pub fn get_template_for_block(content: &str, is_active: bool) -> LiveId {
    // Check for lists first
    if let Some(list_info) = detect_list_item(content) {
        match list_info.list_type {
            ListType::Unordered => {
                if is_active { 
                    live_id!(BlockInputList) 
                } else { 
                    live_id!(BlockInputInactiveList) 
                }
            }
            ListType::Ordered => {
                if is_active { 
                    live_id!(BlockInputOrderedList) 
                } else { 
                    live_id!(BlockInputInactiveOrderedList) 
                }
            }
        }
    } else {
        // Check for headings
        match detect_heading_level(content) {
            Some(1) => if is_active { live_id!(BlockInputH1) } else { live_id!(BlockInputInactiveH1) },
            Some(2) => if is_active { live_id!(BlockInputH2) } else { live_id!(BlockInputInactiveH2) },
            Some(3) => if is_active { live_id!(BlockInputH3) } else { live_id!(BlockInputInactiveH3) },
            _ => if is_active { live_id!(BlockInput) } else { live_id!(BlockInputInactive) },
        }
    }
}
