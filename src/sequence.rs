use itertools::Itertools;

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

#[derive(Clone, Debug)]
pub struct Sequence {
    notes: Vec<Note>,
    at: usize,
}

impl Sequence {
    pub fn new(notes: Vec<Note>) -> Self {
        assert!(!notes.is_empty());
        Self { notes, at: 0 }
    }

    /// Generate an arpeggiating sequence, given a `note` in the pentatonic
    /// minor scale starting at `base`.
    pub fn new_arp(base: Pitch, note: Pitch, duration: Duration) -> Option<Self> {
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
        })
    }

    pub fn next_note(&mut self) -> Note {
        let current = self.notes.get(self.at).unwrap();
        self.at = self.at.wrapping_add(1) % self.notes.len();
        *current
    }
}

impl Iterator for Sequence {
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_note())
    }
}
