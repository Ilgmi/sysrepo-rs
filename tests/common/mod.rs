use std::path::Path;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::SrDatastore;

const _YANG: &str = "./assets/yang";
const TEST_MODULE: &str = "./assets/yang/test_module.yang";
const _INSTALL_IMPORT: &str = "/assets/yang/install-import-test.yang";
const _INSTALL_TEST: &str = "./assets/yang/install-test.yang";
const _SUB: &str = "./assets/yang/sub.yang";

pub struct _Setup {
    _connection: SrConnection,
}

impl _Setup {
    pub fn _setup_test_module() -> Self {
        let mut connection =
            SrConnection::new(ConnectionOptions::Datastore_Running).unwrap();
        connection
            .install_module(Path::new(TEST_MODULE), None, None)
            .unwrap();

        let stores = vec![
            SrDatastore::Startup,
            SrDatastore::Running,
            SrDatastore::Candidate,
        ];
        for store in stores {
            let session = connection.start_session(store).unwrap();
            session.replace_config(None, None, None).unwrap();
        }
        Self {
            _connection: connection,
        }
    }

    pub fn _setup_example() -> Self {
        let con =
            SrConnection::new(ConnectionOptions::Datastore_Running).unwrap();
        con.install_module(
            Path::new("./assets/yang/examples@2017-01-19.yang"),
            None,
            None,
        )
        .unwrap();
        Self { _connection: con }
    }
}
