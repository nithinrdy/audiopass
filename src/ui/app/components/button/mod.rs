use eframe::egui::{self, RichText};

use crate::ui::{App, style};

pub enum ButtonVariant {
    Primary,
    Secondary,
}

pub fn show(
    _app: &mut App,
    ui: &mut egui::Ui,
    text: &str,
    variant: ButtonVariant,
) -> egui::InnerResponse<bool> {
    ui.scope(|ui| {
        let visuals = ui.visuals_mut();
        visuals.widgets.inactive.weak_bg_fill = match variant {
            ButtonVariant::Primary => style::ACCENT,
            ButtonVariant::Secondary => style::SECONDARY_BACKGROUND,
        };
        visuals.widgets.inactive.fg_stroke = match variant {
            ButtonVariant::Primary => egui::Stroke::new(1.0, style::CONTRAST_TEXT),
            ButtonVariant::Secondary => egui::Stroke::new(1.0, style::PRIMARY_TEXT),
        };

        visuals.widgets.hovered.weak_bg_fill = match variant {
            ButtonVariant::Primary => style::ACCENT,
            ButtonVariant::Secondary => style::SECONDARY_BACKGROUND,
        };
        visuals.widgets.hovered.fg_stroke = match variant {
            ButtonVariant::Primary => egui::Stroke::new(1.0, style::CONTRAST_TEXT),
            ButtonVariant::Secondary => egui::Stroke::new(1.0, style::PRIMARY_TEXT),
        };

        ui.add(egui::Button::new(RichText::new(text).size(16.0)).min_size(egui::vec2(0.0, 32.0)))
            .clicked()
    })
}
