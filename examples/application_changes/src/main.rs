//
// Sysrepo-examples.
//   application_changes
//

use std::env;
use std::ffi::CStr;
use std::thread;
use std::time;

use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrLogLevel};
use sysrepo::errors::SrError;
use sysrepo::session::{SrChangeOperation, SrEvent, SrSession};
use sysrepo::*;
use sysrepo_sys::sr_val_t;
use utils::*;

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <module-to-subscribe> [<xpath-to-subscribe>]",
        program
    );
}

/// Print change.
fn print_change(change_operation: SrChangeOperation) {
    match change_operation {
        SrChangeOperation::Created(created) => {
            print!("CREATED: ");
            let val: &sr_val_t = unsafe { &*created.value.as_raw() };
            print_val(val);
        }
        SrChangeOperation::Modified(modified) => {
            let old_val: &sr_val_t = unsafe { &*modified.prev_value.as_raw() };
            let new_val: &sr_val_t = unsafe { &*modified.value.as_raw() };
            print!("MODIFIED: ");
            print_val(&old_val);
            print!("to ");
            print_val(&new_val);
        }
        SrChangeOperation::Deleted(deleted) => {
            let old_val: &sr_val_t = unsafe { &*deleted.value.as_raw() };
            print!("DELETED: ");
            print_val(&old_val);
        }
        SrChangeOperation::Moved(moved) => {
            let val: &sr_val_t = unsafe { &*moved.value.as_raw() };
            let xpath = unsafe { CStr::from_ptr(val.xpath).to_str().unwrap() };
            println!("MOVED: {}", xpath);
        }
    }
}

/// Print current config.
fn print_current_config(sess: &mut SrSession, mod_name: &str) {
    let xpath = format!("/{}:*//.", mod_name);
    let xpath = &xpath[..];

    // Get the values.
    match sess.get_items(&xpath, None, 0) {
        Err(_) => {}
        Ok(mut values) => {
            for v in values.as_raw_slice() {
                print_val(&v);
            }
        }
    }
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
        std::process::exit(1);
    }

    let mod_name = args[1].clone();

    println!(
        r#"Application will watch for changes in "{}"."#,
        if args.len() == 3 {
            args[2].clone()
        } else {
            args[1].clone()
        }
    );

    // Turn logging on.
    log_stderr(SrLogLevel::Warn);

    // Connect to sysrepo.
    let mut sr = match SrConnection::new(ConnectionOptions::Datastore_StartUp) {
        Ok(sr) => sr,
        Err(_) => return false,
    };

    // Start session.
    let mut sess = match sr.start_session(SrDatastore::Running) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Read current config.
    println!("");
    println!(" ========== READING RUNNING CONFIG: ==========");
    println!("");
    print_current_config(&mut sess, &mod_name);

    let f = |mut sess: SrSession,
             sub_id: u32,
             mod_name: &str,
             _path: Option<&str>,
             event: SrEvent,
             _request_id: u32|
     -> Result<(), SrError> {
        let path = "//.";
        let mut iter = match sess.get_changes_iter(&path) {
            Ok(iter) => iter,
            Err(_) => return Ok(()),
        };

        println!("");
        println!("");
        println!(
            " ========== EVENT ({}) {} CHANGES: ====================================",
            sub_id, event
        );
        println!("");

        for op in iter {
            print_change(op);
        }

        println!("");
        print!(" ========== END OF CHANGES =======================================");

        if event == SrEvent::Done {
            println!("");
            println!("");
            println!(" ========== CONFIG HAS CHANGED, CURRENT RUNNING CONFIG: ==========");
            println!("");
            print_current_config(&mut sess, mod_name);
        }
        return Ok(());
    };

    // Subscribe for changes in running config.
    if args.len() == 3 {
        let xpath = args[2].clone();
        match sess.on_module_change_subscribe(&mod_name, Some(&xpath[..]), f, 0, 0) {
            Err(_) => return false,
            Ok(subscr) => subscr,
        }
    } else {
        match sess.on_module_change_subscribe(&mod_name, None, f, 0, 0) {
            Err(_) => return false,
            Ok(subscr) => subscr,
        }
    };

    println!("\n\n ========== LISTENING FOR CHANGES ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    println!("Application exit requested, exiting.");

    true
}
