use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub TopBar = {{TopBar}}{
        view: <View> {
            width: Fill, height: Fit
            flow: Right, spacing: 0, padding: {left: 10, right: 10, top: 10, bottom: 10}
            show_bg: true
            draw_bg: { color: (NORD_POLAR_1) }

            <View> { width: Fill }

            title = <Label> {
                text: "Ndown Editor"
                draw_text: {
                    text_style: <THEME_FONT_BOLD> {font_size: 14}
                    color: (NORD_SNOW_2)
                }
            }

            <View> { width: Fill }
        }
    }
}

#[derive(Live, Widget)]
pub struct TopBar {
    #[deref]
    #[live]
    view: View,
}

impl LiveHook for TopBar {}

impl Widget for TopBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope)
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
