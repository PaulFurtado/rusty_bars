#![allow(unstable)]
#![allow(dead_code)]


extern crate libc;
use self::libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void, size_t};
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::fmt;
use std::slice;
use std::sync::{Arc, Mutex};
use std::io::{Reader, Writer, IoResult, IoError, IoErrorKind};

pub use pulse_types::*;
pub use self::stream::*;

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
