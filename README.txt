to compile:
    rustc rust_simple_pulse.rs

to find your <device name>:
    pactl info | grep 'Default Sink'

to record to stdout:
    ./rust_simple_pulse <device name>.monitor

to play form a recorded file:
    <testaudio ./rust_simple_pulse <device_name>
