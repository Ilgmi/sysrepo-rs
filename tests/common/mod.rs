use std::path::Path;
use sysrepo::connection::{ConnectionOptions, SrConnection};

const YANG: &str = "./assets/yang";
const TEST_MODULE: &str = "./assets/yang/test_module.yang";
const INSTALL_IMPORT: &str = "/assets/yang/install-import-test.yang";
const INSTALL_TEST: &str = "./assets/yang/install-test.yang";
const SUB: &str = "./assets/yang/sub.yang";

pub struct Setup {
    connection: SrConnection,
    modules: Vec<String>,
}

impl Setup {
    pub fn setup_test_module() -> Self {
        let connection = SrConnection::new(ConnectionOptions::Datastore_Running).unwrap();
        match connection.remove_module("test_module", true) {
            Ok(_) => (),
            Err(e) => println!("Setup Test Module: {e}"),
        }

        connection
            .install_module(Path::new(TEST_MODULE), None, None)
            .unwrap();
        let modules = vec!["test_module".to_string()];
        Self {
            connection,
            modules,
        }
    }
}
