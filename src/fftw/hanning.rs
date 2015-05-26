
use std::f64::consts::PI;
//use std::num::Float;


/// Precomputes the multipliers for the hanning window function so computing
/// the value only takes a single multiplication.
pub struct HanningWindowCalculator {
    multipliers: Vec<f64>
}


impl HanningWindowCalculator {
    /// The constructor computes the cache of hanning window multiplier values
    pub fn new(fft_size: usize) -> HanningWindowCalculator {
        let mut multipliers: Vec<f64> = Vec::with_capacity(fft_size);
        let divider: f64 = (fft_size - 1) as f64;

        for i in (0..fft_size) {
            let cos_inner: f64 = 2.0 * PI * (i as f64) / divider;
            let cos_part: f64 = cos_inner.cos();
            let multiplier: f64 = 0.5 * (1.0 - cos_part);
            multipliers.push(multiplier);
        }

        HanningWindowCalculator{multipliers: multipliers}
    }

    /// Multiplies the given value against the hanning window multiplier value
    /// for this index
    pub fn get_value(&self, index: usize, val: f64) -> f64 {
        self.multipliers[index] * val
    }
}
