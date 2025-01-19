use sysrepo_sys::{sr_subscription_ctx_t, sr_unsubscribe};

pub type SrSubscriptionId = *const sr_subscription_ctx_t;

/// Sysrepo Subscription.
pub struct SrSubscription {
    /// Raw Pointer to subscription.
    raw_subscription: *mut sr_subscription_ctx_t,
}

impl SrSubscription {
    pub fn new() -> Self {
        Self {
            raw_subscription: std::ptr::null_mut(),
        }
    }

    pub fn from(subscr: *mut sr_subscription_ctx_t) -> Self {
        Self {
            raw_subscription: subscr,
        }
    }

    pub fn id(&self) -> SrSubscriptionId {
        self.raw_subscription
    }
}

impl Drop for SrSubscription {
    fn drop(&mut self) {
        unsafe {
            sr_unsubscribe(self.raw_subscription);
        }
    }
}
