extern crate libc;
extern crate rusty_bars;

use rusty_bars::pulse::PulseAudioMainloop;
use rusty_bars::viz_runner::VizRunner;

/// Start the visualizer for your default PulseAudio output.
fn main() {
    let mainloop = PulseAudioMainloop::new();
    VizRunner::new(&mainloop);
    mainloop.run();
}
