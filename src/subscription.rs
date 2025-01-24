use crate::common::str_to_cstring;
use crate::connection::SrConnection;
use crate::errors::SrError;
use crate::session::{SrEvent, SrSession};
use libyang3_sys::lyd_node;
use std::ffi::CStr;
use std::mem::zeroed;
use std::os::raw::{c_char, c_void};
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{
    sr_acquire_context, sr_error_t_SR_ERR_CALLBACK_FAILED, sr_error_t_SR_ERR_OK, sr_event_t,
    sr_module_change_subscribe, sr_oper_get_subscribe, sr_session_ctx_t, sr_session_get_connection,
    sr_subscr_options_t, sr_subscription_ctx_t,
};
use yang3::data::{Data, DataTree};
use yang3::iter::NodeIterable;
use yang3::utils::Binding;

pub type SrSubscriptionId = *const ffi_sys::sr_subscription_ctx_t;

/// Sysrepo Subscription.
pub struct SrSubscription {
    /// Raw Pointer to subscription.
    raw_subscription: *mut ffi_sys::sr_subscription_ctx_t,
}

impl SrSubscription {
    pub fn new() -> Self {
        Self {
            raw_subscription: std::ptr::null_mut(),
        }
    }

    pub unsafe fn get_raw_mut(&self) -> *mut ffi_sys::sr_subscription_ctx_t {
        self.raw_subscription
    }

    pub unsafe fn get_raw(&self) -> *const ffi_sys::sr_subscription_ctx_t {
        self.raw_subscription as *const ffi_sys::sr_subscription_ctx_t
    }

    pub fn from(subscr: *mut ffi_sys::sr_subscription_ctx_t) -> Self {
        Self {
            raw_subscription: subscr,
        }
    }

    pub fn id(&self) -> SrSubscriptionId {
        self.raw_subscription
    }
}

impl SrSubscription {
    unsafe extern "C" fn call_module_change<F>(
        sess: *mut sr_session_ctx_t,
        sub_id: u32,
        mod_name: *const c_char,
        path: *const c_char,
        event: sr_event_t,
        request_id: u32,
        private_data: *mut c_void,
    ) -> i32
    where
        F: FnMut(SrSession, u32, &str, Option<&str>, SrEvent, u32) -> Result<(), SrError>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = if path == std::ptr::null_mut() {
            None
        } else {
            Some(CStr::from_ptr(path).to_str().unwrap())
        };
        let event = SrEvent::try_from(event).expect("Convert error");
        let sess = SrSession::from(sess, false);

        let result = callback(sess, sub_id, mod_name, path, event, request_id);
        match result {
            Ok(_) => sr_error_t_SR_ERR_OK as i32,
            Err(err) => err as i32,
        }
    }

    pub fn on_module_change<F>(
        session: &SrSession,
        module_name: &str,
        xpath: Option<&str>,
        module_change_cb: F,
        priority: u32,
        options: sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(SrSession, u32, &str, Option<&str>, SrEvent, u32) -> Result<(), SrError>,
    {
        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t = std::ptr::null_mut();
        let data = Box::into_raw(Box::new(module_change_cb));
        let module_name = str_to_cstring(module_name)?;
        let module_name = unsafe { libc::strdup(module_name.as_ptr()) };
        let xpath = match xpath {
            None => std::ptr::null_mut(),
            Some(path) => unsafe { libc::strdup(str_to_cstring(path)?.as_ptr()) },
        };

        let rc = unsafe {
            sr_module_change_subscribe(
                session.get_raw_mut(),
                module_name,
                xpath,
                Some(Self::call_module_change::<F>),
                data as *mut c_void,
                priority,
                options,
                &mut subscription_ctx,
            )
        };

        match rc {
            0 => Ok(Self {
                raw_subscription: subscription_ctx,
            }),
            rc => Err(SrError::from(rc)),
        }
    }
}

impl SrSubscription {
    pub fn oper_get_subscribe<F>(
        session: &SrSession,
        module_name: &str,
        xpath: &str,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: for<'a> FnMut(
            &'a yang3::context::Context,
            u32,
            &'a str,
            &'a str,
            Option<&'a str>,
            u32,
            &'a DataTree<'a>,
        ) -> Option<yang3::data::DataTree<'a>>,
    {
        let mut subscription_ctx: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(module_name)?;
        let path = str_to_cstring(xpath)?;

        let rc = unsafe {
            sr_oper_get_subscribe(
                session.get_raw_mut(),
                mod_name.as_ptr(),
                path.as_ptr(),
                Some(Self::call_get_items::<F>),
                data as *mut _,
                opts,
                &mut subscription_ctx,
            )
        };

        match rc {
            0 => Ok(Self {
                raw_subscription: subscription_ctx,
            }),
            rc => Err(SrError::from(rc)),
        }
    }

    unsafe extern "C" fn call_get_items<F>(
        sess: *mut sr_session_ctx_t,
        sub_id: u32,
        mod_name: *const c_char,
        path: *const c_char,
        request_xpath: *const c_char,
        request_id: u32,
        parent: *mut *mut lyd_node,
        private_data: *mut c_void,
    ) -> i32
    where
        F: for<'a> FnMut(
            &'a yang3::context::Context,
            u32,
            &'a str,
            &'a str,
            Option<&'a str>,
            u32,
            &'a DataTree<'a>,
        ) -> Option<yang3::data::DataTree<'a>>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let ctx = sr_acquire_context(sr_session_get_connection(sess)) as *mut libyang3_sys::ly_ctx;

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = CStr::from_ptr(path).to_str().unwrap();
        let request_xpath = if request_xpath == std::ptr::null_mut() {
            None
        } else {
            Some(CStr::from_ptr(request_xpath).to_str().unwrap())
        };

        let ctx = yang3::context::Context::from_raw(&(), ctx);
        let p = unsafe { DataTree::from_raw(&ctx, *parent) };
        let node = callback(&ctx, sub_id, mod_name, path, request_xpath, request_id, &p);

        match node {
            Some(node) => match node.reference() {
                None => {
                    return sr_error_t_SR_ERR_CALLBACK_FAILED as i32;
                }
                Some(r) => match r.parent() {
                    None => {
                        return sr_error_t_SR_ERR_CALLBACK_FAILED as i32;
                    }
                    Some(p) => {
                        *parent = p.raw();
                    }
                },
            },
            None => {}
        }

        sr_error_t_SR_ERR_OK as i32
    }
}

impl Drop for SrSubscription {
    fn drop(&mut self) {
        unsafe {
            ffi_sys::sr_unsubscribe(self.raw_subscription);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::ConnectionOptions;
    use crate::enums::{SrDatastore, SrLogLevel};
    use crate::log_stderr;
    use std::ops::{AddAssign, DerefMut};
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_call_module_change() {
        log_stderr(SrLogLevel::Info);

        let mut connection = SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
        let mut session = connection.start_session(SrDatastore::Running).unwrap();
        let check = Arc::new(Mutex::new(0));
        let change_cb_value = check.clone();
        let t = |session: SrSession,
                 sub_id: u32,
                 module_name: &str,
                 xpath: Option<&str>,
                 event: SrEvent,
                 request_id: u32|
         -> Result<(), SrError> {
            change_cb_value.lock().unwrap().deref_mut().add_assign(1);
            Ok(())
        };

        let _res = session.set_item_str("/examples:cont/l", "123", None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        let sub_id =
            session.on_module_change_subscribe("examples", Some("/examples:cont/l"), t, 0, 0);
        assert!(sub_id.is_ok());

        // TODO: Update Value
        let _res = session.set_item_str("/examples:cont/l", "321", None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        // Change is called 2 times
        assert_eq!(*check.lock().unwrap(), 2);
    }

    #[test]
    fn test_call_module_change() {
        log_stderr(SrLogLevel::Info);

        let mut connection = SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
        let mut session = connection.start_session(SrDatastore::Running).unwrap();
        let check = Arc::new(Mutex::new(0));
        let change_cb_value = check.clone();
        let t = |session: SrSession,
                 sub_id: u32,
                 module_name: &str,
                 xpath: Option<&str>,
                 event: SrEvent,
                 request_id: u32|
         -> Result<(), SrError> {
            change_cb_value.lock().unwrap().deref_mut().add_assign(1);
            Ok(())
        };

        let _res = session.set_item_str("/examples:cont/l", "123", None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        let sub_id = session.on_module_change_subscribe("examples", None, t, 0, 0);
        assert!(sub_id.is_ok());

        // TODO: Update Value
        let _res = session.set_item_str("/examples:cont/l", "321", None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        // Change is called 2 times
        assert_eq!(*check.lock().unwrap(), 2);
    }
}
