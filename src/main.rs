#![allow(unused)]

mod aio;
mod bytes;
mod data;
mod envelope;
mod float;
mod math;
mod notes;
mod sampler;
mod wavetable;

use std::thread;
use std::time::Duration;

fn main() {
    pretty_env_logger::init();

    let frequency = notes::pitch::Pitch::C4.as_frequency();

    let wt = wavetable::Wavetable::<3>::new_sine();
    let mut env = envelope::AttackDecay::new(0.02, 0.3);
    let (mut producer, _stream) = aio::play_audio().unwrap();

    let n = 4.0 * sampler::SAMPLE_RATE as f64;
    let seconds_per_sample = 1.0 / sampler::SAMPLE_RATE as f64;

    for i in 0..4 * sampler::SAMPLE_RATE / aio::BUFFER_SIZE {
        let chunk = loop {
            if let Ok(chunk) = producer.write_chunk_uninit(aio::BUFFER_SIZE) {
                break chunk;
            }
        };

        chunk.fill_from_iter((0..aio::BUFFER_SIZE).map(|j| {
            let sample_no = i * aio::BUFFER_SIZE + j;
            let y = sample_no as f64 / n;
            let sample = sampler::sample(&wt, frequency, i * aio::BUFFER_SIZE + j, y);
            let gain = env.value();
            env.step(seconds_per_sample);
            (sample * gain) as f32
        }));
    }
}
