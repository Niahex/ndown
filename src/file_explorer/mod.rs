use makepad_widgets::*;
use std::fs;
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub FileExplorer = {{FileExplorer}}{
        width: 250, height: Fill
        flow: Down, padding: 10
        show_bg: true
        draw_bg: { color: #434c5e }

        title = <Label> {
            margin: {bottom: 10}
            text: "EXPLORATEUR"
            draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 12}, color: #81a1c1 }
        }

        file_list = <PortalList> {
            width: Fill, height: Fill
            flow: Down

            // Le template doit être ICI
            FileItem = <View> {
                width: Fill, height: 30, flow: Right, align: {y: 0.5}, padding: 5
                show_bg: true
                        draw_bg: {
                            instance hover: 0.0
                            instance down: 0.0
                            color: #4c566a00
                            fn pixel(self) -> vec4 {
                                return mix(mix(self.color, #4c566a, self.hover), #3b4252, self.down);
                            }
                        }

                        animator: {
                            hover = {
                                default: off
                                off = { from: {all: Forward {duration: 0.1}} apply: { draw_bg: {hover: 0.0} } }
                                on = { from: {all: Forward {duration: 0.1}} apply: { draw_bg: {hover: 1.0} } }
                            }
                            down = {
                                default: off
                                off = { from: {all: Forward {duration: 0.1}} apply: { draw_bg: {down: 0.0} } }
                                on = { from: {all: Forward {duration: 0.1}} apply: { draw_bg: {down: 1.0} } }
                            }
                        }
                icon = <Label> { text: "📄", draw_text: { color: #88c0d0 } }
                name = <Label> {
                    text: "filename.md",
                    draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: #d8dee9 }
                }
            }
        }
    }
}

#[derive(Live, Widget)]
pub struct FileExplorer {
    #[deref]
    view: View,

    #[rust]
    files: Vec<String>,
}

impl LiveHook for FileExplorer {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        self.load_files();
    }
}

impl FileExplorer {
    fn load_files(&mut self) {
        let path = std::env::current_dir().unwrap_or(PathBuf::from("."));
        self.files.clear();

        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    // On affiche tout ce qui n'est pas caché pour l'instant
                    if !name.starts_with('.') {
                        self.files.push(name);
                    }
                }
            }
        }
        self.files.sort();
    }
}

impl Widget for FileExplorer {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.as_portal_list().borrow_mut() {
                list.set_item_range(cx, 0, self.files.len());
                while let Some(item_id) = list.next_visible_item(cx) {
                    if item_id < self.files.len() {
                        let file_name = &self.files[item_id];
                        let item = list.item(cx, item_id, live_id!(FileItem));
                        item.label(ids!(name)).set_text(cx, file_name);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}
