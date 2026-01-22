use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    use crate::block_editor::BlockEditor;
    
    App = {{App}} {
        ui: <Root> {
            main_window = <Window> {
                window: {inner_size: vec2(800, 600)}
                show_bg: true
                draw_bg: {
                    fn pixel(self) -> vec4 {
                        return #1a1a1a
                    }
                }
                body = <View> {
                    flow: Down
                    padding: 20
                    spacing: 10
                    
                    <Label> {
                        draw_text: {
                            text_style: <THEME_FONT_BOLD> {font_size: 20}
                            color: #ffffff
                        }
                        text: "ndown - Markdown Editor"
                    }
                    
                    block_editor = <BlockEditor> {}
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
        crate::block_editor::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, _cx: &mut Cx) {
        ::log::info!("App started");
    }
    
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
