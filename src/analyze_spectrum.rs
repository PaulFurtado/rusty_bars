#![allow(unstable)]

extern crate libc;
use self::libc::{c_int};
use std::num::Float;
use std::f64::consts::PI;
use std::f64;
use std::ptr;

/*
This module is responsible for a number of tasks:
1. Split the audio channels out of the stereo data
2. Translate the audio to f64 data
3. Appy a window function?! :/
4. Compute the FFT
5. Compute the equalizer bands from the FFT output
*/


// make sure we're as good as this asshole:
// http://www.swharden.com/blog/2013-05-09-realtime-fft-audio-visualization-with-python/



mod ext {
    extern crate libc;
    use self::libc::{c_int};
    use super::{FftwPlan, FftwComplex};


    #[link(name="fftw3")]
    extern {
        pub fn fftw_plan_dft_r2c_1d(n: c_int, input: *mut f64, output: *mut FftwComplex, flags: c_int) -> *const FftwPlan;
        pub fn fftw_execute(plan: *const FftwPlan);
    }
}


/// {FFTW_ESTIMATE} or 64. Specifies that, instead of actual measurements of
/// different algorithms, a simple heuristic is used to pick a (probably
/// sub-optimal) plan quickly. With this flag, the input/output arrays are not
/// overwritten during planning. It is the default value
const FFTW_ESTIMATE: c_int = (1 << 6);
/// FFTW_MEASURE or 0. tells FFTW to find an optimized plan by actually
/// computing several FFTs and measuring their execution time. Depending on
/// your machine, this can take some time (often a few seconds).
const FFTW_MEASURE: c_int = 0;
/// FFTW_PATIENT or 32. It is like "FFTW_MEASURE", but considers a wider range
/// of algorithms and often produces a “more optimal” plan (especially for large
/// transforms), but at the expense of several times longer planning time
/// (especially for large transforms).
const FFTW_PATIENT: c_int = 32;
/// FFTW_EXHAUSTIVE or 8. It is like "FFTW_PATIENT", but considers an even wider
/// range of algorithms, including many that we think are unlikely to be fast,
/// to produce the most optimal plan but with a substantially increased planning
/// time.
const FFTW_EXHAUSTIVE: c_int = 8;



#[derive(Copy)]
pub enum FftwPlan {}


#[repr(C)]
#[derive(Copy)]
pub struct FftwComplex {
    re: f64,
    im: f64
}


impl FftwComplex {
    pub fn abs(&self) -> f64 {
        ((self.re * self.re) + (self.im * self.im)).sqrt()
    }
}


fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
}


#[test]
fn test_pwer_two() {
    assert!(is_power_of_two(1024));
    assert!(is_power_of_two(512));
    assert!(is_power_of_two(2));
    assert!(is_power_of_two(4));
    assert!(is_power_of_two(8));
    assert!(is_power_of_two(16));
    assert!(is_power_of_two(32));
    assert!(!is_power_of_two(1));
    assert!(!is_power_of_two(7));
    assert!(!is_power_of_two(500));
}



pub fn compute_input_size(desired_output: usize) -> usize {
    for size in (desired_output..8192) {
        if is_power_of_two(size) {
            return desired_output * 2;
        }
    }
    return 8192 * 2;
}




/// Scales down a vector by averaging the elements between the resulting points
pub fn scale_fft_output(input: &Vec<f64>, new_len: usize) -> Vec<f64> {
    if new_len >= input.len() {
        return input.clone();
    }

    let band_size: usize = input.len() / new_len;
    assert!(band_size > 0);
    let mut output: Vec<f64> = Vec::with_capacity(new_len);

    let mut temp_count: usize = 0;
    let mut sum: f64 = 0.0;

    for &x in input.iter() {
        if temp_count >= band_size {
            let avg: f64 = sum/temp_count as f64;
            output.push(avg);
            temp_count = 0;
            sum = 0.0;
        } else {
            sum += x;
            temp_count+=1;
        }
    }

    if temp_count >= band_size {
        output.push(sum/temp_count as f64);
    }

    output
}



/// Precomputes the multipliers for the hanning window function so computing
/// the value only takes a single multiplication.
pub struct HanningWindowCalculator {
    multipliers: Vec<f64>
}


impl HanningWindowCalculator {
    /// The constructor computes the cache of hanning window multiplier values
    fn new(fft_size: usize) -> HanningWindowCalculator {
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
    fn get_value(&self, index: usize, val: f64) -> f64 {
        self.multipliers[index] * val
    }
}

pub struct ChannelInputManager {
    window_calculator: HanningWindowCalculator,
    channel_inputs: Vec<Vec<f64>>,
    fft_size: usize,
    channel_count: usize
}


impl ChannelInputManager {
    pub fn new(channel_count: usize, fft_size: usize) -> ChannelInputManager {
        let mut inputs: Vec<Vec<f64>> = Vec::with_capacity(channel_count);

        // Create an initialize a vec of the right length
        let mut base_vec: Vec<f64> = Vec::with_capacity(fft_size);
        for _ in (0..fft_size) {
            base_vec.push(0f64);
        }

        // Clone the first channel for all additional channels (rather than
        // initializing a fresh vector).
        for _ in (0..channel_count) {
            inputs.push(base_vec.clone());
        }

        ChannelInputManager{
            window_calculator: HanningWindowCalculator::new(fft_size),
            channel_inputs: inputs,
            fft_size: fft_size,
            channel_count: channel_count
        }
    }

    pub fn get_chan_zero_ptr(&mut self) -> *mut f64 {
        self.channel_inputs[0].as_mut_ptr()
    }

    /// Load raw data from an S16LE buffer of audio data into each channel,
    /// performing the hanning window function as it loads them
    pub fn load_in_data(&mut self, buffer: &[u8]) {
        // Cast the &[u8] pointer to an i16 pointer so we don't have to do a
        // conversion of each element to i16
        let buf_ptr: *const i16 = buffer.as_ptr() as *const i16;
        for i in (0..buffer.len()/2) {
            let channel = i % self.channel_count;
            let channel_index = i/self.channel_count;

            let value: i16 = unsafe { *buf_ptr.offset(i as isize) };
            let value_f64: f64 = self.window_calculator.get_value(channel_index, value as f64);
            self.channel_inputs[channel][channel_index] = value_f64;
        }
    }

    /// Load the channel number into channel 0 so the FFT can be executed on it
    pub fn load_into_zero(&mut self, channel: usize) {
        unsafe {
            ptr::copy_memory(
                self.channel_inputs[0].as_mut_ptr(),
                self.channel_inputs[channel].as_ptr(),
                self.fft_size
            )
        }
    }
}


/// Takes output from FFTW for each channel and combines them, using only one
/// vector instead of a vector for each channel.
pub struct ChannelOutputManager {
    channel_count: usize,
    fft_size: usize,
    combined: Vec<f64>
}


impl ChannelOutputManager {
    /// Initialize a ChannelOutputManager
    pub fn new(channel_count: usize, fft_size: usize) -> ChannelOutputManager {
        let mut combined: Vec<f64> = Vec::with_capacity(fft_size/2);
        for _ in (0..fft_size/2) {
            combined.push(0.0);
        }

        ChannelOutputManager {
            channel_count: channel_count,
            fft_size: fft_size,
            combined: combined
        }
    }

    /// Add values from an execution of FFTW. Combines multiple channels of
    /// FFT by picking whichever channel is highest for each frequency bin.
    /// Also handles conversion from FFT data to decibels
    /// Arguments:
    ///     values - a vector of values to load in
    ///     overwrite - if true, overwrites the data in this ChannelOutputManager
    ///                 with fresh data.
    pub fn load_values(&mut self, values: &Vec<FftwComplex>, overwrite: bool) {
        for i in (0..self.fft_size/2) {
            let val: f64 = 20.0 * values[i].abs().log10();
            if overwrite || val > self.combined[i] {
                self.combined[i] = val;
            }
        }
    }

    /// Gets the combined output
    pub fn get_combined(&self) -> Vec<f64> {
        // TODO: just return an iterator instead of cloning
        self.combined.clone()
    }
}




pub struct AudioFFT<'a> {
    channels: usize,
    input_manager: ChannelInputManager,
    output_manager: ChannelOutputManager,
    output: Vec<FftwComplex>,
    plan: *const FftwPlan,
    n: usize,
}


impl<'a> AudioFFT<'a> {
    pub fn new(n: usize, channels: usize) -> AudioFFT<'a> {
        if !is_power_of_two(n) {
            panic!("n should be a power of two!");
        }

        // output is where the FFT puts its data.
        // FFTs are symmetrical and the real FFT optimizes by returning a
        // half-length array rather than doing extra computation
        let mut output: Vec<FftwComplex> = Vec::with_capacity(n/2);

        // initialize the arrays.
        for _ in range(0, n/2) {
             output.push(FftwComplex{im:0f64,re:0f64});
        }

        let mut input_manager = ChannelInputManager::new(channels, n);

        let plan = unsafe {
            ext::fftw_plan_dft_r2c_1d(
                n as i32,
                input_manager.get_chan_zero_ptr(),
                output.as_mut_ptr(),
                FFTW_MEASURE
            )
        };

        AudioFFT {
            channels: channels,
            output: output,
            plan: plan,
            input_manager: input_manager,
            output_manager: ChannelOutputManager::new(channels, n),
            n: n
        }
    }

    /// Returns the amount of data we need to make this work.
    pub fn get_buf_size(&self) -> usize {
        const BYTES_PER_SAMPLE: usize = 2; // 16 bit
        self.n * BYTES_PER_SAMPLE * self.channels
    }

    /// Reads the output from the FFT and converts it into averages of parts of
    /// the power spectrum. (Ex: an equalizer visualizer).
    /// This function may need some work.
    fn get_output(&self) -> Vec<f64> {
        // Convert the FFT data into decibals (power)
        self.output.iter().map(|x| 20.0 * x.abs().log10()).collect()
    }

    /// Turn a buffer into equalizer data.
    pub fn execute(&mut self, buffer: &[u8]) -> Vec<f64> {
        if buffer.len() != self.get_buf_size() {
            panic!("incorrect buffer length");
        }
        self.input_manager.load_in_data(buffer);

        for channel in (0..self.channels) {
            if channel > 0 {
                self.input_manager.load_into_zero(channel);
            }
            unsafe { ext::fftw_execute(self.plan) };
            self.output_manager.load_values(&self.output, channel == 0);
        }

        self.output_manager.get_combined()
    }

}
