use crate::model::Server;
use std::fs;
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

/// If the server has inline private key content and an IdentityFile path,
/// write the key content to that path (keeping them in sync).
fn sync_key_to_file(server: &Server) {
    let path = match server.identity_file.as_deref() {
        Some(p) if !p.is_empty() => p,
        _ => return,
    };
    let key = match server.private_key.as_deref() {
        Some(k) if !k.is_empty() => k,
        _ => return,
    };

    let expanded = expand_home(path);

    if let Some(parent) = expanded.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::write(&expanded, key).is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&expanded, fs::Permissions::from_mode(0o600));
        }
        #[cfg(windows)]
        {
            // Windows SSH requires key files to only be accessible by the current user.
            // Remove inherited ACLs, then grant only the current user full control.
            let path_str = expanded.to_string_lossy();
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
}

/// Launches the system's native terminal with SSH for the given server.
pub fn launch_native_terminal(server: &Server) -> Result<(), String> {
    // Sync inline key to IdentityFile path before connecting
    sync_key_to_file(server);

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

    if let Some(ref path) = server.identity_file {
        if !path.is_empty() {
            args.push("-i".to_string());
            args.push(path.clone());
        }
    }

    args.push(format!("{}@{}", server.username, server.host));
    args
}
