use std::path::Path;
use sysrepo::connection::{ConnectionOptions, SrConnection};

mod common;

#[test]
fn install_and_remove_module_successful() {
    let connection = SrConnection::new(ConnectionOptions::Datastore_Running)
        .expect("Should be Ok");
    let install = connection.install_module(
        Path::new("./assets/yang/install-test.yang"),
        None,
        None,
    );
    assert!(install.is_ok());
    let remove = connection.remove_module("install-test", false);
    assert!(remove.is_ok());

    let modules = vec![
        ("sub", None),
        ("install-import-test", Some(vec!["sub-feature"])),
    ];
    let yang = "./assets/yang/";
    let connection = SrConnection::new(ConnectionOptions::Datastore_Running)
        .expect("Should be Ok");
    for (module_name, features) in &modules {
        let bind = Path::new(yang).join(format!("{module_name}.yang"));
        let module_path = bind.as_path();
        assert!(module_path.exists());

        let features = match &features {
            None => None,
            Some(features) => Some(&features[..]),
        };

        let install =
            connection.install_module(module_path, Some(yang), features);
        assert!(install.is_ok(), "Could not install module {module_name}");
    }

    for (module_name, _features) in modules.iter().rev() {
        let remove = connection.remove_module(module_name, false);
        assert!(remove.is_ok());
    }
}
