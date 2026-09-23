use crate::{
    backend::pipewire::{PipewireAppSource, PipewirePhysicalSource},
    utils::{LICENSE_TEXT, PRIVACY_TEXT, THIRD_PARTY_TEXT},
};

// console state stuff
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

pub struct ConsoleState {
    pub physical_sources: Vec<PipewirePhysicalSource>,
    pub selected_physical_source_id: Option<u32>,
    pub app_sources: Vec<PipewireAppSource>,
    pub selected_app_source_id: Option<u32>,
    pub app_audio_gain: u32,
}

impl Default for ConsoleState {
    fn default() -> Self {
        Self {
            physical_sources: Vec::new(),
            selected_physical_source_id: None,
            app_sources: Vec::new(),
            selected_app_source_id: None,
            app_audio_gain: 100,
        }
    }
}

// about state stuff
#[derive(PartialEq)]
pub enum LegalDoc {
    License,
    ThirdParty,
    Privacy,
}

impl LegalDoc {
    pub fn iterable() -> [LegalDoc; 3] {
        [Self::License, Self::ThirdParty, Self::Privacy]
    }

    pub fn get_label(&self) -> String {
        (match self {
            Self::License => "LICENSE",
            Self::ThirdParty => "THIRD PARTY NOTICES",
            Self::Privacy => "PRIVACY POLICY",
        })
        .to_string()
    }

    pub fn get_content(&self) -> &'static str {
        match self {
            Self::License => LICENSE_TEXT,
            Self::ThirdParty => THIRD_PARTY_TEXT,
            Self::Privacy => PRIVACY_TEXT,
        }
    }
}

pub struct AboutState {
    pub selected_doc: LegalDoc,
}

impl Default for AboutState {
    fn default() -> Self {
        Self { selected_doc: LegalDoc::License }
    }
}

// help state stuff
#[derive(PartialEq)]
pub enum HelpCategory {
    General,
    PhysicalMic,
    ApplicationAudio,
    LocalFile,
}

impl HelpCategory {
    pub fn iterable() -> [Self; 4] {
        [Self::General, Self::PhysicalMic, Self::ApplicationAudio, Self::LocalFile]
    }
}

pub struct HelpState {
    pub selected_category: HelpCategory,
}

impl Default for HelpState {
    fn default() -> Self {
        Self {
            selected_category: HelpCategory::General,
        }
    }
}
