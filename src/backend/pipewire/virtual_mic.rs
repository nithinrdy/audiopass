// Partly based on this example: https://sources.debian.org/src/rust-pipewire/0.9.2-2/examples/audio-capture.rs/#L43

use pipewire::spa::{pod::Pod, utils::Direction};
use ringbuf::{
    HeapCons,
    traits::{Consumer, Observer},
};
use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use crate::backend::{
    constants,
    pipewire::{CustomEventSender, PipewireEvent, utils::get_serialized_vec_for_pod},
};

struct AdditionalData {
    sender: CustomEventSender,
    audio_consumer: HeapCons<f32>,
    output_sample_buffer: Vec<f32>,
    clear_stale_ring_samples: Arc<AtomicBool>,
    gain_percent: Arc<AtomicU32>,
}

pub struct VirtualMic {
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
    _stream: pipewire::stream::StreamRc,
}

impl Drop for VirtualMic {
    fn drop(&mut self) {
        let _ = self._stream.disconnect();
    }
}

impl VirtualMic {
    pub fn new(
        pw_core: pipewire::core::CoreRc,
        state_change_sender: CustomEventSender,
        audio_consumer: HeapCons<f32>,
        clear_stale_ring_samples: Arc<AtomicBool>,
        gain_percent: Arc<AtomicU32>,
    ) -> Result<Self, String> {
        // pre-allocate vec so dont have to allocate inside process() closure for rt-safety
        let output_sample_buffer = vec![0.0; constants::AUDIOPASS_MIC_RING_CAPACITY_IN_SAMPLES];

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
                output_sample_buffer,
                clear_stale_ring_samples,
                gain_percent,
            })
            .state_changed(|_stream, data, _old_state, new_state| {
                // so rustfmt doesnt inline this
                match new_state {
                    pipewire::stream::StreamState::Error(e) => {
                        data.sender.send(PipewireEvent::VirtualMicReady { state: Err(e) });
                    }
                    pipewire::stream::StreamState::Paused => {
                        data.sender.send(PipewireEvent::VirtualMicReady { state: Ok(()) });
                    }
                    pipewire::stream::StreamState::Streaming => {
                        data.sender.send(PipewireEvent::VirtualMicReady { state: Ok(()) });
                    }
                    _ => (),
                }
            })
            .process(|stream, data| {
                if data.clear_stale_ring_samples.swap(false, Ordering::Acquire) {
                    data.audio_consumer.clear();
                }
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
                    let max_possible_frame_count_for_provided_bytes_array = bytes_in_buffer.len() / constants::AUDIOPASS_BYTES_PER_FRAME;

                    // buffer.requested() is a suggestion, I think buffer.requested() == 0 doesn't mean leave empty
                    // (because the example does fill the whole array even when it's 0)
                    // I think buffer.requested() is "fill this much, or fill the whole writable array" (but not 100% sure)
                    let frame_count_to_write = if requested_frame_count == 0 {
                        max_possible_frame_count_for_provided_bytes_array
                    } else {
                        requested_frame_count.min(max_possible_frame_count_for_provided_bytes_array)
                    };
                    let output_size_in_bytes = frame_count_to_write * constants::AUDIOPASS_BYTES_PER_FRAME;
                    let output_sample_count = frame_count_to_write * constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT as usize;
                    let available_in_ring_sample_count = data.audio_consumer.occupied_len();
                    let sample_count_to_pop_from_ring =
                        output_sample_count.min(available_in_ring_sample_count - (available_in_ring_sample_count % constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT as usize));

                    let popped_sample_count = data.audio_consumer.pop_slice(&mut data.output_sample_buffer[..sample_count_to_pop_from_ring]);

                    let gain = data.gain_percent.load(Ordering::Relaxed) as f32 / 100.0;
                    for sample_idx in 0..popped_sample_count {
                        let byte_idx = sample_idx * constants::AUDIOPASS_BYTES_PER_SAMPLE;
                        bytes_in_buffer[byte_idx..(byte_idx + constants::AUDIOPASS_BYTES_PER_SAMPLE)].copy_from_slice(&(data.output_sample_buffer[sample_idx] * gain).to_le_bytes());
                    }

                    // fill with silence if ring didnt have enough complete frames.
                    bytes_in_buffer[(popped_sample_count * constants::AUDIOPASS_BYTES_PER_SAMPLE)..output_size_in_bytes].fill(0);
                    output_size_in_bytes
                } else {
                    0
                };

                // marking valid region for pipewire after writing (last part of on_process())
                let chunk = buffer_data.chunk_mut();
                *chunk.offset_mut() = 0;
                *chunk.stride_mut() = constants::AUDIOPASS_BYTES_PER_FRAME as i32;
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
                None => return Err("Failed to serialize libspa POD while creating virtual mic stream".to_string()),
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
