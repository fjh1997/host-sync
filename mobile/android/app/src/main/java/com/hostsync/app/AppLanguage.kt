package com.hostsync.app

import android.content.Context
import java.util.Locale

enum class AppLanguageSetting(val storedValue: String) {
    SYSTEM("system"),
    ENGLISH("en"),
    CHINESE("zh");

    companion object {
        fun fromStoredValue(value: String?): AppLanguageSetting {
            return when (value?.lowercase(Locale.ROOT)) {
                ENGLISH.storedValue -> ENGLISH
                CHINESE.storedValue, "zh-cn", "zh-hans" -> CHINESE
                else -> SYSTEM
            }
        }
    }
}

enum class AppLanguage {
    ENGLISH,
    CHINESE,
}

data class AppStrings(
    val language: AppLanguage,
    val subtitle: String,
    val signInWithGitHub: String,
    val settings: String,
    val languageTitle: String,
    val followSystem: String,
    val languageEnglish: String,
    val languageChinese: String,
    val back: String,
    val logout: String,
    val noServersYet: String,
    val connect: String,
    val commandHint: String,
    val send: String,
    val openAndEnterCode: String,
    val waitingForAuthorization: String,
    val openGitHub: String,
    val tapToCopy: String,
    val codeCopied: String,
    val proxySettings: String,
    val proxyHost: String,
    val proxyPort: String,
    val syncPassphraseTitle: String,
    val syncPassphraseDesc: String,
    val syncPassphrase: String,
    val syncPassphraseError: String,
    val confirm: String,
    val copy: String,
    val edit: String,
    val delete: String,
    val deleteConfirm: String,
    val cancel: String,
    val add: String,
    val name: String,
    val host: String,
    val port: String,
    val username: String,
    val password: String,
    val passwordBadge: String,
    val sshKey: String,
    val identityFile: String,
    val notes: String,
    val save: String,
    val copiedToClipboard: String,
    val termuxTitle: String,
    val termuxDesc: String,
    val openSettings: String,
    val connectAnyway: String,
) {
    fun systemLanguageDetected(name: String): String {
        return when (language) {
            AppLanguage.ENGLISH -> "Current system language: $name"
            AppLanguage.CHINESE -> "当前系统语言：$name"
        }
    }

    fun connectingTo(host: String, port: Int): String {
        return when (language) {
            AppLanguage.ENGLISH -> "Connecting to $host:$port...\n"
            AppLanguage.CHINESE -> "正在连接到 $host:$port...\n"
        }
    }

    fun connectionClosed(): String {
        return when (language) {
            AppLanguage.ENGLISH -> "\n[Connection closed]\n"
            AppLanguage.CHINESE -> "\n[连接已关闭]\n"
        }
    }

    fun error(message: String): String {
        return when (language) {
            AppLanguage.ENGLISH -> "\n[Error: $message]\n"
            AppLanguage.CHINESE -> "\n[错误：$message]\n"
        }
    }
}

object LanguagePrefs {
    private const val PREFS_NAME = "hostsync_settings"
    private const val KEY_LANGUAGE = "language"

    fun load(context: Context): AppLanguageSetting {
        val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        return AppLanguageSetting.fromStoredValue(
            prefs.getString(KEY_LANGUAGE, AppLanguageSetting.SYSTEM.storedValue)
        )
    }

    fun save(context: Context, setting: AppLanguageSetting) {
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit()
            .putString(KEY_LANGUAGE, setting.storedValue)
            .apply()
    }

    fun resolve(context: Context, setting: AppLanguageSetting): AppLanguage {
        return when (setting) {
            AppLanguageSetting.SYSTEM -> fromLocale(currentLocale(context))
            AppLanguageSetting.ENGLISH -> AppLanguage.ENGLISH
            AppLanguageSetting.CHINESE -> AppLanguage.CHINESE
        }
    }

    fun strings(context: Context, setting: AppLanguageSetting): AppStrings {
        return when (resolve(context, setting)) {
            AppLanguage.ENGLISH -> AppStrings(
                language = AppLanguage.ENGLISH,
                subtitle = "Manage your Linux servers & SSH keys",
                signInWithGitHub = "Sign in with GitHub",
                settings = "Settings",
                languageTitle = "Language",
                followSystem = "Follow System",
                languageEnglish = "English",
                languageChinese = "Chinese",
                back = "Back",
                logout = "Logout",
                noServersYet = "No servers yet",
                connect = "Connect",
                commandHint = "Type command...",
                send = "Send",
                openAndEnterCode = "Open GitHub and enter code:",
                waitingForAuthorization = "Waiting for authorization...",
                openGitHub = "Open GitHub",
                tapToCopy = "Tap to copy",
                codeCopied = "Code copied!",
                proxySettings = "Proxy (SOCKS5)",
                proxyHost = "Host (e.g. 192.168.1.5)",
                proxyPort = "Port (e.g. 10808)",
                syncPassphraseTitle = "Sync Passphrase",
                syncPassphraseDesc = "Enter your sync passphrase to decrypt server data from GitHub Gist.",
                syncPassphrase = "Sync Passphrase",
                syncPassphraseError = "Wrong passphrase or no data in Gist",
                confirm = "Confirm",
                copy = "Copy",
                edit = "Edit",
                delete = "Delete",
                deleteConfirm = "Are you sure you want to delete this server?",
                cancel = "Cancel",
                add = "Add Server",
                name = "Name",
                host = "Host",
                port = "Port",
                username = "Username",
                password = "Password",
                passwordBadge = "pw",
                sshKey = "SSH Key",
                identityFile = "Identity File",
                notes = "Notes",
                save = "Save",
                copiedToClipboard = "Command copied to clipboard",
                termuxTitle = "Termux Required",
                termuxDesc = "Please install Termux, then in the app info page:\n1. Enable \"Allow external apps\" in Termux settings\n2. Enable \"Associated launch\" / \"Cross-app launch\" permission (MIUI/ColorOS/HarmonyOS)",
                openSettings = "Termux App Info",
                connectAnyway = "Connect Anyway",
            )
            AppLanguage.CHINESE -> AppStrings(
                language = AppLanguage.CHINESE,
                subtitle = "管理你的 Linux 服务器与 SSH 密钥",
                signInWithGitHub = "使用 GitHub 登录",
                settings = "设置",
                languageTitle = "语言",
                followSystem = "跟随系统",
                languageEnglish = "英文",
                languageChinese = "中文",
                back = "返回",
                logout = "退出登录",
                noServersYet = "还没有服务器",
                connect = "连接",
                commandHint = "输入命令...",
                send = "发送",
                openAndEnterCode = "打开 GitHub 并输入验证码：",
                waitingForAuthorization = "等待授权中...",
                openGitHub = "打开 GitHub",
                tapToCopy = "点击复制",
                codeCopied = "已复制验证码",
                proxySettings = "代理 (SOCKS5)",
                proxyHost = "主机 (如 192.168.1.5)",
                proxyPort = "端口 (如 10808)",
                syncPassphraseTitle = "同步口令",
                syncPassphraseDesc = "请输入同步口令以解密 GitHub Gist 中的服务器数据。",
                syncPassphrase = "同步口令",
                syncPassphraseError = "口令错误或 Gist 中无数据",
                confirm = "确认",
                copy = "复制",
                edit = "编辑",
                delete = "删除",
                deleteConfirm = "确定要删除这个服务器吗？",
                cancel = "取消",
                add = "添加服务器",
                name = "名称",
                host = "主机",
                port = "端口",
                username = "用户名",
                password = "密码",
                passwordBadge = "密码",
                sshKey = "SSH 密钥",
                identityFile = "密钥文件",
                notes = "备注",
                save = "保存",
                copiedToClipboard = "命令已复制到剪贴板",
                termuxTitle = "需要安装 Termux",
                termuxDesc = "请先安装 Termux，然后在应用详情页中：\n1. 开启 Termux 设置中的「允许外部应用」\n2. 国产系统还需开启「关联启动」权限（MIUI/ColorOS/鸿蒙等）",
                openSettings = "Termux 应用详情",
                connectAnyway = "仍然连接",
            )
        }
    }

    fun currentSystemLanguageName(context: Context, strings: AppStrings): String {
        return when (fromLocale(currentLocale(context))) {
            AppLanguage.ENGLISH -> strings.languageEnglish
            AppLanguage.CHINESE -> strings.languageChinese
        }
    }

    private fun currentLocale(context: Context): Locale {
        val locales = context.resources.configuration.locales
        return if (!locales.isEmpty) locales[0] else Locale.getDefault()
    }

    private fun fromLocale(locale: Locale): AppLanguage {
        return if (locale.language.lowercase(Locale.ROOT).startsWith("zh")) {
            AppLanguage.CHINESE
        } else {
            AppLanguage.ENGLISH
        }
    }
}
