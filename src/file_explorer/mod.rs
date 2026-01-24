use makepad_widgets::*;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, DefaultNone, Debug)]
pub enum FileExplorerAction {
    FileSelected(String),
    None,
}

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub FileExplorer = {{FileExplorer}}{
        width: 250, height: Fill
        flow: Down, padding: 10
        show_bg: true
        draw_bg: { color: (NORD_POLAR_1) }

        title = <Label> {
            margin: {bottom: 10}
            text: "EXPLORATEUR"
            draw_text: { text_style: <THEME_FONT_BOLD> {font_size: 12}, color: (NORD_FROST_2) }
        }

        file_list = <PortalList> {
            width: Fill, height: Fill
            flow: Down

            FileItem = <View> {
                width: Fill, height: 30, flow: Overlay
                
                content = <View> {
                    width: Fill, height: Fill, flow: Right, align: {y: 0.5}, padding: 5
                    
                    icon = <Label> { text: "📄", draw_text: { color: (NORD_FROST_1) } }
                    name = <Label> {
                        text: "filename.md",
                        draw_text: { text_style: <THEME_FONT_REGULAR> {font_size: 11}, color: (NORD_SNOW_0) }
                    }
                }
                
                btn = <Button> {
                    width: Fill, height: Fill
                    draw_bg: {
                        fn pixel(self) -> vec4 { return vec4(0.,0.,0.,0.); }
                    }
                    text: ""
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
                    if !name.starts_with('.') {
                        self.files.push(name);
                    }
                }
            }
        }
        self.files.sort();
    }
    
    pub fn handle_file_actions(&mut self, _cx: &mut Cx, actions: &Actions) -> Option<String> {
        let list = self.view.portal_list(ids!(file_list));
        for (item_id, item) in list.items_with_actions(actions) {
             if item.button(ids!(btn)).clicked(actions) {
                 return self.files.get(item_id).cloned();
             }
        }
        None
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
                        item.view(ids!(content)).label(ids!(name)).set_text(cx, file_name);
                        item.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}