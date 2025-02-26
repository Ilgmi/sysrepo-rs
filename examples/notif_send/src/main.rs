//
// Sysrepo-examples.
//   notif_send
//

use std::env;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrLogLevel};
use sysrepo::*;
use yang3::data::{DataTree, NewValueCreationOptions};

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <notification-path> [<node-to-set> <node-value>]",
        program
    );
}

/// Main.
fn main() {
    if run() {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

fn run() -> bool {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() < 2 || args.len() > 4 || args.len() == 3 {
        print_help(&program);
        return false;
    }

    let path = args[1].clone();
    let node_path_val = if args.len() == 4 {
        Some((args[2].clone(), args[3].clone()))
    } else {
        None
    };

    println!(
        r#"Application will send notification "{}" notification."#,
        path
    );

    // Turn logging on.
    log_stderr(SrLogLevel::Warn);

    // Connect to sysrepo.
    let mut sr = match SrConnection::new(ConnectionOptions::Datastore_Running) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Get Lib Yang Context from sysrepo connection.
    let ly_ctx = sr.get_context();

    // Start session.
    let sess = match sr.start_session(SrDatastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Create the notification.
    let mut notif = DataTree::new(&ly_ctx);
    let mut _val = match notif.new_path(&path, None, NewValueCreationOptions::NEW_ANY_USE_VALUE) {
        Ok(notif) => notif.unwrap(),
        Err(_) => {
            println!(r#"Creating notification "{}" failed."#, path);
            return false;
        }
    };

    // Add the input value.
    if let Some((path, value)) = node_path_val {
        match notif.new_path(
            &path,
            Some(&value),
            NewValueCreationOptions::NEW_ANY_USE_VALUE,
        ) {
            Ok(_) => {}
            Err(_) => {
                println!(r#"Creating value "{}" failed."#, path);
                return false;
            }
        }
    }

    // Send the notification.
    if let Err(_) = sess.notif_send_tree(&notif, 0, 0) {
        return false;
    }

    true
}
