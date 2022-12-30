pub trait MathExt {
    /// Get the value with the largest absolute value.
    fn max_abs(self, other: Self) -> Self;
}

impl MathExt for i16 {
    fn max_abs(self, other: Self) -> Self {
        if self == i16::MIN {
            self
        } else if other == i16::MIN {
            other
        } else if self.abs() > other.abs() {
            self
        } else {
            other
        }
    }
}
