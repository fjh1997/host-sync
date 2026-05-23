<div align="center">
  <img src="./altstore/icon.png" alt="HostSync" width="88" />
  <h1>HostSync</h1>
  <p>Securely manage, sync and connect to all your Linux servers — SSH keys, passwords & configs, encrypted across every device.</p>
  <p>安全管理、同步和连接你所有的 Linux 服务器 — SSH 密钥、密码和配置，加密存储并在所有设备间同步。</p>

  <p>
    <img alt="Rust" src="https://img.shields.io/badge/Rust-Native-orange?style=flat-square" />
    <img alt="Platforms" src="https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS%20%7C%20Android%20%7C%20iOS-1f6feb?style=flat-square" />
    <img alt="Package" src="https://img.shields.io/badge/Package-DMG%20%7C%20ZIP%20%7C%20APK%20%7C%20IPA-2da44e?style=flat-square" />
    <img alt="Size" src="https://img.shields.io/badge/Binary-~5MB%20%7C%20RAM%20~5MB-6f42c1?style=flat-square" />
  </p>

  <p>
    <a href="https://github.com/fjh1997/host-sync/releases">Download / 下载</a>
  </p>
</div>

## Features / 功能

- **SSH Key Management / SSH 密钥管理** — Store SSH private keys (PEM/OpenSSH), IdentityFile paths, and passphrases in one place / 集中存储 SSH 私钥、IdentityFile 路径和密钥口令
- **Encrypted Key Sync / 加密密钥同步** — AES-256-GCM encrypted (Argon2 key derivation) before syncing. Private keys never leave your devices in plaintext / AES-256-GCM 加密（Argon2 密钥派生），私钥永远不会以明文离开你的设备
- **Cross-Device Sync / 跨设备同步** — Encrypted credentials synced through a private GitHub Gist. Add a server on your PC, connect from your phone / 加密凭据通过私有 Gist 同步，电脑上添加，手机上即可连接
- **SSH Config Compatible / SSH Config 兼容** — Import/export `~/.ssh/config`, IdentityFile paths preserved / 从 `~/.ssh/config` 导入导出，IdentityFile 路径完整保留
- **GitHub OAuth Login / GitHub OAuth 登录** — Device Flow, no server needed, no secret in binary / Device Flow 登录，无需后端服务器，无 secret 泄露
- **Desktop: Native Terminal / 桌面端：原生终端** — Opens system terminal with `ssh -F` referencing your keys / 调用系统终端连接
- **Mobile: Built-in SSH / 移动端：内置 SSH** — Connect with stored keys directly from Android/iOS / 在 Android/iOS 上直接使用已存储的密钥连接
- **Truly Native / 真正原生** — No Electron, no WebView, no Flutter, no VM. Pure compiled Rust / 无 Electron、无 WebView、无 Flutter、无虚拟机

## Why HostSync? / 为什么用 HostSync？

Managing SSH keys across multiple machines is painful:

跨设备管理 SSH 密钥很麻烦：

- Keys scattered across `~/.ssh/` on different devices / 密钥散落在不同设备的 `~/.ssh/` 目录
- Copying private keys via insecure channels (email, chat, USB) / 通过不安全的渠道复制私钥
- Forgetting which key authenticates to which server / 忘记哪把密钥对应哪台服务器
- No way to access your servers from your phone / 手机上无法访问你的服务器

HostSync solves this: **one encrypted vault** for all your SSH keys, passwords, and server configs — synced securely across every device you own.

HostSync 解决这些问题：**一个加密保险库**存放所有 SSH 密钥、密码和服务器配置 — 在你的所有设备间安全同步。

## Security Model / 安全模型

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

## Installation / 安装

Download from [Releases](https://github.com/fjh1997/host-sync/releases):

从 [Releases](https://github.com/fjh1997/host-sync/releases) 下载：

| Platform / 平台 | x86_64 | ARM64 |
|----------|--------|-------|
| Windows  | `hostsync-windows-x64.zip` | `hostsync-windows-arm64.zip` |
| Linux    | `hostsync-linux-x64.tar.gz` | `hostsync-linux-arm64.tar.gz` |
| macOS    | `HostSync-macos-x64.dmg` | `HostSync-macos-arm64.dmg` |
| Android  | `HostSync.apk` (all ABIs) | |
| iOS      | AltStore (see below) | |

### Windows

Build the desktop app and create a Start menu shortcut for the current user:

构建桌面端并为当前用户创建开始菜单快捷方式：

```powershell
cargo build --release -p hostsync-desktop
powershell -ExecutionPolicy Bypass -File scripts\install-windows-start-menu.ps1
```

After this, searching `hostsync` or `HostSync` in the Start menu launches the app.

完成后，在开始菜单搜索 `hostsync` 或 `HostSync` 即可启动应用。

To build a Windows installer that installs to `C:\Program Files\HostSync` and creates
the Start menu entry:

构建会安装到 `C:\Program Files\HostSync` 并创建开始菜单入口的 Windows 安装器：

```powershell
cargo install cargo-packager --locked
cargo build --release -p hostsync-desktop
cargo packager -f nsis --release
```

Run the generated installer as Administrator for the same per-machine behavior as
apps installed under `Program Files`.

以管理员身份运行生成的安装器，即可获得和安装到 `Program Files` 的应用一致的全局安装行为。

### macOS

Download the `.dmg`, open it, drag **HostSync** to **Applications**.

下载 `.dmg`，打开后将 HostSync 拖入 Applications 即可。

> First launch: right-click → Open to bypass Gatekeeper / 首次启动：右键 → 打开 以绕过 Gatekeeper

### iOS (AltStore)

1. Install [AltStore](https://altstore.io/) on your iOS device / 在 iOS 设备上安装 AltStore
2. Open AltStore → **Browse** → **Sources** → tap **+** / 打开 AltStore → 浏览 → 源 → 点击 +
3. Add source URL / 添加源地址：
   ```
   https://fjh1997.github.io/host-sync/source.json
   ```
4. Find **HostSync** and install / 在源中找到 HostSync 并安装

## SSH Key Management / SSH 密钥管理

HostSync supports two ways to store SSH keys per server:

HostSync 支持两种方式存储每台服务器的 SSH 密钥：

| Method / 方式 | Field / 字段 | Use case / 使用场景 |
|---|---|---|
| **IdentityFile path** | `~/.ssh/id_rsa` | Desktop — references key file on disk / 桌面端引用磁盘上的密钥文件 |
| **Inline private key** | PEM content in vault | Mobile & cross-device — encrypted and synced / 移动端跨设备加密同步 |

Both can be set simultaneously: desktop uses the file path, mobile uses the inline key.

两者可以同时设置：桌面端使用文件路径，移动端使用内联密钥。

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

## Architecture / 架构

```
host-sync/
├── crates/
│   ├── hostsync-core/        # Rust core (crypto, SSH config, storage, OAuth, sync)
│   │                         # → .dll/.so/.dylib (desktop), .so (Android JNI), .a (iOS FFI)
│   └── hostsync-desktop/     # iced GUI → native Win/Linux/Mac binary
├── mobile/
│   ├── android/              # Kotlin + Jetpack Compose + JNI → Rust .so
│   └── ios/                  # Swift + SwiftUI + C FFI → Rust .a
└── .github/workflows/       # CI/CD
```

## Build from Source / 从源码构建

```bash
# Desktop (current platform)
cargo build --release -p hostsync-desktop

# Android shared library
cargo ndk -t arm64-v8a build --release -p hostsync-core

# iOS static library
cargo build --release --target aarch64-apple-ios -p hostsync-core
```

## CI/CD

| Trigger | Action |
|---------|--------|
| Push to `main` / PR | test + clippy + build all platforms → **pre-release** |
| Push `v*` tag | test + build → formal **Release** + update AltStore source.json |

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=fjh1997%2Fhost-sync&type=Date)](https://www.star-history.com/#fjh1997/host-sync&Date)

---

## Claude-Assisted Development / Claude 辅助开发

This project was developed with **Claude** (Anthropic) using **Claude Code** CLI.

本项目由 **Claude**（Anthropic）通过 **Claude Code** CLI 辅助开发。

<details>
<summary>All Prompts Used / 使用的所有提示词</summary>

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

> Claude initially proposed Go; user asked "为什么用go不用rust？", Claude agreed Rust is better for the size/memory requirements.

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

</details>

## License / 许可证

MIT
