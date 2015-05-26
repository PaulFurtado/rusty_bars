#![allow(unstable)]

//use std::num::Float;


#[repr(C)]
/// An opaque pointer to an FFTW C plan
pub struct fftw_plan;


#[repr(C)]
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
#[derive(Clone, Copy)]
/// Represents a 64-bit complex number.
pub struct FftwComplex {
    pub re: f64,
    pub im: f64
}

impl FftwComplex {
    /// Get the absolute value (distance from zero) of the complex number
    pub fn abs(&self) -> f64 {
        ((self.re * self.re) + (self.im * self.im)).sqrt()
    }
}
