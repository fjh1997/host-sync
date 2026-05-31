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

#[no_mangle]
pub extern "C" fn hostsync_has_sync_passphrase() -> i32 {
    if storage::has_sync_passphrase() { 1 } else { 0 }
}

/// # Safety
/// `passphrase` must be a valid, non-null, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hostsync_set_sync_passphrase(passphrase: *const c_char) -> i32 {
    let c_str = unsafe { CStr::from_ptr(passphrase) };
    let pp_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    match storage::save_sync_passphrase(pp_str) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// # Safety
/// `s` must be a pointer previously returned by a `hostsync_*` function, or null.
#[no_mangle]
pub unsafe extern "C" fn hostsync_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

// ---------------------------------------------------------------------------
// Android JNI wrappers – bridge Kotlin `external fun` to the C FFI above.
// ---------------------------------------------------------------------------
#[cfg(target_os = "android")]
mod android_jni {
    use jni::objects::{JClass, JString};
    use jni::sys::jint;
    use jni::JNIEnv;
    use std::ffi::{CStr, CString};

    fn rust_to_java_string(env: &mut JNIEnv, s: *mut std::os::raw::c_char) -> jni::sys::jstring {
        if s.is_null() {
            return std::ptr::null_mut();
        }
        let c_str = unsafe { CStr::from_ptr(s) };
        let java_str = env.new_string(c_str.to_str().unwrap_or("")).unwrap();
        unsafe { super::hostsync_free_string(s) };
        java_str.into_raw()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncLoadServersJson(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jni::sys::jstring {
        let s = super::hostsync_load_servers_json();
        rust_to_java_string(&mut env, s)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncSaveServersJson(
        mut env: JNIEnv,
        _class: JClass,
        json: JString,
    ) -> jint {
        let json_str: String = env.get_string(&json).unwrap().into();
        let c_json = CString::new(json_str).unwrap();
        unsafe { super::hostsync_save_servers_json(c_json.as_ptr()) }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncParseSshConfig(
        mut env: JNIEnv,
        _class: JClass,
        config: JString,
    ) -> jni::sys::jstring {
        let config_str: String = env.get_string(&config).unwrap().into();
        let c_config = CString::new(config_str).unwrap();
        let s = unsafe { super::hostsync_parse_ssh_config(c_config.as_ptr()) };
        rust_to_java_string(&mut env, s)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncGenerateSshConfig(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jni::sys::jstring {
        let s = super::hostsync_generate_ssh_config();
        rust_to_java_string(&mut env, s)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncIsLoggedIn(
        _env: JNIEnv,
        _class: JClass,
    ) -> jint {
        super::hostsync_is_logged_in()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncGetGithubUsername(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jni::sys::jstring {
        let s = super::hostsync_get_github_username();
        rust_to_java_string(&mut env, s)
    }
}
