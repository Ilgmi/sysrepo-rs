use crate::value::SrValue;

use sysrepo_sys as ffi_sys;
use sysrepo_sys::sr_val_t;

pub struct SrValues {
    values: Vec<SrValue>,
    raw_values: *mut ffi_sys::sr_val_t,
    owned: bool,
}

impl SrValues {
    pub fn new(size: usize, owned: bool) -> Self {
        let raw_values = unsafe { libc::malloc(std::mem::size_of::<ffi_sys::sr_val_t>() * size) }
            as *mut ffi_sys::sr_val_t;
        let values = Vec::with_capacity(size);
        Self {
            raw_values,
            values,
            owned,
        }
    }

    pub fn from_raw(values: *mut ffi_sys::sr_val_t, len: usize, owned: bool) -> Self {
        let mut own_values = Vec::new();
        let vals = unsafe { std::slice::from_raw_parts_mut(values, len) };
        for val in vals {
            let v = SrValue::from(val, false);
            own_values.push(v);
        }
        Self {
            values: own_values,
            raw_values: values,
            owned,
        }
    }

    pub fn as_raw(&self) -> (*mut ffi_sys::sr_val_t, usize) {
        (self.raw_values, self.values.len())
    }

    pub fn as_raw_slice(&self) -> &[sr_val_t] {
        unsafe { std::slice::from_raw_parts(self.raw_values, self.values.len()) }
    }

    pub fn as_slice(&self) -> &[SrValue] {
        self.values.as_slice()
    }
}
