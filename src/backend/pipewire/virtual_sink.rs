use pipewire::spa::{pod::Pod, utils::Direction};

use crate::backend::{
    constants,
    pipewire::{CustomEventSender, PipewireEvent, utils::get_serialized_vec_for_pod},
};
use std::ops::Deref;

struct AdditionalData {
    sender: CustomEventSender,
}

pub struct VirtualSink {
    _stream: pipewire::stream::StreamRc,
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
}

impl VirtualSink {
    pub fn new(pw_core: pipewire::core::CoreRc, selected_node_name: String, state_change_sender: CustomEventSender) -> Result<Self, String> {
        // https://docs.pipewire.org/group__pw__stream.html#ga712ca485dc634252d144556074980f0a
        let stream = pipewire::stream::StreamRc::new(
            pw_core,
            constants::AUDIOPASS_VIRTUAL_SINK_STREAM_NAME,
            // https://docs.pipewire.org/src_2pipewire_2keys_8h.html
            pipewire::properties::properties! {
                *pipewire::keys::MEDIA_TYPE => "Audio",
                *pipewire::keys::MEDIA_CATEGORY => "Capture",
                *pipewire::keys::MEDIA_ROLE => "DSP",
                *pipewire::keys::TARGET_OBJECT => selected_node_name.as_str(),
                *pipewire::keys::NODE_VIRTUAL => "true",
            },
        );

        let stream = match stream {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to create Pipewire stream for virtual sink: {}", e)),
        };

        let listener = stream
            .add_local_listener_with_user_data(AdditionalData { sender: state_change_sender })
            .state_changed(|_stream, data, _old_state, new_state| {
                // so rustfmt doesnt inline this
                match new_state {
                    pipewire::stream::StreamState::Error(e) => {
                        let _ = data.sender.send(PipewireEvent::VirtualSinkReady { state: Err(e) });
                    }
                    pipewire::stream::StreamState::Paused => {
                        let _ = data.sender.send(PipewireEvent::VirtualSinkReady { state: Ok(()) });
                    }
                    pipewire::stream::StreamState::Streaming => {
                        let _ = data.sender.send(PipewireEvent::VirtualSinkReady { state: Ok(()) });
                    }
                    _ => return,
                }
            })
            .process(|stream, data| {
                // todo
            })
            .register();

        let listener = match listener {
            Ok(l) => l,
            Err(e) => return Err(format!("Failed to attach callbacks to Pipewire stream for virtual sink: {}", e)),
        };

        match stream.connect(
            Direction::Input,
            None,
            // https://docs.pipewire.org/group__pw__stream.html#ga058907c2dffbb8fb5ede8a53d7604106
            pipewire::stream::StreamFlags::AUTOCONNECT | pipewire::stream::StreamFlags::MAP_BUFFERS | pipewire::stream::StreamFlags::RT_PROCESS,
            &mut ([match Pod::from_bytes(get_serialized_vec_for_pod()?.deref()) {
                Some(p) => p,
                None => return Err(format!("Failed to serialize libspa POD while creating virtual sink stream")),
            }]),
        ) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to connect virtual sink stream: {}", e)),
        }

        Ok(VirtualSink {
            _stream: stream,
            _callback_listener: listener,
        })
    }
}
