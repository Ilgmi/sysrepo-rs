use crate::value::{Data, SrValue};

use crate::errors::SrError;
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{sr_free_values, sr_val_t};

pub struct SrValues {
    raw_values: *mut ffi_sys::sr_val_t,
    len: usize,
    owned: bool,
}

impl SrValues {
    pub fn new(size: usize, owned: bool) -> Self {
        let raw_values = unsafe { libc::malloc(std::mem::size_of::<ffi_sys::sr_val_t>() * size) }
            as *mut ffi_sys::sr_val_t;
        Self {
            raw_values,
            len: size,
            owned,
        }
    }

    pub fn from_raw(values: *mut ffi_sys::sr_val_t, size: usize, owned: bool) -> Self {
        Self {
            raw_values: values,
            len: size,
            owned,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn add_value(
        &mut self,
        index: usize,
        xpath: String,
        data: Data,
        dflt: bool,
    ) -> Result<(), SrError> {
        if index >= self.len {
            return Err(SrError::Internal);
        }

        let mut values = unsafe { std::slice::from_raw_parts_mut(self.raw_values, self.len) };
        let mut value = values[index];
        let val = SrValue::new(&mut value, xpath, data, dflt, false)?;
        unsafe {
            let _ = std::mem::replace(&mut values[index], *val.as_raw());
        }
        Ok(())
    }

    pub fn get_value_mut(&self, index: usize) -> Result<SrValue, SrError> {
        if index >= self.len {
            return Err(SrError::Internal);
        }

        let mut values = unsafe { std::slice::from_raw_parts_mut(self.raw_values, self.len) };
        let value = SrValue::from(&mut values[index], false);
        Ok(value)
    }

    pub fn as_raw(&self) -> (*mut ffi_sys::sr_val_t, usize) {
        (self.raw_values, self.len)
    }

    pub fn as_raw_slice(&self) -> &[sr_val_t] {
        unsafe { std::slice::from_raw_parts(self.raw_values, self.len) }
    }
}

impl Drop for SrValues {
    fn drop(&mut self) {
        if self.owned {
            unsafe { sr_free_values(self.raw_values, self.len) }
        }
    }
}
