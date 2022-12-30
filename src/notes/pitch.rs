use std::ops::Add;
use std::ops::Sub;

/// The base frequency, in Hertz, of A4.
const A4: f64 = 440.0;
const TWELFTH_ROOT_TWO: f64 = 1.059_463_094_359_295_3;

/// A pitch is an exponential frequency. Represented as a semi-tone offset from
/// A4.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Pitch(i32);

impl Pitch {
    pub const A2: Pitch = Pitch(-24);

    pub const C4: Pitch = Pitch(-9);
    pub const CS4: Pitch = Pitch(-8);
    pub const D4: Pitch = Pitch(-7);
    pub const DS4: Pitch = Pitch(-8);
    pub const E4: Pitch = Pitch(-5);
    pub const F4: Pitch = Pitch(-4);
    pub const FS4: Pitch = Pitch(-3);
    pub const G4: Pitch = Pitch(-2);
    pub const GS4: Pitch = Pitch(-1);
    pub const A4: Pitch = Pitch(0);
    pub const AS4: Pitch = Pitch(1);
    pub const B4: Pitch = Pitch(2);

    /// Get the frequency, in Hz, of this pitch.
    pub fn as_frequency(&self) -> f64 {
        A4 * (TWELFTH_ROOT_TWO.powi(self.0))
    }

    pub fn in_pentatonic_minor(&self, n: i32) -> Pitch {
        self.in_scale(n, &[3, 2, 2, 3, 2])
    }

    pub fn in_major(&self, n: i32) -> Pitch {
        self.in_scale(n, &[2, 2, 1, 2, 2, 2, 1])
    }

    pub fn in_minor(&self, n: i32) -> Pitch {
        self.in_scale(n, &[2, 1, 2, 2, 1, 2, 2])
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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Octave;

impl Add<Octave> for Pitch {
    type Output = Self;

    fn add(self, _: Octave) -> Self::Output {
        Pitch(self.0 + 12)
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
