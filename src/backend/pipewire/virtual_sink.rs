use pipewire::spa::{pod::Pod, utils::Direction};
use ringbuf::{HeapProd, traits::Producer};
use std::ops::Deref;

use crate::backend::{
    constants,
    pipewire::{CustomEventSender, PipewireEvent, utils::get_serialized_vec_for_pod},
};

struct AdditionalData {
    sender: CustomEventSender,
    audio_producer: HeapProd<u8>,
}

pub struct VirtualSink {
    _stream: pipewire::stream::StreamRc,
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
}

impl VirtualSink {
    pub fn new(pw_core: pipewire::core::CoreRc, selected_node_name: String, state_change_sender: CustomEventSender, audio_producer: HeapProd<u8>) -> Result<Self, String> {
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
            .add_local_listener_with_user_data(AdditionalData {
                sender: state_change_sender,
                audio_producer,
            })
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
                // similar to source stream but valid region is marked by pipewire, read by me (opposite of me marking, pipewire reading)
                let mut buffer = match stream.dequeue_buffer() {
                    Some(b) => b,
                    None => return,
                };
                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }
                let buffer_data = &mut datas[0];

                // valid audio may not start and end exactly at beginning and end of the bytes array https://docs.pipewire.org/structspa__chunk.html
                let size = buffer_data.chunk().size() as usize;
                let start = buffer_data.chunk().offset() as usize;

                let bytes_in_buffer = match buffer_data.data() {
                    Some(b) => {
                        if b.is_empty() {
                            return;
                        } else {
                            b
                        }
                    }
                    None => return,
                };

                let start = start % bytes_in_buffer.len(); // % sample_bytes.len() because https://docs.pipewire.org/structspa__chunk.html#ae7a889b81a5d56ff5babd521b5ce7cb7
                let end = (start + size).min(bytes_in_buffer.len()); // ignoring any bytes that wrap around past end, TODO later: don't?
                if start >= end {
                    return;
                }

                let slice_length_adjusted_for_whole_frames_only = (end - start) - ((end - start) % constants::AUDIOPASS_AUDIO_FRAME_SIZE_BYTES);
                data.audio_producer.push_slice(&bytes_in_buffer[start..start + slice_length_adjusted_for_whole_frames_only]);
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
