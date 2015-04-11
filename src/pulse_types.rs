#![allow(unstable)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(raw_pointer_derive)]


pub use self::cb::*;
pub use self::opaque::*;
pub use self::enums::*;
pub use self::structs::*;
pub use self::types::*;


/// For callback signatures
pub mod cb {
    extern crate libc;
    use self::libc::{c_int, c_void};
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
        nbytes: c_int,
        *mut c_void
    );
}


/// For types we only have pointers to. Use structs with names so there is at
/// least pointer type safety.
/// NOTE: Don't #[derive(Copy)] for these, to enforce that it gives an error
///       if an attempt is made to dereference one of these.
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
    #[derive(Copy)]
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
    #[derive(Copy)]
    pub enum pa_sink_state_t {
        PA_SINK_INVALID_STATE = -1,
        PA_SINK_RUNNING = 0,
        PA_SINK_IDLE = 1,
        PA_SINK_SUSPENDED = 2,
        PA_SINK_INIT = -2,
        PA_SINK_UNLINKED = -3
    }

    #[repr(C)]
    #[derive(Copy)]
    pub enum pa_sink_flags_t {
        PA_SINK_NOFLAGS = 0x0000is,
        PA_SINK_HW_VOLUME_CTRL = 0x0001is,
        PA_SINK_LATENCY = 0x0002is,
        PA_SINK_HARDWARE = 0x0004is,
        PA_SINK_NETWORK = 0x0008is,
        PA_SINK_HW_MUTE_CTRL = 0x0010is,
        PA_SINK_DECIBEL_VOLUME = 0x0020is,
        PA_SINK_FLAT_VOLUME = 0x0040is,
        PA_SINK_DYNAMIC_LATENCY = 0x0080is
    }


    #[repr(C)]
    #[derive(Copy)]
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
}

pub mod structs {
    extern crate libc;
    use self::libc::{c_int, c_char, c_void, strlen};
    use std::{str, slice, mem};
    use super::types::*;
    use super::enums::*;
    use super::opaque::*;

    #[derive(Copy)]
    #[repr(C)]
    pub struct pa_sample_spec {
      format: c_int,
      rate: u32,
      channels: u8
    }

    #[repr(C)]
    #[derive(Copy)]
    pub struct pa_cvolume {
        channels: u8,
        values: [pa_volume_t; 32]
    }

    #[repr(C)]
    #[derive(Copy)]
    pub struct pa_channel_map {
        channels: u8,
        values: [pa_channel_position_t; 32]
    }

    #[repr(C)]
    #[derive(Copy)]
    pub struct pa_sink_info<'a> {
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
    #[derive(Copy)]
    pub struct pa_server_info<'a> {
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
    #[derive(Copy)]
    pub struct pa_format_info {
        pub encoding: pa_encoding_t,
        pub plist: *mut pa_proplist
    }

    /// Impl for making it easy to get string values from pa_server_info
    impl<'a> pa_server_info<'a> {
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

    impl<'a> pa_sink_info<'a> {
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
        let slice: &[c_char] = unsafe{ slice::from_raw_buf(c_buf, len) };
        str::from_utf8(unsafe{ mem::transmute(slice) }).unwrap()
    }

}

/// For types that are just renamed.
pub mod types {
    pub type pa_volume_t = u32;
    pub type pa_usec_t = u64;
}
