pub trait MathExt {
    /// Get the value with the largest absolute value.
    fn max_abs(self, other: Self) -> Self;

    /// Compute the greatest common divisor of two numbers.
    fn gcd(self, other: Self) -> Self;

    /// Compute the least common multiple of two numbers.
    fn lcm(self, other: Self) -> Self;
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

    fn gcd(self, other: Self) -> Self {
        let (mut a, mut b) = (self, other);
        while b != 0 {
            (a, b) = (b, a % b);
        }

        a.abs()
    }

    fn lcm(self, other: Self) -> Self {
        (self * other) / self.gcd(other)
    }
}

impl MathExt for usize {
    fn max_abs(self, other: Self) -> Self {
        self.max(other)
    }

    fn gcd(self, other: Self) -> Self {
        let (mut a, mut b) = (self, other);
        while b != 0 {
            (a, b) = (b, a % b);
        }
        a
    }

    fn lcm(self, other: Self) -> Self {
        (self * other) / self.gcd(other)
    }
}
