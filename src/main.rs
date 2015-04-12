#![allow(unstable)]
#![feature(unsafe_destructor)]

// TODO: Automatically get default sink

extern crate libc;

use self::libc::{c_int, c_char, size_t};
use std::ptr;
use std::os;
use std::mem;
use std::mem::transmute;
use std::ffi::CString;
use std::str::from_utf8;
use libc::funcs::c95::string::strlen;
use std::cmp::max;
pub mod analyze_spectrum;
pub mod visualizer;
mod fftw_wrapper;
mod ncurses_wrapper;
mod pulse_types;
mod pulse;



#[link(name="pulse-simple")]
#[link(name="pulse")]
#[link(name="fftw3")]
extern {
    fn pa_simple_new(
        server: *const c_char,
        name: *const c_char,
        dir: c_int,
        dev: *const c_char,
        steam_name: *const c_char,
        sample_spec: *const PulseSampleSpec,
        channel_map: *const u8,
        attr: *const u8,
        error: *mut c_int
    ) -> *mut PaSimpleC;

    fn pa_simple_free(pa: *mut PaSimpleC);

    fn pa_simple_write(
        pa: *mut PaSimpleC,
        data: *const u8,
        bytes: size_t,
        error: *mut c_int
    ) -> c_int;

    fn pa_simple_read(
        pa: *mut PaSimpleC,
        data: *mut u8,
        bytes: size_t,
        error: *mut c_int
    ) -> c_int;


    fn pa_simple_drain(
        pa: *mut PaSimpleC,
        error: *mut c_int
    ) -> c_int;

    fn pa_strerror(error: c_int) -> *const c_char;
}


#[repr(C)]
#[derive(Copy)]
pub struct PaSimpleC;


#[derive(Copy,Clone)]
#[repr(C)]
pub enum StreamDirection {
    NoDirection,
    StreamPlayback,
    StreamRecord,
    StreamUpload
}



// see pa_sample_format
pub static PA_SAMPLE_S16LE: c_int = 3_i32;


#[derive(Copy)]
#[repr(C)]
pub struct PulseSampleSpec {
  format: c_int,
  rate: u32,
  channels: u8
}



fn pa_err_to_string(err: c_int) -> Result<(), String> {
    if err == 0 {
        Ok(())
    } else {
        unsafe {
            let err_msg_ptr: *const c_char = pa_strerror(err);
            let size = strlen(err_msg_ptr) as usize;
            let slice: Vec<u8> = Vec::from_raw_buf((err_msg_ptr as *const u8), size);
            Err(String::from_utf8(slice).unwrap())
        }
    }
}



pub struct PulseSimple {
    pa: *mut PaSimpleC
}


impl PulseSimple {

    pub fn new(device: &str, mode: StreamDirection, sample_spec: &PulseSampleSpec) -> Result<PulseSimple, String> {
        let pa_name_c = CString::from_slice("rustviz".as_bytes());
        let stream_name_c = CString::from_slice("playback".as_bytes());
        let dev_c = CString::from_slice(device.as_bytes());
        let mut err: c_int = 0;

        let pa = unsafe {
            pa_simple_new(
              ptr::null(),
              pa_name_c.as_ptr(),
              mode as c_int,
              dev_c.as_ptr(),
              stream_name_c.as_ptr(),
              transmute(sample_spec),
              ptr::null(),
              ptr::null(),
              &mut err
            )
        };

        try!(pa_err_to_string(err));
        Ok(PulseSimple{pa: pa})
    }

    /// Read some data from the server
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<(), String> {
        let mut err: c_int = 0;
        unsafe { pa_simple_read(
            self.pa,
            buffer.as_mut_ptr(),
            buffer.len() as size_t,
            &mut err
        ) };

        pa_err_to_string(err)
    }

    pub fn write(&mut self, buffer: &[u8], count: size_t) -> Result<(), String> {
        let mut err: c_int = 0;
        unsafe { pa_simple_write(
            self.pa,
            buffer.as_ptr(),
            count as size_t,
            &mut err
        ) };

        pa_err_to_string(err)
    }

    pub fn drain(&mut self) -> Result<(), String> {
        let mut err: c_int = 0;
        unsafe { pa_simple_drain(self.pa, &mut err) };
        pa_err_to_string(err)
    }

}


impl Drop for PulseSimple {
    fn drop(&mut self) {
        unsafe { pa_simple_free(self.pa); };
    }
}


fn simple_run_analyzer(dev: &str) {
    // The sample spec to record from pulseaudio at
    let sample_spec = PulseSampleSpec{
        format: PA_SAMPLE_S16LE,
        rate: 44100,
        channels: 2
    };

    let mut vis = visualizer::Visualizer::new();
    let width = vis.get_width();
    // TODO: compute_input_size is totally broken.
    let fft_size = analyze_spectrum::compute_input_size(max(width, 512));


    let mut fft = fftw_wrapper::AudioFft::new(1024, 2);



    // Initialize the FFT first so that we don't hold up pulseaudio while
    // waiting for the FFT planner
    //let mut fft = analyze_spectrum::AudioFFT::new(fft_size, 2);

    // Initialize the buffer we use for reading from pulse audio
    let mut buffer_vec: Vec<u8> = Vec::with_capacity(2048);
    for _ in range(0, 2048) {
        buffer_vec.push(0);
    }
    let mut buffer = buffer_vec.as_mut_slice();

    // Initialize pulseaudio
    let mut pulse = PulseSimple::new(dev, StreamDirection::StreamRecord, &sample_spec).unwrap();


    loop {
        pulse.read(buffer).unwrap();
        let mut total: usize = buffer.len();
        let mut processed: usize = 0;

        let mut count = 0;
        loop {
            processed += fft.feed_u8_data(buffer.slice_from(processed));

            if processed < total {
                fft.execute();
                fft.compute_output();
                vis.render_frame(fft.get_output()).unwrap();
            } else {
                break;
            }
        }

    }
}




fn main() {
    use pulse::*;



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


                                let sample_spec = pulse_types::structs::pa_sample_spec {
                                    format: PA_SAMPLE_S16LE,
                                    rate: 44100,
                                    channels: 2
                                };


                                let mut vis = visualizer::Visualizer::new();
                                let width = vis.get_width();
                                let mut fft = fftw_wrapper::AudioFft::new(1024, 2);

                                let mut stream = context.create_stream("rs_client", &sample_spec, None);

                                stream.set_read_callback(move |mut stream, nbytes| {
                                    //let foo: &[u8] = stream.peek().unwrap();
                                    fft.feed_u8_data(stream.peek().unwrap());
                                    fft.execute();
                                    fft.compute_output();
                                    vis.render_frame(fft.get_output()).unwrap();
                                });


                                stream.connect_record(Some(info.get_monitor_source_name()), None, None);

                                return;

                                //simple_run_analyzer(info.get_monitor_source_name());
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
