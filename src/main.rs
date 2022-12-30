#![allow(unused)]

mod aio;
mod bytes;
mod data;
mod float;
mod math;
mod notes;
mod sampler;
mod wavetable;

use std::thread;
use std::time::Duration;

fn main() {
    pretty_env_logger::init();

    let frequency = (notes::pitch::Pitch::C4 - notes::pitch::Octave).as_frequency();

    let wt = wavetable::Wavetable::<500>::new_sine();
    let (mut producer, _stream) = aio::play_audio().unwrap();

    for i in 0..sampler::SAMPLE_RATE / aio::BUFFER_SIZE {
        let chunk = loop {
            if let Ok(chunk) = producer.write_chunk_uninit(aio::BUFFER_SIZE) {
                break chunk;
            }
        };

        chunk.fill_from_iter(
            (0..aio::BUFFER_SIZE)
                .map(|j| sampler::sample(&wt, frequency, i * aio::BUFFER_SIZE + j, 0.0) as f32),
        );
    }
}
