mod backend;
mod ui;
use ui::constants::IS_DEV;

fn main() -> eframe::Result<()> {
    if IS_DEV {
        unsafe {
            std::env::set_var("PIPEWIRE_DEBUG", "4");
        }
    }

    ui::run_app()
}
