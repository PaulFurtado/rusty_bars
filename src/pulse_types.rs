#![allow(unstable)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(raw_pointer_derive)]


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
        eol: c_int, userdata:
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
        PA_SINK_DYNAMIC_LATENCY = 0x0080is,
        PA_SINK_SET_FORMATS = 0x0100is
    }
}

pub mod structs {
    extern crate libc;
    use self::libc::{c_int, c_char, c_void};
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
        values: pa_volume_t
    }

    #[repr(C)]
    #[derive(Copy)]
    pub struct pa_channel_map {
        channels: u8,
        values: pa_channel_position_t
    }

    #[repr(C)]
    #[derive(Copy)]
    pub struct pa_sink_info {
        name: *const c_char,               //**< Name of the sink */
        index: u32,                        //**< Index of the sink */
        description: *const c_char,  //**< Description of this sink */
        sample_spec: pa_sample_spec,        //**< Sample spec of this sink */
        channel_map: pa_channel_map,        //**< Channel map */
        owner_module: u32,             //**< Index of the owning module of this sink, or PA_INVALID_INDEX. */
        volume: pa_cvolume,                 //**< Volume of the sink */
        mute: c_int,                          //**< Mute switch of the sink */
        monitor_source: u32,          //**< Index of the monitor source connected to this sink. */
        monitor_source_name: *const c_char,   //**< The name of the monitor source. */
        latency: pa_usec_t,                 //**< Length of queued audio in the output buffer. */
        driver: *const c_char,                //**< Driver name */
        flags: pa_sink_flags_t,             //**< Flags */
        proplist: *mut pa_proplist,             //**< Property list \since 0.9.11 */
        configured_latency: pa_usec_t,      //**< The latency this device has been configured to. \since 0.9.11 */
        base_volume: pa_volume_t,           //**< Some kind of "base" volume that refers to unamplified/unattenuated volume in the context of the output device. \since 0.9.15 */
        state: pa_sink_state_t,             //**< State \since 0.9.15 */
        n_volume_steps: u32,           //**< Number of volume steps for sinks which do not support arbitrary volumes. \since 0.9.15 */
        card: u32,                     //**< Card index, or PA_INVALID_INDEX. \since 0.9.15 */
        n_ports: u32,                  //**< Number of entries in port array \since 0.9.16 */
        ports: *mut c_void,
        //pa_sink_port_info** ports;         //**< Array of available ports, or NULL. Array is terminated by an entry set to NULL. The number of entries is stored in n_ports. \since 0.9.16 */
        active_port: *mut c_void,
        //pa_sink_port_info* active_port;    //**< Pointer to active port in the array, or NULL. \since 0.9.16 */
        n_formats: u8,
        formats: *mut c_void          //**< Number of formats supported by the sink. \since 1.0 */
        //pa_format_info **formats;          //**< Array of formats supported by the sink. \since 1.0 */
    }
}

/// For types that are just renamed.
pub mod types {
    pub type pa_volume_t = u32;
    pub type pa_usec_t = u64;
}
