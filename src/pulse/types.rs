#![allow(non_camel_case_types)]
#![allow(raw_pointer_derive)]
#![allow(missing_copy_implementations)]

pub use self::cb::*;
pub use self::opaque::*;
pub use self::enums::*;
pub use self::structs::*;
pub use self::types::*;


/// For callback signatures
pub mod cb {
    extern crate libc;
    use self::libc::{c_int, c_void, size_t};
    use super::opaque::*;
    use super::structs::*;

    pub type pa_context_notify_cb_t = extern "C" fn(
        *mut pa_context,
        *mut c_void
    );

    pub type pa_sink_info_cb_t = extern "C" fn(
        c: *mut pa_context,
        i: *const pa_sink_info,
        eol: c_int,
        userdata: *mut c_void
    );

    pub type pa_server_info_cb_t = extern "C" fn(
        c: *mut pa_context,
        i: *const pa_server_info,
        *mut c_void
    );

    pub type pa_stream_request_cb_t = extern "C" fn(
        p: *mut pa_stream,
        nbytes: size_t,
        *mut c_void
    );

    pub type pa_context_subscribe_cb_t = extern "C" fn(
        p: *mut pa_context,
        t: c_int,
        idx: u32,
        userdata: *mut c_void
    );

    pub type pa_context_success_cb_t = extern "C" fn(
        c: *mut pa_context,
        success: c_int,
        userdata: *mut c_void,
    );
}

/// For types we only have pointers to. Use structs with names so there is at
/// least pointer type safety.
pub mod opaque {
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
    pub struct pa_operation;
}

/// For enum types
pub mod enums {
    #[repr(C)]
    #[derive(Copy,Clone)]
    pub enum pa_stream_direction {
        NoDirection,
        StreamPlayback,
        StreamRecord,
        StreamUpload
    }

    #[repr(C)]
    #[derive(Copy,Clone)]
    pub enum pa_context_flags {
        NOAUTOSPAWN, // Disabled autospawning of the PulseAudio daemon if required.
        NOFAIL       // Don't fail if the daemon is not available when pa_context_connect()
                     // is called, instead enter PA_CONTEXT_CONNECTING state and wait for
                     // the daemon to appear.
    }

    #[repr(C)]
    #[derive(Copy,Clone)]
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
    #[derive(Copy,Clone)]
    pub enum pa_sample_format {
        PA_SAMPLE_U8,
        PA_SAMPLE_ALAW,
        PA_SAMPLE_ULAW,
        PA_SAMPLE_S16LE,
        PA_SAMPLE_S16BE,
        PA_SAMPLE_FLOAT32LE,
        PA_SAMPLE_FLOAT32BE,
        PA_SAMPLE_S32LE,
        PA_SAMPLE_S32BE,
        PA_SAMPLE_S24LE,
        PA_SAMPLE_S24BE,
        PA_SAMPLE_S24_32LE,
        PA_SAMPLE_S24_32BE,
        PA_SAMPLE_MAX,
        PA_SAMPLE_INVALID = -1
    }

    #[repr(C)]
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
        pub enum pa_sink_state_t {
        PA_SINK_INVALID_STATE = -1,
        PA_SINK_RUNNING = 0,
        PA_SINK_IDLE = 1,
        PA_SINK_SUSPENDED = 2,
        PA_SINK_INIT = -2,
        PA_SINK_UNLINKED = -3
    }

    #[repr(C)]
        pub enum pa_sink_flags_t {
        PA_SINK_NOFLAGS = 0x0000isize,
        PA_SINK_HW_VOLUME_CTRL = 0x0001isize,
        PA_SINK_LATENCY = 0x0002isize,
        PA_SINK_HARDWARE = 0x0004isize,
        PA_SINK_NETWORK = 0x0008isize,
        PA_SINK_HW_MUTE_CTRL = 0x0010isize,
        PA_SINK_DECIBEL_VOLUME = 0x0020isize,
        PA_SINK_FLAT_VOLUME = 0x0040isize,
        PA_SINK_DYNAMIC_LATENCY = 0x0080isize,
    }

    #[repr(C)]
        pub enum pa_encoding_t {
        PA_ENCODING_ANY,
        PA_ENCODING_PCM,
        PA_ENCODING_AC3_IEC61937,
        PA_ENCODING_EAC3_IEC61937,
        PA_ENCODING_MPEG_IEC61937,
        PA_ENCODING_DTS_IEC61937,
        PA_ENCODING_MPEG2_AAC_IEC61937,
        PA_ENCODING_MAX,
        PA_ENCODING_INVALID = -1,
    }

    #[repr(C)]
        pub enum pa_subscription_event_type {
        SINK = 0x0000,
        SOURCE = 0x0001,
        SINK_INPUT = 0x0002,
        SOURCE_OUTPUT = 0x0003,
        MODULE = 0x0004,
        CLIENT = 0x0005,
        SAMPLE_CACHE = 0x0006,
        SERVER = 0x0007,
        AUTOLOAD = 0x0008,
        CARD = 0x0009,
        FACILITY_MASK = 0x000F,
        //XXX: Sigh. PulseAudio uses 0 in this enum multiple times. Rust doesn't
        //XXX: support this.
        //NEW = 0x0000,
        CHANGE = 0x0010,
        REMOVE = 0x0020,
        TYPE_MASK = 0x0030,
    }

    #[repr(C)]
        pub enum pa_subscription_mask {
        NULL = 0x0000,
        SINK = 0x0001,
        SOURCE = 0x0002,
        SINK_INPUT = 0x0004,
        SOURCE_OUTPUT = 0x0008,
        MODULE = 0x0010,
        CLIENT = 0x0020,
        SAMPLE_CACHE = 0x0040,
        SERVER = 0x0080,
        AUTOLOAD = 0x0100,
        CARD = 0x0200,
    }

    #[repr(C)]
        pub enum pa_stream_flags_t {
        /// Default option -- no flags necessary.
        PA_STREAM_NOFLAGS = 0x0000,
        /// Starts the stream as "corked", requiring it to be uncorked before
        /// playback.
        PA_STREAM_START_CORKED = 0x0001,
        /// Interpolate the latency for this stream.
        PA_STREAM_INTERPOLATE_TIMING = 0x0002,
        /// Don't force time to increase monotonically.
        PA_STREAM_NOT_MONOTONIC = 0x0004,
        /// Issue timing updates automatically.
        PA_STREAM_AUTO_TIMING_UPDATE = 0x0008,
        /// Remap channels by index instead of name.
        /// Ignored by PA servers <0.9.8
        PA_STREAM_NO_REMAP_CHANNELS = 0x0010,
        /// Don't up/downmix channels remapped by name to related channels.
        /// Ignored by PA servers <0.9.8
        PA_STREAM_NO_REMIX_CHANNELS = 0x0020,
        /// Use the sample format of the sink/device this stream is connected to
        /// Ignored by PA servers <0.9.8
        PA_STREAM_FIX_FORMAT = 0x0040,
        /// Use the sample rate of the sink.
        /// Ignored by PA servers <0.9.8
        PA_STREAM_FIX_RATE = 0x0080,
        /// Use the sink's number of channels and channel map.
        /// Ignored by PA servers <0.9.8
        PA_STREAM_FIX_CHANNELS = 0x0100,
        /// Don't allow moving of this stream to another sink/device.
        /// Ignored by PA servers <0.9.8
        PA_STREAM_DONT_MOVE = 0x0200,
        /// Allow dynamic changing of the sampling rate during playback
        /// Ignored by PA servers <0.9.8
        PA_STREAM_VARIABLE_RATE = 0x0400,
        /// Find peaks instead of resampling.
        /// Ignored by PA servers <0.9.11
        PA_STREAM_PEAK_DETECT = 0x0800,
        /// Create in muted state.
        /// If neither PA_STREAM_START_UNMUTED nor PA_STREAM_START_MUTED is set,
        /// the server to decide if the stream is muted or not on creation.
        /// Ignored by PA servers <0.9.11
        PA_STREAM_START_MUTED = 0x1000,
        /// Adjust sink/source's latency based on the requested buffer metrics.
        /// Ignored by PA servers <0.9.11
        PA_STREAM_ADJUST_LATENCY = 0x2000,
        /// Ignored by PA servers <0.9.12
        PA_STREAM_EARLY_REQUESTS = 0x4000,
        /// Don't inhibit the connected device from auto-suspending
        /// Ignored by PA servers <0.9.15
        PA_STREAM_DONT_INHIBIT_AUTO_SUSPEND = 0x8000,
        /// Create in unmuted state.
        /// Ignored by PA servers <0.9.15
        PA_STREAM_START_UNMUTED = 0x10000,
        /// Fail to create streams on suspended devices, and terminate the
        /// stream if device suspend.
        /// Ignored by PA servers <0.9.15
        PA_STREAM_FAIL_ON_SUSPEND = 0x20000,
        /// Consider this stream's volume relative to the sink's current volume.
        /// Ignored by PA servers <0.9.20
        PA_STREAM_RELATIVE_VOLUME = 0x40000,
        /// The stream's data will be rendered by passthrough sinks, and not
        /// reformatted or resampled.
        /// Ignored by PA servers <1.0
        PA_STREAM_PASSTHROUGH = 0x80000
    }
}

pub mod structs {
    extern crate libc;
    use self::libc::{c_int, c_char, c_void, strlen};
    use std::{str, slice, mem};
    use super::types::*;
    use super::enums::*;
    use super::opaque::*;

        #[repr(C)]
    pub struct pa_sample_spec {
      pub format: pa_sample_format,
      pub rate: u32,
      pub channels: u8
    }

    #[repr(C)]
        pub struct pa_cvolume {
        channels: u8,
        values: [pa_volume_t; 32]
    }

    #[repr(C)]
        pub struct pa_channel_map {
        channels: u8,
        values: [pa_channel_position_t; 32]
    }

    #[repr(C)]
        pub struct pa_sink_info {
        pub name: *const c_char,               //**< Name of the sink */
        pub index: u32,                        //**< Index of the sink */
        pub description: *const c_char,  //**< Description of this sink */
        pub sample_spec: pa_sample_spec,        //**< Sample spec of this sink */
        pub channel_map: pa_channel_map,        //**< Channel map */
        pub owner_module: u32,             //**< Index of the owning module of this sink, or PA_INVALID_INDEX. */
        pub volume: pa_cvolume,                 //**< Volume of the sink */
        pub mute: c_int,                          //**< Mute switch of the sink */
        pub monitor_source: u32,          //**< Index of the monitor source connected to this sink. */
        pub monitor_source_name: *const c_char,   //**< The name of the monitor source. */
        pub latency: pa_usec_t,                 //**< Length of queued audio in the output buffer. */
        pub driver: *const c_char,                //**< Driver name */
        pub flags: pa_sink_flags_t,             //**< Flags */
        pub proplist: *mut pa_proplist,             //**< Property list \since 0.9.11 */
        pub configured_latency: pa_usec_t,      //**< The latency this device has been configured to. \since 0.9.11 */
        pub base_volume: pa_volume_t,           //**< Some kind of "base" volume that refers to unamplified/unattenuated volume in the context of the output device. \since 0.9.15 */
        pub state: pa_sink_state_t,             //**< State \since 0.9.15 */
        pub n_volume_steps: u32,           //**< Number of volume steps for sinks which do not support arbitrary volumes. \since 0.9.15 */
        pub card: u32,                     //**< Card index, or PA_INVALID_INDEX. \since 0.9.15 */
        pub n_ports: u32,                  //**< Number of entries in port array \since 0.9.16 */
        pub ports: *mut *mut c_void,
        //pa_sink_port_info** ports;         //**< Array of available ports, or NULL. Array is terminated by an entry set to NULL. The number of entries is stored in n_ports. \since 0.9.16 */
        pub active_port: *mut c_void,
        //pa_sink_port_info* active_port;    //**< Pointer to active port in the array, or NULL. \since 0.9.16 */
        pub n_formats: u8,
        pub formats: *mut *mut c_void          //**< Number of formats supported by the sink. \since 1.0 */
        //pa_format_info **formats;          //**< Array of formats supported by the sink. \since 1.0 */
    }

    #[repr(C)]
        pub struct pa_server_info {
        pub user_name: *const c_char,
        pub host_name: *const c_char,
        pub server_version: *const c_char,
        pub server_name: *const c_char,
        pub sample_spec: pa_sample_spec,
        pub default_sink_name: *const c_char,
        pub default_source_name: *const c_char,
        pub cookie: u32,
        pub channel_map: pa_channel_map
    }

    #[repr(C)]
        pub struct pa_format_info {
        pub encoding: pa_encoding_t,
        pub plist: *mut pa_proplist
    }

    #[repr(C)]
        pub struct pa_buffer_attr {
        pub max_length: u32,
        pub tlength: u32,
        pub prebuf: u32,
        pub minreq: u32,
        pub fragsize: u32
    }

    /// Impl for making it easy to get string values from pa_server_info
    impl<'a> pa_server_info {
        pub fn get_user_name(&'a self) -> &'a str {
            get_str(&self.user_name)
        }

        pub fn get_host_name(&'a self) -> &'a str {
            get_str(&self.host_name)
        }

        pub fn get_server_version(&'a self) -> &'a str {
            get_str(&self.server_version)
        }

        pub fn get_server_name(&'a self) -> &'a str {
            get_str(&self.server_name)
        }

        pub fn get_default_sink_name(&'a self) -> &'a str {
            get_str(&self.default_sink_name)
        }

        pub fn get_default_source_name(&'a self) -> &'a str {
            get_str(&self.default_source_name)
        }
    }

    impl<'a> pa_sink_info {
        pub fn get_name(&'a self) -> &'a str {
            get_str(&self.name)
        }

        pub fn get_description(&'a self) -> &'a str {
            get_str(&self.description)
        }

        pub fn get_monitor_source_name(&'a self) -> &'a str {
            get_str(&self.monitor_source_name)
        }

        pub fn get_driver(&'a self) -> &'a str {
            get_str(&self.driver)
        }
    }

    /// Turn a raw c pointer with a life time into an &str
    fn get_str<'a>(c_buf: &'a *const c_char) -> &'a str {
        let len = unsafe{ strlen(*c_buf) } as usize;
        let slice: &[c_char] = unsafe{ slice::from_raw_parts(*c_buf, len) };
        str::from_utf8(unsafe{ mem::transmute(slice) }).unwrap()
    }

}

/// For types that are just renamed.
pub mod types {
    pub type pa_volume_t = u32;
    pub type pa_usec_t = u64;
}
