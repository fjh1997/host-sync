fn main() {
    // Pass git short hash as HOSTSYNC_BUILD at compile time
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    let hash = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    };
    println!("cargo:rustc-env=HOSTSYNC_BUILD={}", hash);

    // Embed Windows application icon
    #[cfg(windows)]
    let _ = embed_resource::compile("icon.rc", embed_resource::NONE);
}
