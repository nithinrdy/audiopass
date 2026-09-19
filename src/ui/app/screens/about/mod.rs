use eframe::egui::{self, Button, RichText};

use crate::ui::{self, style};
use crate::utils::{LICENSE_TEXT, PRIVACY_TEXT, THIRD_PARTY_TEXT};

const EXTERNAL_LINKS: [(&str, &str); 2] = [("Homepage", "https://audiopass.nithinrdy.com"), ("Source Code", "https://github.com/nithinrdy/audiopass")];

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

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.label(RichText::new(format!("AudioPass {}", env!("CARGO_PKG_VERSION"))).color(style::PRIMARY_TEXT).size(16.0));
    ui.label(RichText::new("Copyright © 2026 Vishnu Nithin Reddy, MIT licensed.").color(style::SECONDARY_TEXT).size(14.0));

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let button_width = (ui.available_width() / 2.0) - 5.0;

        for (label, url) in EXTERNAL_LINKS {
            if ui
                .add(
                    Button::new(RichText::new(format!("{}  {}", label, egui_phosphor::regular::ARROW_SQUARE_OUT)).color(style::PRIMARY_TEXT).size(16.0))
                        .fill(style::SECONDARY_BACKGROUND)
                        .min_size(egui::vec2(button_width, 40.0)),
                )
                .on_hover_text(url)
                .clicked()
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let button_width = (ui.available_width() / 3.0) - 6.0;

        for doc in LegalDoc::iterable() {
            let is_selected = app.about_state.selected_doc == doc;
            if ui
                .add(
                    Button::new(RichText::new(doc.get_label()).color(if is_selected { style::CONTRAST_TEXT } else { style::PRIMARY_TEXT }).size(16.0))
                        .fill(if is_selected { style::ACCENT } else { style::SECONDARY_BACKGROUND })
                        .min_size(egui::vec2(button_width, 40.0)),
                )
                .clicked()
            {
                app.about_state.selected_doc = doc;
            }
        }
    });

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .id_salt("about_doc")
        .auto_shrink([false, false])
        .max_height(ui.available_height())
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.about_state.selected_doc.get_content())
                    .interactive(false)
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Monospace),
            );
        });
}
