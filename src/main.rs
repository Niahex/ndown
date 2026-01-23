pub mod app;
pub mod logger;
pub mod rich_text_input;

fn main() {
    logger::init();
    ::log::info!("Starting ndown markdown editor");
    app::app_main();
}
