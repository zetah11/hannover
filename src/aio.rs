//! Audio I/O (but mostly just O).

use anyhow::anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, Data, OutputCallbackInfo, Sample, SampleFormat, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};

pub const BUFFER_SIZE: usize = 256;

pub fn play_audio() -> anyhow::Result<(Producer<f32>, Stream)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("no output device"))?;

    let mut supported_config = device.supported_output_configs()?;
    let supported_config = supported_config
        .next()
        .ok_or_else(|| anyhow!("no supported configurations"))?
        .with_max_sample_rate();

    assert_eq!(
        crate::sampler::SAMPLE_RATE,
        supported_config.sample_rate().0 as usize
    );
    assert_eq!(SampleFormat::F32, supported_config.sample_format());

    let config = supported_config.config();
    let config = StreamConfig {
        buffer_size: BufferSize::Fixed(BUFFER_SIZE as u32),
        ..config
    };

    let (audio_in, audio_out) = RingBuffer::new(4 * BUFFER_SIZE);

    let stream = device.build_output_stream(&config, make_data_callback(audio_out), |err| {
        eprintln!("error playing audio: {err}")
    })?;

    stream.play()?;

    Ok((audio_in, stream))
}

fn make_data_callback(
    mut audio_out: Consumer<f32>,
) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static {
    move |buffer, info| {
        for buf in buffer.iter_mut() {
            *buf = audio_out.pop().unwrap_or(0.0);
        }
    }
}
