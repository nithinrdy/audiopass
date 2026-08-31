use std::sync::mpsc;

use crate::backend::playback::PlaybackCommand;

pub struct PlaybackWorker {
    command_receiver: mpsc::Receiver<PlaybackCommand>,
}

impl PlaybackWorker {
    pub fn new(command_receiver: mpsc::Receiver<PlaybackCommand>) -> Self {
        PlaybackWorker { command_receiver }
    }

    pub fn run(&self) {
        while let Ok(cmd) = self.command_receiver.recv() {
            match cmd {
                PlaybackCommand::Shutdown {} => {
                    break;
                }
            }
        }
    }
}

impl PlaybackWorker {}
