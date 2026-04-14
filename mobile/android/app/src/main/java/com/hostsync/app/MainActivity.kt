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
                var screen by remember { mutableStateOf(if (hostsyncIsLoggedIn() == 1) "home" else "login") }

                Surface(modifier = Modifier.fillMaxSize()) {
                    when (screen) {
                        "login" -> LoginScreen(
                            onLogin = {
                                // Device Flow: open GitHub device auth page
                                val url = "https://github.com/login/device"
                                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                            }
                        )
                        "home" -> HomeScreen(
                            servers = servers,
                            onRefresh = { servers = loadServers() },
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
                            }
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
fun LoginScreen(onLogin: () -> Unit) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally
    ) {
        Text("HostSync", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text("Manage your Linux servers", style = MaterialTheme.typography.bodyMedium)
        Spacer(modifier = Modifier.height(32.dp))
        Button(onClick = onLogin) {
            Text("Sign in with GitHub")
        }
    }
}

@Composable
fun HomeScreen(
    servers: List<JSONObject>,
    onRefresh: () -> Unit,
    onConnect: (JSONObject) -> Unit
) {
    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("HostSync", style = MaterialTheme.typography.headlineSmall)
        Spacer(modifier = Modifier.height(12.dp))

        if (servers.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("No servers yet")
            }
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                itemsIndexed(servers) { _, server ->
                    ServerCard(server = server, onConnect = { onConnect(server) })
                }
            }
        }
    }
}

@Composable
fun ServerCard(server: JSONObject, onConnect: () -> Unit) {
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
                Text("Connect")
            }
        }
    }
}
