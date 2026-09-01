use eframe::egui;
mod app_picker;
mod mic_picker;
mod track_list;

use crate::{
    backend::pipewire::{PipewireAppSource, PipewirePhysicalSource},
    ui,
};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    mic_picker::show(app, ui);
    ui.add_space(20.0);
    // track_list::show(app, ui); // TODO-file-playback: revisit and finish later
    app_picker::show(app, ui);
}

#[derive(Default)]
pub struct ConsoleState {
    pub physical_sources: Vec<PipewirePhysicalSource>,
    pub selected_physical_source_id: Option<u32>,
    pub app_sources: Vec<PipewireAppSource>,
    pub selected_app_source_id: Option<u32>,
}
