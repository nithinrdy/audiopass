use crate::ui::{self, style};
use eframe::egui::{self, Label, Layout, Margin, RichText, Stroke};
use std::sync::mpsc;
use uuid::Uuid;

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    process_picker_result(app);
    let file_picker_open = app.track_picker_receiver.is_some();

    ui.horizontal(|ui| {
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
        ui.label(RichText::new("Track List").size(16.0));

        if ui
            .add_enabled_ui(!file_picker_open, |ui| {
                ui.small_button(RichText::new(format!("{} Add tracks", egui_phosphor::regular::PLUS)).size(16.0))
            })
            .inner
            .clicked()
        {
            open_track_picker(app, ui.ctx().clone());
        }
    });

    ui.add_space(8.0);

    if app.datastore.tracks().get().len() == 0 {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("No tracks yet.").size(20.0).color(style::SECONDARY_TEXT));
        });
        ui.add_space(10.0);
        return;
    }

    ui.vertical(|ui| {
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 40.0;
            ui.spacing_mut().button_padding.x = 12.0;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

            let mut track_id_to_remove = None as Option<Uuid>;

            egui::ScrollArea::vertical()
                .content_margin(Margin {
                    left: 20,
                    top: 0,
                    bottom: 0,
                    right: 20,
                })
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .max_height(400.0)
                .show(ui, |ui| {
                    for (i, track) in app.datastore.tracks().get().iter().enumerate() {
                        let track_name = track.path.file_name().unwrap();
                        let track_parent_dir = track.path.parent().unwrap();

                        ui.with_layout(Layout::left_to_right(egui::Align::TOP), |ui| {
                            egui::Frame::default()
                                .stroke(egui::Stroke::new(1.0, style::SECONDARY_TEXT))
                                .fill(style::SECONDARY_BACKGROUND)
                                .corner_radius(4.0)
                                .inner_margin(Margin {
                                    left: -1,
                                    right: -1,
                                    top: -1,
                                    bottom: -1,
                                })
                                .outer_margin(Margin { left: 0, right: 0, top: 0, bottom: 2 })
                                .show(ui, |ui| {
                                    let remove_button_width = 40.0;
                                    let spacing = ui.spacing().item_spacing.x;

                                    ui.horizontal(|ui| {
                                        ui.add_enabled_ui(
                                            match app.playback_controller.get_active_track() {
                                                Some(t) => t.id != track.id,
                                                None => true,
                                            },
                                            |ui| {
                                                if ui.button(egui::RichText::new(egui_phosphor::regular::PLAY).size(16.0).color(style::ACCENT)).clicked() {
                                                    app.playback_controller.reset_and_new(track.clone());
                                                    app.playback_controller.start_or_resume();
                                                }
                                            },
                                        );
                                        ui.add_space(-9.0);

                                        egui::Frame::default().inner_margin(2).stroke(egui::Stroke::new(1.0, style::SECONDARY_TEXT)).show(ui, |ui| {
                                            ui.vertical(|ui| {
                                                ui.add_space(1.0);
                                                ui.label(RichText::new(i.to_string()).size(16.0));
                                            });
                                        });

                                        ui.vertical(|ui| {
                                            ui.set_width(ui.available_width() - remove_button_width - spacing);
                                            ui.add_space(2.0);
                                            ui.add(Label::new(RichText::new(track_name.to_string_lossy()).size(16.0).color(style::PRIMARY_TEXT)).truncate());
                                            ui.add_space(-1.0);
                                            ui.add(Label::new(RichText::new(track_parent_dir.to_string_lossy()).size(12.0).color(style::SECONDARY_TEXT)).truncate());
                                        });

                                        if ui.button(egui::RichText::new(egui_phosphor::regular::TRASH).size(16.0).color(style::DANGER)).clicked() {
                                            track_id_to_remove = Some(track.id);
                                        }
                                        ui.add_space(-9.0);
                                    })
                                });
                        });
                    }
                });

            if let Some(uuid) = track_id_to_remove {
                match app.playback_controller.get_active_track() {
                    Some(t) => {
                        if t.id == uuid {
                            app.playback_controller.reset();
                        }
                    }
                    None => {}
                }
                match app.datastore.tracks().remove(uuid) {
                    Ok(_) => {}
                    Err(e) => app.critical_error = Some(e),
                }
            }
        });
    });
}

fn open_track_picker(app: &mut ui::App, context: egui::Context) {
    let (files_sender, files_receiver) = mpsc::channel::<Option<Vec<std::path::PathBuf>>>();
    app.track_picker_receiver = Some(files_receiver);

    std::thread::spawn(move || {
        let selected_tracks = rfd::FileDialog::new().set_title("Add tracks").pick_files(); // TODO: add extension filter
        let _ = files_sender.send(selected_tracks);
        context.request_repaint();
    });
}

fn process_picker_result(app: &mut ui::App) {
    if let Some(receiver) = app.track_picker_receiver.as_ref() {
        let received = receiver.try_recv();

        match received {
            Ok(optional_pathlist) => match optional_pathlist {
                Some(pathlist) => match app.datastore.tracks().add(pathlist) {
                    Ok(_) => {}
                    Err(e) => app.critical_error = Some(e),
                },
                None => app.track_picker_receiver = None,
            },
            Err(e) => match e {
                mpsc::TryRecvError::Empty => {}
                mpsc::TryRecvError::Disconnected => app.track_picker_receiver = None,
            },
        }
    }
}
