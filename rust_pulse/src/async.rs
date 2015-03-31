extern crate libc;

use self::libc::{c_int, c_char, size_t, c_void};
use std::ptr;
use std::os;
use std::mem::transmute;
use std::ffi::CString;


/*
    See this example for async API usage with clickable functions:
    http://maemo.org/api_refs/5.0/5.0-final/pulseaudio/paplay_8c-example.html
    
    Docs here too:
    http://maemo.org/api_refs/5.0/5.0-final/pulseaudio/context_8h.html#2784c754947a97f02c78b73d7b1c2d5f
*/



#[allow(non_camel_case_types)]
pub struct pa_mainloop;
pub struct pa_mainloop_api;
pub struct pa_context;
pub struct pa_spawn_api;
pub enum pa_context_state {
    UNCONNECTED,  // 	The context hasn't been connected yet.
    CONNECTING,   //	A connection is being established.
    AUTHORIZING,  //	The client is authorizing itself to the daemon.
    SETTING_NAME, //	The client is passing its application name to the daemon.
    READY,        //	The connection is established, the context is ready to execute operations.
    FAILED,       // The connection failed or was disconnected.
    TERMINATED,   // The connection was terminated cleanly.
}




#[link(name="pulse")]
extern {
    fn pa_mainloop_new() -> *mut pa_mainloop;
    fn pa_mainloop_get_api(m: *mut pa_mainloop) -> *mut pa_mainloop_api;
    fn pa_context_new(mainloop: *mut pa_mainloop, client_name: *const c_char) -> *mut pa_context;
    // TODO: define a real type for the callback
    fn pa_context_set_state_callback(context: *mut pa_context, cb: *const c_void, userdata: *mut c_void);
    fn pa_context_connect(context: *mut pa_context, server: *const c_char, flags: c_int, api: *const pa_spawn_api) -> c_int;
    fn pa_context_get_state(context: *mut pa_context) -> pa_context_state;
    fn pa_mainloop_run(m: *mut pa_mainloop, result: *mut c_int) -> c_int;
}




fn rs_pa_context_new(mainloop: *mut pa_mainloop, client_name: &str) -> *mut pa_context {
    let path = CString::from_slice(client_name.as_bytes());

    unsafe { pa_context_new(mainloop, path.as_ptr()) }
}

fn rs_pa_context_connect(context: *mut pa_context, server: &str, flags: c_int, api: *const pa_spawn_api) -> c_int {
     let server = CString::from_slice(server.as_bytes());
     let result = unsafe{ pa_context_connect(context, server.as_ptr(), flags, api) };
     if result < 0 {
        panic!("ahhhhh");
     }
     result
}



pub fn context_state_callback(context: *mut pa_context, userdata: *mut c_void) {
    println!("yay called back");
    let state = unsafe{ pa_context_get_state(context) };
    println!("state={}", state as c_int);
    
}


pub fn main() {
    let mainloop: *mut pa_mainloop = unsafe{ pa_mainloop_new() };
    let mainloop_api: *mut pa_mainloop_api = unsafe{ pa_mainloop_get_api(mainloop) };
    let context = rs_pa_context_new(mainloop, "rs_test_async");
    unsafe { pa_context_set_state_callback(context, transmute(context_state_callback), ptr::null_mut()) };
    rs_pa_context_connect(context, "myname", 0, ptr::null());
    let mut res: c_int = 0;
    if unsafe{ pa_mainloop_run(mainloop, &mut res) } < 0 {
        println!("baddd");
    }
    
    println!("woohooo asynccc");
}







