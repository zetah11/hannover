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

use std::thread;

use single_value_channel::channel_starting_with;

fn main() {
    pretty_env_logger::init();

    thread::scope(|s| {
        let (recv, send) = channel_starting_with(String::new());

        s.spawn(|| {
            gui::Gui::run(send).unwrap();
        });

        let poll = gui::InputPoller::new(recv);

        let aio = aio::play_audio().unwrap();

        player::play(aio.audio_in, aio.sample_rate, poll);
    })
}
