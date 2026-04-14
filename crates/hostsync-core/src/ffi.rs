/// C FFI exports for mobile platforms (Android JNI / iOS Swift).
/// All functions use C-compatible types and return heap-allocated strings
/// that the caller must free with `hostsync_free_string`.
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::{model::Server, ssh_config, storage};

#[no_mangle]
pub extern "C" fn hostsync_load_servers_json() -> *mut c_char {
    let servers = storage::load_servers();
    let json = serde_json::to_string(&servers).unwrap_or_else(|_| "[]".to_string());
    CString::new(json).unwrap().into_raw()
}

/// # Safety
/// `json` must be a valid, non-null, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hostsync_save_servers_json(json: *const c_char) -> i32 {
    let c_str = unsafe { CStr::from_ptr(json) };
    let json_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let servers: Vec<Server> = match serde_json::from_str(json_str) {
        Ok(s) => s,
        Err(_) => return -2,
    };
    match storage::save_servers(&servers) {
        Ok(_) => 0,
        Err(_) => -3,
    }
}

/// # Safety
/// `config` must be a valid, non-null, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hostsync_parse_ssh_config(config: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(config) };
    let config_str = c_str.to_str().unwrap_or("");
    let servers = ssh_config::parse(config_str);
    let json = serde_json::to_string(&servers).unwrap_or_else(|_| "[]".to_string());
    CString::new(json).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn hostsync_generate_ssh_config() -> *mut c_char {
    let servers = storage::load_servers();
    let config = ssh_config::generate(&servers);
    CString::new(config).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn hostsync_is_logged_in() -> i32 {
    if storage::is_logged_in() { 1 } else { 0 }
}

#[no_mangle]
pub extern "C" fn hostsync_get_github_username() -> *mut c_char {
    let state = storage::load_github_state();
    let name = state.username.unwrap_or_default();
    CString::new(name).unwrap().into_raw()
}

/// # Safety
/// `s` must be a pointer previously returned by a `hostsync_*` function, or null.
#[no_mangle]
pub unsafe extern "C" fn hostsync_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}
