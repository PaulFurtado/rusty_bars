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
type PaStreamRequestCallback = FnMut(PulseAudioStream, size_t) + Send; // XXX
type BoxedPaStreamRequestCallback = Box<PaStreamRequestCallback>;


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
