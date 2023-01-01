use rtrb::Producer;
use single_value_channel::Updater;

use crate::aio::BUFFER_SIZE;
use crate::bytes::NibbleStream;
use crate::delay::Delay;
use crate::envelope::AttackDecay;
use crate::float::Float;
use crate::gui::InputPoller;
use crate::notes::Duration;
use crate::sampler::Sampler;
use crate::voice::VoiceGroup;
use crate::wavetable::Wavetable;

pub const BPM: usize = 100;

pub fn play(
    mut audio_channel: Producer<f32>,
    sample_rate: usize,
    mut input: InputPoller,
    wt_send: Updater<Vec<u8>>,
) {
    let samples_per_duration = samples_per_duration(sample_rate, BPM);
    let seconds_per_sample = 1.0 / sample_rate as f64;

    let sampler = Sampler::new(sample_rate);

    let mut delay1 = Delay::new(2_000, 0.9, 0.7, 0.3);
    let mut delay2 = Delay::new(15_000, 0.8, 0.8, 0.2);
    let mut delay3 = Delay::new(40_000, 0.7, 0.9, 0.1);

    let mut wt = Wavetable::<50>::new_sine();
    let mut y = Float::new();

    wt_send.update(wt.slice(y.sample()).to_vec()).unwrap();

    let data = input.poll().unwrap_or("").as_bytes();
    let mut y_nibbles = NibbleStream::<5>::new(data);
    let mut note_nibbles = NibbleStream::<3>::new(data);
    let mut wt_nibbles = NibbleStream::<1>::new(data);

    let mut voices = VoiceGroup::new(8, AttackDecay::new(0.05, 0.4));
    let note = note_nibbles.next_note();
    let mut duration = note.duration;
    voices.add(note);

    let mut buffer = [0.0; BUFFER_SIZE];

    let mut sample_no = 0;
    loop {
        for _ in 0..samples_per_duration / BUFFER_SIZE {
            buffer.fill(0.0);

            let wt_y = y.sample();

            for voice in voices.iter_mut() {
                if let Some(pitch) = voice.pitch() {
                    let frequency = pitch.as_frequency();

                    for (i, buf) in buffer.iter_mut().enumerate() {
                        let sample = sampler.sample(&wt, frequency, sample_no + i, wt_y);
                        let gain = voice.env();
                        voice.step(seconds_per_sample);
                        *buf += sample * gain;
                    }
                }
            }

            sample_no += buffer.len();

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

        y.add(0.01 * y_nibbles.next_coarse_float());
        wt.execute(wt_nibbles.next_instruction());
        wt.increment();

        for voice in voices.iter_mut() {
            voice.delta_step();
        }

        if let Some(new) = duration.decrement() {
            duration = new;
        } else {
            let note = note_nibbles.next_note();
            duration = note.duration;
            voices.add(note);
        }

        if let Some(data) = input.poll() {
            let data = data.as_bytes();
            note_nibbles = note_nibbles.with_new_data(data);
            y_nibbles = y_nibbles.with_new_data(data);
            wt_nibbles = wt_nibbles.with_new_data(data);
        }

        wt_send.update(wt.slice(y.sample())).unwrap();
    }
}

/// Get the number of samples
fn samples_per_duration(sample_rate: usize, bpm: usize) -> usize {
    let seconds = Duration::DELTA.as_time(bpm);
    (sample_rate as f64 * seconds) as usize
}
