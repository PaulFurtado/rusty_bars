rusty_bars
----------

A PulseAudio music visualizer written in Rust.

Usage:
    rusty_bars takes no arguments. At startup it detects the current PulseAudio
    output device and begins visualizing its output. If the system's default
    audio output changes, the visualizer will change automatically.

Description:
    This is a text-based audio visualizer that runs in your terminal. It reads
    audio from your system's default audio output, runs an FFT on it using the
    FFTW library, and displays the visual using ncurses.

Building:
    Simply run "cargo build". This project depends on libpulse, ncurses, and
    FFTW, however these packages are likely already installed on any desktop
    linux distribution.

Background:
    This was our final project for the course "Building Extensible Systems" at
    Northeastern University, an experimental course in Rust taught by Jesse Tov
    and Matthias Felleisen. Ali Ukani was my partner. The purpose of this
    project was to experiment with the foreign function interface.

    Ali: http://ali.io/  https://github.com/ali
    The course: http://www.ccs.neu.edu/home/matthias/4620-s15/index.html

OS X:
    There's a chance that this works on OS X using the PulseAudio OS X port,
    however, we haven't tried it. It would be cool to make this work on OS X
    using soundflower to get audio data.

Other notes:
    One of the goals of this project is to use this code to drive LED
    visualizations. We've done a good deal of this in Python, but Rust is
    lower-level and can provide better latency, especially when running on
    embedded devices with limited CPUs.

    The PulseAudio wrapping code in this is pretty dirty. The PulseAudio C API
    is entirely async and we wanted to use Rust closures for the callbacks.
    Our implementation involves circular references and reference counters,
    drop is not properly implemented on any of the structs, and none of it is
    thread safe. If I have time, I'm definitely interested in cleaning up this
    code and building a PulseAudio crate since this would open up Rust to be
    used by audio applications for linux.

    The FFTW wrapping code is fairly clean and provides a realistic abstraction
    over FFTW plans. It currently only supports the fftw_plan_dft_r2c_1d plan
    function, but it would be trivial to support the rest of them. I'm planning
    to build an FFTW crate when I get a chance.

    Our FFT-related math may not be totally correct; when we first started this,
    we had no idea how to use FFTs, but watching the visualizer while playing
    tones in a tone generator seems to confirm that it is mostyl correct. If
    you notice anything wrong, please tell us!
