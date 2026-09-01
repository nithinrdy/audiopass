use std::collections::HashMap;
use std::sync::{Once, mpsc::Sender};

use eframe::egui::Context;

pub use crate::backend::pipewire::worker::PipewireAppSource;
pub use crate::backend::pipewire::worker::PipewirePhysicalSource;
mod utils;
mod virtual_mic;
mod virtual_sink;
mod worker;

static PIPEWIRE_INIT: Once = Once::new();

pub enum PipewireCommand {
    CreateVirtualMic,
    CreateVirtualSink { selected_node_name: String },
    Shutdown,
}
pub enum PipewireEvent {
    PipewireError { error: String },
    VirtualMicReady { state: Result<(), String> },
    VirtualSinkReady { state: Result<(), String> },
    PhysicalSources { sources: Vec<PipewirePhysicalSource> },
    AppSources { sources: HashMap<u32, PipewireAppSource> },
}

#[derive(Clone)]
pub struct CustomEventSender {
    sender: Sender<PipewireEvent>,
    egui_context: Context,
}
impl CustomEventSender {
    pub fn send(&self, ev: PipewireEvent) {
        let _ = self.sender.send(ev);
        self.egui_context.request_repaint();
    }
}

pub struct PipewireHook {
    command_sender: pipewire::channel::Sender<PipewireCommand>,
    pub event_receiver: std::sync::mpsc::Receiver<PipewireEvent>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub fn start_pipewire_worker(cloned_egui_context: Context) -> Result<PipewireHook, String> {
    PIPEWIRE_INIT.call_once(pipewire::init);

    // app to pipewire
    let (command_sender, command_receiver) = pipewire::channel::channel::<PipewireCommand>();
    // pipewire to app
    let (event_sender, event_receiver) = std::sync::mpsc::channel::<PipewireEvent>();

    let thread = std::thread::Builder::new()
        .name("audiopass-pipewire".to_string())
        .spawn(move || {
            let event_sender = CustomEventSender {
                sender: event_sender,
                egui_context: cloned_egui_context,
            };

            let worker = match worker::PipewireWorkerWrapper::new(event_sender.clone()) {
                Ok(w) => w,
                Err(e) => {
                    let _ = event_sender.send(PipewireEvent::PipewireError { error: e });
                    return;
                }
            };

            worker.activate(command_receiver);
        })
        .map_err(|e| format!("Failed to start Pipewire worker thread: {}", e))?;

    Ok(PipewireHook {
        command_sender,
        event_receiver,
        thread: Some(thread),
    })
}

impl PipewireHook {
    pub fn create_virtual_mic(&mut self) {
        let _ = self.command_sender.send(PipewireCommand::CreateVirtualMic);
    }

    pub fn create_virtual_sink(&mut self, selected_node_name: String) {
        let _ = self.command_sender.send(PipewireCommand::CreateVirtualSink { selected_node_name });
    }

    pub fn shutdown(&mut self) {
        let _ = self.command_sender.send(PipewireCommand::Shutdown);
        match self.thread.take() {
            Some(t) => {
                let _ = t.join();
            }
            None => {}
        };
    }
}
