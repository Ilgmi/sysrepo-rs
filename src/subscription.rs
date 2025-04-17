use crate::common::{dup_str, str_to_cstring};
use crate::enums::SrNotifType;
use crate::errors::SrError;
use crate::session::{SrEvent, SrSession};
use crate::values::SrValues;
use libyang3_sys::lyd_node;
use std::ffi::CStr;
use std::mem::{zeroed, ManuallyDrop};
use std::os::raw::{c_char, c_void};
use sysrepo_sys as ffi_sys;

use yang3::context::Context;
use yang3::data::DataTree;
use yang3::utils::Binding;

pub type SrSubscriptionId = *const ffi_sys::sr_subscription_ctx_t;

/// Sysrepo Subscription.
pub struct SrSubscription {
    /// Raw Pointer to subscription.
    raw_subscription: *mut ffi_sys::sr_subscription_ctx_t,
}

impl SrSubscription {
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
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        mod_name: *const c_char,
        path: *const c_char,
        event: ffi_sys::sr_event_t,
        request_id: u32,
        private_data: *mut c_void,
    ) -> i32
    where
        F: FnMut(
            SrSession,
            u32,
            &str,
            Option<&str>,
            SrEvent,
            u32,
        ) -> Result<(), SrError>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = if path.is_null() {
            None
        } else {
            Some(CStr::from_ptr(path).to_str().unwrap())
        };
        let event = SrEvent::try_from(event).expect("Convert error");
        let sess = SrSession::from(sess, false);

        let result = callback(sess, sub_id, mod_name, path, event, request_id);
        match result {
            Ok(_) => ffi_sys::sr_error_t_SR_ERR_OK as i32,
            Err(err) => err as i32,
        }
    }

    pub fn on_module_change<F>(
        session: &SrSession,
        module_name: &str,
        xpath: Option<&str>,
        module_change_cb: F,
        priority: u32,
        options: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(
            SrSession,
            u32,
            &str,
            Option<&str>,
            SrEvent,
            u32,
        ) -> Result<(), SrError>,
    {
        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            std::ptr::null_mut();
        let data = Box::into_raw(Box::new(module_change_cb));
        let module_name = dup_str(module_name)?;
        let xpath = match xpath {
            None => std::ptr::null_mut(),
            Some(path) => dup_str(path)?,
        };

        let rc = unsafe {
            ffi_sys::sr_module_change_subscribe(
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
    pub fn on_oper_get_subscribe<F>(
        session: &SrSession,
        module_name: &str,
        xpath: &str,
        callback: F,
        opts: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: for<'a> FnMut(
            &'a mut SrSession,
            &'a Context,
            u32,
            &'a str,
            &'a str,
            Option<&'a str>,
            u32,
            Option<DataTree<'a>>,
        ) -> Result<Option<DataTree<'a>>, SrError>,
    {
        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            unsafe { zeroed::<*mut ffi_sys::sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let module_name = dup_str(module_name)?;
        let path = dup_str(xpath)?;

        let rc = unsafe {
            ffi_sys::sr_oper_get_subscribe(
                session.get_raw_mut(),
                module_name,
                path,
                Some(Self::oper_get_subscribe_cb::<F>),
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

    unsafe extern "C" fn oper_get_subscribe_cb<F>(
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        module_name: *const c_char,
        path: *const c_char,
        request_xpath: *const c_char,
        request_id: u32,
        parent: *mut *mut lyd_node,
        private_data: *mut c_void,
    ) -> i32
    where
        F: for<'a> FnMut(
            &'a mut SrSession,
            &'a Context,
            u32,
            &'a str,
            &'a str,
            Option<&'a str>,
            u32,
            Option<DataTree<'a>>,
        ) -> Result<Option<DataTree<'a>>, SrError>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let module_name = CStr::from_ptr(module_name).to_str().unwrap();
        let xpath = CStr::from_ptr(path).to_str().unwrap();
        let request_xpath = if request_xpath.is_null() {
            None
        } else {
            Some(CStr::from_ptr(request_xpath).to_str().unwrap())
        };

        let mut session = SrSession::from(sess, false);
        let ctx = ManuallyDrop::new(session.get_context());

        let node_opt = if (*parent).is_null() {
            None
        } else {
            Some(DataTree::from_raw(&ctx, *parent))
        };

        let res = callback(
            &mut session,
            &ctx,
            sub_id,
            module_name,
            xpath,
            request_xpath,
            request_id,
            node_opt,
        );

        match res {
            Ok(node) => {
                match node {
                    None => *parent = std::ptr::null_mut(),
                    Some(node) => *parent = node.into_raw(),
                }
                SrError::Ok as i32
            }
            Err(error) => error as i32,
        }
    }
}

impl SrSubscription {
    unsafe extern "C" fn call_rpc_cb<F>(
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const ffi_sys::sr_val_t,
        input_cnt: usize,
        event: ffi_sys::sr_event_t,
        request_id: u32,
        output: *mut *mut ffi_sys::sr_val_t,
        output_cnt: *mut usize,
        private_data: *mut c_void,
    ) -> i32
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let op_path = CStr::from_ptr(op_path).to_str().unwrap();
        let inputs = SrValues::from_raw(
            input as *mut ffi_sys::sr_val_t,
            input_cnt,
            false,
        );
        let sess = SrSession::from(sess, false);
        let event = SrEvent::try_from(event).expect("Convert error");

        let sr_outputs =
            callback(sess, sub_id, op_path, inputs, event, request_id);
        let (raw, len) = sr_outputs.as_raw();
        *output = raw;
        *output_cnt = len;
        println!("output {:?}", output);

        SrError::Ok as i32
    }

    pub fn on_rpc_subscribe<F>(
        session: &SrSession,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        options: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues
            + 'static,
    {
        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };

        let xpath_ptr = xpath.as_ref().map_or(std::ptr::null(), |x| x.as_ptr());

        let rc = unsafe {
            ffi_sys::sr_rpc_subscribe(
                session.get_raw_mut(),
                xpath_ptr,
                Some(Self::call_rpc_cb::<F>),
                data as *mut _,
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

    unsafe extern "C" fn call_rpc_tree_cb<F>(
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const lyd_node,
        event: ffi_sys::sr_event_t,
        request_id: u32,
        output: *mut lyd_node,
        private_data: *mut c_void,
    ) -> i32
    where
        F: for<'a> FnMut(
            &'a mut SrSession,
            &'a Context,
            u32,
            &str,
            &DataTree<'a>,
            &mut DataTree<'a>,
            SrEvent,
            u32,
        ),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let op_path = CStr::from_ptr(op_path).to_str().unwrap();

        let mut sess = SrSession::from(sess, false);
        let ctx = sess.get_context();

        let inputs =
            ManuallyDrop::new(DataTree::from_raw(&ctx, input as *mut _));
        let mut output =
            ManuallyDrop::new(DataTree::from_raw(&ctx, output as *mut _));

        let event = SrEvent::try_from(event).expect("Convert error");

        callback(
            &mut sess,
            &ctx,
            sub_id,
            op_path,
            &inputs,
            &mut output,
            event,
            request_id,
        );

        SrError::Ok as i32
    }

    pub fn on_rpc_subscribe_tree<F>(
        session: &SrSession,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        options: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: for<'a> FnMut(
            &'a mut SrSession,
            &'a Context,
            u32,
            &str,
            &DataTree<'a>,
            &mut DataTree<'a>,
            SrEvent,
            u32,
        ),
    {
        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };

        let xpath_ptr = xpath.as_ref().map_or(std::ptr::null(), |x| x.as_ptr());

        let rc = unsafe {
            ffi_sys::sr_rpc_subscribe_tree(
                session.get_raw_mut(),
                xpath_ptr,
                Some(Self::call_rpc_tree_cb::<F>),
                data as *mut _,
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
    /// Subscribe event notification.
    pub fn on_notification_subscribe<F>(
        session: &SrSession,
        module_name: &str,
        xpath: Option<&str>,
        start_time: Option<*mut ffi_sys::timespec>,
        stop_time: Option<*mut ffi_sys::timespec>,
        callback: F,
        opts: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(
                SrSession,
                u32,
                SrNotifType,
                Option<&str>,
                SrValues,
                *mut ffi_sys::timespec,
            ) + 'static,
    {
        let mod_name = dup_str(module_name)?;
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };

        let xpath_ptr = xpath.as_ref().map_or(std::ptr::null(), |x| x.as_ptr());

        let start_time = start_time.unwrap_or(std::ptr::null_mut());
        let stop_time = stop_time.unwrap_or(std::ptr::null_mut());

        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let rc = unsafe {
            ffi_sys::sr_notif_subscribe(
                session.get_raw_mut(),
                mod_name,
                xpath_ptr,
                start_time,
                stop_time,
                Some(Self::call_event_notif_cb::<F>),
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

    pub fn on_notification_subscribe_tree<F>(
        session: &SrSession,
        module_name: &str,
        xpath: Option<&str>,
        start_time: Option<*mut ffi_sys::timespec>,
        stop_time: Option<*mut ffi_sys::timespec>,
        callback: F,
        opts: ffi_sys::sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(
            &SrSession,
            u32,
            SrNotifType,
            &DataTree<'_>,
            *mut ffi_sys::timespec,
        ),
    {
        let mod_name = dup_str(module_name)?;
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };

        let xpath_ptr = xpath.as_ref().map_or(std::ptr::null(), |x| x.as_ptr());

        let start_time = start_time.unwrap_or(std::ptr::null_mut());
        let stop_time = stop_time.unwrap_or(std::ptr::null_mut());

        let mut subscription_ctx: *mut ffi_sys::sr_subscription_ctx_t =
            std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let rc = unsafe {
            ffi_sys::sr_notif_subscribe_tree(
                session.get_raw_mut(),
                mod_name,
                xpath_ptr,
                start_time,
                stop_time,
                Some(Self::call_event_notif_tree_cb::<F>),
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

    unsafe extern "C" fn call_event_notif_tree_cb<F>(
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        notif_type: ffi_sys::sr_ev_notif_type_t,
        notif: *const lyd_node,
        timestamp: *mut ffi_sys::timespec,
        private_data: *mut std::os::raw::c_void,
    ) where
        F: FnMut(
            &SrSession,
            u32,
            SrNotifType,
            &DataTree<'_>,
            *mut ffi_sys::timespec,
        ),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let session = SrSession::from(sess, false);
        let ctx = session.get_context();
        let data_tree =
            ManuallyDrop::new(DataTree::from_raw(&ctx, notif as *mut _));

        let notif_type =
            SrNotifType::try_from(notif_type).map_err(|_| SrError::Internal);
        if let Ok(notif_type) = notif_type {
            callback(&session, sub_id, notif_type, &data_tree, timestamp);
        }
    }

    unsafe extern "C" fn call_event_notif_cb<F>(
        sess: *mut ffi_sys::sr_session_ctx_t,
        sub_id: u32,
        notif_type: ffi_sys::sr_ev_notif_type_t,
        path: *const c_char,
        values: *const ffi_sys::sr_val_t,
        values_cnt: usize,
        timestamp: *mut ffi_sys::timespec,
        private_data: *mut c_void,
    ) where
        F: FnMut(
            SrSession,
            u32,
            SrNotifType,
            Option<&str>,
            SrValues,
            *mut ffi_sys::timespec,
        ),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;
        let xpath = if path.is_null() {
            None
        } else {
            Some(CStr::from_ptr(path).to_str().unwrap())
        };
        let sr_values = SrValues::from_raw(
            values as *mut ffi_sys::sr_val_t,
            values_cnt,
            false,
        );
        let sess = SrSession::from(sess, false);
        let notif_type =
            SrNotifType::try_from(notif_type).map_err(|_| SrError::Internal);
        if let Ok(notif_type) = notif_type {
            callback(sess, sub_id, notif_type, xpath, sr_values, timestamp);
        }
    }
}

impl Drop for SrSubscription {
    fn drop(&mut self) {
        unsafe {
            ffi_sys::sr_unsubscribe(self.raw_subscription);
        }
    }
}
