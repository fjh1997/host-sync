use crate::i18n::{system_language, I18n, Language, LanguageSetting};
use crate::{App, Msg};
use iced::widget::{
    button, column, container, horizontal_space, row, scrollable, text, text_editor, text_input,
    Space,
};
use iced::{Alignment, Element, Length};

pub fn login_view<'a>(
    status: &'a str,
    user_code: &'a str,
    logging_in: bool,
    i18n: I18n,
) -> Element<'a, Msg> {
    let mut content = column![
        Space::with_height(80),
        text("HostSync").size(40),
        text(format!(
            "v{} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("HOSTSYNC_BUILD")
        ))
        .size(12),
        text(i18n.manage_servers()).size(14),
        Space::with_height(30),
    ]
    .spacing(12)
    .align_x(Alignment::Center);

    if !user_code.is_empty() {
        // Show the device code for the user to enter (selectable so user can copy)
        content = content
            .push(text(i18n.enter_code_on_github()).size(14))
            .push(text_input("", user_code).size(36).padding(8))
            .push(text(i18n.copied_to_clipboard()).size(12))
            .push(text(i18n.waiting_for_authorization()).size(13));
    } else {
        content = content.push(
            button(
                text(if logging_in {
                    i18n.requesting()
                } else {
                    i18n.sign_in_with_github()
                })
                .size(16),
            )
            .padding([12, 32])
            .on_press_maybe(if logging_in { None } else { Some(Msg::Login) }),
        );
    }

    if !status.is_empty() && user_code.is_empty() {
        content = content.push(text(status).size(13));
    }

    content = content
        .push(Space::with_height(16))
        .push(button(text(i18n.settings()).size(13)).on_press(Msg::GoSettings));

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn home_view(app: &App, i18n: I18n) -> Element<'_, Msg> {
    let state = hostsync_core::storage::load_github_state();
    let username = state
        .username
        .clone()
        .unwrap_or_else(|| i18n.user_fallback().to_string());

    // Toolbar
    let toolbar = row![
        text("HostSync").size(22),
        text(format!(
            "v{} ({})",
            env!("CARGO_PKG_VERSION"),
            env!("HOSTSYNC_BUILD")
        ))
        .size(11),
        horizontal_space(),
        button(i18n.upload()).on_press(if app.syncing {
            Msg::Noop
        } else {
            Msg::SyncUpload
        }),
        button(i18n.download()).on_press(if app.syncing {
            Msg::Noop
        } else {
            Msg::SyncDownload
        }),
        button(i18n.import_ssh()).on_press(Msg::ImportSystem),
        button(i18n.import_text()).on_press(Msg::GoImportPaste),
        button(i18n.export_copy()).on_press(Msg::ExportClipboard),
        button(i18n.export_ssh()).on_press(Msg::ExportSystem),
        button(i18n.settings()).on_press(Msg::GoSettings),
        text(username).size(14),
        button(i18n.logout()).on_press(Msg::Logout),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(8);

    // Search
    let search = text_input(i18n.search_servers(), &app.search)
        .on_input(Msg::SearchChanged)
        .padding(8);

    // Server list
    let indices = app.filtered_indices();
    let list: Element<Msg> = if indices.is_empty() {
        container(
            column![
                text(if app.servers.is_empty() {
                    i18n.no_servers_yet()
                } else {
                    i18n.no_matching_servers()
                })
                .size(16),
                if app.servers.is_empty() {
                    Element::<Msg>::from(button(i18n.add_first_server()).on_press(Msg::GoAdd))
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
            .map(|&i| server_row(&app.servers[i], i, i18n))
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
            button(text(i18n.add_server()).size(14))
                .padding([8, 16])
                .on_press(Msg::GoAdd),
        ]
        .padding(8)
        .align_y(Alignment::Center),
    ]
    .spacing(4);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn server_row(
    server: &hostsync_core::model::Server,
    idx: usize,
    i18n: I18n,
) -> Element<'static, Msg> {
    let auth_icon = match server.auth_type {
        hostsync_core::model::AuthType::Key => i18n.key_badge(),
        hostsync_core::model::AuthType::Password => i18n.password_badge(),
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
        button(i18n.connect()).on_press(Msg::Connect(idx)),
        button(i18n.copy()).on_press(Msg::CopyCommand(idx)),
        button(i18n.edit()).on_press(Msg::GoEdit(idx)),
        button(i18n.delete()).on_press(Msg::Delete(idx)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(8)
    .into()
}

pub fn form_view(app: &App, edit_idx: Option<usize>, i18n: I18n) -> Element<'_, Msg> {
    let title = if edit_idx.is_some() {
        i18n.edit_server()
    } else {
        i18n.add_server_title()
    };

    let is_key = app.form_auth_type == hostsync_core::model::AuthType::Key;

    let auth_buttons = row![
        button(text(i18n.password()).size(14).align_x(Alignment::Center))
            .width(Length::Fill)
            .padding(10)
            .style(if !is_key {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Msg::FormAuthPassword),
        button(text(i18n.ssh_key()).size(14).align_x(Alignment::Center))
            .width(Length::Fill)
            .padding(10)
            .style(if is_key {
                button::primary
            } else {
                button::secondary
            })
            .on_press(Msg::FormAuthKey),
    ]
    .spacing(1)
    .width(Length::Fill);

    let mut fields = column![
        text(title).size(26),
        Space::with_height(12),
        // Basic Info Group
        column![
            text(i18n.basic_information()).size(16),
            Space::with_height(4),
            column![
                text(i18n.host_alias()).size(13),
                text_input(i18n.host_alias_placeholder(), &app.form_name)
                    .on_input(Msg::FormName)
                    .padding(10),
            ]
            .spacing(4),
            row![
                column![
                    text(i18n.hostname()).size(13),
                    text_input(i18n.hostname_placeholder(), &app.form_host)
                        .on_input(Msg::FormHost)
                        .padding(10),
                ]
                .spacing(4)
                .width(Length::FillPortion(3)),
                column![
                    text(i18n.port()).size(13),
                    text_input("22", &app.form_port)
                        .on_input(Msg::FormPort)
                        .padding(10),
                ]
                .spacing(4)
                .width(Length::FillPortion(1)),
            ]
            .spacing(12),
            column![
                text(i18n.user()).size(13),
                text_input("root", &app.form_user)
                    .on_input(Msg::FormUser)
                    .padding(10),
            ]
            .spacing(4),
        ]
        .spacing(12),
        Space::with_height(16),
        // Auth Group
        column![
            text(i18n.authentication()).size(16),
            Space::with_height(4),
            auth_buttons,
        ]
        .spacing(8),
    ]
    .spacing(16);

    let key_path_hint = if cfg!(windows) {
        "e.g. C:\\Users\\you\\.ssh\\id_rsa  or  ~/.ssh/id_rsa"
    } else {
        "e.g. ~/.ssh/id_rsa"
    };

    if is_key {
        fields = fields
            .push(
                column![
                    text(i18n.identity_file()).size(13),
                    row![
                        text_input(key_path_hint, &app.form_identity_file)
                            .on_input(Msg::FormIdentityFile)
                            .padding(10),
                        button(i18n.browse())
                            .padding(10)
                            .on_press(Msg::FormBrowsePrivateKey),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(4),
            )
            .push(
                column![
                    text(i18n.private_key_optional()).size(13),
                    container(
                        text_editor(&app.form_private_key)
                            .on_action(Msg::FormPrivateKey)
                            .padding(8),
                    )
                    .height(180)
                    .style(container::bordered_box),
                ]
                .spacing(4),
            )
            .push(
                column![
                    text(i18n.key_passphrase_optional()).size(13),
                    text_input("", &app.form_passphrase)
                        .on_input(Msg::FormPassphrase)
                        .padding(10)
                        .secure(true),
                ]
                .spacing(4),
            );
    } else {
        fields = fields.push(
            column![
                text(i18n.password()).size(13),
                text_input("", &app.form_password)
                    .on_input(Msg::FormPassword)
                    .padding(10)
                    .secure(true),
            ]
            .spacing(4),
        );
    }

    // SSH config preview
    let preview = format!(
        "Host {}\n    HostName {}\n{}    User {}{}",
        if app.form_name.trim().is_empty() {
            i18n.preview_alias_placeholder()
        } else {
            app.form_name.trim()
        },
        if app.form_host.trim().is_empty() {
            i18n.preview_hostname_placeholder()
        } else {
            app.form_host.trim()
        },
        if app.form_port.trim() != "22" && !app.form_port.trim().is_empty() {
            format!("    Port {}\n", app.form_port.trim())
        } else {
            String::new()
        },
        if app.form_user.trim().is_empty() {
            "root"
        } else {
            app.form_user.trim()
        },
        if is_key && !app.form_identity_file.trim().is_empty() {
            format!("\n    IdentityFile {}", app.form_identity_file.trim())
        } else {
            String::new()
        },
    );

    fields = fields
        .push(
            column![
                text(i18n.notes_optional()).size(13),
                container(
                    text_editor(&app.form_notes)
                        .on_action(Msg::FormNotes)
                        .padding(8),
                )
                .height(100)
                .style(container::bordered_box),
            ]
            .spacing(4),
        )
        .push(Space::with_height(8))
        .push(
            column![
                text(i18n.ssh_config_preview()).size(14),
                container(scrollable(
                    text(preview).size(13).font(iced::Font::MONOSPACE)
                ))
                .padding(12)
                .width(Length::Fill)
                .style(|_theme| {
                    container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb8(
                            30, 30, 30,
                        ))),
                        border: iced::Border {
                            color: iced::Color::from_rgb8(60, 60, 60),
                            width: 1.0,
                            radius: 4.0.into(),
                        },
                        ..Default::default()
                    }
                }),
            ]
            .spacing(8),
        )
        .push(Space::with_height(16))
        .push(
            row![
                button(i18n.cancel())
                    .padding([10, 20])
                    .on_press(Msg::GoHome),
                horizontal_space(),
                button(text(i18n.save()).size(16))
                    .padding([10, 40])
                    .on_press(Msg::FormSave),
            ]
            .spacing(12),
        );

    scrollable(
        container(fields)
            .width(Length::Fill)
            .max_width(600)
            .padding(20),
    )
    .into()
}

pub fn paste_view(paste_text: &text_editor::Content, i18n: I18n) -> Element<'_, Msg> {
    let content = column![
        text(i18n.paste_ssh_config()).size(22),
        Space::with_height(8),
        container(
            text_editor(paste_text)
                .on_action(Msg::PasteTextChanged)
                .padding(8),
        )
        .height(300)
        .style(container::bordered_box),
        Space::with_height(12),
        row![
            button(i18n.cancel()).on_press(Msg::GoHome),
            horizontal_space(),
            button(i18n.import())
                .padding([8, 24])
                .on_press(Msg::ImportPasteConfirm),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn settings_view(
    proxy_input: &str,
    language_setting: LanguageSetting,
    i18n: I18n,
) -> Element<'_, Msg> {
    let current = hostsync_core::storage::load_proxy();
    let status_text = match &current {
        Some(p) => i18n.current_proxy(p),
        None => i18n.no_proxy_configured().to_string(),
    };
    let system_name = match system_language() {
        Language::English => i18n.language_english(),
        Language::Chinese => i18n.language_chinese(),
    };

    let content = column![
        text(i18n.settings()).size(22),
        Space::with_height(8),
        text(i18n.language()).size(16),
        text(i18n.system_language_detected(system_name)).size(13),
        row![
            button(text(i18n.language_follow_system()).size(13))
                .padding([8, 12])
                .style(if language_setting == LanguageSetting::System {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Msg::LanguageSelected(LanguageSetting::System)),
            button(text(i18n.language_english()).size(13))
                .padding([8, 12])
                .style(if language_setting == LanguageSetting::English {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Msg::LanguageSelected(LanguageSetting::English)),
            button(text(i18n.language_chinese()).size(13))
                .padding([8, 12])
                .style(if language_setting == LanguageSetting::Chinese {
                    button::primary
                } else {
                    button::secondary
                })
                .on_press(Msg::LanguageSelected(LanguageSetting::Chinese)),
        ]
        .spacing(8),
        Space::with_height(12),
        text(i18n.proxy_settings()).size(16),
        text(status_text).size(13),
        Space::with_height(12),
        text(i18n.proxy_url_label()).size(13),
        text_input(i18n.proxy_url_placeholder(), proxy_input)
            .on_input(Msg::ProxyInput)
            .padding(8),
        text(i18n.proxy_direct_hint()).size(12),
        Space::with_height(16),
        row![
            button(i18n.cancel()).on_press(Msg::GoHome),
            horizontal_space(),
            button(text(i18n.save()).size(14))
                .padding([8, 24])
                .on_press(Msg::ProxySave),
        ]
        .spacing(8),
    ]
    .spacing(8)
    .padding(20);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn passphrase_view<'a>(
    passphrase_input: &'a str,
    is_new: bool,
    status_msg: &'a str,
    i18n: I18n,
) -> Element<'a, Msg> {
    let title = if is_new {
        i18n.set_sync_passphrase()
    } else {
        i18n.enter_sync_passphrase()
    };
    let description = if is_new {
        i18n.sync_passphrase_description_new()
    } else {
        i18n.sync_passphrase_description_existing()
    };

    let mut content = column![
        Space::with_height(60),
        text(title).size(24),
        Space::with_height(8),
        text(description).size(13),
        Space::with_height(20),
        text(i18n.sync_passphrase()).size(13),
        text_input(i18n.enter_passphrase_placeholder(), passphrase_input)
            .on_input(Msg::SyncPassphraseInput)
            .on_submit(Msg::SyncPassphraseConfirm)
            .padding(10)
            .secure(true),
        Space::with_height(16),
        row![
            button(i18n.cancel()).on_press(Msg::GoHome),
            horizontal_space(),
            button(text(i18n.confirm()).size(14))
                .padding([8, 24])
                .on_press(Msg::SyncPassphraseConfirm),
        ]
        .spacing(8),
    ]
    .spacing(6)
    .align_x(Alignment::Center)
    .max_width(400);

    if !status_msg.is_empty() {
        content = content
            .push(Space::with_height(8))
            .push(text(status_msg).size(12));
    }

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
