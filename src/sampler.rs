use crate::wavetable::Wavetable;

pub struct Sampler {
    sample_rate: f64,
}

impl Sampler {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate: sample_rate as f64,
        }
    }

    pub fn sample<const S: usize>(
        &self,
        wt: &Wavetable<S>,
        frequency: f64,
        sample_no: usize,
        y: f64,
    ) -> f64 {
        let per_second = sample_no as f64 / self.sample_rate;
        let per_loop = (frequency * per_second).rem_euclid(1.0);

        wt.sample(per_loop, y)
    }
}
