//! The Settings page.

use egui::{Align, CornerRadius, Frame, Layout, Margin, Stroke, Vec2};

use crate::api::models::pick_image;
use crate::app::App;
use crate::i18n::{gettext, ngettext, pgettext};
use crate::model::{Action, Dialog};
use crate::theme::{self, Icon, Palette};

use super::widgets;

const PLAYBACK_DIRTY_ID: &str = "playback-settings-dirty";
pub(crate) const PERSONAL_APP_FOCUS_ID: &str = "focus-personal-app-setup";
const SETTINGS_FILTER_ID: &str = "settings-filter";

/// Whether a settings row matches the filter query (case-insensitive).
/// An empty query matches everything, so the page reads exactly as
/// before when the search field is empty.
pub(crate) fn row_matches(needle: &str, title: &str, description: &str) -> bool {
    let needle = needle.trim().to_lowercase();
    if needle.is_empty() {
        return true;
    }
    title.to_lowercase().contains(&needle) || description.to_lowercase().contains(&needle)
}

/// Text shared by section selection and the rendered row. Conditional rows
/// participate only on the platforms and account states where they exist.
pub(super) struct RowText<'a> {
    title: std::borrow::Cow<'a, str>,
    description: std::borrow::Cow<'a, str>,
    available: bool,
}

impl<'a> RowText<'a> {
    pub(super) fn new(
        title: impl Into<std::borrow::Cow<'a, str>>,
        description: impl Into<std::borrow::Cow<'a, str>>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            available: true,
        }
    }

    pub(super) fn when(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    fn matches(&self, needle: &str, section: &str) -> bool {
        self.available
            && (needle.is_empty()
                || section.to_lowercase().contains(needle)
                || row_matches(needle, &self.title, &self.description))
    }
}

/// The GitHub release page of the running version, opened by About's
/// Release notes button. Every release is tagged `v` and its version.
const RELEASE_NOTES_URL: &str = concat!(
    env!("CARGO_PKG_REPOSITORY"),
    "/releases/tag/v",
    env!("CARGO_PKG_VERSION")
);

fn section_matches(needle: &str, title: &str, rows: &[RowText<'_>]) -> bool {
    let needle = needle.trim().to_lowercase();
    rows.iter().any(|row| row.matches(&needle, title))
}

pub(super) fn filtered_row(
    ui: &mut egui::Ui,
    palette: &Palette,
    needle: &str,
    section: &str,
    row: &RowText<'_>,
    control: impl FnOnce(&mut egui::Ui),
) {
    if row.matches(needle, section) {
        widgets::setting_row(ui, palette, &row.title, &row.description, control);
    }
}

/// `filtered_row` for a row with no control.
pub(super) fn filtered_note(
    ui: &mut egui::Ui,
    palette: &Palette,
    needle: &str,
    section: &str,
    row: &RowText<'_>,
) {
    if row.matches(needle, section) {
        widgets::setting_note(ui, palette, &row.title, &row.description);
    }
}

/// `filtered_row` for a control that needs `control_width` points.
fn filtered_row_sized(
    ui: &mut egui::Ui,
    palette: &Palette,
    needle: &str,
    section: &str,
    row: &RowText<'_>,
    control_width: f32,
    control: impl FnOnce(&mut egui::Ui),
) {
    if row.matches(needle, section) {
        widgets::setting_row_sized(
            ui,
            palette,
            &row.title,
            &row.description,
            control_width,
            control,
        );
    }
}

/// Forget the search text, so a flow that lands on a specific row (like
/// the Personal App setup) always finds that row visible and focusable.
pub(crate) fn clear_search(ctx: &egui::Context) {
    ctx.data_mut(|data| data.remove::<String>(egui::Id::new(SETTINGS_FILTER_ID)));
}

fn section(
    ui: &mut egui::Ui,
    palette: &Palette,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    ui.add_space(10.0);
    theme::text(ui, title, theme::bold(18.0), palette.text);
    ui.add_space(8.0);
    Frame::new()
        .fill(
            palette
                .surface
                .gamma_multiply(if palette.dark { 0.7 } else { 1.0 }),
        )
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(theme::RADIUS + 2))
        // Every row leaves 10 points below itself for the next one, so the
        // bottom margin is that much less than the top one (#1037).
        .inner_margin(Margin {
            left: 20,
            right: 20,
            top: 16,
            bottom: 6,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width().min(760.0));
            add_contents(ui);
        });
    ui.add_space(8.0);
}

/// What the local files row says: the folder and how many songs it holds,
/// or what a folder is for.
fn local_files_detail(app: &App) -> String {
    let locale = app.locale;
    let Some(folder) = app.settings.local_files_folder() else {
        return gettext(
            locale,
            "Play the local files your playlists list from a folder on this computer.",
        )
        .into_owned();
    };
    let folder = folder.display().to_string();
    match &app.local_files {
        Some((_, uris)) => ngettext(
            locale,
            // Translators: {count} is a number of songs, {folder} a folder on this computer.
            "{count} song in {folder}",
            "{count} songs in {folder}",
            uris.len() as u32,
        )
        .replace("{count}", &uris.len().to_string())
        .replace("{folder}", &folder),
        // Translators: {folder} is a folder on this computer.
        None => gettext(locale, "Looking for songs in {folder}…").replace("{folder}", &folder),
    }
}

/// Soloist supplies audio, while this separate Spotify approval supplies
/// library features. Reuse the existing grant without changing players.
pub(super) fn spotify_access_control(app: &mut App, ui: &mut egui::Ui) {
    use crate::backend::LocalPlayback;
    let locale = app.locale;
    let palette = app.palette;
    let (status, busy, action) = match &app.local_playback {
        LocalPlayback::Ready { .. } => (gettext(locale, "Connected"), false, None),
        LocalPlayback::Authorizing => (
            gettext(locale, "Finish authorizing in your browser."),
            true,
            None,
        ),
        LocalPlayback::Connecting => (gettext(locale, "Connecting to Spotify…"), true, None),
        LocalPlayback::WaitingForNetwork => (gettext(locale, "Waiting for network…"), true, None),
        LocalPlayback::Unavailable => ("".into(), false, Some(gettext(locale, "Authorize"))),
        LocalPlayback::Failed(_) => ("".into(), false, Some(gettext(locale, "Try again"))),
    };
    if let Some(action) = action {
        let mut response = theme::pill_button(ui, &palette, &action, true);
        if let LocalPlayback::Failed(error) = &app.local_playback {
            response = response.on_hover_text(error);
        }
        if response.clicked() {
            app.actions.push(Action::EnablePlayback);
        }
    } else {
        ui.horizontal_wrapped(|ui| {
            if busy {
                ui.add(egui::Spinner::new().size(14.0).color(palette.secondary));
            }
            let galley = crate::bidi::layout(
                ui.painter(),
                &status,
                theme::regular(13.0),
                palette.secondary,
                ui.available_width(),
                usize::MAX,
                None,
            );
            ui.add(egui::Label::new(galley));
        });
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let locale = app.locale;
    ui.add_space(8.0);
    theme::text(
        ui,
        gettext(locale, "Settings"),
        theme::bold(28.0),
        palette.text,
    );
    ui.add_space(4.0);
    let mut filter = ui
        .data(|data| data.get_temp::<String>(egui::Id::new(SETTINGS_FILTER_ID)))
        .unwrap_or_default();
    widgets::search_field(
        ui,
        &palette,
        app.locale,
        egui::Id::new("settings-search-field"),
        &mut filter,
        &gettext(locale, "Search settings"),
        ui.available_width().min(400.0),
    );
    let needle = filter.trim().to_lowercase();
    ui.data_mut(|data| data.insert_temp(egui::Id::new(SETTINGS_FILTER_ID), filter));
    ui.add_space(4.0);
    let dirty_id = egui::Id::new(PLAYBACK_DIRTY_ID);
    let mut playback_dirty = ui
        .data(|data| data.get_temp::<bool>(dirty_id))
        .unwrap_or(false);
    let mut changed = false;
    let mut any_visible = false;
    let soloist = gettext(locale, "Spotify Soloist");
    let soloist_rows = super::soloist::rows(app);
    let soloist_keywords = row_matches(
        &needle,
        &soloist,
        &gettext(locale, "Playback lossless setup key download updates"),
    );
    if soloist_keywords || section_matches(&needle, &soloist, &soloist_rows) {
        any_visible = true;
        // A word only the section knows shows it whole.
        let rows_needle = if soloist_keywords {
            ""
        } else {
            needle.as_str()
        };
        section(ui, &palette, &soloist, |ui| {
            super::soloist::settings(app, ui, rows_needle, &soloist, &soloist_rows);
        });
    }

    let open_folder = gettext(locale, "Open folder");

    let wanted = app
        .settings
        .web_client_id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string);
    let in_use = wanted
        .as_deref()
        .is_some_and(|wanted| app.web_app.as_deref() == Some(wanted));
    let account = gettext(locale, "Account");
    let sign_out = gettext(locale, "Sign out");
    let account_rows = [
        RowText::new(
            gettext(locale, "Spotify registration"),
            if app.shared_grant && !in_use {
                gettext(
                    locale,
                    "You're still on the shared sign-in Spotifrost used before, which Spotify can stop accepting at any time. Register Spotifrost to keep listening.",
                )
            } else {
                gettext(
                    locale,
                    "Spotifrost connects with the Client ID you registered with Spotify.",
                )
            },
        ),
        RowText::new(
            gettext(locale, "Register Spotifrost"),
            gettext(
                locale,
                "It takes about two minutes in Spotify's developer dashboard.",
            ),
        )
        .when(!in_use),
        RowText::new(
            gettext(locale, "Spotifrost registered"),
            if app.shared_grant {
                gettext(
                    locale,
                    "Supported requests use your Client ID. Other requests use the shared sign-in.",
                )
            } else {
                gettext(
                    locale,
                    "Spotifrost uses your Client ID for Spotify's Web API.",
                )
            },
        )
        .when(in_use),
        RowText::new(
            gettext(locale, "Authorize your Client ID"),
            gettext(
                locale,
                "Spotify opens in your browser to verify the account.",
            ),
        )
        .when(!in_use && wanted.is_some()),
        RowText::new(sign_out.clone(), account.clone()),
    ];
    if section_matches(&needle, &account, &account_rows) {
        any_visible = true;
        section(ui, &palette, &account, |ui| {
            if row_matches(&needle, &format!("{sign_out} {account}"), "") {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 14.0;
                    let avatar = app.user.as_ref().and_then(|user| {
                        pick_image(&user.images, app.art_px(56.0)).map(str::to_string)
                    });
                    widgets::cover(ui, &palette, avatar.as_deref(), 56.0, 28.0, Icon::User);
                    ui.vertical(|ui| {
                        let name = app
                            .user
                            .as_ref()
                            .map(|user| user.name().to_string())
                            .unwrap_or_default();
                        theme::text(ui, name, theme::semibold(16.0), palette.text);
                        let product = app
                            .user
                            .as_ref()
                            .and_then(|user| user.product.clone())
                            .map(|product| match product.as_str() {
                                "premium" => "Spotify Premium".to_string(),
                                "free" | "open" => {
                                    gettext(locale, "Spotify Free, local playback needs Premium")
                                        .into_owned()
                                }
                                other => other.to_string(),
                            })
                            .unwrap_or_default();
                        theme::text(ui, product, theme::regular(13.0), palette.secondary);
                        if let Some(username) =
                            app.local.connected.then(|| app.local.username.clone())
                            && !username.is_empty()
                        {
                            theme::text(
                                ui,
                                // Translators: {username} is the Spotify account's user name.
                                gettext(locale, "Connected as {username}")
                                    .replace("{username}", &username),
                                theme::regular(12.0),
                                palette.dim,
                            );
                        }
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if theme::pill_button(ui, &palette, &sign_out, false).clicked() {
                            app.actions.push(Action::SignOut);
                        }
                    });
                });
                ui.add_space(10.0);
            }
            // What is typed stays a draft while it is empty: clearing the
            // field must not forget the app the session signs in with.
            let draft_id = egui::Id::new("personal-web-client-id-draft");
            let mut client_id = ui
                .data(|data| data.get_temp::<String>(draft_id))
                .unwrap_or_else(|| app.settings.web_client_id.clone().unwrap_or_default());
            filtered_row(ui, &palette, &needle, &account, &account_rows[0], |ui| {
                let response = Frame::new()
                    .fill(palette.surface)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::symmetric(10, 6))
                    .show(ui, |ui| {
                        widgets::text_edit(
                            ui,
                            app.locale,
                            egui::TextEdit::singleline(&mut client_id)
                                .id(egui::Id::new("personal-web-client-id"))
                                .hint_text(
                                    egui::RichText::new(gettext(locale, "Client ID"))
                                        .color(palette.dim),
                                )
                                .font(theme::regular(13.0))
                                .frame(egui::Frame::NONE)
                                .desired_width(200.0),
                        )
                    })
                    .inner;
                if ui
                    .data_mut(|data| data.remove_temp::<bool>(egui::Id::new(PERSONAL_APP_FOCUS_ID)))
                    .unwrap_or(false)
                {
                    response.scroll_to_me(Some(Align::Center));
                    response.request_focus();
                }
                if response.changed() {
                    let trimmed = client_id.trim().to_string();
                    if trimmed.is_empty() {
                        ui.data_mut(|data| data.insert_temp(draft_id, client_id.clone()));
                    } else {
                        ui.data_mut(|data| data.remove_temp::<String>(draft_id));
                        app.settings.web_client_id = Some(trimmed);
                        changed = true;
                    }
                }
            });
            filtered_row(ui, &palette, &needle, &account, &account_rows[1], |ui| {
                if theme::pill_button(ui, &palette, &gettext(locale, "Setup guide"), false)
                    .clicked()
                {
                    app.actions.push(Action::OpenUrl(
                        "https://github.com/spham2mail-star/spotifrost#set-up".into(),
                    ));
                }
            });
            if in_use {
                filtered_row(ui, &palette, &needle, &account, &account_rows[2], |_| {});
            } else if wanted.is_some() {
                filtered_row(ui, &palette, &needle, &account, &account_rows[3], |ui| {
                    if theme::pill_button(ui, &palette, &gettext(locale, "Authorize"), true)
                        .clicked()
                    {
                        app.actions.push(Action::ConfigurePersonalWebApp);
                    }
                });
            }
        });
    }

    let (status, detail, action) = match &app.local_playback {
        crate::backend::LocalPlayback::Ready { .. } => (
            pgettext(locale, "playback status", "Ready"),
            gettext(locale, "This computer is a Spotify Connect device."),
            None,
        ),
        crate::backend::LocalPlayback::Authorizing => (
            pgettext(locale, "playback status", "Setting up"),
            gettext(locale, "Finish authorizing in your browser."),
            None,
        ),
        crate::backend::LocalPlayback::Connecting => (
            pgettext(locale, "playback status", "Connecting"),
            gettext(locale, "Connecting to Spotify…"),
            None,
        ),
        crate::backend::LocalPlayback::WaitingForNetwork => (
            pgettext(locale, "playback status", "Waiting for network"),
            gettext(
                locale,
                "Waiting for network connectivity to resume playback.",
            ),
            None,
        ),
        crate::backend::LocalPlayback::Failed(message) => (
            pgettext(locale, "playback status", "Unavailable"),
            message.clone().into(),
            Some(gettext(locale, "Try again")),
        ),
        crate::backend::LocalPlayback::Unavailable => (
            pgettext(locale, "playback status", "Not set up"),
            gettext(
                locale,
                "Requires Spotify Premium and a one-time browser sign-in.",
            ),
            Some(gettext(locale, "Enable playback here")),
        ),
    };
    let built_in = !app.settings.soloist_enabled;
    let playback = if built_in {
        gettext(locale, "Playback on this computer")
    } else {
        gettext(locale, "Application")
    };
    let normalize_volume = gettext(locale, "Normalize volume");
    let gapless = gettext(locale, "Gapless playback");
    let keep_playing = gettext(locale, "Keep music playing when the window closes");
    let apply_playback = gettext(locale, "Apply and restart playback");
    let apply_playback_note = gettext(locale, "Restart local playback to apply these settings.");
    let start_with_computer = gettext(locale, "Start with your computer");
    let start_choice = app.start_with_computer();
    let start_detail = start_with_computer_detail(
        locale,
        app.autostart,
        start_choice,
        app.settings.keep_playing_in_background,
    );
    let start_available = app
        .autostart
        .is_some_and(|state| state != fastframe_autostart::State::Unsupported);
    let mut playback_rows = [
        RowText::new(
            // Translators: {status} is a playback state such as Ready or Not set up.
            gettext(locale, "Status: {status}").replace("{status}", &status),
            detail,
        ),
        RowText::new(
            gettext(locale, "Audio quality"),
            gettext(locale, "Higher bitrates use more data and cache space."),
        ),
        RowText::new(
            normalize_volume.clone(),
            gettext(locale, "Keep loud and quiet tracks at a similar level."),
        ),
        RowText::new(
            gapless.clone(),
            gettext(locale, "Play tracks without silence between them."),
        ),
        RowText::new(
            keep_playing.clone(),
            super::keys::platform_shortcut(
                &gettext(
                    locale,
                    "Spotifrost hides to the system tray. Quit from the tray menu or with Ctrl+Q.",
                ),
                // macOS keeps the Dock icon and the notch player instead.
                &gettext(
                    locale,
                    "Spotifrost keeps playing in the notch. Click the Dock icon to come back, or quit with Cmd+Q.",
                ),
            )
            .to_owned(),
        ),
        RowText::new(apply_playback.clone(), apply_playback_note.clone()).when(playback_dirty),
        RowText::new(gettext(locale, "Playback settings applied"), "").when(!playback_dirty),
        RowText::new(
            gettext(locale, "Notch player"),
            gettext(
                locale,
                "When the window is minimized or closed, the playing song waits in the notch.",
            ),
        )
        .when(cfg!(target_os = "macos")),
        RowText::new(gettext(locale, "Local files"), local_files_detail(app)),
        // The macOS menu bar already draws its icon black or white to match.
        RowText::new(
            gettext(locale, "Tray icon"),
            gettext(locale, "White or black can match your panel or taskbar."),
        )
        .when(!cfg!(target_os = "macos")),
        RowText::new(start_with_computer.clone(), start_detail).when(start_available),
    ];
    for index in [0, 1, 2, 3, 5, 6, 8] {
        playback_rows[index].available &= built_in;
    }
    if section_matches(&needle, &playback, &playback_rows) {
        any_visible = true;
        section(ui, &palette, &playback, |ui| {
            filtered_row(ui, &palette, &needle, &playback, &playback_rows[0], |ui| {
                if let Some(label) = action {
                    if theme::pill_button(ui, &palette, &label, true).clicked() {
                        app.actions.push(Action::EnablePlayback);
                    }
                } else if app.local_ready
                    && theme::soft_button(
                        ui,
                        &palette,
                        Some(Icon::Refresh),
                        &gettext(locale, "Reconnect"),
                        false,
                    )
                    .clicked()
                {
                    app.actions.push(Action::RestartEngine);
                }
            });
            // Normal to Very high, left to right when side by side and top
            // to bottom in a column.
            let choices = [
                (96u16, gettext(locale, "Normal · 96 kbps")),
                (160, gettext(locale, "High · 160 kbps")),
                (320, gettext(locale, "Very high · 320 kbps")),
            ];
            let choice_gap = 6.0;
            let choices_width = choices
                .iter()
                .map(|(_, label)| theme::soft_button_width(ui, label))
                .sum::<f32>()
                + choice_gap * (choices.len() - 1) as f32;
            filtered_row_sized(
                ui,
                &palette,
                &needle,
                &playback,
                &playback_rows[1],
                choices_width,
                |ui| {
                    let mut choose = |ui: &mut egui::Ui, kbps: u16, label: &str| {
                        if theme::soft_button(
                            ui,
                            &palette,
                            None,
                            label,
                            app.settings.bitrate == kbps,
                        )
                        .clicked()
                            && app.settings.bitrate != kbps
                        {
                            app.settings.bitrate = kbps;
                            changed = true;
                            playback_dirty = true;
                        }
                    };
                    if ui.available_width() >= choices_width {
                        // Laid right to left, so the last choice goes first.
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = choice_gap;
                            for (kbps, label) in choices.iter().rev() {
                                choose(ui, *kbps, label);
                            }
                        });
                    } else {
                        // Too narrow even for a line of their own: a column.
                        ui.with_layout(Layout::top_down(Align::Max), |ui| {
                            ui.spacing_mut().item_spacing.y = choice_gap;
                            for (kbps, label) in &choices {
                                choose(ui, *kbps, label);
                            }
                        });
                    }
                },
            );
            filtered_row(ui, &palette, &needle, &playback, &playback_rows[2], |ui| {
                if widgets::switch(
                    ui,
                    &palette,
                    &normalize_volume,
                    &mut app.settings.normalisation,
                )
                .changed()
                {
                    changed = true;
                    playback_dirty = true;
                }
            });
            filtered_row(ui, &palette, &needle, &playback, &playback_rows[3], |ui| {
                if widgets::switch(ui, &palette, &gapless, &mut app.settings.gapless).changed() {
                    changed = true;
                    playback_dirty = true;
                }
            });
            filtered_row(ui, &palette, &needle, &playback, &playback_rows[4], |ui| {
                if widgets::switch(
                    ui,
                    &palette,
                    &keep_playing,
                    &mut app.settings.keep_playing_in_background,
                )
                .changed()
                {
                    changed = true;
                }
            });
            {
                use crate::settings::TrayIcon;
                let choices = [
                    (TrayIcon::Colour, pgettext(locale, "tray icon", "Colour")),
                    (TrayIcon::White, pgettext(locale, "tray icon", "White")),
                    (TrayIcon::Black, pgettext(locale, "tray icon", "Black")),
                ];
                let choice_gap = 6.0;
                let choices_width = choices
                    .iter()
                    .map(|(_, label)| theme::soft_button_width(ui, label))
                    .sum::<f32>()
                    + choice_gap * (choices.len() - 1) as f32;
                filtered_row_sized(
                    ui,
                    &palette,
                    &needle,
                    &playback,
                    &playback_rows[9],
                    choices_width,
                    |ui| {
                        let mut choose = |ui: &mut egui::Ui, icon: TrayIcon, label: &str| {
                            if theme::soft_button(
                                ui,
                                &palette,
                                None,
                                label,
                                app.settings.tray_icon == icon,
                            )
                            .clicked()
                                && app.settings.tray_icon != icon
                            {
                                app.settings.tray_icon = icon;
                                changed = true;
                            }
                        };
                        if ui.available_width() >= choices_width {
                            // Laid right to left, so the last choice goes first.
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = choice_gap;
                                for (icon, label) in choices.iter().rev() {
                                    choose(ui, *icon, label);
                                }
                            });
                        } else {
                            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                                ui.spacing_mut().item_spacing.y = choice_gap;
                                for (icon, label) in &choices {
                                    choose(ui, *icon, label);
                                }
                            });
                        }
                    },
                );
            }
            if start_available {
                start_with_computer_row(
                    app,
                    ui,
                    &palette,
                    &needle,
                    &playback,
                    &playback_rows[10],
                    start_choice,
                );
            }
            #[cfg(target_os = "macos")]
            filtered_row(ui, &palette, &needle, &playback, &playback_rows[7], |ui| {
                if widgets::switch(
                    ui,
                    &palette,
                    &gettext(locale, "Notch player"),
                    &mut app.settings.notch_player,
                )
                .changed()
                {
                    changed = true;
                }
            });
            let choose_folder = gettext(locale, "Choose folder");
            let remove_folder = gettext(locale, "Remove");
            let has_folder = app.settings.local_files_folder().is_some();
            // Pill buttons are a little wider than soft ones; the open
            // button carries an icon. Narrow windows put them under the text.
            let pill_width = |label: &str| theme::soft_button_width(ui, label) + 16.0;
            let folder_controls_width = pill_width(&choose_folder)
                + if has_folder {
                    pill_width(&remove_folder)
                        + theme::soft_button_width(ui, &open_folder)
                        + 22.0
                        + 12.0
                } else {
                    0.0
                };
            filtered_row_sized(
                ui,
                &palette,
                &needle,
                &playback,
                &playback_rows[8],
                folder_controls_width,
                |ui| {
                    // The control area lays out right-to-left: add the rightmost item first.
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if has_folder {
                        if theme::pill_button(ui, &palette, &remove_folder, false).clicked() {
                            app.actions.push(Action::ForgetLocalFilesFolder);
                        }
                        if theme::soft_button(
                            ui,
                            &palette,
                            Some(Icon::ExternalLink),
                            &open_folder,
                            false,
                        )
                        .clicked()
                        {
                            app.actions.push(Action::OpenLocalFilesFolder);
                        }
                    }
                    if theme::pill_button(ui, &palette, &choose_folder, false).clicked() {
                        app.actions.push(Action::ChooseLocalFilesFolder);
                    }
                },
            );
            ui.add_space(4.0);
            if built_in
                && (playback_dirty
                    || playback_rows[5].matches(&needle, &playback)
                    || playback_rows[6].matches(&needle, &playback))
            {
                ui.horizontal(|ui| {
                    if playback_dirty {
                        if theme::pill_button(ui, &palette, &apply_playback, true).clicked() {
                            app.actions.push(Action::RestartEngine);
                            playback_dirty = false;
                        }
                        theme::subtle(ui, &palette, &apply_playback_note);
                    } else {
                        theme::subtle(ui, &palette, &gettext(locale, "Playback settings applied."));
                    }
                });
            }
        });
    }

    let storage = gettext(locale, "Storage");
    let mut storage_rows = [
        RowText::new(
            gettext(locale, "Artwork cache"),
            // Translators: {folder} is the path of a cache folder.
            gettext(locale, "Stored in {folder}")
                .replace("{folder}", &app.dirs.art_cache_dir().display().to_string()),
        ),
        RowText::new(
            gettext(locale, "Sign-in"),
            gettext(
                locale,
                "Sign-ins are saved in the system credential store when available.",
            ),
        ),
    ];
    if section_matches(&needle, &storage, &storage_rows) {
        any_visible = true;
        app.storage_usage.update(&app.dirs, ui.ctx());
        for (row, size) in storage_rows.iter_mut().zip(app.storage_usage.sizes) {
            let size = match size {
                crate::storage::Size::Checking => gettext(locale, "Checking…").into_owned(),
                crate::storage::Size::Bytes(bytes) => crate::storage::format_bytes(bytes),
                crate::storage::Size::Unavailable => gettext(locale, "Unavailable").into_owned(),
            };
            row.title = format!("{} · {size}", row.title).into();
        }
        section(ui, &palette, &storage, |ui| {
            filtered_row(ui, &palette, &needle, &storage, &storage_rows[0], |ui| {
                if theme::soft_button(
                    ui,
                    &palette,
                    Some(Icon::Trash),
                    &gettext(locale, "Clear artwork"),
                    false,
                )
                .clicked()
                {
                    app.actions.push(Action::ClearArtCache);
                }
            });
            filtered_row(ui, &palette, &needle, &storage, &storage_rows[1], |_| {});
        });
    }

    let about = gettext(locale, "About");
    // What the credit below says, for the settings search.
    let built_with = "Built for fun by spham2mail-star. Based on the open-source Spotifast by Carmine Paolino. Not affiliated with Spotify.";
    let keyboard_shortcuts = gettext(locale, "Keyboard shortcuts");
    let release_notes = gettext(locale, "Release notes");
    let source_code = gettext(locale, "Source code");
    let about_rows = [
        RowText::new(
            format!("Spotifrost {}", env!("CARGO_PKG_VERSION")),
            built_with,
        ),
        RowText::new(
            release_notes.to_string(),
            format!("{keyboard_shortcuts} {source_code}"),
        ),
    ];
    if section_matches(&needle, &about, &about_rows) {
        any_visible = true;
        section(ui, &palette, &about, |ui| {
            ui.horizontal(|ui| {
                let (logo, _) = ui.allocate_exact_size(Vec2::splat(40.0), egui::Sense::hover());
                theme::logo(ui, logo.center(), 40.0);
                ui.vertical(|ui| {
                    theme::text(
                        ui,
                        format!("Spotifrost {}", env!("CARGO_PKG_VERSION")),
                        theme::semibold(15.0),
                        palette.text,
                    );
                    if let Some(url) = widgets::credit(ui, &palette, locale) {
                        app.actions.push(Action::OpenUrl(url.to_owned()));
                    }
                });
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(8.0);
                if theme::soft_button(
                    ui,
                    &palette,
                    Some(Icon::ExternalLink),
                    &release_notes,
                    false,
                )
                .clicked()
                {
                    app.actions
                        .push(Action::OpenUrl(RELEASE_NOTES_URL.to_owned()));
                }
                if theme::soft_button(ui, &palette, Some(Icon::Info), &keyboard_shortcuts, false)
                    .clicked()
                {
                    app.actions.push(Action::ShowDialog(Dialog::Shortcuts));
                }
                if theme::soft_button(ui, &palette, Some(Icon::ExternalLink), &source_code, false)
                    .clicked()
                {
                    ui.ctx()
                        .open_url(egui::OpenUrl::new_tab(env!("CARGO_PKG_REPOSITORY")));
                }
            });
        });
    }

    if !needle.is_empty() && !any_visible {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Search,
            // Translators: {query} is the text typed into the settings search field.
            &gettext(locale, "No settings for “{query}”").replace("{query}", &needle),
            &gettext(locale, "Try fewer words, or check the spelling."),
        );
    }

    ui.data_mut(|data| data.insert_temp(dirty_id, playback_dirty));
    if changed {
        app.actions.push(Action::SettingsChanged);
    }
}

/// The choices for starting with the computer, left to right when side by
/// side and top to bottom in a column.
fn start_with_computer_choices(
    locale: crate::i18n::Locale,
) -> [(
    Option<fastframe_autostart::Start>,
    std::borrow::Cow<'static, str>,
); 3] {
    use fastframe_autostart::Start;
    [
        (None, pgettext(locale, "start with your computer", "Off")),
        (Some(Start::Window), gettext(locale, "With the window")),
        (
            Some(Start::Hidden),
            if cfg!(target_os = "macos") {
                gettext(locale, "In the menu bar")
            } else {
                gettext(locale, "In the tray")
            },
        ),
    ]
}

/// What the row says under its title: how to finish a registration the
/// system holds for approval, why a start in the tray opens the window
/// while nothing keeps Spotifrost there, or what the choices do.
fn start_with_computer_detail(
    locale: crate::i18n::Locale,
    state: Option<fastframe_autostart::State>,
    choice: Option<fastframe_autostart::Start>,
    keeps_running: bool,
) -> std::borrow::Cow<'static, str> {
    use fastframe_autostart::{Start, State};
    if matches!(state, Some(State::NeedsApproval(_))) && choice.is_some() {
        gettext(
            locale,
            "Allow Spotifrost in System Settings, General, Login Items.",
        )
    } else if choice == Some(Start::Hidden) && !keeps_running {
        if cfg!(target_os = "macos") {
            gettext(
                locale,
                "Starting in the menu bar needs Keep music playing when the window closes. Until then, the window opens.",
            )
        } else {
            gettext(
                locale,
                "Starting in the tray needs Keep music playing when the window closes. Until then, the window opens.",
            )
        }
    } else if cfg!(target_os = "macos") {
        gettext(
            locale,
            "Open Spotifrost when you log in, with its window or in the menu bar.",
        )
    } else {
        gettext(
            locale,
            "Open Spotifrost when you log in, with its window or in the tray.",
        )
    }
}

fn start_with_computer_row(
    app: &mut App,
    ui: &mut egui::Ui,
    palette: &Palette,
    needle: &str,
    section: &str,
    row: &RowText<'_>,
    current: Option<fastframe_autostart::Start>,
) {
    let choices = start_with_computer_choices(app.locale);
    let gap = 6.0;
    let width = choices
        .iter()
        .map(|(_, label)| theme::soft_button_width(ui, label))
        .sum::<f32>()
        + gap * (choices.len() - 1) as f32;
    filtered_row_sized(ui, palette, needle, section, row, width, |ui| {
        let mut choose =
            |ui: &mut egui::Ui, start: Option<fastframe_autostart::Start>, label: &str| {
                if theme::soft_button(ui, palette, None, label, current == start).clicked()
                    && current != start
                {
                    app.actions.push(Action::SetStartWithComputer(start));
                }
            };
        if ui.available_width() >= width {
            // Laid right to left, so the last choice goes first.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                for (start, label) in choices.iter().rev() {
                    choose(ui, *start, label);
                }
            });
        } else {
            ui.with_layout(Layout::top_down(Align::Max), |ui| {
                ui.spacing_mut().item_spacing.y = gap;
                for (start, label) in &choices {
                    choose(ui, *start, label);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{RELEASE_NOTES_URL, RowText, row_matches, section_matches};

    /// macOS has a menu bar where Linux and Windows have a tray, and the
    /// start-up choices say so (#1006).
    #[test]
    fn starting_hidden_names_the_menu_bar_on_macos_and_the_tray_elsewhere() {
        use fastframe_autostart::{Start, State};
        let locale = crate::i18n::Locale::English;
        let [_, _, (_, hidden)] = super::start_with_computer_choices(locale);
        let texts = [
            hidden.into_owned(),
            super::start_with_computer_detail(
                locale,
                Some(State::On(Start::Hidden)),
                Some(Start::Hidden),
                false,
            )
            .into_owned(),
            super::start_with_computer_detail(locale, Some(State::Off), None, true).into_owned(),
        ];
        for text in texts {
            if cfg!(target_os = "macos") {
                assert!(
                    text.contains("menu bar") && !text.contains("tray"),
                    "{text}"
                );
            } else {
                assert!(
                    text.contains("tray") && !text.contains("menu bar"),
                    "{text}"
                );
            }
        }
    }

    #[test]
    fn release_notes_open_the_running_version_release_page() {
        assert_eq!(
            RELEASE_NOTES_URL,
            format!(
                "https://github.com/spham2mail-star/spotifrost/releases/tag/v{}",
                env!("CARGO_PKG_VERSION")
            )
        );
    }

    #[test]
    fn empty_filter_matches_everything() {
        assert!(row_matches(
            "",
            "Normalize volume",
            "Keep loud tracks level"
        ));
        assert!(section_matches(
            "",
            "Playback on this computer",
            &[RowText::new("Status", "Spotify")]
        ));
    }

    #[test]
    fn row_matches_title_or_description_case_insensitively() {
        assert!(row_matches(
            "volume",
            "Normalize volume",
            "Keep loud tracks level"
        ));
        assert!(row_matches(
            "VOLUME",
            "Normalize volume",
            "Keep loud tracks level"
        ));
        assert!(row_matches(
            "quiet",
            "Normalize volume",
            "Keep loud and quiet tracks level"
        ));
        assert!(!row_matches(
            "theme",
            "Normalize volume",
            "Keep loud tracks level"
        ));
    }

    #[test]
    fn section_matches_title_or_any_row() {
        let rows = [
            RowText::new("Normalize volume", "Keep loud and quiet tracks level"),
            RowText::new("Autoplay", "Keep playing similar songs"),
        ];
        assert!(section_matches(
            "playback",
            "Playback on this computer",
            &rows
        ));
        assert!(section_matches(
            "autoplay",
            "Playback on this computer",
            &rows
        ));
        assert!(section_matches("quiet", "Playback on this computer", &rows));
        assert!(!section_matches(
            "theme",
            "Playback on this computer",
            &rows
        ));
    }
}
