to compile:
    cargo build

to find your <device name>:
    export DEFAULT_SINK=$(pactl info | grep 'Default Sink' | cut -b 15-)
    echo $DEFAULT_SINK

to record to stdout:
    cargo run record "$DEFAULT_SINK".monitor

to play form a recorded file:
    cargo run play "$DEFAULT_SINK" <src/testaudio
