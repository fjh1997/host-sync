# HostSync

Securely manage, sync and connect to all your Linux servers — SSH keys, passwords, and configs, encrypted and synced across every device.

安全管理、同步和连接你所有的 Linux 服务器 — SSH 密钥、密码和配置，加密存储并在所有设备间同步。

Cross-platform native binaries built in Rust. ~5MB binary, ~5MB RAM.

跨平台原生二进制，Rust 构建。约 5MB 体积，约 5MB 内存。

## Features / 功能

- **SSH Key Management / SSH 密钥管理** — Store SSH private keys (PEM/OpenSSH), IdentityFile paths, and passphrases in one place. Never lose track of which key goes to which server. / 集中存储 SSH 私钥（PEM/OpenSSH 格式）、IdentityFile 路径和密钥口令，再也不会搞混哪台服务器用哪把钥匙
- **Encrypted Key Sync / 加密密钥同步** — All SSH keys and passwords are AES-256-GCM encrypted (Argon2 key derivation) before syncing. Your private keys never leave your devices in plaintext. / 所有 SSH 密钥和密码在同步前均经过 AES-256-GCM 加密（Argon2 密钥派生），私钥永远不会以明文离开你的设备
- **Cross-Device Sync via GitHub Gists / 通过 GitHub Gist 跨设备同步** — Encrypted credentials synced through a private Gist. Add a server on your PC, connect from your phone. / 加密凭据通过私有 Gist 同步，在电脑上添加服务器，手机上即可连接
- **SSH Config Compatible / SSH Config 兼容** — Import keys and hosts from `~/.ssh/config`, export back. IdentityFile paths preserved. / 从 `~/.ssh/config` 导入密钥和主机，也可导出回去，IdentityFile 路径完整保留
- **GitHub OAuth Login / GitHub OAuth 登录** — Sign in with GitHub, no separate account needed / 使用 GitHub 登录，无需单独注册账号
- **Desktop: Native Terminal / 桌面端：原生终端** — Opens system terminal with `ssh -F` referencing your keys / 调用系统终端通过 `ssh -F` 引用你的密钥连接
- **Mobile: Built-in SSH Terminal / 移动端：内置 SSH 终端** — Connect with stored keys directly from Android/iOS / 在 Android/iOS 上直接使用已存储的密钥连接
- **Truly Native / 真正原生** — No Electron, no WebView, no Flutter, no VM. Pure compiled Rust binaries. / 无 Electron、无 WebView、无 Flutter、无虚拟机，纯 Rust 编译二进制

## Why HostSync? / 为什么用 HostSync？

Managing SSH keys across multiple machines is painful:
- Keys scattered across `~/.ssh/` on different devices
- Copying private keys via insecure channels (email, chat, USB)
- Forgetting which key authenticates to which server
- No way to access your servers from your phone

HostSync solves this: one encrypted vault for all your SSH keys, passwords, and server configs — synced securely across every device you own.

跨设备管理 SSH 密钥很麻烦：
- 密钥散落在不同设备的 `~/.ssh/` 目录
- 通过不安全的渠道（邮件、聊天、U盘）复制私钥
- 忘记哪把密钥对应哪台服务器
- 手机上无法访问你的服务器

HostSync 解决这些问题：一个加密保险库存放所有 SSH 密钥、密码和服务器配置 — 在你的所有设备间安全同步。

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

| Platform / 平台 | x86_64 | ARM64 |
|----------|------|------|
| Windows  | `hostsync-windows-x64.zip` | `hostsync-windows-arm64.zip` |
| Linux    | `hostsync-linux-x64.tar.gz` | `hostsync-linux-arm64.tar.gz` |
| macOS    | `hostsync-macos-x64.tar.gz` | `hostsync-macos-arm64.tar.gz` |
| Android  | `android-jniLibs.zip` (x86, x86_64, armeabi-v7a, arm64-v8a) ||
| iOS      | `libhostsync_core.a` (arm64) | sim: universal (arm64 + x86_64) |

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

## SSH Key Management / SSH 密钥管理

HostSync supports two ways to store SSH keys per server:

HostSync 支持两种方式存储每台服务器的 SSH 密钥：

| Method / 方式 | Field / 字段 | Use case / 使用场景 |
|---|---|---|
| **IdentityFile path** | `~/.ssh/id_rsa` | Desktop — references key file on disk, used by native `ssh` / 桌面端 — 引用磁盘上的密钥文件，由原生 `ssh` 使用 |
| **Inline private key** | PEM content stored in vault | Mobile & cross-device — key content encrypted and synced / 移动端和跨设备 — 密钥内容加密后同步 |

Both can be set simultaneously: desktop uses the file path, mobile uses the inline key.

两者可以同时设置：桌面端使用文件路径，移动端使用内联密钥。

### Security model / 安全模型

```
Your SSH keys & passwords
        ↓
AES-256-GCM encrypt (Argon2 derived key)
        ↓
Encrypted blob stored locally
        ↓ (sync)
Private GitHub Gist (still encrypted)
        ↓ (other device)
AES-256-GCM decrypt (same key)
        ↓
Your SSH keys & passwords
```

- Keys are encrypted at rest and in transit / 密钥在存储和传输中均加密
- GitHub never sees your plaintext keys / GitHub 永远看不到你的明文密钥
- Encryption key is generated locally and never uploaded / 加密密钥在本地生成，永不上传

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

#### Prompt 6 — Emphasize SSH key management / 强调 SSH 密钥管理
```
在readme和about里面重点强调一下ssh key的管理和同步
```

#### Prompt 7 — Dual architecture support / 双架构支持
```
所有端要同时支持x86和arm架构
```

## License / 许可证

MIT
