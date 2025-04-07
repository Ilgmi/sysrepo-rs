use crate::common::dup_str;
use crate::enums::{DefaultOperation, SrEditFlag, SrNotifType};
use crate::errors::SrError;
use crate::str_to_cstring;
use crate::subscription::{SrSubscription, SrSubscriptionId};
use crate::value::SrValue;
use crate::values::SrValues;
use libc::c_int;
use libyang3_sys::lyd_node;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::mem::{zeroed, ManuallyDrop};
use std::ops::Deref;
use std::os::raw::c_char;
use std::time::Duration;
use std::{fmt, ptr};
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{
    sr_acquire_context, sr_apply_changes, sr_change_iter_t, sr_change_oper_t,
    sr_change_oper_t_SR_OP_CREATED, sr_change_oper_t_SR_OP_DELETED,
    sr_change_oper_t_SR_OP_MODIFIED, sr_change_oper_t_SR_OP_MOVED, sr_data_t, sr_delete_item,
    sr_edit_batch, sr_event_t_SR_EV_ABORT, sr_event_t_SR_EV_CHANGE, sr_event_t_SR_EV_DONE,
    sr_event_t_SR_EV_ENABLED, sr_event_t_SR_EV_RPC, sr_event_t_SR_EV_UPDATE, sr_free_change_iter,
    sr_get_change_next, sr_get_change_tree_next, sr_get_changes_iter, sr_get_data, sr_get_items,
    sr_get_node, sr_notif_send, sr_notif_send_tree, sr_release_data, sr_replace_config,
    sr_rpc_send, sr_rpc_send_tree, sr_session_ctx_t, sr_session_get_connection, sr_session_stop,
    sr_set_item_str, sr_subscr_options_t, sr_val_t, timespec,
};
use yang3::context::Context;
use yang3::data::{Data, DataTree};
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
    pub unsafe fn get_raw_mut(&self) -> *mut sr_session_ctx_t {
        self.raw_session
    }

    pub unsafe fn get_raw(&self) -> *const sr_session_ctx_t {
        self.raw_session
    }

    /// Returns the libyang3 context associated with this Session
    pub fn get_context(&self) -> ManuallyDrop<Context> {
        let ctx = unsafe {
            let ctx = sr_acquire_context(sr_session_get_connection(self.raw_session))
                as *mut libyang3_sys::ly_ctx;
            Context::from_raw(&(), ctx)
        };
        ManuallyDrop::new(ctx)
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
        context: &'a Context,
        xpath: &str,
        max_depth: Option<u32>,
        timeout: Option<Duration>,
        opts: u32,
    ) -> Result<DataTree<'a>, SrError> {
        let xpath = dup_str(xpath)?;
        let max_depth = max_depth.unwrap_or(0);
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        // SAFETY: data is used as output by sr_get_data and is not read
        let mut data: *mut sr_data_t = std::ptr::null_mut();

        let rc = unsafe {
            sr_get_data(
                self.raw_session,
                xpath,
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

        Ok(unsafe { DataTree::from_raw(&context, (*data).tree) })
    }

    /// Get node by xpath
    pub fn get_node<'a>(
        &mut self,
        context: &'a Context,
        xpath: &str,
        timeout: Option<Duration>,
        _opts: u32,
    ) -> Result<DataTree<'a>, SrError> {
        let xpath = dup_str(xpath)?;
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        // SAFETY: data is used as output by sr_get_data and is not read
        let mut data: *mut sr_data_t = ptr::null_mut();

        let rc = unsafe { sr_get_node(self.raw_session, xpath, timeout_ms, &mut data) };

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

    pub fn edit_batch<'a>(
        &mut self,
        node: &DataTree<'a>,
        oper: DefaultOperation,
    ) -> Result<(), SrError> {
        let oper = dup_str(oper.as_str())?;
        let ret = unsafe { sr_edit_batch(self.raw_session, node.raw(), oper) };

        if ret != SrError::Ok as i32 {
            return Err(SrError::from(ret));
        }

        Ok(())
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

    pub fn remove_item(&mut self, path: &str, option: SrEditFlag) -> Result<(), SrError> {
        let path = CString::new(path).map_err(|_| SrError::Internal)?;
        let ret = unsafe {
            sr_delete_item(
                self.raw_session,
                path.as_ptr(),
                option as sysrepo_sys::sr_edit_options_t,
            )
        };

        if ret != SrError::Ok as i32 {
            Err(SrError::Internal)
        } else {
            Ok(())
        }
    }

    pub fn replace_config<'a>(
        &mut self,
        node: &DataTree<'a>,
        module: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<(), SrError> {
        let module = match module {
            None => std::ptr::null(),
            Some(module) => dup_str(module)?,
        };
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);
        let ret = unsafe { sr_replace_config(self.raw_session, module, node.raw(), timeout_ms) };

        if ret != SrError::Ok as i32 {
            return Err(SrError::from(ret));
        }

        Ok(())
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
    pub fn on_notif_subscribe<F>(
        &mut self,
        module_name: &str,
        xpath: Option<&str>,
        start_time: Option<*mut timespec>,
        stop_time: Option<*mut timespec>,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, SrNotifType, Option<&str>, SrValues, *mut timespec) + 'static,
    {
        let sub = SrSubscription::on_notification_subscribe(
            self,
            module_name,
            xpath,
            start_time,
            stop_time,
            callback,
            opts,
        )?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
    }

    pub fn on_notif_subscribe_tree<F>(
        &mut self,
        module_name: &str,
        xpath: Option<&str>,
        start_time: Option<*mut timespec>,
        stop_time: Option<*mut timespec>,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(&SrSession, u32, SrNotifType, &DataTree, *mut timespec),
    {
        let sub = SrSubscription::on_notification_subscribe_tree(
            self,
            module_name,
            xpath,
            start_time,
            stop_time,
            callback,
            opts,
        )?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
    }

    /// Subscribe RPC.
    pub fn on_rpc_subscribe<F>(
        &mut self,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues + 'static,
    {
        let sub = SrSubscription::on_rpc_subscribe(self, xpath, callback, priority, opts)?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
    }

    pub fn on_rpc_subscribe_tree<F>(
        &mut self,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
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
        let sub = SrSubscription::on_rpc_subscribe_tree(self, xpath, callback, priority, opts)?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
    }

    pub fn on_oper_get_subscribe<F>(
        &mut self,
        module_name: &str,
        xpath: &str,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
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
        let sub = SrSubscription::on_oper_get_subscribe(self, module_name, xpath, callback, opts)?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
    }

    /// Subscribe module change.
    pub fn on_module_change_subscribe<F>(
        &mut self,
        mod_name: &str,
        path: Option<&str>,
        callback: F,
        priority: u32,
        opts: sr_subscr_options_t,
    ) -> Result<&mut SrSubscription, SrError>
    where
        F: FnMut(SrSession, u32, &str, Option<&str>, SrEvent, u32) -> Result<(), SrError>,
    {
        let sub = SrSubscription::on_module_change(self, mod_name, path, callback, priority, opts)?;
        let id = self.insert_subscription(sub);
        Ok(self.subscriptions.get_mut(&id).unwrap())
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

    pub fn get_changes_iter_tree(&self, path: &str) -> Result<SrChangeIteratorTree, SrError> {
        let mut it = unsafe { zeroed::<*mut sr_change_iter_t>() };
        let path = str_to_cstring(path)?;
        let rc = unsafe { sr_get_changes_iter(self.raw_session, path.as_ptr(), &mut it) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(SrChangeIteratorTree::from(self, it))
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

    /// Send event notify.
    pub fn notif_send(
        &mut self,
        xpath: &str,
        values: &SrValues,
        timeout_ms: u32,
        wait: i32,
    ) -> Result<(), SrError> {
        let xpath = dup_str(xpath)?;
        let (values, len) = values.as_raw();
        let rc = unsafe { sr_notif_send(self.raw_session, xpath, values, len, timeout_ms, wait) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(())
        }
    }

    /// Send RPC.
    pub fn rpc_send(
        &mut self,
        xpath: &str,
        input: Option<SrValues>,
        timeout: Option<Duration>,
    ) -> Result<SrValues, SrError> {
        let xpath = dup_str(xpath)?;

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
                xpath,
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
            Ok(SrValues::from_raw(output, output_count, false))
        }
    }

    /// Send RPC Tree
    pub fn rpc_send_tree<'a>(
        &mut self,
        ctx: &'a Context,
        input: Option<DataTree<'a>>,
        timeout: Option<Duration>,
    ) -> Result<DataTree<'a>, SrError> {
        let input = match input {
            None => std::ptr::null_mut(),
            Some(input) => input.into_raw(),
        };

        let timeout = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        let mut output: *mut sr_data_t = std::ptr::null_mut();

        let rc = unsafe { sr_rpc_send_tree(self.raw_session, input, timeout, &mut output) };

        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let output = unsafe {
                let output = (*output).tree;
                DataTree::from_raw(&ctx, output)
            };

            Ok(output)
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

    pub fn get_key_value(
        &self,
        xpath: &str,
        node_name: &str,
        key_name: &str,
    ) -> Result<String, SrError> {
        let xpath = CString::new(xpath).unwrap();
        let node_name = CString::new(node_name).unwrap();
        let key_name = CString::new(key_name).unwrap();

        let mut ctx: sysrepo_sys::sr_xpath_ctx_s = unsafe {
            sysrepo_sys::sr_xpath_ctx_s {
                begining: std::ptr::null_mut(),
                current_node: std::ptr::null_mut(),
                replaced_position: std::ptr::null_mut(),
                replaced_char: 0,
            }
        };

        let ret = unsafe {
            sysrepo_sys::sr_xpath_key_value(
                xpath.as_ptr() as _,
                node_name.as_ptr(),
                key_name.as_ptr(),
                &mut ctx,
            )
        };
        if ret.is_null() {
            return Err(SrError::NotFound);
        }

        unsafe { Ok(CStr::from_ptr(ret).to_str().unwrap().to_string()) }
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

pub struct OperationData {
    pub value: SrValue,
    pub prev_value: Option<SrValue>,
}

impl OperationData {
    pub fn new(value: SrValue, prev_value: Option<SrValue>) -> Self {
        Self { value, prev_value }
    }

    pub fn without_prev_value(value: SrValue) -> Self {
        Self {
            value,
            prev_value: None,
        }
    }
}

pub enum SrChangeOperation {
    Created(OperationData),
    Modified(OperationData),
    Deleted(OperationData),
    Moved(OperationData),
}

impl Iterator for SrChangeIterator<'_> {
    type Item = SrChangeOperation;

    fn next(&mut self) -> Option<Self::Item> {
        let mut oper: sr_change_oper_t = 0;
        let mut old_value: *mut sr_val_t = std::ptr::null_mut();
        let mut new_value: *mut sr_val_t = std::ptr::null_mut();
        let rc = unsafe {
            sr_get_change_next(
                self.session.get_raw_mut(),
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
                        SrChangeOperation::Created(OperationData::without_prev_value(new_value))
                    }
                    SrChangeOper::Modified => {
                        let old_value = SrValue::from(old_value, false);
                        SrChangeOperation::Modified(OperationData::new(new_value, Some(old_value)))
                    }
                    SrChangeOper::Deleted => {
                        SrChangeOperation::Deleted(OperationData::without_prev_value(new_value))
                    }
                    SrChangeOper::Moved => {
                        let old_value = SrValue::from(old_value, false);
                        SrChangeOperation::Moved(OperationData::new(new_value, Some(old_value)))
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

pub struct SrChangeIteratorTree<'a> {
    /// Raw pointer to iter.
    iter: *mut sr_change_iter_t,
    session: &'a SrSession,
    _ctx: ManuallyDrop<Context>,
}

impl<'a> SrChangeIteratorTree<'a> {
    pub fn from(session: &'a SrSession, iter: *mut sr_change_iter_t) -> Self {
        let _ctx = session.get_context();
        Self {
            session,
            iter,
            _ctx,
        }
    }

    pub fn iter(&mut self) -> *mut sr_change_iter_t {
        self.iter
    }
}

pub struct OperationDataTree {
    pub node: *const lyd_node,
    pub prev_value: Option<String>,
    pub prev_list: Option<String>,
    pub prev_default_value: bool,
}

pub enum SrChangeOperationTree {
    Created(OperationDataTree),
    Modified(OperationDataTree),
    Deleted(OperationDataTree),
    Moved(OperationDataTree),
}

impl<'a> SrChangeOperationTree {
    pub fn from(
        op: SrChangeOper,
        node: *const lyd_node,
        prev_value: Option<String>,
        prev_list: Option<String>,
        prev_dflt: bool,
    ) -> Self {
        let operation_data = OperationDataTree {
            node,
            prev_value,
            prev_list,
            prev_default_value: prev_dflt,
        };
        match op {
            SrChangeOper::Created => SrChangeOperationTree::Created(operation_data),
            SrChangeOper::Modified => SrChangeOperationTree::Modified(operation_data),
            SrChangeOper::Deleted => SrChangeOperationTree::Deleted(operation_data),
            SrChangeOper::Moved => SrChangeOperationTree::Moved(operation_data),
        }
    }
}

impl<'a> Iterator for SrChangeIteratorTree<'a> {
    type Item = SrChangeOperationTree;

    fn next(&mut self) -> Option<Self::Item> {
        let mut oper: sr_change_oper_t = 0;
        let mut node: *const lyd_node = std::ptr::null_mut();
        let mut prev_value: *const c_char = std::ptr::null_mut();
        let mut prev_list: *const c_char = std::ptr::null_mut();
        let mut prev_default_value: c_int = 0;
        let rc = unsafe {
            sr_get_change_tree_next(
                self.session.get_raw_mut(),
                self.iter(),
                &mut oper,
                &mut node,
                &mut prev_value,
                &mut prev_list,
                &mut prev_default_value,
            )
        };

        if rc == SrError::NotFound as _ {
            return None;
        }

        let oper = match SrChangeOper::try_from(oper) {
            Ok(oper) => oper,
            Err(_) => return None,
        };

        let prev_value = match prev_value.is_null() {
            true => None,
            false => unsafe { Some(CStr::from_ptr(prev_value).to_string_lossy().into_owned()) },
        };
        let prev_list = match prev_list.is_null() {
            true => None,
            false => Some(unsafe { CStr::from_ptr(prev_list).to_string_lossy().into_owned() }),
        };

        let prev_default_value = if prev_default_value > 0 { true } else { false };

        Some(SrChangeOperationTree::from(
            oper,
            node,
            prev_value,
            prev_list,
            prev_default_value,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::{ConnectionOptions, SrConnection};
    use crate::enums::SrDatastore;

    #[test]
    fn get_session_successful() {
        let mut connection = SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("Failed to create connection");
        let session = connection.start_session(SrDatastore::Running);

        assert!(session.is_ok());
    }
}
