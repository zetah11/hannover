use crate::bytes::NibbleStream;
use crate::markov::MarkovMelody;
use crate::notes::{Duration, Note, Pitch};
use crate::sequence::{Direction, Sequence};

#[derive(Debug)]
pub struct NoteSource {
    note_nibbles: NibbleStream<3>,
    random_nibbles: NibbleStream<2>,
    state_nibbles: NibbleStream<1>,

    arp: Sequence,
    chain: MarkovMelody,
    prev: Option<Note>,

    count: usize,
    state: u8,
}

impl NoteSource {
    pub fn new(input: &[u8]) -> Self {
        Self {
            note_nibbles: NibbleStream::new(input),
            random_nibbles: NibbleStream::new(input),
            state_nibbles: NibbleStream::new(input),

            arp: Sequence::new(vec![], Direction::Up),
            chain: MarkovMelody::new(),
            prev: None,

            count: 0,
            state: 0,
        }
    }

    pub fn next(&mut self, base: Pitch) -> Note {
        let (next, added_to_chain) = match (self.prev, self.state) {
            (Some(_), 0 | 3 | 5 | 0xa | 0xc | 0xf) | (None, _) => {
                (self.note_nibbles.next_note(base), false)
            }

            (Some(prev), 1 | 4 | 7 | 9 | 0xd) => {
                let [a, b] = self.random_nibbles.next_nibbles();
                let random = (a << 4) | b;

                match self.chain.next(prev, random) {
                    Some(note) => (note, true),
                    None => (self.note_nibbles.next_note(base), true),
                }
            }

            (Some(_), 2 | 6 | 8 | 0xb | 0xe) => match self.arp.next() {
                Some(note) => (note, false),
                None => (self.note_nibbles.next_note(base), false),
            },

            (_, 16..=u8::MAX) => unreachable!(),
        };

        if !added_to_chain {
            let [a, b] = self.random_nibbles.next_nibbles();
            let random = (a << 4) | b;
            let _ = self.chain.next(next, random);
        }

        if self.count % 5 == 0 {
            if let Some(pitch) = next.pitch {
                let dir = match self.state >> 2 {
                    0b00 => Direction::Up,
                    0b01 => Direction::Down,
                    0b10 => Direction::PingPong,
                    0b11 => Direction::Up,
                    _ => unreachable!(),
                };

                if let Some(arp) = Sequence::new_arp(base, pitch, dir, Duration::EIGHT) {
                    self.arp = arp;
                }
            }
        }

        if self.count % 7 == 0 {
            self.state = self.state_nibbles.next_nibble();
        }

        self.count = self.count.wrapping_add(1);

        self.prev = Some(next);
        next
    }

    pub fn update_input(&mut self, input: &[u8]) {
        self.note_nibbles = self.note_nibbles.with_new_data(input);
        self.random_nibbles = self.random_nibbles.with_new_data(input);
        self.state_nibbles = self.state_nibbles.with_new_data(input);
    }
}
