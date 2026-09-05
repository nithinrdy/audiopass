use eframe::egui::{self, Button, Margin, RichText};
mod app_picker;
mod mic_picker;
mod track_list;

use crate::ui::{self, app::types::state, style};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.label(RichText::new("Mode").size(16.0));
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        let button_width = ui.available_width() / state::PlaybackMode::iterable().len() as f32 - 6.0;

        for mode in state::PlaybackMode::iterable() {
            if ui
                .add_enabled(
                    mode != state::PlaybackMode::LocalFile,
                    Button::new(
                        RichText::new(match mode {
                            state::PlaybackMode::None => "None",
                            state::PlaybackMode::PhysicalMic => "Physical Mic",
                            state::PlaybackMode::ApplicationAudio => "Application Audio",
                            state::PlaybackMode::LocalFile => "Local File",
                        })
                        .color(if app.playback_mode == mode { style::CONTRAST_TEXT } else { style::PRIMARY_TEXT })
                        .size(14.0),
                    )
                    .fill(if app.playback_mode == mode { style::ACCENT } else { style::SECONDARY_BACKGROUND })
                    .min_size(egui::Vec2 { x: button_width, y: 40.0 }),
                )
                .on_disabled_hover_text(
                    RichText::new("Local file playback is currently a work-in-progress, coming soon!")
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
        state::PlaybackMode::None => {}
        state::PlaybackMode::PhysicalMic => {
            mic_picker::show(app, ui);
            ui.add_space(4.0);
            ui.separator();
        }
        state::PlaybackMode::ApplicationAudio => {
            app_picker::show(app, ui);
            ui.add_space(4.0);
            ui.separator();
        }
        state::PlaybackMode::LocalFile => {
            // track_list::show(app, ui); // TODO-file-playback: revisit and finish later
        }
    }

    egui::Frame::default().inner_margin(Margin::same(50)).show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| match app.playback_mode {
            state::PlaybackMode::None => {
                ui.label(
                    RichText::new("AudioPass is currently not playing audio through the virtual microphone.")
                        .color(style::SECONDARY_TEXT)
                        .size(20.0),
                );
            }
            state::PlaybackMode::PhysicalMic => {
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
            state::PlaybackMode::ApplicationAudio => {
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
            state::PlaybackMode::LocalFile => {}
        });
    });
}
