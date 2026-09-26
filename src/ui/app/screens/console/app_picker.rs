use crate::ui::{self, style};
use eframe::egui::{self, Align, FontId, RichText, Stroke, TextFormat};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    let mut selected_node = None;

    if app.console_state.app_sources.len() == 0 {
        egui::Frame::new().inner_margin(50.0).show(ui, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.label(RichText::new("No audio-playing applications were detected.").color(style::WARNING).size(20.0));
                ui.add_space(8.0);
                ui.label(
                    RichText::new(
                        "If AudioPass is failing to detect an application, try playing audio from it for a moment to let PipeWire create a node for the app. \
                        This will let AudioPass detect the application.",
                    )
                    .color(style::SECONDARY_TEXT)
                    .size(16.0),
                );
            });
        });
        return;
    }

    ui.horizontal(|ui| {
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
        ui.label(RichText::new("Selected Application").size(16.0));
    });

    ui.add_space(2.0);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            let selected_app_source = app.console_state.app_sources.iter().find(|s| Some(s.id) == app.console_state.selected_app_source_id);
            let mut selected_app_label = egui::text::LayoutJob::default();
            if let Some(app) = selected_app_source {
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
                if let Some(n) = app.info__props__media_name.as_ref() {
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
            };

            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 40.0;
                ui.spacing_mut().button_padding.x = 12.0;
                ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
                ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

                egui::ComboBox::from_id_salt(&("audio_playing_app_picker".to_owned() + &app.console_state.app_sources.len().to_string()))
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

                        for s in &app.console_state.app_sources {
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
                            if let Some(n) = s.info__props__media_name.as_ref() {
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

                            if ui.selectable_value(&mut app.console_state.selected_app_source_id, Some(s.id), option_label).changed() {
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

    ui.add_space(12.0);
    ui.label(RichText::new(format!("Gain [{:.2}x]", app.console_state.app_audio_gain as f32 / 100.0)).size(16.0));
    let gain_changed = ui
        .scope(|ui| {
            ui.spacing_mut().slider_width = ui.available_width();
            ui.style_mut().visuals.selection.bg_fill = style::ACCENT;
            ui.add(
                egui::Slider::new(&mut app.console_state.app_audio_gain, 0..=200)
                    .show_value(false)
                    .handle_shape(egui::style::HandleShape::Circle)
                    .trailing_fill(true),
            )
            .changed()
        })
        .inner;
    if gain_changed {
        app.pipewire_instance.set_gain(app.console_state.app_audio_gain);
    }

    ui.add_space(4.0);
}
