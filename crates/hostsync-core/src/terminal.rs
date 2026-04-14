use crate::model::Server;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Expands a path that may start with ~ or %USERPROFILE% to an absolute path.
fn expand_home(path: &str) -> std::path::PathBuf {
    let home = dirs::home_dir();

    // ~/... or ~\...
    if let Some(stripped) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        if let Some(ref h) = home {
            return h.join(stripped);
        }
    }

    // %USERPROFILE%\...
    if let Some(stripped) = path
        .strip_prefix("%USERPROFILE%\\")
        .or_else(|| path.strip_prefix("%USERPROFILE%/"))
    {
        if let Some(ref h) = home {
            return h.join(stripped);
        }
    }

    path.into()
}

/// Writes the server's inline private key to a unique file under ~/.ssh/hostsync_keys/
/// using the server's ID as the filename. This avoids overwriting existing keys or
/// collisions between servers that share the same identity_file path.
/// Returns the actual path to use for `-i` in SSH args.
fn sync_key_to_file(server: &Server) -> Option<String> {
    let key = match server.private_key.as_deref() {
        Some(k) if !k.is_empty() => k,
        _ => {
            // No inline key — use the original identity_file path as-is
            return server.identity_file.as_ref().map(|p| {
                expand_home(p).to_string_lossy().to_string()
            });
        }
    };

    // Write to hostsync-managed key directory with unique name per server
    let key_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ssh")
        .join("hostsync_keys");

    let _ = fs::create_dir_all(&key_dir);

    let key_path = key_dir.join(format!("{}.key", server.id));

    // Only write if content differs (avoid unnecessary writes)
    let needs_write = match fs::read_to_string(&key_path) {
        Ok(existing) => existing != key,
        Err(_) => true,
    };

    if needs_write {
        if fs::write(&key_path, key).is_err() {
            return server.identity_file.as_ref().map(|p| {
                expand_home(p).to_string_lossy().to_string()
            });
        }
        // Fix permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        }
        #[cfg(windows)]
        {
            let path_str = key_path.to_string_lossy();
            let _ = Command::new("icacls")
                .args([path_str.as_ref(), "/inheritance:r"])
                .output();
            if let Ok(user) = std::env::var("USERNAME") {
                let _ = Command::new("icacls")
                    .args([path_str.as_ref(), "/grant:r", &format!("{}:F", user)])
                    .output();
            }
        }
    }

    Some(key_path.to_string_lossy().to_string())
}

/// Launches the system's native terminal with SSH for the given server.
pub fn launch_native_terminal(server: &Server) -> Result<(), String> {
    // Sync inline key to a unique file and get the actual path
    let key_path = sync_key_to_file(server);
    let ssh_args = build_ssh_args(server, key_path.as_deref());

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

fn build_ssh_args(server: &Server, key_path: Option<&str>) -> Vec<String> {
    let mut args = vec!["-p".to_string(), server.port.to_string()];

    if let Some(path) = key_path {
        if !path.is_empty() {
            args.push("-i".to_string());
            args.push(path.to_string());
        }
    }

    args.push(format!("{}@{}", server.username, server.host));
    args
}

