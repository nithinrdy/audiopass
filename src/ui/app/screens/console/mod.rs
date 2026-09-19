use eframe::egui::{self, Button, Margin, RichText};
mod app_picker;
mod mic_picker;
mod track_list;

use crate::ui::{self, app::types::state::PlaybackMode, style};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.label(RichText::new("Mode").size(16.0));
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        let button_width = ui.available_width() / PlaybackMode::iterable().len() as f32 - 6.0;

        for mode in PlaybackMode::iterable() {
            let icon_for_mode = match mode {
                PlaybackMode::None => egui_phosphor::regular::SPEAKER_SIMPLE_SLASH,
                PlaybackMode::PhysicalMic => egui_phosphor::regular::MICROPHONE,
                PlaybackMode::ApplicationAudio => egui_phosphor::regular::MUSIC_NOTES_SIMPLE,
                PlaybackMode::LocalFile => egui_phosphor::regular::FILE_AUDIO,
            };
            let text_for_mode = match mode {
                PlaybackMode::None => "None",
                PlaybackMode::PhysicalMic => "Physical Mic",
                PlaybackMode::ApplicationAudio => "App Audio",
                PlaybackMode::LocalFile => "Local File",
            };

            if ui
                .add_enabled(
                    app.worker_healthy && mode != PlaybackMode::LocalFile,
                    Button::new(
                        RichText::new(format!("{}  {}", icon_for_mode, text_for_mode))
                            .color(if app.playback_mode == mode { style::CONTRAST_TEXT } else { style::PRIMARY_TEXT })
                            .size(16.0),
                    )
                    .fill(if app.playback_mode == mode { style::ACCENT } else { style::SECONDARY_BACKGROUND })
                    .min_size(egui::Vec2 { x: button_width, y: 40.0 }),
                )
                .on_hover_text(
                    RichText::new(match mode {
                        PlaybackMode::None => "Transmit nothing (silence) through the virtual microphone.",
                        PlaybackMode::PhysicalMic => "Route audio from the selected physical microphone into the virtual mic.",
                        PlaybackMode::ApplicationAudio => "Route the audio output of the selected application into the virtual mic.",
                        PlaybackMode::LocalFile => "Route the content of selected local audio files into the virtual mic.",
                    })
                    .color(style::SECONDARY_TEXT)
                    .size(14.0),
                )
                .on_disabled_hover_text(
                    RichText::new(if !app.worker_healthy {
                        "Audio routing is unavailable. Please try restarting AudioPass."
                    } else {
                        "Local file playback is currently a work-in-progress, coming soon!"
                    })
                    .color(style::SECONDARY_TEXT)
                    .size(14.0),
                )
                .clicked()
            {
                app.set_playback_mode(mode);
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(12.0);

    match app.playback_mode {
        PlaybackMode::None => {}
        PlaybackMode::PhysicalMic => {
            mic_picker::show(app, ui);
            ui.add_space(4.0);
            ui.separator();
        }
        PlaybackMode::ApplicationAudio => {
            app_picker::show(app, ui);
            ui.add_space(4.0);
            ui.separator();
        }
        PlaybackMode::LocalFile => {
            // track_list::show(app, ui); // TODO-file-playback: revisit and finish later
        }
    }

    egui::Frame::default().inner_margin(Margin::same(50)).show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| match app.playback_mode {
            PlaybackMode::None => {
                ui.label(
                    RichText::new("AudioPass is currently not playing audio through the virtual microphone.")
                        .color(style::SECONDARY_TEXT)
                        .size(20.0),
                );
            }
            PlaybackMode::PhysicalMic => {
                if app.console_state.selected_physical_source_id.is_some() {
                    ui.label(
                        RichText::new("AudioPass is playing input from the selected physical mic through the virtual microphone.")
                            .color(style::SECONDARY_TEXT)
                            .size(20.0),
                    );
                } else {
                    ui.label(
                        RichText::new("AudioPass is currently not playing audio through the virtual microphone.")
                            .color(style::SECONDARY_TEXT)
                            .size(20.0),
                    );

                    ui.add_space(20.0);

                    ui.label(
                        RichText::new("Select a physical mic from the list above to play its input through the virtual microphone.")
                            .color(style::ACCENT)
                            .size(16.0),
                    );
                }
            }
            PlaybackMode::ApplicationAudio => {
                if app.console_state.selected_app_source_id.is_some() {
                    ui.label(
                        RichText::new("AudioPass is playing audio from the selected application through the virtual microphone.")
                            .color(style::SECONDARY_TEXT)
                            .size(20.0),
                    );
                } else {
                    ui.label(
                        RichText::new("AudioPass is currently not playing audio through the virtual microphone.")
                            .color(style::SECONDARY_TEXT)
                            .size(20.0),
                    );

                    ui.add_space(20.0);

                    ui.label(
                        RichText::new("Select an application from the list above to play its audio through the virtual microphone.")
                            .color(style::ACCENT)
                            .size(16.0),
                    );
                }
            }
            PlaybackMode::LocalFile => {}
        });
    });
}
