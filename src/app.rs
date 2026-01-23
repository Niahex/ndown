use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use crate::top_bar::*;
    use crate::file_explorer::*;
    use crate::outline_panel::*;
    use crate::editor::*;

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
    #[live]
    ui: WidgetRef,
    #[rust]
    left_visible: bool,
    #[rust]
    right_visible: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self
            .ui
            .button(ids!(body.top_bar.left_toggle))
            .clicked(actions)
        {
            self.left_visible = !self.left_visible;
            if self.left_visible {
                self.ui
                    .view(ids!(body.content.left_sidebar))
                    .apply_over(cx, live! {width: 250});
            } else {
                self.ui
                    .view(ids!(body.content.left_sidebar))
                    .apply_over(cx, live! {width: 0});
            }
            self.ui.redraw(cx);
        }

        if self
            .ui
            .button(ids!(body.top_bar.right_toggle))
            .clicked(actions)
        {
            self.right_visible = !self.right_visible;
            if self.right_visible {
                self.ui
                    .view(ids!(body.content.right_sidebar))
                    .apply_over(cx, live! {width: 250});
            } else {
                self.ui
                    .view(ids!(body.content.right_sidebar))
                    .apply_over(cx, live! {width: 0});
            }
            self.ui.redraw(cx);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            let editor = self.ui.view(ids!(body.content.editor));
            cx.set_key_focus(editor.area());
        }
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
