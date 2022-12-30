use std::collections::VecDeque;
use std::f64::consts::TAU;

use log::{debug, trace};

use crate::bytes::NibbleStream;
use crate::data::{PRIMES, RIJNDAEL_SBOX};
use crate::math::MathExt;

pub const MAX_CURSORS: usize = 10;

pub struct Wavetable<const S: usize> {
    data: [[u8; S]; S],
    cursors: VecDeque<(usize, usize)>,
}

impl<const S: usize> Wavetable<S> {
    const SIZE: f64 = S as f64;

    pub fn new_sine() -> Self {
        let mut data = [[0; S]; S];
        for (y, data) in data.iter_mut().enumerate() {
            for (x, data) in data.iter_mut().enumerate() {
                let x = x as f64 / Self::SIZE;
                let y = y as f64 / Self::SIZE;

                let value = (TAU * x - TAU * y).sin();
                let value = 0.5 * value + 0.5;
                let value = value * 255.0;

                *data = value as u8;
            }
        }

        Self {
            data,
            cursors: VecDeque::from([(S / 2, S / 2)]),
        }
    }

    /// Sample the wavetable at the given `x` `y` coordinates. `x` and `y` are
    /// in the range `[0, 1)`, and the resulting value is a bilinear
    /// interpolation at this point and lies in the range `[0, 1]`.
    pub fn sample(&self, x: f64, y: f64) -> f64 {
        // Compute the four coordinates close to this one.
        let x1_index = (x * Self::SIZE) as usize;
        let y1_index = (y * Self::SIZE) as usize;

        let x1 = x1_index as f64 / Self::SIZE;
        let x2 = (x1_index + 1) as f64 / Self::SIZE;

        let y1 = y1_index as f64 / Self::SIZE;
        let y2 = (y1_index + 1) as f64 / Self::SIZE;

        // Compute the x and y t-values (for lerping).
        let t_x = (x - x1) / (x2 - x1);
        let t_y = (y - y1) / (y2 - y1);

        // Get the wavetable values at the four coordinates.
        let x2_index = (x1_index + 1) % S;
        let y2_index = (y1_index + 1) % S;

        let q11 = self.data[y1_index][x1_index] as f64 / 255.0;
        let q21 = self.data[y1_index][x2_index] as f64 / 255.0;
        let q12 = self.data[y2_index][x1_index] as f64 / 255.0;
        let q22 = self.data[y2_index][x2_index] as f64 / 255.0;

        // Interpolate!
        let top = (1.0 - t_x) * q11 + t_x * q21;
        let bot = (1.0 - t_x) * q12 + t_x * q22;
        (1.0 - t_y) * top + t_y * bot
    }

    pub fn increment(&mut self) {
        for (x, y) in self.cursors.iter_mut() {
            *x = if *x == 0 { S - 1 } else { *x - 1 };
            *y = if *y == 0 { S - 1 } else { *y - 1 };
        }
    }

    pub fn execute(&mut self, inst: Instruction) {
        trace!("{:?}", self.cursors);

        match inst {
            Instruction::Noop => {
                debug!("no-op");
            }
            Instruction::MultiplyCursor(by_x, by_y) => {
                debug!("multiply cursor {by_x} {by_y}");
                for (x, y) in self.cursors.iter_mut() {
                    *x = (*x * by_x) % S;
                    *y = (*y * by_y) % S;
                }
            }
            Instruction::VoronoiCursor => {
                debug!("voronoi cursor");
                let mut furthest = ((S / 2, S / 2), 0);

                for y in (0..S).rev() {
                    for x in (0..S).rev() {
                        let dist = self
                            .cursors
                            .iter()
                            .copied()
                            .map(|(x2, y2)| {
                                ((x as isize - x2 as isize).abs() + (y as isize - y2 as isize))
                                    as usize
                            })
                            .min()
                            .unwrap_or(0);
                        if dist <= furthest.1 {
                            furthest = ((x, y), dist);
                        }
                    }
                }

                self.cursors.push_back(furthest.0);
                if self.cursors.len() >= MAX_CURSORS {
                    self.cursors.pop_front();
                }
            }
            Instruction::MultiplyData(by) => {
                debug!("multiply data {by}");
                for (x, y) in self.cursors.iter().copied() {
                    self.data[y][x] = self.data[y][x].wrapping_mul(by as u8);
                }
            }
            Instruction::Slant => {
                debug!("slant");
                for (x, y) in self.cursors.iter().copied() {
                    slant(&mut self.data, x, y);
                }
            }
            Instruction::Smooth => {
                debug!("smooth");
                for (x, y) in self.cursors.iter().copied() {
                    smooth(&mut self.data, x, y);
                }
            }
            Instruction::Substitution => {
                debug!("substitution");
                for (x, y) in self.cursors.iter().copied() {
                    self.data[y][x] = RIJNDAEL_SBOX[self.data[y][x] as usize];
                }
            }
            Instruction::SignedDataMove => {
                debug!("signed data move");
                for (x, y) in self.cursors.iter_mut() {
                    let data = self.data[*y][*x];
                    let xoff = data & 0xf0;
                    let yoff = data << 4;

                    let xoff = (xoff & 0x70) as isize + if xoff & 0x80 != 0 { -0x80 } else { 0 };
                    let yoff = (yoff & 0x70) as isize + if yoff & 0x80 != 0 { -0x80 } else { 0 };

                    *x = (*x as isize + xoff).rem_euclid(S as isize) as usize;
                    *y = (*y as isize + yoff).rem_euclid(S as isize) as usize;
                }
            }
            Instruction::MoveDiagonal(start) => {
                debug!("move diagonal {start}");
                if !self.cursors.is_empty() {
                    let start = start % self.cursors.len();
                    let mut off = 1;

                    for i in (start..self.cursors.len()).chain(0..start) {
                        let (x, y) = self.cursors[i];
                        self.cursors[i] = ((x + off) % S, (y + off) % S);
                        off += 1;
                    }
                }
            }
            Instruction::Transpose => {
                debug!("transpose");
                for y in 0..S - 1 {
                    for x in y + 1..S {
                        let tmp = self.data[y][x];
                        self.data[y][x] = self.data[x][y];
                        self.data[x][y] = tmp;
                    }
                }
            }
            Instruction::RemoveOldest => {
                debug!("remove oldest");
                if self.cursors.len() > 1 {
                    self.cursors.pop_front();
                }
            }
        }
    }
}

/// An instruction for the wavetable virtual machine.
pub enum Instruction {
    /// Do nothing.
    Noop,
    /// Add the given `x` `y` pair to each cursor.
    MultiplyCursor(usize, usize),
    /// Introduce a cursor at the smallest x, y pair furthest from every other
    /// cursor.
    VoronoiCursor,
    /// Multiply the data by the given value.
    MultiplyData(usize),
    /// Increase the slant in the neighborhood.
    Slant,
    /// Smooth out the neighborhood.
    Smooth,
    /// Apply the Rijndael S-Box to the data.
    Substitution,
    /// Move the cursor by interpreting the data as a pair of signed 4-bit
    /// offsets.
    SignedDataMove,
    /// Move each cursor diagonal by its index, starting with the one at the
    /// index specified by the given value (modulo the cursor length).
    MoveDiagonal(usize),
    /// Transpose the wavetable.
    Transpose,
    /// Remove the oldest cursor
    RemoveOldest,
}

impl NibbleStream<1> {
    pub fn next_instruction(&mut self) -> Instruction {
        match self.next_nibble() {
            0x0 | 0xb..=0xf => Instruction::Noop,

            0x1 => {
                let nibble = self.next_nibble();
                let prime1 = PRIMES[nibble as usize];
                let prime2 = PRIMES[0x10 - nibble as usize];
                Instruction::MultiplyCursor(prime1, prime2)
            }

            0x2 => Instruction::VoronoiCursor,
            0x3 => Instruction::MultiplyData(self.next_prime()),
            0x4 => Instruction::Slant,
            0x5 => Instruction::Smooth,
            0x6 => Instruction::Substitution,
            0x7 => Instruction::SignedDataMove,
            0x8 => Instruction::MoveDiagonal(self.next_prime()),
            0x9 => Instruction::Transpose,
            0xa => Instruction::RemoveOldest,

            0x10..=u8::MAX => unreachable!("next_nibble returns a nibble"),
        }
    }

    fn next_prime(&mut self) -> usize {
        PRIMES[self.next_nibble() as usize]
    }
}

/// Make the immediate neighborhood of this `x` `y` coordinate pair into a plane
/// with the largest slope.
fn slant<const S: usize>(data: &mut [[u8; S]; S], x: usize, y: usize) {
    let x1 = if x == 0 { S - 1 } else { x - 1 };
    let x2 = if x == S - 1 { 0 } else { x + 1 };

    let y1 = if y == 0 { S - 1 } else { y - 1 };
    let y2 = if y == S - 1 { 0 } else { y + 1 };

    let thi = data[y][x] as i16;

    let lef = data[y][x1] as i16;
    let rig = data[y][x2] as i16;

    let bot = data[y1][x] as i16;
    let top = data[y2][x] as i16;

    let (dx1, dx2) = (thi - lef, rig - thi);
    let (dy1, dy2) = (thi - bot, top - thi);

    let dx = dx1.max_abs(dx2);
    let dy = dy1.max_abs(dy2);

    data[y1][x1] = (data[y1][x1] as i16 - dx - dy).rem_euclid(u8::MAX as i16) as u8;
    data[y1][x] = (data[y1][x] as i16 - dy).rem_euclid(u8::MAX as i16) as u8;
    data[y1][x2] = (data[y1][x2] as i16 + dx - dy).rem_euclid(u8::MAX as i16) as u8;
    data[y][x1] = (data[y1][x1] as i16 - dx).rem_euclid(u8::MAX as i16) as u8;
    data[y][x2] = (data[y1][x2] as i16 + dx).rem_euclid(u8::MAX as i16) as u8;
    data[y2][x1] = (data[y2][x1] as i16 - dx + dy).rem_euclid(u8::MAX as i16) as u8;
    data[y2][x] = (data[y2][x] as i16 + dy).rem_euclid(u8::MAX as i16) as u8;
    data[y2][x2] = (data[y2][x2] as i16 + dx + dy).rem_euclid(u8::MAX as i16) as u8;
}

/// Smooth out the neighborhood of this `x` `y` coordinate pair.
fn smooth<const S: usize>(data: &mut [[u8; S]; S], x: usize, y: usize) {
    let x1 = if x == 0 { S - 1 } else { x - 1 };
    let x2 = if x == S - 1 { 0 } else { x + 1 };

    let y1 = if y == 0 { S - 1 } else { y - 1 };
    let y2 = if y == S - 1 { 0 } else { y + 1 };

    smooth_one(data, x1, y1);
    smooth_one(data, x, y1);
    smooth_one(data, x2, y1);
    smooth_one(data, x1, y);
    smooth_one(data, x, y);
    smooth_one(data, x2, y);
    smooth_one(data, x1, y2);
    smooth_one(data, x, y2);
    smooth_one(data, x2, y2);
}

/// Smooth out this `x` `y` coordinate pair.
#[inline(always)]
fn smooth_one<const S: usize>(data: &mut [[u8; S]; S], x: usize, y: usize) {
    let x1 = if x == 0 { S - 1 } else { x - 1 };
    let x2 = if x == S - 1 { 0 } else { x + 1 };

    let y1 = if y == 0 { S - 1 } else { y - 1 };
    let y2 = if y == S - 1 { 0 } else { y + 1 };

    let thi = data[y][x] as i16;

    let lef = data[y][x1] as i16;
    let rig = data[y][x2] as i16;

    let bot = data[y1][x] as i16;
    let top = data[y2][x] as i16;

    let dx2 = rig - 2 * thi + lef;
    let dy2 = top - 2 * thi + bot;

    let diff = dx2 + dy2;
    data[y][x] = (data[y][x] as i16)
        .saturating_add(diff)
        .rem_euclid(u8::MAX as i16) as u8;
}
