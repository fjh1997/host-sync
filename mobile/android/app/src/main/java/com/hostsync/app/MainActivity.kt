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
    SYNC_PASSPHRASE,
    HOME,
    SETTINGS,
    ADD_EDIT,  // editIdx == null means add, otherwise edit
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
    private external fun hostsyncSetSyncPassphrase(passphrase: String): Int
    private external fun hostsyncHasSyncPassphrase(): Int
    private external fun hostsyncSetDataDir(path: String)

    // Public wrappers for composable access
    fun saveGithubToken(token: String): Int = hostsyncSaveGithubToken(token)
    fun fetchUsername(): Int = hostsyncFetchUsername()
    fun syncDownload(): Int = hostsyncSyncDownload()
    fun setSyncPassphrase(pp: String): Int = hostsyncSetSyncPassphrase(pp)
    fun hasSyncPassphrase(): Boolean = hostsyncHasSyncPassphrase() == 1

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
        // Tell Rust where to store data
        hostsyncSetDataDir(filesDir.absolutePath)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                var servers by remember { mutableStateOf(loadServers()) }
                var screen by remember {
                    mutableStateOf(if (hostsyncIsLoggedIn() == 1) AppScreen.HOME else AppScreen.LOGIN)
                }
                var settingsReturnScreen by remember { mutableStateOf(screen) }
                var languageSetting by remember { mutableStateOf(LanguagePrefs.load(this)) }
                val strings = LanguagePrefs.strings(this, languageSetting)

                val syncScope = rememberCoroutineScope()

                // Form state for add/edit
                var editIdx by remember { mutableStateOf<Int?>(null) }
                var formName by remember { mutableStateOf("") }
                var formHost by remember { mutableStateOf("") }
                var formPort by remember { mutableStateOf("22") }
                var formUser by remember { mutableStateOf("root") }
                var formAuthType by remember { mutableStateOf("password") }
                var formPassword by remember { mutableStateOf("") }
                var formIdentityFile by remember { mutableStateOf("") }
                var formPrivateKey by remember { mutableStateOf("") }
                var formPassphrase by remember { mutableStateOf("") }
                var formNotes by remember { mutableStateOf("") }

                Surface(modifier = Modifier.fillMaxSize()) {
                    when (screen) {
                        AppScreen.LOGIN -> LoginScreen(
                            strings = strings,
                            activity = this@MainActivity,
                            onLoginSuccess = {
                                if (this@MainActivity.hasSyncPassphrase()) {
                                    syncScope.launch(Dispatchers.IO) {
                                        this@MainActivity.syncDownload()
                                        withContext(Dispatchers.Main) {
                                            servers = loadServers()
                                            screen = AppScreen.HOME
                                        }
                                    }
                                } else {
                                    screen = AppScreen.SYNC_PASSPHRASE
                                }
                            },
                            onOpenSettings = {
                                settingsReturnScreen = AppScreen.LOGIN
                                screen = AppScreen.SETTINGS
                            },
                        )
                        AppScreen.SYNC_PASSPHRASE -> SyncPassphraseScreen(
                            strings = strings,
                            activity = this@MainActivity,
                            scope = syncScope,
                            onSuccess = {
                                syncScope.launch(Dispatchers.IO) {
                                    this@MainActivity.syncDownload()
                                    withContext(Dispatchers.Main) {
                                        servers = loadServers()
                                        screen = AppScreen.HOME
                                    }
                                }
                            },
                        )
                        AppScreen.HOME -> HomeScreen(
                            strings = strings,
                            servers = servers,
                            onConnect = { },
                            onCopy = { server ->
                                val cmd = buildSshCommand(server)
                                val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                                clipboard.setPrimaryClip(ClipData.newPlainText("ssh_command", cmd))
                                Toast.makeText(this@MainActivity, strings.copiedToClipboard, Toast.LENGTH_SHORT).show()
                            },
                            onEdit = { idx ->
                                editIdx = idx
                                val s = servers[idx]
                                formName = s.optString("name", "")
                                formHost = s.optString("host", "")
                                formPort = s.optInt("port", 22).toString()
                                formUser = s.optString("username", "root")
                                formAuthType = s.optString("auth_type", "password")
                                formPassword = s.optString("password", "")
                                formIdentityFile = s.optString("identity_file", "")
                                formPrivateKey = s.optString("private_key", "")
                                formPassphrase = s.optString("passphrase", "")
                                formNotes = s.optString("notes", "")
                                screen = AppScreen.ADD_EDIT
                            },
                            onDelete = { idx ->
                                val s = servers.toMutableList()
                                s.removeAt(idx)
                                servers = s
                                saveServers(servers)
                                syncScope.launch(Dispatchers.IO) {
                                    this@MainActivity.syncDownload()
                                }
                            },
                            onOpenSettings = {
                                settingsReturnScreen = AppScreen.HOME
                                screen = AppScreen.SETTINGS
                            },
                        )
                        AppScreen.ADD_EDIT -> FormScreen(
                            strings = strings,
                            editIdx = editIdx,
                            name = formName, onNameChange = { formName = it },
                            host = formHost, onHostChange = { formHost = it },
                            port = formPort, onPortChange = { formPort = it },
                            user = formUser, onUserChange = { formUser = it },
                            authType = formAuthType, onAuthTypeChange = { formAuthType = it },
                            password = formPassword, onPasswordChange = { formPassword = it },
                            identityFile = formIdentityFile, onIdentityFileChange = { formIdentityFile = it },
                            privateKey = formPrivateKey, onPrivateKeyChange = { formPrivateKey = it },
                            passphrase = formPassphrase, onPassphraseChange = { formPassphrase = it },
                            notes = formNotes, onNotesChange = { formNotes = it },
                            onSave = {
                                val server = JSONObject().apply {
                                    put("id", editIdx?.let { servers[it].optString("id", "") } ?: java.util.UUID.randomUUID().toString())
                                    put("name", formName)
                                    put("host", formHost)
                                    put("port", formPort.toIntOrNull() ?: 22)
                                    put("username", formUser)
                                    put("auth_type", formAuthType)
                                    put("password", formPassword)
                                    put("identity_file", formIdentityFile)
                                    put("private_key", formPrivateKey)
                                    put("passphrase", formPassphrase)
                                    put("notes", formNotes)
                                }
                                val s = servers.toMutableList()
                                if (editIdx != null) {
                                    s[editIdx!!] = server
                                } else {
                                    s.add(server)
                                }
                                servers = s
                                saveServers(servers)
                                screen = AppScreen.HOME
                                syncScope.launch(Dispatchers.IO) {
                                    this@MainActivity.syncDownload()
                                }
                            },
                            onBack = { screen = AppScreen.HOME },
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

    fun saveServers(servers: List<JSONObject>) {
        val arr = JSONArray()
        servers.forEach { arr.put(it) }
        hostsyncSaveServersJson(arr.toString())
    }
}

/// Build SSH command string for a server (same logic as desktop CopyCommand)
fun buildSshCommand(server: JSONObject): String {
    val port = server.optInt("port", 22)
    val user = server.optString("username", "root")
    val host = server.optString("host", "")
    val idFile = server.optString("identity_file", "")
    val password = server.optString("password", "")

    val sshArgs = if (idFile.isNotEmpty()) {
        "-i $idFile -p $port $user@$host"
    } else {
        "-p $port $user@$host"
    }

    return if (password.isNotEmpty()) {
        val escaped = password.replace("'", "'\\''")
        "sshpass -p '$escaped' ssh -tt -o PreferredAuthentications=password $sshArgs"
    } else {
        "ssh $sshArgs"
    }
}

/// Launch Termux to run a command
/// Returns true if Termux was launched, false if not installed
fun launchTermux(context: Context, command: String): Boolean {
    // Copy command to clipboard so user can paste it
    val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
    clipboard.setPrimaryClip(ClipData.newPlainText("ssh_command", command))

    // Try RUN_COMMAND service first
    try {
        val intent = Intent("com.termux.RUN_COMMAND").apply {
            setPackage("com.termux")
            putExtra("com.termux.RUN_COMMAND_PATH", "/data/data/com.termux/files/usr/bin/bash")
            putExtra("com.termux.RUN_COMMAND_ARGUMENTS", arrayOf("-c", command))
            putExtra("com.termux.RUN_COMMAND_WORKDIR", "/data/data/com.termux/files/home")
            putExtra("com.termux.RUN_COMMAND_SESSION_ACTION", "0")
        }
        context.startService(intent)
        Toast.makeText(context, "SSH command sent to Termux", Toast.LENGTH_SHORT).show()
        return true
    } catch (_: Exception) {}

    // Fallback: launch TermuxActivity (user pastes manually)
    try {
        val intent = Intent().apply {
            setClassName("com.termux", "com.termux.app.TermuxActivity")
        }
        context.startActivity(intent)
        Toast.makeText(context, "Command copied. Long-press to paste in Termux.", Toast.LENGTH_LONG).show()
        return true
    } catch (_: Exception) {}

    // Termux not installed at all
    return false
}

/// Check if Termux is installed
fun isTermuxInstalled(context: Context): Boolean {
    return try {
        context.packageManager.getPackageInfo("com.termux", 0)
        true
    } catch (_: Exception) {
        false
    }
}

/// Open Termux settings — open system app info page (covers all ROMs)
fun openTermuxSettings(context: Context) {
    try {
        // Open system app info page for Termux — user can find all permissions here
        // including "关联启动" (cross-app launch) on MIUI/ColorOS/HarmonyOS etc.
        val intent = Intent(android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
            data = Uri.parse("package:com.termux")
        }
        context.startActivity(intent)
    } catch (_: Exception) {
        // Fallback: try launching Termux settings directly
        try {
            val intent = Intent().apply {
                setClassName("com.termux", "com.termux.app.TermuxPreferencesActivity")
            }
            context.startActivity(intent)
        } catch (_: Exception) {}
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
fun SyncPassphraseScreen(
    strings: AppStrings,
    activity: MainActivity,
    scope: CoroutineScope,
    onSuccess: () -> Unit,
) {
    var passphrase by remember { mutableStateOf("") }
    var errorMsg by remember { mutableStateOf<String?>(null) }
    var loading by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier.fillMaxSize().padding(32.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text(strings.syncPassphraseTitle, style = MaterialTheme.typography.headlineSmall)
        Spacer(modifier = Modifier.height(8.dp))
        Text(strings.syncPassphraseDesc, style = MaterialTheme.typography.bodyMedium)
        Spacer(modifier = Modifier.height(24.dp))
        OutlinedTextField(
            value = passphrase,
            onValueChange = { passphrase = it },
            label = { Text(strings.syncPassphrase) },
            modifier = Modifier.fillMaxWidth(),
            singleLine = true,
        )
        Spacer(modifier = Modifier.height(16.dp))
        if (loading) {
            CircularProgressIndicator(modifier = Modifier.size(24.dp))
        } else {
            Button(
                onClick = {
                    if (passphrase.isBlank()) return@Button
                    loading = true
                    errorMsg = null
                    scope.launch(Dispatchers.IO) {
                        activity.setSyncPassphrase(passphrase)
                        val result = activity.syncDownload()
                        withContext(Dispatchers.Main) {
                            loading = false
                            if (result == 0) {
                                onSuccess()
                            } else {
                                errorMsg = strings.syncPassphraseError
                            }
                        }
                    }
                },
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(strings.confirm)
            }
        }
        if (errorMsg != null) {
            Spacer(modifier = Modifier.height(8.dp))
            Text(errorMsg!!, color = MaterialTheme.colorScheme.error)
        }
    }
}

@Composable
fun HomeScreen(
    strings: AppStrings,
    servers: List<JSONObject>,
    onConnect: (JSONObject) -> Unit,
    onCopy: (JSONObject) -> Unit,
    onEdit: (Int) -> Unit,
    onDelete: (Int) -> Unit,
    onOpenSettings: () -> Unit,
) {
    var deleteIdx by remember { mutableStateOf<Int?>(null) }
    var showTermuxDialog by remember { mutableStateOf(false) }
    var pendingCommand by remember { mutableStateOf<String?>(null) }
    val context = LocalContext.current

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
                itemsIndexed(servers) { idx, server ->
                    ServerCard(
                        strings = strings,
                        server = server,
                        onConnect = {
                            pendingCommand = buildSshCommand(server)
                            showTermuxDialog = true
                        },
                        onCopy = { onCopy(server) },
                        onEdit = { onEdit(idx) },
                        onDelete = { deleteIdx = idx },
                    )
                }
            }
        }
    }

    // Delete confirmation dialog
    if (deleteIdx != null) {
        AlertDialog(
            onDismissRequest = { deleteIdx = null },
            title = { Text(strings.delete) },
            text = { Text(strings.deleteConfirm) },
            confirmButton = {
                TextButton(onClick = {
                    onDelete(deleteIdx!!)
                    deleteIdx = null
                }) { Text(strings.confirm) }
            },
            dismissButton = {
                TextButton(onClick = { deleteIdx = null }) { Text(strings.cancel) }
            },
        )
    }

    // Termux guidance dialog
    if (showTermuxDialog) {
        AlertDialog(
            onDismissRequest = { showTermuxDialog = false },
            title = { Text(strings.termuxTitle) },
            text = { Text(strings.termuxDesc) },
            confirmButton = {
                TextButton(onClick = {
                    openTermuxSettings(context)
                }) { Text(strings.openSettings) }
            },
            dismissButton = {
                Row {
                    TextButton(onClick = { showTermuxDialog = false }) { Text(strings.cancel) }
                    TextButton(onClick = {
                        showTermuxDialog = false
                        pendingCommand?.let { launchTermux(context, it) }
                    }) { Text(strings.connectAnyway) }
                }
            },
        )
    }
}

@Composable
fun ServerCard(
    strings: AppStrings,
    server: JSONObject,
    onConnect: () -> Unit,
    onCopy: () -> Unit,
    onEdit: () -> Unit,
    onDelete: () -> Unit,
) {
    val authBadge = when (server.optString("auth_type", "password")) {
        "key" -> "[${strings.sshKey}]"
        else -> "[${strings.passwordBadge}]"
    }
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(modifier = Modifier.weight(1f)) {
                    Text(server.optString("name", ""), style = MaterialTheme.typography.titleMedium)
                    Text(
                        "${server.optString("username", "root")}@${server.optString("host", "")}:${server.optInt("port", 22)}  $authBadge",
                        style = MaterialTheme.typography.bodySmall
                    )
                }
                // Action buttons row
                OutlinedButton(onClick = onConnect, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)) {
                    Text(strings.connect, style = MaterialTheme.typography.labelSmall)
                }
                Spacer(modifier = Modifier.width(4.dp))
                OutlinedButton(onClick = onCopy, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)) {
                    Text(strings.copy, style = MaterialTheme.typography.labelSmall)
                }
                Spacer(modifier = Modifier.width(4.dp))
                OutlinedButton(onClick = onEdit, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)) {
                    Text(strings.edit, style = MaterialTheme.typography.labelSmall)
                }
                Spacer(modifier = Modifier.width(4.dp))
                OutlinedButton(onClick = onDelete, contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp)) {
                    Text(strings.delete, style = MaterialTheme.typography.labelSmall)
                }
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

@Composable
fun FormScreen(
    strings: AppStrings,
    editIdx: Int?,
    name: String, onNameChange: (String) -> Unit,
    host: String, onHostChange: (String) -> Unit,
    port: String, onPortChange: (String) -> Unit,
    user: String, onUserChange: (String) -> Unit,
    authType: String, onAuthTypeChange: (String) -> Unit,
    password: String, onPasswordChange: (String) -> Unit,
    identityFile: String, onIdentityFileChange: (String) -> Unit,
    privateKey: String, onPrivateKeyChange: (String) -> Unit,
    passphrase: String, onPassphraseChange: (String) -> Unit,
    notes: String, onNotesChange: (String) -> Unit,
    onSave: () -> Unit,
    onBack: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                if (editIdx != null) strings.edit else strings.add,
                style = MaterialTheme.typography.headlineSmall,
                modifier = Modifier.weight(1f)
            )
            TextButton(onClick = onBack) { Text(strings.back) }
        }
        Spacer(modifier = Modifier.height(16.dp))

        OutlinedTextField(value = name, onValueChange = onNameChange,
            label = { Text(strings.name) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(value = host, onValueChange = onHostChange,
            label = { Text(strings.host) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(value = port, onValueChange = onPortChange,
            label = { Text(strings.port) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Spacer(modifier = Modifier.height(8.dp))
        OutlinedTextField(value = user, onValueChange = onUserChange,
            label = { Text(strings.username) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        Spacer(modifier = Modifier.height(8.dp))

        // Auth type toggle
        Row {
            FilterChip(selected = authType == "password", onClick = { onAuthTypeChange("password") }, label = { Text(strings.passwordBadge) })
            Spacer(modifier = Modifier.width(8.dp))
            FilterChip(selected = authType == "key", onClick = { onAuthTypeChange("key") }, label = { Text(strings.sshKey) })
        }
        Spacer(modifier = Modifier.height(8.dp))

        if (authType == "password") {
            OutlinedTextField(value = password, onValueChange = onPasswordChange,
                label = { Text(strings.password) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        } else {
            OutlinedTextField(value = identityFile, onValueChange = onIdentityFileChange,
                label = { Text(strings.identityFile) }, modifier = Modifier.fillMaxWidth(), singleLine = true)
        }
        Spacer(modifier = Modifier.height(16.dp))

        OutlinedTextField(value = notes, onValueChange = onNotesChange,
            label = { Text(strings.notes) }, modifier = Modifier.fillMaxWidth(), minLines = 2)
        Spacer(modifier = Modifier.height(16.dp))

        Button(onClick = onSave, modifier = Modifier.fillMaxWidth()) {
            Text(strings.save)
        }
    }
}
