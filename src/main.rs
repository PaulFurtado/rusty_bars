#![allow(unstable)]

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



fn run_analyzer(dev: &str) {
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

    // Initialize the FFT first so that we don't hold up pulseaudio while
    // waiting for the FFT planner
    let mut fft = analyze_spectrum::AudioFFT::new(fft_size, 2);

    // Initialize the buffer we use for reading from pulse audio
    let mut buffer_vec: Vec<u8> = Vec::with_capacity(fft.get_buf_size());
    for _ in range(0, fft.get_buf_size()) {
        buffer_vec.push(0);
    }
    let mut buffer = buffer_vec.as_mut_slice();

    // Initialize pulseaudio
    let mut pulse = PulseSimple::new(dev, StreamDirection::StreamRecord, &sample_spec).unwrap();


    loop {
        pulse.read(buffer).unwrap();
        let output = fft.execute(buffer);
        //println!("sup");
        match vis.render_frame(output) {
            Err(x) => panic!("error: {}", x),
            Ok(_) => {}
        }
        // Commented out code below is for pumping the data to a client
        //let temp: Vec<String> = output.iter().map(|x| format!("{}", x)).collect();
        //println!("{}", temp.connect(", "));
    }
}

mod async {
    extern crate libc;
    use libc::{c_void, c_int};
    use pulse::*;
    use std::mem::transmute;
    use std::mem;


    extern "C" fn state_callback(context: *mut opaque::pa_context, userdata: *mut c_void) {
        let state = pa_context_get_state(context);

        let myint_ptr = userdata as *mut usize;
        let myint = unsafe{ *myint_ptr };

        println!("i haz state. myint={}, state={}", myint, state as c_int);

        match state {
            enums::pa_context_state::READY => {
                println!("ready to twerk!");
                pa_context_get_sink_info_list(context, sink_list_callback, myint_ptr as *mut c_void);
                pa_context_get_server_info(context, server_info_callback, myint_ptr as *mut c_void);
            },
            _ => {}
        }
    }

    extern "C" fn server_info_callback(c: *mut opaque::pa_context, info: *const structs::pa_server_info, userdata: *mut c_void) {

        let info = unsafe{ *info };
        println!("===================== server_info_callback =======================");
        println!("user_name: {}", cstr_to_string(info.user_name));
        println!("host_name: {}", cstr_to_string(info.host_name));
        println!("server_version: {}", cstr_to_string(info.server_version));
        println!("server_name: {}", cstr_to_string(info.server_name));
        println!("default_sink_name: {}", cstr_to_string(info.default_sink_name));
        println!("default_source_name: {}", cstr_to_string(info.default_source_name));
        println!("===================== end server_info_callback =======================");

    }


    extern "C" fn sink_list_callback(c: *mut opaque::pa_context,
        info: *const structs::pa_sink_info, eol: c_int, userdata: *mut c_void) {
        // XXX: Memory errors. I think that pa_sink_info struct doesn't match
        // XXX: up exactly right.
        println!("eol: {}", eol);


        if info.is_null() {
            println!("null sink lol");
        } else {
            let info = unsafe{ *info };
            println!("Callback for card: {}", cstr_to_string(info.name));
        }
        //println!("Callback for card: {}", cstr_to_string(info.name));
        //unsafe{ mem::forget(info) };
    }

    pub fn main_async() {
        let mainloop = pa_mainloop_new();
        let mainloop_api = pa_mainloop_get_api(mainloop);
        let context = pa_context_new(mainloop_api, "rust_viz");
        let mut myint: usize = 12345;
        let myint_ptr: *mut usize = (&mut myint) as *mut usize;

        pa_context_set_state_callback(context, state_callback, myint_ptr as *mut c_void);
        pa_context_connect(context, None, enums::pa_context_flags::NOAUTOSPAWN, None);

        let mut mainloop_res: c_int = 0;
        pa_mainloop_run(mainloop, &mut mainloop_res);
    }
}

fn main() {
    use pulse::*;

    let mut papi = pulse::PulseAudioApi::new("rs_client");
    papi.set_state_callback(|papi, state| {
        println!("hey gimme gimme callbacks {}", state as c_int);
        match state {
            pa_context_state::READY => {
                println!("calling!");
                papi.get_server_info(|p, i| {
                    println!("called!");
                });

            },
            _ => {}
        }

    });


    papi.connect(None, pulse::pa_context_flags::NOAUTOSPAWN);
    papi.run_mainloop();


    println!("sizeof pa_sink_info: {}", mem::size_of::<pulse_types::structs::pa_sink_info>());
    println!("sizeof pa_cvolume: {}", mem::size_of::<pulse_types::structs::pa_cvolume>());

    return async::main_async();

    let args = os::args();
    if args.len() != 2 {
        panic!("
            I need a device.
            Ex: <binary_name> <device_name>.monitor
        ");
    }

    run_analyzer(args[1].as_slice());
}

// USEFUL DOCS:
//
// Stereo vs mono:
// http://stackoverflow.com/questions/3287911/how-to-represent-stereo-audio-data-for-fft
// http://stackoverflow.com/questions/14477454/apply-fft-to-a-both-channels-of-a-stereo-signal-seperately
// http://stackoverflow.com/questions/4714542/pcm-wave-file-stereo-to-mono
