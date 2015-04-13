#![allow(unstable)]

/// A module for subscribing to events on a PulseAudio server.

extern crate libc;

use self::libc::c_int;
use pulse::types::pa_subscription_mask;


/// Helper for managing the subscription mask for subscribed events.
/// PulseAudio uses the pa_subscription_mask enum to store each subscription
/// event type. This struct helps manage the combined mask of subscriptions.
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
