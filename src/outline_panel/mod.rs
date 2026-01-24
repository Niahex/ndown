use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub OutlinePanel = {{OutlinePanel}}{
        view: <View> {
            width: 250, height: Fill
            flow: Down, padding: 10
            show_bg: true
            draw_bg: { color: (NORD_POLAR_1) }
            visible: true
            clip_x: true

            header = <View> {
                width: Fill, height: Fit
                flow: Right, spacing: 10, align: {y: 0.5}, margin: {bottom: 10}
                
                toggle_btn = <Button> {
                    width: 30, height: 30
                    text: "☰"
                    draw_text: { color: (NORD_SNOW_2) }
                }

                <Label> {
                    text: "STRUCTURE"
                    draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 12}, color: (NORD_FROST_2) }
                }
            }

            <Label> {
                margin: {top: 20}
                text: "# Heading 1\n## Heading 2\n### Heading 3"
                draw_text: {
                    text_style: <THEME_FONT_REGULAR> {font_size: 14}
                    color: (NORD_SNOW_0)
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
