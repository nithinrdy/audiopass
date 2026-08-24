use std::io::Cursor;
use pipewire::spa::{
    self,
    param::audio::{AudioFormat, AudioInfoRaw},
    pod::serialize::PodSerializer,
};

use crate::backend::constants;

pub fn get_serialized_vec_for_pod() -> Result<Vec<u8>, String> {
    let mut audio_info = AudioInfoRaw::default();
    audio_info.set_format(AudioFormat::F32LE);
    audio_info.set_rate(constants::AUDIOPASS_VIRTUAL_MIC_SAMPLE_RATE.into());
    audio_info.set_channels(constants::AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT.into());

    let mut pos = [0u32; 64];
    pos[0] = spa::sys::SPA_AUDIO_CHANNEL_FL;
    pos[1] = spa::sys::SPA_AUDIO_CHANNEL_FR;
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
