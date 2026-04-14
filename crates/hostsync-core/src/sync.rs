use crate::storage;
use reqwest::Client;

const GIST_FILE_NAME: &str = "hostsync_data.enc";
const GIST_DESCRIPTION: &str = "HostSync Encrypted Server Data";

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

/// Uploads encrypted server data to a GitHub Gist.
pub async fn upload() -> Result<(), String> {
    let state = storage::load_github_state();
    let token = state.token.as_deref().ok_or("not logged in")?;
    let data = storage::get_raw_encrypted().ok_or("no data to upload")?;
    let client = make_client()?;

    if let Some(ref gist_id) = state.gist_id {
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
        if resp.status().is_success() {
            let gist: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if let Some(id) = gist["id"].as_str() {
                let mut s = storage::load_github_state();
                s.gist_id = Some(id.to_string());
                storage::save_github_state(&s).map_err(|e| e.to_string())?;
            }
        } else {
            return Err(format!("create gist failed: {}", resp.status()));
        }
    }
    Ok(())
}

/// Downloads encrypted data from GitHub Gist and saves locally.
pub async fn download() -> Result<(), String> {
    let mut state = storage::load_github_state();
    let token = state.token.as_deref().ok_or("not logged in")?;
    let client = make_client()?;

    // Find gist if we don't have the ID
    if state.gist_id.is_none() {
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
                        state.gist_id = Some(id.to_string());
                        storage::save_github_state(&state).map_err(|e| e.to_string())?;
                        break;
                    }
                }
            }
        }
    }

    let gist_id = state.gist_id.as_deref().ok_or("no gist found")?;
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

    storage::set_raw_encrypted(content).map_err(|e| e.to_string())
}
