use crate::envelope::AttackDecay;
use crate::notes::Note;

use super::Voice;

#[derive(Debug)]
pub struct VoiceGroup {
    voices: Vec<Voice>,
    at: usize,
}

impl VoiceGroup {
    pub fn new(voices: usize, env: AttackDecay) -> Self {
        Self {
            voices: vec![Voice::new(env); voices],
            at: 0,
        }
    }

    /// Add a note to one of the voices in this group.
    pub fn add(&mut self, note: Note) {
        if let Some(voice) = self.voices.iter_mut().find(|voice| voice.is_done()) {
            voice.replace(note);
        } else if !self.voices.is_empty() {
            self.voices.get_mut(self.at).unwrap().replace(note);
            self.at = self.at.wrapping_add(1) % self.voices.len();
        }
    }

    /// Get an iterator over all the non-silent voices in this group.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Voice> {
        self.voices
            .iter_mut()
            .filter_map(|voice| (!voice.is_done()).then_some(voice))
    }
}
