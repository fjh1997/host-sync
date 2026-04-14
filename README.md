# HostSync

Cross-platform Linux server manager with SSH config compatibility.

Manage, sync and connect to all your Linux servers from Windows, Linux, macOS, Android, and iOS.

## Features

- **GitHub OAuth Login** — Sign in with GitHub, no separate account needed
- **AES-256 Encrypted Storage** — Passwords and private keys are encrypted locally with PBKDF2 key derivation
- **Cloud Sync via GitHub Gists** — Encrypted server data synced across all your devices through a private Gist
- **SSH Config Compatible** — Import/export standard `~/.ssh/config` format; generated configs work with `ssh -F`
- **Desktop: Native Terminal** — Clicking "Connect" opens your system terminal (Windows Terminal / Terminal.app / gnome-terminal) with the SSH command
- **Mobile: Built-in Terminal** — Android & iOS include a full interactive SSH shell via dartssh2 + xterm
- **Lightweight** — Flutter-based, small binary footprint, low memory usage

## Installation

### Desktop

Download from [Releases](https://github.com/fjh1997/host-sync/releases):

| Platform | File |
|----------|------|
| Windows  | `HostSync-windows-x64.zip` |
| Linux    | `HostSync-linux-x64.tar.gz` |
| macOS    | `HostSync-macos.zip` |

### Android

Download `app-release.apk` from [Releases](https://github.com/fjh1997/host-sync/releases) and install.

### iOS (AltStore)

1. Install [AltStore](https://altstore.io/) on your iOS device
2. Open AltStore, go to **Browse** → **Sources** → tap **+** (top-left)
3. Add source URL:
   ```
   https://fjh1997.github.io/host-sync/source.json
   ```
4. Find **HostSync** in the source and install

## Setup

### 1. Create a GitHub OAuth App

1. Go to [GitHub Settings → Developer settings → OAuth Apps → New OAuth App](https://github.com/settings/applications/new)
2. Fill in:
   - **Application name**: HostSync
   - **Homepage URL**: `https://github.com/fjh1997/host-sync`
   - **Authorization callback URL**: `http://localhost:9876/callback`
3. Note the **Client ID** and **Client Secret**
4. Edit `lib/services/auth_service.dart` and fill in the constants:
   ```dart
   static const clientId = 'YOUR_GITHUB_CLIENT_ID';
   static const clientSecret = 'YOUR_GITHUB_CLIENT_SECRET';
   ```

### 2. Build from source

```bash
# Install Flutter: https://docs.flutter.dev/get-started/install
flutter pub get

# Build for your platform
flutter build windows --release
flutter build linux --release
flutter build macos --release
flutter build apk --release
flutter build ios --release --no-codesign
```

## SSH Config Compatibility

HostSync stores server configurations in a format compatible with OpenSSH `~/.ssh/config`.

### Import

- **From system**: Import directly from `~/.ssh/config` (desktop only)
- **Paste text**: Paste SSH config content on any platform

### Export

- **Copy to clipboard**: Generates standard SSH config text
- **Merge into `~/.ssh/config`**: Appends/updates HostSync-managed entries while preserving existing ones

### Generated format

```ssh-config
#@HostSync-Id a1b2c3d4-e5f6-...
#@HostSync-AuthType key
#@HostSync-CreatedAt 2026-04-14T15:00:00.000
#@HostSync-UpdatedAt 2026-04-14T15:00:00.000
Host prod-web
    HostName 10.0.1.50
    Port 2222
    User deploy
    IdentityFile ~/.ssh/prod_key
```

HostSync metadata is stored as `#@HostSync-*` comments — OpenSSH ignores them, so the file works with `ssh -F` directly.

## Architecture

```
lib/
├── main.dart                          # Entry point, theme, routing
├── models/
│   └── server.dart                    # Server data model with SSH config support
├── services/
│   ├── auth_service.dart              # GitHub OAuth flow
│   ├── crypto_service.dart            # AES-256-CBC + PBKDF2 encryption
│   ├── storage_service.dart           # Local encrypted storage
│   ├── sync_service.dart              # GitHub Gist cloud sync
│   ├── ssh_config_service.dart        # SSH config parser/generator
│   └── terminal_service.dart          # Desktop native terminal launcher
├── screens/
│   ├── login_screen.dart              # GitHub OAuth login page
│   ├── home_screen.dart               # Server list + import/export
│   ├── server_form_screen.dart        # Add/edit form with SSH config preview
│   └── terminal_screen.dart           # Mobile built-in SSH terminal
└── widgets/
    └── server_card.dart               # Server list card widget
```

## CI/CD

- **Every push to `main` / Pull Request**: Runs analyze + test, builds all 5 platforms, uploads to a `nightly-*` pre-release
- **Every `v*` tag**: Runs analyze + test, builds all 5 platforms, creates a formal GitHub Release, updates AltStore `source.json` on GitHub Pages

---

## Claude-Assisted Development

This project was developed with the assistance of **Claude** (Anthropic's AI assistant) using **Claude Code** CLI.

### Prompts Used

Below are all prompts given to Claude during the development of this project, in chronological order:

#### Prompt 1 — Initial project creation

```
开发一个图形化软件，支持windows，linux，mac，安卓，ios，使用github oauth登录即可，
能够添加管理并同步你的所有的linux服务器的的域名，密码，密钥。点击连接电脑端不是使用
内置终端，而是调用系统原生终端进行连接。安卓端与ios端则可以使用内置shell界面连接，
使用任何语言都可以，要求体积足够小，内存占用足够轻量化。
```

> Translation: Develop a GUI application supporting Windows, Linux, Mac, Android, iOS. Use GitHub OAuth login. Manage and sync all your Linux servers' domains, passwords, and keys. On desktop, clicking connect should open the system's native terminal for SSH connection instead of a built-in terminal. On Android and iOS, use a built-in shell interface. Any language is fine, but the binary should be small and memory usage lightweight.

#### Prompt 2 — SSH config compatibility

```
配置文件要与 SSH config 文件格式兼容
```

> Translation: Configuration files must be compatible with SSH config file format.

#### Prompt 3 — GitHub push, CI/CD, AltStore, README

```
帮我推送到github，要求每次commit或者pr，以及加v的tag的release都要触发github action
构建测试，commit的构建需要推送到pre-release，加vtag的release推送到正式release，
readme里面要写上claude辅助开发以及本次涉及到的所有提示词。ios要支持altstore安装，
例子：AltStore 安装
在 iOS 设备上安装 AltStore
打开 AltStore，进入 Browse → Sources → 点击左上角 +
粘贴源地址：
https://lingyan000.github.io/fluxdo/source.json
在源中找到 FluxDO 并安装
```

> Translation: Push to GitHub. Every commit/PR and v-tag release should trigger GitHub Actions build and test. Commit builds should be pushed to pre-release, v-tag releases to formal releases. The README should include Claude-assisted development credits and all prompts used. iOS should support AltStore installation (with example of how AltStore source installation works).

## License

MIT
