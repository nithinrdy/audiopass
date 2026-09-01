use crate::ui::{self, app::components::button, style};
use eframe::egui::{self, RichText, Stroke};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.set_width(400.0);
        ui.add_space(8.0);
        ui.label(RichText::new("AudioPass").color(style::ACCENT).size(32.0).extra_letter_spacing(2.0).strong());

        ui.add_space(40.0);
        if app.console_state.physical_sources.len() < 1 {
            ui.label(
                RichText::new("No physical mics or input sources detected. To proceed, please connect a physical input source and ensure Pipewire can detect it.")
                    .size(16.0)
                    .color(style::WARNING),
            );
            ui.add_space(20.0);

            return;
        }

        ui.label(
            RichText::new("To continue, please select a physical mic from the list below.")
                .size(16.0)
                .extra_letter_spacing(-0.5)
                .color(style::SECONDARY_TEXT),
        );

        let mut allow_continue = false;

        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let selected_mic = app.console_state.physical_sources.iter().find(|s| Some(s.id) == app.console_state.selected_physical_source_id);
                let selected_mic_label = match selected_mic {
                    Some(m) => {
                        allow_continue = true;
                        m.description.clone()
                    }
                    _ => {
                        allow_continue = false;
                        "-".to_string()
                    }
                };
                ui.scope(|ui| {
                    ui.spacing_mut().interact_size.y = 40.0;
                    ui.spacing_mut().button_padding.x = 12.0;
                    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
                    ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

                    egui::ComboBox::from_id_salt("startup_mic_picker")
                        .selected_text(RichText::new(selected_mic_label).size(16.0))
                        .width(ui.available_width())
                        .truncate()
                        .show_ui(ui, |ui| {
                            ui.spacing_mut().interact_size.y = 40.0;
                            ui.spacing_mut().button_padding.x = 12.0;
                            ui.visuals_mut().selection.bg_fill = style::SECONDARY_BACKGROUND;
                            ui.visuals_mut().selection.stroke = Stroke::new(1.0, style::PRIMARY_TEXT);
                            ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, style::ACCENT);

                            for s in &app.console_state.physical_sources {
                                if ui
                                    .selectable_value(&mut app.console_state.selected_physical_source_id, Some(s.id), RichText::new(&s.description).size(16.0))
                                    .changed()
                                {
                                    app.pipewire_instance.create_virtual_capture(s.node_name.clone());
                                };
                            }
                        });
                });
            });
        });

        ui.add_space(10.0);
        ui.add_enabled_ui(allow_continue, |ui| {
            let continued = button::show(app, ui, "Continue", button::ButtonVariant::Primary).inner;
            if continued {
                app.pipewire_instance.create_virtual_mic()
            }
        });
    });
}
