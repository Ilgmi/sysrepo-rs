#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod common;
pub mod connection;
pub mod enums;
pub mod errors;
pub mod session;
pub mod subscription;
pub mod value;
pub mod values;

use crate::common::str_to_cstring;
use crate::enums::SrLogLevel;
use crate::errors::SrError;
use sysrepo_sys as ffi_sys;

/// Set Log Stderr.
pub fn log_stderr(log_level: SrLogLevel) {
    unsafe {
        ffi_sys::sr_log_stderr(log_level as u32);
    }
}

/// Set Log Syslog.
pub fn log_syslog(
    app_name: &str,
    log_level: SrLogLevel,
) -> Result<(), SrError> {
    let app_name = str_to_cstring(app_name)?;
    unsafe {
        ffi_sys::sr_log_syslog(app_name.as_ptr(), log_level as u32);
    }

    Ok(())
}
