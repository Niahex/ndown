pub fn generate_heading_templates() -> String {
    let headings = [
        ("H1", 25.0),
        ("H2", 21.0), 
        ("H3", 18.0),
    ];
    
    let mut templates = String::new();
    
    for (level, size) in headings {
        // Active template
        templates.push_str(&format!(
            "BlockInput{} = <TextInput> {{
                width: Fill, height: Fit, padding: 10
                draw_bg: {{ color: #2a2a2a }}
                draw_text: {{
                    text_style: <THEME_FONT_REGULAR> {{font_size: {}}}
                    color: #ffffff
                }}
            }}
            
            ", level, size
        ));
        
        // Inactive template  
        templates.push_str(&format!(
            "BlockInputInactive{} = <TextInput> {{
                width: Fill, height: Fit, padding: 10
                is_read_only: true
                draw_bg: {{ color: #1a1a1a }}
                draw_text: {{
                    text_style: <THEME_FONT_REGULAR> {{font_size: {}}}
                    color: #cccccc
                }}
            }}
            
            ", level, size
        ));
    }
    
    templates
}
