#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
use libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void};
use std::ffi::CString;
pub use pulse_types::*;
use std::ptr;


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


mod ext {
    extern crate libc;
    use self::libc::{c_void, c_char, c_int};
    use pulse_types::*;

    #[link(name="pulse")]
    extern {
        pub fn pa_mainloop_new() -> *mut opaque::pa_mainloop;

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
    }
}
