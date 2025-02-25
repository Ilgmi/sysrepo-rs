use crate::common::dup_str;
use crate::enums::SrDatastore;
use crate::errors::SrError;
use crate::session::{SrSession, SrSessionId};
use libc::c_int;
use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::os::raw::c_char;
use std::path::Path;
use std::ptr;
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{
    sr_acquire_context, sr_connect, sr_disconnect, sr_install_module, sr_remove_module,
    sr_session_start,
};
use yang3::context::Context;
use yang3::utils::Binding;

pub enum ConnectionOptions {
    Datastore_StartUp = ffi_sys::sr_datastore_t_SR_DS_STARTUP as isize,
    Datastore_Running = ffi_sys::sr_datastore_t_SR_DS_RUNNING as isize,
    Datastore_Candidate = ffi_sys::sr_datastore_t_SR_DS_CANDIDATE as isize,
    Datastore_Operational = ffi_sys::sr_datastore_t_SR_DS_OPERATIONAL as isize,
    Datastore_Factory_Default = ffi_sys::sr_datastore_t_SR_DS_FACTORY_DEFAULT as isize,
}

pub struct SrConnection {
    raw_connection: *mut ffi_sys::sr_conn_ctx_t,
    sessions: HashMap<SrSessionId, SrSession>,
}

unsafe impl Send for SrConnection {}
unsafe impl Sync for SrConnection {}

impl SrConnection {
    pub fn new(options: ConnectionOptions) -> Result<Self, SrError> {
        let mut conn = std::ptr::null_mut();
        let options = options as ffi_sys::sr_conn_options_t;

        let rc = unsafe { sr_connect(options, &mut conn) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            Ok(Self {
                raw_connection: conn,
                sessions: HashMap::new(),
            })
        }
    }

    /// Disconnect.
    pub fn disconnect(&mut self) {
        unsafe {
            sr_disconnect(self.raw_connection);
        }
        self.raw_connection = std::ptr::null_mut();
    }

    /// Add session to map.
    pub fn insert_session(&mut self, id: SrSessionId, sess: SrSession) {
        self.sessions.insert(id, sess);
    }

    /// Add session to map.
    pub fn remove_session(&mut self, id: &SrSessionId) {
        self.sessions.remove(id);
    }

    /// Lookup session from map.
    pub fn lookup_session(&mut self, id: &SrSessionId) -> Option<&mut SrSession> {
        self.sessions.get_mut(id)
    }

    pub fn start_session(&mut self, ds: SrDatastore) -> Result<&mut SrSession, SrError> {
        let mut sess = std::ptr::null_mut();
        let rc = unsafe { sr_session_start(self.raw_connection, ds as u32, &mut sess) };
        if rc != SrError::Ok as i32 {
            Err(SrError::from(rc))
        } else {
            let id = sess;
            self.insert_session(id, SrSession::from(sess, true));
            Ok(self.sessions.get_mut(&(id as SrSessionId)).unwrap())
        }
    }

    /// Returns the libyang3 context associated with this Session
    pub fn get_context(&self) -> ManuallyDrop<Context> {
        let ctx = unsafe {
            let ctx = sr_acquire_context(self.raw_connection) as *mut libyang3_sys::ly_ctx;
            Context::from_raw(&(), ctx)
        };
        ManuallyDrop::new(ctx)
    }

    pub fn install_module(
        &self,
        file: &Path,
        search_dirs: Option<&str>,
        features: Option<&[&str]>,
    ) -> Result<(), SrError> {
        let path = match file.to_str() {
            None => return Err(SrError::NotFound),
            Some(path) => match dup_str(path) {
                Ok(path) => path,
                Err(_) => return Err(SrError::InvalArg),
            },
        };

        let search_dirs = match search_dirs {
            None => ptr::null(),
            Some(dirs) => match dup_str(dirs) {
                Ok(dirs) => dirs,
                Err(_) => return Err(SrError::InvalArg),
            },
        };

        let features = match features {
            None => ptr::null_mut(),
            Some(features) => {
                let mut f = Vec::new();
                for feature in features {
                    match dup_str(feature) {
                        Ok(feature) => {
                            f.push(feature as *const c_char);
                        }
                        Err(_) => {}
                    }
                }
                f.as_mut_ptr()
            }
        };

        let ret = unsafe { sr_install_module(self.raw_connection, path, search_dirs, features) };

        if ret != SrError::Ok as i32 {
            return Err(SrError::from(ret));
        }

        Ok(())
    }

    pub fn remove_module(&self, file: &Path, force: bool) -> Result<(), SrError> {
        let path = match file.to_str() {
            None => return Err(SrError::NotFound),
            Some(path) => match dup_str(path) {
                Ok(path) => path,
                Err(_) => return Err(SrError::InvalArg),
            },
        };

        let force = match force {
            true => 1 as c_int,
            false => 0 as c_int,
        };

        let ret = unsafe { sr_remove_module(self.raw_connection, path, force) };

        if ret != SrError::Ok as i32 {
            return Err(SrError::from(ret));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_new_connection_successful() {
        let c = SrConnection::new(ConnectionOptions::Datastore_Running);
        assert!(c.is_ok());
    }

    #[test]
    fn creat_new_session_successful() {
        let c = SrConnection::new(ConnectionOptions::Datastore_Running);
        assert!(c.is_ok());
        let mut c = c.unwrap();
        let session = c.start_session(SrDatastore::Running);
        assert!(session.is_ok());
    }
}
