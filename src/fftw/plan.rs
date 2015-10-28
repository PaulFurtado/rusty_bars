extern crate libc;
use fftw::types::{FftwComplex, PlannerFlags};
use fftw::ext;
use fftw::aligned_array::FftwAlignedArray;


/// Rust wrapper for an FFTW plan
pub struct FftwPlan {
    input: FftwAlignedArray<f64>,
    output: FftwAlignedArray<FftwComplex>,
    size: usize,
    plan: *mut ext::FftwPlan,
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
        &self.output.as_slice()[0..(self.size/2)]
    }
}

/// Unsafe because it has lifetimes.
impl Drop for FftwPlan {
    /// Runds fftw_destroy plan when a plan goes out of scope
    fn drop(&mut self) {
        unsafe { ext::fftw_destroy_plan(self.plan); }
    }
}


/// Determine if a number is a power of two
fn is_power_of_two(x: usize) -> bool {
    (x != 0) && (x != 1) && ((x & (x - 1)) == 0)
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
