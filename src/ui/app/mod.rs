use std::{path::PathBuf, sync::mpsc};

use eframe::egui;

mod components;
mod screens;
use screens::MainScreen;

use crate::{
    backend::{
        datastore,
        pipewire::{PipewireAppSource, PipewireEvent, PipewireHook, PipewirePhysicalSource},
        // playback,
    },
    ui::constants::IS_DEV,
};

pub struct App {
    startup_complete: bool,
    active_screen: Option<screens::MainScreen>,
    pipewire_instance: PipewireHook,

    physical_sources: Vec<PipewirePhysicalSource>,
    selected_physical_source_id: Option<u32>,
    app_sources: Vec<PipewireAppSource>,
    selected_app_source_id: Option<u32>,

    critical_error: Option<String>,
    datastore: datastore::DatastoreManager,
    track_picker_receiver: Option<mpsc::Receiver<Option<Vec<PathBuf>>>>,
    // playback_controller: playback::PlaybackController, // TODO-file-playback
}

impl App {
    pub fn new(
        mut pw_instance: PipewireHook,
        // playback_controller: playback::PlaybackController
    ) -> Self {
        if IS_DEV {
            pw_instance.create_virtual_mic();
        }

        Self {
            startup_complete: IS_DEV,
            active_screen: (if IS_DEV { Some(MainScreen::Console) } else { None }),
            pipewire_instance: pw_instance,
            physical_sources: Vec::new(),
            selected_physical_source_id: None,
            app_sources: Vec::new(),
            selected_app_source_id: None,
            critical_error: None,
            datastore: datastore::DatastoreManager::new(),
            track_picker_receiver: None,
            // playback_controller: playback_controller,
        }
    }
}

impl App {
    fn process_pipewire_events(&mut self) {
        loop {
            let event = match self.pipewire_instance.event_receiver.try_recv() {
                Ok(event) => event,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.critical_error = Some("Error: Pipewire worker disconnected".to_string());
                    break;
                }
            };

            match event {
                PipewireEvent::PhysicalSources { sources } => {
                    self.physical_sources = sources.into_iter().filter(|s| !s.is_audiopass_mic).collect();
                    if self.selected_physical_source_id.is_some() && self.physical_sources.iter().find(|s| s.id == self.selected_physical_source_id.unwrap()).is_none() {
                        self.selected_physical_source_id = None; // clear selected mic id if not in list of sources
                    }
                }

                PipewireEvent::AppSources { sources } => {
                    self.app_sources = sources.keys().map(|id| sources[id].clone()).collect::<Vec<PipewireAppSource>>();
                    if self.selected_app_source_id.is_some() && self.app_sources.iter().find(|s| s.id == self.selected_app_source_id.unwrap()).is_none() {
                        self.selected_app_source_id = None; // clear selected app source id if not in list
                    }
                }

                PipewireEvent::VirtualCaptureReady { state: Ok(()) } => {}

                PipewireEvent::VirtualCaptureReady { state: Err(err) } => {
                    self.critical_error = Some(format!("Failed to create virtual capture: {err}"));
                }

                PipewireEvent::VirtualMicReady { state: Ok(()) } => {
                    self.startup_complete = true;
                    self.active_screen = Some(MainScreen::Console);
                }

                PipewireEvent::VirtualMicReady { state: Err(error) } => {
                    self.critical_error = Some(format!("Failed to create virtual microphone: {error}"));
                }

                PipewireEvent::PipewireError { error } => {
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
