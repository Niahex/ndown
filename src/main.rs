pub mod app;
pub mod logger;
pub mod block;
pub mod editor_state;

fn main() {
    logger::init();
    ::log::info!("Starting ndown markdown editor");
    app::app_main();
}
