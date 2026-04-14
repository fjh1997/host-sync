use crate::crypto;
use crate::model::Server;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("hostsync")
}

fn servers_path() -> PathBuf {
    data_dir().join("servers.enc")
}

fn token_path() -> PathBuf {
    data_dir().join("github.json")
}

/// Ensures data directory exists.
fn ensure_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(data_dir())
}

/// Derives the encryption key from the GitHub OAuth token.
/// Same account → same token → same key on every device.
pub fn get_encryption_key() -> String {
    load_github_state()
        .token
        .unwrap_or_default()
}

pub fn load_servers() -> Vec<Server> {
    let encrypted = match std::fs::read_to_string(servers_path()) {
        Ok(s) if !s.trim().is_empty() => s,
        _ => return Vec::new(),
    };
    let key = get_encryption_key();
    match crypto::decrypt(encrypted.trim(), &key) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_servers(servers: &[Server]) -> std::io::Result<()> {
    ensure_dir()?;
    let json = serde_json::to_string(servers).map_err(std::io::Error::other)?;
    let key = get_encryption_key();
    let encrypted = crypto::encrypt(&json, &key);
    std::fs::write(servers_path(), encrypted)
}

/// Returns the raw encrypted data for cloud sync upload.
pub fn get_raw_encrypted() -> Option<String> {
    std::fs::read_to_string(servers_path()).ok()
}

/// Sets raw encrypted data from cloud sync download.
pub fn set_raw_encrypted(data: &str) -> std::io::Result<()> {
    ensure_dir()?;
    std::fs::write(servers_path(), data)
}

// --- GitHub auth state ---

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct GithubState {
    pub token: Option<String>,
    pub gist_id: Option<String>,
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

pub fn load_github_state() -> GithubState {
    std::fs::read_to_string(token_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_github_state(state: &GithubState) -> std::io::Result<()> {
    ensure_dir()?;
    let json = serde_json::to_string_pretty(state).map_err(std::io::Error::other)?;
    std::fs::write(token_path(), json)
}

pub fn clear_github_state() -> std::io::Result<()> {
    let _ = std::fs::remove_file(token_path());
    Ok(())
}

pub fn is_logged_in() -> bool {
    load_github_state().token.is_some()
}

// --- Proxy settings ---

fn proxy_path() -> PathBuf {
    data_dir().join("proxy.txt")
}

/// Load saved proxy URL (e.g. "http://127.0.0.1:10808" or "socks5://127.0.0.1:1080").
/// Returns None if no proxy is configured.
pub fn load_proxy() -> Option<String> {
    std::fs::read_to_string(proxy_path())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Save proxy URL. Pass empty string to clear.
pub fn save_proxy(proxy: &str) -> std::io::Result<()> {
    ensure_dir()?;
    if proxy.trim().is_empty() {
        let _ = std::fs::remove_file(proxy_path());
        Ok(())
    } else {
        std::fs::write(proxy_path(), proxy.trim())
    }
}
