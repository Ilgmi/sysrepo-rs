//
// Sysrepo-examples.
//   oper_data
//

use std::env;
use std::thread;
use std::time;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrLogLevel};
use sysrepo::errors::SrError;
use sysrepo::session::SrSession;
use sysrepo::*;
use utils::*;
use yang3::context::Context;
use yang3::data::{DataTree, NewValueCreationOptions};

/// Show help.
fn print_help(program: &str) {
    println!(
        "Usage: {} <module-to-provide-data-from> <path-to-provide>",
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

    if args.len() != 3 {
        print_help(&program);
        std::process::exit(1);
    }

    let mod_name = args[1].clone();
    let path = args[2].clone();

    println!(
        r#"Application will provide data "{}" of "{}"."#,
        path, mod_name
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

    // Subscribe for the providing the operational data.
    if let Err(_) = sess.on_oper_get_subscribe(
        &mod_name,
        &path,
        |session: &mut SrSession,
         ctx: &Context,
         sub_id: u32,
         mod_name: &str,
         path: &str,
         _request_xpath: Option<&str>,
         _request_id: u32,
         _node_opt|
         -> Result<Option<DataTree>, SrError> {
            println!("");
            println!("");
            println!(
                r#" ========== DATA ({}) FOR "{}" "{}" REQUESED ======================="#,
                sub_id, mod_name, path
            );
            println!("");

            if mod_name == "examples" && path == "/examples:stats" {
                let mut node = DataTree::new(&ctx);
                let _val1 = node.new_path(
                    "/examples:stats/counter",
                    Some("852"),
                    NewValueCreationOptions::NEW_ANY_USE_VALUE,
                );
                let _val2 = node.new_path(
                    "/examples:stats/counter2",
                    Some("1052"),
                    NewValueCreationOptions::NEW_ANY_USE_VALUE,
                );

                Ok(Some(node))
            } else {
                Ok(None)
            }
        },
        0,
    ) {
        return false;
    }

    println!("\n\n ========== LISTENING FOR REQUESTS ==========\n");

    signal_init();
    while !is_sigint_caught() {
        thread::sleep(time::Duration::from_secs(1));
    }

    true
}
