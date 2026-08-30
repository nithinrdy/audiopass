use crate::backend;

use backend::datastore::types::track;

pub struct PlaybackController {
    active_track: Option<track::Track>,
}

impl PlaybackController {
    pub fn new() -> Self {
        PlaybackController { active_track: None }
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
        if track_list.len() == 0 {
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
        if track_list.len() == 0 {
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
