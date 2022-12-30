pub mod duration;
pub mod pitch;

use crate::bytes::NibbleStream;

use duration::Duration;
use pitch::Pitch;

#[derive(Clone, Copy, Debug)]
pub struct Note {
    pub pitch: Option<Pitch>,
    pub duration: Duration,
}

impl NibbleStream<3> {
    pub fn next_note(&mut self) -> Note {
        let [a, b, c] = self.next_nibbles();
        let noisy = a & 4 != 0 || a & 2 != 0;

        let mut duration = Duration::SIXTEENTH;

        if b & 8 != 0 {
            duration *= 2;
        }

        if b & 1 != 0 {
            duration = duration.dotted();
        }

        let pitch = (a & 7) as i32;
        let pitch = if a & 8 != 0 {
            Pitch::C4.in_minor(pitch)
        } else {
            Pitch::C4.in_major(pitch)
        };

        Note {
            pitch: noisy.then_some(pitch),
            duration,
        }
    }
}
