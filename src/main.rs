mod backend;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    let instance = single_instance::SingleInstance::new(backend::constants::AUDIOPASS_APP_ID).map_err(|error| eframe::Error::AppCreation(Box::new(error)))?;
    if !instance.is_single() {
        return Ok(());
    }

    ui::run_app()
}
