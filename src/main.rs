#![allow(unstable)]

// TODO: Automatically get default sink

extern crate libc;
extern crate rust_pulse;

use self::libc::{c_int, c_char, size_t};

use std::{mem, os, ptr};
use std::mem::transmute;
use std::ffi::CString;
use std::str::from_utf8;
use libc::funcs::c95::string::strlen;
use std::cmp::max;

use rust_pulse::pulse::*;
use rust_pulse::stream::*;
use rust_pulse::pulse_types::*;
use rust_pulse::visualizer;
use rust_pulse::fftw_wrapper;


macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);




fn main() {
    let mainloop = PulseAudioMainloop::new();
    let mut context = mainloop.create_context("rs_client");
    context.set_state_callback(move |mut context, state| {
        println!("state: {}", state as c_int);
        match state {
            pa_context_state::READY => {
                println!("calling!");
                context.get_server_info(move |context, info| {
                    println!("===================== server_info_callback =======================");
                    println!("user_name: {}", info.get_user_name());
                    println!("host_name: {}", info.get_host_name());
                    println!("server_version: {}", info.get_server_version());
                    println!("server_name: {}", info.get_server_name());
                    println!("default_sink_name: {}", info.get_default_sink_name());
                    println!("default_source_name: {}", info.get_default_source_name());
                    println!("===================== end server_info_callback =======================");
                    println!("\n\n");
                    context.get_sink_info_by_name(info.get_default_sink_name(), move |context, info| {
                        match info {
                            Some(info) => {
                                println!("===================== sink_info_callback =======================");
                                println!("name: {}", info.get_name());
                                println!("description: {}", info.get_description());
                                println!("monitor_source: {}", info.get_monitor_source_name());
                                println!("monitor_source index: {}", info.monitor_source);
                                println!("driver: {}", info.get_driver());
                                println!("===================== end sink_info_callback =======================");

                                //simple_run_analyzer(info.get_monitor_source_name());
                                //return;

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
    context.connect(None, pa_context_flags::NOAUTOSPAWN);
    mainloop.run();

}
