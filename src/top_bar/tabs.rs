use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub TabItem = {{TabItem}} {
        view: <View> {
            width: Fit, height: Fit
            padding: {left: 10, right: 10, top: 5, bottom: 5}
            spacing: 5
            flow: Right
            show_bg: true
            draw_bg: {
                color: (NORD_POLAR_3)
            }

            <Label> {
                text: "document.md"
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 12}
                    color: (NORD_SNOW_2)
                }
            }

            close_btn = <Button> {
                text: "×"
                draw_text: {
                    color: (NORD_SNOW_0)
                    text_style: <THEME_FONT_REGULAR> {font_size: 14}
                }
            }
        }
    }

    pub TabBar = {{TabBar}} {
        view: <View> {
            width: Fill, height: Fit
            flow: Right, spacing: 5
            padding: {left: 5, right: 5}

            <TabItem> {}
            <TabItem> {}
        }
    }
}

#[derive(Live, Widget)]
pub struct TabItem {
    #[deref]
    #[live]
    view: View,
}

impl LiveHook for TabItem {}

impl Widget for TabItem {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

#[derive(Live, Widget)]
pub struct TabBar {
    #[deref]
    #[live]
    view: View,
}

impl LiveHook for TabBar {}

impl Widget for TabBar {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
