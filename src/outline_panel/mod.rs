use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub OutlinePanel = {{OutlinePanel}}{
        view: <View> {
            width: 250, height: Fill
            flow: Down, padding: 10
            show_bg: true
            draw_bg: { color: #434c5e }
            visible: true

            <Label> {
                margin: {top: 20}
                text: "# Heading 1\n## Heading 2\n### Heading 3"
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 14}
                    color: #d8dee9
                }
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct OutlinePanel {
    #[deref]
    #[live]
    view: View,
}

impl LiveHook for OutlinePanel {}

impl Widget for OutlinePanel {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
