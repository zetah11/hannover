//! Audio I/O (but mostly just O).

use anyhow::anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, OutputCallbackInfo, SampleFormat, Stream, StreamConfig};
use log::warn;
use rtrb::{Consumer, Producer, RingBuffer};

pub const BUFFER_SIZE: usize = 256;

pub struct AudioIo {
    pub sample_rate: usize,
    pub audio_in: Producer<f32>,
    pub stream: Stream,
}

pub fn play_audio() -> anyhow::Result<AudioIo> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("no output device"))?;

    let mut supported_config = device.supported_output_configs()?;
    let supported_config = supported_config
        .next()
        .ok_or_else(|| anyhow!("no supported configurations"))?
        .with_max_sample_rate();

    assert_eq!(SampleFormat::F32, supported_config.sample_format());

    let config = supported_config.config();
    let config = StreamConfig {
        buffer_size: BufferSize::Fixed(BUFFER_SIZE as u32),
        ..config
    };

    let (audio_in, audio_out) = RingBuffer::new(16 * BUFFER_SIZE);

    let stream = device.build_output_stream(&config, make_data_callback(audio_out), |err| {
        eprintln!("error playing audio: {err}")
    })?;

    stream.play()?;

    Ok(AudioIo {
        sample_rate: config.sample_rate.0 as usize,
        audio_in,
        stream,
    })
}

fn make_data_callback(
    mut audio_out: Consumer<f32>,
) -> impl FnMut(&mut [f32], &OutputCallbackInfo) + Send + 'static {
    move |buffer, _info| {
        for buf in buffer.iter_mut() {
            *buf = audio_out.pop().unwrap_or_else(|_| {
                warn!("ring buffer empty");
                0.0
            });
        }
    }
}
