use eframe::egui;
mod app_picker;
mod mic_picker;
mod track_list;

use crate::ui;

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    mic_picker::show(app, ui);
    ui.add_space(20.0);
    // track_list::show(app, ui); // TODO-file-playback: revisit and finish later
    app_picker::show(app, ui);
}
