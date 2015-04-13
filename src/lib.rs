#![allow(unstable)]
#![feature(unsafe_destructor)] // For destructing parameterized types
#![allow(unstable_features)] // To suppress the warning of unsafe_destructor

pub mod ncurses_wrapper;
pub mod pulse;
pub mod visualizer;
pub mod fftw;
pub mod viz_runner;
