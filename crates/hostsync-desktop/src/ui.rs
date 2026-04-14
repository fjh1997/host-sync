use crate::{App, Msg};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

pub fn login_view<'a>(status: &'a str, user_code: &'a str, logging_in: bool) -> Element<'a, Msg> {
    let mut content = column![
        Space::with_height(80),
        text("HostSync").size(40),
        text("Manage your Linux servers & SSH keys").size(14),
        Space::with_height(30),
    ]
    .spacing(12)
    .align_x(Alignment::Center);

    if !user_code.is_empty() {
        // Show the device code for the user to enter
        content = content
            .push(text("Enter this code on GitHub:").size(14))
            .push(text(user_code).size(36))
            .push(text("(copied to clipboard)").size(12))
            .push(text("Waiting for authorization...").size(13));
    } else {
        content = content.push(
            button(text(if logging_in { "Requesting..." } else { "Sign in with GitHub" }).size(16))
                .padding([12, 32])
                .on_press_maybe(if logging_in { None } else { Some(Msg::Login) }),
        );
    }

    if !status.is_empty() && user_code.is_empty() {
        content = content.push(text(status).size(13));
    }

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn home_view(app: &App) -> Element<'_, Msg> {
    let state = hostsync_core::storage::load_github_state();
    let username = state.username.clone().unwrap_or_else(|| "User".to_string());

    // Toolbar
    let toolbar = row![
        text("HostSync").size(22),
        horizontal_space(),
        button("Upload").on_press(if app.syncing { Msg::Noop } else { Msg::SyncUpload }),
        button("Download").on_press(if app.syncing { Msg::Noop } else { Msg::SyncDownload }),
        button("Import SSH").on_press(Msg::ImportSystem),
        button("Import Text").on_press(Msg::GoImportPaste),
        button("Export Copy").on_press(Msg::ExportClipboard),
        button("Export SSH").on_press(Msg::ExportSystem),
        text(username).size(14),
        button("Logout").on_press(Msg::Logout),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(8);

    // Search
    let search = text_input("Search servers...", &app.search)
        .on_input(Msg::SearchChanged)
        .padding(8);

    // Server list
    let indices = app.filtered_indices();
    let list: Element<Msg> = if indices.is_empty() {
        container(
            column![
                text(if app.servers.is_empty() {
                    "No servers yet"
                } else {
                    "No matching servers"
                })
                .size(16),
                if app.servers.is_empty() {
                    Element::<Msg>::from(button("Add your first server").on_press(Msg::GoAdd))
                } else {
                    Space::new(0, 0).into()
                },
            ]
            .spacing(8)
            .align_x(Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    } else {
        let items: Vec<Element<Msg>> = indices
            .iter()
            .map(|&i| server_row(&app.servers[i], i))
            .collect();
        scrollable(column(items).spacing(4).padding(8)).into()
    };

    // Status bar
    let status = if !app.status_msg.is_empty() {
        text(&app.status_msg).size(12)
    } else {
        text("").size(12)
    };

    let content = column![
        toolbar,
        container(search).padding([0, 8]),
        list,
        row![
            status,
            horizontal_space(),
            button(text("+  Add Server").size(14))
                .padding([8, 16])
                .on_press(Msg::GoAdd),
        ]
        .padding(8)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    container(content).width(Length::Fill).height(Length::Fill).into()
}

fn server_row(server: &hostsync_core::model::Server, idx: usize) -> Element<'static, Msg> {
    let auth_icon = match server.auth_type {
        hostsync_core::model::AuthType::Key => "[key]",
        hostsync_core::model::AuthType::Password => "[pw]",
    };

    let info = column![
        text(server.name.clone()).size(15),
        text(format!(
            "{}@{}:{}  {}",
            server.username, server.host, server.port, auth_icon
        ))
        .size(12),
    ]
    .spacing(2);

    let notes_text = server.notes.clone().unwrap_or_default();
    let notes_row: Element<'static, Msg> = if !notes_text.is_empty() {
        text(notes_text).size(11).into()
    } else {
        Space::new(0, 0).into()
    };

    row![
        column![info, notes_row].spacing(2).width(Length::Fill),
        button("Connect").on_press(Msg::Connect(idx)),
        button("Edit").on_press(Msg::GoEdit(idx)),
        button("Delete").on_press(Msg::Delete(idx)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(8)
    .into()
}

pub fn form_view(app: &App, edit_idx: Option<usize>) -> Element<'_, Msg> {
    let title = if edit_idx.is_some() {
        "Edit Server"
    } else {
        "Add Server"
    };

    let is_key = app.form_auth_type == hostsync_core::model::AuthType::Key;

    let auth_buttons = row![
        button(if !is_key { "> Password" } else { "  Password" })
            .on_press(Msg::FormAuthPassword),
        button(if is_key { "> SSH Key" } else { "  SSH Key" })
            .on_press(Msg::FormAuthKey),
    ]
    .spacing(8);

    let mut fields = column![
        text(title).size(22),
        Space::with_height(8),
        text("Host Alias (Name)").size(13),
        text_input("e.g. prod-web", &app.form_name).on_input(Msg::FormName).padding(8),
        row![
            column![
                text("HostName").size(13),
                text_input("e.g. 192.168.1.100", &app.form_host).on_input(Msg::FormHost).padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(3)),
            column![
                text("Port").size(13),
                text_input("22", &app.form_port).on_input(Msg::FormPort).padding(8),
            ]
            .spacing(4)
            .width(Length::FillPortion(1)),
        ]
        .spacing(8),
        text("User").size(13),
        text_input("root", &app.form_user).on_input(Msg::FormUser).padding(8),
        text("Authentication").size(13),
        auth_buttons,
    ]
    .spacing(6)
    .padding(16);

    if is_key {
        fields = fields
            .push(text("IdentityFile (path)").size(13))
            .push(
                text_input("~/.ssh/id_rsa", &app.form_identity_file)
                    .on_input(Msg::FormIdentityFile)
                    .padding(8),
            )
            .push(text("Private Key Content (optional)").size(13))
            .push(
                text_input("-----BEGIN OPENSSH PRIVATE KEY-----", &app.form_private_key)
                    .on_input(Msg::FormPrivateKey)
                    .padding(8),
            )
            .push(text("Key Passphrase (optional)").size(13))
            .push(
                text_input("", &app.form_passphrase)
                    .on_input(Msg::FormPassphrase)
                    .padding(8)
                    .secure(true),
            );
    } else {
        fields = fields.push(text("Password").size(13)).push(
            text_input("", &app.form_password)
                .on_input(Msg::FormPassword)
                .padding(8)
                .secure(true),
        );
    }

    // SSH config preview
    let preview = format!(
        "Host {}\n    HostName {}\n{}    User {}{}",
        if app.form_name.trim().is_empty() { "<alias>" } else { app.form_name.trim() },
        if app.form_host.trim().is_empty() { "<hostname>" } else { app.form_host.trim() },
        if app.form_port.trim() != "22" && !app.form_port.trim().is_empty() {
            format!("    Port {}\n", app.form_port.trim())
        } else {
            String::new()
        },
        if app.form_user.trim().is_empty() { "root" } else { app.form_user.trim() },
        if is_key && !app.form_identity_file.trim().is_empty() {
            format!("\n    IdentityFile {}", app.form_identity_file.trim())
        } else {
            String::new()
        },
    );

    fields = fields
        .push(text("Notes (optional)").size(13))
        .push(
            text_input("", &app.form_notes)
                .on_input(Msg::FormNotes)
                .padding(8),
        )
        .push(Space::with_height(8))
        .push(text("SSH Config Preview").size(13))
        .push(text(preview).size(12))
        .push(Space::with_height(12))
        .push(
            row![
                button("Cancel").on_press(Msg::GoHome),
                horizontal_space(),
                button(text("Save").size(14)).padding([8, 24]).on_press(Msg::FormSave),
            ]
            .spacing(8),
        );

    scrollable(container(fields).width(Length::Fill).padding(16)).into()
}

pub fn paste_view(paste_text: &str) -> Element<'_, Msg> {
    let content = column![
        text("Paste SSH Config").size(22),
        Space::with_height(8),
        text_input(
            "Host myserver\n    HostName 192.168.1.1\n    User root",
            paste_text,
        )
        .on_input(Msg::PasteTextChanged)
        .padding(8),
        Space::with_height(12),
        row![
            button("Cancel").on_press(Msg::GoHome),
            horizontal_space(),
            button("Import").padding([8, 24]).on_press(Msg::ImportPasteConfirm),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(20);

    container(content).width(Length::Fill).height(Length::Fill).into()
}
