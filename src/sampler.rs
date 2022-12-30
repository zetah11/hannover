use crate::wavetable::Wavetable;

/// The number of samples per second.
pub const SAMPLE_RATE: usize = 10;

/// Produce an audio sample at the given frequency (in Hz) and sample number.
/// Linearly interpolates between the two samples in the wavetable it lands
/// between.
pub fn sample<const S: usize>(wt: &Wavetable<S>, frequency: f64, sample_no: usize, y: f64) -> f64 {
    let per_second = sample_no as f64 / SAMPLE_RATE as f64;
    let per_loop = (frequency * per_second).rem_euclid(1.0);

    wt.sample(per_loop, y)
}
