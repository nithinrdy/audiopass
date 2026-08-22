use crate::backend::pipewire;
mod app;
pub mod constants;
mod style;

pub fn run_app() -> eframe::Result<()> {
    eframe::run_native(
        "AudioPass",
        constants::eframe_options(),
        Box::new(|context| {
            // passing context to pipewire thread to let it trigger repaints from the other side because
            // https://stackoverflow.com/a/77211190
            let pipewire_instance = pipewire::start_pipewire_worker(context.egui_ctx.clone())?;
            Ok(Box::new(app::App::new(pipewire_instance)))
        }),
    )
}

pub use app::App;
