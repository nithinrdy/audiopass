use crate::{
    backend::pipewire::{PipewireAppSource, PipewirePhysicalSource},
    ui::app::screens::about::LegalDoc,
};

#[derive(PartialEq)]
pub enum PlaybackMode {
    None,
    PhysicalMic,
    ApplicationAudio,
    LocalFile, // TODO-file-playback
}

impl PlaybackMode {
    pub fn iterable() -> [Self; 4] {
        [Self::None, Self::PhysicalMic, Self::ApplicationAudio, Self::LocalFile]
    }
}

#[derive(Default)]
pub struct ConsoleState {
    pub physical_sources: Vec<PipewirePhysicalSource>,
    pub selected_physical_source_id: Option<u32>,
    pub app_sources: Vec<PipewireAppSource>,
    pub selected_app_source_id: Option<u32>,
}

pub struct AboutState {
    pub selected_doc: LegalDoc,
}

impl Default for AboutState {
    fn default() -> Self {
        AboutState { selected_doc: LegalDoc::License }
    }
}
