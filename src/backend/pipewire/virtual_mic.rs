// mostly from this example: https://sources.debian.org/src/rust-pipewire/0.9.2-2/examples/audio-capture.rs/#L43.
// I honestly do not understand the PodSerializer::serialize() part.

use pipewire::spa::{
    param::audio::{AudioFormat, AudioInfoRaw},
    pod::{Pod, serialize::PodSerializer},
    sys::{SPA_AUDIO_CHANNEL_FL, SPA_AUDIO_CHANNEL_FR},
    utils::Direction,
};

use crate::backend::{constants, pipewire::{CustomEventSender, PipewireEvent}};
use std::{io::Cursor, ops::Deref};

struct AdditionalData {
    sender: CustomEventSender,
}

pub struct VirtualMic {
    _stream: pipewire::stream::StreamRc,
    _callback_listener: pipewire::stream::StreamListener<AdditionalData>,
}

impl VirtualMic {
    pub fn new(pw_core: pipewire::core::CoreRc, state_change_sender: CustomEventSender) -> Result<Self, String> {
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
            .add_local_listener_with_user_data(AdditionalData { sender: state_change_sender })
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
                // todo
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

        // match state_change_receiver.recv() {
        //     Ok(s) => (if let Err(e) = s { Err(format!("Failed to create virtual mic: {}", e)) } else { Ok(()) }),
        //     Err(e) => Err(format!("Virtual mic creation terminated unexpectedly: {}", e)),
        // }?;

        Ok(VirtualMic {
            _stream: stream,
            _callback_listener: listener,
        })
    }
}

fn get_serialized_vec_for_pod() -> Result<Vec<u8>, String> {
    let mut audio_info = AudioInfoRaw::default();
    audio_info.set_format(AudioFormat::F32LE);
    audio_info.set_rate(constants::AUDIOPASS_VIRTUAL_MIC_SAMPLE_RATE.into());
    audio_info.set_channels(constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT.into());

    let mut pos = [0u32; 64];
    pos[0] = SPA_AUDIO_CHANNEL_FL;
    pos[1] = SPA_AUDIO_CHANNEL_FR;
    audio_info.set_position(pos);

    let vec: Vec<u8> = vec![0; 1024];
    match PodSerializer::serialize(
        Cursor::new(vec),
        &(pipewire::spa::pod::Value::Object(pipewire::spa::pod::Object {
            type_: pipewire::spa::utils::SpaTypes::ObjectParamFormat.as_raw(),
            id: pipewire::spa::param::ParamType::EnumFormat.as_raw(),
            properties: audio_info.into(),
        })),
    ) {
        Ok(cursor) => Ok(cursor.0.into_inner()),
        Err(e) => Err(format!("Failed to serialize libspa POD while creating virtual mic stream: {}", e)),
    }
}
