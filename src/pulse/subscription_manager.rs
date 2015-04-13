#![allow(unstable)]

/// A module for subscribing to events on a PulseAudio server.

extern crate libc;

use self::libc::funcs::c95::string::strlen;
use self::libc::{c_int, c_char, c_void};

use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

use pulse::ext;
use pulse::mainloop::PulseAudioMainloop;
use pulse::stream::PulseAudioStream;
use pulse::types::*;



/// Represents subscribed events on a Context.
/// An event type needs a subscription before its callback is triggered.
pub struct SubscriptionManager {
    mask: c_int,
}


impl SubscriptionManager {
    /// Create a new SubscriptionManager
    pub fn new() -> SubscriptionManager {
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
