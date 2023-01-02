use crate::bytes::NibbleStream;
use crate::envelope::AttackDecay;
use crate::float::Float;
use crate::notes::{Duration, Pitch};
use crate::sampler::Sampler;
use crate::source::NoteSource;
use crate::voice::VoiceGroup;
use crate::wavetable::Wavetable;

pub const BASE: Pitch = Pitch::A2;

pub struct Performer<const S: usize> {
    source: NoteSource,

    table: Wavetable<S>,
    table_nibbles: NibbleStream<1>,

    y: Float,
    y_nibbles: NibbleStream<5>,

    voices: VoiceGroup,
    duration: Duration,
}

impl<const S: usize> Performer<S> {
    pub fn new(input: &[u8], env: AttackDecay) -> Self {
        Self {
            source: NoteSource::new(input),

            table: Wavetable::new_sine(),
            table_nibbles: NibbleStream::new(input),

            y: Float::new(),
            y_nibbles: NibbleStream::new(input),

            voices: VoiceGroup::new(8, env),
            duration: Duration::DELTA,
        }
    }

    pub fn slice(&self) -> Vec<u8> {
        self.table.slice(self.y.sample())
    }

    pub fn update(&mut self) {
        self.y.add(0.01 * self.y_nibbles.next_coarse_float());
        self.table.execute(self.table_nibbles.next_instruction());
        self.table.increment();

        for voice in self.voices.iter_mut() {
            voice.delta_step();
        }

        if let Some(new) = self.duration.decrement() {
            self.duration = new;
        } else {
            let note = self.source.next(BASE);
            self.duration = note.duration;
            self.voices.add(note);
        }
    }

    pub fn update_input(&mut self, input: &[u8]) {
        self.source.update_input(input);
        self.table_nibbles = self.table_nibbles.with_new_data(input);
        self.y_nibbles = self.y_nibbles.with_new_data(input);
    }

    /// Sample this performer in the given buffer.
    pub fn sample_in(&mut self, sampler: &Sampler, buffer: &mut [f64]) {
        let y = self.y.sample();
        let by = sampler.seconds_per_sample();

        for voice in self.voices.iter_mut() {
            if let Some(pitch) = voice.pitch() {
                let frequency = pitch.as_frequency();

                for (i, buf) in buffer.iter_mut().enumerate() {
                    let sample = sampler.sample(&self.table, frequency, i, y);
                    let gain = voice.env();
                    voice.step(by);
                    *buf += sample * gain;
                }
            }
        }
    }
}
