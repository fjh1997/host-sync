mod ui;

use hostsync_core::storage;
use iced::{Element, Task, Theme};

fn main() -> iced::Result {
    iced::application("HostSync", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window_size((900.0, 640.0))
        .run_with(App::new)
}

#[derive(Debug, Clone)]
enum Screen {
    Login,
    Home,
    AddEdit(Option<usize>), // None = add, Some(idx) = edit
    ImportPaste,
}

struct App {
    screen: Screen,
    servers: Vec<hostsync_core::model::Server>,
    search: String,
    // Form fields
    form_name: String,
    form_host: String,
    form_port: String,
    form_user: String,
    form_auth_type: hostsync_core::model::AuthType,
    form_password: String,
    form_identity_file: String,
    form_private_key: String,
    form_passphrase: String,
    form_notes: String,
    // Import
    paste_text: String,
    // Status
    status_msg: String,
    syncing: bool,
}

#[derive(Debug, Clone)]
enum Msg {
    // Navigation
    GoHome,
    GoAdd,
    GoEdit(usize),
    GoImportPaste,
    // Auth
    Login,
    LoginDone(Result<String, String>),
    Logout,
    // Server list
    SearchChanged(String),
    Connect(usize),
    Delete(usize),
    // Form
    FormName(String),
    FormHost(String),
    FormPort(String),
    FormUser(String),
    FormAuthPassword,
    FormAuthKey,
    FormPassword(String),
    FormIdentityFile(String),
    FormPrivateKey(String),
    FormPassphrase(String),
    FormNotes(String),
    FormSave,
    // Sync
    SyncUpload,
    SyncUploadDone(Result<(), String>),
    SyncDownload,
    SyncDownloadDone(Result<(), String>),
    // Import/Export
    ImportSystem,
    ImportSystemDone(Vec<hostsync_core::model::Server>),
    ImportPasteConfirm,
    ExportClipboard,
    ExportSystem,
    PasteTextChanged(String),
    // Misc
    Noop,
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let servers = storage::load_servers();
        let logged_in = storage::is_logged_in();
        (
            Self {
                screen: if logged_in { Screen::Home } else { Screen::Login },
                servers,
                search: String::new(),
                form_name: String::new(),
                form_host: String::new(),
                form_port: "22".into(),
                form_user: "root".into(),
                form_auth_type: hostsync_core::model::AuthType::Password,
                form_password: String::new(),
                form_identity_file: String::new(),
                form_private_key: String::new(),
                form_passphrase: String::new(),
                form_notes: String::new(),
                paste_text: String::new(),
                status_msg: String::new(),
                syncing: false,
            },
            Task::none(),
        )
    }

    fn clear_form(&mut self) {
        self.form_name.clear();
        self.form_host.clear();
        self.form_port = "22".into();
        self.form_user = "root".into();
        self.form_auth_type = hostsync_core::model::AuthType::Password;
        self.form_password.clear();
        self.form_identity_file.clear();
        self.form_private_key.clear();
        self.form_passphrase.clear();
        self.form_notes.clear();
    }

    fn load_form_from(&mut self, idx: usize) {
        let s = &self.servers[idx];
        self.form_name = s.name.clone();
        self.form_host = s.host.clone();
        self.form_port = s.port.to_string();
        self.form_user = s.username.clone();
        self.form_auth_type = s.auth_type.clone();
        self.form_password = s.password.clone().unwrap_or_default();
        self.form_identity_file = s.identity_file.clone().unwrap_or_default();
        self.form_private_key = s.private_key.clone().unwrap_or_default();
        self.form_passphrase = s.passphrase.clone().unwrap_or_default();
        self.form_notes = s.notes.clone().unwrap_or_default();
    }

    fn filtered_indices(&self) -> Vec<usize> {
        let q = self.search.to_lowercase();
        self.servers
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                q.is_empty()
                    || s.name.to_lowercase().contains(&q)
                    || s.host.to_lowercase().contains(&q)
                    || s.username.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect()
    }

    fn save_form(&mut self, edit_idx: Option<usize>) {
        let port = self.form_port.parse().unwrap_or(22);
        let now = chrono::Utc::now();
        let server = hostsync_core::model::Server {
            id: edit_idx
                .map(|i| self.servers[i].id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            name: self.form_name.trim().to_string(),
            host: self.form_host.trim().to_string(),
            port,
            username: self.form_user.trim().to_string(),
            auth_type: self.form_auth_type.clone(),
            identity_file: if self.form_auth_type == hostsync_core::model::AuthType::Key
                && !self.form_identity_file.trim().is_empty()
            {
                Some(self.form_identity_file.trim().to_string())
            } else {
                None
            },
            password: if self.form_auth_type == hostsync_core::model::AuthType::Password
                && !self.form_password.is_empty()
            {
                Some(self.form_password.clone())
            } else {
                None
            },
            private_key: if self.form_auth_type == hostsync_core::model::AuthType::Key
                && !self.form_private_key.trim().is_empty()
            {
                Some(self.form_private_key.clone())
            } else {
                None
            },
            passphrase: if self.form_auth_type == hostsync_core::model::AuthType::Key
                && !self.form_passphrase.is_empty()
            {
                Some(self.form_passphrase.clone())
            } else {
                None
            },
            notes: if self.form_notes.trim().is_empty() {
                None
            } else {
                Some(self.form_notes.trim().to_string())
            },
            created_at: edit_idx
                .map(|i| self.servers[i].created_at)
                .unwrap_or(now),
            updated_at: now,
        };

        if let Some(idx) = edit_idx {
            self.servers[idx] = server;
        } else {
            self.servers.push(server);
        }
        let _ = storage::save_servers(&self.servers);
    }

    fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::GoHome => {
                self.screen = Screen::Home;
                self.status_msg.clear();
            }
            Msg::GoAdd => {
                self.clear_form();
                self.screen = Screen::AddEdit(None);
            }
            Msg::GoEdit(idx) => {
                self.load_form_from(idx);
                self.screen = Screen::AddEdit(Some(idx));
            }
            Msg::GoImportPaste => {
                self.paste_text.clear();
                self.screen = Screen::ImportPaste;
            }
            Msg::Login => {
                let url = hostsync_core::auth::auth_url();
                let _ = open::that(&url);
                return Task::perform(
                    async { hostsync_core::auth::login().await },
                    Msg::LoginDone,
                );
            }
            Msg::LoginDone(result) => match result {
                Ok(_) => {
                    self.servers = storage::load_servers();
                    self.screen = Screen::Home;
                }
                Err(e) => self.status_msg = format!("Login failed: {}", e),
            },
            Msg::Logout => {
                let _ = hostsync_core::auth::logout();
                self.screen = Screen::Login;
            }
            Msg::SearchChanged(s) => self.search = s,
            Msg::Connect(idx) => {
                let server = &self.servers[idx];
                if let Err(e) = hostsync_core::terminal::launch_native_terminal(server) {
                    self.status_msg = format!("Failed: {}", e);
                }
            }
            Msg::Delete(idx) => {
                self.servers.remove(idx);
                let _ = storage::save_servers(&self.servers);
            }
            Msg::FormName(s) => self.form_name = s,
            Msg::FormHost(s) => self.form_host = s,
            Msg::FormPort(s) => self.form_port = s,
            Msg::FormUser(s) => self.form_user = s,
            Msg::FormAuthPassword => {
                self.form_auth_type = hostsync_core::model::AuthType::Password;
            }
            Msg::FormAuthKey => {
                self.form_auth_type = hostsync_core::model::AuthType::Key;
            }
            Msg::FormPassword(s) => self.form_password = s,
            Msg::FormIdentityFile(s) => self.form_identity_file = s,
            Msg::FormPrivateKey(s) => self.form_private_key = s,
            Msg::FormPassphrase(s) => self.form_passphrase = s,
            Msg::FormNotes(s) => self.form_notes = s,
            Msg::FormSave => {
                if let Screen::AddEdit(edit_idx) = self.screen {
                    self.save_form(edit_idx);
                    self.screen = Screen::Home;
                }
            }
            Msg::SyncUpload => {
                self.syncing = true;
                return Task::perform(
                    async { hostsync_core::sync::upload().await },
                    Msg::SyncUploadDone,
                );
            }
            Msg::SyncUploadDone(r) => {
                self.syncing = false;
                self.status_msg = match r {
                    Ok(_) => "Uploaded to cloud".into(),
                    Err(e) => format!("Upload failed: {}", e),
                };
            }
            Msg::SyncDownload => {
                self.syncing = true;
                return Task::perform(
                    async { hostsync_core::sync::download().await },
                    Msg::SyncDownloadDone,
                );
            }
            Msg::SyncDownloadDone(r) => {
                self.syncing = false;
                match r {
                    Ok(_) => {
                        self.servers = storage::load_servers();
                        self.status_msg = "Downloaded from cloud".into();
                    }
                    Err(e) => self.status_msg = format!("Download failed: {}", e),
                }
            }
            Msg::ImportSystem => {
                return Task::perform(
                    async { hostsync_core::ssh_config::parse_system_config() },
                    Msg::ImportSystemDone,
                );
            }
            Msg::ImportSystemDone(imported) => {
                let existing: std::collections::HashSet<String> =
                    self.servers.iter().map(|s| s.name.clone()).collect();
                let mut added = 0;
                for s in imported {
                    if !existing.contains(&s.name) {
                        self.servers.push(s);
                        added += 1;
                    }
                }
                let _ = storage::save_servers(&self.servers);
                self.status_msg = format!("Imported {} host(s)", added);
                self.screen = Screen::Home;
            }
            Msg::ImportPasteConfirm => {
                let imported = hostsync_core::ssh_config::parse(&self.paste_text);
                let existing: std::collections::HashSet<String> =
                    self.servers.iter().map(|s| s.name.clone()).collect();
                let mut added = 0;
                for s in imported {
                    if !existing.contains(&s.name) {
                        self.servers.push(s);
                        added += 1;
                    }
                }
                let _ = storage::save_servers(&self.servers);
                self.status_msg = format!("Imported {} host(s)", added);
                self.screen = Screen::Home;
            }
            Msg::ExportClipboard => {
                let config = hostsync_core::ssh_config::generate(&self.servers);
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(config);
                    self.status_msg = "SSH config copied to clipboard".into();
                }
            }
            Msg::ExportSystem => {
                match hostsync_core::ssh_config::merge_into_system_config(&self.servers) {
                    Ok(_) => self.status_msg = "Merged into ~/.ssh/config".into(),
                    Err(e) => self.status_msg = format!("Export failed: {}", e),
                }
            }
            Msg::PasteTextChanged(s) => self.paste_text = s,
            Msg::Noop => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Msg> {
        match &self.screen {
            Screen::Login => ui::login_view(&self.status_msg),
            Screen::Home => ui::home_view(self),
            Screen::AddEdit(edit_idx) => ui::form_view(self, *edit_idx),
            Screen::ImportPaste => ui::paste_view(&self.paste_text),
        }
    }
}
