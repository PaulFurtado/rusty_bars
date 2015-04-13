#![allow(unstable)]

extern crate libc;
use self::libc::{c_int, size_t, c_void};
use std::num::Float;
use std::f64::consts::PI;
use std::{mem, slice};


/// External functions for interacting with FFTW
mod ext {
    extern crate libc;
    use self::libc::{c_int, size_t, c_void};
    use super::{FftwPlan, FftwComplex, PlannerFlags};

    #[repr(C)]
    #[derive(Copy)]
    /// An opaque pointer to an FFTW C plan
    pub struct fftw_plan;

    #[link(name="fftw3")]
    extern {
        /// Creates a 1-dimensional real-to-complex FFT plan
        pub fn fftw_plan_dft_r2c_1d(n: c_int, input: *mut f64, output: *mut FftwComplex, flags: PlannerFlags) -> *mut fftw_plan;

        /// Executes an FFTW plan
        pub fn fftw_execute(plan: *const fftw_plan);

        /// Performs a proper-aligned malloc
        pub fn fftw_malloc(n: size_t) -> *mut c_void;

        /// Frees an fftw-malloc'ed chunk of memor
        pub fn fftw_free(ptr: *mut c_void);

        /// Destroy an fftw plan. Should be called when done with a plan.
        pub fn fftw_destroy_plan(plan: *mut fftw_plan);
    }
}

#[repr(C)]
#[derive(Copy)]
/// Thse can be found in fftw/api/fftw2.h in the FFW source. In FFTW, they are
/// defined with #define.
/// You can read more about planner flags here:
/// http://www.fftw.org/doc/Planner-Flags.html
pub enum PlannerFlags {
    Measure         = 0,
    DestroyInput    = 1 << 0,
    Unaligned       = 1 << 1,
    ConserveMemory  = 1 << 2,
    Exhaustive      = 1 << 3,
    PreserveInput   = 1 << 4,
    Patient         = 1 << 5,
    Estimate        = 1 << 6,
    WisdomOnly      = 1 << 21,
}


#[repr(C)]
#[derive(Copy)]
/// Represents a 64-bit complex number.
pub struct FftwComplex {
    re: f64,
    im: f64
}

impl FftwComplex {
    /// Get the absolute value (distance from zero) of the complex number
    pub fn abs(&self) -> f64 {
        ((self.re * self.re) + (self.im * self.im)).sqrt()
    }
}


/// Wrapper around fftw_malloc which automatically allocates the right amount of
/// space for Rust objects, similar to calloc.
/// Returns None if allocation failed.
fn fftw_ralloc<T>(count: usize) -> Option<*mut T> {
    let element_size = mem::size_of::<T>();
    let total_size = element_size * count;
    let result: *mut T = unsafe { ext::fftw_malloc(total_size as size_t) } as *mut T;
    if result.is_null() {
        None
    } else {
        Some(result)
    }
}


/// FFTW depends on memory alignment in order to take advantage of SIMD
/// instructions. While this isn't a massive FFT, alignment is still important
/// because when FFTW chooses the algorithm to use, it needs to consider the
/// alignment. Ex: an algorithm with lots of SIMD instructions would be
/// unusable on unaligned data. Since we have multiple audio channels to run
/// FFTs on, FFTW can plan once and operate on many different arrays if they
/// are all aligned the exact same way.
/// See: http://www.fftw.org/doc/Memory-Allocation.html
/// FftAlignedArray is a type which utilizes FFTW's malloc to take advantage of
/// alignment. It may be possible to use Vec::from_raw_parts, but you need to
/// run FFTW's free function when you're done with it and Vec frees its pointer
/// when it is dropped so stopping that would involve hacks.
/// The FftAlignedArray struct doesn't implement any features a Vec does,
/// instead, it just gives you back slices so you can do
pub struct FftwAlignedArray<T> {
    len: usize,
    ptr: *const T,
    mut_ptr: *mut T,
}

impl<T: Copy> FftwAlignedArray<T> {
    /// Create a new FftwAlignedArray.
    /// Len is the number of elements, not the size in bytes.
    /// Panics if memory allocation fails.
    fn new(len: usize) -> FftwAlignedArray<T> {
        let ptr: *mut T = fftw_ralloc::<T>(len).unwrap();
        FftwAlignedArray {
            len: len,
            ptr: ptr as *const T,
            mut_ptr: ptr
        }
    }

    /// Initialize every element in the array with init_val
    fn initialize(&mut self, init_val: T) {
        for val in self.as_mut_slice().iter_mut() {
            *val = init_val;
        }
    }

    /// Get an immutable raw pointer to the memory backing this array
    fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Get an mutable raw pointer to the memory backing this array
    fn as_mut_ptr(&self) -> *mut T {
        self.mut_ptr
    }

    /// Modify the contents of this array via a mutable slice
    fn as_mut_slice<'a>(&'a mut self) -> &'a mut [T] {
        unsafe{ slice::from_raw_mut_buf(&self.mut_ptr, self.len) }
    }
}


impl<T> AsSlice<T> for FftwAlignedArray<T> {
    /// Access the contents of this array via an immutable slice
    fn as_slice(&self) -> &[T] {
        unsafe{ slice::from_raw_buf(&self.ptr, self.len) }
    }
}


#[unsafe_destructor]
/// Unsafe because it has lifetimes.
impl<T> Drop for FftwAlignedArray<T> {
    /// Free the array with the right deallocator
    fn drop(&mut self) {
        unsafe{ ext::fftw_free(self.mut_ptr as *mut c_void) };
    }
}


/// Rust wrapper for an FFTW plan
pub struct FftwPlan {
    input: FftwAlignedArray<f64>,
    output: FftwAlignedArray<FftwComplex>,
    size: usize,
    plan: *mut ext::fftw_plan,
}


impl FftwPlan {
    /// Create a new wrapper around an FFTW plan
    pub fn new(size: usize) -> FftwPlan {
        if !is_power_of_two(size) {
            panic!("FFT size should be a power of two!");
        }

        let mut input = FftwAlignedArray::new(size);
        input.initialize(0.0);
        let mut output = FftwAlignedArray::new(size);
        output.initialize(FftwComplex{re: 0.0, im: 0.0});

        let plan = unsafe {
            ext::fftw_plan_dft_r2c_1d(
                size as i32,
                input.as_mut_ptr(),
                output.as_mut_ptr(),
                PlannerFlags::Measure
            )
        };

        FftwPlan {
            input: input,
            output: output,
            size: size,
            plan: plan
        }
    }

    /// Execute the plan
    pub fn execute(&mut self) {
        unsafe { ext::fftw_execute(self.plan) };
    }

    /// Get a slice of the FFTW plan's input buffer
    pub fn get_input_slice<'a>(&'a mut self) -> &'a mut [f64] {
        self.input.as_mut_slice()
    }

    /// Get a slice of the FFTW plan's output buffer
    pub fn get_output_slice<'a>(&'a self) -> &'a [FftwComplex] {
        // A real FFT outputs half of the input size.
        self.output.as_slice().slice_to(self.size/2)
    }
}

#[unsafe_destructor]
/// Unsafe because it has lifetimes.
impl Drop for FftwPlan {
    /// Runds fftw_destroy plan when a plan goes out of scope
    fn drop(&mut self) {
        unsafe { ext::fftw_destroy_plan(self.plan); }
    }
}



/// An FFT for multiple channels of data.
pub struct MultiChannelFft {
    /// The size of the FFTs to be run
    size: usize,
    /// The number of channels
    channel_count: usize,
    /// The plans for each channel
    channel_plans: Vec<FftwPlan>
}


impl MultiChannelFft {
    //// Create and initialize a new MultiChannelFft
    fn new(size: usize, channel_count: usize) -> MultiChannelFft {
        let mut channel_plans: Vec<FftwPlan> = Vec::with_capacity(channel_count);

        for _ in (0..channel_count) {
            channel_plans.push(FftwPlan::new(size));
        }
        channel_plans.shrink_to_fit();

        MultiChannelFft {
            size: size,
            channel_count: channel_count,
            channel_plans: channel_plans,
        }
    }

    /// Return a borrowed reference to the plan for a channel
    fn get_channel<'a>(&'a self, index: usize) -> Option<&'a FftwPlan> {
        self.channel_plans.get(index)
    }

    /// Return a mutable borrowed reference to the plan for a channel
    fn get_channel_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut FftwPlan> {
        self.channel_plans.get_mut(index)
    }

    /// Execute all of the FFT channels
    fn execute(&mut self) {
        for plan in self.channel_plans.iter_mut() {
            plan.execute();
        }
    }

}

/// Determine if a number is a power of two
fn is_power_of_two(x: usize) -> bool {
    (x != 0) && ((x & (x - 1)) == 0)
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


/// Audio FFT for 16bit little endian audio data (S16LE)
pub struct AudioFft {
    /// The multichannel fft object that does the work for us
    multichan_fft: MultiChannelFft,
    /// The input cursor indicates how much data has been read in. Input is
    /// 16bit integers, inerleaved by channels. The so that means the maximum
    /// value of input_cursor is channel_count * fft_size
    input_cursor: usize,
    /// The size of the FFT
    fft_size: usize,
    /// The number of audio channels. Ex: 2 for stereo audio.
    channel_count: usize,
    /// Helper for executing the Hanning window function as data is inserted
    hanning: HanningWindowCalculator,
    /// Holds output for the combined channels
    output: Vec<f64>
}


impl AudioFft {
    /// Create a new AudioFft
    pub fn new(fft_size: usize, channel_count: usize) -> AudioFft {
        let mut out_vec = Vec::with_capacity(fft_size/2);
        for _ in (0..fft_size/2) {
            out_vec.push(0.0);
        }
        AudioFft {
            multichan_fft: MultiChannelFft::new(fft_size, channel_count),
            input_cursor: 0,
            fft_size: fft_size,
            channel_count: channel_count,
            hanning: HanningWindowCalculator::new(fft_size),
            output: out_vec,
        }
    }

    pub fn execute(&mut self) {
        self.multichan_fft.execute();
        self.input_cursor = 0;
    }


    /// Allows a client to feed data into the FFT in chunks. This is useful for
    /// ineracting with PulseAudio because its asynchronous API gives audio data
    /// in seemingly arbitrary chunks.
    /// Returns the number of bytes it read. If the number of bytes returned is
    /// less than the input size, the FFT is ready to execute.
    pub fn feed_data(&mut self, input: &[i16]) -> usize {
        let mut bytes_read: usize = 0;
        let total_input = self.channel_count * self.fft_size;

        let mut inputs: Vec<&mut [f64]> = Vec::with_capacity(self.channel_count);
        for channel in self.multichan_fft.channel_plans.iter_mut() {
            inputs.push(channel.get_input_slice());
        }

        for value in input.iter() {
            if self.input_cursor == total_input {
                return bytes_read;
            }
            // The channel number the current value is for
            let channel_num = self.input_cursor % self.channel_count;
            // The index of the value in its channel
            let channel_index = self.input_cursor / self.channel_count;
            inputs[channel_num][channel_index] = self.hanning.get_value(channel_index, *value as f64);
            bytes_read += 1;
            self.input_cursor += 1;
        }

        bytes_read
    }

    pub fn feed_u8_data(&mut self, input: &[u8]) -> usize {
        let i16_ptr: *const i16 = input.as_ptr() as *const i16;
        self.feed_data(unsafe{ slice::from_raw_buf(&i16_ptr, input.len()/2) }) * 2
    }


    /// Computes the combined output of all channels
    pub fn compute_output(&mut self) {
        let mut first = true;
        for channel in self.multichan_fft.channel_plans.iter() {
            for (index, &value) in channel.output.as_slice().slice_to(self.fft_size/2).iter().enumerate() {
                // Turn the FFT output value into decibals
                let power: f64 = 20.0 * value.abs().log10();
                // If it's bigger than the biggest value for this channel for
                // this execution, then replace the current value
                if first || power > self.output[index] {
                    self.output[index] = power;
                }
            }
            first = false;
        }
    }

    /// Borrow the combined output vector
    pub fn get_output(&self) -> &Vec<f64> {
        &self.output
    }
}

unsafe impl Send for AudioFft {}


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
