mod duration;
mod pitch;

pub use self::duration::Duration;
pub use self::pitch::{Octave, Pitch};

use crate::bytes::NibbleStream;

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub pitch: Option<Pitch>,
    pub duration: Duration,
}

impl NibbleStream<3> {
    pub fn next_note(&mut self, base: Pitch) -> Note {
        let [a, b, c] = self.next_nibbles();
        let noisy = a != 0;

        let mut duration = Duration::SIXTEENTH;

        if b & 8 != 0 {
            duration *= 3;
        }

        if b & 4 != 0 {
            duration *= 2;
        }

        if b & 1 != 0 {
            duration = duration.dotted();
        }

        let pitch = (c & 7) as i32 + if c & 8 != 0 { -8 } else { 0 };
        let pitch = base.in_pentatonic_minor(pitch);

        Note {
            pitch: noisy.then_some(pitch),
            duration,
        }
    }
}
