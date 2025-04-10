//
// Sysrepo-examples.
//   sr_get_data
//

use std::env;
use sysrepo::connection::{ConnectionOptions, SrConnection};
use sysrepo::enums::{SrDatastore, SrGetOptions, SrLogLevel};
use sysrepo::*;
use yang3::data::{Data, DataFormat, DataPrinterFlags};

/// Show help.
fn print_help(program: &str) {
    println!("Usage: {} <x-path-to-get> [running/operational]", program);
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

    let xpath = args[1].clone();
    let mut ds = SrDatastore::Running;

    if args.len() == 3 {
        if args[2] == "running" {
            ds = SrDatastore::Running;
        } else if args[2] == "operational" {
            ds = SrDatastore::Operational;
        } else {
            println!("Invalid datastore {}.", args[2]);
            return false;
        }
    }

    println!(
        r#"Application will get "{}" from "{}" datastore."#,
        xpath,
        if ds == SrDatastore::Running {
            "running"
        } else {
            "operational"
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
    let sess = match sr.start_session(ds) {
        Ok(sess) => sess,
        Err(_) => return false,
    };

    // Setup libyang context.
    let ctx = sess.get_context();

    // Get the data.
    let data = sess
        .get_data(&ctx, &xpath, 0, None, SrGetOptions::SR_OPER_DEFAULT)
        .expect("Failed to get data");

    // Print data tree in the XML format.

    data.print_file(
        std::io::stdout(),
        DataFormat::XML,
        DataPrinterFlags::WD_ALL | DataPrinterFlags::WITH_SIBLINGS,
    )
    .expect("Failed to print data tree");

    true
}
