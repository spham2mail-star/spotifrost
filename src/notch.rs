//! Cross-platform notch widget interface.

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotchCommand {
    PlayPause,
    Next,
    Previous,
    ShowWindow,
    ToggleSaved(String),
    Seek(u32),
    ToggleShuffle,
    CycleRepeat,
}

impl NotchCommand {
    pub fn action(&self) -> crate::model::Action {
        match self {
            Self::PlayPause => crate::model::Action::TogglePlay,
            Self::Next => crate::model::Action::Next,
            Self::Previous => crate::model::Action::Previous,
            Self::ShowWindow => crate::model::Action::ShowWindow,
            Self::ToggleSaved(uri) => crate::model::Action::ToggleSaved(uri.clone()),
            Self::Seek(pos) => crate::model::Action::Seek(*pos),
            Self::ToggleShuffle => crate::model::Action::ToggleShuffle,
            Self::CycleRepeat => crate::model::Action::CycleRepeat,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NotchTrackInfo {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u32,
    pub position_ms: u32,
    pub art_path: Option<std::path::PathBuf>,
    pub uri: String,
    pub saved: bool,
    pub accent: Option<[u8; 3]>,
    pub is_episode: bool,
    pub is_remote: bool,
    pub shuffle: bool,
    pub repeat: crate::player::RepeatMode,
}

use std::sync::Mutex;

static COMMANDS: Mutex<Vec<NotchCommand>> = Mutex::new(Vec::new());
static WAKER: Mutex<Option<Box<dyn Fn() + Send + Sync>>> = Mutex::new(None);

pub fn set_waker(wake: impl Fn() + Send + Sync + 'static) {
    if let Ok(mut w) = WAKER.lock() {
        *w = Some(Box::new(wake));
    }
}

pub fn wake() {
    if let Ok(w) = WAKER.lock()
        && let Some(wake) = w.as_ref()
    {
        wake();
    }
}

pub fn push_command(cmd: NotchCommand) {
    if let Ok(mut list) = COMMANDS.lock() {
        list.push(cmd);
    }
    wake();
}

pub fn drain_commands() -> Vec<NotchCommand> {
    if let Ok(mut list) = COMMANDS.lock() {
        std::mem::take(&mut *list)
    } else {
        Vec::new()
    }
}

#[cfg(target_os = "macos")]
pub use crate::mac_notch::{init, is_active, sync_state};

#[cfg(not(target_os = "macos"))]
pub fn init() {}

#[cfg(not(target_os = "macos"))]
pub fn sync_state(_enabled: bool, _is_background: bool, _track: Option<&NotchTrackInfo>) {}

#[cfg(not(target_os = "macos"))]
pub fn is_active() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notch_command_queuing_and_draining() {
        push_command(NotchCommand::PlayPause);
        push_command(NotchCommand::Next);
        push_command(NotchCommand::Previous);
        push_command(NotchCommand::ShowWindow);

        let commands = drain_commands();
        assert_eq!(
            commands,
            vec![
                NotchCommand::PlayPause,
                NotchCommand::Next,
                NotchCommand::Previous,
                NotchCommand::ShowWindow,
            ]
        );

        let empty = drain_commands();
        assert!(empty.is_empty());
    }

    #[test]
    fn notch_track_info_equality() {
        let a = NotchTrackInfo {
            playing: true,
            title: "Test Track".into(),
            artist: "Test Artist".into(),
            album: "Test Album".into(),
            duration_ms: 180000,
            position_ms: 45000,
            art_path: None,
            uri: "spotify:track:test".into(),
            saved: false,
            accent: Some([30, 215, 96]),
            is_episode: false,
            is_remote: false,
            shuffle: false,
            repeat: crate::player::RepeatMode::Off,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn notch_command_actions() {
        use crate::model::Action;
        assert!(matches!(
            NotchCommand::PlayPause.action(),
            Action::TogglePlay
        ));
        assert!(matches!(NotchCommand::Next.action(), Action::Next));
        assert!(matches!(NotchCommand::Previous.action(), Action::Previous));
        assert!(matches!(
            NotchCommand::ShowWindow.action(),
            Action::ShowWindow
        ));
        assert!(matches!(
            NotchCommand::ToggleSaved("spotify:track:abc".into()).action(),
            Action::ToggleSaved(uri) if uri == "spotify:track:abc"
        ));
        assert!(matches!(
            NotchCommand::Seek(12345).action(),
            Action::Seek(12345)
        ));
        assert!(matches!(
            NotchCommand::ToggleShuffle.action(),
            Action::ToggleShuffle
        ));
        assert!(matches!(
            NotchCommand::CycleRepeat.action(),
            Action::CycleRepeat
        ));
    }
}
