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

/// Request a GitHub Device Flow device code. Returns JSON string.
#[no_mangle]
pub extern "C" fn hostsync_request_device_code() -> *mut c_char {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(crate::auth::request_device_code());
    match result {
        Ok(dc) => {
            let json = serde_json::json!({
                "user_code": dc.user_code,
                "verification_uri": dc.verification_uri,
                "device_code": dc.device_code,
                "interval": dc.interval,
                "expires_in": dc.expires_in,
            });
            CString::new(json.to_string()).unwrap().into_raw()
        }
        Err(e) => {
            let json = serde_json::json!({ "error": e });
            CString::new(json.to_string()).unwrap().into_raw()
        }
    }
}

/// Poll for GitHub access token using a device code. Blocks until done.
///
/// # Safety
///
/// `device_code` must be a valid null-terminated C string.
/// `interval` is a plain integer (polling interval in seconds).
#[no_mangle]
pub unsafe extern "C" fn hostsync_poll_for_token(
    device_code: *const c_char,
    interval: u64,
) -> i32 {
    let c_str = unsafe { CStr::from_ptr(device_code) };
    let dc_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let dc = crate::auth::DeviceCode {
        user_code: String::new(),
        verification_uri: String::new(),
        device_code: dc_str.to_string(),
        interval,
        expires_in: 900,
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(crate::auth::poll_for_token(&dc)) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Save a GitHub token directly (for Device Flow completion from Kotlin side).
///
/// # Safety
///
/// `token` must be a valid null-terminated C string containing the GitHub access token.
#[no_mangle]
pub unsafe extern "C" fn hostsync_save_github_token(token: *const c_char) -> i32 {
    let c_str = unsafe { CStr::from_ptr(token) };
    let token_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let existing = storage::load_github_state();
    let state = storage::GithubState {
        token: Some(token_str.to_string()),
        gist_id: existing.gist_id,
        username: existing.username,
        avatar_url: existing.avatar_url,
    };
    match storage::save_github_state(&state) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Logout by removing the GitHub state file.
#[no_mangle]
pub extern "C" fn hostsync_logout() -> i32 {
    match storage::clear_github_state() {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Fetch GitHub username using saved token and update state. Returns 0 on success.
#[no_mangle]
pub extern "C" fn hostsync_fetch_username() -> i32 {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let state = storage::load_github_state();
        let token = match state.token.as_deref() {
            Some(t) => t,
            None => return -1,
        };
        let client = match crate::http::client() {
            Ok(c) => c,
            Err(_) => return -2,
        };
        let resp = match client
            .get("https://api.github.com/user")
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "HostSync")
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => return -3,
        };
        let user: serde_json::Value = match resp.json().await {
            Ok(v) => v,
            Err(_) => return -4,
        };
        let new_state = storage::GithubState {
            token: state.token,
            gist_id: state.gist_id,
            username: user["login"].as_str().map(String::from),
            avatar_url: user["avatar_url"].as_str().map(String::from),
        };
        match storage::save_github_state(&new_state) {
            Ok(_) => 0,
            Err(_) => -5,
        }
    })
}

/// Download servers from GitHub gist. Returns 0 on success.
#[no_mangle]
pub extern "C" fn hostsync_sync_download() -> i32 {
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(crate::sync::download(None)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
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

/// Set the data directory path (for Android).
/// # Safety
/// `path` must be a valid, non-null, NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn hostsync_set_data_dir(path: *const c_char) {
    let c_str = unsafe { CStr::from_ptr(path) };
    if let Ok(s) = c_str.to_str() {
        storage::set_data_dir(s);
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
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncLogout(
        _env: JNIEnv,
        _class: JClass,
    ) -> jint {
        super::hostsync_logout()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncGetGithubUsername(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jni::sys::jstring {
        let s = super::hostsync_get_github_username();
        rust_to_java_string(&mut env, s)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncRequestDeviceCode(
        mut env: JNIEnv,
        _class: JClass,
    ) -> jni::sys::jstring {
        let s = super::hostsync_request_device_code();
        rust_to_java_string(&mut env, s)
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncSaveGithubToken(
        mut env: JNIEnv,
        _class: JClass,
        token: JString,
    ) -> jint {
        let token_str: String = env.get_string(&token).unwrap().into();
        let c_token = CString::new(token_str).unwrap();
        unsafe { super::hostsync_save_github_token(c_token.as_ptr()) }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncFetchUsername(
        _env: JNIEnv,
        _class: JClass,
    ) -> jint {
        super::hostsync_fetch_username()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncSyncDownload(
        _env: JNIEnv,
        _class: JClass,
    ) -> jint {
        super::hostsync_sync_download()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncSetSyncPassphrase(
        mut env: JNIEnv,
        _class: JClass,
        passphrase: JString,
    ) -> jint {
        let pp_str: String = env.get_string(&passphrase).unwrap().into();
        let c_pp = CString::new(pp_str).unwrap();
        unsafe { super::hostsync_set_sync_passphrase(c_pp.as_ptr()) }
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncHasSyncPassphrase(
        _env: JNIEnv,
        _class: JClass,
    ) -> jint {
        super::hostsync_has_sync_passphrase()
    }

    #[no_mangle]
    pub extern "system" fn Java_com_hostsync_app_MainActivity_hostsyncSetDataDir(
        mut env: JNIEnv,
        _class: JClass,
        path: JString,
    ) {
        let path_str: String = env.get_string(&path).unwrap().into();
        let c_path = CString::new(path_str).unwrap();
        unsafe { super::hostsync_set_data_dir(c_path.as_ptr()) }
    }
}
