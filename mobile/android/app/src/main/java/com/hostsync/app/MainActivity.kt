package com.hostsync.app

import android.os.Bundle
import android.content.Intent
import android.net.Uri
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import org.json.JSONArray
import org.json.JSONObject

private enum class AppScreen {
    LOGIN,
    HOME,
    SETTINGS,
}

class MainActivity : ComponentActivity() {
    companion object {
        init {
            System.loadLibrary("hostsync_core")
        }
    }

    // JNI bindings to Rust FFI
    private external fun hostsyncLoadServersJson(): String
    private external fun hostsyncSaveServersJson(json: String): Int
    private external fun hostsyncParseSshConfig(config: String): String
    private external fun hostsyncGenerateSshConfig(): String
    private external fun hostsyncIsLoggedIn(): Int
    private external fun hostsyncGetGithubUsername(): String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                var servers by remember { mutableStateOf(loadServers()) }
                var screen by remember {
                    mutableStateOf(if (hostsyncIsLoggedIn() == 1) AppScreen.HOME else AppScreen.LOGIN)
                }
                var settingsReturnScreen by remember { mutableStateOf(screen) }
                var languageSetting by remember { mutableStateOf(LanguagePrefs.load(this)) }
                val strings = LanguagePrefs.strings(this, languageSetting)

                Surface(modifier = Modifier.fillMaxSize()) {
                    when (screen) {
                        AppScreen.LOGIN -> LoginScreen(
                            strings = strings,
                            onLogin = {
                                // Device Flow: open GitHub device auth page
                                val url = "https://github.com/login/device"
                                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                            },
                            onOpenSettings = {
                                settingsReturnScreen = AppScreen.LOGIN
                                screen = AppScreen.SETTINGS
                            },
                        )
                        AppScreen.HOME -> HomeScreen(
                            strings = strings,
                            servers = servers,
                            onConnect = { server ->
                                // Launch built-in SSH terminal activity
                                val intent = Intent(this, TerminalActivity::class.java).apply {
                                    putExtra("host", server.getString("host"))
                                    putExtra("port", server.optInt("port", 22))
                                    putExtra("username", server.getString("username"))
                                    putExtra("authType", server.optString("auth_type", "password"))
                                    putExtra("password", server.optString("password", ""))
                                    putExtra("privateKey", server.optString("private_key", ""))
                                }
                                startActivity(intent)
                            },
                            onOpenSettings = {
                                settingsReturnScreen = AppScreen.HOME
                                screen = AppScreen.SETTINGS
                            },
                        )
                        AppScreen.SETTINGS -> SettingsScreen(
                            strings = strings,
                            languageSetting = languageSetting,
                            systemLanguageName = LanguagePrefs.currentSystemLanguageName(this, strings),
                            onLanguageSelected = { selected ->
                                languageSetting = selected
                                LanguagePrefs.save(this, selected)
                            },
                            onBack = {
                                screen = settingsReturnScreen
                            },
                        )
                    }
                }
            }
        }
    }

    private fun loadServers(): List<JSONObject> {
        val json = hostsyncLoadServersJson()
        val arr = JSONArray(json)
        return (0 until arr.length()).map { arr.getJSONObject(it) }
    }
}

@Composable
fun LoginScreen(
    strings: AppStrings,
    onLogin: () -> Unit,
    onOpenSettings: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text("HostSync", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text(strings.subtitle, style = MaterialTheme.typography.bodyMedium)
        Spacer(modifier = Modifier.height(32.dp))
        Button(onClick = onLogin) {
            Text(strings.signInWithGitHub)
        }
        Spacer(modifier = Modifier.height(12.dp))
        TextButton(onClick = onOpenSettings) {
            Text(strings.settings)
        }
    }
}

@Composable
fun HomeScreen(
    strings: AppStrings,
    servers: List<JSONObject>,
    onConnect: (JSONObject) -> Unit,
    onOpenSettings: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                "HostSync",
                style = MaterialTheme.typography.headlineSmall,
                modifier = Modifier.weight(1f)
            )
            TextButton(onClick = onOpenSettings) {
                Text(strings.settings)
            }
        }
        Spacer(modifier = Modifier.height(12.dp))

        if (servers.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text(strings.noServersYet)
            }
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                itemsIndexed(servers) { _, server ->
                    ServerCard(strings = strings, server = server, onConnect = { onConnect(server) })
                }
            }
        }
    }
}

@Composable
fun ServerCard(strings: AppStrings, server: JSONObject, onConnect: () -> Unit) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Row(
            modifier = Modifier.padding(16.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(modifier = Modifier.weight(1f)) {
                Text(server.optString("name", ""), style = MaterialTheme.typography.titleMedium)
                Text(
                    "${server.optString("username", "root")}@${server.optString("host", "")}:${server.optInt("port", 22)}",
                    style = MaterialTheme.typography.bodySmall
                )
            }
            Button(onClick = onConnect) {
                Text(strings.connect)
            }
        }
    }
}

@Composable
fun SettingsScreen(
    strings: AppStrings,
    languageSetting: AppLanguageSetting,
    systemLanguageName: String,
    onLanguageSelected: (AppLanguageSetting) -> Unit,
    onBack: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                strings.settings,
                style = MaterialTheme.typography.headlineSmall,
                modifier = Modifier.weight(1f)
            )
            TextButton(onClick = onBack) {
                Text(strings.back)
            }
        }
        Spacer(modifier = Modifier.height(16.dp))
        Text(strings.languageTitle, style = MaterialTheme.typography.titleMedium)
        Spacer(modifier = Modifier.height(8.dp))
        Text(
            strings.systemLanguageDetected(systemLanguageName),
            style = MaterialTheme.typography.bodyMedium
        )
        Spacer(modifier = Modifier.height(16.dp))

        LanguageOptionButton(
            label = strings.followSystem,
            selected = languageSetting == AppLanguageSetting.SYSTEM,
            onClick = { onLanguageSelected(AppLanguageSetting.SYSTEM) }
        )
        Spacer(modifier = Modifier.height(8.dp))
        LanguageOptionButton(
            label = strings.languageEnglish,
            selected = languageSetting == AppLanguageSetting.ENGLISH,
            onClick = { onLanguageSelected(AppLanguageSetting.ENGLISH) }
        )
        Spacer(modifier = Modifier.height(8.dp))
        LanguageOptionButton(
            label = strings.languageChinese,
            selected = languageSetting == AppLanguageSetting.CHINESE,
            onClick = { onLanguageSelected(AppLanguageSetting.CHINESE) }
        )
    }
}

@Composable
private fun LanguageOptionButton(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    val colors = if (selected) {
        ButtonDefaults.buttonColors()
    } else {
        ButtonDefaults.outlinedButtonColors()
    }

    OutlinedButton(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        colors = colors,
    ) {
        Text(label)
    }
}
