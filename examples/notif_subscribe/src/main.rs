//
// Sysrepo-examples.
//   notif_subscribe
//

use std::env;
use std::thread;
use std::time;

use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrLogLevel, SrNotifType};
use sysrepo::session::SrSession;
use sysrepo::values::SrValues;
use sysrepo::*;
use sysrepo_sys::timespec;
use utils::*;

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <module-with-notification> [<xpath-filtering-notifications>]",
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

    if args.len() != 2 && args.len() != 3 {
        print_help(&program);
        return false;
    }

    let mod_name = args[1].clone();
    let xpath = if args.len() == 3 {
        Some(args[2].as_str())
    } else {
        None
    };

    println!(
        r#"Application will subscribe "{}" notifications."#,
        mod_name
    );

    // Turn logging on.
    log_stderr(SrLogLevel::Warn);

    // Connect to sysrepo.
    let mut sr = match SrConnection::new(ConnectionOptions::Datastore_StartUp) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Start session.
    let sess = match sr.start_session(SrDatastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Callback function.
    let f = |_sess: SrSession,
             sub_id: u32,
             _notif_type: SrNotifType,
             path: Option<&str>,
             values: SrValues,
             _timestamp: *mut timespec| {
        let path = path.unwrap_or("");
        println!("");
        println!("");
        println!(
            r#" ========== NOTIFICATION ({}) "{}" RECEIVED ======================="#,
            sub_id, path
        );
        println!("");

        for v in values.as_raw_slice() {
            print_val(&v);
        }
    };

    // Subscribe for the notifications.
    if let Err(_) = sess.on_notif_subscribe(&mod_name, xpath, None, None, f, 0)
    {
        return false;
    }

    println!("\n\n ========== LISTENING FOR NOTIFICATIONS ==========\n");

    // Loop until ctrl-c is pressed / SIGINT is received.
    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    true
}
