use crate::notes::{Duration, Note, Pitch};

// 0 2  3 5 7  8 10
// C D Eb F G Ab Bb C
// C   Eb F G    Bb C
//
// C  Eb G  =  0  3  7
// Eb G  Bb =  3  7 10
// F  Ab C  =  0  5  8
// G  Bb D  =  2  7 10
// Bb D  F  =  2  5 10
const PENTATONIC_MINOR_CHORDS: [[i32; 3]; 5] =
    [[0, 3, 7], [3, 7, 10], [0, 5, 8], [2, 7, 10], [2, 5, 10]];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Up,
    Down,
    PingPong,
}

impl Direction {
    fn with_step(self) -> (Self, isize) {
        let step = match self {
            Direction::Up => 1,
            Direction::Down => -1,
            Direction::PingPong => 1,
        };

        (self, step)
    }
}

#[derive(Clone, Debug)]
pub struct Sequence {
    notes: Vec<Note>,
    at: usize,
    dir: (Direction, isize),
}

impl Sequence {
    pub fn new(notes: Vec<Note>, dir: Direction) -> Self {
        Self {
            notes,
            at: 0,
            dir: dir.with_step(),
        }
    }

    /// Generate an arpeggiating sequence, given a `note` in the pentatonic
    /// minor scale starting at `base`.
    pub fn new_arp(base: Pitch, note: Pitch, dir: Direction, duration: Duration) -> Option<Self> {
        let degree = base.pentatonic_minor_scale_number(note)?;
        let chord = PENTATONIC_MINOR_CHORDS.get(degree).unwrap();

        Some(Self {
            notes: chord
                .iter()
                .map(|n| Note {
                    pitch: Some(base + *n),
                    duration,
                })
                .collect(),
            at: 0,
            dir: dir.with_step(),
        })
    }

    pub fn next_note(&mut self) -> Option<Note> {
        let current = self.notes.get(self.at)?;
        let at = self.at as isize;
        let len = self.notes.len() as isize;
        self.at = match self.dir {
            (Direction::Up | Direction::Down, step) => {
                at.wrapping_add(step).rem_euclid(len) as usize
            }

            (Direction::PingPong, step) => {
                let step = if at + step == len || at + step == -1 {
                    -step
                } else {
                    step
                };

                self.dir.1 = step;
                at.wrapping_add(step).rem_euclid(len) as usize
            }
        };

        Some(*current)
    }
}

impl Iterator for Sequence {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_note()
    }
}
