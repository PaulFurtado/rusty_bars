#![allow(unstable)]

/// A module for connecting to a PulseAudio server.

extern crate libc;

use self::libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void};

use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use pulse::ext;
use pulse::mainloop::PulseAudioMainloop;
pub use pulse::stream::PulseAudioStream;
use pulse::types::*;


/// Types for callback closures
type StateCallback<'a> = FnMut(Context, pa_context_state) + 'a;
type ServerInfoCallback<'a> = FnMut(Context, &pa_server_info) + 'a;
type SinkInfoCallback<'a> = FnMut(Context, Option<&pa_sink_info>) + 'a;
type SubscriptionCallback<'a> = FnMut(Context, c_int, u32) + 'a;
type PaContextSuccessCallback<'a> = FnMut(Context, bool) + 'a;


/// Boxed types for callback closures.
type BoxedStateCallback<'a> = Box<StateCallback<'a>>;
type BoxedServerInfoCallback<'a> = Box<ServerInfoCallback<'a>>;
type BoxedSinkInfoCallback<'a> = Box<SinkInfoCallback<'a>>;
type BoxedSubscriptionCallback<'a> = Box<SubscriptionCallback<'a>>;
type BoxedPaContextSuccessCallback<'a> = Box<PaContextSuccessCallback<'a>>;


/// Represents a connection to a PulseAudio server.
#[derive(Clone)]
pub struct Context<'a> {
    internal: Rc<RefCell<ContextInternal<'a>>>
}


impl<'a> Context<'a> {
    /// Get a new PulseAudio context. It's probably easier to get this via the
    /// mainloop.
    pub fn new(mainloop: &PulseAudioMainloop, client_name: &str) -> Context<'a> {
        let context = Context {
            internal: Rc::new(RefCell::new(ContextInternal::new(mainloop, client_name))),
        };
        {
            let mut internal = context.internal.borrow_mut();
            internal.external = Some(context.clone());
        }
        context
    }

    /// Set the callback for server state. This callback gets called many times.
    /// Do not start sending commands until this returns pa_context_state::READY
    pub fn set_state_callback<C>(&self, cb: C) where C: FnMut(Context, pa_context_state) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.state_cb = Some(Box::new(cb) as BoxedStateCallback);
        pa_context_set_state_callback(internal.ptr, _state_callback, internal.as_void_ptr());
    }

    /// Connect to the server
    /// 1. Before calling this, you probably want to run set_state_callback
    /// 2. After setting a state callback, run this method
    /// 3. Directly after running this method, start the mainloop to start
    ///    getting callbacks.
    pub fn connect(&self, server: Option<&str>, flags: pa_context_flags) {
        let internal = self.internal.borrow_mut();
        pa_context_connect(internal.ptr, server, flags, None);
    }

    /// Gets basic information about the server. See the pa_server_info struct
    /// for more details.
    pub fn get_server_info<C>(&self, cb: C) where C: FnMut(Context, &pa_server_info) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.server_info_cb = Some(Box::new(cb) as BoxedServerInfoCallback);
        pa_context_get_server_info(internal.ptr, _server_info_callback, internal.as_void_ptr());
    }

    /// Get information about a sink using its name.
    /// PulseAudio uses the same callback type for getting a single sink as
    /// getting the list of all sinks, so a single sink works like a single
    /// element list. You should get two callbacks from this function: one with
    /// the information about the sink, and one with None indicating the end of
    /// the list.
    pub fn get_sink_info_by_name<C>(&self, name: &str, cb: C) where C: FnMut(Context, Option<&pa_sink_info>) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.sink_info_cb = Some(Box::new(cb) as BoxedSinkInfoCallback);
        pa_context_get_sink_info_by_name(internal.ptr, name, _sink_info_callback, internal.as_void_ptr());
    }

    /// Adds an event subscription
    pub fn add_subscription<C>(&self, mask: pa_subscription_mask, cb: C) where C: FnMut(Context, bool) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.context_success_cb = Some(Box::new(cb) as BoxedPaContextSuccessCallback);
        internal.subscriptions.add(mask);
        let new_mask = internal.subscriptions.get_mask();
        pa_context_subscribe(internal.ptr, new_mask, _subscription_success_callback, internal.as_void_ptr());
    }

    /// Removes an event subscription
    pub fn remove_subscription<C>(&self, mask: pa_subscription_mask, cb: C) where C: FnMut(Context, bool) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.context_success_cb = Some(Box::new(cb) as BoxedPaContextSuccessCallback);
        internal.subscriptions.remove(mask);
        let new_mask = internal.subscriptions.get_mask();
        pa_context_subscribe(internal.ptr, new_mask, _subscription_success_callback, internal.as_void_ptr());
    }

    /// Set the callback for subscriptions
    pub fn set_event_callback<C>(&self, cb: C) where C: FnMut(Context, c_int, u32) + 'a {
        let mut internal = self.internal.borrow_mut();
        internal.event_cb = Some(Box::new(cb) as BoxedSubscriptionCallback);
        pa_context_set_subscribe_callback(internal.ptr, _subscription_event_callback, internal.as_void_ptr());
    }

    /// Create an unconnected PulseAudioStream on this server.
    ///
    /// Args:
    ///    name: A name for this stream.
    ///    ss: The sample format of the stream.
    ///    map: The desired channel. If None, PulseAudio uses its default.
    pub fn create_stream(&mut self, name: &str, ss: &pa_sample_spec, map: Option<&pa_channel_map>) -> PulseAudioStream<'a> {
        let internal = self.internal.borrow_mut();

        let channel_map_ptr: *const pa_channel_map = match map {
            Some(map) => map,
            None => ptr::null()
        };

        PulseAudioStream::new(internal.ptr, name, ss, channel_map_ptr)
    }
}


/// Inner representation of a Context.
/// Used to call back to Rust from C.
struct ContextInternal<'a> {
    /// A pointer to the pa_context object
    ptr: *mut pa_context,
    /// A pointer to our external API
    external: Option<Context<'a>>,
    /// Callback closure for state changes. Called every time the state changes
    state_cb: Option<BoxedStateCallback<'a>>,
    /// Callback closure for get_server_info. Called once per execution of
    /// get_server_info.
    server_info_cb: Option<BoxedServerInfoCallback<'a>>,
    /// Callback closure for getting sink info.  Called once for for each
    /// element in the list of sinks
    sink_info_cb: Option<BoxedSinkInfoCallback<'a>>,
    /// Called for events
    event_cb: Option<BoxedSubscriptionCallback<'a>>,
    /// Called for event subscription events
    context_success_cb: Option<BoxedPaContextSuccessCallback<'a>>,
    /// Manages subscriptions to events
    subscriptions: SubscriptionManager,
}


impl<'a> ContextInternal<'a> {
    /// Never invoke directly. Always go through Context
    fn new(mainloop: &PulseAudioMainloop, client_name: &str) -> ContextInternal<'a> {
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

    /// Get the current context state. This function is synchronous.
    fn get_state(&self) -> pa_context_state {
        pa_context_get_state(self.ptr)
    }

    /// Get a c_void pointer to this object
    fn as_void_ptr(&mut self) -> *mut c_void {
        self.as_mut_ptr() as *mut c_void
    }

    /// Get a mutable raw pointer to this object
    fn as_mut_ptr(&mut self) -> *mut ContextInternal<'a> {
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


/// Currently the drop method has nothing to trigger it. Need to figure out a
/// game plan here.
#[unsafe_destructor]
impl<'a> Drop for ContextInternal<'a> {
    fn drop(&mut self) {
        println!("drop ContextInternal");
    }
}


/// A safe interface to pa_context_set_state_callback
pub fn pa_context_set_state_callback(context: *mut opaque::pa_context,
    cb: cb::pa_context_notify_cb_t, userdata: *mut c_void) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_set_state_callback(context, cb, userdata) };
}


/// A safe wrapper for pa_context_connect
fn pa_context_connect(context: *mut opaque::pa_context, server_name: Option<&str>,
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


/// A rust wrapper around pa_context_disconnect.
/// Immediately/synchronously disconnect from the PulseAudio server.
pub fn pa_context_disconnect(context: *mut opaque::pa_context) {
    assert!(!context.is_null());
    unsafe { ext::pa_context_disconnect(context) };
}


/// Gets sink info by the sink's name
pub fn pa_context_get_sink_info_by_name(c: *mut pa_context, name: &str, cb: pa_sink_info_cb_t, userdata: *mut c_void) {
    assert!(!c.is_null());
    let name = CString::from_slice(name.as_bytes());
    unsafe{ ext::pa_context_get_sink_info_by_name(c, name.as_ptr(), cb, userdata) };
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

/// A safe interface to pa_context_new
pub fn pa_context_new(mainloop_api: *mut opaque::pa_mainloop_api, client_name: &str) -> *mut opaque::pa_context {
    assert!(!mainloop_api.is_null());
    let client_name_c = CString::from_slice(client_name.as_bytes());
    let context = unsafe{ ext::pa_context_new(mainloop_api, client_name_c.as_ptr()) };
    assert!(!context.is_null());
    return context;
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
