use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    pub TabItem = {{TabItem}} {
        width: Fit, height: Fit
        padding: {left: 10, right: 10, top: 5, bottom: 5}
        spacing: 5
        flow: Right
        show_bg: true
        draw_bg: { 
            color: #4c566a
            radius: 3.0
        }
        
        <Label> {
            text: "document.md"
            draw_text: {
                text_style: <THEME_FONT_REGULAR> {font_size: 12}
                color: #eceff4
            }
        }
        
        close_btn = <Button> {
            text: "×"
            draw_text: { 
                color: #d8dee9
                text_style: <THEME_FONT_REGULAR> {font_size: 14}
            }
        }
    }
    
    pub TabBar = {{TabBar}} {
        width: Fill, height: Fit
        flow: Right, spacing: 5
        padding: {left: 5, right: 5}
        
        <TabItem> {}
        <TabItem> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TabItem {
    #[deref] view: View,
}

impl Widget for TabItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TabBar {
    #[deref] view: View,
}

impl Widget for TabBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
