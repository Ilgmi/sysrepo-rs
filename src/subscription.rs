use crate::common::dup_str;
use crate::connection::SrConnection;
use crate::errors::SrError;
use crate::session::{SrEvent, SrSession};
use crate::value_slice::SrValues;
use libyang3_sys::lyd_node;
use std::ffi::CStr;
use std::mem::{zeroed, ManuallyDrop};
use std::os::raw::{c_char, c_void};
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{
    sr_error_t_SR_ERR_OK, sr_event_t, sr_module_change_subscribe, sr_oper_get_subscribe,
    sr_rpc_subscribe, sr_rpc_subscribe_tree, sr_session_ctx_t, sr_subscr_options_t,
    sr_subscription_ctx_t, sr_val_t,
};
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
        let module_name = dup_str(module_name)?;
        let xpath = match xpath {
            None => std::ptr::null_mut(),
            Some(path) => dup_str(path)?,
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
    pub fn on_oper_get_subscribe<F>(
        session: &SrSession,
        module_name: &str,
        xpath: &str,
        callback: F,
        opts: sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: for<'a> FnMut(
            &'a SrSession,
            &'a Context,
            u32,
            &'a str,
            &'a str,
            Option<&'a str>,
            u32,
            Option<DataTree<'a>>,
        ) -> Result<Option<DataTree<'a>>, SrError>,
    {
        let mut subscription_ctx: *mut sr_subscription_ctx_t =
            unsafe { zeroed::<*mut sr_subscription_ctx_t>() };
        let data = Box::into_raw(Box::new(callback));
        let module_name = dup_str(module_name)?;
        let path = dup_str(xpath)?;

        let rc = unsafe {
            sr_oper_get_subscribe(
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
        sess: *mut sr_session_ctx_t,
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
            &'a SrSession,
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
        let request_xpath = if request_xpath == std::ptr::null_mut() {
            None
        } else {
            Some(CStr::from_ptr(request_xpath).to_str().unwrap())
        };

        let session = SrSession::from(sess, false);
        let ctx = ManuallyDrop::new(session.get_context());

        let node_opt = if *parent == std::ptr::null_mut() {
            None
        } else {
            Some(DataTree::from_raw(&ctx, *parent))
        };

        let res = callback(
            &session,
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

        let sr_outputs = callback(sess, sub_id, op_path, inputs, event, request_id);
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
        options: sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: FnMut(SrSession, u32, &str, SrValues, SrEvent, u32) -> SrValues + 'static,
    {
        let mut subscription_ctx: *mut sr_subscription_ctx_t = std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let xpath = match xpath {
            Some(path) => dup_str(path)?,
            None => std::ptr::null_mut(),
        };

        let rc = unsafe {
            sr_rpc_subscribe(
                session.get_raw_mut(),
                xpath,
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
        sess: *mut sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const lyd_node,
        event: sr_event_t,
        request_id: u32,
        mut output: *mut lyd_node,
        private_data: *mut c_void,
    ) -> i32
    where
        F: for<'a> FnMut(
            &'a SrSession,
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

        let sess = SrSession::from(sess, false);
        let ctx = sess.get_context();

        let inputs = ManuallyDrop::new(DataTree::from_raw(&ctx, input as *mut _));
        let mut output = ManuallyDrop::new(DataTree::from_raw(&ctx, output as *mut _));

        let event = SrEvent::try_from(event).expect("Convert error");

        callback(
            &sess,
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
        options: sr_subscr_options_t,
    ) -> Result<Self, SrError>
    where
        F: for<'a> FnMut(
            &'a SrSession,
            &'a Context,
            u32,
            &str,
            &DataTree<'a>,
            &mut DataTree<'a>,
            SrEvent,
            u32,
        ),
    {
        let mut subscription_ctx: *mut sr_subscription_ctx_t = std::ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let xpath = match xpath {
            Some(path) => dup_str(path)?,
            None => std::ptr::null_mut(),
        };

        let rc = unsafe {
            sr_rpc_subscribe_tree(
                session.get_raw_mut(),
                xpath,
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

    mod test_module_change {
        use super::*;

        #[test]
        fn test_call_module_container_value_change() {
            log_stderr(SrLogLevel::Info);

            let mut connection = SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
            let session = connection.start_session(SrDatastore::Running).unwrap();
            let check = Arc::new(Mutex::new(0));
            let change_cb_value = check.clone();
            let callback = |_session: SrSession,
                            _sub_id: u32,
                            _module_name: &str,
                            _xpath: Option<&str>,
                            _event: SrEvent,
                            _request_id: u32|
             -> Result<(), SrError> {
                change_cb_value.lock().unwrap().deref_mut().add_assign(1);
                Ok(())
            };

            let _res = session.set_item_str("/examples:cont/l", "123", None, 0);
            let _res = session.apply_changes(None);
            assert!(_res.is_ok());

            let sub_id = session.on_module_change_subscribe(
                "examples",
                Some("/examples:cont/l"),
                callback,
                0,
                0,
            );
            assert!(sub_id.is_ok());

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
            let session = connection.start_session(SrDatastore::Running).unwrap();
            let check = Arc::new(Mutex::new(0));
            let change_cb_value = check.clone();
            let callback = |_session: SrSession,
                            _sub_id: u32,
                            _module_name: &str,
                            _xpath: Option<&str>,
                            _event: SrEvent,
                            _request_id: u32|
             -> Result<(), SrError> {
                change_cb_value.lock().unwrap().deref_mut().add_assign(1);
                Ok(())
            };

            let _res = session.set_item_str("/examples:cont/l", "123", None, 0);
            let _res = session.apply_changes(None);
            assert!(_res.is_ok());

            let sub_id = session.on_module_change_subscribe("examples", None, callback, 0, 0);
            assert!(sub_id.is_ok());

            let _res = session.set_item_str("/examples:cont/l", "321", None, 0);
            let _res = session.apply_changes(None);
            assert!(_res.is_ok());

            // Change is called 2 times
            assert_eq!(*check.lock().unwrap(), 2);
        }
    }

    mod test_oper_get_subscribe {
        use super::*;
        use yang3::data::DataDiffFlags;

        #[test]
        fn test_call_module_container_value_change() {
            log_stderr(SrLogLevel::Info);

            let mut connection =
                SrConnection::new(ConnectionOptions::Datastore_Operational).unwrap();
            let mut session = connection.start_session(SrDatastore::Operational).unwrap();

            let sub_id = session.on_oper_get_subscribe(
                "examples",
                "/examples:stats",
                move |sess, ctx, u_id, path, request, xpath, request_id, data| {
                    let mut node = DataTree::new(&ctx);
                    let _ref = node
                        .new_path("/examples:stats", None, false)
                        .map_err(|e| SrError::Internal)?;
                    let _ref = node
                        .new_path("/examples:stats/counter", Some("123"), false)
                        .map_err(|e| SrError::Internal)?;

                    return Ok(Some(node));
                },
                0,
            );
            assert!(sub_id.is_ok());
            let ctx = session.get_context();
            let _res = session.get_data(&ctx, "/examples:stats", None, None, 0);

            let mut expected_node = DataTree::new(&ctx);
            let _ref = expected_node
                .new_path("/examples:stats", None, false)
                .map_err(|e| SrError::Internal)
                .unwrap();
            let _ref = expected_node
                .new_path("/examples:stats/counter", Some("123"), false)
                .map_err(|e| SrError::Internal)
                .unwrap();

            assert!(_res.is_ok());
            let data = _res.unwrap();
            let diff = data.diff(&expected_node, DataDiffFlags::empty());
            assert!(diff.is_ok());
            assert!(diff.iter().next().is_none())
        }
    }

    mod test_rpc_subscribe {
        use super::*;
        use crate::value::Data;
        use yang3::data::Data as YangData;
        use yang3::schema::DataValue;

        #[test]
        fn test_on_rpc_subscribe() {
            log_stderr(SrLogLevel::Info);

            let mut connection =
                SrConnection::new(ConnectionOptions::Datastore_Operational).unwrap();
            let mut session = connection.start_session(SrDatastore::Operational).unwrap();

            let sub_id = session.on_rpc_subscribe(
                Some("/examples:oper"),
                |session, sub_id, xpath, inputs, event, request_id| {
                    let mut output = SrValues::new(1, false);
                    let _r = output.add_value(
                        0,
                        "/examples:oper/ret".to_string(),
                        Data::Int64(123),
                        false,
                    );
                    output
                },
                0,
                0,
            );
            assert!(sub_id.is_ok());

            let mut input = SrValues::new(2, false);
            let r = input.add_value(
                0,
                "/examples:oper/arg".to_string(),
                Data::String("123".to_string()),
                false,
            );
            let r = input.add_value(1, "/examples:oper/arg2".to_string(), Data::Int8(123), false);
            assert!(r.is_ok());
            let data = session.rpc_send("/examples:oper", Some(input), None);
            assert!(data.is_ok());
            let data = data.unwrap();
            let output = data.get_value_mut(0);
            assert!(output.is_ok());
            let output = output.unwrap();
            let path = output.xpath();
            let val = match output.data() {
                Data::Int64(val) => *val,
                _ => panic!("Expected a decimal64 output"),
            };
            assert_eq!(val, 123);
        }

        #[test]
        fn test_on_rpc_subscribe_tree() {
            log_stderr(SrLogLevel::Error);

            let mut connection =
                SrConnection::new(ConnectionOptions::Datastore_Operational).unwrap();
            let mut session = connection.start_session(SrDatastore::Operational).unwrap();

            let sub_id = session.on_rpc_subscribe_tree(
                Some("/examples:oper"),
                |session, context, sub_id, xpath, inputs, output, event, request_id| {
                    let _r = output.new_path("/examples:oper/ret", Some("123"), true);
                },
                0,
                0,
            );
            assert!(sub_id.is_ok());

            let ctx = session.get_context();
            let mut input = DataTree::new(&ctx);
            let _r = input
                .new_path("/examples:oper/arg", Some("123"), false)
                .unwrap();
            let _r = input.new_path("/examples:oper/arg2", Some("1"), false);

            let data = session.rpc_send_tree(&ctx, Some(input), None);
            assert!(data.is_ok());
            let data = data.unwrap();
            let output = data.find_path("/examples:oper/ret", true);
            assert!(output.is_ok());
            let output = output.unwrap();
            let path = output.path();
            let val = output.value();
            assert!(val.is_some());
            let val = val.unwrap();

            assert_eq!(val, DataValue::Int64(123));
        }
    }
}
