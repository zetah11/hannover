#![allow(unused)]

mod aio;
mod bytes;
mod data;
mod delay;
mod envelope;
mod float;
mod math;
mod notes;
mod player;
mod sampler;
mod structures;
mod wavetable;

use std::thread;
use std::time::Duration;

fn main() {
    pretty_env_logger::init();

    let frequency = notes::pitch::Pitch::C4.as_frequency();

    let mut wt = wavetable::Wavetable::<50>::new_sine();
    let mut env = envelope::AttackDecay::new(0.02, 1.5);

    let aio = aio::play_audio().unwrap();

    player::play(aio.audio_in, aio.sample_rate, "h".as_bytes());
}
