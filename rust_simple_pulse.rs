#![allow(unstable)]
#![feature(link_args)]

extern crate libc;

use self::libc::{c_int, c_char, size_t};
use std::ptr;
use std::io::File;
use std::path::Path;
use std::os;
use  std::mem::transmute;
use std::ffi::CString;
use  std::str::from_utf8;
use std::io::fs::PathExtensions;
use std::io::stdio;

#[link_args = "-lpulse-simple -lpulse"]
extern {
  fn pa_simple_new(server: *const c_char,
                   name: *const c_char,
                   dir: c_int,
                   dev: *const c_char,
                   steam_name: *const c_char,
                   sample_spec: *const pa_sample_spec,
                   channel_map: *const u8,
                   attr: *const u8,
                   error: *mut c_int) -> *mut pa_simple;

  fn pa_simple_free(pa: *mut pa_simple);

  fn pa_simple_write(pa: *mut pa_simple,
                     data: *const u8,
                     bytes: size_t,
                     error: *mut c_int) -> c_int;

    fn pa_simple_read(pa: *mut pa_simple, data: *mut u8, bytes: size_t, error: *mut c_int) -> c_int;


  fn pa_simple_drain(pa: *mut pa_simple,
                     error: *mut c_int) -> c_int;

  fn pa_strerror(error: c_int) -> *const c_char;
}

// typedef struct pa_simple pa_simple
pub struct pa_simple;

// defined as enum pa_stream_direction
//pub static PA_STREAM_NODIRECTION: c_int = 0_i32;
pub static PA_STREAM_PLAYBACK:    c_int = 1_i32;
pub static PA_STREAM_RECORD:      c_int = 2_i32;
//pub static PA_STREAM_UPLOAD:      c_int = 3_i32;

// see pa_sample_format
pub static PA_SAMPLE_S16LE: c_int = 3_i32;

// see pulse/def.h
pub struct pa_sample_spec {
  format: c_int,
  rate: u32,
  channels: u8
}



// --------------------
pub fn pa_new(pa_name: &str, stream_name: &str) -> Box<*mut pa_simple> {
  unsafe {
    let mut err: c_int = 0;

    let s_spec = pa_sample_spec{
                      format: PA_SAMPLE_S16LE,
                      rate: 44100,
                      channels: 2};

    let pa_name_c = CString::from_slice(pa_name.as_bytes());
    let stream_name_c = CString::from_slice(stream_name.as_bytes());
    let dev_c = CString::from_slice("alsa_output.usb-NuForce__Inc._NuForce___DAC_2-01-N2.analog-stereo.monitor".as_bytes());

    let pa = pa_simple_new(
                  ptr::null(),
                  pa_name_c.as_ptr(),
                  PA_STREAM_RECORD,
                  dev_c.as_ptr(),
                  stream_name_c.as_ptr(),
                  transmute(&s_spec),
                  ptr::null(),
                  ptr::null(),
                  &mut err);
    if ( err != 0 ) {
      //panic!("err code {} from pulse: '{}'", err, from_utf8(transmute(pa_strerror(err))).unwrap() );
      panic!("err code {} from pulse", err );
    }
    Box::new(pa) // cast to region pointer, owning pointer
  }
}

//---------------------------------------------

pub fn record(pa: Box<*mut pa_simple>) -> bool {
    static BUFSIZE: usize = 1024us;
    let mut buffer = [0u8; 1024];
    let mut err: c_int = 0;
    let mut stdout = stdio::stdout();

    loop {
         let r_res = unsafe {pa_simple_read(*pa, buffer.as_mut_ptr(), buffer.len() as size_t, &mut err) };

         if ( r_res < 0) {
            //println!("ERROR code {} from pulse: \"{}\"",
            //         err,  from_utf8(pa_strerror(err)).unwrap());
            println!("errr code {} from pulse", err);
            return false;
        }

        stdout.write(&buffer).unwrap();

    }

}



pub fn play_file(pa: Box<*mut pa_simple>, path: &Path) -> bool {
  if ( !path.is_file() ) {
    println!("This is not a file!");
    return false;
  }

  println!("Gonna play: {}", path.as_str().unwrap());

  let mut err: c_int = 0;
  let mut file_reader = File::open(path);
  unsafe {
    static BUFSIZE: usize = 1024us;
    let mut buffer = [0u8; 1024];
    let mut total_read = 0us;
    loop {
      let b_read = match file_reader.read(&mut buffer) {
        Err(x) => break, // eof
        Ok(s) => s//read smth
      };
      let w_res = pa_simple_write(
                    *pa,
                    buffer.as_ptr(),
                    b_read as size_t,
                    &mut err);
      if ( w_res < 0) {
        //println!("ERROR code {} from pulse: \"{}\"",
        //         err,  from_utf8(pa_strerror(err)).unwrap());
        println!("errr code {} from pulse", err);
        return false;
      }
      total_read += b_read;
    }
    println!("bytes read: {}", total_read);

    pa_simple_drain(*pa, &mut err);
  }
  true
}

//---------------------------------------------

pub fn free_pa(pa: Box<*mut pa_simple>) {
  unsafe {
    pa_simple_free(*pa);
  }
}

//---------------------------------------------

fn main()
{
  let args = os::args();
  if ( args.len() != 2 ) {
    panic!("BAAHH I need a file to play as a parameter.");
  }
  let f_name = args[1].clone();

  let path = Path::new(f_name);
  let pa_name = "rust_simple_pulse";
  let stream_name  = "rust_playback";

  let pa = pa_new(pa_name, stream_name);


  if ( !record(pa) )
  {
    panic!("Dude I was not able to record the file.");
  }
  //free_pa(pa);
}
