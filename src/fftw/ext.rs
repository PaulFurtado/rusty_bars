extern crate libc;
use self::libc::{c_int, size_t, c_void};
use fftw::types::*;


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
