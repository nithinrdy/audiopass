use eframe::egui::{self};
mod mic_picker;
mod track_list;

use crate::ui::{self};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    mic_picker::show(app, ui);
    ui.add_space(20.0);
    track_list::show(app, ui);
}
