use crate::errors::SrError;
use libc::strdup;
use std::ffi::CString;
use std::os::raw::c_char;

pub fn str_to_cstring(s: &str) -> Result<CString, SrError> {
    CString::new(s).map_err(|_| SrError::InvalArg)
}

pub fn dup_str(s: &str) -> Result<*mut c_char, SrError> {
    let s = unsafe { strdup(str_to_cstring(s)?.as_ptr()) };
    Ok(s)
}
