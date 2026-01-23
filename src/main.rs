pub mod app;
pub mod logger;
pub mod block;
pub mod editor_state;
pub mod block_editor;
pub mod templates;
pub mod markdown;
pub mod ui;
pub mod storage;
pub mod rich_text_input;

fn main() {
    logger::init();
    ::log::info!("Starting ndown markdown editor");
    app::app_main();
}
