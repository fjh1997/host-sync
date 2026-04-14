import SwiftUI

@main
struct HostSyncApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var servers: [ServerItem] = []
    @State private var isLoggedIn = HostSyncBridge.isLoggedIn()

    var body: some View {
        NavigationStack {
            if isLoggedIn {
                HomeView(servers: $servers, onLogout: {
                    isLoggedIn = false
                })
                .onAppear { servers = HostSyncBridge.loadServers() }
            } else {
                LoginView(onLogin: {
                    isLoggedIn = true
                    servers = HostSyncBridge.loadServers()
                })
            }
        }
    }
}

struct LoginView: View {
    let onLogin: () -> Void

    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "server.rack")
                .font(.system(size: 64))
                .foregroundColor(.secondary)
            Text("HostSync")
                .font(.largeTitle.bold())
            Text("Manage your Linux servers")
                .foregroundColor(.secondary)
            Spacer().frame(height: 32)
            Button("Sign in with GitHub") {
                // Open OAuth URL in Safari, callback handled by core
                onLogin()
            }
            .buttonStyle(.borderedProminent)
            Spacer()
        }
    }
}

struct HomeView: View {
    @Binding var servers: [ServerItem]
    let onLogout: () -> Void

    var body: some View {
        List {
            ForEach(servers) { server in
                NavigationLink(destination: TerminalView(server: server)) {
                    VStack(alignment: .leading, spacing: 4) {
                        Text(server.name).font(.headline)
                        Text("\(server.username)@\(server.host):\(server.port)")
                            .font(.caption).monospaced()
                            .foregroundColor(.secondary)
                    }
                    .padding(.vertical, 4)
                }
            }
        }
        .navigationTitle("HostSync")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button("Logout", action: onLogout)
            }
        }
    }
}

struct TerminalView: View {
    let server: ServerItem
    @State private var output: String = ""
    @State private var input: String = ""

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                Text(output)
                    .font(.system(.caption, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(8)
            }
            Divider()
            HStack {
                TextField("Command...", text: $input)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .onSubmit { sendCommand() }
                Button("Send") { sendCommand() }
            }
            .padding(8)
        }
        .navigationTitle(server.name)
        .onAppear {
            output = "Connecting to \(server.host):\(server.port)...\n"
            // SSH connection would be handled via NMSSH or libssh2 binding
        }
    }

    private func sendCommand() {
        guard !input.isEmpty else { return }
        output += "$ \(input)\n"
        input = ""
    }
}

// MARK: - Bridge to Rust FFI

struct ServerItem: Identifiable, Codable {
    let id: String
    let name: String
    let host: String
    let port: Int
    let username: String
    let auth_type: String
    let identity_file: String?
    let password: String?
    let private_key: String?
}

class HostSyncBridge {
    static func loadServers() -> [ServerItem] {
        guard let ptr = hostsync_load_servers_json() else { return [] }
        let json = String(cString: ptr)
        hostsync_free_string(ptr)
        guard let data = json.data(using: .utf8),
              let items = try? JSONDecoder().decode([ServerItem].self, from: data)
        else { return [] }
        return items
    }

    static func isLoggedIn() -> Bool {
        return hostsync_is_logged_in() == 1
    }

    static func getUsername() -> String {
        guard let ptr = hostsync_get_github_username() else { return "User" }
        let name = String(cString: ptr)
        hostsync_free_string(ptr)
        return name
    }
}
