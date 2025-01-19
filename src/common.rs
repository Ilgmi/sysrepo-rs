use crate::errors::SrError;
use std::ffi::CString;

pub fn str_to_cstring(s: &str) -> Result<CString, SrError> {
    CString::new(s).map_err(|_| SrError::InvalArg)
}
