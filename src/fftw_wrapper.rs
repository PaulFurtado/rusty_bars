#![allow(unstable)]
#![feature(unsafe_destructor)]


extern crate libc;
use self::libc::{c_int, size_t, c_void};
use std::num::Float;
use std::f64::consts::PI;
use std::{ptr, mem, slice};


/// External functions for interacting with FFTW
mod ext {
    extern crate libc;
    use self::libc::{c_int, size_t, c_void};
    use super::{FftwPlan, FftwComplex};

    #[repr(C)]
    #[derive(Copy)]
    /// An opaque pointer to an FFTW C plan
    pub struct fftw_plan;

    #[link(name="fftw3")]
    extern {
        /// Creates a 1-dimensional real-to-complex FFT plan
        pub fn fftw_plan_dft_r2c_1d(n: c_int, input: *mut f64, output: *mut FftwComplex, flags: c_int) -> *const fftw_plan;

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
pub enum PlannerFlags {
    MEASURE = 0,
    DESTROY_INPUT = 1 << 0,
    UNALIGNED = 1 << 1,
    CONSERVE_MEMORY = 1U << 2,
    EXHAUSTIVE = 1U << 3,
    PRESERVE_INPUT = 1U << 4,
    PATIENT = 1U << 5,
    ESTIMATE = 1U << 6,
    WISDOM_ONLY = 1U << 21,
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
struct FftwAlignedArray<'a, T> {
    len: usize,
    ptr: *const T,
    mut_ptr: *mut T,
}

impl<'a, T> FftwAlignedArray<'a, T> {
    /// Create a new FftwAlignedArray.
    /// Len is the number of elements, not the size in bytes.
    /// Panics if memory allocation fails.
    fn new(len: usize) -> FftwAlignedArray<'a, T> {
        let ptr: *mut T = fftw_ralloc::<T>(len).unwrap();
        FftwAlignedArray {
            len: len,
            ptr: ptr as *const T,
            mut_ptr: ptr
        }
    }

    /// Get an immutable raw pointer to the memory backing this array
    fn as_ptr(&'a self) -> *const T {
        self.ptr
    }

    /// Get an mutable raw pointer to the memory backing this array
    fn as_mut_ptr(&'a self) -> *mut T {
        self.mut_ptr
    }

    /// Modify the contents of this array via a mutable slice
    fn as_mut_slice(&'a mut self) -> &'a mut [T] {
        unsafe{ slice::from_raw_mut_buf(&self.mut_ptr, self.len) }
    }
}


impl<'a, T> AsSlice<T> for FftwAlignedArray<'a, T> {
    /// Access the contents of this array via an immutable slice
    fn as_slice(&self) -> &[T] {
        unsafe{ slice::from_raw_buf(&self.ptr, self.len) }
    }
}


#[unsafe_destructor]
/// Unsafe because it has lifetimes.
impl<'a, T> Drop for FftwAlignedArray<'a, T> {
    /// Free the array with the right deallocator
    fn drop(&mut self) {
        unsafe{ ext::fftw_free(self.mut_ptr as *mut c_void) };
    }
}


/// Rust wrapper for an FFTW plan
pub struct FftwPlan<'a> {
    input: FftwAlignedArray<'a, f64>,
    output: FftwAlignedArray<'a, FftwComplex>,
    size: usize,
    plan: *const ext::fftw_plan,
}


impl<'a> FftwPlan<'a> {
    /// Create a new wrapper around an FFTW plan
    pub fn new(size: usize) -> FftwPlan<'a> {
        if !is_power_of_two(size) {
            panic!("FFT size should be a power of two!");
        }

        let input = FftwAlignedArray::new(size);
        let output = FftwAlignedArray::new(size);

        let plan = unsafe {
            ext::fftw_plan_dft_r2c_1d(
                size as i32,
                input.as_mut_ptr(),
                output.as_mut_ptr(),
                FFTW_MEASURE
            )
        };

        FftwPlan2 {
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
    pub fn get_input_slice(&'a mut self) -> &'a mut [f64] {
        self.input.as_mut_slice()
    }

    /// Get a slice of the FFTW plan's output buffer
    pub fn get_output_slice(&'a self) -> &'a [FftwComplex] {
        // A real FFT outputs half of the input size.
        self.output.as_slice().slice_to(self.size/2)
    }
}

#[unsafe_destructor]
/// Unsafe because it has lifetimes.
impl<'a> Drop for FftwPlan {
    /// Runds fftw_destroy plan when a plan goes out of scope
    fn drop(&'a mut self) {
        unsafe { ext::fftw_destroy_plan(self.plan); }
    }
}



/// An FFT for multiple channels of data.
struct MultiChannelFft<'a> {
    /// The size of the FFTs to be run
    size: usize,
    /// The number of channels
    channel_count: usize,
    /// The plans for each channel
    channel_plans: Vec<FftwPlan<'a>>
}


impl<'a> MultiChannelFft<'a> {
    //// Create and initialize a new MultiChannelFft
    fn new(size: usize, channel_count: usize) -> MultiChannelFft<'a> {
        let mut channel_plans: Vec<FftwPlan> = Vec::with_capacity(channel_count);

        for _ in (0..channel_count) {
            channel_plans.push(FftwPlan2::new(size));
        }
        channel_plans.shrink_to_fit();

        MultiChannelFft {
            size: size,
            channel_count: channel_count,
            channel_plans: channel_plans,
        }
    }

    /// Return a borrowed reference to the plan for a channel
    fn get_channel(&'a self, index: usize) -> Option<&'a FftwPlan2> {
        self.channel_plans.get(index)
    }

    /// Return a mutable borrowed reference to the plan for a channel
    fn get_channel_mut(&'a mut self, index: usize) -> Option<&'a mut FftwPlan2> {
        self.channel_plans.get_mut(index)
    }
}
