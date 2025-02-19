use sysrepo_sys::{
    sr_conn_flag_t_SR_CONN_CACHE_RUNNING, sr_conn_flag_t_SR_CONN_DEFAULT,
    sr_datastore_t_SR_DS_CANDIDATE, sr_datastore_t_SR_DS_OPERATIONAL, sr_datastore_t_SR_DS_RUNNING,
    sr_datastore_t_SR_DS_STARTUP, sr_edit_flag_t_SR_EDIT_DEFAULT, sr_edit_flag_t_SR_EDIT_ISOLATE,
    sr_edit_flag_t_SR_EDIT_NON_RECURSIVE, sr_edit_flag_t_SR_EDIT_STRICT,
    sr_ev_notif_type_t_SR_EV_NOTIF_MODIFIED, sr_ev_notif_type_t_SR_EV_NOTIF_REALTIME,
    sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY, sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY_COMPLETE,
    sr_ev_notif_type_t_SR_EV_NOTIF_RESUMED, sr_ev_notif_type_t_SR_EV_NOTIF_SUSPENDED,
    sr_ev_notif_type_t_SR_EV_NOTIF_TERMINATED, sr_get_oper_flag_t_SR_OPER_DEFAULT,
    sr_get_oper_flag_t_SR_OPER_NO_CONFIG, sr_get_oper_flag_t_SR_OPER_NO_STATE,
    sr_get_oper_flag_t_SR_OPER_NO_STORED, sr_get_oper_flag_t_SR_OPER_NO_SUBS,
    sr_get_oper_flag_t_SR_OPER_WITH_ORIGIN, sr_log_level_t_SR_LL_DBG, sr_log_level_t_SR_LL_ERR,
    sr_log_level_t_SR_LL_INF, sr_log_level_t_SR_LL_NONE, sr_log_level_t_SR_LL_WRN,
    sr_move_position_t_SR_MOVE_AFTER, sr_move_position_t_SR_MOVE_BEFORE,
    sr_move_position_t_SR_MOVE_FIRST, sr_move_position_t_SR_MOVE_LAST,
    sr_subscr_flag_t_SR_SUBSCR_DEFAULT, sr_subscr_flag_t_SR_SUBSCR_DONE_ONLY,
    sr_subscr_flag_t_SR_SUBSCR_ENABLED, sr_subscr_flag_t_SR_SUBSCR_NO_THREAD,
    sr_subscr_flag_t_SR_SUBSCR_OPER_MERGE, sr_subscr_flag_t_SR_SUBSCR_PASSIVE,
    sr_subscr_flag_t_SR_SUBSCR_UPDATE, LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_DATATREE,
    LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_JSON, LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_LYB,
    LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_STRING, LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_XML,
};

/// Log level.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrLogLevel {
    None = sr_log_level_t_SR_LL_NONE as isize,
    Error = sr_log_level_t_SR_LL_ERR as isize,
    Warn = sr_log_level_t_SR_LL_WRN as isize,
    Info = sr_log_level_t_SR_LL_INF as isize,
    Debug = sr_log_level_t_SR_LL_DBG as isize,
}

/// Conn Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrConnFlag {
    Default = sr_conn_flag_t_SR_CONN_DEFAULT as isize,
    CacheRunning = sr_conn_flag_t_SR_CONN_CACHE_RUNNING as isize,
}

/// Datastore.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrDatastore {
    Startup = sr_datastore_t_SR_DS_STARTUP as isize,
    Running = sr_datastore_t_SR_DS_RUNNING as isize,
    Candidate = sr_datastore_t_SR_DS_CANDIDATE as isize,
    Operational = sr_datastore_t_SR_DS_OPERATIONAL as isize,
}

/// Get Oper Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrGetOperFlag {
    Default = sr_get_oper_flag_t_SR_OPER_DEFAULT as isize,
    NoState = sr_get_oper_flag_t_SR_OPER_NO_STATE as isize,
    NoConfig = sr_get_oper_flag_t_SR_OPER_NO_CONFIG as isize,
    NoSubs = sr_get_oper_flag_t_SR_OPER_NO_SUBS as isize,
    NoStored = sr_get_oper_flag_t_SR_OPER_NO_STORED as isize,
    WithOrigin = sr_get_oper_flag_t_SR_OPER_WITH_ORIGIN as isize,
}

/// Edit Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrEditFlag {
    Default = sr_edit_flag_t_SR_EDIT_DEFAULT as isize,
    NonRecursive = sr_edit_flag_t_SR_EDIT_NON_RECURSIVE as isize,
    Strict = sr_edit_flag_t_SR_EDIT_STRICT as isize,
    Isolate = sr_edit_flag_t_SR_EDIT_ISOLATE as isize,
}

/// Move Position.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrMovePosition {
    Before = sr_move_position_t_SR_MOVE_BEFORE as isize,
    After = sr_move_position_t_SR_MOVE_AFTER as isize,
    First = sr_move_position_t_SR_MOVE_FIRST as isize,
    Last = sr_move_position_t_SR_MOVE_LAST as isize,
}

/// Subscribe Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrSubcribeFlag {
    Default = sr_subscr_flag_t_SR_SUBSCR_DEFAULT as isize,
    NoThread = sr_subscr_flag_t_SR_SUBSCR_NO_THREAD as isize,
    Passive = sr_subscr_flag_t_SR_SUBSCR_PASSIVE as isize,
    DoneOnly = sr_subscr_flag_t_SR_SUBSCR_DONE_ONLY as isize,
    Enabled = sr_subscr_flag_t_SR_SUBSCR_ENABLED as isize,
    Update = sr_subscr_flag_t_SR_SUBSCR_UPDATE as isize,
    OperMerge = sr_subscr_flag_t_SR_SUBSCR_OPER_MERGE as isize,
}

/// Notification Type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SrNotifType {
    Realtime = sr_ev_notif_type_t_SR_EV_NOTIF_REALTIME as isize,
    Replay = sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY as isize,
    ReplayComplete = sr_ev_notif_type_t_SR_EV_NOTIF_REPLAY_COMPLETE as isize,
    Terminated = sr_ev_notif_type_t_SR_EV_NOTIF_TERMINATED as isize,
    Modified = sr_ev_notif_type_t_SR_EV_NOTIF_MODIFIED as isize,
    Suspended = sr_ev_notif_type_t_SR_EV_NOTIF_SUSPENDED as isize,
    Resumed = sr_ev_notif_type_t_SR_EV_NOTIF_RESUMED as isize,
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
    String = LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_STRING as isize,
    Json = LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_JSON as isize,
    Xml = LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_XML as isize,
    Datatree = LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_DATATREE as isize,
    Lyb = LYD_ANYDATA_VALUETYPE_LYD_ANYDATA_LYB as isize,
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
