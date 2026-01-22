use makepad_widgets::*;
use crate::editor_state::EditorState;
use crate::block::{Block, BlockType};

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;
    
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
                    
                    <View> {
                        flow: Down
                        spacing: 5
                        
                        block_0 = <Label> {
                            width: Fill
                            padding: 10
                            draw_text: {
                                text_style: <THEME_FONT_REGULAR> {font_size: 14}
                                color: #ffffff
                            }
                        }
                        
                        block_1 = <Label> {
                            width: Fill
                            padding: 10
                            draw_text: {
                                text_style: <THEME_FONT_BOLD> {font_size: 24}
                                color: #ffffff
                            }
                        }
                        
                        block_2 = <Label> {
                            width: Fill
                            padding: 10
                            draw_text: {
                                text_style: <THEME_FONT_REGULAR> {font_size: 14}
                                color: #ffffff
                            }
                        }
                        
                        block_3 = <Label> {
                            width: Fill
                            padding: 10
                            draw_text: {
                                text_style: <THEME_FONT_BOLD> {font_size: 20}
                                color: #ffffff
                            }
                        }
                        
                        block_4 = <Label> {
                            width: Fill
                            padding: 10
                            draw_text: {
                                text_style: <THEME_FONT_REGULAR> {font_size: 14}
                                color: #ffffff
                            }
                        }
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
    #[rust] editor_state: EditorState,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl MatchEvent for App {
    fn handle_startup(&mut self, cx: &mut Cx) {
        ::log::info!("App started with {} blocks", self.editor_state.blocks().len());
        
        // Demo: Create some test blocks
        self.editor_state.create_block(1, Block::heading(1, "Welcome to ndown".to_string()));
        self.editor_state.create_block(2, Block::text("This is a test block".to_string()));
        self.editor_state.create_block(3, Block::heading(2, "Features".to_string()));
        self.editor_state.create_block(4, Block::text("Block-based editing".to_string()));
        
        ::log::info!("Created demo blocks. Total blocks: {}", self.editor_state.blocks().len());
        
        // Update labels with block content
        self.update_blocks(cx);
    }
    
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {}
}

impl App {
    fn update_blocks(&mut self, cx: &mut Cx) {
        for (index, block) in self.editor_state.blocks().iter().enumerate() {
            let content = match &block.block_type {
                BlockType::Text => block.content.clone(),
                BlockType::Heading(level) => {
                    format!("{} {}", "#".repeat(*level as usize), block.content)
                }
            };
            
            match index {
                0 => self.ui.label(ids!(block_0)).set_text(cx, &content),
                1 => self.ui.label(ids!(block_1)).set_text(cx, &content),
                2 => self.ui.label(ids!(block_2)).set_text(cx, &content),
                3 => self.ui.label(ids!(block_3)).set_text(cx, &content),
                4 => self.ui.label(ids!(block_4)).set_text(cx, &content),
                _ => continue,
            };
        }
        self.ui.redraw(cx);
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
    }
}
