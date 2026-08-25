use eframe::egui;
use crate::backend::constants;

pub const IS_DEV: bool = cfg!(debug_assertions);

pub fn eframe_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(constants::AUDIOPASS_APP_ID)
            .with_title("AudioPass")
            .with_inner_size([980.0, 720.0])
            .with_min_inner_size([760.0, 560.0]),
        ..Default::default()
    }
}
