use crate::common::Setup;
use std::ops::{AddAssign, DerefMut};
use std::sync::{Arc, Mutex};
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrLogLevel};
use sysrepo::errors::SrError;
use sysrepo::log_stderr;
use sysrepo::session::{SrEvent, SrSession};

mod common;

#[test]
fn test_subscriptions() {
    let _setup = Setup::setup_example();

    test_module_change::test_call_module_container_value_change();
    test_module_change::test_call_module_change();

    test_oper_get_subscribe::test_call_module_container_value_change();

    test_rpc_subscribe::test_on_rpc_subscribe();

    test_on_notification_subscribe::test_on_notification_subscribe();
    test_on_notification_subscribe::test_on_notification_subscribe_tree();
}

mod test_module_change {
    use super::*;
    pub fn test_call_module_container_value_change() {
        log_stderr(SrLogLevel::Info);
        let _setup = Setup::setup_test_module();

        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
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

        let _res =
            session.set_item_str("/examples:cont/l", Some("123"), None, 0);
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

        let _res =
            session.set_item_str("/examples:cont/l", Some("321"), None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        // Change is called 2 times
        assert_eq!(*check.lock().unwrap(), 2);
    }

    pub fn test_call_module_change() {
        log_stderr(SrLogLevel::Info);

        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
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

        let _res =
            session.set_item_str("/examples:cont/l", Some("123"), None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        let sub_id = session
            .on_module_change_subscribe("examples", None, callback, 0, 0);
        assert!(sub_id.is_ok());

        let _res =
            session.set_item_str("/examples:cont/l", Some("321"), None, 0);
        let _res = session.apply_changes(None);
        assert!(_res.is_ok());

        // Change is called 2 times
        assert_eq!(*check.lock().unwrap(), 2);
    }
}

mod test_oper_get_subscribe {
    use super::*;
    use sysrepo::enums::SrGetOptions;
    use yang3::data::{DataDiffFlags, DataTree};

    pub fn test_call_module_container_value_change() {
        log_stderr(SrLogLevel::Info);

        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_Operational)
                .unwrap();
        let session =
            connection.start_session(SrDatastore::Operational).unwrap();

        let sub_id = session.on_oper_get_subscribe(
            "examples",
            "/examples:stats",
            |_sess, ctx, _u_id, _path, _request, _xpath, _request_id, _data| {
                let mut node = DataTree::new(&ctx);
                let _ref = node
                    .new_path("/examples:stats", None, false)
                    .map_err(|_e| SrError::Internal)?;
                let _ref = node
                    .new_path("/examples:stats/counter", Some("123"), false)
                    .map_err(|_e| SrError::Internal)?;

                return Ok(Some(node));
            },
            0,
        );
        assert!(sub_id.is_ok());
        let ctx = session.get_context();
        let _res = session.get_data(
            &ctx,
            "/examples:stats",
            0,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        );

        let mut expected_node = DataTree::new(&ctx);
        let _ref = expected_node
            .new_path("/examples:stats", None, false)
            .map_err(|_e| SrError::Internal)
            .unwrap();
        let _ref = expected_node
            .new_path("/examples:stats/counter", Some("123"), false)
            .map_err(|_e| SrError::Internal)
            .unwrap();

        assert!(_res.is_ok());
        let data = _res.unwrap();
        let diff = data.diff(&expected_node, DataDiffFlags::empty());
        assert!(diff.is_ok());
        let diff = diff.unwrap();
        assert_eq!(diff.iter().count(), 0);
    }
}

mod test_rpc_subscribe {
    use super::*;
    use sysrepo::value::Data;
    use sysrepo::values::SrValues;
    use yang3::data::DataTree;

    pub fn test_on_rpc_subscribe() {
        log_stderr(SrLogLevel::Info);

        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_Operational)
                .unwrap();
        let session =
            connection.start_session(SrDatastore::Operational).unwrap();

        let sub_id = session.on_rpc_subscribe(
            Some("/examples:oper"),
            |_session, _sub_id, _xpath, _inputs, _event, _request_id| {
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
        assert!(r.is_ok());
        let r = input.add_value(
            1,
            "/examples:oper/arg2".to_string(),
            Data::Int8(123),
            false,
        );
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
        assert_eq!(&path, "/examples:oper/ret");
    }

    fn test_on_rpc_subscribe_tree() {
        log_stderr(SrLogLevel::Error);

        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_Operational)
                .unwrap();
        let session =
            connection.start_session(SrDatastore::Operational).unwrap();

        let sub_id = session.on_rpc_subscribe_tree(
            Some("/examples:oper"),
            |_session,
             _context,
             _sub_id,
             _xpath,
             _inputs,
             output,
             _event,
             _request_id| {
                let _r =
                    output.new_path("/examples:oper/ret", Some("123"), true);
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
        let output_path = "/examples:oper/ret";
        // let output = data.find_output_path(output_path);
        // assert!(output.is_ok());
        // let output = output.unwrap();
        // let path = output.path();
        // let val = output.value();
        // assert!(val.is_some());
        // let val = val.unwrap();

        // assert_eq!(val, DataValue::Int64(123));
        // assert_eq!(&path, output_path);
    }
}

mod test_on_notification_subscribe {
    use super::*;
    use sysrepo::enums::SrNotifType;
    use sysrepo::value::Data;
    use sysrepo::values::SrValues;
    use yang3::data::{Data as yang_data, DataTree};
    use yang3::schema::DataValue;

    pub fn test_on_notification_subscribe() {
        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();

        let session = connection.start_session(SrDatastore::Running).unwrap();
        let check_cb = Arc::new(Mutex::new(0));
        let check_for_cb = check_cb.clone();
        let subscription = session.on_notif_subscribe(
            "examples",
            Some("/examples:notif"),
            None,
            None,
            move |_session,
                  _sub_id,
                  _notify_type,
                  xpath,
                  values,
                  _timestamp| {
                match _notify_type {
                    SrNotifType::Realtime | SrNotifType::Replay => {
                        assert_eq!(xpath, Some("/examples:notif"));
                        assert_eq!(values.len(), 1);
                        let value = values.get_value_mut(0).expect("value");
                        match value.data() {
                            Data::Decimal64(data) => {
                                assert!((*data).eq(&123.0))
                            }
                            _ => panic!("Expected a decimal64 output"),
                        }
                    }
                    _ => {}
                }

                check_for_cb.lock().unwrap().add_assign(1);
            },
            0,
        );
        assert!(subscription.is_ok());
        let mut values = SrValues::new(1, false);
        assert!(values
            .add_value(
                0,
                "/examples:notif/val".to_string(),
                Data::Decimal64(123.0),
                false
            )
            .is_ok());

        let notification_send =
            session.notif_send("/examples:notif", &values, 0, 1);
        assert!(notification_send.is_ok());
    }

    pub fn test_on_notification_subscribe_tree() {
        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_StartUp).unwrap();
        let session = connection.start_session(SrDatastore::Running).unwrap();
        let check_cb = Arc::new(Mutex::new(0));
        let check_for_cb = check_cb.clone();
        let subscription = session.on_notif_subscribe_tree(
            "examples",
            Some("/examples:notif"),
            None,
            None,
            move |_session, _sub_id, _notify_type, node, _timestamp| {
                match _notify_type {
                    SrNotifType::Realtime | SrNotifType::Replay => {
                        let node = node.reference().expect("node");
                        let xpath = node.path();
                        assert_eq!(xpath, "/examples:notif");

                        let value_node = node
                            .find_path("/examples:notif/val")
                            .expect("value");
                        let value = value_node.value();

                        match value {
                            Some(value) => match value {
                                DataValue::Other(data) => {
                                    assert!(data.eq("123.0"))
                                }
                                _ => panic!("Expected a decimal64 output"),
                            },
                            None => {
                                panic!("Expected a decimal64 output")
                            }
                        }
                    }
                    _ => {}
                }

                check_for_cb.lock().unwrap().add_assign(1);
            },
            0,
        );
        assert!(subscription.is_ok());

        let ctx = session.get_context();
        let mut notf_node = DataTree::new(&ctx);
        let r = notf_node.new_path("/examples:notif/val", Some("123.0"), false);
        assert!(r.is_ok());
        session.notif_send_tree(&notf_node, 0, 1).unwrap()
    }
}
