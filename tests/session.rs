use crate::common::Setup;
use std::fmt::{Debug, Display};
use std::mem::ManuallyDrop;
use std::time::Duration;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{
    DefaultOperation, SrDatastore, SrEditFlag, SrGetOptions, SrLogLevel,
};
use sysrepo::errors::SrError;
use sysrepo::session::SrSession;
use sysrepo::{log_stderr, value};
use yang3::context::Context;
use yang3::data::{Data, DataFormat, DataPrinterFlags, DataTree};
use yang3::schema::{DataValue, SchemaPathFormat};

mod common;

const LEAF: &str = "/test_module:testInt32";

#[test]
fn test_session() {
    test_data_manipulation();
    test_get_data_max_depth();
    test_get_data_options_for_operational_ds();
    test_edit_batch();
    test_get_items();
    test_pending_changes();
    test_replace_config_with_none();
    test_replace_config_with_config();
    test_copy_config_from_startup_to_running();
}

fn test_data_manipulation() {
    // Turn logging on.
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .copy_config(SrDatastore::Startup, None, Duration::from_secs(3))
        .expect("Copy Startup to Running");

    let data =
        session.get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT);
    assert!(data.is_err());

    match data {
        Ok(_) => {
            panic!("Expect to be empty")
        }
        Err(err) => {
            assert_eq!(err, SrError::NotFound)
        }
    }

    assert!(session.set_item_str(LEAF, Some("123"), None, 0).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data = session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .expect("Get Data");
    assert_eq!(
        data.reference().unwrap().value(),
        Some(DataValue::Int32(123))
    );

    let node = session.get_node(&ctx, LEAF, None);
    assert!(node.is_ok());
    assert_eq!(
        node.unwrap().reference().unwrap().value(),
        Some(DataValue::Int32(123))
    );

    assert!(session.set_item_str(LEAF, Some("420"), None, 0).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data =
        session.get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT);
    assert!(data.is_ok());
    assert_eq!(
        data.unwrap().reference().unwrap().value(),
        Some(DataValue::Int32(420))
    );

    assert!(session.remove_item(LEAF, SrEditFlag::Default).is_ok());
    assert!(session.apply_changes(None).is_ok());

    let data =
        session.get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT);
    assert!(data.is_err_and(|e| e == SrError::NotFound));

    assert!(session.set_item_str(LEAF, Some("420"), None, 0).is_ok());
    assert!(session.discard_changes().is_ok());

    let data =
        session.get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT);
    assert!(data.is_err_and(|e| e == SrError::NotFound));

    assert!(session
        .set_item_str("/test_module:cont/l", Some("test 123"), None, 0)
        .is_ok());
    let data = session.get_data(
        &ctx,
        "/test_module:cont/l",
        0,
        None,
        SrGetOptions::SR_OPER_DEFAULT,
    );
    assert!(data.is_ok());
    let data = data.unwrap();
    let data = data.reference().unwrap();

    assert_eq!(data.path(), "/test_module:cont");
    let val = data.find_path("/test_module:cont/l");
    assert!(val.is_ok());
    let val = val.unwrap();

    assert_eq!(val.value(), Some(DataValue::Other("test 123".to_string())));

    let node = session.get_node(&ctx, "/test_module:cont/l", None);
    assert!(node.is_ok());
    let node = node.unwrap();
    let node_ref = node.reference().unwrap();
    assert_eq!(node_ref.path(), "/test_module:l");
    assert_eq!(
        node_ref.schema().path(SchemaPathFormat::DATA),
        "/test_module:cont/l"
    );
    assert_eq!(
        node_ref.value(),
        Some(DataValue::Other("test 123".to_string()))
    );

    let node = session.get_node(&ctx, "/test_module:cont", None);
    assert!(node.is_ok());
    let node = node.unwrap();
    let node_ref = node.reference().unwrap();
    assert_eq!(node_ref.path(), "/test_module:cont");
    assert!(node_ref.value().is_none());

    session.discard_changes().unwrap();

    assert!(session
        .set_item_str("/test_module:not-existing", Some("test 123"), None, 0)
        .is_err_and(|e| e == SrError::Ly));
    assert!(session
        .get_data(
            &ctx,
            "/test_module:not-existing",
            0,
            None,
            SrGetOptions::SR_OPER_DEFAULT
        )
        .is_err_and(|e| e == SrError::NotFound));
    assert!(session
        .get_node(&ctx, "/test_module:not-existing", None)
        .is_err_and(|e| e == SrError::NotFound));
}

fn test_get_data_max_depth() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .set_item_str(
            "/test_module:cont/sub/test-list[name='test']/val",
            Some("test"),
            None,
            0,
        )
        .unwrap();
    session
        .set_item_str(
            "/test_module:cont/sub/test-list[name='nop']",
            None,
            None,
            0,
        )
        .unwrap();

    let data = session
        .get_data(
            &ctx,
            "/test_module:cont",
            0,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .expect("Data should exist");
    let str = data
        .print_string(DataFormat::JSON, DataPrinterFlags::KEEP_EMPTY_CONT)
        .expect("Expect to print");
    assert_eq!(
        str,
        r#"{
  "test_module:cont": {
    "sub": {
      "test-list": [
        {
          "name": "nop"
        },
        {
          "name": "test",
          "val": "test"
        }
      ]
    }
  }
}
"#
    );
    let data = session
        .get_data(
            &ctx,
            "/test_module:cont",
            1,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .unwrap();
    let str = data
        .print_string(DataFormat::JSON, DataPrinterFlags::KEEP_EMPTY_CONT)
        .expect("Expect to print");
    assert_eq!(
        str,
        r#"{
  "test_module:cont": {
    "sub": {}
  }
}
"#
    );

    let data = session
        .get_data(
            &ctx,
            "/test_module:cont",
            2,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .unwrap();
    let str = data
        .print_string(DataFormat::JSON, DataPrinterFlags::KEEP_EMPTY_CONT)
        .expect("Expect to print");
    assert_eq!(
        str,
        r#"{
  "test_module:cont": {
    "sub": {
      "test-list": [
        {
          "name": "nop"
        },
        {
          "name": "test"
        }
      ]
    }
  }
}
"#
    );

    let data = session
        .get_data(
            &ctx,
            "/test_module:cont",
            3,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .unwrap();
    let str = data
        .print_string(DataFormat::JSON, DataPrinterFlags::KEEP_EMPTY_CONT)
        .expect("Expect to print");
    assert_eq!(
        str,
        r#"{
  "test_module:cont": {
    "sub": {
      "test-list": [
        {
          "name": "nop"
        },
        {
          "name": "test",
          "val": "test"
        }
      ]
    }
  }
}
"#
    );

    let data = session
        .get_data(
            &ctx,
            "/test_module:cont",
            4,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .unwrap();
    let str = data
        .print_string(DataFormat::JSON, DataPrinterFlags::KEEP_EMPTY_CONT)
        .expect("Expect to print");
    assert_eq!(
        str,
        r#"{
  "test_module:cont": {
    "sub": {
      "test-list": [
        {
          "name": "nop"
        },
        {
          "name": "test",
          "val": "test"
        }
      ]
    }
  }
}
"#
    );
}

fn test_get_data_options_for_operational_ds() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .switch_datastore(SrDatastore::Operational)
        .expect("Should Switch");

    session
        .set_item_str("/test_module:stateLeaf", Some("42"), None, 0)
        .unwrap();
    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    session.apply_changes(None).unwrap();

    let data = session
        .get_data(
            &ctx,
            "/test_module:*",
            0,
            None,
            SrGetOptions::SR_OPER_DEFAULT,
        )
        .unwrap();
    assert!(data.find_path("/test_module:stateLeaf").is_ok());
    assert!(data.find_path(LEAF).is_ok());

    let data = session
        .get_data(
            &ctx,
            "/test_module:*",
            0,
            None,
            SrGetOptions::SR_OPER_NO_STATE,
        )
        .unwrap();
    assert!(data.find_path("/test_module:stateLeaf").is_err());
    assert!(data.find_path(LEAF).is_ok());
}

fn test_edit_batch() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    session
        .copy_config(
            SrDatastore::Startup,
            Some("test_module"),
            Duration::from_secs(0),
        )
        .unwrap();

    assert!(session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .is_err());

    let mut batch = DataTree::new(&ctx);
    batch.new_path(LEAF, Some("123"), false).unwrap();

    assert!(session.edit_batch(&batch, DefaultOperation::Merge).is_ok());
    assert!(session.apply_changes(None).is_ok());
    let data = session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .unwrap();
    let data = data.reference().unwrap().value();
    assert_eq!(data, Some(DataValue::Int32(123)));
}

fn test_get_items() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");

    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    session.apply_changes(None).unwrap();

    let values = session.get_items(LEAF, None, 0);
    assert!(values.is_ok());

    let values = values.unwrap();
    assert_eq!(values.len(), 1);

    let value = values.get_value_mut(0);
    assert!(value.is_ok());

    let value = value.unwrap();
    assert_eq!(value.xpath(), LEAF);
    match value.data() {
        value::Data::Int32(val) => {
            assert_eq!(*val, 1)
        }
        _ => panic!("Wrong data type"),
    }
}

fn test_pending_changes() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    assert!(session.get_pending_changes(&ctx).is_none());
    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    let changes = session.get_pending_changes(&ctx);
    assert!(changes.is_some());
    let changes = changes.unwrap();
    assert_eq!(
        changes.reference().unwrap().value().unwrap(),
        DataValue::Int32(1)
    );
    session.apply_changes(None).unwrap();
    assert!(session.get_pending_changes(&ctx).is_none());

    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    let changes = session.get_pending_changes(&ctx);
    assert!(changes.is_some());
    let changes = changes.unwrap();
    assert_eq!(
        changes.reference().unwrap().value().unwrap(),
        DataValue::Int32(1)
    );

    session.discard_changes().unwrap();
    assert!(session.get_pending_changes(&ctx).is_none());

    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    let changes = session.get_pending_changes(&ctx);
    assert!(changes.is_some());
    let changes = changes.unwrap();
    assert_eq!(
        changes.reference().unwrap().value().unwrap(),
        DataValue::Int32(1)
    );

    session.discard_items(LEAF).unwrap();
    assert!(session.get_pending_changes(&ctx).is_none());
}

fn prepare_test_replace_config<'a>(
    session: &mut SrSession,
    ctx: &'a ManuallyDrop<Context>,
) -> ManuallyDrop<DataTree<'a>> {
    assert!(session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .is_err());

    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    session.apply_changes(None).unwrap();

    let conf =
        session.get_data(&ctx, "/*", 0, None, SrGetOptions::SR_OPER_DEFAULT);
    assert!(conf.is_ok());
    let conf = conf.unwrap();

    session.set_item_str(LEAF, Some("123"), None, 0).unwrap();
    session.apply_changes(None).unwrap();

    let data = session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .unwrap();
    let data = data.reference().unwrap().value();
    assert_eq!(data, Some(DataValue::Int32(123)));

    ManuallyDrop::new(conf)
}

fn test_replace_config_with_none() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    prepare_test_replace_config(session, &ctx);

    assert!(session
        .replace_config(None, Some("test_module"), None)
        .is_ok());
    assert!(session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .is_err());
}

fn test_replace_config_with_config() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut connection =
        SrConnection::new(ConnectionOptions::Datastore_Running)
            .expect("connect");
    let session = connection
        .start_session(SrDatastore::Running)
        .expect("session");
    let ctx = session.get_context();

    let conf = prepare_test_replace_config(session, &ctx);

    assert!(session
        .replace_config(Some(&conf), Some("test_module"), None)
        .is_ok());
    let data = session
        .get_data(&ctx, LEAF, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .unwrap();
    let value = data.reference().unwrap().value();
    assert_eq!(value, Some(DataValue::Int32(1)));
}

fn test_copy_config_from_startup_to_running() {
    log_stderr(SrLogLevel::Error);
    let _setup = Setup::setup_test_module();

    let mut con = SrConnection::new(ConnectionOptions::Datastore_StartUp)
        .expect("connect");
    let ctx = con.get_context();
    let session = con.start_session(SrDatastore::Startup).expect("session");

    session.set_item_str(LEAF, Some("1"), None, 0).unwrap();
    session.apply_changes(None).unwrap();

    session.switch_datastore(SrDatastore::Running).unwrap();

    assert!(session
        .copy_config(SrDatastore::Startup, None, Duration::from_secs(2))
        .is_ok());

    let data = session.get_data(&ctx, LEAF, 0, None, SrGetOptions::empty());
    assert!(data.is_ok());
    let data = data.unwrap();
    let value = data.reference().unwrap().value();
    assert_eq!(value, Some(DataValue::Int32(1)));
}
