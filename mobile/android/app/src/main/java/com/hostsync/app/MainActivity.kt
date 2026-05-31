package com.hostsync.app

import android.os.Bundle
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.*
import okhttp3.*
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONArray
import org.json.JSONObject
import java.net.InetSocketAddress
import java.net.Proxy
import java.util.concurrent.TimeUnit

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
    private external fun hostsyncSaveGithubToken(token: String): Int
    private external fun hostsyncFetchUsername(): Int
    private external fun hostsyncSyncDownload(): Int

    // Public wrappers for composable access
    fun saveGithubToken(token: String): Int = hostsyncSaveGithubToken(token)
    fun fetchUsername(): Int = hostsyncFetchUsername()
    fun syncDownload(): Int = hostsyncSyncDownload()

    // Shared OkHttpClient with optional SOCKS5 proxy
    fun httpClient(): OkHttpClient {
        val prefs = getSharedPreferences("hostsync_settings", MODE_PRIVATE)
        val proxyHost = prefs.getString("proxy_host", null)
        val proxyPort = prefs.getInt("proxy_port", 0)
        val builder = OkHttpClient.Builder()
            .connectTimeout(30, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .protocols(listOf(Protocol.HTTP_1_1))  //禁用 HTTP/2 避免代理兼容问题
        if (!proxyHost.isNullOrBlank() && proxyPort > 0) {
            val proxy = Proxy(Proxy.Type.SOCKS, InetSocketAddress(proxyHost, proxyPort))
            builder.proxy(proxy)
        }
        return builder.build()
    }

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
                            activity = this@MainActivity,
                            onLoginSuccess = {
                                servers = loadServers()
                                screen = AppScreen.HOME
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
                            activity = this@MainActivity,
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
    activity: MainActivity,
    onLoginSuccess: () -> Unit,
    onOpenSettings: () -> Unit,
) {
    var userCode by remember { mutableStateOf<String?>(null) }
    var verificationUri by remember { mutableStateOf<String?>(null) }
    var deviceCode by remember { mutableStateOf<String?>(null) }
    var polling by remember { mutableStateOf(false) }
    var errorMsg by remember { mutableStateOf<String?>(null) }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text("HostSync", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text(strings.subtitle, style = MaterialTheme.typography.bodyMedium)
        Spacer(modifier = Modifier.height(32.dp))

        // Show error outside the card too
        if (errorMsg != null && userCode == null) {
            Spacer(modifier = Modifier.height(8.dp))
            Text(errorMsg!!, color = MaterialTheme.colorScheme.error, modifier = Modifier.padding(horizontal = 32.dp))
        }

        if (userCode != null && deviceCode != null) {
            // Show device code and polling status
            Card(
                modifier = Modifier.padding(16.dp),
                colors = CardDefaults.cardColors(
                    containerColor = MaterialTheme.colorScheme.surfaceVariant
                )
            ) {
                Column(
                    modifier = Modifier.padding(24.dp),
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Text(
                        strings.openAndEnterCode,
                        style = MaterialTheme.typography.titleMedium
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    // Clickable device code — tap to copy
                    Text(
                        userCode!!,
                        style = MaterialTheme.typography.headlineMedium,
                        color = MaterialTheme.colorScheme.primary,
                        modifier = Modifier.clickable {
                            val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                            val clip = ClipData.newPlainText("device_code", userCode)
                            clipboard.setPrimaryClip(clip)
                            Toast.makeText(context, strings.codeCopied, Toast.LENGTH_SHORT).show()
                        }
                    )
                    Text(
                        strings.tapToCopy,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        verificationUri ?: "https://github.com/login/device",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    Spacer(modifier = Modifier.height(16.dp))
                    if (polling) {
                        CircularProgressIndicator(modifier = Modifier.size(24.dp))
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(strings.waitingForAuthorization, style = MaterialTheme.typography.bodySmall)
                    }
                    if (errorMsg != null) {
                        Spacer(modifier = Modifier.height(8.dp))
                        Text(errorMsg!!, color = MaterialTheme.colorScheme.error)
                    }
                }
            }
            Spacer(modifier = Modifier.height(16.dp))
        }

        Button(
            onClick = {
                if (deviceCode == null) {
                    // Step 1: Request device code
                    scope.launch(Dispatchers.IO) {
                        try {
                            // Request device code via OkHttp directly
                            withContext(Dispatchers.Main) {
                                Toast.makeText(context, "Requesting device code...", Toast.LENGTH_SHORT).show()
                            }
                            val client = withContext(Dispatchers.Main) { activity.httpClient() }

                            val dcFormBody = FormBody.Builder()
                                .add("client_id", "Ov23liGz0a5kU4v1LwKI")
                                .add("scope", "gist read:user")
                                .build()

                            val dcRequest = Request.Builder()
                                .url("https://github.com/login/device/code")
                                .header("Accept", "application/json")
                                .header("User-Agent", "HostSync/1.0")
                                .post(dcFormBody)
                                .build()

                            val dcResponse = client.newCall(dcRequest).execute()
                            val bodyStr = dcResponse.body?.string() ?: "{}"
                            dcResponse.close()
                            val json = JSONObject(bodyStr)

                            if (json.has("error")) {
                                withContext(Dispatchers.Main) {
                                    errorMsg = json.optString("error_description", json.getString("error"))
                                }
                                return@launch
                            }
                            val uc = json.getString("user_code")
                            val dc = json.getString("device_code")
                            val uri = json.getString("verification_uri")
                            val intervalSec0 = json.optLong("interval", 5)

                            withContext(Dispatchers.Main) {
                                userCode = uc
                                verificationUri = uri
                                deviceCode = dc
                                polling = true
                            }

                            // Open browser for user to enter code
                            withContext(Dispatchers.Main) {
                                activity.startActivity(
                                    Intent(Intent.ACTION_VIEW, Uri.parse(uri))
                                )
                            }

                            // Step 2: Poll for token
                            var intervalSec = intervalSec0
                            val deadline = System.currentTimeMillis() + 900_000
                            while (System.currentTimeMillis() < deadline) {
                                delay(intervalSec * 1000)

                                val pollFormBody = FormBody.Builder()
                                    .add("client_id", "Ov23liGz0a5kU4v1LwKI")
                                    .add("device_code", dc)
                                    .add("grant_type", "urn:ietf:params:oauth:grant-type:device_code")
                                    .build()

                                val pollRequest = Request.Builder()
                                    .url("https://github.com/login/oauth/access_token")
                                    .header("Accept", "application/json")
                                    .header("User-Agent", "HostSync/1.0")
                                    .post(pollFormBody)
                                    .build()

                                val pollResponse = client.newCall(pollRequest).execute()
                                val pollBodyStr = pollResponse.body?.string() ?: "{}"
                                pollResponse.close()
                                val body = JSONObject(pollBodyStr)

                                if (body.has("access_token")) {
                                    val token = body.getString("access_token")
                                    // Save token via Rust FFI
                                    activity.saveGithubToken(token)
                                    // Fetch username from GitHub API
                                    activity.fetchUsername()
                                    // Sync: download servers from gist
                                    activity.syncDownload()
                                    withContext(Dispatchers.Main) {
                                        polling = false
                                        onLoginSuccess()
                                    }
                                    return@launch
                                }

                                when (body.optString("error")) {
                                    "slow_down" -> intervalSec += 5
                                    "expired_token", "access_denied" -> {
                                        withContext(Dispatchers.Main) {
                                            errorMsg = body.optString("error_description", "Login failed")
                                            polling = false
                                        }
                                        return@launch
                                    }
                                    // authorization_pending → keep polling
                                }
                            }
                            withContext(Dispatchers.Main) {
                                errorMsg = "Device code expired"
                                polling = false
                            }
                        } catch (e: Exception) {
                            withContext(Dispatchers.Main) {
                                errorMsg = e.message ?: "Unknown error"
                                polling = false
                            }
                        }
                    }
                } else {
                    // Re-open browser
                    activity.startActivity(
                        Intent(Intent.ACTION_VIEW, Uri.parse(verificationUri ?: "https://github.com/login/device"))
                    )
                }
            },
            enabled = !polling
        ) {
            Text(
                if (userCode == null) strings.signInWithGitHub
                else strings.openGitHub
            )
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
    activity: MainActivity,
    languageSetting: AppLanguageSetting,
    systemLanguageName: String,
    onLanguageSelected: (AppLanguageSetting) -> Unit,
    onBack: () -> Unit,
) {
    val prefs = remember { activity.getSharedPreferences("hostsync_settings", Context.MODE_PRIVATE) }
    var proxyHost by remember { mutableStateOf(prefs.getString("proxy_host", "") ?: "") }
    var proxyPort by remember { mutableStateOf(if (prefs.getInt("proxy_port", 0) > 0) prefs.getInt("proxy_port", 0).toString() else "") }

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

        Spacer(modifier = Modifier.height(24.dp))
        Text(strings.proxySettings, style = MaterialTheme.typography.titleMedium)
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(
            value = proxyHost,
            onValueChange = {
                proxyHost = it
                prefs.edit().putString("proxy_host", it).apply()
            },
            label = { Text(strings.proxyHost) },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true
        )
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(
            value = proxyPort,
            onValueChange = {
                proxyPort = it
                val port = it.toIntOrNull() ?: 0
                prefs.edit().putInt("proxy_port", port).apply()
            },
            label = { Text(strings.proxyPort) },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true
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
