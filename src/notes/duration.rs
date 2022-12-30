use std::ops::{Add, AddAssign, Mul, MulAssign};

use crate::math::MathExt;

/// A note duration. Implemented as a multiple of thirty-second notes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Duration(usize);

impl Duration {
    pub const SIXTEENTH: Duration = Duration(2);

    /// The smallest non-zero duration represented by this.
    pub const DELTA: Duration = Duration(1);

    pub const ZERO: Duration = Duration(0);

    /// Get the number of seconds this duration lasts, for the given BPM. The
    /// BPM measures the number of *beats* - quarter notes - in one minute.
    pub fn as_time(&self, bpm: usize) -> f64 {
        let beats_per_second = bpm as f64 / 60.0;
        let num_beats = self.0 as f64 / 8.0;

        num_beats / beats_per_second
    }

    pub fn dotted(&self) -> Self {
        Self(self.0 + self.0 / 2)
    }

    /// Decrement this duration by a sixteenth note. Returns `None` in place of
    /// zero.
    pub fn decrement(&self) -> Option<Self> {
        if self.0 < 2 {
            None
        } else {
            Some(Self(self.0 - 1))
        }
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Mul<usize> for Duration {
    type Output = Self;

    fn mul(self, rhs: usize) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<Duration> for usize {
    type Output = Duration;

    fn mul(self, rhs: Duration) -> Self::Output {
        rhs * self
    }
}

impl MulAssign<usize> for Duration {
    fn mul_assign(&mut self, rhs: usize) {
        self.0 *= rhs;
    }
}
