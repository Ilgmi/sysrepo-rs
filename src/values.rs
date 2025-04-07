use crate::value::{Data, SrValue};

use crate::errors::SrError;
use sysrepo_sys as ffi_sys;

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

        let values = unsafe { std::slice::from_raw_parts_mut(self.raw_values, self.len) };
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

        let values = unsafe { std::slice::from_raw_parts_mut(self.raw_values, self.len) };
        let value = SrValue::from(&mut values[index], false);
        Ok(value)
    }

    pub fn as_raw(&self) -> (*mut ffi_sys::sr_val_t, usize) {
        (self.raw_values, self.len)
    }

    pub fn as_raw_slice(&self) -> &[ffi_sys::sr_val_t] {
        unsafe { std::slice::from_raw_parts(self.raw_values, self.len) }
    }
}

impl Drop for SrValues {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ffi_sys::sr_free_values(self.raw_values, self.len) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_values_and_add_some_value_successful() {
        let mut values = SrValues::new(1, false);
        let r = values.add_value(0, "".to_string(), Data::String("test".to_string()), false);
        assert_eq!(values.len(), 1);
        assert!(r.is_ok());
    }

    #[test]
    fn get_value_successful() {
        let expected_path = String::from("/examples:example");
        let expected_value = String::from("test");

        let mut values = SrValues::new(1, false);
        let _r = values.add_value(
            0,
            expected_path.clone(),
            Data::String(expected_value.clone()),
            false,
        );

        let value = values.get_value_mut(0);
        assert!(value.is_ok());
        let value = value.unwrap();
        assert_eq!(&value.xpath(), &expected_path);
        match value.data() {
            Data::String(data) => {
                assert_eq!(data, &expected_value);
            }
            _ => panic!("Expected a string data, got {:?}", value),
        }
    }
}
