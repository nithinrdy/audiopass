use eframe::egui::{self, RichText, Stroke};

use crate::ui::{self, style};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    let mut selected_node = None;

    if app.console_state.physical_sources.len() == 0 {
        egui::Frame::new().inner_margin(50.0).show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(RichText::new("No physical microphones were detected.").color(style::WARNING).size(20.0));
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Try reconnecting your mic(s) and ensure it/they can be seen in the system sound settings.")
                        .color(style::SECONDARY_TEXT)
                        .size(16.0),
                );
            });
        });
        return;
    }

    ui.label(RichText::new("Selected Physical Mic").size(16.0));
    ui.add_space(2.0);
    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let selected_mic = app.console_state.physical_sources.iter().find(|s| Some(s.id) == app.console_state.selected_physical_source_id);
            let selected_mic_label = match selected_mic {
                Some(m) => m.description.clone(),
                _ => "".to_string(),
            };

            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 40.0;
                ui.spacing_mut().button_padding.x = 12.0;
                ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
                ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

                egui::ComboBox::from_id_salt("mic_picker")
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
                                .selectable_value(
                                    &mut app.console_state.selected_physical_source_id,
                                    Some(s.id),
                                    RichText::new(&s.description).color(style::PRIMARY_TEXT).size(16.0),
                                )
                                .changed()
                            {
                                selected_node = Some(s.node_name.clone());
                            }
                        }
                    });
            });
        });
    });
    if let Some(node_name) = selected_node {
        app.create_virtual_capture(node_name);
    }
}
