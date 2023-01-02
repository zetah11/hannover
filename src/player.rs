use rtrb::Producer;
use single_value_channel::Updater;

use crate::aio::BUFFER_SIZE;
use crate::delay::Delay;
use crate::envelope::AttackDecay;
use crate::gui::InputPoller;
use crate::notes::Duration;
use crate::performer::Performer;
use crate::sampler::Sampler;

pub const BPM: usize = 100;

pub fn play(
    mut audio_channel: Producer<f32>,
    sample_rate: usize,
    mut input: InputPoller,
    wt_send: Updater<Vec<u8>>,
) {
    let samples_per_duration = samples_per_duration(sample_rate, BPM);

    let mut sampler = Sampler::new(sample_rate);

    let mut delay1 = Delay::new(2_000, 0.9, 0.8, 0.2);
    let mut delay2 = Delay::new(15_000, 0.8, 0.7, 0.3);
    let mut delay3 = Delay::new(40_000, 0.7, 0.6, 0.4);

    let data = input.poll().unwrap_or("").as_bytes();
    let mut performer = Performer::<50>::new(data, AttackDecay::new(0.05, 0.4));

    wt_send.update(performer.slice()).unwrap();

    let mut buffer = [0.0; BUFFER_SIZE];

    loop {
        for _ in 0..samples_per_duration / BUFFER_SIZE {
            buffer.fill(0.0);
            performer.sample_in(&sampler, &mut buffer);
            sampler.step(buffer.len());

            let chunk = loop {
                if let Ok(chunk) = audio_channel.write_chunk_uninit(BUFFER_SIZE) {
                    break chunk;
                }
            };

            chunk.fill_from_iter(buffer.iter().copied().map(|sample| {
                let sample = delay1.process(sample);
                let sample = delay2.process(sample);
                let sample = delay3.process(sample);
                sample as f32
            }));
        }

        performer.update();

        if let Some(data) = input.poll() {
            performer.update_input(data.as_bytes());
        }

        wt_send.update(performer.slice()).unwrap();
    }
}

/// Get the number of samples
fn samples_per_duration(sample_rate: usize, bpm: usize) -> usize {
    let seconds = Duration::DELTA.as_time(bpm);
    (sample_rate as f64 * seconds) as usize
}
