use crate::enums::SrNotifType;
use crate::errors::SrError;
use crate::str_to_cstring;
use crate::subscription::{SrSubscription, SrSubscriptionId};
use crate::value::SrValue;
use crate::value_slice::SrValues;
use libyang3_sys::lyd_node;
use std::collections::HashMap;
use std::ffi::CStr;
use std::fmt;
use std::mem::zeroed;
use std::os::raw::{c_char, c_void};
use std::sync::Arc;
use std::time::Duration;
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{
    sr_acquire_context, sr_apply_changes, sr_change_iter_t, sr_change_oper_t,
    sr_change_oper_t_SR_OP_CREATED, sr_change_oper_t_SR_OP_DELETED,
    sr_change_oper_t_SR_OP_MODIFIED, sr_change_oper_t_SR_OP_MOVED, sr_data_t,
    sr_error_t_SR_ERR_CALLBACK_FAILED, sr_error_t_SR_ERR_OK, sr_ev_notif_type_t, sr_event_t,
    sr_event_t_SR_EV_ABORT, sr_event_t_SR_EV_CHANGE, sr_event_t_SR_EV_DONE,
    sr_event_t_SR_EV_ENABLED, sr_event_t_SR_EV_RPC, sr_event_t_SR_EV_UPDATE, sr_free_change_iter,
    sr_get_change_next, sr_get_changes_iter, sr_get_data, sr_get_items, sr_get_node,
    sr_module_change_subscribe, sr_notif_send_tree, sr_notif_subscribe, sr_oper_get_subscribe,
    sr_release_data, sr_rpc_send, sr_rpc_subscribe, sr_session_acquire_context, sr_session_ctx_t,
    sr_session_get_connection, sr_session_stop, sr_set_item_str, sr_subscr_options_t,
    sr_subscription_ctx_t, sr_val_t, timespec,
};
use yang3::context::Context;
use yang3::data::{Data, DataTree};
use yang3::iter::NodeIterable;
use yang3::utils::Binding;

/// Event.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrEvent {
    Update = sr_event_t_SR_EV_UPDATE as isize,
    Change = sr_event_t_SR_EV_CHANGE as isize,
    Done = sr_event_t_SR_EV_DONE as isize,
    Abort = sr_event_t_SR_EV_ABORT as isize,
    Enabled = sr_event_t_SR_EV_ENABLED as isize,
    Rpc = sr_event_t_SR_EV_RPC as isize,
}

impl TryFrom<u32> for SrEvent {
    type Error = &'static str;

    fn try_from(t: u32) -> Result<Self, Self::Error> {
        match t {
            sr_event_t_SR_EV_UPDATE => Ok(SrEvent::Update),
            sr_event_t_SR_EV_CHANGE => Ok(SrEvent::Change),
            sr_event_t_SR_EV_DONE => Ok(SrEvent::Done),
            sr_event_t_SR_EV_ABORT => Ok(SrEvent::Abort),
            sr_event_t_SR_EV_ENABLED => Ok(SrEvent::Enabled),
            sr_event_t_SR_EV_RPC => Ok(SrEvent::Rpc),
            _ => Err("Invalid SrEvent"),
        }
    }
}

impl fmt::Display for SrEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            SrEvent::Update => "Update",
            SrEvent::Change => "Change",
            SrEvent::Done => "Done",
            SrEvent::Abort => "Abort",
            SrEvent::Enabled => "Enabled",
            SrEvent::Rpc => "RPC",
        };
        write!(f, "{}", s)
    }
}

pub type SrSessionId = *const ffi_sys::sr_session_ctx_t;

pub struct SrSession {
    raw_session: *mut ffi_sys::sr_session_ctx_t,
    owned: bool,

    /// Map from raw pointer to subscription.
    subscriptions: HashMap<SrSubscriptionId, SrSubscription>,
}

impl SrSession {
    pub fn from(sess: *mut sr_session_ctx_t, owned: bool) -> Self {
        Self {
            raw_session: sess,
            owned,
            subscriptions: HashMap::new(),
        }
    }

    /// Create unowned clone.
    pub fn clone(&self) -> Self {
        Self {
            raw_session: self.raw_session,
            owned: false,
            subscriptions: HashMap::new(),
        }
    }

    /// Get raw session context.
    pub unsafe fn get_raw(&self) -> *mut sr_session_ctx_t {
        self.raw_session
    }

    /// Returns the libyang3 context associated with this Session
    pub fn get_context(&self) -> yang3::context::Context {
        let d =
            unsafe { sr_session_acquire_context(self.raw_session) } as *mut libyang3_sys::ly_ctx;
        let t = ();
        unsafe { yang3::context::Context::from_raw(&t, d) }
    }

    /// Insert subscription.
    pub fn insert_subscription(&mut self, subscription: SrSubscription) -> SrSubscriptionId {
        let id = subscription.id();
        self.subscriptions.insert(id, subscription);
        id
    }

    /// Remove subscription.
    pub fn remove_subscription(&mut self, subscription: &SrSubscription) {
        let id = subscription.id();
        self.subscriptions.remove(&id);
    }

    /// Get tree from given XPath.
    pub fn get_data<'a>(
        &mut self,
        context: &'a Arc<Context>,
        xpath: &str,
        max_depth: Option<u32>,
        timeout: Option<Duration>,
        opts: u32,
    ) -> Result<DataTree<'a>, SrError> {
        let xpath = str_to_cstring(xpath)?;
        let max_depth = max_depth.unwrap_or(0);
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        // SAFETY: data is used as output by sr_get_data and is not read
        let mut data: *mut sr_data_t = unsafe { zeroed::<*mut sr_data_t>() };

        let rc = unsafe {
            sr_get_data(
                self.raw_session,
                xpath.as_ptr(),
                max_depth,
                timeout_ms,
                opts,
                &mut data,
            )
        };

        if rc != SrError::Ok as i32 {
            return Err(SrError::from(rc));
        }

        if data.is_null() {
            return Err(SrError::NotFound);
        }

        let conn = unsafe { sr_session_get_connection(self.raw_session) };

        if unsafe { (*data).conn } != conn {
            // It should never happen that the returned connection does not match the supplied one
            // SAFETY: data was checked as not NULL just above
            unsafe {
                sr_release_data(data);
            }

            return Err(SrError::Internal);
        }

        Ok(unsafe { DataTree::from_raw(context, (*data).tree) })
    }

    /// Get node by xpath
    pub fn get_node<'a>(
        &mut self,
        context: &'a Arc<Context>,
        xpath: &str,
        timeout: Option<Duration>,
        opts: u32,
    ) -> Result<DataTree<'a>, SrError> {
        let xpath = str_to_cstring(xpath)?;
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        // SAFETY: data is used as output by sr_get_data and is not read
        let mut data: *mut *mut sr_data_t = unsafe { zeroed::<*mut *mut sr_data_t>() };

        let rc = unsafe { sr_get_node(self.raw_session, xpath.as_ptr(), timeout_ms, data) };

        if rc != SrError::Ok as i32 {
            return Err(SrError::from(rc));
        }

        if data.is_null() {
            return Err(SrError::NotFound);
        }

        let conn = unsafe { sr_session_get_connection(self.raw_session) };

        if unsafe { (*(*data)).conn } != conn {
            // It should never happen that the returned connection does not match the supplied one
            // SAFETY: data was checked as not NULL just above
            unsafe {
                sr_release_data(*data);
            }

            return Err(SrError::Internal);
        }

        Ok(unsafe { DataTree::from_raw(context, (*(*data)).tree) })
    }

    /// Get items from given Xpath, anre return result in Value slice.
    pub fn get_items(
        &mut self,
        xpath: &str,
        timeout: Option<Duration>,
        opts: u32,
    ) -> Result<SrValues, SrError> {
        let xpath = str_to_cstring(xpath)?;
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);
        let mut values_count: usize = 0;
        let mut values: *mut sr_val_t = unsafe { zeroed::<*mut sr_val_t>() };

        let rc = unsafe {
            sr_get_items(
                self.raw_session,
                xpath.as_ptr(),
                timeout_ms,
                opts,
                &mut values,
                &mut values_count,
            )
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(SrValues::from_raw(values, values_count, true))
        }
    }

    /// Set string item to given Xpath.
    pub fn set_item_str(
        &mut self,
        path: &str,
        value: &str,
        origin: Option<&str>,
        opts: u32,
    ) -> Result<(), SrError> {
        let path = str_to_cstring(path)?;
        let value = str_to_cstring(value)?;
        let origin = match origin {
            Some(orig) => Some(str_to_cstring(orig)?),
            None => None,
        };
        let origin_ptr = origin.map_or(std::ptr::null(), |orig| orig.as_ptr());

        let rc = unsafe {
            sr_set_item_str(
                self.raw_session,
                path.as_ptr(),
                value.as_ptr(),
                origin_ptr,
                opts,
            )
        };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(())
        }
    }

    /// Apply changes for the session.
    pub fn apply_changes(&mut self, timeout: Option<Duration>) -> Result<(), SrError> {
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        let rc = unsafe { sr_apply_changes(self.raw_session, timeout_ms) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(())
        }
    }

    /// Subscribe event notification.
    pub fn notif_subscribe<F>(
        &mut self,
        mod_name: &str,
        xpath: Option<String>,
        start_time: Option<*mut timespec>,
        stop_time: Option<*mut timespec>,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, SrNotifType, &str, SrValues, *mut timespec) + 'static,
    {
        let mod_name = str_to_cstring(mod_name)?;
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(&path)?),
            None => None,
        };
        let xpath_ptr = xpath.map_or(std::ptr::null(), |xpath| xpath.as_ptr());
        let start_time = start_time.unwrap_or(std::ptr::null_mut());
        let stop_time = stop_time.unwrap_or(std::ptr::null_mut());

        let mut subscr: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let rc = unsafe {
            sr_notif_subscribe(
                self.raw_session,
                mod_name.as_ptr(),
                xpath_ptr,
                start_time,
                stop_time,
                Some(SrSession::call_event_notif::<F>),
                data as *mut _,
                opts,
                &mut subscr,
            )
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let id = self.insert_subscription(SrSubscription::from(subscr));
            Ok(self.subscriptions.get_mut(&id).unwrap())
        }
    }

    unsafe extern "C" fn call_event_notif<F>(
        sess: *mut sr_session_ctx_t,
        sub_id: u32,
        notif_type: sr_ev_notif_type_t,
        path: *const c_char,
        values: *const sr_val_t,
        values_cnt: usize,
        timestamp: *mut timespec,
        private_data: *mut c_void,
    ) where
        F: FnMut(SrSession, u32, SrNotifType, &str, SrValues, *mut timespec),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let path = CStr::from_ptr(path).to_str().unwrap();
        let sr_values = SrValues::from_raw(values as *mut sr_val_t, values_cnt, false);
        let sess = SrSession::from(sess, false);
        let notif_type = SrNotifType::try_from(notif_type).expect("Convert error");

        callback(sess, sub_id, notif_type, path, sr_values, timestamp);
    }

    /// Subscribe RPC.
    pub fn rpc_subscribe<F>(
        &mut self,
        xpath: Option<String>,
        callback: F,
        priority: u32,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues + 'static,
    {
        let mut subscr: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));

        let rc = unsafe {
            match xpath {
                Some(xpath) => {
                    let xpath = str_to_cstring(&xpath)?;
                    sr_rpc_subscribe(
                        self.raw_session,
                        xpath.as_ptr(),
                        Some(SrSession::call_rpc::<F>),
                        data as *mut _,
                        priority,
                        opts,
                        &mut subscr,
                    )
                }
                None => sr_rpc_subscribe(
                    self.raw_session,
                    std::ptr::null_mut(),
                    Some(SrSession::call_rpc::<F>),
                    data as *mut _,
                    priority,
                    opts,
                    &mut subscr,
                ),
            }
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let id = self.insert_subscription(SrSubscription::from(subscr));
            Ok(self.subscriptions.get_mut(&id).unwrap())
        }
    }

    unsafe extern "C" fn call_rpc<F>(
        sess: *mut sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const sr_val_t,
        input_cnt: usize,
        event: sr_event_t,
        request_id: u32,
        output: *mut *mut sr_val_t,
        output_cnt: *mut usize,
        private_data: *mut c_void,
    ) -> i32
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let op_path = CStr::from_ptr(op_path).to_str().unwrap();
        let inputs = SrValues::from_raw(input as *mut sr_val_t, input_cnt, false);
        let sess = SrSession::from(sess, false);
        let event = SrEvent::try_from(event).expect("Convert error");

        let sr_output = callback(sess, sub_id, op_path, inputs, event, request_id);
        let (raw, len) = sr_output.as_raw();
        *output = raw;
        *output_cnt = len;

        sr_error_t_SR_ERR_OK as i32
    }

    pub fn oper_get_subscribe<F>(
        &mut self,
        mod_name: &str,
        path: &str,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
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
        let mut subscr: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(mod_name)?;
        let path = str_to_cstring(path)?;

        let rc = unsafe {
            sr_oper_get_subscribe(
                self.raw_session,
                mod_name.as_ptr(),
                path.as_ptr(),
                Some(SrSession::call_get_items::<F>),
                data as *mut _,
                opts,
                &mut subscr,
            )
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let id = self.insert_subscription(SrSubscription::from(subscr));
            Ok(self.subscriptions.get_mut(&id).unwrap())
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

    /// Subscribe module change.
    pub fn module_change_subscribe<F>(
        &mut self,
        mod_name: &str,
        path: Option<&str>,
        callback: F,
        priority: u32,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, &str, Option<&str>, SrEvent, u32) -> () + 'static,
    {
        let mut subscr: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(mod_name)?;
        let path = match path {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };
        let path_ptr = path.map_or(std::ptr::null(), |path| path.as_ptr());

        let rc = unsafe {
            sr_module_change_subscribe(
                self.raw_session,
                mod_name.as_ptr(),
                path_ptr,
                Some(SrSession::call_module_change::<F>),
                data as *mut _,
                priority,
                opts,
                &mut subscr,
            )
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let id = self.insert_subscription(SrSubscription::from(subscr));
            Ok(self.subscriptions.get_mut(&id).unwrap())
        }
    }

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
        F: FnMut(SrSession, u32, &str, Option<&str>, SrEvent, u32) -> (),
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

        callback(sess, sub_id, mod_name, path, event, request_id);

        sr_error_t_SR_ERR_OK as i32
    }

    /// Get changes iter.
    pub fn get_changes_iter(&self, path: &str) -> Result<SrChangeIterator, SrError> {
        let mut it = unsafe { zeroed::<*mut sr_change_iter_t>() };

        let path = str_to_cstring(path)?;
        let rc = unsafe { sr_get_changes_iter(self.raw_session, path.as_ptr(), &mut it) };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(SrChangeIterator::from(self, it))
        }
    }

    /// Send event notify tree.
    pub fn notif_send_tree(
        &mut self,
        notif: &DataTree,
        timeout_ms: u32,
        wait: i32,
    ) -> Result<(), SrError> {
        let rc = unsafe { sr_notif_send_tree(self.raw_session, notif.raw(), timeout_ms, wait) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(())
        }
    }

    /// Send RPC.
    pub fn rpc_send(
        &mut self,
        path: &str,
        input: Option<SrValues>,
        timeout: Option<Duration>,
    ) -> Result<SrValues, SrError> {
        let path = str_to_cstring(path)?;

        let (input, input_cnt) = match input {
            None => (std::ptr::null_mut(), 0),
            Some(input) => input.as_raw(),
        };

        let timeout = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        let mut output: *mut sr_val_t = unsafe { zeroed::<*mut sr_val_t>() };
        let mut output_count: usize = 0;

        let rc = unsafe {
            sr_rpc_send(
                self.raw_session,
                path.as_ptr(),
                input,
                input_cnt as usize,
                timeout,
                &mut output,
                &mut output_count,
            )
        };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(SrValues::from_raw(output, output_count, true))
        }
    }

    /// Return oper, old_value, new_value with next iter.
    pub fn get_change_next(
        &mut self,
        iter: &mut SrChangeIterator,
    ) -> Option<(SrChangeOper, SrValue, SrValue)> {
        let mut oper: sr_change_oper_t = 0;
        let mut old_value: *mut sr_val_t = std::ptr::null_mut();
        let mut new_value: *mut sr_val_t = std::ptr::null_mut();

        let rc = unsafe {
            sr_get_change_next(
                self.raw_session,
                iter.iter(),
                &mut oper,
                &mut old_value,
                &mut new_value,
            )
        };

        if rc == SrError::Ok as i32 {
            match SrChangeOper::try_from(oper) {
                Ok(oper) => Some((
                    oper,
                    SrValue::from(old_value, false),
                    SrValue::from(new_value, false),
                )),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

impl Drop for SrSession {
    fn drop(&mut self) {
        if self.owned {
            self.subscriptions.drain();

            unsafe {
                sr_session_stop(self.raw_session);
            }
        }
    }
}

/// Change Oper.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrChangeOper {
    Created = sr_change_oper_t_SR_OP_CREATED as isize,
    Modified = sr_change_oper_t_SR_OP_MODIFIED as isize,
    Deleted = sr_change_oper_t_SR_OP_DELETED as isize,
    Moved = sr_change_oper_t_SR_OP_MOVED as isize,
}

impl TryFrom<u32> for SrChangeOper {
    type Error = &'static str;

    fn try_from(t: u32) -> Result<Self, Self::Error> {
        match t {
            sr_change_oper_t_SR_OP_CREATED => Ok(SrChangeOper::Created),
            sr_change_oper_t_SR_OP_MODIFIED => Ok(SrChangeOper::Modified),
            sr_change_oper_t_SR_OP_DELETED => Ok(SrChangeOper::Deleted),
            sr_change_oper_t_SR_OP_MOVED => Ok(SrChangeOper::Moved),
            _ => Err("Invalid SrChangeOper"),
        }
    }
}

impl fmt::Display for SrChangeOper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            SrChangeOper::Created => "Created",
            SrChangeOper::Modified => "Modified",
            SrChangeOper::Deleted => "Deleted",
            SrChangeOper::Moved => "Moved",
        };
        write!(f, "{}", s)
    }
}

/// Sysrepo Changes Iterator.
pub struct SrChangeIterator<'a> {
    /// Raw pointer to iter.
    iter: *mut sr_change_iter_t,
    session: &'a SrSession,
}

impl<'a> SrChangeIterator<'a> {
    pub fn from(session: &'a SrSession, iter: *mut sr_change_iter_t) -> Self {
        Self { session, iter }
    }

    pub fn iter(&mut self) -> *mut sr_change_iter_t {
        self.iter
    }
}

pub struct CreatedOperation {
    pub value: SrValue,
}

pub struct ModifiedOperation {
    pub value: SrValue,
    pub prev_value: SrValue,
}

pub struct DeletedOperation {
    pub value: SrValue,
}

pub struct MovedOperation {
    pub value: SrValue,
}

pub enum SrChangeOperation {
    Created(CreatedOperation),
    Modified(ModifiedOperation),
    Deleted(DeletedOperation),
    Moved(MovedOperation),
}

impl Iterator for SrChangeIterator<'_> {
    type Item = SrChangeOperation;

    fn next(&mut self) -> Option<Self::Item> {
        let mut oper: sr_change_oper_t = 0;
        let mut old_value: *mut sr_val_t = std::ptr::null_mut();
        let mut new_value: *mut sr_val_t = std::ptr::null_mut();
        let rc = unsafe {
            sr_get_change_next(
                self.session.get_raw(),
                self.iter(),
                &mut oper,
                &mut old_value,
                &mut new_value,
            )
        };

        if rc == SrError::Ok as i32 {
            let new_value = SrValue::from(new_value, false);

            let op = match SrChangeOper::try_from(oper) {
                Ok(oper) => match oper {
                    SrChangeOper::Created => {
                        SrChangeOperation::Created(CreatedOperation { value: new_value })
                    }
                    SrChangeOper::Modified => {
                        let old_value = SrValue::from(old_value, false);
                        SrChangeOperation::Modified(ModifiedOperation {
                            value: new_value,
                            prev_value: old_value,
                        })
                    }
                    SrChangeOper::Deleted => {
                        SrChangeOperation::Deleted(DeletedOperation { value: new_value })
                    }
                    SrChangeOper::Moved => {
                        SrChangeOperation::Moved(MovedOperation { value: new_value })
                    }
                },
                Err(_) => return None,
            };
            Some(op)
        } else {
            None
        }
    }
}

impl Drop for SrChangeIterator<'_> {
    fn drop(&mut self) {
        unsafe {
            sr_free_change_iter(self.iter);
        }
    }
}
