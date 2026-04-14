use crate::model::Server;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Returns the temp directory for HostSync key files.
fn temp_key_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("hostsync_keys");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// If the server has an inline private key but no IdentityFile path,
/// writes the key to a temp file and returns the path.
fn resolve_key_file(server: &Server) -> Option<String> {
    // Prefer explicit IdentityFile path
    if let Some(ref path) = server.identity_file {
        if !path.is_empty() {
            return Some(path.clone());
        }
    }
    // Fall back to writing inline key to temp file
    if let Some(ref key) = server.private_key {
        if !key.is_empty() {
            let key_path = temp_key_dir().join(format!("key_{}", server.id));
            if fs::write(&key_path, key).is_ok() {
                // Unix: chmod 600
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
                }
                return Some(key_path.to_string_lossy().to_string());
            }
        }
    }
    None
}

/// Launches the system's native terminal with SSH for the given server.
pub fn launch_native_terminal(server: &Server) -> Result<(), String> {
    let ssh_args = build_ssh_args(server);

    #[cfg(target_os = "windows")]
    {
        let wt = Command::new("cmd")
            .args(["/c", "start", "wt", "ssh"])
            .args(&ssh_args)
            .spawn();
        if wt.is_ok() {
            return Ok(());
        }
        Command::new("cmd")
            .args(["/c", "start", "cmd", "/k", "ssh"])
            .args(&ssh_args)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        let escaped = ssh_args.join(" ");
        let script = format!(
            "tell application \"Terminal\" to do script \"ssh {}\"",
            escaped
        );
        Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        for term in &["gnome-terminal", "konsole", "xfce4-terminal", "xterm"] {
            let result = match *term {
                "gnome-terminal" => Command::new(term)
                    .arg("--")
                    .arg("ssh")
                    .args(&ssh_args)
                    .spawn(),
                "konsole" => Command::new(term)
                    .arg("-e")
                    .arg("ssh")
                    .args(&ssh_args)
                    .spawn(),
                _ => Command::new(term)
                    .arg("-e")
                    .arg(format!("ssh {}", ssh_args.join(" ")))
                    .spawn(),
            };
            if result.is_ok() {
                return Ok(());
            }
        }
        Err("no terminal emulator found".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    Err("unsupported platform".to_string())
}

fn build_ssh_args(server: &Server) -> Vec<String> {
    let mut args = vec!["-p".to_string(), server.port.to_string()];

    if let Some(key_path) = resolve_key_file(server) {
        args.push("-i".to_string());
        args.push(key_path);
    }

    args.push(format!("{}@{}", server.username, server.host));
    args
}
