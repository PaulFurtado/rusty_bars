#![allow(unstable)]

extern crate libc;
extern crate rust_pulse;

use self::libc::{c_int, size_t};
use std::rc::Rc;
use std::cell::RefCell;

use rust_pulse::fftw_wrapper;
use rust_pulse::pulse::{Context, PulseAudioMainloop, PulseAudioStream};
use rust_pulse::pulse::types::*;
use rust_pulse::visualizer;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


const DEFAULT_SAMPLE_SPEC: pa_sample_spec = pa_sample_spec {
    format:pa_sample_format::PA_SAMPLE_S16LE,
    rate: 44100,
    channels: 2
};


#[derive(Clone)]
/// The culmination of all of the visualizer parts
struct VizRunner<'a> {
    internal: Rc<RefCell<VizRunnerInternal<'a>>>
}


impl<'a> VizRunner<'a> {
    /// Create a new visuaizer
    fn new(mainloop: &'a PulseAudioMainloop) ->  VizRunner<'a> {
        let vzr = VizRunner {
            internal: Rc::new(RefCell::new(VizRunnerInternal::new(mainloop)))
        };
        {
            let clone = vzr.clone();
            let mut internal = vzr.internal.borrow_mut();
            internal.external = Some(clone);
            internal.connect();
        }
        vzr
    }
}




struct VizRunnerInternal<'a> {
    context: Context<'a>,
    fft: fftw_wrapper::AudioFft,
    viz: visualizer::Visualizer,
    external: Option<VizRunner<'a>>,
    stream: Option<PulseAudioStream<'a>>,
}


impl<'a> VizRunnerInternal<'a> {
    /// Create a new instance of the VizRunnerInternal struct
    fn new(mainloop: &'a PulseAudioMainloop) -> VizRunnerInternal<'a> {
        let context = mainloop.create_context("rs_client");
        VizRunnerInternal {
            context: context,
            fft: fftw_wrapper::AudioFft::new(1024, 2),
            viz: visualizer::Visualizer::new(),
            external: None,
            stream: None
        }
    }

    /// Subscribe to the default sink changing on the server
    fn subscribe_to_sink_changes(&mut self) {
        let external = self.external.clone().unwrap();

        self.context.set_event_callback(move |_, event, _| {
            let facility = event & (pa_subscription_event_type::FACILITY_MASK as c_int);
            let ev_type = event & (pa_subscription_event_type::TYPE_MASK as c_int);

            if facility == pa_subscription_event_type::SERVER as c_int {
                if ev_type == pa_subscription_event_type::CHANGE as c_int {
                    let mut internal = external.internal.borrow_mut();
                    internal.update_sink();
                }
            }
        });

        self.context.add_subscription(pa_subscription_mask::SERVER, move |_, success| {
            if !success {
                println_stderr!("failed to subscribe to server changes!");
            }
        });
    }

    /// Connect to PulseAudio
    fn connect(&mut self) {
        let external = self.external.clone().unwrap();
        self.context.set_state_callback(move |_, state| {
            match state {
                pa_context_state::READY => {
                    external.internal.borrow_mut().on_ready()
                },
                _ => {}
            }
        });

        self.context.connect(None, pa_context_flags::NOAUTOSPAWN);
    }

    /// Callled when the context is ready
    fn on_ready(&mut self) {
        self.update_sink();
        self.subscribe_to_sink_changes();
    }

    /// Gets the monitor for the current default sink and then calls set_sink
    fn update_sink(&mut self) {
        let external = self.external.clone().unwrap();
        self.context.get_server_info(move |_, info| {
            let internal = external.internal.borrow();
            let external = external.clone();
            internal.context.get_sink_info_by_name(info.get_default_sink_name(), move |_, info| {
                match info {
                    Some(info) => {
                        let mut internal = external.internal.borrow_mut();
                        internal.set_sink(info.get_monitor_source_name());
                    },
                    None => {}
                }
            });
        });
    }

    /// Switches to a new sink
    fn set_sink(&mut self, monitor_name: &str) {
        match self.stream {
            Some(ref mut stream) => { stream.disconnect(); },
            None => {}
        }
        self.stream = None;

        let mut stream = self.context.create_stream("rs_client", &DEFAULT_SAMPLE_SPEC, None);
        let external = self.external.clone().unwrap();

        stream.set_read_callback(move |stream, nbytes| {
            let mut internal = external.internal.borrow_mut();
            internal.stream_read_callback(stream, nbytes);
        });
        stream.connect_record(Some(monitor_name), None, None).unwrap();
        self.stream = Some(stream);
    }

    /// Called whenever the FFT has enough data to run a frame of the visualizer
    fn on_fft_frame_ready(&mut self) {
        self.fft.execute();
        self.fft.compute_output();
        self.viz.render_frame(self.fft.get_output()).unwrap();
    }

    /// Handles a stale stream and returns true if the stream was stale
    /// A stale stream can occur when switching streams before the current
    /// stream was in the "ready" state. Rather than wasting waiting for the
    /// stream to reach the ready state only to disconnect it, this will get
    /// called the first time the stream has data available and disconnect it
    /// then.
    fn handle_stale_stream(&mut self, stream: &mut PulseAudioStream) -> bool {
        match self.stream {
            Some(ref s) => {
                if s.get_raw_ptr() != stream.get_raw_ptr() {
                    // disconnect frequently fails if the stream is in the wrong state,
                    // so if we got data for a stale stream, try disconnecting it again
                    stream.disconnect();
                    true
                } else {
                    false
                }
            },
            None => {
                false
            }
        }
    }

    /// Handle the callback from PulseAudio telling us that stream data is ready
    fn stream_read_callback(&mut self, mut stream: PulseAudioStream, _: size_t) {
        if self.handle_stale_stream(&mut stream) {
            return
        }

        match stream.peek() {
            Ok(data) => {
                let mut fed_count: usize = 0;
                while fed_count < data.len() {
                    fed_count += self.fft.feed_u8_data(data);
                    if fed_count < data.len() {
                        self.on_fft_frame_ready();
                    }
                }
            },
            Err(_) => return
        }

        stream.drop_fragment().unwrap();
    }
}



fn main() {
    let mainloop = PulseAudioMainloop::new();
    VizRunner::new(&mainloop);
    mainloop.run();
}
