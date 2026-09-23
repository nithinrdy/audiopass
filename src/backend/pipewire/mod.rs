use std::collections::HashMap;
use std::sync::{
    Arc, Once,
    atomic::{AtomicU32, Ordering},
    mpsc::Sender,
};

use eframe::egui::Context;

pub use crate::backend::pipewire::worker::PipewireAppSource;
pub use crate::backend::pipewire::worker::PipewirePhysicalSource;
mod utils;
mod virtual_capture;
mod virtual_mic;
mod worker;

static PIPEWIRE_INIT: Once = Once::new();

pub enum PipewireCommand {
    CreateVirtualMic,
    CreateVirtualCapture { selected_node_name: String },
    DropVirtualCapture,
    Shutdown,
}
pub enum PipewireEvent {
    PipewireError { error: String },
    WorkerFailure { error: String },
    VirtualMicReady { state: Result<(), String> },
    VirtualCaptureReady { state: Result<(), String> },
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
    gain_percent: Arc<AtomicU32>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub fn start_pipewire_worker(cloned_egui_context: Context) -> Result<PipewireHook, String> {
    PIPEWIRE_INIT.call_once(pipewire::init);

    // app to pipewire
    let (command_sender, command_receiver) = pipewire::channel::channel::<PipewireCommand>();
    // pipewire to app
    let (event_sender, event_receiver) = std::sync::mpsc::channel::<PipewireEvent>();
    let gain_percent = Arc::new(AtomicU32::new(100));
    let worker_gain_percent = gain_percent.clone();

    let thread = std::thread::Builder::new()
        .name("audiopass-pipewire".to_string())
        .spawn(move || {
            let event_sender = CustomEventSender {
                sender: event_sender,
                egui_context: cloned_egui_context,
            };

            let worker = match worker::PipewireWorkerWrapper::new(event_sender.clone(), worker_gain_percent) {
                Ok(w) => w,
                Err(e) => {
                    event_sender.send(PipewireEvent::WorkerFailure { error: e });
                    return;
                }
            };

            worker.activate(command_receiver);
        })
        .map_err(|e| format!("Failed to start Pipewire worker thread: {}", e))?;

    Ok(PipewireHook {
        command_sender,
        event_receiver,
        gain_percent,
        thread: Some(thread),
    })
}

impl PipewireHook {
    pub fn create_virtual_mic(&mut self) -> Result<(), String> {
        self.command_sender
            .send(PipewireCommand::CreateVirtualMic)
            .map_err(|_| "Failed to send virtual microphone creation request to Pipewire worker".to_string())
    }

    pub fn create_virtual_capture(&mut self, selected_node_name: String) -> Result<(), String> {
        self.command_sender
            .send(PipewireCommand::CreateVirtualCapture { selected_node_name })
            .map_err(|_| "Failed to send virtual capture creation request to Pipewire worker".to_string())
    }

    pub fn drop_virtual_capture(&mut self) -> Result<(), String> {
        self.command_sender
            .send(PipewireCommand::DropVirtualCapture)
            .map_err(|_| "Failed to send virtual capture drop request to Pipewire worker".to_string())
    }

    pub fn set_gain(&self, gain: u32) {
        self.gain_percent.store(gain.max(0).min(200), Ordering::Relaxed);
    }

    pub fn shutdown(&mut self) {
        let _ = self.command_sender.send(PipewireCommand::Shutdown);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        };
    }
}
