pub mod app;
pub mod logger;
pub mod block;
pub mod editor_state;
pub mod block_editor;
pub mod templates;
pub mod markdown;
pub mod ui;
pub mod storage;

fn main() {
    logger::init();
    ::log::info!("Starting ndown markdown editor");
    app::app_main();
}
