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
}


impl<'a> VizRunnerInternal<'a> {
    pub fn new(mainloop: &'a PulseAudioMainloop) -> VizRunnerInternal<'a> {
        let context = mainloop.create_context("rs_client");
        VizRunnerInternal {
            context: context,
            fft: fftw_wrapper::AudioFft::new(1024, 2),
            viz: visualizer::Visualizer::new(),
            external: None
        }
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

    pub fn on_new_sink(&mut self, monitor_name: &str) {
        println_stderr!("new sink: {}", monitor_name);
        let sample_spec = pa_sample_spec {
            format:pa_sample_format::PA_SAMPLE_S16LE,
            rate: 44100,
            channels: 2
        };

        let mut stream = self.context.create_stream("rs_client", &sample_spec, None);

        let mut external = self.external.clone().unwrap();

        stream.set_read_callback(move |mut stream, nbytes| {
            let internal = external.internal.borrow_mut();
            internal.stream_read_callback(stream, nbytes);
        });

    }

    pub fn stream_read_callback(&mut self, stream: PulseAudioStream, nbytes: size_t) {
        println!("got data");

    }


}



fn main_new() {
    let mainloop = PulseAudioMainloop::new();
    let mut viz = VizRunner::new(&mainloop);
    println_stderr!("connect");
    viz.connect();
    println_stderr!("starting the loop");
    mainloop.run();
}


fn main() {
    main_new();
    return;
    let mainloop = PulseAudioMainloop::new();
    let mut context = mainloop.create_context("rs_client");
    /*
    context.set_state_callback(move |context, state| {
        match state {
            pa_context_state::READY => {
                context.get_server_info(move |context, info| {
                    context.get_sink_info_by_name(info.get_default_sink_name(), move |context, info| {
                        match info {
                            Some(info) => {
                                let sample_spec = pa_sample_spec {
                                    format:pa_sample_format::PA_SAMPLE_S16LE,
                                    rate: 44100,
                                    channels: 2
                                };

                                let mut vis = visualizer::Visualizer::new();
                                let mut fft = fftw_wrapper::AudioFft::new(1024, 2);
                                let mut stream = context.create_stream("rs_client", &sample_spec, None);
                                stream.set_read_callback(move |mut stream, nbytes| {
                                    match stream.peek() {
                                        Ok(data) => {
                                            let mut fed_count: usize = 0;
                                            let mut iterations: usize = 0;
                                            while fed_count < data.len() {
                                                fed_count += fft.feed_u8_data(data);
                                                println_stderr!("iteration: {}, bytes: {}", iterations, data.len());
                                                if fed_count < data.len() {
                                                    println_stderr!("executing!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!");
                                                    fft.execute();
                                                    fft.compute_output();
                                                    vis.render_frame(fft.get_output()).unwrap();

                                                } else {
                                                    println_stderr!("not executing.");
                                                }
                                            }
                                        },
                                        Err(_) => return
                                    }
                                    stream.drop_fragment().unwrap();
                                });

                                stream.connect_record(Some(info.get_monitor_source_name()), None, None);
                                return;

                                context.set_event_callback(move |context, event, index| {
                                    let facility = (event & (pa_subscription_event_type::FACILITY_MASK as c_int));
                                    let ev_type = (event & (pa_subscription_event_type::TYPE_MASK as c_int));

                                    if facility == pa_subscription_event_type::SERVER as c_int {
                                        if ev_type == pa_subscription_event_type::CHANGE as c_int {
                                            context.get_server_info(move |context, info| {
                                                println!("new output: {}", info.get_default_sink_name());
                                            });
                                        }
                                    }
                                });

                                context.add_subscription(pa_subscription_mask::SERVER, move |context, success| {
                                    if !success {
                                        println!("failed to subscribe to event!");
                                    }
                                });
                            },
                            None => {}
                        }
                    });
                });
            },
            _ => {}
        }
    });
    */
    context.connect(None, pa_context_flags::NOAUTOSPAWN);
    mainloop.run();
}
