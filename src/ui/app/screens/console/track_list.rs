use eframe::egui::{self, Label, Layout, Margin, RichText, Stroke};

use crate::ui::{self, style};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    let mut track_list = &mut app.track_list;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Track List").size(16.0));
        ui.add_space(ui.available_width() - 86.0);
        if ui.small_button(RichText::new("+ Add tracks").size(14.0)).clicked() {
            // file picker open
        }
    });

    ui.add_space(4.0);

    ui.vertical(|ui| {
        ui.scope(|ui| {
            ui.set_max_height(400.0);
            ui.spacing_mut().interact_size.y = 40.0;
            ui.spacing_mut().button_padding.x = 12.0;
            ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, style::SECONDARY_TEXT);
            ui.visuals_mut().widgets.hovered.weak_bg_fill = style::SECONDARY_BACKGROUND;

            for (i, track) in track_list.iter().enumerate() {
                let track_name = track.file_name().unwrap();
                let track_parent_dir = track.parent().unwrap();

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
                            let remove_button_width = 82.0;
                            let spacing = ui.spacing().item_spacing.x;

                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("Play").size(16.0).color(style::ACCENT)).clicked() {
                                    // play
                                }
                                ui.vertical(|ui| {
                                    ui.set_width(ui.available_width() - remove_button_width - spacing);
                                    ui.add_space(2.0);
                                    ui.add(Label::new(RichText::new(format!("{}. {}", i + 1, (track_name.to_string_lossy()))).size(16.0).color(style::PRIMARY_TEXT)).truncate());
                                    ui.add_space(-1.0);
                                    ui.add(Label::new(RichText::new(track_parent_dir.to_string_lossy()).size(12.0).color(style::SECONDARY_TEXT)).truncate());
                                });
                                if ui.button(RichText::new("Remove").size(16.0).color(style::DANGER)).clicked() {
                                    // remove from list
                                }
                                ui.add_space(-9.0);
                            })
                        });
                });
            }
        });
    });
}
