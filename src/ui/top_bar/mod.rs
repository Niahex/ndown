use makepad_widgets::*;

pub mod tabs;
pub use tabs::*;

live_design!{
    use link::theme::*;
    use link::widgets::*;
    use crate::ui::top_bar::tabs::*;
        
    pub TopBar = {{TopBar}}{
        view: <View> {
            width: Fill, height: Fit
            flow: Right, spacing: 0, padding: {left: 10, right: 10, top: 10, bottom: 10}
            show_bg: true
            draw_bg: { color: #3b4252 }
            
            left_toggle = <Button> {
                text: "☰"
                draw_text: { color: #eceff4 }
            }
            
            <View> {
                width: 215, height: Fit
            }
            
            tabs = <TabBar> {
                width: Fill
            }
            
            <View> {
                width: 215, height: Fit
            }
            
            right_toggle = <Button> {
                text: "☰"
                draw_text: { color: #eceff4 }
            }
        }
    }
}
 
#[derive(Live, Widget)] 
pub struct TopBar{
    #[deref] #[live] view: View
}

impl LiveHook for TopBar{}

impl Widget for TopBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
