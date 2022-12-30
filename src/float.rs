use crate::bytes::NibbleStream;

/// Stores a value in the range `[0, 1)`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Float {
    value: f64,
}

impl Float {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    /// Add a value to this one.
    pub fn add(&mut self, value: f64) {
        self.value = (self.value + value).rem_euclid(1.0);
    }

    pub fn sample(&self) -> f64 {
        self.value
    }
}

impl NibbleStream<5> {
    pub fn next_coarse_float(&mut self) -> f64 {
        let nibbles = self.next_nibbles();
        let bytes = [
            (nibbles[0] << 4) | nibbles[1],
            (nibbles[2] << 4) | nibbles[3],
            (nibbles[4] << 4) | nibbles[0],
            ((nibbles[1] ^ nibbles[3]) << 4) | (nibbles[2] ^ nibbles[4]),
        ];

        let value = u32::from_le_bytes(bytes);
        value as f64 / (u32::MAX as f64 + 1.0)
    }
}
