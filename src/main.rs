pub mod app;
pub mod logger;

fn main() {
    logger::init();
    log::info!("Starting ndown markdown editor");
    app::app_main();
}
