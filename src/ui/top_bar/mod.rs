use makepad_widgets::*;

pub mod tabs;
pub use tabs::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::ui::top_bar::tabs::*;
    
    pub TopBar = {{TopBar}} {
        width: Fill, height: Fit
        flow: Right, spacing: 10, padding: 10
        show_bg: true
        draw_bg: { color: #3b4252 }
        
        left_toggle = <Button> {
            text: "☰ Files"
            draw_text: { color: #eceff4 }
        }
        
        tabs = <TabBar> {}
        
        right_toggle = <Button> {
            text: "Outline ☰"
            draw_text: { color: #eceff4 }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TopBar {
    #[deref] view: View,
}

impl Widget for TopBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
