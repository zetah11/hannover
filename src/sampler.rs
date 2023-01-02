use crate::wavetable::Wavetable;

pub struct Sampler {
    sample_rate: f64,
    sample_offset: usize,
}

impl Sampler {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            sample_rate: sample_rate as f64,
            sample_offset: 0,
        }
    }

    pub fn sample<const S: usize>(
        &self,
        wt: &Wavetable<S>,
        frequency: f64,
        sample_no: usize,
        y: f64,
    ) -> f64 {
        let per_second = (sample_no + self.sample_offset) as f64 / self.sample_rate;
        let per_loop = (frequency * per_second).rem_euclid(1.0);

        wt.sample(per_loop, y)
    }

    pub fn step(&mut self, by: usize) {
        self.sample_offset += by;
    }

    pub fn seconds_per_sample(&self) -> f64 {
        1.0 / self.sample_rate
    }
}
