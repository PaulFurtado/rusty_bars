#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
use libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void};
use std::ffi::CString;
pub use pulse_types::*;
use std::ptr;
use std::mem;
use std::sync::{Arc, Mutex, MutexGuard};
use std::rc::{Rc, Weak};


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


/// A safe wrapper around pa_context_disconnect.
/// Immediately/synchronously disconnect from the PulseAudio server.
pub fn pa_context_disconnect(context: *mut opaque::pa_context) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_disconnect(context) };
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



// Types for callback closures
type StateCallback = Fn(Context, pa_context_state) + Send;
type ServerInfoCallback = Fn(Context, &pa_server_info) + Send;

// Boxed types for callback closures
type BoxedStateCallback = Box<StateCallback>;
type BoxedServerInfoCallback = Box<ServerInfoCallback>;




/// State callback for C to call. Takes a ContextInternal and calls its
/// server_info_callback method.
extern fn _state_callback(_: *mut pa_context, context: *mut c_void) {
    let context_internal = unsafe{ &* (context as *mut ContextInternal) };
    context_internal.state_callback();
}

/// Server info callback for C to call. Takes a ContextInternal and calls its
/// server_info_callback method.
extern fn _server_info_callback(_: *mut pa_context, info: *const pa_server_info, context: *mut c_void) {
    let context_internal = unsafe{ &* (context as *mut ContextInternal) };
    context_internal.server_info_callback(unsafe{ &*info });
}


/// A struct which wraps the PulseAudio async main loop.
pub struct PulseAudioMainloop {
    internal: *mut pa_mainloop
}


impl PulseAudioMainloop {
    /// Create a new mainloop.
    pub fn new() -> PulseAudioMainloop {
        PulseAudioMainloop{
            internal: pa_mainloop_new()
        }
    }

    /// Helper method to get the raw mainloop api
    fn get_raw_mainloop_api(&self) -> *mut pa_mainloop_api {
        pa_mainloop_get_api(self.internal)
    }

    //// Creates a new context with this mainloop
    pub fn create_context(&self, client_name: &str) -> Context {
        Context::new(self, client_name)
    }

    /// Run the mainloop.
    pub fn run(&self) {
        let mut mainloop_res: c_int = 0;
        pa_mainloop_run(self.internal, &mut mainloop_res);
        // TODO: error handling
    }
}


unsafe impl Send for ContextInternal {}



#[derive(Clone)]
pub struct Context {
    internal: Arc<Mutex<ContextInternal>>
}


impl Context {
    /// Get a new PulseAudio context. It's probably easier to get this via the
    /// mainloop.
    pub fn new(mainloop: &PulseAudioMainloop, client_name: &str) -> Context{
        let mut context = Context {
            internal: Arc::new(Mutex::new(ContextInternal::new(mainloop, client_name))),
        };
        {
            let internal_guard = context.internal.lock();
            let mut internal = internal_guard.unwrap();
            internal.external = Some(context.clone());
        }
        context
    }

    /// Get a mutable raw pointer to this object
    fn as_mut_ptr(&mut self) -> *mut Context {
        self
    }

    /// Get a mutable void pointer to this object
    fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }

    /// Set the callback for server state. This callback gets called many times.
    /// Do not start sending commands until this returns pa_context_state::READY
    pub fn set_state_callback<C>(&mut self, cb: C) where C: Fn(Context, pa_context_state) + Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.state_cb = Some(Box::new(cb) as BoxedStateCallback);
        pa_context_set_state_callback(internal.ptr, _state_callback, internal.as_void_ptr());
    }

    /// Connect to the server
    /// 1. Before calling this, you probably want to run set_state_callback
    /// 2. After setting a state callback, run this method
    /// 3. Directly after running this method, start the mainloop to start
    ///    getting callbacks.
    pub fn connect(&mut self, server: Option<&str>, flags: pa_context_flags) {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        pa_context_connect(internal.ptr, server, flags, None);
    }

    /// Gets basic information about the server. See the pa_server_info struct
    /// for more details.
    pub fn get_server_info<C>(&mut self, cb: C) where C: Fn(Context, &pa_server_info), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.server_info_cb = Some(Box::new(cb) as BoxedServerInfoCallback);
        pa_context_get_server_info(internal.ptr, _server_info_callback, internal.as_void_ptr());
    }
}


struct ContextInternal {
    /// A pointer to the pa_context object
    ptr: *mut pa_context,
    /// A pointer to our external API
    external: Option<Context>,
    /// Callback closure for state changes. Called every time the state changes
    state_cb: Option<BoxedStateCallback>,
    /// Callback closure for get_server_info. Called once per execution of
    /// get_server_info.
    server_info_cb: Option<BoxedServerInfoCallback>,
}


/// Currently the drop method has nothing to trigger it. Need to figure out a
/// game plan here.
impl Drop for ContextInternal {
    fn drop(&mut self) {
        println!("drop internal");
    }
}


impl ContextInternal {
    /// Never invoke directly. Always go through Context
    fn new(mainloop: &PulseAudioMainloop, client_name: &str) -> ContextInternal {
        ContextInternal{
            ptr: pa_context_new(mainloop.get_raw_mainloop_api(), client_name),
            // TODO: deal with dropping circular reference to external with the Arc
            external: None,
            state_cb: None,
            server_info_cb: None,
        }
    }

    /// Gets a new clone of the external API
    fn get_new_external(&self) -> Context {
        self.external.clone().unwrap()
    }

    /// Get the current context state. This function is synchronous.
    fn get_state(&self) -> pa_context_state {
        pa_context_get_state(self.ptr)
    }

    /// Get a c_void pointer to this object
    fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }

    /// Get a mutable raw pointer to this object
    fn as_mut_ptr(&mut self) -> *mut ContextInternal {
        self
    }


    /// Called back for state changes. Wraps the user's closure
    fn state_callback(&self) {
        let state = self.get_state();
        let mut external = self.external.clone().unwrap();
        match self.state_cb {
            Some(ref cb) => cb(external, state),
            None => println!("warning: no context state callback set")
        }
    }

    /// Called back for get_server_info. Wraps the user's closure
    fn server_info_callback(&self, info: &pa_server_info) {
        let mut external = self.external.clone().unwrap();
        match self.server_info_cb {
            Some(ref cb) => cb(external, info),
            None => println!("warning: no server info callback is set"),
        }
    }
}




/// Rusty wrapper for PulseAudio's API.
pub struct PulseAudioApi {
    context: *mut pa_context,
    mainloop: *mut pa_mainloop,
    mainloop_api: *mut pa_mainloop_api,
    state_cb: Option<BoxedStateCallback>,
}

unsafe impl Send for PulseAudioApi {}


impl PulseAudioApi {
    /// Create a new PulseAudioApi instance.
    /// client_name is the name this client will appear as to PulseAudio
    pub fn new(client_name: &str) -> PulseAudioApi {
        let mainloop = pa_mainloop_new();
        let mainloop_api = pa_mainloop_get_api(mainloop);
        let context = pa_context_new(mainloop_api, client_name);

        PulseAudioApi {
            mainloop: mainloop,
            mainloop_api: mainloop_api,
            context: context,
            state_cb: None,
        }
    }






    pub fn connect(&mut self, server: Option<&str>, flags: pa_context_flags) {
        pa_context_connect(self.context, server, flags, None);
    }


    fn state_callback(&mut self) {
        let self_ptr: *mut Self = self;
        let self2 = unsafe { &mut *self_ptr };

    /*
        match self.state_cb {
            Some(ref mut cb) => cb(self2, pa_context_get_state(self.context)),
            None => println!("Warning: No state callback set.")
        }*/
    }

    pub fn get_server_info<C>(&mut self, cb: C) where C: Fn(&mut PulseAudioApi, &pa_server_info) + 'static {
        /*
        let mut b = Box::new(cb) as BoxedServerInfoCallback;
        let mut wrapper = InfoCallbackWrapper::new(self, b);
        let mut boxed_wrapper = wrapper.to_box();
        let wrapper_ptr: *mut Box<InfoCallbackWrapper> = &mut boxed_wrapper;
        pa_context_get_server_info(self.context, _server_info_callback, wrapper_ptr as *mut c_void);
        unsafe{ mem::forget(boxed_wrapper) };*/
    }

    pub fn set_state_callback<C>(&mut self, cb: C) where C: Fn(&mut PulseAudioApi, pa_context_state) + Send+ 'static {
        //self.state_cb = Some(Box::new(cb) as BoxedStateCallback);
        pa_context_set_state_callback(self.context, _state_callback, self.as_void_ptr());
    }

    /// Runs the mainloop on the current thread.
    pub fn run_mainloop(&mut self) -> Result<(), String> {
        let mut mainloop_res: c_int = 0;
        pa_mainloop_run(self.mainloop, &mut mainloop_res);
        // TODO: handle errors
        Ok(())
    }

    /// Gets a raw pointer to this PulseAudioApi instance
    fn as_mut_ptr(&mut self) -> *mut Self {
        self
    }

    /// Gets a raw c_void pointer to this PulseAudioApi instance
    fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }
}


impl Drop for PulseAudioApi  {
    fn drop(&mut self) {
        pa_context_disconnect(self.context);
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


/*
pub fn pa_context_get_server_info_closure<C>(context: *mut opaque::pa_context, cb: C)
    where C: Fn(&pa_server_info) + 'static {

    let mut cb = Box::new(cb) as BoxedServerInfoCallback;
    let cbp: *mut BoxedServerInfoCallback = &mut cb;
    pa_context_get_server_info(context, _server_info_callback, cbp as *mut c_void);
}
*/

type ServerInfoRawCb = (Fn(&pa_server_info) + Send);
type BoxedServerInfoRawCb = Box<ServerInfoRawCb>;



pub fn pa_context_get_server_info_closure<C>(context: *mut pa_context, cb: C) where C: FnMut(&pa_server_info), C: Send {

    let boxed_cb: &mut Box<C> = &mut Box::new(cb);
    println!("Rectangle occupies {} bytes in the stack",
             mem::size_of_val(boxed_cb));
    let boxed_cb_ptr: *mut Box<C> = boxed_cb;
    let ptr_num = boxed_cb_ptr as u64;
    println!("created box: {:x}", ptr_num);
    unsafe{ mem::forget(boxed_cb) };
    //unsafe{ mem::forget(boxed_cb_ptr) };
    pa_context_get_server_info(context, pa_context_get_server_info_closure_cb::<C>, boxed_cb_ptr as *mut c_void);
    unsafe{ mem::forget(boxed_cb_ptr) };

}


extern "C" fn pa_context_get_server_info_closure_cb<C>(_: *mut pa_context, info: *const pa_server_info, userdata: *mut c_void) where C: FnMut(&pa_server_info), C: Send {
    println!("ptr num: {:x}", userdata as u64);
    let cb: &mut Box<C> = unsafe{ &mut * (userdata as *mut Box<C>)  };
    let ptr_num = (cb as *const Box<C>) as u64;
    println!("ptr num: {:x}", ptr_num);

    let info: pa_server_info = unsafe{ *info };
    println!("got info");
    cb(&info);
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

        pub fn pa_context_disconnect(context: *mut opaque::pa_context);

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
