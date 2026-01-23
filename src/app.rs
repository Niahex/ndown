use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(800, 600)},
            pass: {clear_color: #2e3440},
            
            body = <View> {
                width: Fill, height: Fill
                flow: Down, spacing: 20, padding: 20
                
                <Label> {
                    text: "RichTextInput Test"
                    draw_text: {
                        text_style: <THEME_FONT_BOLD> {font_size: 20}
                        color: #eceff4
                    }
                }
                
                <Label> {
                    text: "Type with markdown: **bold**, *italic*, _underline_, `code`"
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 12}
                        color: #d8dee9
                    }
                }
                
                editor = <RichTextInputBase> {
                    width: Fill, height: 200
                }
                
                <Label> {
                    text: "Shortcuts: Ctrl+B (bold), Ctrl+I (italic)"
                    draw_text: {
                        text_style: <THEME_FONT_REGULAR> {font_size: 12}
                        color: #d8dee9
                    }
                }
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
        crate::rich_text_input::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {}
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
