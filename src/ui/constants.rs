use eframe::egui;

pub const IS_DEV: bool = cfg!(debug_assertions);
const APP_ID: &str = "com.nithinrdy.audiopass";

pub fn eframe_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_title("AudioPass")
            .with_inner_size([980.0, 720.0])
            .with_min_inner_size([760.0, 560.0]),
        ..Default::default()
    }
}
