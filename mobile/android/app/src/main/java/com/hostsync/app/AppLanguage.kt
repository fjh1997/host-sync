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
