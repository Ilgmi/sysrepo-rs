use crate::common::Setup;
use std::fmt::{Debug, Display};
use std::time::Duration;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrEditFlag, SrLogLevel};
use sysrepo::errors::SrError;
use sysrepo::log_stderr;
use yang3::data::Data;
use yang3::schema::{DataValue, SchemaPathFormat};

mod common;

#[test]
fn test_data_manipulation() {
    // Turn logging on.
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let leaf = "/examples:testInt32";

    let mut connection = SrConnection::new(ConnectionOptions::Datastore_Running).expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .copy_config(SrDatastore::Startup, None, Duration::from_secs(3))
        .expect("Copy Startup to Running");

    let data = session.get_data(&ctx, leaf, None, None, 0);
    assert!(data.is_err());

    match data {
        Ok(_) => {
            panic!("Expect to be empty")
        }
        Err(err) => {
            assert_eq!(err, SrError::NotFound)
        }
    }

    assert!(session.set_item_str(leaf, Some("123"), None, 0).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data = session
        .get_data(&ctx, leaf, None, None, 0)
        .expect("Get Data");
    assert_eq!(
        data.reference().unwrap().value(),
        Some(DataValue::Int32(123))
    );

    let node = session.get_node(&ctx, leaf, None);
    assert!(node.is_ok());
    assert_eq!(
        node.unwrap().reference().unwrap().value(),
        Some(DataValue::Int32(123))
    );

    assert!(session.set_item_str(leaf, Some("420"), None, 0).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data = session.get_data(&ctx, leaf, None, None, 0);
    assert!(data.is_ok());
    assert_eq!(
        data.unwrap().reference().unwrap().value(),
        Some(DataValue::Int32(420))
    );

    assert!(session.remove_item(leaf, SrEditFlag::Default).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data = session.get_data(&ctx, leaf, None, None, 0);
    assert!(data.is_err_and(|e| e == SrError::NotFound));

    assert!(session.set_item_str(leaf, Some("420"), None, 0).is_ok());
    assert!(session.discard_changes().is_ok());

    let data = session.get_data(&ctx, leaf, None, None, 0);
    assert!(data.is_err_and(|e| e == SrError::NotFound));

    assert!(session
        .set_item_str("/examples:cont/l", Some("test 123"), None, 0)
        .is_ok());
    let data = session.get_data(&ctx, "/examples:cont/l", None, None, 0);
    assert!(data.is_ok());
    let data = data.unwrap();
    let data = data.reference().unwrap();

    assert_eq!(data.path(), "/examples:cont");
    let val = data.find_path("/examples:cont/l", false);
    assert!(val.is_ok());
    let val = val.unwrap();

    assert_eq!(val.value(), Some(DataValue::Other("test 123".to_string())));

    let node = session.get_node(&ctx, "/examples:cont/l", None);
    assert!(node.is_ok());
    let node = node.unwrap();
    let node_ref = node.reference().unwrap();
    assert_eq!(node_ref.path(), "/examples:l");
    assert_eq!(
        node_ref.schema().path(SchemaPathFormat::DATA),
        "/examples:cont/l"
    );
    assert_eq!(
        node_ref.value(),
        Some(DataValue::Other("test 123".to_string()))
    );

    let node = session.get_node(&ctx, "/examples:cont", None);
    assert!(node.is_ok());
    let node = node.unwrap();
    let node_ref = node.reference().unwrap();
    assert_eq!(node_ref.path(), "/examples:cont");
    assert!(node_ref.value().is_none());

    session.discard_changes().unwrap();

    assert!(session
        .set_item_str("/examples:not-existing", Some("test 123"), None, 0)
        .is_err_and(|e| e == SrError::Ly));
    assert!(session
        .get_data(&ctx, "/examples:not-existing", None, None, 0)
        .is_err_and(|e| e == SrError::NotFound));
    assert!(session
        .get_node(&ctx, "/examples:not-existing", None)
        .is_err_and(|e| e == SrError::NotFound));
}

#[test]
fn test_get_data() {
    log_stderr(SrLogLevel::Error);
    let mut connection = SrConnection::new(ConnectionOptions::Datastore_Running).expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .set_item_str(
            "/examples:cont/test-list[name='test']/val",
            Some("test"),
            None,
            0,
        )
        .unwrap();
    session
        .set_item_str("/examples:cont/test-list[name='nop']", None, None, 0)
        .unwrap();
}
