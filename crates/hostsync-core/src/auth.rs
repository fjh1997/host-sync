use crate::storage;
use reqwest::Client;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

pub const CLIENT_ID: &str = "YOUR_GITHUB_CLIENT_ID";
pub const CLIENT_SECRET: &str = "YOUR_GITHUB_CLIENT_SECRET";
const REDIRECT_URI: &str = "http://localhost:9876/callback";
const SCOPES: &str = "gist,read:user";

/// Returns the GitHub OAuth authorization URL.
pub fn auth_url() -> String {
    let state = chrono::Utc::now().timestamp_millis().to_string();
    format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}&state={}",
        CLIENT_ID, REDIRECT_URI, SCOPES, state
    )
}

/// Starts a local HTTP server, waits for the OAuth callback, exchanges the
/// code for a token, fetches user info, and saves everything.
/// Returns the access token on success.
pub async fn login() -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:9876").map_err(|e| e.to_string())?;
    listener
        .set_nonblocking(false)
        .map_err(|e| e.to_string())?;

    // Accept one connection (blocking in a tokio::spawn_blocking)
    let (mut stream, _) = tokio::task::spawn_blocking(move || {
        listener.accept()
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).map_err(|e| e.to_string())?;

    // Parse code from GET /callback?code=xxx&state=yyy
    let code = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|path| {
            let query = path.split('?').nth(1).unwrap_or("");
            query
                .split('&')
                .find_map(|pair| {
                    let (k, v) = pair.split_once('=')?;
                    if k == "code" { Some(v.to_string()) } else { None }
                })
        })
        .ok_or("no code in callback")?;

    // Exchange code for token
    let client = Client::new();
    let resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", CLIENT_ID),
            ("client_secret", CLIENT_SECRET),
            ("code", code.as_str()),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let token = body["access_token"]
        .as_str()
        .ok_or("no access_token in response")?
        .to_string();

    // Fetch user info
    let user_resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "HostSync")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let user: serde_json::Value = user_resp.json().await.map_err(|e| e.to_string())?;

    // Save state
    let state = storage::GithubState {
        token: Some(token.clone()),
        gist_id: storage::load_github_state().gist_id,
        username: user["login"].as_str().map(String::from),
        avatar_url: user["avatar_url"].as_str().map(String::from),
    };
    storage::save_github_state(&state).map_err(|e| e.to_string())?;

    // Send success response
    let html = r#"<html><body style="display:flex;justify-content:center;align-items:center;height:100vh;font-family:system-ui;background:#0d1117;color:#f0f6fc;"><div style="text-align:center"><h1>&#10004; Login Successful</h1><p>You can close this window.</p></div></body></html>"#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );
    let _ = stream.write_all(response.as_bytes());

    Ok(token)
}

pub fn logout() -> Result<(), String> {
    storage::clear_github_state().map_err(|e| e.to_string())
}
