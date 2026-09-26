use eframe::egui::{self, Button, RichText};

use crate::ui::{self, app::types::state::LegalDoc, style};

const EXTERNAL_LINKS: [(&str, &str, &str); 2] = [
    ("Homepage", "https://audiopass.nithinrdy.com", egui_phosphor::regular::GLOBE),
    ("Check out AudioPass on", "https://github.com/nithinrdy/audiopass", egui_phosphor::regular::GITHUB_LOGO),
];

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.label(RichText::new(format!("AudioPass {}", env!("CARGO_PKG_VERSION"))).color(style::PRIMARY_TEXT).size(16.0));
    ui.label(RichText::new("Copyright © 2026 Vishnu Nithin Reddy, MIT licensed.").color(style::SECONDARY_TEXT).size(14.0));

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let button_width = (ui.available_width() / 2.0) - 5.0;

        for (label, url, icon) in EXTERNAL_LINKS {
            if ui
                .add(
                    Button::new(RichText::new(format!("{}  {}", label, icon)).color(style::PRIMARY_TEXT).size(16.0))
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
