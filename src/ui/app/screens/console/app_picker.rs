use crate::ui::{self, style};
use eframe::egui::{self, Align, FontId, RichText, Stroke, TextFormat};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
        ui.label(RichText::new("Selected App Source").size(16.0));
    });

    ui.add_space(2.0);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let selected_app_source = app.app_sources.iter().find(|s| Some(s.id) == app.selected_app_source_id);
            let mut selected_app_label = egui::text::LayoutJob::default();
            match selected_app_source {
                Some(app) => {
                    selected_app_label.append(
                        if app.info__state {
                            egui_phosphor::regular::WAVEFORM
                        } else {
                            egui_phosphor::regular::WAVEFORM_SLASH
                        },
                        0.0,
                        TextFormat {
                            font_id: FontId::proportional(24.0),
                            color: if app.info__state { style::ACCENT } else { style::SECONDARY_TEXT },
                            valign: Align::Center,
                            ..Default::default()
                        },
                    );
                    selected_app_label.append(
                        &app.info__props__application_name,
                        12.0,
                        TextFormat {
                            font_id: FontId::proportional(16.0),
                            color: style::PRIMARY_TEXT,
                            valign: Align::Center,
                            ..Default::default()
                        },
                    );
                    match app.info__props__media_name.as_ref() {
                        Some(n) => {
                            if *n == app.info__props__application_name {
                                // Spotify (and maybe other apps) has media.name the same as application.name?
                            } else {
                                selected_app_label.append(
                                    &format!("({})", n),
                                    8.0,
                                    TextFormat {
                                        font_id: FontId::proportional(16.0),
                                        color: style::SECONDARY_TEXT,
                                        valign: Align::Center,
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                        None => {}
                    }
                }
                _ => {}
            };

            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 40.0;
                ui.spacing_mut().button_padding.x = 12.0;
                ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
                ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

                egui::ComboBox::from_id_salt(&("audio_playing_app_picker".to_owned() + &app.app_sources.len().to_string()))
                    .selected_text(selected_app_label)
                    .width(ui.available_width())
                    .truncate()
                    .show_ui(ui, |ui| {
                        ui.spacing_mut().interact_size.y = 40.0;
                        ui.spacing_mut().button_padding.x = 12.0;
                        ui.visuals_mut().selection.bg_fill = style::SECONDARY_BACKGROUND;
                        ui.visuals_mut().selection.stroke = Stroke::new(1.0, style::PRIMARY_TEXT);
                        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, style::ACCENT);
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                        for s in &app.app_sources {
                            let mut option_label = egui::text::LayoutJob::default();
                            option_label.append(
                                if s.info__state {
                                    egui_phosphor::regular::WAVEFORM
                                } else {
                                    egui_phosphor::regular::WAVEFORM_SLASH
                                },
                                0.0,
                                TextFormat {
                                    font_id: FontId::proportional(24.0),
                                    color: if s.info__state { style::ACCENT } else { style::SECONDARY_TEXT },
                                    valign: Align::Center,
                                    ..Default::default()
                                },
                            );
                            option_label.append(
                                &s.info__props__application_name,
                                12.0,
                                TextFormat {
                                    font_id: FontId::proportional(16.0),
                                    color: style::PRIMARY_TEXT,
                                    valign: Align::Center,
                                    ..Default::default()
                                },
                            );
                            match s.info__props__media_name.as_ref() {
                                Some(n) => {
                                    if *n == s.info__props__application_name {
                                        // Spotify (and maybe other apps) has media.name the same as application.name?
                                    } else {
                                        option_label.append(
                                            &format!("({})", n),
                                            8.0,
                                            TextFormat {
                                                font_id: FontId::proportional(16.0),
                                                color: style::SECONDARY_TEXT,
                                                valign: Align::Center,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                }
                                None => {}
                            }

                            if ui.selectable_value(&mut app.selected_app_source_id, Some(s.id), option_label).changed() {
                                app.pipewire_instance.create_virtual_capture(s.node_name.clone());
                            }
                        }
                    });
            });
        });
    });
}
