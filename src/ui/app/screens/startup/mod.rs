use crate::ui::{self, style};
use eframe::egui::{self, Button, RichText};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.set_width(400.0);
        ui.add_space(8.0);
        ui.label(RichText::new("AudioPass").color(style::ACCENT).size(32.0).extra_letter_spacing(1.0).strong());

        ui.add_space(20.0);

        ui.label(
            RichText::new("When you click \"Continue\", AudioPass will create a virtual microphone.")
                .size(16.0)
                .extra_letter_spacing(-0.5)
                .color(style::SECONDARY_TEXT),
        );

        ui.add_space(80.0);

        if ui
            .add_enabled(
                !app.startup_in_progress && app.critical_error.is_none(),
                Button::new(RichText::new(if app.startup_in_progress { "Starting…" } else { "Continue" }).color(style::CONTRAST_TEXT).size(18.0))
                    .fill(style::ACCENT)
                    .min_size(egui::Vec2 { x: 20.0, y: 40.0 }),
            )
            .clicked()
        {
            match app.pipewire_instance.create_virtual_mic() {
                Ok(()) => app.startup_in_progress = true,
                Err(error) => app.critical_error = Some(error),
            }
        }

        if let Some(error) = app.critical_error.as_ref() {
            ui.add_space(16.0);
            ui.separator();
            egui::ScrollArea::new([false, true]).max_height(160.0).show(ui, |ui| {
                ui.label(RichText::new(error).color(style::DANGER).size(14.0));
            });
            ui.separator();
        }
    });
}
