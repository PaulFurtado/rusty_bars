#![allow(unstable)]
#![feature(unsafe_destructor)]

extern crate libc;
use self::libc::{c_int, size_t, c_void};
use std::num::Float;
use std::f64::consts::PI;
use std::{ptr, mem, slice};




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
