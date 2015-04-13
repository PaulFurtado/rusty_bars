#![allow(unstable)]

extern crate libc;
extern crate rust_pulse;

use rust_pulse::pulse::PulseAudioMainloop;
use rust_pulse::viz_runner::VizRunner;


fn main() {
    let mainloop = PulseAudioMainloop::new();
    VizRunner::new(&mainloop);
    mainloop.run();
}
