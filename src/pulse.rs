#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
use libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void};
use std::ffi::CString;
pub use pulse_types::*;
use std::ptr;
use std::mem;

/// Coverts a pulse error code to a String
fn pa_err_to_string(err: c_int) -> Result<(), String> {
    if err == 0 {
        Ok(())
    } else {
        unsafe {
            let err_msg_ptr: *const c_char = ext::pa_strerror(err);
            let size = strlen(err_msg_ptr) as usize;
            let slice: Vec<u8> = Vec::from_raw_buf((err_msg_ptr as *const u8), size);
            Err(String::from_utf8(slice).unwrap())
        }
    }
}

/// A safe interface to pa_mainloop_get_api
pub fn pa_mainloop_get_api(mainloop: *mut opaque::pa_mainloop) -> *mut opaque::pa_mainloop_api {
    assert!(!mainloop.is_null());
    let mainloop_api = unsafe { ext::pa_mainloop_get_api(mainloop) };
    assert!(!mainloop_api.is_null());
    return mainloop_api;
}

/// A safe interface to pa_context_new
pub fn pa_context_new(mainloop_api: *mut opaque::pa_mainloop_api, client_name: &str) -> *mut opaque::pa_context {
    assert!(!mainloop_api.is_null());
    let client_name_c = CString::from_slice(client_name.as_bytes());
    let context = unsafe{ ext::pa_context_new(mainloop_api, client_name_c.as_ptr()) };
    assert!(!context.is_null());
    return context;
}

/// A safe interface to pa_mainloop_new
pub fn pa_mainloop_new() -> *mut opaque::pa_mainloop {
    let mainloop = unsafe{ ext::pa_mainloop_new() };
    assert!(!mainloop.is_null());
    return mainloop;
}


/// A safe interface to pa_context_set_state_callback
pub fn pa_context_set_state_callback(context: *mut opaque::pa_context,
    cb: cb::pa_context_notify_cb_t, userdata: *mut c_void) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_set_state_callback(context, cb, userdata) };
}


/// A safe wrapper for pa_context_connect
pub fn pa_context_connect(context: *mut opaque::pa_context, server_name: Option<&str>,
    flags: enums::pa_context_flags, spawn_api: Option<*const opaque::pa_spawn_api>) {

    assert!(!context.is_null());

    let server: *const c_char = match server_name {
        None => ptr::null(),
        Some(name) => CString::from_slice(name.as_bytes()).as_ptr()
    };

    let spawn_api_ptr: *const opaque::pa_spawn_api = match spawn_api {
        Some(api_ptr) => {
            assert!(!api_ptr.is_null());
            api_ptr
        },
        None => ptr::null()
    };

    let res = unsafe { ext::pa_context_connect(context, server, flags, spawn_api_ptr) };
    assert!(res == 0);
}

/// A safe wrapper around pa_mainloop_run
pub fn pa_mainloop_run(mainloop: *mut opaque::pa_mainloop, result: &mut c_int) {
    assert!(!mainloop.is_null());
    let res = unsafe{ ext::pa_mainloop_run(mainloop, result as *mut c_int) };
    assert!(res == 0);
}

/// A safe wrapper around pa_context_get_state
pub fn pa_context_get_state(context: *mut opaque::pa_context) -> enums::pa_context_state {
    assert!(!context.is_null());
    unsafe{ ext::pa_context_get_state(context) }
}


/// A safe wrapper around pa_context_get_sink_info_list
pub fn pa_context_get_sink_info_list(context: *mut opaque::pa_context,
    callback: cb::pa_sink_info_cb_t, userdata: *mut c_void) -> Option<*mut opaque::pa_operation> {

    assert!(!context.is_null());
    let result = unsafe{ ext::pa_context_get_sink_info_list(context, callback, userdata) };
    if result.is_null() {
        None
    } else {
        Some(result)
    }
}



pub struct PulseAudioApi {
    context: *mut pa_context,
    mainloop: *mut pa_mainloop,
    mainloop_api: *mut pa_mainloop_api,
    state_callback: Option<Box<FnMut(pa_context_state) + 'static>>
}


extern fn _state_callback(context: *mut opaque::pa_context, papi: *mut c_void) {
    let papi: &mut PulseAudioApi =  unsafe{ mem::transmute(papi) };
    papi.state_callback();
}



impl PulseAudioApi {
    pub fn new(client_name: &str) -> PulseAudioApi {
        let mainloop = pa_mainloop_new();
        let mainloop_api = pa_mainloop_get_api(mainloop);
        let context = pa_context_new(mainloop_api, client_name);
        PulseAudioApi {
            mainloop: mainloop,
            mainloop_api: mainloop_api,
            context: context,
            state_callback: None
        }
    }

    pub fn connect(&mut self, server: Option<&str>, flags: pa_context_flags) {
        pa_context_connect(self.context, server, flags, None);
    }

    fn state_callback(&mut self) {
        println!("heyyoooooo");
    }


    pub fn set_state_callback<C>(&mut self, cb: C) where C: FnMut(pa_context_state) + 'static {
        self.state_callback = Some(Box::new(cb));
        let papi: *mut PulseAudioApi = self;
        pa_context_set_state_callback(self.context, _state_callback, papi as *mut c_void);
    }

    /// Runs the mainloop on the current thread.
    pub fn run_mainloop(&mut self) -> Result<(), String> {
        let mut mainloop_res: c_int = 0;
        pa_mainloop_run(self.mainloop, &mut mainloop_res);
        // TODO: handle errors
        Ok(())
    }
}





/// Utility to convert C strings to String objects
pub fn cstr_to_string(c_str: *const c_char) -> String {
    let len: usize = unsafe{ strlen(c_str) } as usize;
    let s = unsafe{ String::from_raw_parts(c_str as *mut u8, len, len) };
    let retval = s.clone();
    unsafe{ mem::forget(s) };
    retval
}


pub fn pa_context_get_server_info(context: *mut opaque::pa_context,
    cb: cb::pa_server_info_cb_t, userdata: *mut c_void) -> Option<*mut opaque::pa_operation> {

    assert!(!context.is_null());
    let result = unsafe{ ext::pa_context_get_server_info(context, cb, userdata) };
    if result.is_null() {
        None
    } else {
        Some(result)
    }
}

/// Hack to get the maximum channels since it is a #DEFINE not a global :(
pub fn get_max_channels() -> Option<u8> {
    let mut last_valid: Option<u8> = None;
    for i in (1..255) {
        if unsafe { ext::pa_channels_valid(i as u8) } == 0 {
            return last_valid;
        } else {
            last_valid = Some(i);
        }
    }
    None
}

mod ext {
    extern crate libc;
    use self::libc::{c_void, c_char, c_int};
    use pulse_types::*;

    #[link(name="pulse")]
    extern {
        pub fn pa_mainloop_new() -> *mut opaque::pa_mainloop;

        pub fn pa_channels_valid(channels: u8) -> c_int;

        pub fn pa_mainloop_get_api(
            m: *mut opaque::pa_mainloop
        ) -> *mut opaque::pa_mainloop_api;

        pub fn pa_context_new(
            mainloop_api: *mut opaque::pa_mainloop_api,
            client_name: *const c_char
        ) -> *mut opaque::pa_context;

        pub fn pa_context_set_state_callback(
            context: *mut opaque::pa_context,
            cb: cb::pa_context_notify_cb_t,
            userdata: *mut c_void
        );

        pub fn pa_context_connect(
            context: *mut opaque::pa_context,
            server: *const c_char,
            flags: enums::pa_context_flags,
            api: *const opaque::pa_spawn_api
        ) -> c_int;

        pub fn pa_context_get_state(
            context: *mut opaque::pa_context
        ) -> enums::pa_context_state;

        pub fn pa_mainloop_run(
            m: *mut opaque::pa_mainloop,
            result: *mut c_int
        ) -> c_int;

        pub fn pa_signal_init(
            api: *mut opaque::pa_mainloop_api
        ) -> c_int;


        pub fn pa_proplist_new() -> *mut opaque::pa_proplist;


        pub fn pa_strerror(error: c_int) -> *const c_char;

        pub fn pa_context_get_sink_info_list(
            c: *mut opaque::pa_context,
            cb: cb::pa_sink_info_cb_t,
            userdata: *mut c_void
        ) -> *mut opaque::pa_operation;

        pub fn pa_context_get_server_info(
            c: *mut opaque::pa_context,
            cb: cb::pa_server_info_cb_t,
            userdata: *mut c_void
        ) -> *mut opaque::pa_operation;

    }
}
