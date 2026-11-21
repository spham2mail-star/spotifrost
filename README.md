<h1 align="center">Spotifrost</h1>

<p align="center">
  A minimal, glass-themed Spotify player for your Mac.<br>
  The same fast player, with the extras stripped out and a frosted-glass look that blends into macOS.
</p>

<p align="center">
  <a href="https://github.com/spham2mail-star/spotifrost/releases/latest"><strong>Download for Mac</strong></a> ·
  <a href="#set-up">Set up</a> ·
  <a href="#troubleshooting">Troubleshooting</a> ·
  <a href="#build-from-source">Build from source</a>
</p>

Spotifrost is a "vanilla plus" take on
[Spotifast](https://github.com/crmne/spotifast), the open-source Spotify
client by Carmine Paolino. It keeps the core player and removes what most
people never touch. It's a hobby project, built for fun, and it is not
affiliated with Spotify.

You need **Spotify Premium** to play music.

## What's different from Spotifast

**Added**

- **Glass theme.** The window is see-through frosted glass, tinted by your
  wallpaper. It uses your Mac's accent color and follows light and dark mode.
- **Notch player.** When you minimize or close the window while a song is
  loaded, it waits in the MacBook notch: the cover on the left and a
  play/pause mark on the right. Hover to slide out Previous, Play/Pause and
  Next. Click the cover to bring the window back. On a screen without a notch
  it shows as a small pill at the top center. You can turn it off in Settings.
- **Dock icon brings the window back** after you close it.

**Removed**

- Visualizers, MilkDrop, the equalizer and lyrics
- The mini player (Winamp/WMP mode) and its skins
- Preset themes and the theme picker (it's glass only)
- Friend activity and radio
- Chromecast and Sonos discovery
- The menu bar icon on macOS
- Album-art colors, proxy settings, zoom, frame-rate and title-bar options
- Settings for device name, autoplay, output buffer, audio cache and play
  history (autoplay stays on, and the built-in player keeps a small 512 MB
  audio cache)
- The update checker, demo mode and non-English languages

Lists are always compact, and the app uses the macOS system font.

## Install

1. Download `Spotifrost-macOS.zip` from the
   [latest release](https://github.com/spham2mail-star/spotifrost/releases/latest)
   and unzip it. It runs on Apple Silicon and Intel Macs.
2. Drag **Spotifrost.app** into your **Applications** folder.
3. The app isn't notarized by Apple, so the first time, **right-click it and
   choose Open**, then click **Open** again. If macOS still blocks it, go to
   **System Settings → Privacy & Security** and click **Open Anyway**.

If you used Spotifast on this Mac before, Spotifrost copies your settings and
sign-in on its first launch. Spotifast's own files are left alone.

Windows and Linux can [build from source](#build-from-source). The glass
theme and the notch player are Mac-only.

## Set up

Spotify asks every app like this one to use its own free registration (a
"Client ID"). It takes about two minutes:

1. Open the [Spotify developer dashboard](https://developer.spotify.com/dashboard)
   and sign in with your Spotify account.
2. Click **Create app**. Any name and description will do; nobody else sees it.
3. Under **Redirect URIs**, add exactly this address (the sign-in screen has a
   **Copy** button for it):

   ```
   http://127.0.0.1:8989/login
   ```

4. Tick **Web API**, accept the terms, and save.
5. Copy the **Client ID** from your app's page.
6. Paste it into Spotifrost's sign-in screen and click **Sign in with
   Spotify**. Approve access in your browser, then come back to the app.

Then choose how this computer plays music. **Spotify Soloist** (below) is
recommended. The other choice, **Use the built-in player**, asks you to approve
a second browser prompt, and your computer then appears in Spotify Connect as
**Spotifrost**.

Spotifrost never sees your Spotify password. Sign-ins are kept in your
system's keychain.

## Spotify Soloist

Soloist is Spotify's own player, which Spotifrost downloads from Spotify and
runs for you. It's the most reliable way to play music, because Spotify
sometimes refuses to unlock songs for the built-in player (see
[below](#spotify-refused-the-audio-key)). It also supports lossless audio.

1. After you sign in, choose **Set up Soloist** (or go to **Settings → Spotify
   Soloist** later).
2. Copy your personal key from
   [Spotify's Soloist page](https://developer.spotify.com/dashboard/soloist).
   It's different from your Client ID. Paste it and choose **Download Soloist**.
3. Open the Spotify app on your phone or another computer, on the same Wi-Fi
   and account, and play a song. From its list of devices, choose the
   **Spotifrost Soloist (…)** name shown in setup. You only do this once.
4. Choose **Start listening**.

If Soloist doesn't show up in the device list, try the Spotify desktop app
with a song playing, and make sure no VPN or guest Wi-Fi is hiding it. If you
used Spotifast with Soloist before, Spotifrost reuses your key automatically.

## Troubleshooting

### Spotify is being slow

Spotify limits how many requests each Client ID can make. When it asks the
app to wait, Spotifrost waits and tries again, so searches and your library
may load slowly for a bit. Using your own Client ID (see [Set up](#set-up))
keeps you out of anyone else's limits. If it keeps happening, give it a few
minutes.

### Spotify refused the audio key

Spotify sends songs locked, and the player has to ask for each song's key.
Sometimes Spotify says no to the built-in player. That comes from Spotify,
not from your computer, and it affects every player built on
[librespot](https://github.com/librespot-org/librespot). The fix is to use
[Spotify Soloist](#spotify-soloist), which Spotify unlocks songs for.

### Invalid redirect URI

The address in step 3 of [Set up](#set-up) is missing or differs by a
character. Add it exactly, save, and sign in again.

## Build from source

You need [Rust](https://rustup.rs) (the version in `rust-toolchain.toml` is
installed automatically) and, on macOS, the Xcode command-line tools
(`xcode-select --install`).

```sh
git clone https://github.com/spham2mail-star/spotifrost.git
cd spotifrost
cargo build --release
```

On macOS, wrap the binary into an app:

```sh
packaging/macos/bundle.sh target/release/spotifrost Spotifrost.app "$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"
```

On Linux you'll also need the usual audio, D-Bus and graphics development
packages (ALSA/PipeWire, D-Bus, X11/Wayland, OpenGL).

## Credits and license

Spotifrost is based on [Spotifast](https://github.com/crmne/spotifast) by
Carmine Paolino, which is built with Rust, [egui](https://github.com/emilk/egui)
and [librespot](https://github.com/librespot-org/librespot). Icons are from
[Lucide](https://lucide.dev) (see `assets/icons/LICENSE.txt`).

Released under the [MIT License](LICENSE). Spotify is a trademark of Spotify
AB. This project is not affiliated with, endorsed by, or sponsored by Spotify.
