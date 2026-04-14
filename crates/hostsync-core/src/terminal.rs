use crate::model::Server;
use std::process::Command;

/// Launches the system's native terminal with SSH for the given server.
pub fn launch_native_terminal(server: &Server) -> Result<(), String> {
    let ssh_args = build_ssh_args(server);

    #[cfg(target_os = "windows")]
    {
        // Try Windows Terminal first, fallback to cmd
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
        return Ok(());
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
        return Ok(());
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
        return Err("no terminal emulator found".to_string());
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    Err("unsupported platform".to_string())
}

fn build_ssh_args(server: &Server) -> Vec<String> {
    let mut args = vec!["-p".to_string(), server.port.to_string()];

    if let Some(ref id_file) = server.identity_file {
        if !id_file.is_empty() {
            args.push("-i".to_string());
            args.push(id_file.clone());
        }
    }

    args.push(format!("{}@{}", server.username, server.host));
    args
}
