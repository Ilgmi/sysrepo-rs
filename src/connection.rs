use crate::enums::SrDatastore;
use crate::errors::SrError;
use crate::session::{SrSession, SrSessionId};
use std::collections::HashMap;
use sysrepo_sys as ffi_sys;
use sysrepo_sys::{sr_connect, sr_disconnect, sr_session_start};

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

    pub fn start_session(&mut self, ds: SrDatastore) -> Result<&mut SrSession, i32> {
        let mut sess = std::ptr::null_mut();
        let rc = unsafe { sr_session_start(self.raw_connection, ds as u32, &mut sess) };
        if rc != SrError::Ok as i32 {
            Err(rc)
        } else {
            let id = sess;
            self.insert_session(id, SrSession::from(sess, true));
            Ok(self.sessions.get_mut(&(id as SrSessionId)).unwrap())
        }
    }
}
