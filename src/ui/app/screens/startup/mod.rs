use crate::{ui, ui::app::MainScreen};
use eframe::egui;

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.heading("AudioPass");
    ui.label("Startup Screen");

    if ui.button("Continue").clicked() {
        app.startup_complete = true;
        app.active_screen = Some(MainScreen::Console);
    }
}
