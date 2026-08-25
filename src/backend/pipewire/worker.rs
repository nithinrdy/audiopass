use std::cell::RefCell;
use std::rc::Rc;

use pipewire::context::ContextRc;
use pipewire::core::CoreRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::types::ObjectType;
use ringbuf::{Arc, CachingCons, CachingProd, HeapRb, traits::Consumer};

use crate::backend::constants;
use crate::backend::pipewire::virtual_mic::VirtualMic;
use crate::backend::pipewire::virtual_sink::VirtualSink;
use crate::backend::pipewire::{CustomEventSender, PipewireCommand, PipewireEvent};

#[derive(Clone, Debug)]
pub struct PipewireSource {
    pub id: u32,
    pub node_name: String,
    pub description: String,
    pub is_audiopass_mic: bool, // shouldn't be selectable as a physical mic.
}
#[derive(Default)]
pub struct PipewireRegistryData {
    pub sources: Vec<PipewireSource>,
}

pub struct PipewireWorkerWrapper {
    worker: Rc<RefCell<PipewireWorker>>,
}

struct PipewireWorker {
    mainloop: MainLoopRc,
    core: CoreRc,
    virtual_mic: Option<VirtualMic>,
    virtual_sink: Option<VirtualSink>,
    event_sender: CustomEventSender,
    registry_data: PipewireRegistryData,
    audio_ring: Arc<HeapRb<u8>>,
}

impl PipewireWorkerWrapper {
    pub fn new(event_sender: CustomEventSender) -> Result<Self, String> {
        let properties = {
            // https://docs.pipewire.org/src_2pipewire_2keys_8h.html
            pipewire::properties::properties! {
                *pipewire::keys::APP_ID => constants::AUDIOPASS_APP_ID,
                *pipewire::keys::APP_NAME => constants::AUDIOPASS_APP_NAME,
                *pipewire::keys::CLIENT_NAME => constants::AUDIOPASS_PIPEWIRE_CLIENT_NAME,
            }
        };
        let mainloop = match MainLoopRc::new(None) {
            Ok(m) => m,
            Err(e) => return Err(format!("Failed to create Pipewire mainloop: {e}")),
        };
        let context = match ContextRc::new(&mainloop, None) {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to create context from Pipewire mainloop: {e}")),
        };
        let core = match context.connect_rc(Some(properties)) {
            Ok(c) => c,
            Err(e) => return Err(format!("Failed to create core from Pipewire mainloop: {e}")),
        };

        Ok(PipewireWorkerWrapper {
            worker: Rc::new(RefCell::new({
                PipewireWorker {
                    mainloop,
                    core,
                    virtual_mic: None,
                    virtual_sink: None,
                    event_sender,
                    registry_data: PipewireRegistryData::default(),
                    audio_ring: Arc::new(HeapRb::new(constants::AUDIOPASS_MIC_RING_CAPACITY_BYTES)),
                }
            })),
        })
    }

    pub fn activate(&self, command_receiver: pipewire::channel::Receiver<PipewireCommand>) {
        let closure_worker = Rc::clone(&self.worker);
        let mainloop = self.worker.borrow().mainloop.clone();
        let closure_mainloop = mainloop.clone();
        let closure_event_sender = self.worker.borrow().event_sender.clone();

        let _command_handler = command_receiver.attach(mainloop.loop_(), move |cmd| {
            if let PipewireCommand::Shutdown = cmd {
                closure_mainloop.quit();
            } else {
                let other_command_result = { closure_worker.borrow_mut().handle_command(cmd, closure_event_sender.clone()) };
                if let Err(e) = other_command_result {
                    let _ = closure_event_sender.send(PipewireEvent::PipewireError { error: e });
                }
            }
        });

        let registry = match self.worker.borrow().core.get_registry_rc() {
            Ok(r) => r,
            Err(e) => {
                let _ = self.worker.borrow().event_sender.send(PipewireEvent::PipewireError { error: e.to_string() });
                return;
            }
        };

        let closure_2_worker = Rc::clone(&self.worker);
        let closure_3_worker = Rc::clone(&self.worker);
        let closure_2_event_sender = self.worker.borrow().event_sender.clone();
        let closure_3_event_sender = closure_2_event_sender.clone();

        let _registry_listener = registry
            .add_listener_local()
            // https://docs.pipewire.org/structpw__registry__events.html#a37bd7089a7a07d7154e111e67b25f96c
            .global(move |global| match global.type_ {
                ObjectType::Node => {
                    if let Some(source) = source_from_global(global) {
                        let sources = &mut closure_2_worker.borrow_mut().registry_data.sources;
                        sources.push(source);
                        let _ = closure_2_event_sender.send(PipewireEvent::Sources { sources: sources.clone() });
                    }
                }
                _ => {}
            })
            // https://docs.pipewire.org/structpw__registry__events.html#a04c4f7cbbf5dcc0c54887862887dbc97
            .global_remove(move |removed_id| {
                let sources = &mut closure_3_worker.borrow_mut().registry_data.sources;
                let Some(idx) = sources.iter().position(|s| s.id == removed_id) else {
                    // turns out add/remove callbacks run for a lot more than just audio devices dis/connecting
                    // (even when moving the cursor over app icons in the taskbar???)
                    // return early if change is not related.
                    return;
                };
                sources.remove(idx);
                let _ = closure_3_event_sender.send(PipewireEvent::Sources { sources: sources.clone() });
            })
            .register();

        mainloop.run();
    }
}

impl PipewireWorker {
    fn handle_command(&mut self, cmd: PipewireCommand, event_sender: CustomEventSender) -> Result<(), String> {
        match cmd {
            PipewireCommand::CreateVirtualMic => {
                if self.virtual_mic.is_some() {
                    return Err(format!("You're trying to create a virtual mic after one has already been created, something's gone terribly wrong."));
                }

                let persistent_audio_consumer = CachingCons::new(self.audio_ring.clone());
                self.virtual_mic = Some(VirtualMic::new(self.core.clone(), event_sender.clone(), persistent_audio_consumer)?);
            }
            PipewireCommand::CreateVirtualSink { selected_node_name } => {
                if let Some(sink) = self.virtual_sink.take() {
                    drop(sink) // also drops the associated CachingProd instance tied to this sink, so ::new() in the next line won't panic
                }
                let audio_producer_for_this_mic = CachingProd::new(self.audio_ring.clone());
                self.virtual_sink = Some(VirtualSink::new(self.core.clone(), selected_node_name, event_sender.clone(), audio_producer_for_this_mic)?);
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

fn source_from_global(global: &pipewire::registry::GlobalObject<&pipewire::spa::utils::dict::DictRef>) -> Option<PipewireSource> {
    let props = global.props.as_ref()?;
    if props.get("media.class") != Some("Audio/Source") {
        return None;
    }

    let node_name = props.get("node.name")?.to_string();
    let description = props.get("node.description").or_else(|| props.get("node.nick")).unwrap_or(&node_name).to_string();
    let is_audiopass_mic = node_name.contains("audiopass.virtual-mic.");
    Some(PipewireSource {
        id: global.id,
        node_name,
        description,
        is_audiopass_mic,
    })
}
