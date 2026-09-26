use crate::ui::{self, app::types::state::HelpCategory, style};
use eframe::egui::{self, Button, RichText};

pub fn show(app: &mut ui::App, ui: &mut egui::Ui) {
    ui.add_space(2.0);
    ui.label(RichText::new("Category").size(16.0));
    ui.add_space(2.0);

    ui.horizontal(|ui| {
        let button_width = ui.available_width() / HelpCategory::iterable().len() as f32 - 6.0;

        for category in HelpCategory::iterable() {
            let icon_for_category = match category {
                HelpCategory::General => egui_phosphor::regular::INFO,
                HelpCategory::PhysicalMic => egui_phosphor::regular::MICROPHONE,
                HelpCategory::ApplicationAudio => egui_phosphor::regular::MUSIC_NOTES_SIMPLE,
                HelpCategory::LocalFile => egui_phosphor::regular::FILE_AUDIO,
            };
            let text_for_category = match category {
                HelpCategory::General => "General",
                HelpCategory::PhysicalMic => "Physical Mic",
                HelpCategory::ApplicationAudio => "App Audio",
                HelpCategory::LocalFile => "Local File",
            };

            if ui
                .add_enabled(
                    category != HelpCategory::LocalFile,
                    Button::new(
                        RichText::new(format!("{}  {}", icon_for_category, text_for_category))
                            .color(if app.help_state.selected_category == category {
                                style::CONTRAST_TEXT
                            } else {
                                style::PRIMARY_TEXT
                            })
                            .size(16.0),
                    )
                    .fill(if app.help_state.selected_category == category {
                        style::ACCENT
                    } else {
                        style::SECONDARY_BACKGROUND
                    })
                    .min_size(egui::Vec2 { x: button_width, y: 40.0 }),
                )
                .on_disabled_hover_text(
                    RichText::new("Local file playback is currently a work-in-progress, coming soon!")
                        .color(style::SECONDARY_TEXT)
                        .size(14.0),
                )
                .clicked()
            {
                app.help_state.selected_category = category;
            }
        }
    });
    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    match app.help_state.selected_category {
        HelpCategory::General => {
            egui::ScrollArea::vertical().id_salt("general_help").auto_shrink([false, false]).show(ui, |ui| {
                help_section(
                    ui,
                    "",
                    "AudioPass creates a virtual microphone when you click \"Continue\" on the startup screen. This microphone is destroyed when the app quits.\n\n\
                         While on playback mode \"None\", no audio is trasmitted through the virtual mic. \
                         To play audio through the virtual mic, switch to one of the other modes.",
                );
                help_section(
                    ui,
                    "HOW DO I USE THE VIRTUAL MICROPHONE?",
                    "You should see a new entry in the list of audio inputs in your system sound settings labeled \"AudioPass Virtual Microphone\".\n\n\
                        For apps that automatically listen to the default input source (most games do this), \
                        you must switch to the virtual mic as the active input source in your system sound settings.\n\n\
                        For apps that let you select a mic in their internal settings, select \"AudioPass Virtual Microphone\".",
                );
                help_section(
                    ui,
                    "HOW DO I TEST THE VIRTUAL MIC'S OUTPUT?",
                    "You should be able to use any recording software to play, record and examine some sample audio.\n\n\
                    I prefer using the Steam desktop app's built-in microphone test:\n\
                    1. Go to Steam > Settings > Voice and select \"AudioPass Virtual Microphone\" as the Voice Input Device.\n\
                    2. Turn off \"Echo cancellation\" and \"Noise cancellation\" under Advanced Settings for the test.\n\
                    3. Start the microphone test and pick something to play through the virtual mic.",
                );
            });
        }
        HelpCategory::PhysicalMic => {
            egui::ScrollArea::vertical().id_salt("general_help").auto_shrink([false, false]).show(ui, |ui| {
                help_section(
                    ui,
                    "",
                    "You can pick any physical microphone connected to your machine to let AudioPass route audio input from it through the virtual microphone.\n\n\
                     It'd be the same as using the selected physical mic without AudioPass running. \
                     This mode exists just to make it easier to switch between your physical mic and other modes without having to quit the app.",
                )
            });
        }
        HelpCategory::ApplicationAudio => {
            egui::ScrollArea::vertical().id_salt("general_help").auto_shrink([false, false]).show(ui, |ui| {
                help_section(
                    ui,
                    "",
                    "This mode lets you pick a locally running application that plays audio, to route its audio output through the virtual microphone.\n\n\
                This includes local music players, desktop clients for music streaming services, browser tabs, etc.",
                );
                help_section(
                    ui,
                    "THE DROPDOWN DOESN'T CONTAIN THE APP I WANT TO SELECT",
                    "AudioPass generates the list of audio-playing apps to pick from by querying PipeWire. \
                Sometimes PipeWire only detects an app as \"playing audio\" if the app has played audio at least once.\n\n\
                In other words, if you can't see the app you wish to select in the dropdown, \
                try playing some audio from your app for a moment or two to let PipeWire create a node for the app's audio output. \
                The app should then show up in the dropdown for you to select.",
                );
                help_section(
                    ui,
                    "FIREFOX: TABS SHOW UP IN THE DROPDOWN WITH WRONG NAMES, PLAYBACK DISCONNECTS WHEN I PAUSE/SEEK, ETC.",
                    "Individual tabs show up separately in the dropdown, but sometimes they're labeled as \"AudioStream\" instead of having the right tab name.\n\n\
                This is a known issue with Firefox on PipeWire systems (for example: https://bugzilla.mozilla.org/show_bug.cgi?id=1847824). \
                Can happen when you mute a tab for a while, when you re-open a closed tab that was playing audio, and so on.\n\n\
                Another potential issue in case of multiple tabs is that switching between them will still play audio from only one tab, \
                because the app is designed to target the first tab it can find in the PipeWire registry by node name. \
                This is a limitation, the only workaround right now is to have no more than one audio-playing tab active at a time.\n\n\
                Firefox also tends to frequently destroy and re-create streams whenever you seek playback, pause playback, etc. which causes AudioPass to lose track of them.",
                );
            });
        }
        HelpCategory::LocalFile => {
            // TODO-file-playback: when file playback is ready
        }
    }
}

fn help_section(ui: &mut egui::Ui, title: &str, body: &str) {
    egui::Frame::default().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        if !title.is_empty() {
            ui.label(RichText::new(title).color(style::ACCENT).size(16.0).extra_letter_spacing(0.5));
            ui.add_space(4.0);
        }
        ui.label(RichText::new(body).color(style::PRIMARY_TEXT).size(16.0).line_height(Some(20.0)));
        ui.separator();
        ui.add_space(4.0);
    });
}
