use crate::ui::{App, app::screens::MainScreen, style};
use eframe::egui::{self, Button, Color32, CornerRadius, RichText};

const TABS: [&str; 3] = ["Console", "Help", "About"];

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    ui.vertical_centered_justified(|ui| {
        ui.add_space(8.0);

        for tab in TABS {
            let screen_for_tab = match tab {
                "Console" => MainScreen::Console,
                "Help" => MainScreen::Help,
                "About" => MainScreen::About,
                _ => unreachable!("unknown sidebar tab: {tab}"),
            };

            let icon_for_tab = match tab {
                "Console" => egui_phosphor::regular::SLIDERS,
                "Help" => egui_phosphor::regular::QUESTION,
                "About" => egui_phosphor::regular::FILES,
                _ => unreachable!("unknown sidebar tab: {tab}"),
            };

            let is_active_tab = app.active_screen == Some(screen_for_tab);
            let text_color = if is_active_tab { style::CONTRAST_TEXT } else { style::PRIMARY_TEXT };
            let bg_color = if is_active_tab { style::ACCENT } else { style::SECONDARY_BACKGROUND };

            let clicked = ui
                .scope(|ui| {
                    ui.spacing_mut().button_padding = egui::Vec2 { x: 12.0, y: 0.0 };
                    ui.add(
                        Button::new(RichText::new(tab).color(text_color).size(16.0))
                            .left_text(RichText::new(icon_for_tab).color(text_color).size(24.0))
                            .right_text(RichText::new(icon_for_tab).color(Color32::TRANSPARENT).size(24.0))
                            .fill(bg_color)
                            .min_size(egui::Vec2 { x: 20.0, y: 40.0 }),
                    )
                    .clicked()
                })
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

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.set_height(200.0); // setting max_height() on ScrollArea doesn't work? so limiting the height of the entire container instead
            ui.add_space(12.0);
            ui.label(RichText::new("AudioPass").color(style::ACCENT).size(32.0).extra_letter_spacing(2.0).strong());
            ui.add_space(8.0);
            ui.add(egui::Separator::default().shrink(0.0));

            if let Some(error_msg) = app.critical_error.as_ref() {
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    egui::ScrollArea::new([false, true]).content_margin(4).show(ui, |ui| {
                        ui.label(RichText::new(error_msg).color(style::DANGER).size(14.0));
                    });
                });
                ui.separator();
                if ui
                    .add(
                        egui::Button::new(RichText::new("Dismiss error").size(16.0))
                            .small()
                            .fill(style::SECONDARY_BACKGROUND)
                            .corner_radius(CornerRadius { ne: 2, nw: 0, se: 0, sw: 0 })
                            .stroke(egui::Stroke::NONE),
                    )
                    .clicked()
                {
                    app.critical_error = None;
                }
            }
        });
    });
}
