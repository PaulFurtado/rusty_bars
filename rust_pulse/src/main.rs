#![allow(unstable)]

// TODO: Automatically get default sink

extern crate libc;

use self::libc::{c_int, c_char, size_t};
use std::ptr;
use std::os;
use std::mem::transmute;
use std::ffi::CString;
use std::str::from_utf8;
use libc::funcs::c95::string::strlen;
pub mod analyze_spectrum;



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
    let sample_spec = PulseSampleSpec{
        format: PA_SAMPLE_S16LE,
        rate: 44100,
        channels: 2
    };


    let mut pulse = PulseSimple::new(dev, StreamDirection::StreamRecord, &sample_spec).unwrap();
    let mut fft = analyze_spectrum::AudioFFT::new(1024, 2, 44100, 32);

    let mut buffer_vec: Vec<u8> = Vec::with_capacity(fft.get_buf_size());
    for _ in range(0, fft.get_buf_size()) {
        buffer_vec.push(0);
    }
    let mut buffer = buffer_vec.as_mut_slice();

    loop {
        pulse.read(buffer).unwrap();
        let output = fft.execute(buffer);

        let temp: Vec<String> = output.iter().map(|x| format!("{}", x)).collect();
        println!("output: {}", temp.connect(", "));
    }
}





fn main() {



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
