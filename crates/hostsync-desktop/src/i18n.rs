#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageSetting {
    System,
    English,
    Chinese,
}

impl LanguageSetting {
    pub fn from_storage(value: Option<&str>) -> Self {
        match value
            .unwrap_or("system")
            .trim()
            .to_ascii_lowercase()
            .as_str()
        {
            "en" | "english" => Self::English,
            "zh" | "zh-cn" | "zh-hans" | "chinese" => Self::Chinese,
            _ => Self::System,
        }
    }

    pub fn as_storage_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::English => "en",
            Self::Chinese => "zh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
}

pub fn system_language() -> Language {
    sys_locale::get_locale()
        .as_deref()
        .map(Language::from_locale)
        .unwrap_or(Language::English)
}

pub fn resolve_language(setting: LanguageSetting) -> Language {
    match setting {
        LanguageSetting::System => system_language(),
        LanguageSetting::English => Language::English,
        LanguageSetting::Chinese => Language::Chinese,
    }
}

impl Language {
    fn from_locale(locale: &str) -> Self {
        if locale.trim().to_ascii_lowercase().starts_with("zh") {
            Self::Chinese
        } else {
            Self::English
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn user_fallback(self) -> &'static str {
        match self.language {
            Language::English => "User",
            Language::Chinese => "用户",
        }
    }

    pub fn manage_servers(self) -> &'static str {
        match self.language {
            Language::English => "Manage your Linux servers & SSH keys",
            Language::Chinese => "管理你的 Linux 服务器与 SSH 密钥",
        }
    }

    pub fn enter_code_on_github(self) -> &'static str {
        match self.language {
            Language::English => "Enter this code on GitHub:",
            Language::Chinese => "在 GitHub 中输入此代码：",
        }
    }

    pub fn copied_to_clipboard(self) -> &'static str {
        match self.language {
            Language::English => "(copied to clipboard)",
            Language::Chinese => "（已复制到剪贴板）",
        }
    }

    pub fn waiting_for_authorization(self) -> &'static str {
        match self.language {
            Language::English => "Waiting for authorization...",
            Language::Chinese => "等待授权中...",
        }
    }

    pub fn requesting(self) -> &'static str {
        match self.language {
            Language::English => "Requesting...",
            Language::Chinese => "请求中...",
        }
    }

    pub fn sign_in_with_github(self) -> &'static str {
        match self.language {
            Language::English => "Sign in with GitHub",
            Language::Chinese => "使用 GitHub 登录",
        }
    }

    pub fn settings(self) -> &'static str {
        match self.language {
            Language::English => "Settings",
            Language::Chinese => "设置",
        }
    }

    pub fn upload(self) -> &'static str {
        match self.language {
            Language::English => "Upload",
            Language::Chinese => "上传",
        }
    }

    pub fn download(self) -> &'static str {
        match self.language {
            Language::English => "Download",
            Language::Chinese => "下载",
        }
    }

    pub fn import_ssh(self) -> &'static str {
        match self.language {
            Language::English => "Import SSH",
            Language::Chinese => "导入 SSH",
        }
    }

    pub fn import_text(self) -> &'static str {
        match self.language {
            Language::English => "Import Text",
            Language::Chinese => "导入文本",
        }
    }

    pub fn export_copy(self) -> &'static str {
        match self.language {
            Language::English => "Export Copy",
            Language::Chinese => "导出复制",
        }
    }

    pub fn export_ssh(self) -> &'static str {
        match self.language {
            Language::English => "Export SSH",
            Language::Chinese => "导出 SSH",
        }
    }

    pub fn logout(self) -> &'static str {
        match self.language {
            Language::English => "Logout",
            Language::Chinese => "退出登录",
        }
    }

    pub fn search_servers(self) -> &'static str {
        match self.language {
            Language::English => "Search servers...",
            Language::Chinese => "搜索服务器...",
        }
    }

    pub fn no_servers_yet(self) -> &'static str {
        match self.language {
            Language::English => "No servers yet",
            Language::Chinese => "还没有服务器",
        }
    }

    pub fn no_matching_servers(self) -> &'static str {
        match self.language {
            Language::English => "No matching servers",
            Language::Chinese => "没有匹配的服务器",
        }
    }

    pub fn add_first_server(self) -> &'static str {
        match self.language {
            Language::English => "Add your first server",
            Language::Chinese => "添加第一台服务器",
        }
    }

    pub fn add_server(self) -> &'static str {
        match self.language {
            Language::English => "+  Add Server",
            Language::Chinese => "+  添加服务器",
        }
    }

    pub fn connect(self) -> &'static str {
        match self.language {
            Language::English => "Connect",
            Language::Chinese => "连接",
        }
    }

    pub fn copy(self) -> &'static str {
        match self.language {
            Language::English => "Copy",
            Language::Chinese => "复制",
        }
    }

    pub fn edit(self) -> &'static str {
        match self.language {
            Language::English => "Edit",
            Language::Chinese => "编辑",
        }
    }

    pub fn delete(self) -> &'static str {
        match self.language {
            Language::English => "Delete",
            Language::Chinese => "删除",
        }
    }

    pub fn key_badge(self) -> &'static str {
        match self.language {
            Language::English => "[key]",
            Language::Chinese => "[密钥]",
        }
    }

    pub fn password_badge(self) -> &'static str {
        match self.language {
            Language::English => "[pw]",
            Language::Chinese => "[密码]",
        }
    }

    pub fn edit_server(self) -> &'static str {
        match self.language {
            Language::English => "Edit Server",
            Language::Chinese => "编辑服务器",
        }
    }

    pub fn add_server_title(self) -> &'static str {
        match self.language {
            Language::English => "Add Server",
            Language::Chinese => "添加服务器",
        }
    }

    pub fn password(self) -> &'static str {
        match self.language {
            Language::English => "Password",
            Language::Chinese => "密码",
        }
    }

    pub fn show(self) -> &'static str {
        match self.language {
            Language::English => "Show",
            Language::Chinese => "显示",
        }
    }

    pub fn hide(self) -> &'static str {
        match self.language {
            Language::English => "Hide",
            Language::Chinese => "隐藏",
        }
    }

    pub fn ssh_key(self) -> &'static str {
        match self.language {
            Language::English => "SSH Key",
            Language::Chinese => "SSH 密钥",
        }
    }

    pub fn basic_information(self) -> &'static str {
        match self.language {
            Language::English => "Basic Information",
            Language::Chinese => "基本信息",
        }
    }

    pub fn host_alias(self) -> &'static str {
        match self.language {
            Language::English => "Host Alias (Name)",
            Language::Chinese => "主机别名（名称）",
        }
    }

    pub fn host_alias_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "e.g. prod-web",
            Language::Chinese => "例如：prod-web",
        }
    }

    pub fn hostname(self) -> &'static str {
        match self.language {
            Language::English => "HostName",
            Language::Chinese => "主机地址",
        }
    }

    pub fn hostname_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "e.g. 1.2.3.4",
            Language::Chinese => "例如：1.2.3.4",
        }
    }

    pub fn port(self) -> &'static str {
        match self.language {
            Language::English => "Port",
            Language::Chinese => "端口",
        }
    }

    pub fn user(self) -> &'static str {
        match self.language {
            Language::English => "User",
            Language::Chinese => "用户",
        }
    }

    pub fn authentication(self) -> &'static str {
        match self.language {
            Language::English => "Authentication",
            Language::Chinese => "认证方式",
        }
    }

    pub fn identity_file(self) -> &'static str {
        match self.language {
            Language::English => "IdentityFile (path)",
            Language::Chinese => "IdentityFile（路径）",
        }
    }

    pub fn browse(self) -> &'static str {
        match self.language {
            Language::English => "Browse",
            Language::Chinese => "浏览",
        }
    }

    pub fn select_ssh_key_file(self) -> &'static str {
        match self.language {
            Language::English => "Select SSH Key File",
            Language::Chinese => "选择 SSH 密钥文件",
        }
    }

    pub fn private_key_optional(self) -> &'static str {
        match self.language {
            Language::English => "Private Key Content (optional)",
            Language::Chinese => "私钥内容（可选）",
        }
    }

    pub fn key_passphrase_optional(self) -> &'static str {
        match self.language {
            Language::English => "Key Passphrase (optional)",
            Language::Chinese => "密钥口令（可选）",
        }
    }

    pub fn notes_optional(self) -> &'static str {
        match self.language {
            Language::English => "Notes (optional)",
            Language::Chinese => "备注（可选）",
        }
    }

    pub fn ssh_config_preview(self) -> &'static str {
        match self.language {
            Language::English => "SSH Config Preview",
            Language::Chinese => "SSH 配置预览",
        }
    }

    pub fn cancel(self) -> &'static str {
        match self.language {
            Language::English => "Cancel",
            Language::Chinese => "取消",
        }
    }

    pub fn save(self) -> &'static str {
        match self.language {
            Language::English => "Save",
            Language::Chinese => "保存",
        }
    }

    pub fn preview_alias_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "<alias>",
            Language::Chinese => "<别名>",
        }
    }

    pub fn preview_hostname_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "<hostname>",
            Language::Chinese => "<主机地址>",
        }
    }

    pub fn paste_ssh_config(self) -> &'static str {
        match self.language {
            Language::English => "Paste SSH Config",
            Language::Chinese => "粘贴 SSH 配置",
        }
    }

    pub fn import(self) -> &'static str {
        match self.language {
            Language::English => "Import",
            Language::Chinese => "导入",
        }
    }

    pub fn language(self) -> &'static str {
        match self.language {
            Language::English => "Language",
            Language::Chinese => "语言",
        }
    }

    pub fn language_follow_system(self) -> &'static str {
        match self.language {
            Language::English => "Follow System",
            Language::Chinese => "跟随系统",
        }
    }

    pub fn language_english(self) -> &'static str {
        match self.language {
            Language::English => "English",
            Language::Chinese => "英文",
        }
    }

    pub fn language_chinese(self) -> &'static str {
        match self.language {
            Language::English => "Chinese",
            Language::Chinese => "中文",
        }
    }

    pub fn system_language_detected(self, name: &str) -> String {
        match self.language {
            Language::English => format!("Current system language: {}", name),
            Language::Chinese => format!("当前系统语言：{}", name),
        }
    }

    pub fn proxy_settings(self) -> &'static str {
        match self.language {
            Language::English => "Proxy Settings",
            Language::Chinese => "代理设置",
        }
    }

    pub fn current_proxy(self, proxy: &str) -> String {
        match self.language {
            Language::English => format!("Current proxy: {}", proxy),
            Language::Chinese => format!("当前代理：{}", proxy),
        }
    }

    pub fn no_proxy_configured(self) -> &'static str {
        match self.language {
            Language::English => "No proxy configured (direct connection)",
            Language::Chinese => "未配置代理（直连）",
        }
    }

    pub fn proxy_url_label(self) -> &'static str {
        match self.language {
            Language::English => "HTTP/SOCKS5 Proxy URL",
            Language::Chinese => "HTTP/SOCKS5 代理地址",
        }
    }

    pub fn proxy_url_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "e.g. http://127.0.0.1:10808 or socks5://127.0.0.1:1080",
            Language::Chinese => "例如：http://127.0.0.1:10808 或 socks5://127.0.0.1:1080",
        }
    }

    pub fn proxy_direct_hint(self) -> &'static str {
        match self.language {
            Language::English => "Leave empty to use direct connection.",
            Language::Chinese => "留空则使用直连。",
        }
    }

    pub fn set_sync_passphrase(self) -> &'static str {
        match self.language {
            Language::English => "Set Sync Passphrase",
            Language::Chinese => "设置同步口令",
        }
    }

    pub fn enter_sync_passphrase(self) -> &'static str {
        match self.language {
            Language::English => "Enter Sync Passphrase",
            Language::Chinese => "输入同步口令",
        }
    }

    pub fn sync_passphrase_description_new(self) -> &'static str {
        match self.language {
            Language::English => {
                "Set a passphrase to encrypt your server data.\nYou'll need this same passphrase on other devices."
            }
            Language::Chinese => "设置一个口令来加密你的服务器数据。\n你需要在其他设备上使用同一个口令。",
        }
    }

    pub fn sync_passphrase_description_existing(self) -> &'static str {
        match self.language {
            Language::English => "Enter the sync passphrase you set on your other device.",
            Language::Chinese => "输入你在其他设备上设置的同步口令。",
        }
    }

    pub fn sync_passphrase(self) -> &'static str {
        match self.language {
            Language::English => "Sync Passphrase",
            Language::Chinese => "同步口令",
        }
    }

    pub fn enter_passphrase_placeholder(self) -> &'static str {
        match self.language {
            Language::English => "Enter passphrase...",
            Language::Chinese => "输入口令...",
        }
    }

    pub fn confirm(self) -> &'static str {
        match self.language {
            Language::English => "Confirm",
            Language::Chinese => "确认",
        }
    }

    pub fn syncing_from_cloud(self) -> &'static str {
        match self.language {
            Language::English => "Syncing from cloud...",
            Language::Chinese => "正在从云端同步...",
        }
    }

    pub fn requesting_device_code(self) -> &'static str {
        match self.language {
            Language::English => "Requesting device code...",
            Language::Chinese => "正在请求设备代码...",
        }
    }

    pub fn open_and_enter_code(self, verification_uri: &str, code: &str) -> String {
        match self.language {
            Language::English => format!("Open {} and enter code: {}", verification_uri, code),
            Language::Chinese => format!("打开 {} 并输入代码：{}", verification_uri, code),
        }
    }

    pub fn failed(self, error: &str) -> String {
        match self.language {
            Language::English => format!("Failed: {}", error),
            Language::Chinese => format!("失败：{}", error),
        }
    }

    pub fn login_failed(self, error: &str) -> String {
        match self.language {
            Language::English => format!("Login failed: {}", error),
            Language::Chinese => format!("登录失败：{}", error),
        }
    }

    pub fn command_copied(self) -> &'static str {
        match self.language {
            Language::English => "Command copied to clipboard",
            Language::Chinese => "命令已复制到剪贴板",
        }
    }

    pub fn syncing(self) -> &'static str {
        match self.language {
            Language::English => "Syncing...",
            Language::Chinese => "同步中...",
        }
    }

    pub fn uploading(self) -> &'static str {
        match self.language {
            Language::English => "Uploading...",
            Language::Chinese => "上传中...",
        }
    }

    pub fn uploaded_to_cloud(self) -> &'static str {
        match self.language {
            Language::English => "Uploaded to cloud",
            Language::Chinese => "已上传到云端",
        }
    }

    pub fn upload_failed(self, error: &str) -> String {
        match self.language {
            Language::English => format!("Upload failed: {}", error),
            Language::Chinese => format!("上传失败：{}", error),
        }
    }

    pub fn downloading(self) -> &'static str {
        match self.language {
            Language::English => "Downloading...",
            Language::Chinese => "下载中...",
        }
    }

    pub fn downloaded_from_cloud(self) -> &'static str {
        match self.language {
            Language::English => "Downloaded from cloud",
            Language::Chinese => "已从云端下载",
        }
    }

    pub fn download_failed(self, error: &str) -> String {
        match self.language {
            Language::English => format!("Download failed: {}", error),
            Language::Chinese => format!("下载失败：{}", error),
        }
    }

    pub fn imported_hosts_syncing(self, count: usize) -> String {
        match self.language {
            Language::English => format!("Imported {} host(s), syncing...", count),
            Language::Chinese => format!("已导入 {} 个主机，正在同步...", count),
        }
    }

    pub fn no_new_hosts_to_import(self) -> &'static str {
        match self.language {
            Language::English => "No new hosts to import",
            Language::Chinese => "没有可导入的新主机",
        }
    }

    pub fn ssh_config_copied(self) -> &'static str {
        match self.language {
            Language::English => "SSH config copied to clipboard",
            Language::Chinese => "SSH 配置已复制到剪贴板",
        }
    }

    pub fn merged_into_system_config(self) -> &'static str {
        match self.language {
            Language::English => "Merged into ~/.ssh/config",
            Language::Chinese => "已合并到 ~/.ssh/config",
        }
    }

    pub fn export_failed(self, error: &str) -> String {
        match self.language {
            Language::English => format!("Export failed: {}", error),
            Language::Chinese => format!("导出失败：{}", error),
        }
    }

    pub fn proxy_cleared(self) -> &'static str {
        match self.language {
            Language::English => "Proxy cleared",
            Language::Chinese => "代理已清除",
        }
    }

    pub fn proxy_set_to(self, proxy: &str) -> String {
        match self.language {
            Language::English => format!("Proxy set to {}", proxy),
            Language::Chinese => format!("代理已设置为 {}", proxy),
        }
    }

    pub fn passphrase_cannot_be_empty(self) -> &'static str {
        match self.language {
            Language::English => "Passphrase cannot be empty",
            Language::Chinese => "口令不能为空",
        }
    }
}
