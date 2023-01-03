use crate::bytes::NibbleStream;
use crate::notes::{Duration, Note, Pitch};

#[derive(Debug)]
pub struct Melody {
    nibbles: NibbleStream<1>,
    prev_interval: Option<i32>,
}

impl Melody {
    pub fn new(input: &[u8]) -> Self {
        Self {
            nibbles: NibbleStream::new(input),
            prev_interval: None,
        }
    }

    pub fn update_input(&mut self, input: &[u8]) {
        self.nibbles = self.nibbles.with_new_data(input);
    }

    pub fn next(&mut self, base: Pitch, current: Note) -> Note {
        let nib = self.nibbles.next_nibble();
        let current_num = current
            .pitch
            .and_then(|pitch| base.minor_scale_number(pitch));

        let (pitch, duration) = if nib & 8 == 0 {
            let mut diff = match nib & 3 {
                0b00 | 0b01 => 1,
                0b10 => 2,
                0b11 => 3,
                4.. => unreachable!(),
            };

            if nib & 4 != 0 {
                diff = -diff;
            }

            let pitch = current_num
                .map(|num| num as i32 + diff)
                .map(|num| (num, base.in_minor(num)));

            (pitch, current.duration)
        } else {
            let pitch = match nib >> 2 {
                0b10 => current
                    .pitch
                    .and_then(|pitch| current_num.map(|num| (num as i32, pitch))),

                0b11 => current
                    .pitch
                    .and_then(|pitch| base.minor_scale_number(pitch))
                    .and_then(|num| self.prev_interval.map(|interval| num as i32 - interval))
                    .map(|num| (num, base.in_minor(num))),

                _ => unreachable!(),
            };

            let duration = Duration::EIGHT;
            let duration = match nib & 3 {
                0b00 => duration,
                0b01 => duration.dotted(),
                0b10 => 2 * duration,
                0b11 => (2 * duration).dotted(),

                _ => unreachable!(),
            };

            (pitch, duration)
        };

        if let Some((degree, _)) = pitch {
            if let Some(prev_degree) = current_num {
                self.prev_interval = Some(degree - prev_degree as i32);
            }
        }

        Note {
            pitch: pitch.map(|(_, pitch)| pitch),
            duration,
        }
    }
}
