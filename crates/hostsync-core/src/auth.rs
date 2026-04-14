use crate::storage;
use reqwest::Client;

/// GitHub OAuth App Client ID (public — safe to commit).
/// Create your own at: https://github.com/settings/applications/new
/// Enable "Device Flow" in the OAuth App settings.
pub const CLIENT_ID: &str = "Ov23liGz0a5kU4v1LwKI";

const DEVICE_CODE_URL: &str = "https://github.com/login/device/code";
const ACCESS_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const SCOPES: &str = "gist read:user";

/// Response from the device code request.
#[derive(Debug, Clone)]
pub struct DeviceCode {
    pub user_code: String,
    pub verification_uri: String,
    pub device_code: String,
    pub interval: u64,
    pub expires_in: u64,
}

/// Step 1: Request a device code. Returns the code the user must enter
/// at the verification URL.
pub async fn request_device_code() -> Result<DeviceCode, String> {
    let client = Client::new();
    let resp = client
        .post(DEVICE_CODE_URL)
        .header("Accept", "application/json")
        .form(&[("client_id", CLIENT_ID), ("scope", SCOPES)])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    if let Some(err) = body["error"].as_str() {
        return Err(format!("{}: {}", err, body["error_description"].as_str().unwrap_or("")));
    }

    Ok(DeviceCode {
        user_code: body["user_code"]
            .as_str()
            .ok_or("missing user_code")?
            .to_string(),
        verification_uri: body["verification_uri"]
            .as_str()
            .ok_or("missing verification_uri")?
            .to_string(),
        device_code: body["device_code"]
            .as_str()
            .ok_or("missing device_code")?
            .to_string(),
        interval: body["interval"].as_u64().unwrap_or(5),
        expires_in: body["expires_in"].as_u64().unwrap_or(900),
    })
}

/// Step 2: Poll for the access token after the user has entered the code.
/// This blocks (with async sleep) until the user authorizes or the code expires.
pub async fn poll_for_token(device_code: &DeviceCode) -> Result<String, String> {
    let client = Client::new();
    let interval = std::time::Duration::from_secs(device_code.interval);
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(device_code.expires_in);

    loop {
        if std::time::Instant::now() > deadline {
            return Err("device code expired".to_string());
        }

        tokio::time::sleep(interval).await;

        let resp = client
            .post(ACCESS_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

        if let Some(token) = body["access_token"].as_str() {
            // Got the token — fetch user info and save
            fetch_and_save_user(token).await?;
            return Ok(token.to_string());
        }

        match body["error"].as_str() {
            Some("authorization_pending") => continue,
            Some("slow_down") => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            Some(err) => {
                return Err(format!(
                    "{}: {}",
                    err,
                    body["error_description"].as_str().unwrap_or("")
                ));
            }
            None => continue,
        }
    }
}

/// Convenience: runs both steps and polls until authorized.
/// The caller should open `dc.verification_uri` in a browser and show `dc.user_code`.
pub async fn login() -> Result<String, String> {
    let dc = request_device_code().await?;

    eprintln!(
        "Open {} and enter code: {}",
        dc.verification_uri, dc.user_code
    );

    poll_for_token(&dc).await
}

async fn fetch_and_save_user(token: &str) -> Result<(), String> {
    let client = Client::new();
    let user_resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "HostSync")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let user: serde_json::Value = user_resp.json().await.map_err(|e| e.to_string())?;

    let state = storage::GithubState {
        token: Some(token.to_string()),
        gist_id: storage::load_github_state().gist_id,
        username: user["login"].as_str().map(String::from),
        avatar_url: user["avatar_url"].as_str().map(String::from),
    };
    storage::save_github_state(&state).map_err(|e| e.to_string())
}

pub fn logout() -> Result<(), String> {
    storage::clear_github_state().map_err(|e| e.to_string())
}
