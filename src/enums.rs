use sysrepo_sys as ffi_sys;

/// Log level.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrLogLevel {
    None = ffi_sys::sr_log_level_t_SR_LL_NONE as isize,
    Error = ffi_sys::sr_log_level_t_SR_LL_ERR as isize,
    Warn = ffi_sys::sr_log_level_t_SR_LL_WRN as isize,
    Info = ffi_sys::sr_log_level_t_SR_LL_INF as isize,
    Debug = ffi_sys::sr_log_level_t_SR_LL_DBG as isize,
}

/// Conn Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrConnFlag {
    Default = ffi_sys::sr_conn_flag_t_SR_CONN_DEFAULT as isize,
    CacheRunning = ffi_sys::sr_conn_flag_t_SR_CONN_CACHE_RUNNING as isize,
}

/// Datastore.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrDatastore {
    Startup = ffi_sys::sr_datastore_t_SR_DS_STARTUP as isize,
    Running = ffi_sys::sr_datastore_t_SR_DS_RUNNING as isize,
    Candidate = ffi_sys::sr_datastore_t_SR_DS_CANDIDATE as isize,
    Operational = ffi_sys::sr_datastore_t_SR_DS_OPERATIONAL as isize,
}

/// Get Oper Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrGetOperFlag {
    Default = ffi_sys::sr_get_oper_flag_t_SR_OPER_DEFAULT as isize,
    NoState = ffi_sys::sr_get_oper_flag_t_SR_OPER_NO_STATE as isize,
    NoConfig = ffi_sys::sr_get_oper_flag_t_SR_OPER_NO_CONFIG as isize,
    NoSubs = ffi_sys::sr_get_oper_flag_t_SR_OPER_NO_SUBS as isize,
    NoStored = ffi_sys::sr_get_oper_flag_t_SR_OPER_NO_STORED as isize,
    WithOrigin = ffi_sys::sr_get_oper_flag_t_SR_OPER_WITH_ORIGIN as isize,
}

/// Edit Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrEditFlag {
    Default = ffi_sys::sr_edit_flag_t_SR_EDIT_DEFAULT as isize,
    NonRecursive = ffi_sys::sr_edit_flag_t_SR_EDIT_NON_RECURSIVE as isize,
    Strict = ffi_sys::sr_edit_flag_t_SR_EDIT_STRICT as isize,
    Isolate = ffi_sys::sr_edit_flag_t_SR_EDIT_ISOLATE as isize,
}

/// Move Position.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrMovePosition {
    Before = ffi_sys::sr_move_position_t_SR_MOVE_BEFORE as isize,
    After = ffi_sys::sr_move_position_t_SR_MOVE_AFTER as isize,
    First = ffi_sys::sr_move_position_t_SR_MOVE_FIRST as isize,
    Last = ffi_sys::sr_move_position_t_SR_MOVE_LAST as isize,
}

/// Subscribe Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrSubcribeFlag {
    Default = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_DEFAULT as isize,
    NoThread = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_NO_THREAD as isize,
    Passive = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_PASSIVE as isize,
    DoneOnly = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_DONE_ONLY as isize,
    Enabled = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_ENABLED as isize,
    Update = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_UPDATE as isize,
    OperMerge = ffi_sys::sr_subscr_flag_t_SR_SUBSCR_OPER_MERGE as isize,
}

/// Notification Type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrNotifType {
    Realtime = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_REALTIME as isize,
    Replay = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY as isize,
    ReplayComplete = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY_COMPLETE as isize,
    Terminated = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_TERMINATED as isize,
    Modified = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_MODIFIED as isize,
    Suspended = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_SUSPENDED as isize,
    Resumed = ffi_sys::sr_ev_notif_type_t_SR_EV_NOTIF_RESUMED as isize,
}

impl TryFrom<u32> for SrNotifType {
    type Error = &'static str;

    fn try_from(t: u32) -> Result<Self, Self::Error> {
        match t {
            sr_ev_notif_type_t_SR_EV_NOTIF_REALTIME => Ok(SrNotifType::Realtime),
            sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY => Ok(SrNotifType::Replay),
            sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY_COMPLETE => Ok(SrNotifType::ReplayComplete),
            sr_ev_notif_type_t_SR_EV_NOTIF_TERMINATED => Ok(SrNotifType::Terminated),
            sr_ev_notif_type_t_SR_EV_NOTIF_MODIFIED => Ok(SrNotifType::Modified),
            sr_ev_notif_type_t_SR_EV_NOTIF_SUSPENDED => Ok(SrNotifType::Suspended),
            sr_ev_notif_type_t_SR_EV_NOTIF_RESUMED => Ok(SrNotifType::Resumed),
            _ => Err("Invalid SrNotifType"),
        }
    }
}

/// Lyd Anydata Value Type.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum LydAnyDataValueType {
    String = ffi_sys::LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_STRING as isize,
    Json = ffi_sys::LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_JSON as isize,
    Xml = ffi_sys::LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_XML as isize,
    Datatree = ffi_sys::LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_DATATREE as isize,
    Lyb = ffi_sys::LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_LYB as isize,
}

pub enum DefaultOperation {
    Merge,
    Replace,
    None,
}

impl DefaultOperation {
    pub fn as_str(&self) -> &'static str {
        match self {
            DefaultOperation::Merge => "merge",
            DefaultOperation::Replace => "replace",
            DefaultOperation::None => "none",
        }
    }
}
