pub const AUDIOPASS_APP_ID: &str = "com.nithinrdy.audiopass";
pub const AUDIOPASS_APP_NAME: &str = "AudioPass";

pub const AUDIOPASS_PIPEWIRE_CLIENT_NAME: &str = "AudioPass Pipewire Client";

pub const AUDIOPASS_VIRTUAL_MIC_STREAM_NAME: &str = "AudioPass Virtual Microphone Stream";
pub const AUDIOPASS_VIRTUAL_MIC_DESCRIPTION: &str = "AudioPass Virtual Microphone";
pub const AUDIOPASS_VIRTUAL_MIC_IDENTIFIER_PREFIX: &str = "audiopass.virtual-mic";

pub const AUDIOPASS_VIRTUAL_CAPTURE_STREAM_NAME: &str = "AudioPass Virtual Capture Stream";

pub const AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT: u8 = 2;
pub const AUDIOPASS_VIRTUAL_MIC_SAMPLE_RATE: u16 = 48000;

pub const AUDIOPASS_BYTES_PER_SAMPLE: usize = size_of::<f32>();
pub const AUDIOPASS_BYTES_PER_FRAME: usize = AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT as usize * AUDIOPASS_BYTES_PER_SAMPLE;
pub const AUDIOPASS_MIC_RING_CAPACITY_IN_SAMPLES: usize = (AUDIOPASS_VIRTUAL_MIC_SAMPLE_RATE / 10) as usize // 100ms worth of samples
  * AUDIOPASS_VIRTUAL_MIC_CHANNEL_COUNT as usize;
