use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    pub EditorArea = {{EditorArea}} {
        width: Fill, height: Fill
        flow: Down, padding: 20
        show_bg: true
        draw_bg: { color: #2e3440 }
        
        <Label> {
            text: "Editor Area"
            draw_text: {
                text_style: <THEME_FONT_BOLD> {font_size: 20}
                color: #eceff4
            }
        }
        
        <Label> {
            margin: {top: 10}
            text: "Markdown content will go here..."
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 14}
                color: #d8dee9
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct EditorArea {
    #[deref] view: View,
}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
