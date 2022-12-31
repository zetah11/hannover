#![allow(unused)]

mod aio;
mod bytes;
mod data;
mod delay;
mod envelope;
mod float;
mod gui;
mod math;
mod notes;
mod player;
mod sampler;
mod structures;
mod wavetable;

fn main() {
    pretty_env_logger::init();

    let input = gui::Gui::run().unwrap();
    let aio = aio::play_audio().unwrap();

    player::play(aio.audio_in, aio.sample_rate, input.as_bytes());
}
