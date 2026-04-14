package com.hostsync.app

import android.os.Bundle
import android.view.KeyEvent
import android.widget.EditText
import android.widget.ScrollView
import android.widget.TextView
import androidx.activity.ComponentActivity
import com.jcraft.jsch.ChannelShell
import com.jcraft.jsch.JSch
import com.jcraft.jsch.Session
import java.io.InputStream
import java.io.OutputStream
import kotlin.concurrent.thread

/**
 * Built-in SSH terminal for Android.
 * Uses JSch for SSH connectivity with a simple text-based terminal UI.
 */
class TerminalActivity : ComponentActivity() {
    private var session: Session? = null
    private var channel: ChannelShell? = null
    private var outputStream: OutputStream? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val host = intent.getStringExtra("host") ?: return finish()
        val port = intent.getIntExtra("port", 22)
        val username = intent.getStringExtra("username") ?: "root"
        val authType = intent.getStringExtra("authType") ?: "password"
        val password = intent.getStringExtra("password") ?: ""
        val privateKey = intent.getStringExtra("privateKey") ?: ""

        val scrollView = ScrollView(this)
        val terminalOutput = TextView(this).apply {
            textSize = 13f
            setTextIsSelectable(true)
            setPadding(16, 16, 16, 16)
            typeface = android.graphics.Typeface.MONOSPACE
        }
        scrollView.addView(terminalOutput)

        val inputField = EditText(this).apply {
            hint = "Type command..."
            isSingleLine = true
            setOnKeyListener { _, keyCode, event ->
                if (keyCode == KeyEvent.KEYCODE_ENTER && event.action == KeyEvent.ACTION_DOWN) {
                    val cmd = text.toString() + "\n"
                    outputStream?.write(cmd.toByteArray())
                    outputStream?.flush()
                    text.clear()
                    true
                } else false
            }
        }

        val layout = android.widget.LinearLayout(this).apply {
            orientation = android.widget.LinearLayout.VERTICAL
            addView(scrollView, android.widget.LinearLayout.LayoutParams(
                android.widget.LinearLayout.LayoutParams.MATCH_PARENT, 0, 1f
            ))
            addView(inputField)
        }
        setContentView(layout)

        terminalOutput.append("Connecting to $host:$port...\n")

        thread {
            try {
                val jsch = JSch()
                if (authType == "key" && privateKey.isNotEmpty()) {
                    jsch.addIdentity("inline_key", privateKey.toByteArray(), null, null)
                }

                session = jsch.getSession(username, host, port).apply {
                    if (authType == "password") setPassword(password)
                    setConfig("StrictHostKeyChecking", "no")
                    connect(10000)
                }

                channel = (session!!.openChannel("shell") as ChannelShell).apply {
                    setPtyType("xterm")
                }
                outputStream = channel!!.outputStream
                val inputStream: InputStream = channel!!.inputStream
                channel!!.connect()

                val buffer = ByteArray(4096)
                while (!channel!!.isClosed) {
                    val len = inputStream.read(buffer)
                    if (len > 0) {
                        val text = String(buffer, 0, len)
                        runOnUiThread {
                            terminalOutput.append(text)
                            scrollView.fullScroll(ScrollView.FOCUS_DOWN)
                        }
                    }
                }
                runOnUiThread { terminalOutput.append("\n[Connection closed]\n") }
            } catch (e: Exception) {
                runOnUiThread { terminalOutput.append("\n[Error: ${e.message}]\n") }
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        channel?.disconnect()
        session?.disconnect()
    }
}
