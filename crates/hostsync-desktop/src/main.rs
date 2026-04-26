#![windows_subsystem = "windows"]

mod i18n;
mod ui;

use hostsync_core::storage;
use iced::widget::text_editor;
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
    AddEdit(Option<usize>),
    ImportPaste,
    Settings {
        from_login: bool,
    },
    /// Prompt user to set or enter a sync passphrase.
    /// `is_new` = true means first-time setup (set); false means entering existing passphrase.
    SyncPassphrase {
        is_new: bool,
        next_action: PassphraseAction,
    },
}

#[derive(Debug, Clone)]
enum PassphraseAction {
    Upload,
    Download,
}

struct App {
    screen: Screen,
    servers: Vec<hostsync_core::model::Server>,
    language_setting: i18n::LanguageSetting,
    language: i18n::Language,
    search: String,
    // Form fields
    form_name: String,
    form_host: String,
    form_port: String,
    form_user: String,
    form_auth_type: hostsync_core::model::AuthType,
    form_password: String,
    form_identity_file: String,
    form_private_key: text_editor::Content,
    form_passphrase: String,
    form_notes: text_editor::Content,
    // Import
    paste_text: text_editor::Content,
    // Status
    status_msg: String,
    syncing: bool,
    // Device Flow login
    device_user_code: String,
    logging_in: bool,
    // Proxy
    proxy_input: String,
    // Sync passphrase
    sync_passphrase_input: String,
}

#[derive(Debug, Clone)]
enum Msg {
    // Navigation
    GoHome,
    GoAdd,
    GoEdit(usize),
    GoImportPaste,
    GoSettings,
    // Auth
    Login,
    DeviceCodeReceived(Result<hostsync_core::auth::DeviceCode, String>),
    LoginDone(Result<String, String>),
    Logout,
    // Server list
    SearchChanged(String),
    Connect(usize),
    CopyCommand(usize),
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
    FormBrowsePrivateKey,
    FormBrowsePrivateKeyDone(Option<(String, String)>), // (path, content)
    FormPassphrase(String),
    FormNotes(text_editor::Action),
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
    PasteTextChanged(text_editor::Action),
    LanguageSelected(i18n::LanguageSetting),
    // Settings
    ProxyInput(String),
    ProxySave,
    // Sync passphrase
    SyncPassphraseInput(String),
    SyncPassphraseConfirm,
    // Misc
    Noop,
    FormPrivateKey(text_editor::Action),
}

impl App {
    fn new() -> (Self, Task<Msg>) {
        let logged_in = storage::is_logged_in();
        let language_setting =
            i18n::LanguageSetting::from_storage(storage::load_language_setting().as_deref());
        let language = i18n::resolve_language(language_setting);
        let i18n = i18n::I18n::new(language);
        // Always start with local cache; if logged in, immediately sync from cloud
        let servers = storage::load_servers();
        let task = if logged_in {
            Task::perform(
                async { hostsync_core::sync::download(None).await },
                Msg::SyncDownloadDone,
            )
        } else {
            Task::none()
        };
        (
            Self {
                screen: if logged_in {
                    Screen::Home
                } else {
                    Screen::Login
                },
                servers,
                language_setting,
                language,
                search: String::new(),
                form_name: String::new(),
                form_host: String::new(),
                form_port: "22".into(),
                form_user: "root".into(),
                form_auth_type: hostsync_core::model::AuthType::Password,
                form_password: String::new(),
                form_identity_file: String::new(),
                form_private_key: text_editor::Content::new(),
                form_passphrase: String::new(),
                form_notes: text_editor::Content::new(),
                paste_text: text_editor::Content::new(),
                status_msg: if logged_in {
                    i18n.syncing_from_cloud().into()
                } else {
                    String::new()
                },
                syncing: logged_in,
                device_user_code: String::new(),
                logging_in: false,
                proxy_input: storage::load_proxy().unwrap_or_default(),
                sync_passphrase_input: String::new(),
            },
            task,
        )
    }

    fn i18n(&self) -> i18n::I18n {
        i18n::I18n::new(self.language)
    }

    fn clear_form(&mut self) {
        self.form_name.clear();
        self.form_host.clear();
        self.form_port = "22".into();
        self.form_user = "root".into();
        self.form_auth_type = hostsync_core::model::AuthType::Password;
        self.form_password.clear();
        self.form_identity_file.clear();
        self.form_private_key = text_editor::Content::new();
        self.form_passphrase.clear();
        self.form_notes = text_editor::Content::new();
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
        self.form_private_key =
            text_editor::Content::with_text(&s.private_key.clone().unwrap_or_default());
        self.form_passphrase = s.passphrase.clone().unwrap_or_default();
        self.form_notes = text_editor::Content::with_text(&s.notes.clone().unwrap_or_default());
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
        let server_id = edit_idx
            .map(|i| self.servers[i].id.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let server = hostsync_core::model::Server {
            id: server_id.clone(),
            name: self.form_name.trim().to_string(),
            host: self.form_host.trim().to_string(),
            port,
            username: self.form_user.trim().to_string(),
            auth_type: self.form_auth_type.clone(),
            identity_file: if self.form_auth_type == hostsync_core::model::AuthType::Key
                && !self.form_private_key.text().trim().is_empty()
            {
                // Auto-normalize path to the managed location for inline keys
                Some(format!("~/.ssh/hostsync_keys/{}.key", server_id))
            } else if self.form_auth_type == hostsync_core::model::AuthType::Key
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
                && !self.form_private_key.text().trim().is_empty()
            {
                Some(self.form_private_key.text())
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
            notes: {
                let text = self.form_notes.text();
                if text.trim().is_empty() {
                    None
                } else {
                    Some(text)
                }
            },
            created_at: edit_idx.map(|i| self.servers[i].created_at).unwrap_or(now),
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
                // If coming from proxy settings, return to the correct screen
                if let Screen::Settings { from_login: true } = self.screen {
                    self.screen = Screen::Login;
                } else {
                    self.screen = Screen::Home;
                }
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
                self.paste_text = text_editor::Content::new();
                self.screen = Screen::ImportPaste;
            }
            Msg::Login => {
                self.logging_in = true;
                self.status_msg = self.i18n().requesting_device_code().into();
                return Task::perform(
                    async { hostsync_core::auth::request_device_code().await },
                    Msg::DeviceCodeReceived,
                );
            }
            Msg::DeviceCodeReceived(result) => match result {
                Ok(dc) => {
                    self.device_user_code = dc.user_code.clone();
                    self.status_msg = self
                        .i18n()
                        .open_and_enter_code(&dc.verification_uri, &dc.user_code);
                    // Copy code to clipboard for convenience
                    if let Ok(mut clip) = arboard::Clipboard::new() {
                        let _ = clip.set_text(&dc.user_code);
                    }
                    // Open browser
                    let _ = open::that(&dc.verification_uri);
                    // Start polling
                    return Task::perform(
                        async move { hostsync_core::auth::poll_for_token(&dc).await },
                        Msg::LoginDone,
                    );
                }
                Err(e) => {
                    self.logging_in = false;
                    self.status_msg = self.i18n().failed(&e);
                }
            },
            Msg::LoginDone(result) => {
                self.logging_in = false;
                self.device_user_code.clear();
                match result {
                    Ok(_) => {
                        self.screen = Screen::Home;
                        self.status_msg = self.i18n().syncing_from_cloud().into();
                        self.syncing = true;
                        return Task::perform(
                            async { hostsync_core::sync::download(None).await },
                            Msg::SyncDownloadDone,
                        );
                    }
                    Err(e) => self.status_msg = self.i18n().login_failed(&e),
                }
            }
            Msg::Logout => {
                let _ = hostsync_core::auth::logout();
                self.screen = Screen::Login;
            }
            Msg::SearchChanged(s) => self.search = s,
            Msg::Connect(idx) => {
                let server = &self.servers[idx];
                if let Err(e) = hostsync_core::terminal::launch_native_terminal(server) {
                    self.status_msg = self.i18n().failed(&e);
                }
            }
            Msg::CopyCommand(idx) => {
                let s = &self.servers[idx];
                let mut cmd = format!("ssh -p {} {}@{}", s.port, s.username, s.host);
                if let Some(ref id_file) = s.identity_file {
                    if !id_file.is_empty() {
                        cmd = format!("ssh -i {} -p {} {}@{}", id_file, s.port, s.username, s.host);
                    }
                }
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(cmd);
                    self.status_msg = self.i18n().command_copied().into();
                }
            }
            Msg::Delete(idx) => {
                self.servers.remove(idx);
                let _ = storage::save_servers(&self.servers);
                self.syncing = true;
                self.status_msg = self.i18n().syncing().into();
                return Task::perform(
                    async { hostsync_core::sync::upload(None).await },
                    Msg::SyncUploadDone,
                );
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
            Msg::FormPrivateKey(action) => self.form_private_key.perform(action),
            Msg::FormBrowsePrivateKey => {
                let dialog_title = self.i18n().select_ssh_key_file().to_string();
                return Task::perform(
                    async move {
                        let file = rfd::AsyncFileDialog::new()
                            .set_title(&dialog_title)
                            .pick_file()
                            .await;
                        if let Some(f) = file {
                            let path = f.path().to_string_lossy().to_string();
                            match tokio::fs::read_to_string(f.path()).await {
                                Ok(content) => Some((path, content)),
                                Err(_) => None,
                            }
                        } else {
                            None
                        }
                    },
                    Msg::FormBrowsePrivateKeyDone,
                );
            }
            Msg::FormBrowsePrivateKeyDone(result) => {
                if let Some((path, content)) = result {
                    self.form_identity_file = path;
                    self.form_private_key = text_editor::Content::with_text(&content);
                }
            }
            Msg::FormPassphrase(s) => self.form_passphrase = s,
            Msg::FormNotes(action) => self.form_notes.perform(action),
            Msg::FormSave => {
                if let Screen::AddEdit(edit_idx) = self.screen {
                    self.save_form(edit_idx);
                    self.screen = Screen::Home;
                    self.syncing = true;
                    self.status_msg = self.i18n().syncing().into();
                    return Task::perform(
                        async { hostsync_core::sync::upload(None).await },
                        Msg::SyncUploadDone,
                    );
                }
            }
            Msg::SyncUpload => {
                if !storage::has_sync_passphrase() {
                    self.sync_passphrase_input.clear();
                    self.screen = Screen::SyncPassphrase {
                        is_new: true,
                        next_action: PassphraseAction::Upload,
                    };
                    return Task::none();
                }
                self.syncing = true;
                self.status_msg = self.i18n().uploading().into();
                return Task::perform(
                    async { hostsync_core::sync::upload(None).await },
                    Msg::SyncUploadDone,
                );
            }
            Msg::SyncUploadDone(r) => {
                self.syncing = false;
                match &r {
                    Ok(_) => self.status_msg = self.i18n().uploaded_to_cloud().into(),
                    Err(e) if e == hostsync_core::sync::ERR_NEED_PASSPHRASE => {
                        self.sync_passphrase_input.clear();
                        self.screen = Screen::SyncPassphrase {
                            is_new: true,
                            next_action: PassphraseAction::Upload,
                        };
                    }
                    Err(e) => self.status_msg = self.i18n().upload_failed(e),
                }
            }
            Msg::SyncDownload => {
                self.syncing = true;
                self.status_msg = self.i18n().downloading().into();
                return Task::perform(
                    async { hostsync_core::sync::download(None).await },
                    Msg::SyncDownloadDone,
                );
            }
            Msg::SyncDownloadDone(r) => {
                self.syncing = false;
                match &r {
                    Ok(_) => {
                        self.servers = storage::load_servers();
                        self.status_msg = self.i18n().downloaded_from_cloud().into();
                    }
                    Err(e) if e == hostsync_core::sync::ERR_NEED_PASSPHRASE => {
                        self.sync_passphrase_input.clear();
                        let is_new = !storage::has_sync_passphrase();
                        self.screen = Screen::SyncPassphrase {
                            is_new,
                            next_action: PassphraseAction::Download,
                        };
                    }
                    Err(e) => self.status_msg = self.i18n().download_failed(e),
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
                self.screen = Screen::Home;
                if added > 0 {
                    self.syncing = true;
                    self.status_msg = self.i18n().imported_hosts_syncing(added);
                    return Task::perform(
                        async { hostsync_core::sync::upload(None).await },
                        Msg::SyncUploadDone,
                    );
                } else {
                    self.status_msg = self.i18n().no_new_hosts_to_import().into();
                }
            }
            Msg::ImportPasteConfirm => {
                let imported = hostsync_core::ssh_config::parse(&self.paste_text.text());
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
                self.screen = Screen::Home;
                if added > 0 {
                    self.syncing = true;
                    self.status_msg = self.i18n().imported_hosts_syncing(added);
                    return Task::perform(
                        async { hostsync_core::sync::upload(None).await },
                        Msg::SyncUploadDone,
                    );
                } else {
                    self.status_msg = self.i18n().no_new_hosts_to_import().into();
                }
            }
            Msg::ExportClipboard => {
                let config = hostsync_core::ssh_config::generate(&self.servers);
                if let Ok(mut clip) = arboard::Clipboard::new() {
                    let _ = clip.set_text(config);
                    self.status_msg = self.i18n().ssh_config_copied().into();
                }
            }
            Msg::ExportSystem => {
                match hostsync_core::ssh_config::merge_into_system_config(&self.servers) {
                    Ok(_) => self.status_msg = self.i18n().merged_into_system_config().into(),
                    Err(e) => self.status_msg = self.i18n().export_failed(&e.to_string()),
                }
            }
            Msg::PasteTextChanged(action) => self.paste_text.perform(action),
            Msg::GoSettings => {
                let from_login = matches!(self.screen, Screen::Login);
                self.proxy_input = storage::load_proxy().unwrap_or_default();
                self.screen = Screen::Settings { from_login };
            }
            Msg::LanguageSelected(setting) => {
                self.language_setting = setting;
                self.language = i18n::resolve_language(setting);
                let _ = storage::save_language_setting(setting.as_storage_value());
            }
            Msg::ProxyInput(s) => self.proxy_input = s,
            Msg::ProxySave => {
                let _ = storage::save_proxy(&self.proxy_input);
                self.status_msg = if self.proxy_input.trim().is_empty() {
                    self.i18n().proxy_cleared().into()
                } else {
                    self.i18n().proxy_set_to(self.proxy_input.trim())
                };
                let from_login = matches!(self.screen, Screen::Settings { from_login: true });
                self.screen = if from_login {
                    Screen::Login
                } else {
                    Screen::Home
                };
            }
            Msg::SyncPassphraseInput(s) => self.sync_passphrase_input = s,
            Msg::SyncPassphraseConfirm => {
                if self.sync_passphrase_input.trim().is_empty() {
                    self.status_msg = self.i18n().passphrase_cannot_be_empty().into();
                    return Task::none();
                }
                let pp = self.sync_passphrase_input.clone();
                if let Screen::SyncPassphrase { next_action, .. } = &self.screen {
                    match next_action {
                        PassphraseAction::Upload => {
                            // Save passphrase and re-encrypt in-memory servers
                            // BEFORE calling upload, so disk data uses the new key
                            let servers = self.servers.clone();
                            let _ = storage::save_sync_passphrase(&pp);
                            let _ = storage::save_servers(&servers);
                            self.screen = Screen::Home;
                            self.syncing = true;
                            self.status_msg = self.i18n().uploading().into();
                            return Task::perform(
                                async move { hostsync_core::sync::upload(None).await },
                                Msg::SyncUploadDone,
                            );
                        }
                        PassphraseAction::Download => {
                            self.screen = Screen::Home;
                            self.syncing = true;
                            self.status_msg = self.i18n().downloading().into();
                            return Task::perform(
                                async move { hostsync_core::sync::download(Some(&pp)).await },
                                Msg::SyncDownloadDone,
                            );
                        }
                    }
                }
            }
            Msg::Noop => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Msg> {
        let i18n = self.i18n();
        match &self.screen {
            Screen::Login => ui::login_view(
                &self.status_msg,
                &self.device_user_code,
                self.logging_in,
                i18n,
            ),
            Screen::Home => ui::home_view(self, i18n),
            Screen::AddEdit(edit_idx) => ui::form_view(self, *edit_idx, i18n),
            Screen::ImportPaste => ui::paste_view(&self.paste_text, i18n),
            Screen::Settings { .. } => {
                ui::settings_view(&self.proxy_input, self.language_setting, i18n)
            }
            Screen::SyncPassphrase { is_new, .. } => {
                ui::passphrase_view(&self.sync_passphrase_input, *is_new, &self.status_msg, i18n)
            }
        }
    }
}
