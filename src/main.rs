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

    let (recv, send) = channel_starting_with(String::new());

    let input_thread = thread::spawn(|| gui::Gui::run(send));

    let player_thread = thread::spawn(|| {
        let poll = gui::InputPoller::new(recv);
        let aio = aio::play_audio().unwrap();
        player::play(aio.audio_in, aio.sample_rate, poll);
    });

    match input_thread.join() {
        Ok(Ok(()) | Err(gui::GuiError::Interrupted)) => {}
        Ok(Err(e)) => Err(e).unwrap(),
        Err(e) => Err(e).unwrap(),
    }
}
