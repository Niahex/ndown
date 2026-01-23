use makepad_widgets::*;

live_design!{
    use link::theme::*;
    use link::widgets::*;
        
    pub EditorArea = {{EditorArea}}{
        view: <View> {
            width: Fill, height: Fill
            flow: Down, padding: 20
            show_bg: true
            draw_bg: { color: #2e3440 }
        }
    }
}
 
#[derive(Live, Widget)] 
pub struct EditorArea{
    #[deref] #[live] view: View
}

impl LiveHook for EditorArea{}

impl Widget for EditorArea {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
