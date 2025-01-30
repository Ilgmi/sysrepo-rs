use std::ffi::{CStr, CString};

use crate::common::str_to_cstring;
use crate::errors::SrError;
use sysrepo_sys as sys_ffi;

#[derive(Debug)]
pub enum Data {
    Binary(String),
    Bits(String),
    Boolean(bool),
    Decimal64(f64),
    Empty,
    Enumeration(String),
    IdentityRef(String),
    InstanceIdentifier(String),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    LeafRef(),
    String(String),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Union(UnionData),
}

#[derive(Debug)]
pub enum UnionData {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    String(String),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Boolean(bool),
    Decimal64(f64),
}

#[derive(Debug, PartialEq, Clone)]
enum ValType {
    Unknown = sys_ffi::sr_val_type_t_SR_UNKNOWN_T as isize,
    List = sys_ffi::sr_val_type_t_SR_LIST_T as isize,
    Container = sys_ffi::sr_val_type_t_SR_CONTAINER_T as isize,
    ContainerPresence = sys_ffi::sr_val_type_t_SR_CONTAINER_PRESENCE_T as isize,
    LeafEmpty = sys_ffi::sr_val_type_t_SR_LEAF_EMPTY_T as isize,
    Notification = sys_ffi::sr_val_type_t_SR_NOTIFICATION_T as isize,
    Binary = sys_ffi::sr_val_type_t_SR_BINARY_T as isize,
    Bits = sys_ffi::sr_val_type_t_SR_BITS_T as isize,
    Bool = sys_ffi::sr_val_type_t_SR_BOOL_T as isize,
    Decimal64 = sys_ffi::sr_val_type_t_SR_DECIMAL64_T as isize,
    Enum = sys_ffi::sr_val_type_t_SR_ENUM_T as isize,
    IdentityRef = sys_ffi::sr_val_type_t_SR_IDENTITYREF_T as isize,
    InstanceId = sys_ffi::sr_val_type_t_SR_INSTANCEID_T as isize,
    Int8 = sys_ffi::sr_val_type_t_SR_INT8_T as isize,
    Int16 = sys_ffi::sr_val_type_t_SR_INT16_T as isize,
    Int32 = sys_ffi::sr_val_type_t_SR_INT32_T as isize,
    Int64 = sys_ffi::sr_val_type_t_SR_INT64_T as isize,
    String = sys_ffi::sr_val_type_t_SR_STRING_T as isize,
    Uint8 = sys_ffi::sr_val_type_t_SR_UINT8_T as isize,
    Uint16 = sys_ffi::sr_val_type_t_SR_UINT16_T as isize,
    Uint32 = sys_ffi::sr_val_type_t_SR_UINT32_T as isize,
    Uint64 = sys_ffi::sr_val_type_t_SR_UINT64_T as isize,
    AnyXML = sys_ffi::sr_val_type_t_SR_ANYXML_T as isize,
    AnyData = sys_ffi::sr_val_type_t_SR_ANYDATA_T as isize,
}

impl From<sys_ffi::sr_val_type_t> for ValType {
    fn from(value: sys_ffi::sr_val_type_t) -> Self {
        match value {
            sys_ffi::sr_val_type_t_SR_UNKNOWN_T => Self::Unknown,
            sys_ffi::sr_val_type_t_SR_LIST_T => Self::List,
            sys_ffi::sr_val_type_t_SR_CONTAINER_T => Self::Container,
            sys_ffi::sr_val_type_t_SR_CONTAINER_PRESENCE_T => Self::ContainerPresence,
            sys_ffi::sr_val_type_t_SR_LEAF_EMPTY_T => Self::LeafEmpty,
            sys_ffi::sr_val_type_t_SR_NOTIFICATION_T => Self::Notification,
            sys_ffi::sr_val_type_t_SR_BINARY_T => Self::Binary,
            sys_ffi::sr_val_type_t_SR_BITS_T => Self::Bits,
            sys_ffi::sr_val_type_t_SR_BOOL_T => Self::Bool,
            sys_ffi::sr_val_type_t_SR_DECIMAL64_T => Self::Decimal64,
            sys_ffi::sr_val_type_t_SR_ENUM_T => Self::Enum,
            sys_ffi::sr_val_type_t_SR_IDENTITYREF_T => Self::IdentityRef,
            sys_ffi::sr_val_type_t_SR_INSTANCEID_T => Self::InstanceId,
            sys_ffi::sr_val_type_t_SR_INT8_T => Self::Int8,
            sys_ffi::sr_val_type_t_SR_INT16_T => Self::Int16,
            sys_ffi::sr_val_type_t_SR_INT32_T => Self::Int32,
            sys_ffi::sr_val_type_t_SR_INT64_T => Self::Int64,
            sys_ffi::sr_val_type_t_SR_STRING_T => Self::String,
            sys_ffi::sr_val_type_t_SR_UINT8_T => Self::Uint8,
            sys_ffi::sr_val_type_t_SR_UINT16_T => Self::Uint16,
            sys_ffi::sr_val_type_t_SR_UINT32_T => Self::Uint32,
            sys_ffi::sr_val_type_t_SR_UINT64_T => Self::Uint64,
            sys_ffi::sr_val_type_t_SR_ANYXML_T => Self::AnyXML,
            sys_ffi::sr_val_type_t_SR_ANYDATA_T => Self::AnyData,
            _ => Self::Unknown,
        }
    }
}

impl From<&Data> for ValType {
    fn from(value: &Data) -> Self {
        match value {
            Data::Binary(_) => Self::Binary,
            Data::Bits(_) => Self::Bits,
            Data::Boolean(_) => Self::Bool,
            Data::Decimal64(_) => Self::Decimal64,
            Data::Empty => Self::Unknown,
            Data::Enumeration(_) => Self::Enum,
            Data::IdentityRef(_) => Self::IdentityRef,
            Data::InstanceIdentifier(_) => Self::InstanceId,
            Data::Int8(_) => Self::Int8,
            Data::Int16(_) => Self::Int16,
            Data::Int32(_) => Self::Int32,
            Data::Int64(_) => Self::Int64,
            Data::LeafRef() => Self::Unknown,
            Data::String(_) => Self::String,
            Data::UInt8(_) => Self::Uint8,
            Data::UInt16(_) => Self::Uint16,
            Data::UInt32(_) => Self::Uint32,
            Data::UInt64(_) => Self::Uint64,
            Data::Union(_) => Self::Unknown,
        }
    }
}

/// Single Sysrepo Value.
#[derive(Debug)]
pub struct SrValue {
    sr_value: *mut sys_ffi::sr_val_t,
    data: Data,
    val_type: ValType,
    owned: bool,
}

impl SrValue {
    pub fn from(value: *mut sys_ffi::sr_val_t, owned: bool) -> Self {
        let val_type: ValType = unsafe { (*value).type_.into() };
        let data = match val_type {
            ValType::Unknown => Data::String(String::from("(sr_val_type_t_SR_UNKNOWN_T instance)")),
            ValType::List => Data::String(String::from("(sr_val_type_t_SR_LIST_T instance)")),
            ValType::Container => {
                Data::String(String::from("(sr_val_type_t_SR_CONTAINER_T instance)"))
            }
            ValType::ContainerPresence => Data::String(String::from(
                "(sr_val_type_t_SR_CONTAINER_PRESENCE_T instance)",
            )),
            ValType::LeafEmpty => Data::Empty,
            ValType::Notification => {
                Data::String(String::from("(sr_val_type_t_SR_NOTIFICATION_T instance)"))
            }
            ValType::Binary => {
                let binary_val = unsafe { CStr::from_ptr((*value).data.binary_val) };
                Data::Binary(binary_val.to_string_lossy().to_string())
            }
            ValType::Bits => {
                let bits_val = unsafe { CStr::from_ptr((*value).data.bits_val) };
                Data::Bits(bits_val.to_string_lossy().to_string())
            }
            ValType::Bool => {
                let bool_val = unsafe { (*value).data.bool_val };
                Data::Boolean(bool_val == 1)
            }
            ValType::Decimal64 => {
                let decimal_val = unsafe { (*value).data.decimal64_val };
                Data::Decimal64(decimal_val)
            }
            ValType::Enum => {
                let enum_val = unsafe { CStr::from_ptr((*value).data.enum_val) };
                Data::Enumeration(enum_val.to_str().unwrap().to_string())
            }
            ValType::IdentityRef => {
                let identityref_val = unsafe { CStr::from_ptr((*value).data.identityref_val) };
                Data::InstanceIdentifier(identityref_val.to_str().unwrap().to_string())
            }
            ValType::InstanceId => {
                let instanceid_val = unsafe { CStr::from_ptr((*value).data.instanceid_val) };
                Data::InstanceIdentifier(instanceid_val.to_str().unwrap().to_string())
            }
            ValType::Int8 => {
                let int_8 = unsafe { (*value).data.int8_val };
                Data::Int8(int_8)
            }
            ValType::Int16 => {
                let int_16 = unsafe { (*value).data.int16_val };
                Data::Int16(int_16)
            }
            ValType::Int32 => {
                let int_32 = unsafe { (*value).data.int32_val };
                Data::Int32(int_32)
            }
            ValType::Int64 => {
                let int_64 = unsafe { (*value).data.int64_val };
                Data::Int64(int_64)
            }
            ValType::String => {
                let string_val = unsafe { CStr::from_ptr((*value).data.string_val) };
                Data::String(string_val.to_string_lossy().to_string())
            }
            ValType::Uint8 => {
                let uint_8 = unsafe { (*value).data.uint8_val };
                Data::UInt8(uint_8)
            }
            ValType::Uint16 => {
                let uint_16 = unsafe { (*value).data.uint16_val };
                Data::UInt16(uint_16)
            }
            ValType::Uint32 => {
                let uint_32 = unsafe { (*value).data.uint32_val };
                Data::UInt32(uint_32)
            }
            ValType::Uint64 => {
                let uint_64 = unsafe { (*value).data.uint64_val };
                Data::UInt64(uint_64)
            }
            ValType::AnyXML => Data::String(String::from("(sr_val_type_t_SR_ANYXML_T instance)")),
            ValType::AnyData => Data::String(String::from("(sr_val_type_t_SR_ANYDATA_T instance)")),
        };

        Self {
            sr_value: value,
            data,
            val_type,
            owned,
        }
    }

    pub fn new(
        val: *mut sys_ffi::sr_val_t,
        xpath: String,
        data: Data,
        dflt: bool,
        owned: bool,
    ) -> Result<Self, SrError> {
        let xpath = str_to_cstring(&xpath)?;
        let xpath_ptr: *mut std::os::raw::c_char = xpath.into_raw();
        let val_type = ValType::from(&data);

        unsafe {
            (*val).xpath = xpath_ptr;
            (*val).dflt = if dflt { 1 } else { 0 };
            match &data {
                Data::Binary(data) => {
                    let data = str_to_cstring(data)?;
                    let data_ptr: *mut std::os::raw::c_char = data.into_raw();
                    (*val).data.binary_val = data_ptr;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_BINARY_T;
                }
                Data::Bits(data) => {
                    let data = str_to_cstring(data)?;
                    let data_ptr: *mut std::os::raw::c_char = data.into_raw();
                    (*val).data.bits_val = data_ptr;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_BITS_T;
                }
                Data::Boolean(data) => {
                    (*val).data.bool_val = if *data { 1 } else { 0 };
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_BOOL_T;
                }
                Data::Decimal64(data) => {
                    (*val).data.decimal64_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_DECIMAL64_T;
                }
                Data::Empty => {
                    (*val).data.string_val = std::ptr::null_mut();
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_STRING_T;
                }
                Data::Enumeration(_data) => {}
                Data::IdentityRef(_) => {}
                Data::InstanceIdentifier(_) => {}
                Data::Int8(data) => {
                    (*val).data.int8_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_INT8_T;
                }
                Data::Int16(data) => {
                    (*val).data.int16_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_INT16_T;
                }
                Data::Int32(data) => {
                    (*val).data.int32_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_INT32_T;
                }
                Data::Int64(data) => {
                    (*val).data.int64_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_INT64_T;
                }
                Data::LeafRef() => {}
                Data::String(data) => {
                    let data = str_to_cstring(data)?;
                    let data_ptr: *mut std::os::raw::c_char = data.into_raw();
                    (*val).data.bits_val = data_ptr;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_STRING_T;
                }
                Data::UInt8(data) => {
                    (*val).data.uint8_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_UINT8_T;
                }
                Data::UInt16(data) => {
                    (*val).data.uint16_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_UINT16_T;
                }
                Data::UInt32(data) => {
                    (*val).data.uint32_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_UINT32_T;
                }
                Data::UInt64(data) => {
                    (*val).data.uint64_val = *data;
                    (*val).type_ = sys_ffi::sr_val_type_t_SR_UINT64_T;
                }
                Data::Union(_data) => {}
            }
        }

        Ok(Self {
            sr_value: val,
            val_type,
            data,
            owned,
        })
    }
}

impl SrValue {
    pub fn data(&self) -> &Data {
        &self.data
    }

    pub fn xpath(&self) -> String {
        let xpath = unsafe { CString::from_raw((*self.sr_value).xpath) };
        String::from(xpath.to_string_lossy())
    }

    pub fn as_raw(&self) -> *mut sys_ffi::sr_val_t {
        self.sr_value
    }
}

impl Drop for SrValue {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                sys_ffi::sr_free_val(self.sr_value);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::os::raw::c_char;

    #[test]
    fn create_sr_value() {
        let xpath = String::from("/test/test");
        let type_ = sys_ffi::sr_val_type_t_SR_STRING_T;
        let data = String::from("test");
        let path = string_to_mut_c_char(&xpath).unwrap();
        let data: *mut c_char = string_to_mut_c_char(&data).unwrap();
        let data = sys_ffi::sr_val_data_t { string_val: data };
        let mut test_val_t = sys_ffi::sr_val_t {
            xpath: path,
            type_,
            dflt: 0,
            origin: path,
            data,
        };
        let test_val_t = &mut test_val_t as *mut sys_ffi::sr_val_t;

        let value = SrValue::from(test_val_t, false);
        assert_eq!(value.xpath(), xpath);
        assert_eq!(value.val_type, ValType::String);
    }

    fn string_to_mut_c_char(s: &str) -> Result<*mut std::os::raw::c_char, std::ffi::NulError> {
        let c_string = CString::new(s)?;
        Ok(c_string.into_raw())
    }
}
