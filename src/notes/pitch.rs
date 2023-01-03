use std::ops::Add;
use std::ops::Sub;

pub const PENTATONIC_MINOR: [i32; 5] = [3, 2, 2, 3, 2];
pub const MINOR: [i32; 7] = [2, 1, 2, 2, 1, 2, 2];

/// The base frequency, in Hertz, of A4.
const A4: f64 = 440.0;
const TWELFTH_ROOT_TWO: f64 = 1.059_463_094_359_295_3;

/// A pitch is an exponential frequency. Represented as a semi-tone offset from
/// A4.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pitch(i32);

impl Pitch {
    pub const A2: Pitch = Pitch(-24);

    /// Get the frequency, in Hz, of this pitch.
    pub fn as_frequency(&self) -> f64 {
        A4 * (TWELFTH_ROOT_TWO.powi(self.0))
    }

    pub fn minor_scale_number(&self, pitch: Pitch) -> Option<usize> {
        self.scale_number(pitch, &MINOR)
    }

    pub fn pentatonic_minor_scale_number(&self, pitch: Pitch) -> Option<usize> {
        self.scale_number(pitch, &PENTATONIC_MINOR)
    }

    pub fn in_minor(&self, n: i32) -> Pitch {
        self.in_scale(n, &MINOR)
    }

    pub fn in_pentatonic_minor(&self, n: i32) -> Pitch {
        self.in_scale(n, &PENTATONIC_MINOR)
    }

    /// Get the pitch that is the `n`th note in the given scale with this note
    /// as its base.
    pub fn in_scale(&self, n: i32, scale: &[i32]) -> Pitch {
        let (count, stride) = if n < 0 {
            (-n as usize, -1)
        } else {
            (n as usize, 1)
        };

        let mut step_index: i32 = if n < 0 { (scale.len() - 1) as i32 } else { 0 };
        let mut pitch_offset = 0;
        for _ in 0..count {
            let step = scale[step_index as usize];
            step_index = (step_index + stride).rem_euclid(scale.len() as i32);
            pitch_offset += stride * step;
        }

        Pitch(self.0 + pitch_offset)
    }

    /// Get the scale number for the given pitch in the scale with this one as
    /// its base note.
    pub fn scale_number(&self, mut pitch: Pitch, scale: &[i32]) -> Option<usize> {
        pitch.0 -= 12 * ((pitch.0 - 12) / 12);

        let mut step_index = 0;
        let mut pitch_offset = self.0;

        while pitch_offset < pitch.0 {
            let step = scale[step_index];
            step_index = step_index.wrapping_add(1) % scale.len();
            pitch_offset += step;
        }

        if pitch_offset == pitch.0 {
            Some(step_index)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Octave;

impl Add<i32> for Pitch {
    type Output = Self;

    /// Get the pitch `rhs` semitones from this one.
    fn add(self, rhs: i32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<Octave> for Pitch {
    type Output = Self;

    fn add(self, _: Octave) -> Self::Output {
        Self(self.0 + 12)
    }
}

impl Add<Pitch> for Octave {
    type Output = Pitch;

    fn add(self, rhs: Pitch) -> Self::Output {
        rhs + self
    }
}

impl Sub<Octave> for Pitch {
    type Output = Self;

    fn sub(self, _: Octave) -> Self::Output {
        Pitch(self.0 - 12)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pitch, PENTATONIC_MINOR};

    #[test]
    fn scale_number_two_way() {
        let base = Pitch::A2;
        let scale = &PENTATONIC_MINOR;

        for i in 0..scale.len() {
            let pitch = base.in_scale(i as i32, scale);
            let j = base.scale_number(pitch, scale);

            assert_eq!(Some(i), j);
        }

        for i in -60..=60 {
            let pitch = base.in_scale(i, scale);
            let j = base.scale_number(pitch, scale);

            let wrapped = i.rem_euclid(scale.len() as i32) as usize;
            assert_eq!(Some(wrapped), j);
        }
    }
}
