use spotifrost::{app, backend, paths, settings, single_instance, util};

use clap::{CommandFactory, FromArgMatches, Parser};
use fastframe_shell::window::Geometry;

/// A fast, native Spotify client.
#[derive(Debug, Parser)]
#[command(name = env!("CARGO_BIN_NAME"), version, about)]
struct Cli {
    /// A command for the running instance; without one, the app starts.
    #[command(subcommand)]
    control: Option<Control>,

    /// A Spotify link to open: spotify:track:…, or an open.spotify.com
    /// address. The running Spotifrost opens it when there is one, which
    /// is how the desktop hands links over.
    #[arg(value_name = "LINK")]
    link: Option<String>,

    /// Log more from librespot and the Web API client.
    #[arg(short, long)]
    verbose: bool,

    /// Start in the tray, without a window, as a start at login in the
    /// tray does. Without the tray, or with "Keep music playing when the
    /// window closes" off, the window opens anyway.
    #[arg(long)]
    start_hidden: bool,
}

/// Remote control of the running instance, for Raycast scripts, launchers,
/// and hands on keyboards.
#[derive(Debug, clap::Subcommand)]
enum Control {
    /// Toggle play/pause
    PlayPause,
    /// Start playback if paused
    Play,
    /// Pause playback if playing
    Pause,
    /// Skip to the next track
    Next,
    /// Return to the previous track
    Previous,
    /// Seek by this many seconds; negative seeks backwards
    Seek {
        #[arg(allow_negative_numbers = true)]
        seconds: i64,
    },
    /// Seek to a position, in seconds from the start
    SeekTo { seconds: u32 },
    /// Set the volume to a percentage
    Volume {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        percent: u8,
    },
    /// Raise the volume
    VolumeUp {
        #[arg(default_value_t = 10, value_parser = clap::value_parser!(u8).range(1..=100))]
        percent: u8,
    },
    /// Lower the volume
    VolumeDown {
        #[arg(default_value_t = 10, value_parser = clap::value_parser!(u8).range(1..=100))]
        percent: u8,
    },
    /// Toggle mute
    Mute,
    /// Toggle shuffle, or set it outright
    Shuffle { state: Option<OnOff> },
    /// Cycle the repeat mode, or set it outright
    Repeat { mode: Option<Repeat> },
    /// Save the playing track to your library, or take it back out
    Like,
    /// Play a Spotify URI: a track, album, playlist, artist, or show
    PlayUri { uri: String },
    /// Search Spotify and play the best match: `play-search danza rota`, or
    /// `play-search --kind artist soda stereo`
    PlaySearch {
        /// What to look for: a song, or an album, artist, or playlist to play from the start
        #[arg(long, value_enum, default_value_t = SearchKind::Track)]
        kind: SearchKind,
        /// The words to search for
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,
    },
    /// List the Spotify Connect devices
    Devices {
        /// Print the JSON the running instance sent instead.
        #[arg(long)]
        raw: bool,
    },
    /// Move playback to a device, by the id `devices` prints
    Transfer { device_id: String },
    /// Print the playing track
    NowPlaying {
        /// Print the fields tab-separated instead: state, title, artists,
        /// album, position_ms, duration_ms, volume, shuffle, repeat,
        /// art_url, saved, device.
        #[arg(long)]
        raw: bool,
    },
    /// Bring the window of the running instance forward
    Show,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum SearchKind {
    Track,
    Album,
    Artist,
    Playlist,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum OnOff {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Repeat {
    /// Play through and stop
    Off,
    /// Repeat the album, playlist, or queue
    Context,
    /// Repeat this track
    Track,
}

/// Sends one control verb to the running instance over the
/// single-instance channel.
fn run_control(control: Control) -> i32 {
    let raw = matches!(
        control,
        Control::NowPlaying { raw: true } | Control::Devices { raw: true }
    );
    let verb = match control {
        Control::PlayPause => "playpause".to_owned(),
        Control::Play => "play".to_owned(),
        Control::Pause => "pause".to_owned(),
        Control::Next => "next".to_owned(),
        Control::Previous => "previous".to_owned(),
        Control::Seek { seconds } => format!("seek-by {}", seconds.saturating_mul(1000)),
        Control::SeekTo { seconds } => format!("seek-to {}", u64::from(seconds) * 1000),
        Control::Volume { percent } => format!("volume-set {percent}"),
        Control::VolumeUp { percent } => format!("volume-by {percent}"),
        Control::VolumeDown { percent } => format!("volume-by -{percent}"),
        Control::Mute => "mute".to_owned(),
        Control::Shuffle { state: None } => "shuffle".to_owned(),
        Control::Shuffle { state: Some(state) } => {
            let state = match state {
                OnOff::On => "on",
                OnOff::Off => "off",
            };
            format!("shuffle-set {state}")
        }
        Control::Repeat { mode: None } => "repeat".to_owned(),
        Control::Repeat { mode: Some(mode) } => {
            let mode = match mode {
                Repeat::Off => "off",
                Repeat::Context => "context",
                Repeat::Track => "track",
            };
            format!("repeat-set {mode}")
        }
        Control::Like => "save-toggle".to_owned(),
        Control::PlayUri { uri } => format!("play-uri {uri}"),
        Control::PlaySearch { kind, query } => {
            let kind = match kind {
                SearchKind::Track => "track",
                SearchKind::Album => "album",
                SearchKind::Artist => "artist",
                SearchKind::Playlist => "playlist",
            };
            format!("play-search {kind} {}", query.join(" "))
        }
        Control::Devices { .. } => "devices".to_owned(),
        Control::Transfer { device_id } => format!("transfer {device_id}"),
        Control::NowPlaying { .. } => "nowplaying".to_owned(),
        Control::Show => "show".to_owned(),
    };
    let printed = match single_instance::send(&verb) {
        Ok(single_instance::Reply::Ok) => return 0,
        Ok(single_instance::Reply::NowPlaying(snapshot)) => {
            if raw {
                format!("{snapshot}\n")
            } else {
                format!("{}\n", format_now_playing(&snapshot))
            }
        }
        Ok(single_instance::Reply::Devices(snapshot)) => {
            if raw {
                format!("{snapshot}\n")
            } else {
                format_devices(&snapshot)
            }
        }
        Err(error) => {
            eprintln!("Spotifrost is not running or does not support remote control: {error}");
            return 1;
        }
    };
    write_reply(&mut std::io::stdout().lock(), &printed)
}

/// Writes a command's reply to standard output. A reader that stops early,
/// such as `head`, closes the pipe; that ends the command quietly instead of
/// panicking as `println!` does.
fn write_reply(out: &mut impl std::io::Write, text: &str) -> i32 {
    match out.write_all(text.as_bytes()).and_then(|()| out.flush()) {
        Ok(()) => 0,
        Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => 0,
        Err(error) => {
            eprintln!("cannot print the reply: {error}");
            1
        }
    }
}

/// The desktop entry's name, which the window's app id has to match for the
/// desktop to find its icon: inside a Flatpak the entry is exported under the
/// sandbox's app id, which Flatpak always sets in `FLATPAK_ID`.
#[cfg(target_os = "linux")]
fn desktop_entry() -> String {
    fastframe_now_playing::desktop_entry("spotifrost")
}

/// The `nowplaying` snapshot as one human-readable line.
fn format_now_playing(snapshot: &str) -> String {
    let mut fields = snapshot.split('\t');
    let state = fields.next().unwrap_or_default();
    let title = fields.next().unwrap_or_default();
    let artists = fields.next().unwrap_or_default();
    let _album = fields.next();
    let position_ms: u32 = fields.next().and_then(|ms| ms.parse().ok()).unwrap_or(0);
    let duration_ms: u32 = fields.next().and_then(|ms| ms.parse().ok()).unwrap_or(0);
    let clock = |ms: u32| format!("{}:{:02}", ms / 60_000, ms % 60_000 / 1000);
    match state {
        "playing" | "paused" => {
            let mark = if state == "playing" { "▶" } else { "⏸" };
            format!(
                "{mark} {title} — {artists}  [{} / {}]",
                clock(position_ms),
                clock(duration_ms)
            )
        }
        _ => "Nothing playing".to_owned(),
    }
}

/// The `devices` snapshot as one line per device, the active one marked.
/// The id comes first because `spotifrost transfer` is what it is for.
fn format_devices(snapshot: &str) -> String {
    let Ok(devices) = serde_json::from_str::<Vec<serde_json::Value>>(snapshot) else {
        return String::new();
    };
    let field =
        |device: &serde_json::Value, key: &str| device[key].as_str().unwrap_or_default().to_owned();
    devices
        .iter()
        .map(|device| {
            format!(
                "{}{}\t{}\t{}\n",
                if device["active"].as_bool().unwrap_or(false) {
                    "* "
                } else {
                    "  "
                },
                field(device, "id"),
                field(device, "name"),
                field(device, "kind"),
            )
        })
        .collect()
}

pub(crate) fn run() -> eframe::Result<()> {
    // Before any output is listed or opened.
    spotifrost::sink::name_in_mixers();

    let cli =
        Cli::from_arg_matches(&Cli::command().get_matches()).unwrap_or_else(|error| error.exit());
    // A control launch is a client, not a second app: talk to the running
    // instance and exit before touching the log file it is writing to.
    if let Some(control) = cli.control {
        std::process::exit(run_control(control));
    }
    // A link is read before anything starts: one that is not a Spotify
    // link ends the launch here rather than reaching the running instance.
    let link = cli
        .link
        .as_deref()
        .map(|text| match spotifrost::link::parse(text) {
            Some(uri) => uri,
            None => {
                eprintln!("not a Spotify link: {text}");
                std::process::exit(2);
            }
        });
    // The application (audio engine, Web API, MPRIS, tray) outlives any
    // window. Closing to the tray destroys the window and the shell below
    // creates a new one when the tray or MPRIS asks for it. Plain window
    // lifecycle, portable across desktops.
    let waker = backend::Waker::default();

    let guarded = true;
    let instance = if guarded {
        let hidden = cli.start_hidden && link.is_none();
        match single_instance::acquire(&waker, link.as_deref(), hidden) {
            single_instance::Outcome::Only(guard) => Some(guard),
            single_instance::Outcome::Surfaced if hidden => {
                log::info!("Spotifrost is already running");
                return Ok(());
            }
            single_instance::Outcome::Surfaced => {
                log::info!("Spotifrost is already running; asked it to show its window");
                return Ok(());
            }
        }
    } else {
        None
    };
    let default_filter = if cli.verbose {
        "info,librespot=info,spotifrost=debug"
    } else {
        // Which system face draws each script the interface font lacks,
        // one line per script at startup, for reports of odd-looking text.
        "warn,spotifrost=info,fastframe_fonts=info"
    };
    let demo_dirs: Option<paths::AppDirs> = None;
    // Now that this is the running copy: a portable copy's own folder, made
    // the first time from the per-user one (#703), or the per-user one.
    let folders = match demo_dirs {
        Some(dirs) => paths::Found::given(dirs),
        None => paths::AppDirs::discover(),
    };
    let dirs = folders.dirs.clone();
    let window_file = folders.window_file();
    let dirs_ready = dirs.ensure();
    // Launched from a desktop, stderr goes nowhere; keep the run's log where
    // a bug report can find it, and a line per panic in the panic log (with
    // any link in its message removed: a URL can carry a token).
    if let Err(error) = fastframe_log::Logging::new("spotifrost", env!("CARGO_PKG_VERSION"))
        .filter(default_filter)
        .file(dirs.log_file())
        .panic_log(dirs.panic_log())
        .panic_message(fastframe_log::PanicMessage::Redacted(
            fastframe_log::redact::links,
        ))
        .init()
    {
        eprintln!("not logging: {error}");
    }
    folders.log();
    if let Err(error) = dirs_ready {
        log::warn!("unable to create the application directories: {error}");
    }
    // A Cargo run has no app bundle to supply its Dock icon. Use the same
    // artwork as the packaged app; Fastframe preserves a bundle's own icon.
    #[cfg(target_os = "macos")]
    if let Err(error) = fastframe_dock::set_icon(include_bytes!("../packaging/macos/icon-1024.png"))
    {
        log::warn!("unable to set the Dock icon: {error}");
    }
    let settings = settings::Settings::load(&dirs.settings_file());
    fastframe_titlebar::init();
    fastframe_titlebar::set_enabled(false);
    spotifrost::emoji::install(false);
    // macOS delivers links as Apple Events; install before the event loop.
    #[cfg(target_os = "macos")]
    if let Some(guard) = &instance {
        spotifrost::mac_links::install(guard.commands(), waker.clone());
    }

    // A capture run is a throwaway process next to the real one: no tray
    // icon of its own, and no second MPRIS service to fight over media keys.
    #[allow(unused_mut)]
    let mut options = app::AppOptions::default();
    #[cfg(windows)]
    let desktop_surfaces = options.media_controls;
    #[allow(unused_mut)]
    let mut app = app::App::new(&waker, dirs, settings, options);
    if let Some(guard) = &instance {
        app.set_remote_control(guard);
    }
    let link_given = link.is_some();
    if let Some(uri) = link {
        app.open_link(uri);
    }
    let start_hidden = start_hidden(cli.start_hidden, link_given);
    // From here a polite SIGTERM quits like Quit, stopping Soloist and
    // saving the state, rather than ending the process where it stands.
    spotifrost::terminate::listen(waker.clone());
    let shell = fastframe_shell::Shell::new(app, &waker).start_hidden(start_hidden);
    // The menu-bar item answers only while AppKit's loop runs. Elsewhere
    // nothing needs the main thread between headless ticks, so the shell
    // sleeps until the waker wakes it.
    #[cfg(target_os = "macos")]
    let shell = shell.idle(fastframe_tray::idle);
    shell.run(|lease| {
        #[cfg(windows)]
        let creator_waker = waker.clone();
        // The notch player is clicked while there is no window to repaint,
        // so it wakes the app's loop itself.
        #[cfg(target_os = "macos")]
        let notch_waker = waker.clone();
        let geometry = lease.peek(main_geometry);
        let locale = lease.peek(|app| app.locale);
        let options = native_options(false, None, geometry);
        let options = profile_options(options, window_file.clone());
        #[cfg(windows)]
        let thumbbar_enabled = desktop_surfaces && options.viewport.taskbar != Some(false);
        #[cfg(target_os = "linux")]
        let hide_from_taskbar = options.viewport.taskbar == Some(false);
        // Without the wgpu feature OpenGL is the only renderer, so there is
        // one try and each of these is asked for once. When it cannot draw,
        // fastframe-graphics says why in a native dialog rather than letting
        // the app quit without a word.
        let mut options = Some(options);
        let mut creator: Option<eframe::AppCreator<'_>> = Some(Box::new(move |cc| {
            if let Some(gl) = &cc.gl {
                use eframe::glow::HasContext;
                // eframe has made this window's GL context current
                // before calling the app creator. These identify the
                // renderer actually selected, which may differ from
                // the listed GPU.
                unsafe {
                    log::info!(
                        "OpenGL renderer: {}; vendor: {}; version: {}",
                        gl.get_parameter_string(eframe::glow::RENDERER),
                        gl.get_parameter_string(eframe::glow::VENDOR),
                        gl.get_parameter_string(eframe::glow::VERSION)
                    );
                }
            }
            let mut app = lease.take(&cc.egui_ctx);
            // Built once per window, before the first frame; the
            // handler wakes the loop so a menu pick is not held until
            // the next repaint.
            #[cfg(target_os = "macos")]
            {
                spotifrost::mac_touchbar_crash_guard::install();
                spotifrost::mac_menu::init();
                let ctx = cc.egui_ctx.clone();
                spotifrost::mac_menu::set_waker(move || ctx.request_repaint());

                spotifrost::notch::init();
                let ctx_notch = cc.egui_ctx.clone();
                let notch_waker = notch_waker.clone();
                spotifrost::notch::set_waker(move || {
                    notch_waker.wake();
                    ctx_notch.request_repaint();
                });
            }
            {
                use raw_window_handle::HasDisplayHandle;
                if let Ok(display) = cc.display_handle() {
                    app.window_level_supported =
                        spotifrost::window::supports_window_level(display.as_raw());
                    app.taskbar_hiding_supported =
                        spotifrost::window::supports_hiding_from_taskbar(display.as_raw());
                }
            }
            // winit hides a taskbar button on Windows only; X11 is
            // asked here, while the window is still unmapped.
            #[cfg(target_os = "linux")]
            if hide_from_taskbar {
                use raw_window_handle::HasWindowHandle;
                if let Ok(handle) = cc.window_handle() {
                    spotifrost::window::skip_x11_taskbar(handle.as_raw());
                }
            }
            app.attach(&cc.egui_ctx);
            #[cfg(windows)]
            let thumbbar = {
                use raw_window_handle::{HasWindowHandle, RawWindowHandle};
                let mut toolbar = spotifrost::thumbbar::ThumbBar::new();
                if thumbbar_enabled
                    && let Ok(handle) = cc.window_handle()
                    && let RawWindowHandle::Win32(window) = handle.as_raw()
                {
                    let wake = creator_waker.clone();
                    // The shell and toolbar share this window's thread
                    // and lifetime; the toolbar is detached on shell
                    // drop.
                    unsafe { toolbar.attach(window.hwnd.get(), move || wake.wake()) };
                }
                toolbar
            };
            Ok(Box::new(Shell {
                app,
                recovery_checked: false,
                #[cfg(windows)]
                thumbbar,
                stalls: fastframe_shell::StallWatch::default(),
            }))
        }));
        fastframe_graphics::Graphics::new("Spotifrost")
            .help(spotifrost::window::DOWNLOAD_PAGE)
            .dialog(move |trouble| spotifrost::window::no_window_dialog(locale, trouble))
            .run_native(
                || options.take().expect("one try opens the window"),
                || creator.take().expect("one try opens the window"),
            )
            .inspect_err(|error| log::error!("Native window failed: {error}"))
    })
}

/// Whether the first window waits in the tray: asked for (a start at login
/// in the tray), with no link to show.
fn start_hidden(requested: bool, link: bool) -> bool {
    requested && !link
}

fn main_geometry(app: &app::App) -> Geometry {
    if app.offline {
        Geometry::default()
    } else {
        app.window_geometry
    }
}

const fn main_window_decorated(custom_titlebar: bool) -> bool {
    !custom_titlebar
}

/// The window to open, at the demo's fixed `inner_size` or where `geometry`
/// remembers.
fn native_options(
    fullscreen: bool,
    inner_size: Option<[f32; 2]>,
    geometry: Geometry,
) -> eframe::NativeOptions {
    let persist_window = true;
    let persistence_path: Option<std::path::PathBuf> = None;
    #[cfg(target_os = "linux")]
    // Keep the native profile path even when Flatpak supplies its app ID.
    let persistence_path = persistence_path.or_else(paths::installed_window_file);
    #[cfg(target_os = "linux")]
    let app_id = desktop_entry();
    #[cfg(not(target_os = "linux"))]
    let app_id = "spotifrost";
    let icon = if cfg!(target_os = "macos") {
        // macOS takes the dock icon from the bundle's .icns, which is the
        // 1024px drawing with the platform's rounding. Setting a window
        // icon there replaces it with this flat 128px square.
        egui::IconData::default()
    } else {
        app_icon()
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("Spotifrost")
        .with_app_id(app_id)
        .with_taskbar(true)
        .with_icon(icon);
    let viewport = {
        let size = inner_size.unwrap_or([1240.0, 800.0]);
        let viewport = viewport
            // macOS: no title bar strip above the app. The content runs to
            // the top edge and the traffic lights float over it, the way
            // every other music player on the platform looks; the interface
            // leaves room for them with `theme::titlebar_inset`.
            .with_fullsize_content_view(true)
            .with_titlebar_shown(false)
            .with_title_shown(false)
            // macOS: see-through, for the glass behind the interface
            // (`spotifrost::mac_glass`).
            .with_transparent(cfg!(target_os = "macos"))
            // Windows has no equivalent to macOS's floating traffic lights.
            // Removing its decorations lets the app surface fill the window.
            .with_decorations(main_window_decorated(fastframe_titlebar::enabled()))
            .with_inner_size(size)
            .with_min_inner_size(inner_size.unwrap_or(spotifrost::window::MAIN_MIN_SIZE))
            .with_fullscreen(fullscreen);
        if inner_size.is_some() {
            viewport.with_max_inner_size(size)
        } else {
            // The size, position and maximized state it was left in,
            // before the window exists. eframe's own saved window, where
            // there is one, still wins over this.
            geometry.apply(viewport)
        }
    };
    eframe::NativeOptions {
        viewport,
        persist_window,
        persistence_path,
        ..Default::default()
    }
}

/// Points the main window at `window_file`, which a portable copy keeps in
/// its own folder.
fn profile_options(
    mut options: eframe::NativeOptions,
    window_file: Option<std::path::PathBuf>,
) -> eframe::NativeOptions {
    if options.persist_window {
        options.persistence_path = window_file;
    }
    options
}

#[cfg(test)]
mod native_window_tests {
    use super::*;

    /// A reader that closes the pipe early ends the command quietly.
    #[test]
    fn a_closed_pipe_ends_a_reply_quietly() {
        struct Closed;
        impl std::io::Write for Closed {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        assert_eq!(write_reply(&mut Closed, "playing\n"), 0);
        let mut buffer = Vec::new();
        assert_eq!(write_reply(&mut buffer, "playing\n"), 0);
        assert_eq!(buffer, b"playing\n");
    }

    #[test]
    fn only_windows_removes_the_native_frame() {
        assert!(!main_window_decorated(true));
        assert!(main_window_decorated(false));
    }
}

/// The eframe adapter around the long-lived [`app::App`]: delegates frames
/// and, when the window goes away, hands the state back for the next window.
struct Shell {
    app: fastframe_shell::Held<app::App>,
    /// Whether this window's first frame checked that a monitor shows it.
    recovery_checked: bool,
    #[cfg(windows)]
    thumbbar: spotifrost::thumbbar::ThumbBar,
    /// Logs a warning when an update freezes the window.
    stalls: fastframe_shell::StallWatch,
}

/// The view egui draws this window into.
#[cfg(target_os = "macos")]
fn ns_view(frame: &eframe::Frame) -> Option<*mut std::ffi::c_void> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let handle = frame.winit_window()?.window_handle().ok()?;
    match handle.as_raw() {
        RawWindowHandle::AppKit(appkit) => Some(appkit.ns_view.as_ptr()),
        _ => None,
    }
}

/// Puts the glass behind this window (`spotifrost::mac_glass`).
#[cfg(target_os = "macos")]
fn install_glass(frame: &eframe::Frame) {
    if let Some(view) = ns_view(frame) {
        spotifrost::mac_glass::install(view);
    }
}

impl Shell {
    /// One pass of the window's logic, run under the stall watch.
    fn run_logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // A window restored where no monitor shows it any more (a display
        // unplugged since) comes back onto one.
        if !std::mem::replace(&mut self.recovery_checked, true) {
            fastframe_shell::window::recover_offscreen(ctx, frame);
            #[cfg(target_os = "macos")]
            install_glass(frame);
        }
        let app = &mut *self.app;
        #[cfg(target_os = "macos")]
        for command in spotifrost::mac_menu::drain_commands() {
            use spotifrost::mac_menu::MenuCommand;
            use spotifrost::model::{Action, Dialog, Page};
            let action = match command {
                MenuCommand::PlayPause => Action::TogglePlay,
                MenuCommand::Next => Action::Next,
                MenuCommand::Previous => Action::Previous,
                MenuCommand::SeekForward => Action::SeekBy(10_000),
                MenuCommand::SeekBackward => Action::SeekBy(-10_000),
                MenuCommand::ToggleShuffle => Action::ToggleShuffle,
                MenuCommand::CycleRepeat => Action::CycleRepeat,
                MenuCommand::VolumeUp => Action::VolumeBy(5),
                MenuCommand::VolumeDown => Action::VolumeBy(-5),
                MenuCommand::ToggleMute => Action::ToggleMute,
                MenuCommand::Home => Action::Open(Page::Home),
                MenuCommand::Search => Action::FocusSearch,
                MenuCommand::LikedSongs => Action::Open(Page::LikedSongs),
                MenuCommand::Sidebar => Action::ToggleSidebar,
                MenuCommand::Queue => Action::ToggleQueuePanel,
                MenuCommand::Settings => Action::Open(Page::Settings),
                MenuCommand::Shortcuts => Action::ShowDialog(Dialog::Shortcuts),
                MenuCommand::Back => Action::Back,
                MenuCommand::Forward => Action::Forward,
                MenuCommand::OpenRepo => {
                    ctx.open_url(egui::OpenUrl::new_tab(
                        "https://github.com/spham2mail-star/spotifrost",
                    ));
                    continue;
                }
                // Editing goes through egui, which owns the text field
                // and the clipboard.
                MenuCommand::Cut => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestCut);
                    continue;
                }
                MenuCommand::Copy => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestCopy);
                    continue;
                }
                MenuCommand::Paste => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                    continue;
                }
                MenuCommand::SelectAll => {
                    ctx.input_mut(|input| {
                        input.events.push(egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::COMMAND,
                        });
                    });
                    continue;
                }
            };
            app.actions.push(action);
        }
        #[cfg(windows)]
        for command in self.thumbbar.drain_commands() {
            if let Some(action) = command.action(&app.thumb_state(false)) {
                app.actions.push(action);
            }
        }
        app.background_frame(ctx);
        #[cfg(windows)]
        self.thumbbar
            .sync(app.thumb_state(ctx.system_theme() != Some(egui::Theme::Light)));
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let app = &mut *self.app;
        app.frame_ui(ui);
        #[cfg(windows)]
        self.thumbbar
            .sync(app.thumb_state(ui.ctx().system_theme() != Some(egui::Theme::Light)));
    }
}

impl eframe::App for Shell {
    /// egui's memory stays out of every profile: its options carry a zoom,
    /// which would scale the remembered window geometry the new window opens
    /// with.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Taken out for the pass, so the logic can borrow the whole shell.
        let mut stalls = std::mem::take(&mut self.stalls);
        stalls.logic(|| self.run_logic(ctx, frame));
        self.stalls = stalls;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut stalls = std::mem::take(&mut self.stalls);
        stalls.ui(|| self.draw(ui));
        self.stalls = stalls;
    }

    /// On macOS the glass shows through wherever the interface leaves it;
    /// elsewhere the window paints itself over eframe's own ground.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        if cfg!(target_os = "macos") {
            [0.0; 4]
        } else {
            egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let app = &mut *self.app;
        app.save_state();
        app.flush_files();
    }
}

/// The toolbar goes before the window; the app then returns to the shell
/// with the `Held` field.
#[cfg(windows)]
impl Drop for Shell {
    fn drop(&mut self) {
        self.thumbbar.detach();
    }
}

/// The window icon, from the shared runtime drawing.
fn app_icon() -> egui::IconData {
    const SIZE: usize = 128;
    egui::IconData {
        rgba: util::app_icon_rgba(SIZE),
        width: SIZE as u32,
        height: SIZE as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A link on the command line is a link, and a control verb is still a
    /// verb: the two do not get in each other's way.
    #[test]
    fn a_link_and_a_verb_are_told_apart() {
        // #given / #when / #then
        let launch = Cli::try_parse_from(["spotifrost", "spotify:track:4uLU6hMCjMI75M1A2tKUQC"])
            .expect("a link parses");
        assert_eq!(
            launch.link.as_deref(),
            Some("spotify:track:4uLU6hMCjMI75M1A2tKUQC")
        );
        assert!(launch.control.is_none());

        let launch = Cli::try_parse_from([
            "spotifrost",
            "https://open.spotify.com/album/1DFixLWuPkv3KT3TnV35m3?si=x",
            "--verbose",
        ])
        .expect("a web address parses");
        assert!(launch.link.is_some());
        assert!(launch.verbose);

        let verb = Cli::try_parse_from(["spotifrost", "next"]).expect("a verb parses");
        assert!(matches!(verb.control, Some(Control::Next)));
        assert!(verb.link.is_none());

        let bare = Cli::try_parse_from(["spotifrost"]).expect("a plain launch parses");
        assert!(bare.link.is_none() && bare.control.is_none());
    }
}
