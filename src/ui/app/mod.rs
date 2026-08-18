use eframe::egui;

mod components;
mod screens;
use screens::MainScreen;

use crate::ui::constants::IS_DEV;

pub struct App {
    startup_complete: bool,
    active_screen: Option<screens::MainScreen>,
}

impl App {
    pub fn new() -> Self {
        Self {
            startup_complete: IS_DEV,
            active_screen: if IS_DEV {
                Some(MainScreen::Console)
            } else {
                None
            },
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.visuals_mut().panel_fill = crate::ui::style::BACKGROUND;

        if !self.startup_complete {
            egui::CentralPanel::default().show(ui, |ui| {
                screens::startup::show(self, ui);
            });
        } else {
            egui::Panel::left("sidebar")
                .exact_size(200.0)
                .resizable(false)
                .show(ui, |ui| components::sidebar::show(self, ui));

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
