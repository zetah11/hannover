#[derive(Clone, Copy, Debug)]
pub struct AttackDecay {
    /// Number of seconds for the attack.
    attack: f64,

    /// Number of seconds it takes for the envelope to decay completely. This is
    /// a cumulative value, meaning it must always be `>=` the attack.
    decay_cumulative: f64,

    /// A monotonically increasing time value.
    value: f64,
}

impl AttackDecay {
    pub fn new(attack: f64, decay: f64) -> Self {
        let decay_cumulative = attack + decay;
        Self {
            attack,
            decay_cumulative,
            value: 0.0,
        }
    }

    pub fn reset(&mut self) {
        self.value = 0.0;
    }

    pub fn step(&mut self, by: f64) {
        self.value += by;
    }

    pub fn is_done(&self) -> bool {
        self.value >= self.decay_cumulative
    }

    pub fn value(&self) -> f64 {
        let Self {
            value,
            attack,
            decay_cumulative,
        } = self;

        if value >= decay_cumulative {
            0.0
        } else if value > attack {
            1.0 - (value - attack) / (decay_cumulative - attack)
        } else {
            value / attack
        }
    }
}
