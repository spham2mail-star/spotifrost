//! Interface text. English is the source language and the only one bundled:
//! the gettext calls stay, so a message keeps its context and plural forms.

pub use fastframe_i18n::{gettext, ngettext, pgettext};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum Locale {
    #[default]
    #[value(name = "en", alias = "en-US")]
    English,
}

impl fastframe_i18n::Locale for Locale {
    fn catalog(self) -> Option<&'static dyn fastframe_i18n::Translator> {
        // English is the source language: there is no catalog to read.
        None
    }
}

impl Locale {
    /// The tag this locale is named by on the command line.
    pub fn tag(self) -> &'static str {
        "en"
    }

    /// The locale a stored or typed tag names exactly.
    pub fn from_tag(tag: &str) -> Option<Self> {
        <Self as clap::ValueEnum>::from_str(tag, true).ok()
    }

    /// The language's name in that language.
    pub fn native_name(self) -> &'static str {
        "English"
    }

    /// The interface is in English only.
    pub fn from_system() -> Self {
        Self::English
    }

    pub fn liked_song_count(self, count: u32) -> String {
        ngettext(
            self,
            // Translators: Keep {count} exactly as written. It becomes the number of liked songs.
            "Playlist • {count} song",
            "Playlist • {count} songs",
            count,
        )
        .replace("{count}", &count.to_string())
    }

    pub fn song_count(self, count: u32) -> String {
        ngettext(
            self,
            // Translators: Keep {count} exactly as written. It becomes a number of songs.
            "{count} song",
            "{count} songs",
            count,
        )
        .replace("{count}", &count.to_string())
    }

    pub fn playlist_count(self, count: u32) -> String {
        ngettext(
            self,
            // Translators: Keep {count} exactly as written. It becomes a number of playlists.
            "{count} playlist",
            "{count} playlists",
            count,
        )
        .replace("{count}", &count.to_string())
    }

    pub fn folder_playlist_count(self, count: u32) -> String {
        ngettext(
            self,
            // Translators: Keep {count} exactly as written. It becomes the number of playlists a folder holds.
            "Folder • {count} playlist",
            "Folder • {count} playlists",
            count,
        )
        .replace("{count}", &count.to_string())
    }

    pub fn folder_state_label(self, name: &str, collapsed: bool) -> String {
        // Translators: Keep {name} exactly as written. It becomes the folder name.
        let label = if collapsed {
            gettext(self, "{name}, folder, collapsed")
        } else {
            gettext(self, "{name}, folder, expanded")
        };
        label.replace("{name}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_are_the_english_source() {
        assert_eq!(gettext(Locale::English, "Home"), "Home");
        assert_eq!(Locale::default(), Locale::English);
        assert_eq!(Locale::from_system(), Locale::English);
        assert_eq!(Locale::from_tag("en-US"), Some(Locale::English));
    }
}
