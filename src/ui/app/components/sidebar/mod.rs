use crate::ui::{
    App,
    app::{components::button, screens::MainScreen},
    style,
};
use eframe::egui::{self, RichText};

const TABS: [&str; 3] = ["Console", "Help", "About"];

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.add_space(8.0);
        ui.label(
            RichText::new("AudioPass")
                .color(style::ACCENT)
                .size(32.0)
                .extra_letter_spacing(2.0)
                .strong(),
        );

        ui.add_space(24.0);

        for tab in TABS {
            let screen_for_tab = match tab {
                "Console" => MainScreen::Console,
                "Help" => MainScreen::Help,
                "About" => MainScreen::About,
                _ => unreachable!("unknown sidebar tab: {tab}"),
            };

            let clicked = button::show(
                app,
                ui,
                tab,
                if app.active_screen == Some(screen_for_tab) {
                    button::ButtonVariant::Primary
                } else {
                    button::ButtonVariant::Secondary
                },
            )
            .inner;
            if clicked {
                match tab {
                    "Console" => app.active_screen = Some(MainScreen::Console),
                    "Help" => app.active_screen = Some(MainScreen::Help),
                    "About" => app.active_screen = Some(MainScreen::About),
                    _ => {}
                }
            }
        }
    });
}
