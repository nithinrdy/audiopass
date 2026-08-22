use std::sync::mpsc::TryRecvError;

use eframe::egui;

mod components;
mod screens;
use screens::MainScreen;

use crate::{
    backend::pipewire::{PipewireEvent, PipewireHook, PipewireSource},
    ui::constants::IS_DEV,
};

pub struct App {
    startup_complete: bool,
    active_screen: Option<screens::MainScreen>,
    selected_mic_id: Option<u32>,
    pipewire_instance: PipewireHook,
    critical_error: Option<String>,
    valid_sources: Vec<PipewireSource>,
}

impl App {
    pub fn new(pw_instance: PipewireHook) -> Self {
        Self {
            startup_complete: IS_DEV,
            active_screen: (if IS_DEV { Some(MainScreen::Console) } else { None }),
            selected_mic_id: None,
            pipewire_instance: pw_instance,
            critical_error: None,
            valid_sources: Vec::new(),
        }
    }
}

impl App {
    fn process_pipewire_events(&mut self) {
        loop {
            let event = match self.pipewire_instance.event_receiver.try_recv() {
                Ok(event) => event,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.critical_error = Some("Error: Pipewire worker disconnected".to_string());
                    break;
                }
            };

            match event {
                PipewireEvent::Sources { sources } => self.valid_sources = sources.into_iter().filter(|s| !s.is_audiopass_mic).collect(),

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
