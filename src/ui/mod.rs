mod app;
mod constants;

pub fn run_app() -> eframe::Result<()> {
    eframe::run_native(
        "AudioPass",
        constants::eframe_options(),
        Box::new(|_| Ok(Box::new(app::App::new()))),
    )
}

pub use app::App;
