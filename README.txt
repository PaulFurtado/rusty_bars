to compile:
    cargo build

to find your <device name>:
    export DEFAULT_SINK=$(pactl info | grep 'Default Sink' | cut -b 15-)
    echo $DEFAULT_SINK

to run:
    cargo run "$DEFAULT_SINK".monitor
