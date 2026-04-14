# HostSync

Cross-platform Linux server manager — fully native binaries, no runtime, no VM.

跨平台 Linux 服务器管理器 — 全原生二进制，无运行时，无虚拟机。

Built in Rust for minimal binary size (~5MB) and memory usage (~5MB RAM).

使用 Rust 构建，二进制体积约 5MB，内存占用约 5MB。

## Features / 功能

- **GitHub OAuth Login / GitHub OAuth 登录** — Sign in with GitHub, no separate account needed / 使用 GitHub 登录，无需单独注册账号
- **AES-256-GCM Encrypted Storage / AES-256-GCM 加密存储** — Passwords and keys encrypted locally with Argon2 key derivation / 密码和密钥使用 Argon2 密钥派生在本地加密存储
- **Cloud Sync via GitHub Gists / 通过 GitHub Gist 云同步** — Encrypted data synced across devices through a private Gist / 加密数据通过私有 Gist 在所有设备间同步
- **SSH Config Compatible / SSH Config 兼容** — Import/export standard `~/.ssh/config` format / 导入导出标准 `~/.ssh/config` 格式
- **Desktop: Native Terminal / 桌面端：原生终端** — Opens system terminal (Windows Terminal / Terminal.app / gnome-terminal) / 调用系统终端连接（Windows Terminal / Terminal.app / gnome-terminal）
- **Mobile: Built-in SSH Terminal / 移动端：内置 SSH 终端** — Android (JSch) and iOS (libssh2) native SSH shells / Android (JSch) 和 iOS (libssh2) 原生 SSH 终端
- **Truly Native / 真正原生** — No Electron, no WebView, no Flutter, no VM. Pure compiled binaries. / 无 Electron、无 WebView、无 Flutter、无虚拟机，纯编译二进制

## Architecture / 架构

```
host-sync/
├── crates/
│   ├── hostsync-core/        # Rust 核心库（加密、SSH config、存储、OAuth、同步）
│   │                         # 编译为: .dll/.so/.dylib (桌面), .so (Android JNI), .a (iOS FFI)
│   └── hostsync-desktop/     # iced GUI — 编译为 Win/Linux/Mac 原生二进制
├── mobile/
│   ├── android/              # Kotlin + Jetpack Compose + JNI → Rust .so
│   └── ios/                  # Swift + SwiftUI + C FFI → Rust .a
├── altstore/                 # AltStore 源（iOS 侧载安装）
└── .github/workflows/        # CI/CD
```

## Installation / 安装

### Desktop / 桌面端

Download from [Releases](https://github.com/fjh1997/host-sync/releases):

从 [Releases](https://github.com/fjh1997/host-sync/releases) 下载：

| Platform / 平台 | File / 文件 | Size / 大小 |
|----------|------|------|
| Windows  | `hostsync-windows-x64.zip` | ~5MB |
| Linux    | `hostsync-linux-x64.tar.gz` | ~5MB |
| macOS    | `hostsync-macos.zip` | ~5MB |

Single binary, no installation needed. Just run `hostsync`.

单文件，无需安装，直接运行 `hostsync` 即可。

### iOS (AltStore)

1. Install [AltStore](https://altstore.io/) on your iOS device / 在 iOS 设备上安装 AltStore
2. Open AltStore → **Browse** → **Sources** → tap **+** / 打开 AltStore → 浏览 → 源 → 点击左上角 +
3. Add source URL / 添加源地址：
   ```
   https://fjh1997.github.io/host-sync/source.json
   ```
4. Find **HostSync** and install / 在源中找到 HostSync 并安装

### Build from source / 从源码构建

```bash
# Desktop / 桌面端（当前平台）
cargo build --release -p hostsync-desktop

# Android shared library / Android 共享库
cargo ndk -t arm64-v8a build --release -p hostsync-core

# iOS static library / iOS 静态库
cargo build --release --target aarch64-apple-ios -p hostsync-core
```

## Setup / 配置

1. Create a [GitHub OAuth App](https://github.com/settings/applications/new) / 创建 GitHub OAuth 应用：
   - Callback URL / 回调地址: `http://localhost:9876/callback`
2. Edit `crates/hostsync-core/src/auth.rs` with your Client ID/Secret / 编辑 `crates/hostsync-core/src/auth.rs` 填入你的 Client ID 和 Secret
3. Build and run / 构建并运行

## SSH Config Compatibility / SSH Config 兼容性

Import from `~/.ssh/config` or paste config text. Export generates valid OpenSSH config:

支持从 `~/.ssh/config` 导入或粘贴配置文本。导出生成标准 OpenSSH 配置：

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

- **Push to `main` / PR**: test + clippy + build all platforms → pre-release / 推送到 main 或 PR：测试 + clippy + 构建全平台 → 预发布
- **Push `v*` tag**: test + build → formal Release + update AltStore source.json / 推送 v* 标签：测试 + 构建 → 正式发布 + 更新 AltStore source.json

---

## Claude-Assisted Development / Claude 辅助开发

This project was developed with **Claude** (Anthropic) using **Claude Code** CLI.

本项目由 **Claude**（Anthropic）通过 **Claude Code** CLI 辅助开发。

### All Prompts Used / 使用的所有提示词

#### Prompt 1 — Initial project creation / 初始项目创建
```
开发一个图形化软件，支持windows，linux，mac，安卓，ios，使用github oauth登录即可，
能够添加管理并同步你的所有的linux服务器的的域名，密码，密钥。点击连接电脑端不是使用
内置终端，而是调用系统原生终端进行连接。安卓端与ios端则可以使用内置shell界面连接，
使用任何语言都可以，要求体积足够小，内存占用足够轻量化。
```

#### Prompt 2 — SSH config compatibility / SSH 配置兼容
```
配置文件要与 SSH config 文件格式兼容
```

#### Prompt 3 — CI/CD, AltStore, README
```
帮我推送到github，要求每次commit或者pr，以及加v的tag的release都要触发github action
构建测试，commit的构建需要推送到pre-release，加vtag的release推送到正式release，
readme里面要写上claude辅助开发以及本次涉及到的所有提示词。ios要支持altstore安装
```

#### Prompt 4 — Rust rewrite / Rust 重写
```
帮我重构下，要求能够跨平台，但是全是原生程序
```

> Claude initially proposed Go; user asked "为什么用go不用rust？" (why Go not Rust?), Claude agreed Rust is better for the size/memory requirements, user confirmed with "好".
>
> Claude 最初提议使用 Go；用户问"为什么用 Go 不用 Rust？"，Claude 认同 Rust 更符合体积小、内存轻量的需求，用户确认"好"。

#### Prompt 5 — Bilingual README / 中英双语 README
```
readme里面要有中文翻译，About里面也要
```

## License / 许可证

MIT
