use crate::storage;
use reqwest::Client;

const GIST_FILE_NAME: &str = "hostsync_data.enc";
const GIST_DESCRIPTION: &str = "HostSync Encrypted Server Data";

/// Specific error indicating that the user needs to provide a sync passphrase.
pub const ERR_NEED_PASSPHRASE: &str = "NEED_PASSPHRASE";

fn headers(token: &str) -> reqwest::header::HeaderMap {
    let mut h = reqwest::header::HeaderMap::new();
    h.insert("Authorization", format!("Bearer {}", token).parse().unwrap());
    h.insert("Accept", "application/vnd.github+json".parse().unwrap());
    h.insert("User-Agent", "HostSync".parse().unwrap());
    h
}

fn make_client() -> Result<Client, String> {
    crate::http::client()
}

/// Searches the user's gists for one containing hostsync_data.enc.
/// Returns the gist ID if found.
async fn find_remote_gist(client: &Client, token: &str) -> Result<Option<String>, String> {
    let resp = client
        .get("https://api.github.com/gists?per_page=100")
        .headers(headers(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let gists: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;
    for gist in &gists {
        if let Some(files) = gist["files"].as_object() {
            if files.contains_key(GIST_FILE_NAME) {
                if let Some(id) = gist["id"].as_str() {
                    return Ok(Some(id.to_string()));
                }
            }
        }
    }
    Ok(None)
}

/// Uploads encrypted server data to a GitHub Gist.
/// If `passphrase` is provided, it will be saved locally and used as the encryption key.
/// Always queries remote to find existing gist; creates a new one if none exists.
pub async fn upload(passphrase: Option<&str>) -> Result<(), String> {
    // If a new passphrase is provided, save it locally
    if let Some(pp) = passphrase {
        storage::save_sync_passphrase(pp).map_err(|e| e.to_string())?;
    }

    // Require passphrase to be set before uploading
    if !storage::has_sync_passphrase() {
        return Err(ERR_NEED_PASSPHRASE.to_string());
    }

    // Normalize identity_file path for any server with an inline private key
    // so exported SSH configs point to the managed path automatically
    let mut servers = storage::load_servers();
    let mut changed = false;
    for server in &mut servers {
        if let Some(ref priv_key) = server.private_key {
            if !priv_key.trim().is_empty() {
                let managed_path = format!("~/.ssh/hostsync_keys/{}.key", server.id);
                if server.identity_file.as_deref() != Some(&managed_path) {
                    server.identity_file = Some(managed_path);
                    changed = true;
                }
            }
        }
    }
    if changed {
        storage::save_servers(&servers).map_err(|e| e.to_string())?;
    }

    let state = storage::load_github_state();
    let token = state.token.as_deref().ok_or("not logged in")?;
    let data = storage::get_raw_encrypted().ok_or("no data to upload")?;
    let client = make_client()?;

    if let Some(gist_id) = find_remote_gist(&client, token).await? {
        let body = serde_json::json!({
            "files": { GIST_FILE_NAME: { "content": data } }
        });
        let resp = client
            .patch(format!("https://api.github.com/gists/{}", gist_id))
            .headers(headers(token))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("update gist failed: {}", resp.status()));
        }
    } else {
        let body = serde_json::json!({
            "description": GIST_DESCRIPTION,
            "public": false,
            "files": { GIST_FILE_NAME: { "content": data } }
        });
        let resp = client
            .post("https://api.github.com/gists")
            .headers(headers(token))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("create gist failed: {}", resp.status()));
        }
    }
    Ok(())
}

/// Downloads encrypted data from GitHub Gist and saves locally.
/// If `passphrase` is provided, it will be saved locally and used for decryption.
/// Includes migration logic: if passphrase decryption fails, tries legacy token key.
pub async fn download(passphrase: Option<&str>) -> Result<(), String> {
    // If a passphrase is provided, save it locally
    if let Some(pp) = passphrase {
        storage::save_sync_passphrase(pp).map_err(|e| e.to_string())?;
    }

    let state = storage::load_github_state();
    let token = state.token.as_deref().ok_or("not logged in")?;
    let client = make_client()?;

    let gist_id = find_remote_gist(&client, token)
        .await?
        .ok_or("no hostsync gist found in your GitHub account")?;

    let resp = client
        .get(format!("https://api.github.com/gists/{}", gist_id))
        .headers(headers(token))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Err(format!("fetch gist failed: {}", resp.status()));
    }

    let gist: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let content = gist["files"][GIST_FILE_NAME]["content"]
        .as_str()
        .ok_or("file not found in gist")?;

    let trimmed = content.trim();

    // Try decrypting with current key (passphrase)
    let key = storage::get_encryption_key();
    if !key.is_empty() && crate::crypto::decrypt(trimmed, &key).is_ok() {
        return storage::set_raw_encrypted(content).map_err(|e| e.to_string());
    }

    // Passphrase not set or wrong — user needs to enter the correct one
    Err(ERR_NEED_PASSPHRASE.to_string())
}
