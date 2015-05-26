/// This module contains a Rust interface to PulseAudio's C API.

extern crate libc;

use self::libc::c_int;

use pulse::ext;
use pulse::types::*;
use pulse::context::Context;

/// A struct which wraps the PulseAudio async main loop.
pub struct PulseAudioMainloop {
    internal: *mut pa_mainloop
}


impl<'a> PulseAudioMainloop {
    /// Create a new mainloop.
    pub fn new() -> PulseAudioMainloop {
        PulseAudioMainloop{
            internal: pa_mainloop_new()
        }
    }

    /// Helper method to get the raw mainloop api
    pub fn get_raw_mainloop_api(&self) -> *mut pa_mainloop_api {
        pa_mainloop_get_api(self.internal)
    }

    //// Creates a new context with this mainloop
    pub fn create_context(&'a self, client_name: &str) -> Context<'a> {
        Context::new(self, client_name)
    }

    /// Run the mainloop.
    pub fn run(&self) -> c_int {
        let mut result: c_int = 0;
        pa_mainloop_run(self.internal, &mut result);
        result
    }
}


/// A rust wrapper around pa_mainloop_run
pub fn pa_mainloop_run(mainloop: *mut opaque::pa_mainloop, result: &mut c_int) {
    assert!(!mainloop.is_null());
    let res = unsafe{ ext::pa_mainloop_run(mainloop, result as *mut c_int) };
    assert!(res == 0);
}
/// A safe interface to pa_mainloop_get_api
pub fn pa_mainloop_get_api(mainloop: *mut opaque::pa_mainloop) -> *mut opaque::pa_mainloop_api {
    assert!(!mainloop.is_null());
    let mainloop_api = unsafe { ext::pa_mainloop_get_api(mainloop) };
    assert!(!mainloop_api.is_null());
    return mainloop_api;
}

/// A safe interface to pa_mainloop_new
pub fn pa_mainloop_new() -> *mut opaque::pa_mainloop {
    let mainloop = unsafe{ ext::pa_mainloop_new() };
    assert!(!mainloop.is_null());
    return mainloop;
}
