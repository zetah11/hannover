use crate::structures::FixedQueue;

pub struct Delay {
    mem: FixedQueue<f64>,
    feedback: f64,
    dry: f64,
    wet: f64,
}

impl Delay {
    pub fn new(samples: usize, feedback: f64, dry: f64, wet: f64) -> Self {
        Self {
            mem: FixedQueue::new(samples),
            feedback,
            dry,
            wet,
        }
    }

    pub fn process(&mut self, sample: f64) -> f64 {
        let echo = *self.mem.get();
        let feedback = sample + self.feedback * self.wet * echo;

        let echo = self.mem.push(feedback);
        self.dry * sample + self.wet * echo
    }
}
