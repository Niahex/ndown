use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    use crate::ui::top_bar::*;
    use crate::ui::file_explorer::*;
    use crate::ui::outline_panel::*;
    use crate::ui::editor::*;
    
    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1400, 900)},
            pass: {clear_color: #2e3440},
            
            body = <View> {
                width: Fill, height: Fill
                flow: Down
                
                top_bar = <TopBar> {}
                
                content = <View> {
                    width: Fill, height: Fill
                    flow: Right
                    
                    left_sidebar = <FileExplorer> {}
                    
                    editor = <EditorArea> {}
                    
                    right_sidebar = <OutlinePanel> {}
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] left_visible: bool,
    #[rust] right_visible: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Debug: afficher toutes les actions
        for action in actions {
            log!("Action received: {:?}", action);
        }
        
        if self.ui.button(ids!(ui.body.top_bar.left_toggle)).clicked(actions) {
            log!("Left toggle clicked!");
            self.left_visible = !self.left_visible;
            self.ui.view(ids!(ui.body.content.left_sidebar)).set_visible(cx, self.left_visible);
            self.ui.redraw(cx);
        }
        
        if self.ui.button(ids!(ui.body.top_bar.right_toggle)).clicked(actions) {
            log!("Right toggle clicked!");
            self.right_visible = !self.right_visible;
            self.ui.view(ids!(ui.body.content.right_sidebar)).set_visible(cx, self.right_visible);
            self.ui.redraw(cx);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            ui: WidgetRef::default(),
            left_visible: true,
            right_visible: true,
        }
    }
}
