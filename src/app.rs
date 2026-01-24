use makepad_widgets::*;

// Imports nécessaires pour les actions et extensions de widgets
use crate::file_explorer::*; 
use crate::editor::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    use crate::top_bar::*;
    use crate::file_explorer::*;
    use crate::outline_panel::*;
    use crate::editor::*;

    App = {{App}} {
        ui: <Window> {
            window: {inner_size: vec2(1400, 900)},
            pass: {clear_color: (NORD_POLAR_0)},

            body = <View> {
                width: Fill, height: Fill
                flow: Right

                left_sidebar = <FileExplorer> {}

                center = <View> {
                    width: Fill, height: Fill
                    flow: Down
                    
                    top_bar = <TopBar> {}
                    editor = <EditorArea> {}
                }

                right_sidebar = <OutlinePanel> {}
            }
        }
    }
}

app_main!(App);

#[derive(Live, LiveHook)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust(true)]
    left_visible: bool,
    #[rust(true)]
    right_visible: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        crate::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Toggle Gauche (File Explorer)
        if self.ui.button(ids!(body.left_sidebar.header.toggle_btn)).clicked(actions) {
            self.left_visible = !self.left_visible;
            if self.left_visible {
                self.ui.view(ids!(body.left_sidebar)).apply_over(cx, live! {width: 250});
            } else {
                self.ui.view(ids!(body.left_sidebar)).apply_over(cx, live! {width: 50});
            }
            self.ui.redraw(cx);
        }

        // Toggle Droite (Outline Panel)
        if self.ui.button(ids!(body.right_sidebar.header.toggle_btn)).clicked(actions) {
            self.right_visible = !self.right_visible;
            if self.right_visible {
                self.ui.view(ids!(body.right_sidebar)).apply_over(cx, live! {width: 250});
            } else {
                self.ui.view(ids!(body.right_sidebar)).apply_over(cx, live! {width: 50});
            }
            self.ui.redraw(cx);
        }
        
        // Gestion File Explorer via le helper
        if let Some(file_explorer) = self.ui.file_explorer(ids!(body.left_sidebar)).borrow() {
            if let Some(path) = file_explorer.handle_file_actions(cx, actions) {
                // Mettre à jour le titre dans la TopBar
                self.ui.label(ids!(body.center.top_bar.title)).set_text(cx, &path);
                
                // Charger le fichier dans l'éditeur
                let mut editor = self.ui.editor_area(ids!(body.center.editor));
                editor.load_file(cx, path);
            }
        }
        
        // Gestion des actions de l'éditeur (s'il en émet)
        // Note: EditorArea n'émet pas encore d'actions WidgetAction formelles dans le code actuel,
        // mais si on voulait écouter des événements:
        for action in actions {
            if let Some(EditorAction::FileLoaded) = action.as_widget_action().cast() {
                makepad_widgets::log!("File loaded successfully.");
            }
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::Startup = event {
            let initial_file = "story.md".to_string();
            self.ui.label(ids!(body.center.top_bar.title)).set_text(cx, &initial_file);
            self.ui.editor_area(ids!(body.center.editor)).load_file(cx, initial_file);

            let editor = self.ui.view(ids!(body.center.editor));
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