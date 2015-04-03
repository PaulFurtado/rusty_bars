#![feature(link_args)]
#![allow(unstable)]
#![allow(non_camel_case_types)]
#![allow(missing_copy_implementations)]
#![feature(link_args)]


extern crate libc;

use self::libc::{c_int, c_char, size_t, c_void};
use std::ptr;
use std::mem::{transmute, forget};
use std::ffi::CString;
use libc::funcs::c95::string::strlen;

use std::io::timer;
use std::time::duration::Duration;
use std::ffi;
use std::str;

/*
    See this example for async API usage with clickable functions:
    http://maemo.org/api_refs/5.0/5.0-final/pulseaudio/paplay_8c-example.html
    These are good too:
    http://maemo.org/api_refs/5.0/5.0-final/pulseaudio/pacat_8c-example.html


    The mose useful version of the API docs is here:
    http://freedesktop.org/software/pulseaudio/doxygen/index.html#async_sec


*/

#[repr(C)]
pub struct pa_mainloop;
#[repr(C)]
pub struct pa_mainloop_api;
#[repr(C)]
pub struct pa_context;
#[repr(C)]
pub struct pa_spawn_api;
#[repr(C)]
pub struct pa_threaded_mainloop;
#[repr(C)]
pub struct pa_proplist;
#[repr(C)]
pub struct pa_stream;

#[repr(C)]
#[derive(Copy,Clone)]
pub struct wrapper {
    mainloop: *mut pa_threaded_mainloop,
    context: *mut pa_context,
    stream: *mut pa_stream
}


#[repr(C)]
pub enum pa_context_state {
    UNCONNECTED,  // 	The context hasn't been connected yet.
    CONNECTING,   //	A connection is being established.
    AUTHORIZING,  //	The client is authorizing itself to the daemon.
    SETTING_NAME, //	The client is passing its application name to the daemon.
    READY,        //	The connection is established, the context is ready to execute operations.
    FAILED,       // The connection failed or was disconnected.
    TERMINATED,   // The connection was terminated cleanly.
}

#[repr(C)]
pub enum pa_context_flags {
    NOAUTOSPAWN, // Disabled autospawning of the PulseAudio daemon if required.
    NOFAIL       // Don't fail if the daemon is not available when pa_context_connect()
                 // is called, instead enter PA_CONTEXT_CONNECTING state and wait for
                 // the daemon to appear.
}

//#[repr(C)]
//#[derive(Copy)]
//pub struct PaSimpleC;



#[repr(C)]
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

#[repr(C)]
#[derive(Copy)]
pub struct PaSimpleC {
    mainloop: *mut pa_threaded_mainloop,
    context: *mut pa_context,
    stream: *mut pa_stream,
    direction: StreamDirection,
    read_data: *mut c_void,
    read_index: size_t,
    read_length: size_t,
    operation_success: c_int
}

#[repr(C)]
struct pa_operation;






pub type pa_context_notify_cb_t = extern "C" fn(*mut pa_context, *mut c_void);
pub type pa_sink_info_cb_t = extern "C" fn(c: *mut pa_context, i: *const pa_sink_info, eol: c_int, userdata: *mut c_void);



#[link(name="pulse")]
#[link(name="pulse-simple")]
#[link_args = "-L/home/paul/School/2015/extensible/rust_pulse/src -lpulsehelper"]
extern {
    fn get_pulse() -> *mut wrapper;

    fn pa_mainloop_new() -> *mut pa_mainloop;

    fn pa_mainloop_get_api(m: *mut pa_mainloop) -> *mut pa_mainloop_api;

    fn pa_threaded_mainloop_get_api(m: *mut pa_threaded_mainloop) -> *mut pa_mainloop_api;


    fn pa_context_new(mainloop_api: *mut pa_mainloop_api, client_name:
        *const c_char) -> *mut pa_context;

    fn pa_context_set_state_callback(context: *mut pa_context,
        cb: pa_context_notify_cb_t, userdata: *mut c_void);

    fn pa_context_connect(context: *mut pa_context, server: *const c_char,
        flags: pa_context_flags, api: *const pa_spawn_api) -> c_int;

    fn pa_context_get_state(context: *mut pa_context) -> pa_context_state;

    fn pa_mainloop_run(m: *mut pa_mainloop, result: *mut c_int) -> c_int;

    fn pa_signal_init(api: *mut pa_mainloop_api) -> c_int;

    fn pa_socket_client_new_sockaddr(m: *mut pa_mainloop_api, sa: *const c_void,
        salen: size_t) -> *mut c_void;

    fn pa_proplist_new() -> *mut pa_proplist;

    fn pa_context_new_with_proplist(mainloop: *mut pa_mainloop, client_name: *const c_char, proplist: *mut pa_proplist) -> *mut pa_context;

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

    fn pa_strerror(error: c_int) -> *const c_char;


    fn pa_context_get_sink_info_list(c: *mut pa_context, cb: pa_sink_info_cb_t, userdata: *mut c_void) -> *mut pa_operation;
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
}



/// Thin wrapper around pa_context_new to pass the client name string as a CString
fn rs_pa_context_new(mainloop: *mut pa_mainloop, client_name: &str) -> *mut pa_context {
    let name = CString::from_slice(client_name.as_bytes());
    //let res = unsafe { pa_context_new(mainloop, name.as_ptr()) };
    let proplist = unsafe { pa_proplist_new() };


    let mainloop_api = unsafe { pa_mainloop_get_api(mainloop) };
    let res = unsafe { pa_context_new(mainloop_api, name.as_ptr()) };

    //let res = unsafe { pa_context_new_with_proplist(mainloop, name.as_ptr(), proplist) };

    assert!(!res.is_null());
    unsafe { forget(name) };
    res
}

/// Thin wrapper around pa_context_connect.
fn rs_pa_context_connect(context: *mut pa_context, flags: pa_context_flags) -> c_int {
     unsafe{ pa_context_connect(
        context,
        ptr::null(), // The server argument is null to connect to the default
        flags,
        ptr::null() // The spawn API argument is optional.
    )}
}


pub type pa_volume_t = u32;

pub type pa_usec_t = u64;




#[repr(C)]
#[derive(Copy)]
pub struct pa_cvolume {
    channels: u8,
    values: pa_volume_t
}

#[repr(C)]
#[derive(Copy)]
pub struct pa_channel_map {
    channels: u8,
    values: pa_channel_position_t
}


#[repr(C)]
#[derive(Copy)]
pub enum pa_channel_position_t {
    INVALID,
    MONO,
    FRONT_LEFT,
    FRONT_RIGHT,
    FRONT_CENTER,
    REAR_CENTER,
    REAR_LEFT,
    REAR_RIGHT,
    LFE,
    FRONT_LEFT_OF_CENTER,
    FRONT_RIGHT_OF_CENTER,
    SIDE_LEFT,
    SIDE_RIGHT,
    AUX0,
    AUX1,
    AUX2,
    AUX3,
    AUX4,
    AUX5,
    AUX6,
    AUX7,
    AUX8,
    AUX9,
    AUX10,
    AUX11,
    AUX12,
    AUX13,
    AUX14,
    AUX15,
    AUX16,
    AUX17,
    AUX18,
    AUX19,
    AUX20,
    AUX21,
    AUX22,
    AUX23,
    AUX24,
    AUX25,
    AUX26,
    AUX27,
    AUX28,
    AUX29,
    AUX30,
    AUX31,
    TOP_CENTER,
    TOP_FRONT_LEFT,
    TOP_FRONT_RIGHT,
    TOP_FRONT_CENTER,
    TOP_REAR_LEFT,
    TOP_REAR_RIGHT,
    TOP_REAR_CENTER,
    MAX,
}




#[repr(C)]
#[derive(Copy)]
pub struct pa_sink_info {
    name: *const c_char,               //**< Name of the sink */
    index: u32,                        //**< Index of the sink */
    description: *const c_char,  //**< Description of this sink */
    sample_spec: PulseSampleSpec,        //**< Sample spec of this sink */
    channel_map: pa_channel_map,        //**< Channel map */
    owner_module: u32,             //**< Index of the owning module of this sink, or PA_INVALID_INDEX. */
    volume: pa_cvolume,                 //**< Volume of the sink */
    mute: c_int,                          //**< Mute switch of the sink */
    monitor_source: u32,          //**< Index of the monitor source connected to this sink. */
    monitor_source_name: *const c_char,   //**< The name of the monitor source. */
    latency: pa_usec_t,                 //**< Length of queued audio in the output buffer. */
    driver: *const c_char,                //**< Driver name */
    flags: pa_sink_flags_t,             //**< Flags */
    proplist: *mut pa_proplist,             //**< Property list \since 0.9.11 */
    configured_latency: pa_usec_t,      //**< The latency this device has been configured to. \since 0.9.11 */
    base_volume: pa_volume_t,           //**< Some kind of "base" volume that refers to unamplified/unattenuated volume in the context of the output device. \since 0.9.15 */
    state: pa_sink_state_t,             //**< State \since 0.9.15 */
    n_volume_steps: u32,           //**< Number of volume steps for sinks which do not support arbitrary volumes. \since 0.9.15 */
    card: u32,                     //**< Card index, or PA_INVALID_INDEX. \since 0.9.15 */
    n_ports: u32,                  //**< Number of entries in port array \since 0.9.16 */
    ports: *mut c_void,
    //pa_sink_port_info** ports;         //**< Array of available ports, or NULL. Array is terminated by an entry set to NULL. The number of entries is stored in n_ports. \since 0.9.16 */
    active_port: *mut c_void,
    //pa_sink_port_info* active_port;    //**< Pointer to active port in the array, or NULL. \since 0.9.16 */
    n_formats: u8,
    formats: *mut c_void          //**< Number of formats supported by the sink. \since 1.0 */
    //pa_format_info **formats;          //**< Array of formats supported by the sink. \since 1.0 */
}


#[repr(C)]
#[derive(Copy)]
pub enum pa_sink_state_t {
    PA_SINK_INVALID_STATE = -1,
    PA_SINK_RUNNING = 0,
    PA_SINK_IDLE = 1,
    PA_SINK_SUSPENDED = 2,
    PA_SINK_INIT = -2,
    PA_SINK_UNLINKED = -3
}

#[repr(C)]
#[derive(Copy)]
pub enum pa_sink_flags_t {
    PA_SINK_NOFLAGS = 0x0000is,
    PA_SINK_HW_VOLUME_CTRL = 0x0001is,
    PA_SINK_LATENCY = 0x0002is,
    PA_SINK_HARDWARE = 0x0004is,
    PA_SINK_NETWORK = 0x0008is,
    PA_SINK_HW_MUTE_CTRL = 0x0010is,
    PA_SINK_DECIBEL_VOLUME = 0x0020is,
    PA_SINK_FLAT_VOLUME = 0x0040is,
    PA_SINK_DYNAMIC_LATENCY = 0x0080is,
    PA_SINK_SET_FORMATS = 0x0100is
}





pub extern "C" fn context_state_callback(context: *mut pa_context, userdata: *mut c_void) {
    let state = unsafe{ pa_context_get_state(context) };
    let userdata: *mut c_int = userdata as *mut c_int;
    let userdata_val: c_int = unsafe{ *userdata };
    println!("Callback was called. state={} userdata={}", state as c_int, userdata_val);
}

extern "C" fn sink_list_cb(c: *mut pa_context, i: *const pa_sink_info, eol: c_int, userdata: *mut c_void) {
    if i.is_null() {
        println!("wtf found a null one");
        return;
    }

    println!("omfg");
    let info: pa_sink_info = unsafe { *i };
    let c_name = info.name;
    let slice_name = unsafe { ffi::c_str_to_bytes(&c_name) };
    println!("Name: {}", str::from_utf8(slice_name).unwrap());

    println!("made it");
    unsafe { forget(info) };
}


pub fn main2() {
    let wrapped = unsafe { *get_pulse() };
    let mainloop = wrapped.mainloop;
    let context = wrapped.context;
    let userdata: c_int = 12345;
    let mainloop_api = unsafe { pa_threaded_mainloop_get_api(mainloop) };
    //let context = rs_pa_context_new(mainloop_api, "rs_test_async");


    let name = CString::from_slice("Abc".as_bytes());
    //let res = unsafe { pa_context_new(mainloop, name.as_ptr()) };

    let context = unsafe { pa_context_new(mainloop_api, name.as_ptr()) };

    unsafe { pa_context_set_state_callback(context, context_state_callback, transmute(&userdata)) };
    rs_pa_context_connect(context, pa_context_flags::NOAUTOSPAWN);
}


pub fn main_simple_hack() {
    let ss = PulseSampleSpec{
        format: PA_SAMPLE_S16LE,
        rate: 44100,
        channels: 2
    };
    let ps = PulseSimple::new(
        "alsa_output.usb-NuForce__Inc._NuForce___DAC_2-01-N2.analog-stereo.monitor",
        StreamDirection::StreamRecord,
        &ss
    ).unwrap();

    let pa_c = ps.pa;
    //let pa_c = unsafe { pa_c as *mut pa_simple };
    let pa = unsafe { *pa_c };
    let mainloop = pa.mainloop as *mut pa_mainloop;
    let context = pa.context as *mut pa_context;
    unsafe {
        pa_context_get_sink_info_list(context, sink_list_cb, ptr::null_mut());
    }
    let mut timer = timer::Timer::new().unwrap();
    timer.sleep(Duration::milliseconds(10000));

    println!("hi");

}




// No longer used. This is how we could instantiate the async API ourselves.
pub fn main() {


    let mainloop: *mut pa_mainloop = unsafe{ pa_mainloop_new() };
    let mainloop_api: *mut pa_mainloop_api = unsafe{ pa_mainloop_get_api(mainloop) };

    let r = unsafe { pa_signal_init(mainloop_api) };
    assert!(r==0);

    let context = rs_pa_context_new(mainloop, "rs_test_async");


    let userdata: c_int = 12345;
    unsafe { pa_context_set_state_callback(context, context_state_callback, transmute(&userdata)) };
    println!("Checkpoint.");
    rs_pa_context_connect(context, pa_context_flags::NOAUTOSPAWN);
    println!("Doesn't get here.");

    let mut res: c_int = 0;
    if unsafe{ pa_mainloop_run(mainloop, &mut res) } < 0 {
        println!("error running mainloop");
    }

    println!("That finally worked!");
}
