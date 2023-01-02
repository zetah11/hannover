mod aio;
mod bytes;
mod data;
mod delay;
mod envelope;
mod float;
mod gui;
mod markov;
mod math;
mod notes;
mod player;
mod sampler;
mod sequence;
mod source;
mod structures;
mod voice;
mod wavetable;

use std::thread;

use single_value_channel::channel_starting_with;

fn main() {
    pretty_env_logger::init();

    let (recv, send) = channel_starting_with(String::new());
    let (wt_recv, wt_send) = channel_starting_with(vec![]);

    let poll = gui::InputPoller::new(recv);
    let wt_poll = gui::WavetablePoller::new(wt_recv);

    let input_thread = thread::spawn(|| gui::Gui::run(send, wt_poll));

    let _player_thread = thread::spawn(|| {
        let aio = aio::play_audio().unwrap();
        player::play(aio.audio_in, aio.sample_rate, poll, wt_send);
    });

    match input_thread.join() {
        Ok(Ok(()) | Err(gui::GuiError::Interrupted)) => {}
        Ok(Err(e)) => Err(e).unwrap(),
        Err(e) => Err(e).unwrap(),
    }
}
