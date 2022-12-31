use rtrb::Producer;

use crate::aio::BUFFER_SIZE;
use crate::bytes::NibbleStream;
use crate::delay::Delay;
use crate::envelope::AttackDecay;
use crate::float::Float;
use crate::gui::InputPoller;
use crate::notes::duration::Duration;
use crate::sampler::Sampler;
use crate::wavetable::Wavetable;

pub const BPM: usize = 100;

pub fn play(mut audio_channel: Producer<f32>, sample_rate: usize, mut input: InputPoller) {
    let samples_per_duration = samples_per_duration(sample_rate, BPM);
    let seconds_per_sample = 1.0 / sample_rate as f64;

    let sampler = Sampler::new(sample_rate);

    let mut delay1 = Delay::new(5_000, 0.9, 1.0, 0.1);
    let mut delay2 = Delay::new(20_000, 0.8, 0.9, 0.7);

    let mut env = AttackDecay::new(0.02, 0.04);
    let mut wt = Wavetable::<50>::new_sine();
    let mut y = Float::new();

    let data = input.poll().unwrap_or("").as_bytes();
    let mut y_nibbles = NibbleStream::<5>::new(data);
    let mut note_nibbles = NibbleStream::<3>::new(data);
    let mut wt_nibbles = NibbleStream::<1>::new(data);

    let mut note = note_nibbles.next_note();

    let mut buffer = [0.0; BUFFER_SIZE];

    let mut sample_no = 0;
    loop {
        for _ in 0..samples_per_duration / BUFFER_SIZE {
            let wt_y = y.sample();

            if let Some(pitch) = note.pitch {
                let frequency = pitch.as_frequency();

                for buf in buffer.iter_mut() {
                    let sample = sampler.sample(&wt, frequency, sample_no, wt_y);
                    sample_no += 1;
                    let gain = env.value();
                    env.step(seconds_per_sample);
                    *buf = sample * gain;
                }
            } else {
                buffer.fill(0.0);
            }

            let chunk = loop {
                if let Ok(chunk) = audio_channel.write_chunk_uninit(BUFFER_SIZE) {
                    break chunk;
                }
            };

            chunk.fill_from_iter(buffer.iter().copied().map(|sample| {
                let sample = delay1.process(sample);
                let sample = delay2.process(sample);
                sample as f32
            }));
        }

        y.add(y_nibbles.next_coarse_float());
        wt.execute(wt_nibbles.next_instruction());

        if let Some(duration) = note.duration.decrement() {
            note.duration = duration;
        } else {
            note = note_nibbles.next_note();
            env.reset();
        }

        if let Some(data) = input.poll() {
            let data = data.as_bytes();
            note_nibbles = note_nibbles.with_new_data(data);
            y_nibbles = y_nibbles.with_new_data(data);
            wt_nibbles = wt_nibbles.with_new_data(data);
        }
    }
}

/// Get the number of samples
fn samples_per_duration(sample_rate: usize, bpm: usize) -> usize {
    let seconds = Duration::DELTA.as_time(bpm);
    (sample_rate as f64 * seconds) as usize
}
