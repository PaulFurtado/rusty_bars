#![allow(unstable)]

// TODO: Automatically get default sink

extern crate libc;

use self::libc::{c_int, c_char, size_t};
use std::ptr;
use std::os;
use std::mem::transmute;
use std::ffi::CString;
use std::str::from_utf8;
use std::io::stdio;
use libc::funcs::c95::string::strlen;
use std::num::Float;
pub mod analyze_spectrum;


// The audio sample rate in hz
const SAMPLE_RATE: usize = 44100;
// 2 for stereo audio
const SAMPLE_CHANNELS: usize = 2;
// How many frames per second
const FOURIERS_PER_SECOND: usize = 24;
// How many bytes per sample for a single channel. 2 bytes for s16le
const BYTES_PER_SAMPLE: usize = 2;
// How many bytes to read for a single channel worth of data
const ONE_CHANNEL_CHUNK_SIZE: usize = (SAMPLE_RATE * BYTES_PER_SAMPLE) / FOURIERS_PER_SECOND;
// How many bytes to read to get all channels
const CHUNK_SIZE: usize = ONE_CHANNEL_CHUNK_SIZE * SAMPLE_CHANNELS;




const FOURIER_SPREAD: f64 = 1.0f64/(FOURIERS_PER_SECOND as f64);
const FOURIER_WIDTH: f64 = FOURIER_SPREAD;
const FOURIER_WIDTH_INDEX: f64 = FOURIER_WIDTH * (SAMPLE_RATE as f64);
const FOURIER_SPACING: i64 = ((FOURIER_SPREAD * (SAMPLE_RATE as f64)) + 0.5) as i64;
const SAMPLE_SIZE: f64 = FOURIER_WIDTH_INDEX;
//FREQ = SAMPLE_RATE / SAMPLE_SIZE * np.arange(SAMPLE_SIZE)
//const CHUNK_SIZE: usize = ((SAMPLE_SIZE - 1f64) as usize) * 2;




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

    fn fftw_plan_dft_r2c_1d(n: c_int, input: *mut f64, output: *mut FftwComplex, flags: c_int) -> *const FftwPlan;
    fn fftw_execute(plan: *const FftwPlan);

}

const FFTW_ESTIMATE: c_int = (1 << 6);


#[repr(C)]
#[derive(Copy)]
struct FftwComplex {
    re: f64,
    im: f64
}


impl FftwComplex {
    pub fn abs(&self) -> f64 {
        ((self.re * self.re) + (self.im * self.im)).sqrt()
    }

}

/*
struct MonoSpectrumAnalyzer {
    sample_len: usize,

}


impl MonoSpectrumAnalyzer {
    pub fn new(sample_size: usize) -> MonoSpectrumAnalyzer {
        MonoSpectrumAnalyzer{

    }

}

*/



#[repr(C)]
#[derive(Copy)]
enum FFTWDirection {
    Forward=-1,
    Backward=1
}


#[derive(Copy)]
pub enum FftwPlan {}


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






fn main() {

    println!("chunk size: {}", CHUNK_SIZE);

    let mut buffer_vec: Vec<u8> = Vec::with_capacity(CHUNK_SIZE);

    let mut input_vec: Vec<f64> = Vec::with_capacity(CHUNK_SIZE/2 as usize);
    let mut output_vec: Vec<FftwComplex> = Vec::with_capacity(CHUNK_SIZE/2 as usize);



    for i in range(0, CHUNK_SIZE) {
        buffer_vec.push(0);
        if i < SAMPLE_SIZE as usize + 1 {
            input_vec.push(0f64);
            output_vec.push(FftwComplex{im:0f64,re:0f64});
        }
    }


    let input: &mut [f64] = input_vec.as_mut_slice();
    let output: &mut [FftwComplex] = output_vec.as_mut_slice();


    let plan = unsafe { fftw_plan_dft_r2c_1d(16 as i32, input.as_mut_ptr(), output.as_mut_ptr(), FFTW_ESTIMATE)};




    let args = os::args();
    if args.len() != 3 {
        panic!("
            I need a mode an and a device.
            Ex: <binary_name> <play|record> <device_name>

        ");
    }
    let mode_str = args[1].clone();

    let sample_spec = PulseSampleSpec{
        format: PA_SAMPLE_S16LE,
        rate: 44100,
        channels: 2
    };


    let mode: StreamDirection = if mode_str == "play" {
        StreamDirection::StreamPlayback
    } else if mode_str == "record" {
        StreamDirection::StreamRecord
    } else {
        panic!("Invalid mode!");
    };

    let dev = args[2].clone();


    let mut pulse = PulseSimple::new(dev.as_slice(), mode, &sample_spec).unwrap();


    let buffer: &mut [u8] = buffer_vec.as_mut_slice();

    let mut stdout = stdio::stdout();
    let mut stdin = stdio::stdin();

    match mode {
        StreamDirection::StreamPlayback => {
            loop {
                match stdin.read(buffer) {
                    Ok(count) => {
                        pulse.write(buffer, count as u64).unwrap();
                    },
                    Err(err) => {
                        println!("read error: {}", err);
                        break;
                    }
                }
            }
        },
        StreamDirection::StreamRecord => {
            let mut fft = analyze_spectrum::AudioFFT::new(1024, 2, 44100, 32);

            let mut buffer_vec: Vec<u8> = Vec::with_capacity(fft.get_buf_size());
            for _ in range(0, fft.get_buf_size()) {
                buffer_vec.push(0);
            }
            let mut buffer = buffer_vec.as_mut_slice();

            loop {
                println!("Reading {} bytes", buffer.len());
                pulse.read(buffer).unwrap();
                let output = fft.execute(buffer);

                let temp: Vec<String> = output.iter().map(|x| format!("{}", x)).collect();
                println!("output: {}", temp.connect(", "));
                /*

                let v: Vec<u16> = unsafe{ Vec::from_raw_buf(buffer.as_ptr() as *const u16, buffer.len()) };
                let temp: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
               // println!("input: [{}]\n\n\n", temp.connect(", "));


                for (i, v) in v.iter().enumerate() {
                    if i < input.len() {
                        input[i] = *v as f64;
                    }
                }
                unsafe { fftw_execute(plan) };

                //println!("len: {}", v.len());

                //let temp: Vec<String> = input.iter().map(|x| format!("{}", x)).collect();
                //println!("input: {}", temp.connect(", "));


                //let temp: Vec<String> = output.iter().map(|x| format!("({}, {}i)", x.re, x.im)).collect();
                //println!("result_complex: [{}]\n\n\n", temp.connect(", "));

                let temp: Vec<String> = output.slice_to(16).iter().map(|x| format!("{}", x.abs())).collect();
                println!("result_abs: [{}]", temp.connect(", "));
                //return;
                //stdout.write(buffer).unwrap();*/
            }
        }
        _ => {
            panic!("not implemented");
        }
    }

   match mode {
     StreamDirection::StreamPlayback => { pulse.drain().unwrap(); },
     _ => {}
    }

}

// USEFUL DOCS:
//
// Stereo vs mono:
// http://stackoverflow.com/questions/3287911/how-to-represent-stereo-audio-data-for-fft
// http://stackoverflow.com/questions/14477454/apply-fft-to-a-both-channels-of-a-stereo-signal-seperately
// http://stackoverflow.com/questions/4714542/pcm-wave-file-stereo-to-mono
