use rtrb::Producer;

use crate::aio::BUFFER_SIZE;
use crate::bytes::NibbleStream;
use crate::envelope::AttackDecay;
use crate::float::Float;
use crate::notes::duration::Duration;
use crate::sampler::Sampler;
use crate::wavetable::Wavetable;

pub const BPM: usize = 100;

pub fn play(mut audio_channel: Producer<f32>, sample_rate: usize, input: &[u8]) {
    let samples_per_duration = samples_per_duration(sample_rate, BPM);
    let seconds_per_sample = 1.0 / sample_rate as f64;

    let sampler = Sampler::new(sample_rate);

    let mut env = AttackDecay::new(0.02, 0.1);
    let mut wt = Wavetable::<22>::new_sine();
    let mut y = Float::new();

    let mut y_nibbles = NibbleStream::<5>::new(input);
    let mut note_nibbles = NibbleStream::<3>::new(input);
    let mut wt_nibbles = NibbleStream::<1>::new(input);

    let mut note = note_nibbles.next_note();
    println!("{note:?}");

    let mut sample_no = 0;
    for _ in 0.. {
        for _ in 0..samples_per_duration / BUFFER_SIZE {
            let chunk = loop {
                if let Ok(chunk) = audio_channel.write_chunk_uninit(BUFFER_SIZE) {
                    break chunk;
                }
            };

            let wt_y = y.sample();

            if let Some(pitch) = note.pitch {
                let frequency = pitch.as_frequency();

                chunk.fill_from_iter((0..BUFFER_SIZE).map(|_| {
                    let sample = sampler.sample(&wt, frequency, sample_no, wt_y);
                    sample_no += 1;
                    let gain = env.value();
                    env.step(seconds_per_sample);
                    (sample * gain) as f32
                }));
            } else {
                chunk.fill_from_iter(std::iter::repeat(0.0));
            }
        }

        y.add(y_nibbles.next_coarse_float());
        wt.execute(wt_nibbles.next_instruction());

        if let Some(duration) = note.duration.decrement() {
            note.duration = duration;
        } else {
            note = note_nibbles.next_note();
            println!("{note:?}");
            env.reset();
            env.set_decay(0.5 * note.duration.as_time(BPM));
        }
    }
}

/// Get the number of samples
fn samples_per_duration(sample_rate: usize, bpm: usize) -> usize {
    let seconds = Duration::DELTA.as_time(bpm);
    (sample_rate as f64 * seconds) as usize
}
