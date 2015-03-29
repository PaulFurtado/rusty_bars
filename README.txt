to compile:
    rustc pacat.rs

to find your <device name>:
    pactl info | grep 'Default Sink'

to record to stdout:
    ./pacat <device name>.monitor

to play form a recorded file:
    <testaudio ./pacat <device_name>
