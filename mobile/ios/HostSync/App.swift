import SwiftUI

enum AppLanguageChoice: String, CaseIterable, Identifiable {
    case system
    case english
    case chinese

    var id: String { rawValue }
}

enum AppLanguage {
    case english
    case chinese
}

struct AppStrings {
    let language: AppLanguage
    let subtitle: String
    let signInWithGitHub: String
    let settings: String
    let languageTitle: String
    let followSystem: String
    let languageEnglish: String
    let languageChinese: String
    let done: String
    let logout: String
    let noServersYet: String
    let commandPlaceholder: String
    let send: String

    func systemLanguageDetected(_ name: String) -> String {
        switch language {
        case .english:
            return "Current system language: \(name)"
        case .chinese:
            return "当前系统语言：\(name)"
        }
    }

    func connectingTo(_ host: String, port: Int) -> String {
        switch language {
        case .english:
            return "Connecting to \(host):\(port)...\n"
        case .chinese:
            return "正在连接到 \(host):\(port)...\n"
        }
    }
}

private func systemAppLanguage() -> AppLanguage {
    let preferred = Locale.preferredLanguages.first?.lowercased() ?? "en"
    return preferred.hasPrefix("zh") ? .chinese : .english
}

private func resolvedLanguage(for choice: AppLanguageChoice) -> AppLanguage {
    switch choice {
    case .system:
        return systemAppLanguage()
    case .english:
        return .english
    case .chinese:
        return .chinese
    }
}

private func appStrings(for language: AppLanguage) -> AppStrings {
    switch language {
    case .english:
        return AppStrings(
            language: .english,
            subtitle: "Manage your Linux servers & SSH keys",
            signInWithGitHub: "Sign in with GitHub",
            settings: "Settings",
            languageTitle: "Language",
            followSystem: "Follow System",
            languageEnglish: "English",
            languageChinese: "Chinese",
            done: "Done",
            logout: "Logout",
            noServersYet: "No servers yet",
            commandPlaceholder: "Command...",
            send: "Send"
        )
    case .chinese:
        return AppStrings(
            language: .chinese,
            subtitle: "管理你的 Linux 服务器与 SSH 密钥",
            signInWithGitHub: "使用 GitHub 登录",
            settings: "设置",
            languageTitle: "语言",
            followSystem: "跟随系统",
            languageEnglish: "英文",
            languageChinese: "中文",
            done: "完成",
            logout: "退出登录",
            noServersYet: "还没有服务器",
            commandPlaceholder: "输入命令...",
            send: "发送"
        )
    }
}

private func systemLanguageName(for strings: AppStrings) -> String {
    switch systemAppLanguage() {
    case .english:
        return strings.languageEnglish
    case .chinese:
        return strings.languageChinese
    }
}

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
    @AppStorage("app_language") private var appLanguageRaw = AppLanguageChoice.system.rawValue

    private var languageBinding: Binding<AppLanguageChoice> {
        Binding(
            get: { AppLanguageChoice(rawValue: appLanguageRaw) ?? .system },
            set: { appLanguageRaw = $0.rawValue }
        )
    }

    private var languageChoice: AppLanguageChoice {
        AppLanguageChoice(rawValue: appLanguageRaw) ?? .system
    }

    private var strings: AppStrings {
        appStrings(for: resolvedLanguage(for: languageChoice))
    }

    var body: some View {
        NavigationStack {
            if isLoggedIn {
                HomeView(servers: $servers, strings: strings, languageChoice: languageBinding, onLogout: {
                    isLoggedIn = false
                })
                .onAppear { servers = HostSyncBridge.loadServers() }
            } else {
                LoginView(strings: strings, languageChoice: languageBinding, onLogin: {
                    isLoggedIn = true
                    servers = HostSyncBridge.loadServers()
                })
            }
        }
    }
}

struct LoginView: View {
    let strings: AppStrings
    @Binding var languageChoice: AppLanguageChoice
    let onLogin: () -> Void
    @State private var showSettings = false

    var body: some View {
        VStack(spacing: 16) {
            Spacer()
            Image(systemName: "server.rack")
                .font(.system(size: 64))
                .foregroundColor(.secondary)
            Text("HostSync")
                .font(.largeTitle.bold())
            Text(strings.subtitle)
                .foregroundColor(.secondary)
            Spacer().frame(height: 32)
            Button(strings.signInWithGitHub) {
                // Open OAuth URL in Safari, callback handled by core
                onLogin()
            }
            .buttonStyle(.borderedProminent)
            Button(strings.settings) {
                showSettings = true
            }
            Spacer()
        }
        .sheet(isPresented: $showSettings) {
            NavigationStack {
                SettingsView(languageChoice: $languageChoice)
            }
        }
    }
}

struct HomeView: View {
    @Binding var servers: [ServerItem]
    let strings: AppStrings
    @Binding var languageChoice: AppLanguageChoice
    let onLogout: () -> Void
    @State private var showSettings = false

    var body: some View {
        Group {
            if servers.isEmpty {
                VStack(spacing: 12) {
                    Spacer()
                    Text(strings.noServersYet)
                        .foregroundColor(.secondary)
                    Spacer()
                }
            } else {
                List {
                    ForEach(servers) { server in
                        NavigationLink(destination: TerminalView(server: server, strings: strings)) {
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
            }
        }
        .navigationTitle("HostSync")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(strings.settings) {
                    showSettings = true
                }
            }
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(strings.logout, action: onLogout)
            }
        }
        .sheet(isPresented: $showSettings) {
            NavigationStack {
                SettingsView(languageChoice: $languageChoice)
            }
        }
    }
}

struct TerminalView: View {
    let server: ServerItem
    let strings: AppStrings
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
                TextField(strings.commandPlaceholder, text: $input)
                    .textFieldStyle(.roundedBorder)
                    .font(.system(.body, design: .monospaced))
                    .onSubmit { sendCommand() }
                Button(strings.send) { sendCommand() }
            }
            .padding(8)
        }
        .navigationTitle(server.name)
        .onAppear {
            output = strings.connectingTo(server.host, port: server.port)
            // SSH connection would be handled via NMSSH or libssh2 binding
        }
    }

    private func sendCommand() {
        guard !input.isEmpty else { return }
        output += "$ \(input)\n"
        input = ""
    }
}

struct SettingsView: View {
    @Binding var languageChoice: AppLanguageChoice
    @Environment(\.dismiss) private var dismiss

    private var strings: AppStrings {
        appStrings(for: resolvedLanguage(for: languageChoice))
    }

    var body: some View {
        Form {
            Section(strings.languageTitle) {
                Picker(strings.languageTitle, selection: $languageChoice) {
                    Text(strings.followSystem).tag(AppLanguageChoice.system)
                    Text(strings.languageEnglish).tag(AppLanguageChoice.english)
                    Text(strings.languageChinese).tag(AppLanguageChoice.chinese)
                }
                Text(strings.systemLanguageDetected(systemLanguageName(for: strings)))
                    .font(.footnote)
                    .foregroundColor(.secondary)
            }
        }
        .navigationTitle(strings.settings)
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button(strings.done) {
                    dismiss()
                }
            }
        }
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
