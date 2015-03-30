#![allow(unstable)]

extern crate libc;
use self::libc::{c_int, c_char, size_t};
use std::num::Float;
use std::f64::consts::PI;

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


const FFT_SIZE: usize = 1024;



mod ext {
    extern crate libc;
    use self::libc::{c_int, c_char, size_t};
    use super::{FftwPlan, FftwComplex};



    #[link(name="fftw3")]
    extern {
        pub fn fftw_plan_dft_r2c_1d(n: c_int, input: *mut f64, output: *mut FftwComplex, flags: c_int) -> *const FftwPlan;
        pub fn fftw_execute(plan: *const FftwPlan);
    }
}


const FFTW_ESTIMATE: c_int = (1 << 6);


#[derive(Copy)]
pub enum FftwPlan {}


#[repr(C)]
#[derive(Copy)]
struct FftwComplex {
    re: f64,
    im: f64
}


impl FftwComplex {
    pub fn abs(&self) -> f64 {
        ((self.re * self.re) + (self.im * self.im)).sqrt()
    }
}



pub struct AudioFFT<'a> {
    channels: usize,
    input: Vec<f64>,
    output: Vec<FftwComplex>,
    plan: *const FftwPlan,
    bands: usize,
    n: usize,
    sample_rate: usize
}


impl<'a> AudioFFT<'a> {
    pub fn new(n: usize, channels: usize, sample_rate: usize, bands: usize) -> AudioFFT<'a> {
        // input is the input array to the fft, output is where the fft puts its
        // output
        let mut input: Vec<f64> = Vec::with_capacity(n);
        let mut output: Vec<FftwComplex> = Vec::with_capacity(n/2);

        for _ in range(0, n) {
            input.push(0f64);
        }
        for _ in range(0, n/2) {
             output.push(FftwComplex{im:0f64,re:0f64});
        }



        let plan = unsafe { ext::fftw_plan_dft_r2c_1d(n as i32, input.as_mut_ptr(), output.as_mut_ptr(), FFTW_ESTIMATE)};

        AudioFFT {
            channels: channels,
            input: input,
            output: output,
            bands: bands,
            plan: plan,
            n: n,
            sample_rate: sample_rate
        }
    }

    /// Returns the amount of data we need to make this work
    pub fn get_buf_size(&self) -> usize {
        const BYTES_PER_SAMPLE: usize = 2; // 16 bit
        self.n * BYTES_PER_SAMPLE * self.channels
    }


    fn get_floats(&self, buffer: &[u8]) -> Vec<f64> {
        let short_vec: Vec<i16> = unsafe{ Vec::from_raw_buf(buffer.as_ptr() as *const i16, buffer.len()/2) };
        let mut float_vec: Vec<f64> = Vec::with_capacity(short_vec.len());
        for val in short_vec.iter() {
            float_vec.push(*val as f64);
        }
        float_vec
    }


    fn split_channels(&self, all_floats: &Vec<f64>) -> Vec<Vec<f64>> {
        let mut out: Vec<Vec<f64>> = Vec::new();
        for channel in range(0, self.channels) {
            out.push(Vec::with_capacity(all_floats.len()/self.channels));
        }
        for (i, &val) in all_floats.iter().enumerate() {
            out[i % self.channels].push(val);
        }
        out
    }


    fn load_channel(&mut self, channel_data: &Vec<f64>) {
        for (i, &val) in channel_data.iter().enumerate() {
            self.input[i] = val;
        }
    }

    fn do_hanning_window(&self, channel_data: &mut Vec<f64>) {
        let divider: f64 = (channel_data.len() - 1) as f64;

        for (i, val) in channel_data.iter_mut().enumerate() {
            let cos_inner: f64 = 2.0 * PI * (i as f64) / divider;
            let cos_part: f64 = cos_inner.cos();
            let multiplier: f64 = 0.5 * (1.0 - cos_part);
            //println!("mult: {}, cos: {}, cosi: {}", multiplier, cos_part, cos_inner);
            *val = *val * multiplier;
        }

    }


    fn get_output(&self) -> Vec<f64> {
        // FFT is symmetric over its center so half the values are good enough
        let power:Vec<f64> = self.output.iter().map(|x| 20.0 * x.abs().log10()).collect();
        let band_size = power.len() / self.bands;
        let mut out: Vec<f64> = Vec::new();


        for _ in range(0, self.bands) {
            out.push(0f64);
        }

        for (i, &val) in power.iter().enumerate() {
            // i/band_size is kinda dirty. Maybe loop through bands and take
            // slices instead?
            out[i/band_size] += val;
        }

        for v in out.iter_mut() {
            *v = *v/(band_size as f64);
        }


        out
        //power
    }

    pub fn execute(&mut self, buffer: &[u8]) -> Vec<f64> {
        if buffer.len() != self.get_buf_size() {
            panic!("incorrect buffer length");
        }
        let all_floats = self.get_floats(buffer);
        let mut channel_data = self.split_channels(&all_floats);
        self.do_hanning_window(&mut channel_data[0]);
        self.load_channel(&channel_data[0]);

        unsafe { ext::fftw_execute(self.plan) };
        self.get_output()
    }


}
