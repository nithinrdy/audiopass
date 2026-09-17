use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pipewire::context::ContextRc;
use pipewire::core::CoreRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::node::{Node, NodeChangeMask, NodeListener, NodeState};
use pipewire::types::ObjectType;
use ringbuf::{CachingCons, CachingProd, HeapRb};

use crate::backend::constants;
use crate::backend::pipewire::virtual_capture::VirtualCapture;
use crate::backend::pipewire::virtual_mic::VirtualMic;
use crate::backend::pipewire::{CustomEventSender, PipewireCommand, PipewireEvent};

#[derive(Clone, Debug)]
pub struct PipewirePhysicalSource {
    pub id: u32,
    pub node_name: String,
    pub description: String,
    pub is_audiopass_mic: bool, // shouldn't be selectable as a physical mic.
}

#[derive(Clone, Debug)]
#[allow(non_snake_case)]
pub struct PipewireAppSource {
    pub id: u32,
    pub node_name: String,

    pub info__props__application_name: String,
    pub info__props__media_name: Option<String>,
    pub info__state: bool,
    info__props__application_process_binary: Option<String>,
}

#[derive(Default)]
pub struct PipewireRegistryData {
    pub physical_sources: Vec<PipewirePhysicalSource>,
    pub app_sources: HashMap<u32, PipewireAppSource>,
}

struct WatchedAppNode {
    _listener: NodeListener,
    _node: Node,
}

pub struct PipewireWorkerWrapper {
    worker: Rc<RefCell<PipewireWorker>>,
}

struct PipewireWorker {
    mainloop: MainLoopRc,
    core: CoreRc,
    virtual_mic: Option<VirtualMic>,
    virtual_capture: Option<VirtualCapture>,
    event_sender: CustomEventSender,
    registry_data: PipewireRegistryData,
    audio_ring: Arc<HeapRb<f32>>,
    clear_stale_ring_samples: Arc<AtomicBool>, // 100ms of stale audio is no big deal, just an excuse to try out atomics
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
                    virtual_capture: None,
                    event_sender,
                    registry_data: PipewireRegistryData::default(),
                    audio_ring: Arc::new(HeapRb::new(constants::AUDIOPASS_MIC_RING_CAPACITY_IN_SAMPLES)),
                    clear_stale_ring_samples: Arc::new(AtomicBool::new(false)),
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
                let cmd_is_create_capture = match cmd {
                    PipewireCommand::CreateVirtualCapture { .. } => true,
                    _ => false,
                };
                let other_command_result = { closure_worker.borrow_mut().handle_command(cmd, closure_event_sender.clone()) };
                if let Err(e) = other_command_result {
                    let _ = closure_event_sender.send(if cmd_is_create_capture {
                        PipewireEvent::VirtualCaptureReady { state: Err(e) }
                    } else {
                        PipewireEvent::VirtualMicReady { state: Err(e) }
                    });
                }
            }
        });

        let registry = match self.worker.borrow().core.get_registry_rc() {
            Ok(r) => r,
            Err(e) => {
                let _ = self.worker.borrow().event_sender.send(PipewireEvent::WorkerFailure { error: e.to_string() });
                return;
            }
        };

        let watched_app_nodes = Rc::new(RefCell::new(HashMap::<u32, WatchedAppNode>::new()));
        let add_closure_watched_app_nodes = Rc::clone(&watched_app_nodes);
        let remove_closure_watch_app_nodes = Rc::clone(&watched_app_nodes);

        let add_closure_worker = Rc::clone(&self.worker);
        let remove_closure_worker = Rc::clone(&self.worker);
        let add_closure_event_sender = self.worker.borrow().event_sender.clone();
        let remove_closure_event_sender = add_closure_event_sender.clone();

        let closure_registry = registry.clone();

        let _registry_listener = registry
            .add_listener_local()
            // https://docs.pipewire.org/structpw__registry__events.html#a37bd7089a7a07d7154e111e67b25f96c
            .global(move |global| match global.type_ {
                ObjectType::Node => {
                    if let Some(physical_source) = physical_source_from_global(global) {
                        let physical_sources = &mut add_closure_worker.borrow_mut().registry_data.physical_sources;
                        physical_sources.push(physical_source);
                        let _ = add_closure_event_sender.send(PipewireEvent::PhysicalSources { sources: physical_sources.clone() });
                    } else if let Some(app_source) = app_source_from_global(global) {
                        let id = app_source.id;

                        let node: Node = match closure_registry.bind(global) {
                            Ok(node) => node,
                            Err(err) => {
                                add_closure_event_sender.send(PipewireEvent::PipewireError {
                                    error: format!("Failed to bind node {id}: {err}"),
                                });
                                return;
                            }
                        };

                        let node_listener_closure_worker = Rc::clone(&add_closure_worker);
                        let node_listener_closure_event_sender = add_closure_event_sender.clone();

                        let listener = node
                            .add_listener_local()
                            .info(move |info| {
                                if info.change_mask().contains(NodeChangeMask::STATE) {
                                    let app_sources = &mut node_listener_closure_worker.borrow_mut().registry_data.app_sources;
                                    let source_to_modify = app_sources.get_mut(&id);

                                    match source_to_modify {
                                        Some(s) => {
                                            s.info__state = match info.state() {
                                                NodeState::Running => true,
                                                _ => false,
                                            };

                                            node_listener_closure_event_sender.send(PipewireEvent::AppSources { sources: app_sources.clone() });
                                        }
                                        None => {}
                                    }
                                }

                                if info.change_mask().contains(NodeChangeMask::PROPS) {
                                    let Some(props) = info.props() else {
                                        return;
                                    };

                                    let app_sources = &mut node_listener_closure_worker.borrow_mut().registry_data.app_sources;
                                    let source_to_modify = app_sources.get_mut(&id);
                                    match source_to_modify {
                                        Some(s) => {
                                            s.info__props__application_name = props.get("application.name").map(|n| n.to_string()).unwrap_or(s.info__props__application_name.clone());
                                            s.info__props__application_process_binary = props.get("application.process.binary").map(|n| n.to_string());
                                            s.info__props__media_name = props.get("media.name").map(|n| n.to_string());

                                            node_listener_closure_event_sender.send(PipewireEvent::AppSources { sources: app_sources.clone() });
                                        }
                                        None => {}
                                    };
                                }
                            })
                            .register();

                        // just to persist listener and registry binding (dropped in global_remove())
                        add_closure_watched_app_nodes.borrow_mut().insert(id, WatchedAppNode { _listener: listener, _node: node });
                        let app_sources = &mut add_closure_worker.borrow_mut().registry_data.app_sources;
                        app_sources.insert(app_source.id, app_source);
                        add_closure_event_sender.send(PipewireEvent::AppSources { sources: app_sources.clone() }); // probably not necessary because event_senders inside Node _listener will also emit at least once, but still
                    }
                }
                _ => {}
            })
            .global_remove(move |removed_id| {
                let registry = &mut remove_closure_worker.borrow_mut().registry_data;

                let app_sources = &mut registry.app_sources;
                if app_sources.contains_key(&removed_id) {
                    app_sources.remove(&removed_id);
                    remove_closure_watch_app_nodes.borrow_mut().remove(&removed_id);
                    let _ = remove_closure_event_sender.send(PipewireEvent::AppSources { sources: app_sources.clone() });
                    return;
                };

                let physical_sources = &mut registry.physical_sources;
                if let Some(idx) = physical_sources.iter().position(|s| s.id == removed_id) {
                    physical_sources.remove(idx);
                    let _ = remove_closure_event_sender.send(PipewireEvent::PhysicalSources { sources: physical_sources.clone() });
                }
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
                self.virtual_mic = Some(VirtualMic::new(
                    self.core.clone(),
                    event_sender.clone(),
                    persistent_audio_consumer,
                    self.clear_stale_ring_samples.clone(),
                )?);
            }
            PipewireCommand::CreateVirtualCapture { selected_node_name } => {
                if let Some(capture) = self.virtual_capture.take() {
                    drop(capture) // also drops the associated CachingProd instance tied to this stream, so ::new() in the next line won't panic
                }
                self.clear_stale_ring_samples.store(true, Ordering::Release);
                let audio_producer_for_this_mic = CachingProd::new(self.audio_ring.clone());
                self.virtual_capture = Some(VirtualCapture::new(
                    self.core.clone(),
                    selected_node_name,
                    event_sender.clone(),
                    audio_producer_for_this_mic,
                    self.clear_stale_ring_samples.clone(),
                )?);
            }
            PipewireCommand::DropVirtualCapture => {
                self.virtual_capture = None;
                self.clear_stale_ring_samples.store(true, Ordering::Release);
            }
            _ => unreachable!(),
        }

        Ok(())
    }
}

fn physical_source_from_global(global: &pipewire::registry::GlobalObject<&pipewire::spa::utils::dict::DictRef>) -> Option<PipewirePhysicalSource> {
    let props = global.props.as_ref()?;
    if props.get("media.class") != Some("Audio/Source") {
        return None;
    }

    let node_name = props.get("node.name")?.to_string();
    let description = props.get("node.description").or_else(|| props.get("node.nick")).unwrap_or(&node_name).to_string();
    let is_audiopass_mic = node_name.contains("audiopass.virtual-mic.");
    Some(PipewirePhysicalSource {
        id: global.id,
        node_name,
        description,
        is_audiopass_mic,
    })
}

fn app_source_from_global(global: &pipewire::registry::GlobalObject<&pipewire::spa::utils::dict::DictRef>) -> Option<PipewireAppSource> {
    let props = global.props.as_ref()?;
    if props.get("media.class") != Some("Stream/Output/Audio") {
        return None;
    }

    let node_name = props.get("node.name")?.to_string();
    let application_name = props.get("application.name").unwrap_or(&node_name).to_string();
    Some(PipewireAppSource {
        id: global.id,
        node_name,
        info__props__application_name: application_name,
        info__props__media_name: None,
        info__state: false,
        info__props__application_process_binary: None,
    })
}
