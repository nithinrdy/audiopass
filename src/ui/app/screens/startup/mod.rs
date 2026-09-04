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
            .add(
                Button::new(RichText::new("Continue").color(style::CONTRAST_TEXT).size(18.0))
                    .fill(style::ACCENT)
                    .min_size(egui::Vec2 { x: 20.0, y: 40.0 }),
            )
            .clicked()
        {
            app.pipewire_instance.create_virtual_mic()
        }
    });
}
