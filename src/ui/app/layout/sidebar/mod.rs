use crate::ui::{App, app::screens::MainScreen};
use eframe::egui::{self, RichText};

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.label(
            RichText::new("AudioPass")
                .size(30.0)
                .extra_letter_spacing(1.5)
                .strong(),
        );

        ui.add_space(20.0);

        if ui.button("Console").clicked() {
            app.active_screen = Some(MainScreen::Console);
        }
        if ui.button("Help").clicked() {
            app.active_screen = Some(MainScreen::Help);
        }
        if ui.button("About").clicked() {
            app.active_screen = Some(MainScreen::About);
        }
    });
}
