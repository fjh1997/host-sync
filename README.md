# HostSync

Cross-platform Linux server manager — fully native binaries, no runtime, no VM.

Built in Rust for minimal binary size (~5MB) and memory usage (~5MB RAM).

## Features

- **GitHub OAuth Login** — Sign in with GitHub, no separate account needed
- **AES-256-GCM Encrypted Storage** — Passwords and keys encrypted locally with Argon2 key derivation
- **Cloud Sync via GitHub Gists** — Encrypted data synced across devices through a private Gist
- **SSH Config Compatible** — Import/export standard `~/.ssh/config` format
- **Desktop: Native Terminal** — Opens system terminal (Windows Terminal / Terminal.app / gnome-terminal)
- **Mobile: Built-in SSH Terminal** — Android (JSch) and iOS (libssh2) native SSH shells
- **Truly Native** — No Electron, no WebView, no Flutter, no VM. Pure compiled binaries.

## Architecture

```
host-sync/
├── crates/
│   ├── hostsync-core/        # Pure Rust core library (crypto, SSH config, storage, OAuth, sync)
│   │                         # Compiles to: .dll/.so/.dylib (desktop), .so (Android JNI), .a (iOS FFI)
│   └── hostsync-desktop/     # iced GUI — compiles to native Win/Linux/Mac binary
├── mobile/
│   ├── android/              # Kotlin + Jetpack Compose + JNI → Rust .so
│   └── ios/                  # Swift + SwiftUI + C FFI → Rust .a
├── altstore/                 # AltStore source for iOS sideloading
└── .github/workflows/        # CI/CD
```

## Installation

### Desktop

Download from [Releases](https://github.com/fjh1997/host-sync/releases):

| Platform | File | Size |
|----------|------|------|
| Windows  | `hostsync-windows-x64.zip` | ~5MB |
| Linux    | `hostsync-linux-x64.tar.gz` | ~5MB |
| macOS    | `hostsync-macos.zip` | ~5MB |

Single binary, no installation needed. Just run `hostsync`.

### iOS (AltStore)

1. Install [AltStore](https://altstore.io/) on your iOS device
2. Open AltStore → **Browse** → **Sources** → tap **+**
3. Add source URL:
   ```
   https://fjh1997.github.io/host-sync/source.json
   ```
4. Find **HostSync** and install

### Build from source

```bash
# Desktop (current platform)
cargo build --release -p hostsync-desktop

# Android shared library
cargo ndk -t arm64-v8a build --release -p hostsync-core

# iOS static library
cargo build --release --target aarch64-apple-ios -p hostsync-core
```

## Setup

1. Create a [GitHub OAuth App](https://github.com/settings/applications/new):
   - Callback URL: `http://localhost:9876/callback`
2. Edit `crates/hostsync-core/src/auth.rs` with your Client ID/Secret
3. Build and run

## SSH Config Compatibility

Import from `~/.ssh/config` or paste config text. Export generates valid OpenSSH config:

```ssh-config
#@HostSync-Id a1b2c3d4-...
#@HostSync-AuthType key
Host prod-web
    HostName 10.0.1.50
    Port 2222
    User deploy
    IdentityFile ~/.ssh/prod_key
```

## CI/CD

- **Push to `main` / PR**: test + clippy + build all platforms → pre-release
- **Push `v*` tag**: test + build → formal Release + update AltStore source.json

---

## Claude-Assisted Development

This project was developed with **Claude** (Anthropic) using **Claude Code** CLI.

### All Prompts Used

#### Prompt 1 — Initial project creation
```
开发一个图形化软件，支持windows，linux，mac，安卓，ios，使用github oauth登录即可，
能够添加管理并同步你的所有的linux服务器的的域名，密码，密钥。点击连接电脑端不是使用
内置终端，而是调用系统原生终端进行连接。安卓端与ios端则可以使用内置shell界面连接，
使用任何语言都可以，要求体积足够小，内存占用足够轻量化。
```

#### Prompt 2 — SSH config compatibility
```
配置文件要与 SSH config 文件格式兼容
```

#### Prompt 3 — CI/CD, AltStore, README
```
帮我推送到github，要求每次commit或者pr，以及加v的tag的release都要触发github action
构建测试，commit的构建需要推送到pre-release，加vtag的release推送到正式release，
readme里面要写上claude辅助开发以及本次涉及到的所有提示词。ios要支持altstore安装
```

#### Prompt 4 — Rust rewrite
```
帮我重构下，要求能够跨平台，但是全是原生程序
```

> Claude initially proposed Go; user asked "为什么用go不用rust？" (why Go not Rust?), Claude agreed Rust is better for the size/memory requirements, user confirmed with "好".

## License

MIT
