mod group;

pub use group::VoiceGroup;

use crate::envelope::AttackDecay;
use crate::notes::{Note, Pitch};

#[derive(Clone, Copy, Debug)]
pub struct Voice {
    note: Option<Note>,
    env: AttackDecay,
}

impl Voice {
    pub fn new(env: AttackDecay) -> Self {
        Self { note: None, env }
    }

    /// Get the current pitch for this voice, if any.
    pub fn pitch(&self) -> Option<Pitch> {
        self.note?.pitch
    }

    /// Get the envelope value for this voice.
    pub fn env(&self) -> f64 {
        self.env.value()
    }

    /// Step the envelope forward `by` seconds.
    pub fn step(&mut self, by: f64) {
        self.env.step(by);
    }

    /// Move one [`Duration::DELTA`] forwards in time. Calls `f` to provide a
    /// new note if the current one is done.
    pub fn delta_step(&mut self) {
        let Some(note) = &mut self.note else { return; };

        if let Some(duration) = note.duration.decrement() {
            note.duration = duration;
        }
    }

    pub fn is_done(&self) -> bool {
        self.env.is_done()
    }

    pub fn replace(&mut self, note: Note) {
        self.note = Some(note);
        self.env.reset();
    }
}
