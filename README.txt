rust_pulse
----------

A PulseAudio visualizer written in Rust.

Usage:
    rust_pulse takes no arguments. At startup it detects the current PulseAudio
    output device and begins visualizing its output. If the system's audio
    output changes, the visualizer will change automatically.


Files:

src/
    The main source code directory

    main.rs
        The entry point of rust_pulse

    lib.rs
        Glue for the rust_pulse crate

    ncurses_wrapper.rs
        A wrapper around a small subset of ncurses' features used by the visualizer

    visualizer.rs
        Code for rendering the visualizer

    viz_runner.rs
        The culmination of all parts. Pulls data from PulseAudio, pumps it
        through the FFT, and hands it off to the renderer.

    fftw/
        A module which wraps the FFTW C library.

        ext.rs
            External functions from the FFTW C library.

        types.rs
            Implementations of FFTW's C types in Rust

        alligned_array.rs
            A wrapper around FFTW's properly-aligned arrays for SIMD instructions.

        hanning.rs
            An implementation of the Hanning Window Function which caches some
            of the work.

        plan.rs
            A wrapper around an FFTW plan. A plan is the core of the FFTW
            library and is responsible for executing the FFTs.

        multichannel.rs
            A wrapper around multiple plans, for executing plans on multiple
            channels of input data.

        audio.rs
            A wrapper around multichannel plans specialized for S16LE audio data

    pulse/
        Wrappers for PulseAudio objects

        ext.rs
            External functions from the libpulseaudio C library

        types.rs
            Implementations of PulseAudio's C types

        mainloop.rs
            A wrapper around the PulseAudio mainloop

        context.rs
            A wrapper around a PulseAudio context. A context represents a
            connection to a PulseAudio server

        stream.rs
            A wrapper around a PulseAudio stream

        subscription_manager.rs
            A helper for working with enum masks for PulseAudio event subscriptions

        mod.rs
            Glue for the pulse module
