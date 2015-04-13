#![allow(unstable)]

extern crate libc;
extern crate rust_pulse;

use self::libc::{c_int, size_t};
use rust_pulse::pulse::*;
use rust_pulse::pulse_types::*;
use rust_pulse::visualizer;
use rust_pulse::fftw_wrapper;
use rust_pulse::stream::PulseAudioStream;
use std::rc::Rc;
use std::cell::RefCell;

macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


#[derive(Clone)]
struct VizRunner<'a> {
    internal: Rc<RefCell<VizRunnerInternal<'a>>>
}


impl<'a> VizRunner<'a> {
    fn new(mainloop: &'a PulseAudioMainloop) ->  VizRunner<'a> {
        let vzr = VizRunner {
            internal: Rc::new(RefCell::new(VizRunnerInternal::new(mainloop)))
        };
        {
            //let internal_rc = vzr.internal.clone();
            let clone = vzr.clone();
            let mut internal = vzr.internal.borrow_mut();
            internal.external = Some(clone);
        }
        vzr
    }

    pub fn connect(&mut self) {
        let mut internal = self.internal.borrow_mut();
        println!("sucessful borrow.");
        internal.connect();
        println!("connect ran!");
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
    pub fn new(mainloop: &'a PulseAudioMainloop) -> VizRunnerInternal<'a> {
        let context = mainloop.create_context("rs_client");
        VizRunnerInternal {
            context: context,
            fft: fftw_wrapper::AudioFft::new(1024, 2),
            viz: visualizer::Visualizer::new(),
            external: None,
            stream: None
        }
    }


    pub fn subscribe_to_sink_changes(&mut self) {
        let mut external = self.external.clone().unwrap();
        self.context.set_event_callback(move |context, event, index| {
            let facility = event & (pa_subscription_event_type::FACILITY_MASK as c_int);
            let ev_type = event & (pa_subscription_event_type::TYPE_MASK as c_int);

            if facility == pa_subscription_event_type::SERVER as c_int {
                if ev_type == pa_subscription_event_type::CHANGE as c_int {
                    let mut internal = external.internal.borrow_mut();
                    internal.update_sink();
                }
            }
        });

        self.context.add_subscription(pa_subscription_mask::SERVER, move |context, success| {
            if !success {
                println_stderr!("failed to subscribe to server changes!");
            }
        });

    }

    pub fn connect(&mut self) {
        let mut external = self.external.clone().unwrap();
        self.context.set_state_callback(move |context, state| {
            match state {
                pa_context_state::READY => {
                    external.internal.borrow_mut().on_ready()
                },
                _ => {}
            }
        });

        self.context.connect(None, pa_context_flags::NOAUTOSPAWN);
    }

    pub fn on_ready(&mut self) {
        self.update_sink();
        self.subscribe_to_sink_changes();
    }

    pub fn update_sink(&mut self) {
        let mut external = self.external.clone().unwrap();
        self.context.get_server_info(move |context, info| {
            let internal = external.internal.borrow();
            let external = external.clone();
            internal.context.get_sink_info_by_name(info.get_default_sink_name(), move |context, info| {
                match info {
                    Some(info) => {
                        let mut internal = external.internal.borrow_mut();
                        internal.on_new_sink(info.get_monitor_source_name());
                    },
                    None => {}
                }
            });
        });
    }

    /// Called whenever we change the sink
    pub fn on_new_sink(&mut self, monitor_name: &str) {
        match self.stream {
            Some(ref mut stream) => { stream.disconnect(); },
            None => {}
        }
        self.stream = None;

        // TODO: sample spec should be constant
        let sample_spec = pa_sample_spec {
            format:pa_sample_format::PA_SAMPLE_S16LE,
            rate: 44100,
            channels: 2
        };

        let mut stream = self.context.create_stream("rs_client", &sample_spec, None);
        let mut external = self.external.clone().unwrap();
        println_stderr!("new stream addr: {:x}", stream.get_raw_ptr() as usize);
        stream.set_read_callback(move |mut stream, nbytes| {
            let mut internal = external.internal.borrow_mut();
            internal.stream_read_callback(stream, nbytes);
        });
        stream.connect_record(Some(monitor_name), None, None);
        self.stream = Some(stream);
    }

    /// Called whenever the FFT has enough data to run a frame of the visualizer
    pub fn on_fft_frame_ready(&mut self) {
        self.fft.execute();
        self.fft.compute_output();
        self.viz.render_frame(self.fft.get_output()).unwrap();
    }

    /// Called whenever the stream is ready to be read.
    pub fn stream_read_callback(&mut self, mut stream: PulseAudioStream, nbytes: size_t) {
        // Ignore the callback if the stream just changed.
        match self.stream {
            Some(ref s) => {
                if s.get_raw_ptr() != stream.get_raw_ptr() {
                    // disconnect frequently fails if the stream is in the wrong state,
                    // so if we got data for a stale stream, try disconnecting it again
                    stream.disconnect();
                    return;
                }
            },
            None => {
                return;
            }
        }

        match stream.peek() {
            Ok(data) => {
                let mut fed_count: usize = 0;
                let mut iterations: usize = 0;
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
    let mut viz = VizRunner::new(&mainloop);
    println_stderr!("connecting");
    viz.connect();
    println_stderr!("starting the loop");
    mainloop.run();
}
