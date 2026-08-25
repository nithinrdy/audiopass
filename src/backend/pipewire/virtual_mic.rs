// mostly from this example: https://sources.debian.org/src/rust-pipewire/0.9.2-2/examples/audio-capture.rs/#L43.

use pipewire::spa::{pod::Pod, utils::Direction};
use ringbuf::{HeapCons, traits::Consumer};
use std::ops::Deref;

use crate::backend::{
    constants,
    pipewire::{CustomEventSender, PipewireEvent, utils::get_serialized_vec_for_pod},
};

struct AdditionalData {
    sender: CustomEventSender,
    audio_consumer: HeapCons<u8>,
}

pub struct VirtualMic {
    _stream: pipewire::stream::StreamRc,
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
}

impl VirtualMic {
    pub fn new(pw_core: pipewire::core::CoreRc, state_change_sender: CustomEventSender, audio_consumer: HeapCons<u8>) -> Result<Self, String> {
        // https://docs.pipewire.org/group__pw__stream.html#ga712ca485dc634252d144556074980f0a
        let stream = pipewire::stream::StreamRc::new(
            pw_core,
            constants::AUDIOPASS_VIRTUAL_MIC_STREAM_NAME,
            // https://docs.pipewire.org/src_2pipewire_2keys_8h.html
            pipewire::properties::properties! {
                *pipewire::keys::MEDIA_TYPE => "Audio",
                *pipewire::keys::MEDIA_ROLE => "Communication",
                *pipewire::keys::MEDIA_CLASS => "Audio/Source",
                *pipewire::keys::NODE_NAME => format!("{}.{}", constants::AUDIOPASS_VIRTUAL_MIC_IDENTIFIER_PREFIX, std::process::id()),
                *pipewire::keys::NODE_DESCRIPTION => constants::AUDIOPASS_VIRTUAL_MIC_DESCRIPTION,
                *pipewire::keys::AUDIO_CHANNELS => constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT.to_string().as_str(),
            },
        );

        let stream = match stream {
            Ok(s) => s,
            Err(e) => return Err(format!("Failed to create Pipewire stream for virtual mic: {}", e)),
        };

        let listener = stream
            .add_local_listener_with_user_data(AdditionalData {
                sender: state_change_sender,
                audio_consumer,
            })
            .state_changed(|_stream, data, _old_state, new_state| {
                // so rustfmt doesnt inline this
                match new_state {
                    pipewire::stream::StreamState::Error(e) => {
                        let _ = data.sender.send(PipewireEvent::VirtualMicReady { state: Err(e) });
                    }
                    pipewire::stream::StreamState::Paused => {
                        let _ = data.sender.send(PipewireEvent::VirtualMicReady { state: Ok(()) });
                    }
                    pipewire::stream::StreamState::Streaming => {
                        let _ = data.sender.send(PipewireEvent::VirtualMicReady { state: Ok(()) });
                    }
                    _ => return,
                }
            })
            .process(|stream, data| {
                // mostly this on_process() example: https://docs.pipewire.org/audio-src_8c-example.html
                // buffer, data_in_buffer, data_in_buffer.data() stucture: https://docs.pipewire.org/page_spa_buffer.html
                let Some(mut buffer) = stream.dequeue_buffer() else {
                    return;
                };
                let requested_frame_count = buffer.requested() as usize;

                let datas = buffer.datas_mut();
                if datas.is_empty() {
                    return;
                }
                // on_process() example
                // [0] because F32LE is a packed data format and stored all in one (first) block: https://stackoverflow.com/a/29307174 (understanding: work-in-progress)
                let buffer_data = &mut datas[0];

                let output_size = if let Some(bytes_in_buffer) = buffer_data.data() {
                    let max_possible_frame_count_for_provided_bytes_array = bytes_in_buffer.len() / constants::AUDIOPASS_AUDIO_FRAME_SIZE_BYTES;

                    // buffer.requested() is a suggestion, I think buffer.requested() == 0 doesn't mean leave empty
                    // (because the example does fill the whole array even when it's 0)
                    // I think buffer.requested() is "fill this much, or fill the whole writable array" (but not 100% sure)
                    let frame_count_to_write = if requested_frame_count == 0 {
                        max_possible_frame_count_for_provided_bytes_array
                    } else {
                        requested_frame_count.min(max_possible_frame_count_for_provided_bytes_array)
                    };
                    let output_size_in_bytes = frame_count_to_write * constants::AUDIOPASS_AUDIO_FRAME_SIZE_BYTES;

                    let popped_count = data.audio_consumer.pop_slice(&mut bytes_in_buffer[..output_size_in_bytes]); // = min(arg_array_size, available_ring_contents_size)
                    // if entire array was filled in previous line, popped_count == output_size_in_bytes, next line does nothing.
                    // if ring didnt have enough fill entire array, fill the rest of writable region with silence.
                    bytes_in_buffer[popped_count..output_size_in_bytes].fill(0);
                    output_size_in_bytes
                } else {
                    0
                };

                // marking valid region for pipewire after writing (last part of on_process())
                let chunk = buffer_data.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = constants::AUDIOPASS_AUDIO_FRAME_SIZE_BYTES as i32;
                *chunk.size_mut() = output_size as u32;
            })
            .register();

        let listener = match listener {
            Ok(l) => l,
            Err(e) => return Err(format!("Failed to attach callbacks to Pipewire stream for virtual mic: {}", e)),
        };

        match stream.connect(
            Direction::Output,
            None,
            // https://docs.pipewire.org/group__pw__stream.html#ga058907c2dffbb8fb5ede8a53d7604106
            pipewire::stream::StreamFlags::AUTOCONNECT | pipewire::stream::StreamFlags::MAP_BUFFERS | pipewire::stream::StreamFlags::RT_PROCESS,
            &mut ([match Pod::from_bytes(get_serialized_vec_for_pod()?.deref()) {
                Some(p) => p,
                None => return Err(format!("Failed to serialize libspa POD while creating virtual mic stream")),
            }]),
        ) {
            Ok(_) => {}
            Err(e) => return Err(format!("Failed to connect virtual mic stream: {}", e)),
        }

        Ok(VirtualMic {
            _stream: stream,
            _callback_listener: listener,
        })
    }
}
