use crate::storage;
use reqwest::Client;

/// Builds a reqwest Client that respects the user's proxy settings.
pub fn client() -> Result<Client, String> {
    let mut builder = Client::builder();

    if let Some(proxy_url) = storage::load_proxy() {
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|e| format!("invalid proxy URL: {}", e))?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|e| e.to_string())
}
