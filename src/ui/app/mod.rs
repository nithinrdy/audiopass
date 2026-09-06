use std::{path::PathBuf, sync::mpsc};

use eframe::egui;

mod components;
mod screens;
use screens::MainScreen;
mod types;
use types::state;

use crate::backend::{
    datastore,
    pipewire::{PipewireAppSource, PipewireEvent, PipewireHook},
    // playback,
};

pub struct App {
    startup_complete: bool,
    startup_in_progress: bool,
    active_screen: Option<screens::MainScreen>,
    pipewire_instance: PipewireHook,

    playback_mode: state::PlaybackMode,
    console_state: state::ConsoleState,

    critical_error: Option<String>,
    datastore: datastore::DatastoreManager,
    track_picker_receiver: Option<mpsc::Receiver<Option<Vec<PathBuf>>>>,
    // playback_controller: playback::PlaybackController, // TODO-file-playback
}

impl App {
    pub fn new(
        pw_instance: PipewireHook,
        // playback_controller: playback::PlaybackController
    ) -> Self {
        Self {
            startup_complete: false,
            startup_in_progress: false,
            active_screen: None,
            pipewire_instance: pw_instance,
            console_state: state::ConsoleState::default(),
            playback_mode: state::PlaybackMode::None,
            critical_error: None,
            datastore: datastore::DatastoreManager::new(),
            track_picker_receiver: None,
            // playback_controller: playback_controller,
        }
    }
}

impl App {
    pub fn set_playback_mode(&mut self, mode: state::PlaybackMode) {
        if self.playback_mode == mode {
            return;
        }

        self.pipewire_instance.drop_virtual_capture();

        match mode {
            state::PlaybackMode::None => {}
            state::PlaybackMode::PhysicalMic => {
                if let Some(selected_source) = self.console_state.selected_physical_source_id {
                    if let Some(s) = self.console_state.physical_sources.iter().find(|s| s.id == selected_source) {
                        self.pipewire_instance.create_virtual_capture(s.node_name.clone());
                    }
                }
            }
            state::PlaybackMode::ApplicationAudio => {
                if let Some(selected_source) = self.console_state.selected_app_source_id {
                    if let Some(s) = self.console_state.app_sources.iter().find(|s| s.id == selected_source) {
                        self.pipewire_instance.create_virtual_capture(s.node_name.clone());
                    }
                }
            }
            state::PlaybackMode::LocalFile => {}
        }
        self.playback_mode = mode;
    }
}

impl App {
    fn process_pipewire_events(&mut self) {
        loop {
            let event = match self.pipewire_instance.event_receiver.try_recv() {
                Ok(event) => event,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.startup_in_progress = false;
                    self.critical_error.get_or_insert_with(|| "Error: Pipewire worker disconnected".to_string());
                    break;
                }
            };

            match event {
                PipewireEvent::PhysicalSources { sources } => {
                    self.console_state.physical_sources = sources.into_iter().filter(|s| !s.is_audiopass_mic).collect();
                    if self.console_state.selected_physical_source_id.is_some()
                        && self
                            .console_state
                            .physical_sources
                            .iter()
                            .find(|s| s.id == self.console_state.selected_physical_source_id.unwrap())
                            .is_none()
                    // if new list of sources doesn't contain the selected physical source id
                    {
                        self.console_state.selected_physical_source_id = None;
                        if self.playback_mode == state::PlaybackMode::PhysicalMic {
                            self.pipewire_instance.drop_virtual_capture();
                        }
                    }
                }

                PipewireEvent::AppSources { sources } => {
                    self.console_state.app_sources = sources.keys().map(|id| sources[id].clone()).collect::<Vec<PipewireAppSource>>();
                    if self.console_state.selected_app_source_id.is_some() && self.console_state.app_sources.iter().find(|s| s.id == self.console_state.selected_app_source_id.unwrap()).is_none()
                    // if new list of sources doesn't contain the selected app source id
                    {
                        self.console_state.selected_app_source_id = None;
                        if self.playback_mode == state::PlaybackMode::ApplicationAudio {
                            self.pipewire_instance.drop_virtual_capture();
                        }
                    }
                }

                PipewireEvent::VirtualCaptureReady { state: Ok(()) } => {}

                PipewireEvent::VirtualCaptureReady { state: Err(err) } => {
                    self.critical_error = Some(format!("Failed to create virtual capture: {err}"));
                }

                PipewireEvent::VirtualMicReady { state: Ok(()) } => {
                    if self.startup_in_progress {
                        self.startup_in_progress = false;
                        self.startup_complete = true;
                        self.active_screen = Some(MainScreen::Console);
                    }
                }

                PipewireEvent::VirtualMicReady { state: Err(error) } => {
                    self.startup_in_progress = false;
                    self.critical_error = Some(format!("Failed to create virtual microphone: {error}"));
                }

                PipewireEvent::PipewireError { error } => {
                    self.startup_in_progress = false;
                    self.critical_error = Some(error);
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.process_pipewire_events();

        ui.visuals_mut().panel_fill = crate::ui::style::BACKGROUND;

        if !self.startup_complete {
            egui::CentralPanel::default().show(ui, |ui| {
                screens::startup::show(self, ui);
            });
        } else {
            egui::Panel::left("sidebar").exact_size(200.0).resizable(false).show(ui, |ui| components::sidebar::show(self, ui));

            egui::CentralPanel::default_margins().show(ui, |ui| {
                // so rustfmt doesnt inline this part
                match self.active_screen {
                    Some(MainScreen::Console) => screens::console::show(self, ui),
                    Some(MainScreen::Help) => screens::help::show(self, ui),
                    Some(MainScreen::About) => screens::about::show(self, ui),
                    None => {}
                }
            });
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.pipewire_instance.shutdown();
    }
}
