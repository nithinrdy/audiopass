use pipewire::spa::{pod::Pod, utils::Direction};
use ringbuf::{
    HeapProd,
    traits::{Observer, Producer},
};
use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::backend::{
    constants,
    pipewire::{CustomEventSender, PipewireEvent, utils::get_serialized_vec_for_pod},
};

struct AdditionalData {
    sender: CustomEventSender,
    audio_producer: HeapProd<f32>,
    clear_stale_ring_samples: Arc<AtomicBool>,
}

pub struct VirtualCapture {
    _stream: pipewire::stream::StreamRc,
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
}

impl VirtualCapture {
    pub fn new(
        pw_core: pipewire::core::CoreRc,
        selected_node_name: String,
        state_change_sender: CustomEventSender,
        audio_producer: HeapProd<f32>,
        clear_stale_ring_samples: Arc<AtomicBool>,
    ) -> Result<Self, String> {
        // https://docs.pipewire.org/group__pw__stream.html#ga712ca485dc634252d144556074980f0a
        let stream = pipewire::stream::StreamRc::new(
            pw_core,
            constants::AUDIOPASS_VIRTUAL_CAPTURE_STREAM_NAME,
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
            Err(e) => return Err(format!("Failed to create Pipewire stream for virtual capture: {}", e)),
        };

        let listener = stream
            .add_local_listener_with_user_data(AdditionalData {
                sender: state_change_sender,
                audio_producer,
                clear_stale_ring_samples,
            })
            .state_changed(|_stream, data, _old_state, new_state| {
                // so rustfmt doesnt inline this
                match new_state {
                    pipewire::stream::StreamState::Error(e) => {
                        let _ = data.sender.send(PipewireEvent::VirtualCaptureReady { state: Err(e) });
                    }
                    pipewire::stream::StreamState::Paused => {
                        let _ = data.sender.send(PipewireEvent::VirtualCaptureReady { state: Ok(()) });
                    }
                    pipewire::stream::StreamState::Streaming => {
                        let _ = data.sender.send(PipewireEvent::VirtualCaptureReady { state: Ok(()) });
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

                let complete_frame_byte_count = (end - start) - ((end - start) % constants::AUDIOPASS_BYTES_PER_FRAME);
                let received_sample_count = complete_frame_byte_count / constants::AUDIOPASS_BYTES_PER_SAMPLE;

                let vacant_sample_slots_count = data.audio_producer.vacant_len();
                let sample_count_to_push = received_sample_count.min(vacant_sample_slots_count - (vacant_sample_slots_count % constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT as usize));

                let pushed_sample_count = data.audio_producer.push_iter(
                    bytes_in_buffer[start..(start + (sample_count_to_push * constants::AUDIOPASS_BYTES_PER_SAMPLE))]
                        .chunks_exact(constants::AUDIOPASS_BYTES_PER_SAMPLE)
                        .map(|sample_bytes| f32::from_le_bytes(sample_bytes.try_into().unwrap())), // pipewire callbacks + .chunks_exact(4) will only ever provide whole samples
                );

                if pushed_sample_count < received_sample_count {
                    // all provided frames could not be written = not enough capacity in ring
                    // = audio in the ring is stale (piled up unconsumed samples) = mark for .clear() by consumer
                    data.clear_stale_ring_samples.store(true, Ordering::Release);
                }
            })
            .register();

        let listener = match listener {
            Ok(l) => l,
            Err(e) => return Err(format!("Failed to attach callbacks to Pipewire stream for virtual capture: {}", e)),
        };

        match stream.connect(
            Direction::Input,
            None,
            // https://docs.pipewire.org/group__pw__stream.html#ga058907c2dffbb8fb5ede8a53d7604106
            pipewire::stream::StreamFlags::AUTOCONNECT | pipewire::stream::StreamFlags::MAP_BUFFERS | pipewire::stream::StreamFlags::RT_PROCESS,
            &mut ([match Pod::from_bytes(get_serialized_vec_for_pod()?.deref()) {
                Some(p) => p,
                None => return Err(format!("Failed to serialize libspa POD while creating virtual capture stream")),
            }]),
        ) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to connect virtual capture stream: {}", e)),
        }

        Ok(VirtualCapture {
            _stream: stream,
            _callback_listener: listener,
        })
    }
}
