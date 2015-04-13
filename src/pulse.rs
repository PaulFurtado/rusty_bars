#![feature(unsafe_destructor)]
#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
use libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void, size_t};
use std::ffi::CString;
pub use pulse_types::*;
use std::ptr;
use std::mem;
use std::fmt;
use std::slice;
use std::sync::{Arc, Mutex};
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};


// Types for callback closures
type StateCallback = FnMut(Context, pa_context_state) + Send;
type ServerInfoCallback = FnMut(Context, &pa_server_info) + Send;
type SinkInfoCallback = FnMut(Context, Option<&pa_sink_info>) + Send;
type SubscriptionCallback = FnMut(Context, c_int, u32) + Send;
type PaContextSuccessCallback = FnMut(Context, bool) + Send;
type PaStreamRequestCallback = FnMut(PulseAudioStream, size_t) + Send; // XXX


// Boxed types for callback closures
type BoxedStateCallback = Box<StateCallback>;
type BoxedServerInfoCallback = Box<ServerInfoCallback>;
type BoxedSinkInfoCallback = Box<SinkInfoCallback>;
type BoxedSubscriptionCallback = Box<SubscriptionCallback>;
type BoxedPaContextSuccessCallback = Box<PaContextSuccessCallback>;
type BoxedPaStreamRequestCallback = Box<PaStreamRequestCallback>;


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
        let context = Context {
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
    pub fn set_state_callback<C>(&mut self, cb: C) where C: FnMut(Context, pa_context_state) + Send {
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
    pub fn connect(&self, server: Option<&str>, flags: pa_context_flags) {
        let internal_guard = self.internal.lock();
        let internal = internal_guard.unwrap();
        pa_context_connect(internal.ptr, server, flags, None);
    }

    /// Gets basic information about the server. See the pa_server_info struct
    /// for more details.
    pub fn get_server_info<C>(&self, cb: C) where C: FnMut(Context, &pa_server_info), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.server_info_cb = Some(Box::new(cb) as BoxedServerInfoCallback);
        pa_context_get_server_info(internal.ptr, _server_info_callback, internal.as_void_ptr());
    }

    /// Get information about a sink using its name.
    /// PulseAudio uses the same callback type for getting a single sink as
    /// getting the list of all sinks, so a single sink works like a single
    /// element list. You should get two callbacks from this function: one with
    /// the information about the sink, and one with None indicating the end of
    /// the list.
    pub fn get_sink_info_by_name<C>(&self, name: &str, cb: C) where C: FnMut(Context, Option<&pa_sink_info>), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.sink_info_cb = Some(Box::new(cb) as BoxedSinkInfoCallback);
        pa_context_get_sink_info_by_name(internal.ptr, name, _sink_info_callback, internal.as_void_ptr());
    }

    /// Adds an event subscription
    pub fn add_subscription<C>(&self, mask: pa_subscription_mask, cb: C) where C: FnMut(Context, bool), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.context_success_cb = Some(Box::new(cb) as BoxedPaContextSuccessCallback);
        internal.subscriptions.add(mask);
        let new_mask = internal.subscriptions.get_mask();
        pa_context_subscribe(internal.ptr, new_mask, _subscription_success_callback, internal.as_void_ptr());
    }

    /// Removes an event subscription
    pub fn remove_subscription<C>(&self, mask: pa_subscription_mask, cb: C) where C: FnMut(Context, bool), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.context_success_cb = Some(Box::new(cb) as BoxedPaContextSuccessCallback);
        internal.subscriptions.remove(mask);
        let new_mask = internal.subscriptions.get_mask();
        pa_context_subscribe(internal.ptr, new_mask, _subscription_success_callback, internal.as_void_ptr());
    }

    /// Sets the callback for subscriptions
    pub fn set_event_callback<C>(&self, cb: C) where C: FnMut(Context, c_int, u32), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.event_cb = Some(Box::new(cb) as BoxedSubscriptionCallback);
        pa_context_set_subscribe_callback(internal.ptr, _subscription_event_callback, internal.as_void_ptr());
    }


    /// Create an unconnected PulseAudioStream from this server.
    /// Args:
    ///    name: a name for this stream
    ///    ss: the sample format of the stream
    ///    map: the desired channel
    pub fn create_stream(&self, name: &str, ss: &pa_sample_spec, map: Option<&pa_channel_map>) -> PulseAudioStream {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();

        let channel_map_ptr: *const pa_channel_map = match map {
            Some(map) => map,
            None => ptr::null()
        };

        PulseAudioStream::new(internal.ptr, name, ss, channel_map_ptr)
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
    /// Callback closure for getting sink info.  Called once for for each
    /// element in the list of sinks
    sink_info_cb: Option<BoxedSinkInfoCallback>,
    /// Called for events
    event_cb: Option<BoxedSubscriptionCallback>,
    /// Called for event subscription events
    context_success_cb: Option<BoxedPaContextSuccessCallback>,
    /// Manages subscriptions to events
    subscriptions: SubscriptionManager,
}


/// Currently the drop method has nothing to trigger it. Need to figure out a
/// game plan here.
impl Drop for ContextInternal {
    fn drop(&mut self) {
        println!("drop ContextInternal");
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
            sink_info_cb: None,
            event_cb: None,
            context_success_cb: None,
            subscriptions: SubscriptionManager::new()
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
    fn state_callback(&mut self) {
        let state = self.get_state();
        let external = self.external.clone().unwrap();
        match self.state_cb {
            Some(ref mut cb) => cb(external, state),
            None => println!("warning: no context state callback set")
        }
    }

    /// Called back for get_server_info. Wraps the user's closure
    fn server_info_callback(&mut self, info: &pa_server_info) {
        let external = self.external.clone().unwrap();
        match self.server_info_cb {
            Some(ref mut cb) => cb(external, info),
            None => println!("warning: no server info callback is set"),
        }
    }

    /// Called back for the sink_info_list and get_sink_info commands
    fn sink_info_callback(&mut self, info: Option<&pa_sink_info>) {
        let external = self.external.clone().unwrap();
        match self.sink_info_cb {
            Some(ref mut cb) => cb(external, info),
            None => println!("warning: no sink info callback is set"),
        }
    }

    fn event_callback(&mut self, t: c_int, idx: u32) {
        let external = self.external.clone().unwrap();
        match self.event_cb {
            Some(ref mut cb) => cb(external, t, idx),
            None => println!("warning: no event callback is set")
        }
    }

    fn subscription_success_callback(&mut self, success: bool) {
        let external = self.external.clone().unwrap();
        match self.context_success_cb {
            Some(ref mut cb) => cb(external, success),
            None => println!("warning: no success callback is set"),
        }
    }

}


struct SubscriptionManager {
    mask: c_int,

}

impl SubscriptionManager {
    /// Create a new SubscriptionManager
    fn new() -> SubscriptionManager {
        SubscriptionManager {
            mask: 0,
        }
    }

    /// Get the current subscription mask
    pub fn get_mask(&self) -> c_int {
        self.mask
    }

    /// Add a subscription
    pub fn add(&mut self, sub: pa_subscription_mask) {
        self.mask |= sub as c_int;
    }

    /// Remove a subscription
    pub fn remove(&mut self, sub: pa_subscription_mask) {
        self.mask &= !(sub as c_int);
    }

    /// Check iof a subscription is enabled
    pub fn is_enabled(&self, sub: pa_subscription_mask) -> bool {
        let sub_int = sub as c_int;
        (self.mask & sub_int) == sub_int
    }
}


/// State callback for C to call. Takes a ContextInternal and calls its
/// server_info_callback method.
extern fn _state_callback(_: *mut pa_context, context: *mut c_void) {
    let context_internal = unsafe{ &mut * (context as *mut ContextInternal) };
    context_internal.state_callback();
}

/// Server info callback for C to call. Takes a ContextInternal and calls its
/// server_info_callback method.
extern fn _server_info_callback(_: *mut pa_context, info: *const pa_server_info, context: *mut c_void) {
    let context_internal = unsafe{ &mut * (context as *mut ContextInternal) };
    context_internal.server_info_callback(unsafe{ &*info });
}

/// Sink info callback for C to call.
extern fn _sink_info_callback(_: *mut pa_context, info: *const pa_sink_info, eol: c_int, context: *mut c_void) {
    let context_internal = unsafe{ &mut * (context as *mut ContextInternal) };
    if eol == 1 || info.is_null() {
        context_internal.sink_info_callback(None);
    } else {
        context_internal.sink_info_callback(Some(unsafe{ &*info }));
    }
}

/// Subscription callback for C to call.
extern fn _subscription_event_callback(_: *mut pa_context, t: c_int, idx: u32, context: *mut c_void) {
    let context_internal = unsafe{ &mut * (context as *mut ContextInternal) };
    context_internal.event_callback(t, idx);
}

/// Called back to tell you if a subscription succeded or failed.
extern fn _subscription_success_callback(_: *mut pa_context, success: c_int,  context: *mut c_void) {
    let context_internal = unsafe{ &mut * (context as *mut ContextInternal) };
    context_internal.subscription_success_callback(success==1);
}


// XXX
/// Wrapper for a PulseAudio stream read callback. Called by C when there is
/// audio data available to read.
extern fn _pa_stream_read_callback(
    _: *mut opaque::pa_stream, nbytes: size_t,  userdata: *mut c_void) {

    let stream_internal = unsafe{ &mut * (
        userdata as *mut PulseAudioStreamInternal) };
    stream_internal.read_callback(nbytes);
}




/// A safe interface to pa_context_set_state_callback
pub fn pa_context_set_state_callback(context: *mut opaque::pa_context,
    cb: cb::pa_context_notify_cb_t, userdata: *mut c_void) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_set_state_callback(context, cb, userdata) };
}

/// A rust wrapper around pa_context_disconnect.
/// Immediately/synchronously disconnect from the PulseAudio server.
pub fn pa_context_disconnect(context: *mut opaque::pa_context) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_disconnect(context) };
}

/// A rust wrapper around pa_mainloop_run
pub fn pa_mainloop_run(mainloop: *mut opaque::pa_mainloop, result: &mut c_int) {
    assert!(!mainloop.is_null());
    let res = unsafe{ ext::pa_mainloop_run(mainloop, result as *mut c_int) };
    assert!(res == 0);
}

/// A rust wrapper around pa_context_get_state
pub fn pa_context_get_state(context: *mut opaque::pa_context) -> enums::pa_context_state {
    assert!(!context.is_null());
    unsafe{ ext::pa_context_get_state(context) }
}


/// A rust wrapper around pa_context_get_sink_info_list
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

/// A rust wrapper around pa_context_get_server_info
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

/// Gets sink info by the sink's name
pub fn pa_context_get_sink_info_by_name(c: *mut pa_context, name: &str, cb: pa_sink_info_cb_t, userdata: *mut c_void) {
    assert!(!c.is_null());
    let name = CString::from_slice(name.as_bytes());
    unsafe{ ext::pa_context_get_sink_info_by_name(c, name.as_ptr(), cb, userdata) };
}

/// Subscribe to an event
pub fn pa_context_subscribe(c: *mut pa_context, m: c_int, cb: pa_context_success_cb_t, userdata: *mut c_void) {
    assert!(!c.is_null());
    unsafe{ ext::pa_context_subscribe(c, m, cb, userdata) };
}

/// Set the callback for all subscriptions
pub fn pa_context_set_subscribe_callback(c: *mut pa_context, cb: pa_context_subscribe_cb_t, userdata: *mut c_void) {
    assert!(!c.is_null());
    unsafe{ ext::pa_context_set_subscribe_callback(c, cb, userdata) };
}


/// Set a callback for when there's data available to be read.
pub fn pa_stream_set_read_callback(
    p: *mut opaque::pa_stream,
    cb: pa_stream_request_cb_t,
    userdata: *mut c_void) {
    assert!(!p.is_null());
    unsafe { ext::stream::pa_stream_set_read_callback(p, cb, userdata) }
}

/// Create a new pa_stream
fn pa_stream_new(c: *mut opaque::pa_context, name: &str, ss: *const pa_sample_spec, map: *const pa_channel_map) -> *mut opaque::pa_stream {
    assert!(!c.is_null());
    let name = CString::from_slice(name.as_bytes());
    let res = unsafe { ext::stream::pa_stream_new(c, name.as_ptr(), ss, map) };
    assert!(!res.is_null());
    res
}


/// Set data to a pointer to audio data, and nbytes to point to the amount of
/// bytes available.
fn pa_stream_peek (
    stream: *mut opaque::pa_stream, data: *mut *mut u8,
    nbytes: *mut size_t) -> c_int {
    assert!(!stream.is_null());
    assert!(!data.is_null());
    assert!(!nbytes.is_null());
    return unsafe { ext::stream::pa_stream_peek(stream, data, nbytes) };
}


/// Sets a pa_stream to record from a source.
fn pa_stream_connect_record(
    stream: *mut opaque::pa_stream,
    source_name: Option<&str>,
    buffer_attributes: Option<&pa_buffer_attr>,
    stream_flags: Option<pa_stream_flags_t>) -> Result<c_int, String> {

    assert!(!stream.is_null());

    let dev: *const c_char = match source_name {
        None => ptr::null(),
        Some(name) => {
            let cstr = CString::from_slice(name.as_bytes());
            let cstr_ptr = cstr.as_ptr();
            unsafe{ mem::forget(cstr) };
            cstr_ptr
        }
    };

    let attr: *const pa_buffer_attr = match buffer_attributes {
        None => ptr::null(),
        Some(attributes) => attributes
    };

    let flags: pa_stream_flags_t = match stream_flags {
        None => pa_stream_flags_t::PA_STREAM_NOFLAGS,
        Some(stream_flags) => stream_flags
    };

    let res = unsafe {
        ext::stream::pa_stream_connect_record(stream, dev, attr, flags)
    };

    if res < 0 {
        Err("unknown error".to_string())
    } else {
        Ok(res)
    }

}




/// Holds members and callbacks of PulseAudioStream.
struct PulseAudioStreamInternal {
    /// The underlying pulse audio stream
    pa_stream: *mut opaque::pa_stream,
    /// A pointer to the external PulseAudioStream
    external: Option<PulseAudioStream>,
    /// Called when the stream has data available for reading
    read_cb: Option<BoxedPaStreamRequestCallback>
}


impl PulseAudioStreamInternal {
    /// Never invoke this. Use PulseAudioStream instead.
    fn new(stream: *mut opaque::pa_stream) -> Self {
        PulseAudioStreamInternal {
            pa_stream: stream,
            external: None,
            read_cb: None
        }
    }

    /// Called when the underlying stream has data available
    /// for reading.
    pub fn read_callback(&mut self, nbytes: size_t) {
        assert!(!self.external.is_none());
        assert!(!self.pa_stream.is_null());

        let external = self.external.clone().unwrap();
        match self.read_cb {
            Some(ref mut cb) => cb(external, nbytes),
            None => println!("[PulseAudioStream] warning: read callback called, no read callback set.")
        }
    }

    /// Get a c_void pointer to this object
    pub fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }

    /// Get a mutable raw pointer to this object
    pub fn as_mut_ptr(&mut self) -> *mut PulseAudioStreamInternal {
        self
    }
}


impl Drop for PulseAudioStreamInternal {
    /// TODO: profile this.
    fn drop(&mut self) {
        println!("drop pulse audio stream internal");
    }
}


/// Represents a Pulse Audio stream.
/// Can be used to write to a sink or read from a source.
#[derive(Clone)]
pub struct PulseAudioStream {
    internal: Arc<Mutex<PulseAudioStreamInternal>>,
    /// The last position of the stream's read buffer.
    _last_ptr: *const u8,
}


impl PulseAudioStream {
    /// Create a new stream on a context.
    /// Args:
    ///     context: the pulse audio context to attach to
    ///     name: the name of this stream
    ///     ss: the sample spec for this stream
    ///     map: the channel map for this stream
    pub fn new(context: *mut pa_context, name: &str, ss: *const pa_sample_spec,
        map: *const pa_channel_map) -> Self {

        let stream = pa_stream_new(context, name, ss, map);

        let internal = PulseAudioStreamInternal::new(stream);


        let stream = PulseAudioStream {
            internal: Arc::new(Mutex::new(internal)),
            _last_ptr: ptr::null()
        };

        {
            let internal_guard = stream.internal.lock();
            let mut internal = internal_guard.unwrap();
            internal.external = Some(stream.clone());
        }
        stream
    }

    /// Return the current fragment from Pulse's record stream.
    /// To return the next fragment, drop_fragment must be called after peeking.
    pub fn peek(&mut self) -> IoResult<&[u8]> {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();

        let mut buf: *mut u8 = ptr::null_mut();
        let mut bufptr: *mut *mut u8 = &mut buf;


        let mut nbytes: size_t = 0;

        let mut ret: c_int = 0;
        unsafe {
            ret = pa_stream_peek(internal.pa_stream, bufptr, &mut nbytes);
        };

        if (buf.is_null()) {
            if (nbytes == 0) {
                return Err(IoError {
                    kind: IoErrorKind::NoProgress,
                    desc: "Buffer is empty",
                    detail: None
                });
            } else {
                return Err(IoError {
                    kind: IoErrorKind::OtherIoError,
                    desc: "Hole in input buffer",
                    detail: Some(fmt::format(format_args!("hole of size {}", nbytes))),
                });
            }
        }

        self._last_ptr = buf as *const u8;
        unsafe {
            Ok(slice::from_raw_buf(&self._last_ptr, nbytes as usize))
        }
    }

    /// Drops the current fragment in Pulse's record stream.
    /// Can only be called after peek.
    pub fn drop_fragment(&self) -> IoResult<c_int> {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();

        unsafe { Ok(ext::stream::pa_stream_drop(internal.pa_stream)) }
    }

    /// Record playback from a source.
    /// Args:
    ///    source_name: The name of the source to record from. If none, use the
    ///        default source.
    ///    buffer_attributes: Options on the default buffer.
    ///    stream_flags: Options for the stream.
    pub fn connect_record(
        &mut self,
        source_name: Option<&str>,
        buffer_attributes: Option<&pa_buffer_attr>,
        stream_flags: Option<pa_stream_flags_t>) -> Result<c_int, String> {

        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();

        pa_stream_connect_record(
            internal.pa_stream, source_name, buffer_attributes, stream_flags)
    }

    /// Disconnect the stream from its source/sink.
    pub fn disconnect(&mut self) {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();

        unsafe { ext::stream::pa_stream_disconnect(internal.pa_stream) }
    }

    /// Get a mutable raw pointer to this object
    fn as_mut_ptr(&mut self) -> *mut PulseAudioStream {
        self
    }

    /// Get a mutable void pointer to this object
    fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }

    /// Sets the read callback
    pub fn set_read_callback<C>(&mut self, cb: C) where C: FnMut(PulseAudioStream, size_t), C: Send {
        let internal_guard = self.internal.lock();
        let mut internal = internal_guard.unwrap();
        internal.read_cb = Some(Box::new(cb) as BoxedPaStreamRequestCallback);
        pa_stream_set_read_callback(
            internal.pa_stream,
            _pa_stream_read_callback,
            internal.as_void_ptr());
    }
}


impl Drop for PulseAudioStream {
    fn drop(&mut self) {
        // TODO
        //self.disconnect()
    }
}


unsafe impl Send for PulseAudioStreamInternal {}


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

        pub fn pa_context_get_sink_info_by_name(
            c: *mut pa_context,
            name: *const c_char,
            cb: pa_sink_info_cb_t,
            userdata: *mut c_void
        ) -> *mut pa_operation;

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

        pub fn pa_context_subscribe(
            c: *mut opaque::pa_context,
            m: c_int,
            cb: pa_context_success_cb_t,
            userdata: *mut c_void
        ) -> *mut opaque::pa_operation;

        pub fn pa_context_set_subscribe_callback(
            c: *mut opaque::pa_context,
            cb: pa_context_subscribe_cb_t,
            userdata: *mut c_void
        );
    }


    pub mod stream {
        extern crate libc;
        use self::libc::{c_void, c_char, c_int, size_t};
        use pulse_types::*;

        #[link(name="pulse")]
        extern {
            pub fn pa_stream_new(
                c: *mut opaque::pa_context,
                name: *const c_char,
                ss: *const pa_sample_spec,
                map: *const pa_channel_map
           ) -> *mut opaque::pa_stream;

            pub fn pa_stream_new_extended(
                c: *mut opaque::pa_context,
                name: *const c_char,
                formats: *const *const pa_format_info,
                n_formats: c_int,
                p: *mut pa_proplist
            ) -> *mut opaque::pa_stream;

            pub fn pa_stream_set_read_callback(
                p: *mut opaque::pa_stream,
                cb: pa_stream_request_cb_t,
                userdata: *mut c_void);

            pub fn pa_stream_disconnect(s: *mut pa_stream);

            /// Sets data to a pointer to readable data, and nbytes to the
            /// amount of data available. If data is null and nbytes is 0,
            /// there is no data to read. If data is null and nbytes is >0,
            /// there is a hole in the stream's buffer.
            pub fn pa_stream_peek(
                p: *mut pa_stream,
                data: *mut *mut u8,
                nbytes: *mut size_t
            ) -> c_int;

            /// Drops the data in the stream's current buffer.
            pub fn pa_stream_drop(p: *mut pa_stream) -> c_int;

            /// Connects a stream to a source.
            pub fn pa_stream_connect_record(
                s: *mut pa_stream,
                dev: *const c_char,
                attr: *const pa_buffer_attr,
                flags: pa_stream_flags_t) -> c_int;
        }
    }
}
