use libc::c_int;
use std::fmt::{Display, Formatter};
use sysrepo_sys as ffi_sys;

/// Error.
#[derive(Copy, Clone, Debug)]
pub enum SrError {
    Ok = ffi_sys::sr_error_t_SR_ERR_OK as isize,
    InvalArg = ffi_sys::sr_error_t_SR_ERR_INVAL_ARG as isize,
    Ly = ffi_sys::sr_error_t_SR_ERR_LY as isize,
    Sys = ffi_sys::sr_error_t_SR_ERR_SYS as isize,
    NoMemory = ffi_sys::sr_error_t_SR_ERR_NO_MEMORY as isize,
    NotFound = ffi_sys::sr_error_t_SR_ERR_NOT_FOUND as isize,
    Exists = ffi_sys::sr_error_t_SR_ERR_EXISTS as isize,
    Internal = ffi_sys::sr_error_t_SR_ERR_INTERNAL as isize,
    Unsupported = ffi_sys::sr_error_t_SR_ERR_UNSUPPORTED as isize,
    ValidationFailed = ffi_sys::sr_error_t_SR_ERR_VALIDATION_FAILED as isize,
    OperationFailed = ffi_sys::sr_error_t_SR_ERR_OPERATION_FAILED as isize,
    Unauthorized = ffi_sys::sr_error_t_SR_ERR_UNAUTHORIZED as isize,
    Locked = ffi_sys::sr_error_t_SR_ERR_LOCKED as isize,
    TimeOut = ffi_sys::sr_error_t_SR_ERR_TIME_OUT as isize,
    CallbackFailed = ffi_sys::sr_error_t_SR_ERR_CALLBACK_FAILED as isize,
    CallbackShelve = ffi_sys::sr_error_t_SR_ERR_CALLBACK_SHELVE as isize,
}

impl SrError {
    pub fn as_str(&self) -> &'static str {
        match self {
            SrError::Ok => "Ok",
            SrError::InvalArg => "Invalid Arguments",
            SrError::Ly => "Lib Yang",
            SrError::Sys => "Sys",
            SrError::NoMemory => "No Memory",
            SrError::NotFound => "Not Found",
            SrError::Exists => "Exists",
            SrError::Internal => "Internal",
            SrError::Unsupported => "Unsupported",
            SrError::ValidationFailed => "Validation Failed",
            SrError::OperationFailed => "Operation Failed",
            SrError::Unauthorized => "Unauthorized",
            SrError::Locked => "Locked",
            SrError::TimeOut => "Time Out",
            SrError::CallbackFailed => "Callback Failed",
            SrError::CallbackShelve => "Callback Shelve",
        }
    }
}

impl Display for SrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<c_int> for SrError {
    fn from(value: c_int) -> Self {
        match value {
            0 => Self::Ok,
            1 => Self::InvalArg,
            2 => Self::Ly,
            3 => Self::Sys,
            4 => Self::NoMemory,
            5 => Self::NotFound,
            6 => Self::Exists,
            7 => Self::Internal,
            8 => Self::Unsupported,
            9 => Self::ValidationFailed,
            10 => Self::OperationFailed,
            11 => Self::Unauthorized,
            12 => Self::Locked,
            13 => Self::TimeOut,
            14 => Self::CallbackFailed,
            15 => Self::CallbackShelve,
            _ => Self::Internal,
        }
    }
}
