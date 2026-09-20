use crate::backend;
use std::{sync::mpsc, thread::JoinHandle};
mod worker;
use backend::datastore::types::track;

enum PlaybackCommand {
    Shutdown,
}

pub struct PlaybackController {
    active_track: Option<track::Track>,
    command_sender: mpsc::Sender<PlaybackCommand>,
    _thread: Option<JoinHandle<()>>,
}

impl PlaybackController {
    pub fn new() -> Result<Self, String> {
        let (command_sender, command_receiver) = mpsc::channel();

        let thread = std::thread::Builder::new().name("audiopass-playback".to_string()).spawn(move || {
            let worker = worker::PlaybackWorker::new(command_receiver);
            worker.run();
        });
        let thread = match thread {
            Ok(h) => h,
            Err(e) => return Err(format!("Failed to begin playback worker thread: {e}")),
        };

        Ok(PlaybackController {
            active_track: None,
            command_sender,
            _thread: Some(thread),
        })
    }
}

impl PlaybackController {
    pub fn get_active_track(&self) -> Option<&track::Track> {
        self.active_track.as_ref()
    }

    pub fn reset_and_new(&mut self, track: track::Track) {
        self.active_track = Some(track)
    }

    pub fn reset(&mut self) {
        self.active_track = None
    }

    pub fn start_or_resume(&self) {}

    pub fn stop(&self) {}

    pub fn play_next(&mut self, track_list: &[track::Track]) {
        if track_list.is_empty() {
            return;
        }

        if let Some(active_track) = self.active_track.as_ref() {
            let idx = track_list.iter().position(|t| t.id == active_track.id);
            let idx = match idx {
                Some(idx) => idx,
                None => {
                    self.reset();
                    return;
                }
            };

            self.reset_and_new(track_list[(idx + 1) % track_list.len()].clone());
            self.start_or_resume();
        }
    }

    pub fn play_previous(&mut self, track_list: &[track::Track]) {
        if track_list.is_empty() {
            return;
        }

        if let Some(active_track) = self.active_track.as_ref() {
            let idx = track_list.iter().position(|t| t.id == active_track.id);
            let idx = match idx {
                Some(idx) => idx,
                None => {
                    self.reset();
                    return;
                }
            };

            self.reset_and_new(track_list[((idx + track_list.len()) - 1) % track_list.len()].clone());
            self.start_or_resume();
        }
    }
}

impl Drop for PlaybackController {
    fn drop(&mut self) {
        let _ = self.command_sender.send(PlaybackCommand::Shutdown);
        if let Some(t) = self._thread.take() {
            let _ = t.join();
        };
    }
}
