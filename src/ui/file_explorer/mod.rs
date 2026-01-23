use makepad_widgets::*;

live_design!{
    use link::theme::*;
    use link::widgets::*;
        
    pub FileExplorer = {{FileExplorer}}{
        view: <View> {
            width: 250, height: Fill
            flow: Down, padding: 10
            show_bg: true
            draw_bg: { color: #434c5e }
            visible: true
            
            <Label> {
                text: "File Explorer"
                draw_text: {
                    text_style: <THEME_FONT_BOLD> {font_size: 16}
                    color: #eceff4
                }
            }
            
            <Label> {
                margin: {top: 20}
                text: "📁 Documents\n📁 Projects\n📄 README.md"
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 14}
                    color: #d8dee9
                }
            }
        }
    }
}
 
#[derive(Live, Widget)] 
pub struct FileExplorer{
    #[deref] #[live] view: View
}

impl LiveHook for FileExplorer{}

impl Widget for FileExplorer {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }
    
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
