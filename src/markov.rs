use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

use ordered_float::OrderedFloat;

use crate::notes::{Duration, Note, Pitch};

#[derive(Debug, Default)]
pub struct MarkovMelody {
    pitches: HashMap<Option<Pitch>, HashMap<Option<Pitch>, usize>>,
    durations: HashMap<Duration, HashMap<Duration, usize>>,
    prev: Option<Note>,
}

impl MarkovMelody {
    pub fn new() -> Self {
        Self {
            pitches: HashMap::new(),
            durations: HashMap::new(),
            prev: None,
        }
    }

    /// Get the next note given the `current` note and a `random` two nibble
    /// value. Returns `None` if this chain does not have enough in its memory
    /// to make a decision.
    pub fn next(&mut self, current: Note, random: u8) -> Option<Note> {
        if let Some(prev) = self.prev {
            *self
                .pitches
                .entry(prev.pitch)
                .or_default()
                .entry(current.pitch)
                .or_default() += 1;

            *self
                .durations
                .entry(prev.duration)
                .or_default()
                .entry(current.duration)
                .or_default() += 1;
        }

        self.prev = Some(current);

        let pitch = {
            let pitch_index = random & 0xf;

            let choices = self.pitches.get(&current.pitch)?;
            let choices = fair_chance_array(choices);
            choices[pitch_index as usize]
        };

        let duration = {
            let duration_index = random >> 4;

            let choices = self.durations.get(&current.duration)?;
            let choices = fair_chance_array(choices);
            choices[duration_index as usize]
        };

        Some(Note { pitch, duration })
    }
}

/// Build a 16-element array of `T`s from `choices`, where each `choice` appears
/// approximately proportionally to their proportion of the sum of the values in
/// `choices`. This is a "fair chance" algorithm, because even if a key occupies
/// a very low proportion, it may be given at least one of the array spots. This
/// way, the most frequent choices won't dominate the array.
///
/// Panics if `choices` is empty.
fn fair_chance_array<T: Copy + Debug + Eq + Hash>(choices: &HashMap<T, usize>) -> [T; 16] {
    assert!(!choices.is_empty());

    let factor = 16.0 / choices.values().copied().sum::<usize>() as f64;

    let mut sorted: Vec<_> = choices
        .iter()
        .map(|(k, v)| (*k, OrderedFloat(factor * *v as f64)))
        .collect();

    sorted.sort_by_key(|(_, count)| *count);

    let mut res = Vec::with_capacity(16);
    let mut index = 0;
    let mut counts: HashMap<T, usize> = HashMap::new();

    for (item, _) in sorted.iter() {
        counts.insert(*item, 0);
    }

    while res.len() < 16 {
        let (item, OrderedFloat(min_count)) = sorted[index];
        let count = counts.get_mut(&item).unwrap();

        index = (index + 1) % sorted.len();

        if (*count as f64) < min_count {
            *count += 1;
            res.push(item);
        }
    }

    let res: Box<[T; 16]> = res.into_boxed_slice().try_into().unwrap();
    *res
}
