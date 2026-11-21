//! Sample data for screenshots and headless rendering tests.
//!
//! Nothing here talks to Spotify: the backend is switched offline, the
//! session is marked connected, and every page is filled with plausible
//! content. Cover art comes from a public placeholder service so the artwork
//! pipeline (download, disk cache, accent colour) is exercised too.

use std::time::Instant;

use jiff::{SignedDuration, Timestamp};

use crate::api::models::{
    Album, Artist, ArtistRef, Context, Copyright, Device, Episode, Followers, Image, Owner,
    Page as ApiPage, PlayHistory, PlayableItem, PlaybackState, Playlist, PlaylistItem, Queue,
    ResumePoint, SavedAlbum, SavedEpisode, SavedShow, SavedTrack, SearchResults, Show, Track,
    TrackCount, User,
};
use crate::app::{App, RemoteSnapshot};
use crate::backend::AuthStatus;
use crate::model::*;

fn image(seed: u32) -> Vec<Image> {
    vec![
        Image {
            url: format!("https://picsum.photos/seed/spotifrost{seed}/640/640"),
            width: Some(640),
            height: Some(640),
        },
        Image {
            url: format!("https://picsum.photos/seed/spotifrost{seed}/300/300"),
            width: Some(300),
            height: Some(300),
        },
        Image {
            url: format!("https://picsum.photos/seed/spotifrost{seed}/64/64"),
            width: Some(64),
            height: Some(64),
        },
    ]
}

const ARTISTS: &[&str] = &[
    "Bonobo",
    "Khruangbin",
    "Nils Frahm",
    "Little Simz",
    "Floating Points",
    "Jon Hopkins",
    "Sault",
    "Four Tet",
];

const ALBUMS: &[(&str, usize, &str)] = &[
    ("Fragments", 0, "2022"),
    ("Mordechai", 1, "2020"),
    ("All Melody", 2, "2018"),
    ("Sometimes I Might Be Introvert", 3, "2021"),
    ("Promises", 4, "2021"),
    ("Immunity", 5, "2013"),
    ("Untitled (Black Is)", 6, "2020"),
    ("There Is Love in You", 7, "2010"),
];

const TRACKS: &[&str] = &[
    "Rosewood",
    "Otomo",
    "Shadows",
    "Tides",
    "Elysian",
    "Closer",
    "Counterpart",
    "Sapien",
    "From You",
    "Day by Day",
    "Age of Phase",
    "Polyghost",
    "Time Moves Slow",
    "August 10",
    "So Rare",
    "Fugue",
    "Encores",
    "Sunlight",
    "My Friend the Forest",
    "Kaleidoscope",
];

fn demo_added_at(index: usize, now: Timestamp) -> String {
    let age = match index {
        0 => SignedDuration::from_secs(30),
        1 => SignedDuration::from_mins(5),
        2 => SignedDuration::from_hours(3),
        3 => SignedDuration::from_hours(2 * 24),
        4 => SignedDuration::from_hours(2 * 7 * 24),
        _ => SignedDuration::from_hours((35 + index as i64) * 24),
    };
    (now - age).to_string()
}

const PLAYLISTS: &[&str] = &[
    "Discover Weekly",
    "Late night focus",
    "Sunday morning",
    "Running 2026",
    "Release Radar",
    "Berlin nights",
    "Dinner party",
    "Deep work",
    "Road trip",
    "Kitchen jams",
];

const TOP_MIXES: &[(&str, &str)] = &[
    ("Downtempo Mix", "Bonobo, Khruangbin and Floating Points"),
    ("Nils Frahm Mix", "Nils Frahm, Jon Hopkins and Four Tet"),
    ("Electronic Mix", "Four Tet, Floating Points and Bonobo"),
    ("Chill Mix", "Khruangbin, Sault and Nils Frahm"),
    ("Hip Hop Mix", "Little Simz, Sault and Khruangbin"),
    ("2010s Mix", "Jon Hopkins, Bonobo and Four Tet"),
];

fn artist_ref(index: usize) -> ArtistRef {
    ArtistRef {
        id: Some(format!("art{index}")),
        name: ARTISTS[index % ARTISTS.len()].to_string(),
        uri: Some(format!("spotify:artist:art{index}")),
    }
}

fn artist(index: usize) -> Artist {
    Artist {
        id: format!("art{index}"),
        name: ARTISTS[index % ARTISTS.len()].to_string(),
        uri: format!("spotify:artist:art{index}"),
        images: image(100 + index as u32),
        genres: vec!["electronic".into(), "downtempo".into(), "ambient".into()],
        followers: Some(Followers {
            total: 1_284_930 + index as u64 * 10_431,
        }),
        popularity: Some(70),
        ..Artist::default()
    }
}

fn album(index: usize) -> Album {
    let (name, artist_index, year) = ALBUMS[index % ALBUMS.len()];
    Album {
        id: format!("alb{index}"),
        name: name.to_string(),
        uri: format!("spotify:album:alb{index}"),
        album_type: Some(if index % 4 == 3 {
            "single".into()
        } else {
            "album".into()
        }),
        total_tracks: Some(12),
        images: image(200 + index as u32),
        artists: vec![artist_ref(artist_index)],
        release_date: Some(match index {
            1 => year.to_string(), // known only to the year
            _ => format!("{year}-03-1{}", index % 9),
        }),
        label: Some("Ninja Tune".into()),
        copyrights: vec![Copyright {
            text: format!("{year} Ninja Tune"),
            kind: "C".into(),
        }],
        ..Album::default()
    }
}

fn track(index: usize) -> Track {
    let album_index = index % ALBUMS.len();
    let mut album = album(album_index);
    album.tracks = None;
    Track {
        id: Some(format!("trk{index}")),
        name: TRACKS[index % TRACKS.len()].to_string(),
        uri: format!("spotify:track:trk{index}"),
        duration_ms: 180_000 + (index as u32 * 37_000) % 240_000,
        explicit: index % 7 == 3,
        artists: vec![artist_ref(album_index)],
        album: Some(album),
        track_number: Some((index % 12) as u32 + 1),
        disc_number: Some(1),
        popularity: Some(60 + (index % 40) as u8),
        ..Track::default()
    }
}

fn playlist(index: usize) -> Playlist {
    let name = PLAYLISTS[index % PLAYLISTS.len()];
    let spotify_owned = matches!(name, "Discover Weekly" | "Release Radar");
    Playlist {
        id: format!("pl{index}"),
        name: name.to_string(),
        uri: format!("spotify:playlist:pl{index}"),
        description: Some(if spotify_owned {
            "Your weekly mixtape of fresh music. Enjoy new music and deep cuts picked for you. Updates every Monday.".into()
        } else {
            String::new()
        }),
        images: image(300 + index as u32),
        owner: Owner {
            id: Some(if spotify_owned {
                "spotify".into()
            } else {
                "demo".into()
            }),
            display_name: Some(if spotify_owned {
                "Spotify".into()
            } else {
                "Carmine".into()
            }),
            uri: None,
        },
        public: Some(index.is_multiple_of(2)),
        collaborative: false,
        snapshot_id: Some("snap".into()),
        tracks: Some(TrackCount {
            total: 30 + index as u32 * 7,
        }),
        followers: Some(Followers {
            total: if spotify_owned {
                1_845_680
            } else {
                12_345 + index as u64
            },
        }),
        ..Playlist::default()
    }
}

fn top_mix(index: usize) -> Playlist {
    let (name, artists) = TOP_MIXES[index % TOP_MIXES.len()];
    Playlist {
        id: format!("mix{index}"),
        name: name.to_string(),
        uri: format!("spotify:playlist:mix{index}"),
        description: Some(artists.to_string()),
        images: image(600 + index as u32),
        owner: Owner {
            id: Some("spotify".into()),
            display_name: Some("Spotify".into()),
            uri: None,
        },
        tracks: Some(TrackCount { total: 50 }),
        ..Playlist::default()
    }
}

fn episode(index: usize, show_index: usize) -> Episode {
    Episode {
        id: format!("ep{show_index}_{index}"),
        name: format!("Episode {}: {}", 120 - index, TRACKS[(index * 3) % TRACKS.len()]),
        uri: format!("spotify:episode:ep{show_index}_{index}"),
        duration_ms: 2_400_000 + (index as u32 * 311_000) % 2_000_000,
        description: "A conversation about how software gets made, why some tools feel fast, and what we can learn from the people who build them. Recorded live.".into(),
        images: image(400 + index as u32),
        release_date: Some(format!("2026-0{}-{:02}", 1 + index % 8, 1 + index % 27)),
        resume_point: Some(ResumePoint {
            fully_played: index.is_multiple_of(5),
            resume_position_ms: if index % 3 == 1 { 600_000 } else { 0 },
        }),
        show: Some(show(show_index)),
        ..Episode::default()
    }
}

fn show(index: usize) -> Show {
    Show {
        id: format!("sh{index}"),
        name: ["Rework", "Song Exploder", "The Rest Is History", "Darknet Diaries"][index % 4].into(),
        uri: format!("spotify:show:sh{index}"),
        publisher: ["37signals", "Hrishikesh Hirway", "Goalhanger", "Jack Rhysider"][index % 4].into(),
        description: "A podcast about a better way to work and run your business. Hosted by the founders of 37signals.".into(),
        images: image(500 + index as u32),
        total_episodes: Some(84),
        ..Show::default()
    }
}

fn page<T>(items: Vec<T>) -> ApiPage<T> {
    let total = items.len() as u32;
    ApiPage {
        items,
        total,
        limit: total,
        offset: 0,
        next: None,
    }
}

/// Populates the offline app with cards for screenshots and UI tests.
pub fn populate(app: &mut App) {
    app.backend.set_offline(true);
    app.offline = true;
    // A demo never reads the real login registration; it shows it off.
    app.autostart = Some(fastframe_autostart::State::Off);
    app.auth = AuthStatus::Connected {
        username: "demo".into(),
    };
    app.local_device_id = Some("local-demo".into());
    app.local_ready = true;
    app.local_playback = crate::backend::LocalPlayback::Ready {
        device_id: "local-demo".into(),
    };
    app.user = Some(User {
        id: "demo".into(),
        display_name: Some("Carmine".into()),
        images: image(1),
        product: Some("premium".into()),
        country: Some("DE".into()),
        uri: Some("spotify:user:demo".into()),
    });

    let playlists: Vec<Playlist> = (0..PLAYLISTS.len()).map(playlist).collect();
    for playlist in &playlists {
        app.saved.insert(playlist.uri.clone(), true);
    }
    app.library.playlists = Loadable::Loaded(playlists.clone());

    let tracks: Vec<Track> = (0..40).map(track).collect();
    for (index, track) in tracks.iter().enumerate() {
        app.saved.insert(track.uri.clone(), index % 3 == 0);
    }

    // Playlist page.
    let mut playlist_page = PlaylistPage {
        playlist: Loadable::Loaded(playlists[1].clone()),
        ..PlaylistPage::default()
    };
    let demo_now = Timestamp::now();
    playlist_page.items.absorb(
        0,
        page(
            tracks
                .iter()
                .enumerate()
                .map(|(index, track)| PlaylistItem {
                    // The first rows deliberately cover each relative-date
                    // unit; the rest remain absolute dates.
                    added_at: Some(demo_added_at(index, demo_now)),
                    is_local: false,
                    // Use multiple contributors so Added By and the byline render.
                    added_by: Some(crate::api::models::UserRef {
                        id: Some(if index % 3 == 1 { "kasia" } else { "sam" }.into()),
                    }),
                    item: Some(PlayableItem::Track(track.clone())),
                    track: None,
                })
                .collect(),
        ),
    );
    playlist_page.contributors.insert("kasia".into());
    playlist_page.contributors.insert("sam".into());
    app.playlist_pages.insert("pl1".into(), playlist_page);
    app.user_names.insert("kasia".into(), Some("Kasia".into()));
    app.user_names.insert("sam".into(), Some("Sam".into()));
    // The playlist owner's picture, as the profile view would answer.
    app.user_images.insert(
        "demo".into(),
        Some(image(1).first().map(|image| image.url.clone()).unwrap()),
    );
    let mut discover_page = PlaylistPage {
        playlist: Loadable::Loaded(playlists[0].clone()),
        ..PlaylistPage::default()
    };
    discover_page.items.absorb(
        0,
        page(
            tracks
                .iter()
                .rev()
                .take(30)
                .map(|track| PlaylistItem {
                    added_at: Some("2026-08-24T05:00:00Z".into()),
                    is_local: false,
                    added_by: None,
                    item: Some(PlayableItem::Track(track.clone())),
                    track: None,
                })
                .collect(),
        ),
    );
    app.playlist_pages.insert("pl0".into(), discover_page);
    // Two songs hidden in Discover Weekly, greyed out and skipped.
    app.hidden_songs.insert(
        playlists[0].uri.clone(),
        tracks
            .iter()
            .rev()
            .skip(2)
            .step_by(5)
            .take(2)
            .map(|track| track.uri.clone())
            .collect(),
    );
    app.hidden_songs_revision += 1;

    // Album page.
    let mut album_page = AlbumPage {
        album: Loadable::Loaded(album(0)),
        ..AlbumPage::default()
    };
    album_page
        .tracks
        .absorb(0, page(tracks.iter().take(12).cloned().collect()));
    app.album_pages.insert("alb0".into(), album_page);
    app.saved.insert("spotify:album:alb0".into(), true);

    // Artist page.
    let mut artist_page = ArtistPage {
        artist: Loadable::Loaded(artist(0)),
        top_tracks: Loadable::Loaded(tracks.iter().take(10).cloned().collect()),
        related: Loadable::Loaded((1..8).map(artist).collect()),
        ..ArtistPage::default()
    };
    let mut albums = PagedList::default();
    albums.absorb(0, page((0..8).map(album).collect()));
    artist_page
        .albums
        .insert(DiscographyFilter::All.groups().to_string(), albums);
    app.artist_pages.insert("art0".into(), artist_page);
    app.saved.insert("spotify:artist:art0".into(), true);

    // Play counts for the album's songs and the artist's popular ones.
    for (index, track) in tracks.iter().take(12).enumerate() {
        let plays = 1_348_772_019 / (index as u64 * 3 + 1) + index as u64 * 7_919;
        app.play_counts.insert(track.uri.clone(), plays);
    }
    for uri in [album(0).uri, artist(0).uri] {
        app.play_count_reads
            .insert(uri, crate::app::PlayCountRead::Answered);
    }

    // Show page.
    let mut show_page = ShowPage {
        show: Loadable::Loaded(show(0)),
        ..ShowPage::default()
    };
    show_page
        .episodes
        .absorb(0, page((0..15).map(|index| episode(index, 0)).collect()));
    app.show_pages.insert("sh0".into(), show_page);

    // Library.
    app.library.liked.absorb(
        0,
        page(
            tracks
                .iter()
                .filter(|track| app.saved.get(&track.uri) == Some(&true))
                .map(|track| SavedTrack {
                    added_at: Some("2026-06-12T08:30:00Z".into()),
                    track: track.clone(),
                })
                .collect(),
        ),
    );
    app.library.albums.absorb(
        0,
        page(
            (0..8)
                .map(|index| SavedAlbum {
                    added_at: None,
                    album: album(index),
                })
                .collect(),
        ),
    );
    app.library.artists.items = (0..8).map(artist).collect();
    app.library.artists.loaded_once = true;
    app.library.artists.complete = true;
    app.library.shows.absorb(
        0,
        page(
            (0..4)
                .map(|index| SavedShow {
                    added_at: None,
                    show: show(index),
                })
                .collect(),
        ),
    );
    app.library.episodes.absorb(
        0,
        page(
            (0..6)
                .map(|index| SavedEpisode {
                    added_at: None,
                    episode: episode(index, index % 4),
                })
                .collect(),
        ),
    );

    // Home.
    app.home.requested = true;
    app.home.loaded_at = Some(Instant::now());
    // Played from playlists, an album, Liked Songs, and on their own, as
    // Spotify's history reports them, for Home's quick-access tiles.
    let contexts = [
        Some("spotify:playlist:pl1".to_string()),
        Some("spotify:playlist:pl1".to_string()),
        None,
        Some("spotify:album:alb2".to_string()),
        Some("spotify:user:demo:collection".to_string()),
        Some("spotify:playlist:pl4".to_string()),
        None,
        Some("spotify:album:alb0".to_string()),
        Some("spotify:playlist:pl6".to_string()),
    ];
    app.home.recently_played = Loadable::Loaded(
        tracks
            .iter()
            .skip(5)
            .take(12)
            .enumerate()
            .map(|(index, track)| PlayHistory {
                track: track.clone(),
                played_at: Some("2026-08-26T21:12:00Z".into()),
                context: contexts.get(index).cloned().flatten().map(|uri| {
                    crate::api::models::Context {
                        kind: uri.split(':').nth(1).unwrap_or_default().to_string(),
                        uri,
                    }
                }),
            })
            .collect(),
    );
    // Recents tab (queue sidebar) – deduped, timestamped, paginated.
    let recents_now = Timestamp::now();
    app.recents.items = tracks
        .iter()
        .skip(2)
        .take(24)
        .enumerate()
        .map(|(index, track)| PlayHistory {
            track: track.clone(),
            played_at: Some(demo_added_at(index, recents_now)),
            context: None,
        })
        .collect();
    app.recents.loaded_once = true;
    app.recents.loading = false;
    app.recents.error = None;
    // Has more to load (before cursor of oldest item).
    let oldest = recents_now - SignedDuration::from_hours(48);
    // Cursor is unix millis; jiff Timestamp exposes seconds + nanos.
    let millis = oldest.as_second() * 1000 + i64::from(oldest.subsec_nanosecond() / 1_000_000);
    app.recents.after = Some(millis.to_string());
    app.recents.complete = false;
    // Podcast shelf: episodes to continue and new ones, dated from today so
    // the new ones stay new.
    let today = jiff::Zoned::now().date();
    app.home.podcasts = (0..4)
        .map(|show_index| {
            let episodes = (0..3)
                .map(|position| {
                    let mut episode = episode(show_index * 3 + position, show_index);
                    let age = (show_index * 4 + position * 7) as i64;
                    episode.release_date = today
                        .checked_sub(jiff::Span::new().days(age))
                        .ok()
                        .map(|date| date.to_string());
                    let started = match (show_index, position) {
                        (0, 0) => Some(1_200_000),
                        (1, 1) | (3, 1) => Some(900_000),
                        (1, 0) | (2, 0) => Some(0),
                        _ => None,
                    };
                    episode.resume_point = Some(ResumePoint {
                        fully_played: started.is_none(),
                        resume_position_ms: started.unwrap_or(0),
                    });
                    episode.show = None;
                    episode
                })
                .collect();
            (show(show_index), episodes)
        })
        .collect();
    app.home.podcasts_generation = app.home.generation;
    app.home.top_artists = Loadable::Loaded((0..8).map(artist).collect());
    let top_preview: Vec<_> = tracks.iter().skip(10).take(10).cloned().collect();
    let top_all: Vec<_> = tracks.iter().skip(10).cloned().collect();
    app.home.top_tracks = Loadable::Loaded(top_preview.clone());
    app.home.top_tracks_by_period = [
        Loadable::Loaded(top_preview.clone()),
        Loadable::Loaded(top_preview.clone()),
        Loadable::Loaded(top_preview),
    ];
    app.home.top_songs = Loadable::Loaded(top_all.clone());
    app.home.top_songs_by_period = [
        Loadable::Loaded(top_all.clone()),
        Loadable::Loaded(top_all.clone()),
        Loadable::Loaded(top_all),
    ];
    app.home.top_songs_complete = true;
    app.home.top_songs_by_period_complete = [true, true, true];
    app.home.recommendations = Loadable::Loaded(tracks.iter().skip(20).take(10).cloned().collect());
    app.home.made_for_you = Loadable::Loaded(playlists.iter().take(10).cloned().collect());
    app.home.top_mixes = Loadable::Loaded((0..TOP_MIXES.len()).map(top_mix).collect());
    // New releases: this autumn's records, newest first, and their songs.
    let releases: Vec<Album> = (0..8)
        .map(|index| {
            let mut release = album(index);
            release.release_date = Some(format!("2026-10-{:02}", 16 - index * 2));
            release
        })
        .collect();
    app.home.new_release_tracks = Loadable::Loaded(
        releases
            .iter()
            .enumerate()
            .flat_map(|(index, release)| {
                let songs = if release.is_single_release() { 1 } else { 3 };
                (0..songs).map(move |song| {
                    let mut track = track(index + song * ALBUMS.len());
                    track.album = Some(release.clone());
                    track.artists.clone_from(&release.artists);
                    track
                })
            })
            .collect(),
    );
    app.home.new_release_tracks_for = releases.iter().map(|release| release.uri.clone()).collect();
    app.home.new_releases = Loadable::Loaded(releases);

    // Search.
    app.search.query = "Bonobo".into();
    app.search.committed = "Bonobo".into();
    app.search.results = Loadable::Loaded(SearchResults {
        tracks: Some(page(tracks.iter().take(10).cloned().collect())),
        artists: Some(page((0..6).map(artist).collect())),
        albums: Some(page((0..6).map(album).collect())),
        playlists: Some(page(playlists.iter().take(6).cloned().collect())),
        shows: Some(page((0..4).map(show).collect())),
        episodes: Some(page((0..4).map(|index| episode(index, 1)).collect())),
    });
    app.settings.search_history = vec!["Khruangbin".into(), "ambient".into(), "Rework".into()];

    // Playback: a remote speaker is playing the second playlist.
    app.queue = Loadable::Loaded(Queue {
        currently_playing: Some(PlayableItem::Track(tracks[0].clone())),
        queue: tracks
            .iter()
            .skip(1)
            .take(12)
            .cloned()
            .map(PlayableItem::Track)
            .collect(),
    });
    app.devices = vec![
        Device {
            id: Some("local-demo".into()),
            name: "Spotifrost".into(),
            is_active: false,
            is_restricted: false,
            volume_percent: Some(70),
            supports_volume: Some(true),
            kind: "computer".into(),
        },
        Device {
            id: Some("remote1".into()),
            name: "Kitchen speaker".into(),
            is_active: true,
            is_restricted: false,
            volume_percent: Some(62),
            supports_volume: Some(true),
            kind: "speaker".into(),
        },
        Device {
            id: Some("remote2".into()),
            name: "Pixel 9".into(),
            is_active: false,
            is_restricted: false,
            volume_percent: Some(40),
            supports_volume: Some(true),
            kind: "smartphone".into(),
        },
    ];
    // Include an unsigned ZeroConf receiver in the device picker.
    app.receivers = vec![crate::zeroconf::Receiver {
        name: "House Spotify".into(),
        device_id: Some("house-speaker".into()),
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 42)),
        port: 5555,
        path: "/zc".into(),
    }];
    app.remote = Some(RemoteSnapshot {
        state: PlaybackState {
            device: Some(app.devices[1].clone()),
            repeat_state: "off".into(),
            shuffle_state: true,
            context: Some(Context {
                uri: playlists[1].uri.clone(),
                kind: "playlist".into(),
            }),
            timestamp: 0,
            progress_ms: Some(83_000),
            is_playing: true,
            item: Some(PlayableItem::Track(tracks[0].clone())),
            currently_playing_type: Some("track".into()),
        },
        received_at: Instant::now(),
    });
    for track in &tracks {
        if let Some(id) = &track.id {
            app.track_cache.insert(id.clone(), track.clone());
        }
    }
}

#[cfg(test)]
/// Audio outputs as fastframe-audio lists them on Windows: WASAPI endpoint
/// ids, shown by name.
fn demo_outputs() -> Vec<fastframe_audio::DeviceInfo> {
    [
        ("Speakers (Realtek High Definition Audio)", "4d1c"),
        ("VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)", "8a07"),
        (
            "VoiceMeeter Aux Input (VB-Audio VoiceMeeter AUX VAIO)",
            "8a08",
        ),
        ("LG TV (NVIDIA High Definition Audio)", "c3e2"),
    ]
    .into_iter()
    .enumerate()
    .map(|(index, (name, guid))| fastframe_audio::DeviceInfo {
        id: fastframe_audio::DeviceId::new(format!(
            "{{0.0.0.00000000}}.{{{guid}5f3e-9b1a-4c2d-8e7f-00000000000{index}}}"
        )),
        name: name.into(),
        is_default: index == 0,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppOptions;
    use crate::paths::AppDirs;
    use crate::settings::Settings;
    use std::sync::Arc;

    fn accessible_app(name: &str) -> (egui::Context, App) {
        let root =
            std::env::temp_dir().join(format!("spotifrost-a11y-{name}-{}", std::process::id()));
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            AppDirs {
                config: root.join("config"),
                state: root.join("state"),
                cache: root.join("cache"),
            },
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        // These frames never advance the clock or take pictures.
        app.reveal_theme_changes = false;
        (ctx, app)
    }

    #[test]
    fn change_cover_button_dispatches_the_native_picker_action() {
        let (ctx, mut app) = accessible_app("cover-button");
        app.dialog = Some(Dialog::EditPlaylist {
            id: "pl1".into(),
            name: "Test".into(),
            description: String::new(),
            public: Some(false),
            cover: Default::default(),
        });
        let _ = accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let button = accessible_node(&tree, "Change cover", egui::accesskit::Role::Button);
        let _ = accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(
                button,
                egui::accesskit::Action::Click,
                None,
            )],
        );
        let Some(Dialog::EditPlaylist { cover, .. }) = &app.dialog else {
            panic!()
        };
        assert!(cover.request.is_some());
        app.backend.shutdown();
    }

    fn accessible_frame(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
    ) -> egui::accesskit::TreeUpdate {
        accessible_frame_height(ctx, app, events, 800.0)
    }

    fn accessible_frame_height(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        height: f32,
    ) -> egui::accesskit::TreeUpdate {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, height),
                )),
                events,
                ..Default::default()
            },
            |ui| app.frame_ui(ui),
        );
        output.textures_delta.clear();
        output
            .platform_output
            .accesskit_update
            .expect("screen-reader tree")
    }

    #[test]
    fn artist_upcoming_card_opens_countdown_without_saving_or_playing() {
        let (ctx, mut app) = accessible_app("artist-upcoming");
        let uri = "spotify:prerelease:08LfxqlFRyEpd5h32mB8ho";
        let release_unix = Timestamp::now().as_second() + 14 * 86400;
        app.artist_prereleases.insert(
            artist(0).uri,
            crate::prerelease::UpcomingRelease {
                uri: uri.into(),
                name: "A new chapter".into(),
                images: image(1),
                release_unix,
            },
        );
        app.open(Page::Artist("art0".into()));
        let _ = accessible_frame_height(&ctx, &mut app, vec![], 1200.0);
        let tree = accessible_frame_height(&ctx, &mut app, vec![], 1200.0);
        let button = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node
                        .label()
                        .is_some_and(|label| label.starts_with("Release countdown: A new chapter"))
            })
            .expect("upcoming release is an accessible button")
            .0;
        let saved_before = app.saved.clone();
        let playing_before = app.playing_context_uri();
        let _ = accessible_frame_height(
            &ctx,
            &mut app,
            vec![accessible_action(
                button,
                egui::accesskit::Action::Click,
                None,
            )],
            1200.0,
        );
        assert_eq!(app.pending_spotify_link(), Some(uri));
        assert_eq!(
            app.saved, saved_before,
            "opening the countdown does not pre-save"
        );
        assert_eq!(
            app.playing_context_uri(),
            playing_before,
            "opening the countdown does not play"
        );
        app.backend.shutdown();
    }

    fn accessible_node(
        tree: &egui::accesskit::TreeUpdate,
        label: &str,
        role: egui::accesskit::Role,
    ) -> egui::accesskit::NodeId {
        tree.nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label) && node.role() == role)
            .unwrap_or_else(|| panic!("missing {label:?} with role {role:?}"))
            .0
    }

    fn accessible_action(
        target: egui::accesskit::NodeId,
        action: egui::accesskit::Action,
        data: Option<egui::accesskit::ActionData>,
    ) -> egui::Event {
        egui::Event::AccessKitActionRequest(egui::accesskit::ActionRequest {
            target_tree: egui::accesskit::TreeId::ROOT,
            target_node: target,
            action,
            data,
        })
    }

    fn keyboard(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    #[test]
    fn accessible_navigation_and_pause_work_without_a_pointer() {
        use egui::accesskit::{Action, Role};
        let (ctx, mut app) = accessible_app("navigate");
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let liked = accessible_node(&tree, "Liked Songs", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(liked, Action::Click, None)],
        );
        assert_eq!(app.page(), &Page::LikedSongs);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let pause = accessible_node(&tree, "Pause", Role::Button);
        assert!(app.believed_playing());
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(pause, Action::Focus, None)],
        );
        let tree = accessible_frame(
            &ctx,
            &mut app,
            vec![keyboard(egui::Key::Space, egui::Modifiers::NONE)],
        );
        assert!(
            !app.believed_playing(),
            "focused Space must pause once, without firing the global shortcut too"
        );
        assert_eq!(tree.focus, pause);
        app.backend.shutdown();
    }

    /// #990: the same keys scroll Liked Songs and an album page, the two
    /// other lists the issue names.
    #[test]
    fn page_navigation_scrolls_liked_songs_and_albums() {
        for page in [Page::LikedSongs, Page::Album("alb0".into())] {
            let (ctx, mut app) = accessible_app(&format!("page-navigation-{}", page.encode()));
            app.open(page.clone());
            let scrolled = egui::Id::new(("page-scrolled", page.encode()));
            let mut time = 0.0;
            let mut frame = |app: &mut App, events: Vec<egui::Event>| -> f32 {
                time += 1.0 / 60.0;
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(800.0, 600.0),
                        )),
                        time: Some(time),
                        events,
                        ..Default::default()
                    },
                    |ui| app.frame_ui(ui),
                );
                output.textures_delta.clear();
                ctx.data(|data| data.get_temp::<f32>(scrolled))
                    .unwrap_or(0.0)
            };
            for _ in 0..5 {
                frame(&mut app, vec![]);
            }
            frame(
                &mut app,
                vec![keyboard(egui::Key::PageDown, egui::Modifiers::NONE)],
            );
            let mut offset = 0.0;
            for _ in 0..30 {
                offset = frame(&mut app, vec![]);
            }
            assert!(offset > 0.0, "{page:?}: Page Down scrolls the page");
            frame(
                &mut app,
                vec![keyboard(egui::Key::Home, egui::Modifiers::NONE)],
            );
            for _ in 0..30 {
                offset = frame(&mut app, vec![]);
            }
            assert_eq!(offset, 0.0, "{page:?}: Home returns to the top");
            app.backend.shutdown();
        }
    }

    /// Drive By You through accessible button actions, verify owned rows remain
    /// visible, and confirm switching shelves preserves the remembered choice.
    #[test]
    fn by_you_sidebar_filter_can_be_toggled_and_survives_shelf_switches() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("by-you");
        app.settings.art_expanded = false;
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        accessible_node(&tree, "Discover Weekly", Role::Button);
        let by_you = accessible_node(&tree, "By You", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(by_you, AccessibleAction::Click, None)],
        );
        assert!(app.settings.library_by_you);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        accessible_node(&tree, "Liked Songs", Role::Button);
        accessible_node(&tree, "Late night focus", Role::Button);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Discover Weekly"))
        );
        let albums = accessible_node(&tree, "Albums", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(albums, AccessibleAction::Click, None)],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("By You"))
        );
        let playlists = accessible_node(&tree, "Playlists", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(playlists, AccessibleAction::Click, None)],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let by_you = accessible_node(&tree, "By You", Role::Button);
        assert!(app.settings.library_by_you);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(by_you, AccessibleAction::Click, None)],
        );
        assert!(!app.settings.library_by_you);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        accessible_node(&tree, "Discover Weekly", Role::Button);
        app.backend.shutdown();
    }

    #[test]
    fn translated_sidebar_keeps_keyboard_navigation_and_accessible_names() {
        use crate::i18n::{Locale, gettext};
        use clap::ValueEnum;
        use egui::accesskit::{Action as AccessibleAction, Role};
        for &locale in Locale::value_variants() {
            let (ctx, mut app) = accessible_app(&format!("translated-sidebar-{locale:?}"));
            app.locale = locale;
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let home = accessible_node(&tree, &gettext(locale, "Home"), Role::Button);
            let search = accessible_node(&tree, &gettext(locale, "Search"), Role::Button);
            accessible_node(&tree, &gettext(locale, "Create a playlist"), Role::Button);
            accessible_node(&tree, &gettext(locale, "Albums"), Role::Button);
            accessible_node(&tree, &gettext(locale, "Artists"), Role::Button);
            let liked = accessible_node(&tree, &gettext(locale, "Liked Songs"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(home, AccessibleAction::Focus, None)],
            );
            let tree = accessible_frame(
                &ctx,
                &mut app,
                vec![keyboard(egui::Key::Tab, egui::Modifiers::NONE)],
            );
            assert_eq!(tree.focus, search, "Tab order must survive translation");
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(liked, AccessibleAction::Focus, None)],
            );
            accessible_frame(
                &ctx,
                &mut app,
                vec![keyboard(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert_eq!(app.page(), &Page::LikedSongs);

            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let library_search =
                accessible_node(&tree, &gettext(locale, "Search Your Library"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(
                    library_search,
                    AccessibleAction::Click,
                    None,
                )],
            );
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            accessible_node(
                &tree,
                &gettext(locale, "Search in Your Library"),
                Role::TextInput,
            );
            app.library.filter = gettext(locale, "Liked Songs").to_uppercase();
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            accessible_node(&tree, &gettext(locale, "Liked Songs"), Role::Button);
            assert!(
                !tree
                    .nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some("Discover Weekly"))
            );
            app.backend.shutdown();
        }
    }

    #[test]
    fn translated_player_bar_labels_follow_control_state_and_empty_playback() {
        use crate::i18n::{Locale, gettext};
        use clap::ValueEnum;
        use egui::accesskit::{Action as AccessibleAction, Role};

        for &locale in Locale::value_variants() {
            let (ctx, mut app) = accessible_app(&format!("translated-player-state-{locale:?}"));
            app.locale = locale;
            accessible_frame(&ctx, &mut app, vec![]);
            for (source, next) in [
                ("Repeat", "Repeat one"),
                ("Repeat one", "Repeat off"),
                ("Repeat off", "Repeat"),
                ("Mute", "Unmute"),
                ("Unmute", "Mute"),
            ] {
                let tree = accessible_frame(&ctx, &mut app, vec![]);
                let button = accessible_node(&tree, &gettext(locale, source), Role::Button);
                accessible_frame(
                    &ctx,
                    &mut app,
                    vec![accessible_action(button, AccessibleAction::Click, None)],
                );
                let tree = accessible_frame(&ctx, &mut app, vec![]);
                accessible_node(&tree, &gettext(locale, next), Role::Button);
            }
            // A liked song's heart opens a checklist where unchecking Liked
            // Songs and choosing Done unlikes it; the heart then likes it at
            // once (#792).
            let click = |ctx: &egui::Context, app: &mut App, label: &'static str| {
                let tree = accessible_frame(ctx, app, vec![]);
                let button = accessible_node(&tree, &gettext(locale, label), Role::Button);
                accessible_frame(
                    ctx,
                    app,
                    vec![accessible_action(button, AccessibleAction::Click, None)],
                );
                // A menu's first frame only measures it.
                accessible_frame(ctx, app, vec![]);
            };
            let now = app.now_playing().unwrap().uri;
            assert_eq!(app.is_saved(&now), Some(true));
            click(&ctx, &mut app, "Add to playlist");
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            accessible_node(&tree, &gettext(locale, "Filter playlists"), Role::TextInput);
            assert_eq!(app.is_saved(&now), Some(true), "opening it keeps the like");
            let liked = accessible_node(&tree, &gettext(locale, "Liked Songs"), Role::CheckBox);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(liked, AccessibleAction::Click, None)],
            );
            assert_eq!(app.is_saved(&now), Some(true), "a check waits for Done");
            click(&ctx, &mut app, "Done");
            assert_eq!(app.is_saved(&now), Some(false));
            click(&ctx, &mut app, "Save to Liked Songs");
            assert_eq!(app.is_saved(&now), Some(true));
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            assert!(
                !tree.nodes.iter().any(|(_, node)| {
                    node.label() == Some(gettext(locale, "Filter playlists").as_ref())
                }),
                "liking a song does not open the menu"
            );

            app.remote = None;
            assert!(app.now_playing().is_none());
            let render = |app: &mut App, ui: &mut egui::Ui| crate::ui::player_bar::show(app, ui);
            view_frame(&ctx, &mut app, vec![], render);
            let painted = view_frame(&ctx, &mut app, vec![], render);
            for source in ["Nothing playing", "Pick a song, album, or playlist"] {
                assert!(
                    painted
                        .iter()
                        .any(|(text, _)| text == &gettext(locale, source)),
                    "missing translated empty playback label: {source}"
                );
            }
            app.backend.shutdown();
        }
    }

    #[test]
    fn translated_queue_controls_preserve_manual_and_context_rows() {
        use crate::i18n::{Locale, gettext};
        use clap::ValueEnum;
        use egui::accesskit::{Action as AccessibleAction, Role};

        for &locale in Locale::value_variants() {
            let (ctx, mut app) = accessible_app(&format!("translated-queue-{locale:?}"));
            app.locale = locale;
            app.show_queue_panel = true;
            app.local_ready = true;
            app.local.track = Some(crate::player::LocalTrack {
                uri: app.now_playing().unwrap().uri,
                ..Default::default()
            });
            app.local.playback = crate::player::Playback::Paused;
            let rows = app.queue.get().unwrap().queue.clone();
            app.manual_queue = rows[..2]
                .iter()
                .map(|item| item.uri().to_string())
                .collect();
            let saved_uris = app.queue_playlist_uris();
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let save = accessible_node(&tree, &gettext(locale, "Save as a playlist"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(save, AccessibleAction::Click, None)],
            );
            let Some(Dialog::CreatePlaylist {
                name,
                public,
                add_uris,
            }) = &app.dialog
            else {
                panic!("saving the translated queue must open the existing playlist dialog");
            };
            assert_eq!(name, &app.queue_playlist_name());
            assert!(!public);
            assert_eq!(add_uris, &saved_uris);
            app.dialog = None;
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let clear = accessible_node(&tree, &gettext(locale, "Clear queue"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(clear, AccessibleAction::Click, None)],
            );
            assert!(app.manual_queue.is_empty());
            assert_eq!(app.queue.get().unwrap().queue, rows[2..]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let recent = accessible_node(&tree, &gettext(locale, "Recent"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(recent, AccessibleAction::Click, None)],
            );
            assert_eq!(app.queue_tab, QueueTab::Recents);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let close = accessible_node(&tree, &gettext(locale, "Close"), Role::Button);
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(close, AccessibleAction::Click, None)],
            );
            assert!(!app.show_queue_panel);
            app.backend.shutdown();
        }
    }

    #[test]
    fn library_sort_menu_preserves_saved_order_and_keyboard_activation() {
        use crate::settings::{LibraryShelf, LibrarySort};
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("library-sort");
        app.settings.sidebar_order =
            vec!["spotify:playlist:pl4".into(), "spotify:playlist:pl1".into()];
        let saved = app.settings.sidebar_order.clone();
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let sort = accessible_node(&tree, "Local custom order", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(sort, AccessibleAction::Click, None)],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Recently added"))
        );
        let name = accessible_node(&tree, "Name", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(name, AccessibleAction::Focus, None)],
        );
        accessible_frame(
            &ctx,
            &mut app,
            vec![keyboard(egui::Key::Enter, egui::Modifiers::NONE)],
        );
        assert_eq!(
            app.settings.library_sort.get(&LibraryShelf::Playlists),
            Some(&LibrarySort::Name)
        );
        assert_eq!(app.settings.sidebar_order, saved);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let sort = accessible_node(&tree, "Name", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(sort, AccessibleAction::Click, None)],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let local = accessible_node(&tree, "Local custom order", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(local, AccessibleAction::Click, None)],
        );
        assert_eq!(
            app.settings.library_sort.get(&LibraryShelf::Playlists),
            Some(&LibrarySort::Local)
        );
        assert_eq!(app.settings.sidebar_order, saved);
        let restored: Settings =
            serde_json::from_str(&serde_json::to_string(&app.settings).unwrap()).unwrap();
        assert_eq!(restored.library_sort, app.settings.library_sort);
        assert_eq!(restored.sidebar_order, saved);
        app.backend.shutdown();
    }

    /// Home's podcast shelf uses the ordinary cards, leaves out audiobooks
    /// and shows no longer saved, and is not drawn at all when empty.
    #[test]
    fn the_home_podcast_shelf_shows_saved_podcasts_only() {
        let (ctx, mut app) = accessible_app("home-podcasts");
        let view = crate::ui::home::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let has = |painted: &[(String, egui::Rect)], wanted: &str| {
            painted.iter().any(|(text, _)| text == wanted)
        };
        assert!(has(&painted, "Your podcasts"));
        assert!(has(&painted, "20 min left • Rework"));
        assert!(has(&painted, "New • Song Exploder"));

        let rework = app.home.podcasts[0].0.uri.clone();
        app.audiobook_shows.insert(rework);
        let exploder = app.home.podcasts[1].0.uri.clone();
        app.saved.insert(exploder, false);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        assert!(!has(&painted, "20 min left • Rework"));
        assert!(!has(&painted, "New • Song Exploder"));
        assert!(has(&painted, "43 min left • Darknet Diaries"));

        app.home.podcasts.clear();
        let painted = view_frame(&ctx, &mut app, vec![], view);
        assert!(!has(&painted, "Your podcasts"));
        assert!(has(&painted, "Your top artists"));
        app.backend.shutdown();
    }

    /// The Home card's Play continues from the place that card shows, even
    /// when another loaded copy of the episode, here in saved episodes,
    /// still holds an older place.
    #[test]
    fn a_home_podcast_card_resumes_from_the_place_it_shows() {
        let (ctx, mut app) = accessible_app("home-podcast-resume");
        let card = app.home.podcasts[0].1[0].clone();
        assert_eq!(card.resume_ms(), Some(1_200_000));
        let mut stale = card.clone();
        stale.resume_point = Some(ResumePoint {
            fully_played: false,
            resume_position_ms: 600_000,
        });
        app.library.episodes.items.push(SavedEpisode {
            episode: stale,
            ..SavedEpisode::default()
        });
        let frame = |ctx: &egui::Context, app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 2200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::home::show(app, ui),
            );
            output.textures_delta.clear();
            output
        };
        let painted = view_frame(&ctx, &mut app, vec![], crate::ui::home::show);
        let left = painted
            .iter()
            .find(|(text, _)| text == "20 min left • Rework")
            .map(|(_, rect)| rect.center())
            .expect("the started episode's card");
        frame(&ctx, &mut app, vec![egui::Event::PointerMoved(left)]);
        let tree = frame(&ctx, &mut app, vec![egui::Event::PointerMoved(left)])
            .platform_output
            .accesskit_update
            .unwrap();
        let play = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some("Play"))
            .filter_map(|(_, node)| node.bounds())
            .map(|bounds| {
                egui::pos2(
                    ((bounds.x0 + bounds.x1) / 2.0) as f32,
                    ((bounds.y0 + bounds.y1) / 2.0) as f32,
                )
            })
            // The hovered card's button sits on its cover, just above the text.
            .filter(|center| center.y < left.y && left.distance(*center) < 250.0)
            .min_by(|a, b| left.distance(*a).total_cmp(&left.distance(*b)))
            .expect("the card's Play button on hover");
        app.actions.clear();
        frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(play),
                egui::Event::PointerButton {
                    pos: play,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: play,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        assert!(
            app.actions.iter().any(|action| matches!(
                action,
                Action::PlayEpisode { uri, resume_ms: Some(1_200_000) } if *uri == card.uri
            )),
            "{:?}",
            app.actions
        );
        app.backend.shutdown();
    }

    /// Spotify lists audiobooks among saved shows, but librespot can't play
    /// them, so the Podcasts shelf leaves out any show marked as one.
    #[test]
    fn the_podcasts_shelf_leaves_out_audiobooks() {
        let (ctx, mut app) = accessible_app("library-podcasts-audiobooks");
        let view = crate::ui::sidebar::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let chip = painted
            .iter()
            .find(|(text, _)| text == "Podcasts")
            .unwrap()
            .1
            .center();
        view_frame(
            &ctx,
            &mut app,
            pointer_click(chip, egui::PointerButton::Primary),
            view,
        );
        let shows: Vec<(String, String)> = app
            .library
            .shows
            .items
            .iter()
            .map(|saved| (saved.show.uri.clone(), saved.show.name.clone()))
            .collect();
        assert!(shows.len() >= 2, "the demo library saves several shows");
        let painted = view_frame(&ctx, &mut app, vec![], view);
        assert!(painted.iter().any(|(text, _)| *text == shows[0].1));

        app.audiobook_shows.insert(shows[0].0.clone());
        let painted = view_frame(&ctx, &mut app, vec![], view);
        assert!(!painted.iter().any(|(text, _)| *text == shows[0].1));
        assert!(painted.iter().any(|(text, _)| *text == shows[1].1));
        app.backend.shutdown();
    }

    #[test]
    fn library_grid_toggle_is_accessible_and_persistent() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("library-grid-toggle");
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let grid = accessible_node(&tree, "Show as grid", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(grid, AccessibleAction::Click, None)],
        );
        assert!(app.settings.sidebar_grid);

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let list = accessible_node(&tree, "Show as list", Role::Button);
        assert!(tree.nodes.iter().any(|(_, node)| {
            node.role() == Role::Button && node.label() == Some("Discover Weekly")
        }));
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(list, AccessibleAction::Click, None)],
        );
        assert!(!app.settings.sidebar_grid);
        app.backend.shutdown();
    }

    #[test]
    fn the_liked_songs_menu_is_as_narrow_as_the_other_menus() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("liked-menu-width");
        let button = |tree: &egui::accesskit::TreeUpdate, label: &str| {
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
                .unwrap_or_else(|| panic!("missing button {label}"))
                .1
                .bounds()
                .unwrap();
            (
                bounds.x0 as f32,
                bounds.x1 as f32,
                bounds.y0 as f32,
                bounds.y1 as f32,
            )
        };

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let (x0, x1, y0, y1) = button(&tree, "Liked Songs");
        accessible_frame(
            &ctx,
            &mut app,
            pointer_click(
                egui::pos2((x0 + x1) / 2.0, (y0 + y1) / 2.0),
                egui::PointerButton::Secondary,
            ),
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        // A menu item spans its menu, so its width is the menu's.
        let (x0, x1, _, _) = button(&tree, "Play");
        assert!(x1 - x0 <= 300.0, "the menu is {} points wide", x1 - x0);
        app.backend.shutdown();
    }

    #[test]
    fn library_grid_ignores_song_and_card_drops_behind_expanded_art() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("library-grid-art-drops");
        app.settings.sidebar_grid = true;
        app.settings.art_expanded = true;
        assert!(app.now_playing().unwrap().art_url.is_some());
        let frame = |ctx: &egui::Context, app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 800.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::sidebar::show(app, ui),
            );
            output.textures_delta.clear();
            output
                .platform_output
                .accesskit_update
                .expect("sidebar tree")
        };
        let tree = frame(&ctx, &mut app, vec![]);
        let art = ctx
            .read_response(egui::Id::new("sidebar-art"))
            .expect("expanded artwork")
            .rect;
        let playlists = app.library.playlists.get().expect("demo playlists");
        // Pick an editable card actually covered by the artwork, regardless
        // of platform font metrics and title-bar height.
        let pos = tree
            .nodes
            .iter()
            .find_map(|(_, node)| {
                let label = node.label()?;
                if node.role() != Role::Button
                    || !playlists
                        .iter()
                        .any(|playlist| playlist.name == label && app.can_edit_playlist(playlist))
                {
                    return None;
                }
                let bounds = node.bounds()?;
                let card = egui::Rect::from_min_max(
                    egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                    egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
                );
                let overlap = card.intersect(art);
                (overlap.width() > 16.0 && overlap.height() > 16.0).then(|| overlap.center())
            })
            .expect("an editable card behind the artwork");
        // A visible card, for the control drop that must still land.
        let visible = tree
            .nodes
            .iter()
            .find_map(|(_, node)| {
                let bounds = (node.role() == Role::Button && node.label() == Some("Liked Songs"))
                    .then(|| node.bounds())
                    .flatten()?;
                Some(egui::pos2(bounds.x0 as f32 + 24.0, bounds.y0 as f32 + 24.0))
            })
            .expect("Liked Songs card");
        assert!(!art.contains(visible));
        let release = |app: &mut App, pos| {
            frame(&ctx, app, vec![egui::Event::PointerMoved(pos)]);
            frame(
                &ctx,
                app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        };
        app.actions.clear();
        let song = PlayableItem::Track(Track {
            uri: "spotify:track:art-drop".into(),
            name: "Art drop".into(),
            ..Default::default()
        });
        egui::DragAndDrop::set_payload(
            &ctx,
            DragTrack {
                title: "Art drop".into(),
                image: None,
                items: vec![song],
                from: None,
                source_playlist: None,
            },
        );
        release(&mut app, pos);
        assert!(!app.actions.iter().any(|action| matches!(
            action,
            Action::AddToPlaylist { .. } | Action::SetSavedMany { .. }
        )));
        egui::DragAndDrop::clear_payload(&ctx);

        let drag_card = || {
            egui::DragAndDrop::set_payload(
                &ctx,
                DragEntry {
                    uri: "spotify:playlist:pl1".into(),
                    title: "Late night focus".into(),
                    image: None,
                },
            );
        };
        let rearranged = |app: &App| {
            app.actions
                .iter()
                .any(|action| matches!(action, Action::ArrangeLibrary { .. }))
        };
        drag_card();
        release(&mut app, pos);
        assert!(!rearranged(&app), "a card dropped on the artwork moved");
        egui::DragAndDrop::clear_payload(&ctx);

        drag_card();
        release(&mut app, visible);
        assert!(rearranged(&app), "a card dropped on a visible card stayed");
        egui::DragAndDrop::clear_payload(&ctx);
        app.backend.shutdown();
    }

    #[test]
    fn library_grid_highlights_a_song_drop_target_during_drag() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("library-grid-drop-highlight");
        app.settings.sidebar_grid = true;
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let liked = tree
            .nodes
            .iter()
            .find_map(|(_, node)| {
                (node.role() == Role::Button && node.label() == Some("Liked Songs"))
                    .then(|| node.bounds())
                    .flatten()
            })
            .expect("Liked Songs card");
        let pos = egui::pos2(liked.x0 as f32 + 24.0, liked.y0 as f32 + 24.0);
        egui::DragAndDrop::set_payload(
            &ctx,
            DragTrack {
                title: "Art drop".into(),
                image: None,
                items: vec![PlayableItem::Track(Track {
                    uri: "spotify:track:highlight".into(),
                    ..Default::default()
                })],
                from: None,
                source_playlist: None,
            },
        );
        let run = |app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 800.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| app.frame_ui(ui),
            );
            output.textures_delta.clear();
            output
        };
        // Hold the button down away from the card, as a real drag does:
        // egui then stops reporting other widgets as hovered, so the
        // outline has to follow the pointer instead.
        let start = egui::pos2(900.0, 400.0);
        run(
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        run(&mut app, vec![egui::Event::PointerMoved(pos)]);
        let output = run(&mut app, vec![egui::Event::PointerMoved(pos)]);
        assert!(ctx.input(|input| input.pointer.primary_down()));
        assert!(
            output.shapes.iter().any(|clipped| matches!(&clipped.shape,
                egui::epaint::Shape::Rect(rect)
                    if rect.stroke.width == 2.0
                        && rect.stroke.color == app.palette.accent
                        && rect.rect.contains(pos)
            )),
            "no accent outline on the drop target"
        );
        egui::DragAndDrop::clear_payload(&ctx);
        app.backend.shutdown();
    }

    #[test]
    fn library_grid_cards_navigate_and_their_corner_buttons_play() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("library-grid-card-actions");
        app.settings.sidebar_grid = true;
        app.open(Page::Search);

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let card = accessible_node(&tree, "Sunday morning", Role::Button);
        let previous_context = app.playing_context_uri();
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(card, AccessibleAction::Click, None)],
        );
        assert_eq!(app.page(), &Page::Playlist("pl2".into()));
        assert_eq!(app.playing_context_uri(), previous_context);

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let play = accessible_node(&tree, "Play Sunday morning", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(play, AccessibleAction::Click, None)],
        );
        assert_eq!(
            app.playing_context_uri().as_deref(),
            Some("spotify:playlist:pl2")
        );
        assert_eq!(app.page(), &Page::Playlist("pl2".into()));
        app.backend.shutdown();
    }

    #[test]
    fn double_clicking_a_library_grid_card_only_navigates() {
        let (ctx, mut app) = accessible_app("library-grid-double-click");
        app.settings.sidebar_grid = true;
        app.open(Page::Search);
        let view = crate::ui::sidebar::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let card = sidebar_text(&painted, "Sunday morning").center();
        app.actions.clear();

        let [first, second] = double_click(card);
        view_frame(&ctx, &mut app, first, view);
        view_frame(&ctx, &mut app, second, view);

        assert!(played_contexts(&app).is_empty());
        assert!(
            app.actions
                .iter()
                .any(|action| matches!(action, Action::Open(Page::Playlist(id)) if id == "pl2"))
        );
        app.backend.shutdown();
    }

    #[test]
    fn library_sorts_finish_paging_without_retrying_failed_pages() {
        use crate::settings::{LibraryShelf, LibrarySort};
        for (shelf, label, page) in [
            (LibraryShelf::Albums, "Albums", Page::Albums),
            (LibraryShelf::Artists, "Artists", Page::Artists),
            (LibraryShelf::Podcasts, "Podcasts", Page::Podcasts),
        ] {
            let (ctx, mut app) = accessible_app(&format!("library-sort-paging-{shelf:?}"));
            let view = crate::ui::sidebar::show;
            app.settings.library_sort.insert(shelf, LibrarySort::Name);
            app.library.albums.next_offset = Some(50);
            app.library.artists.complete = false;
            app.library.artists.after = Some("next".into());
            app.library.shows.next_offset = Some(50);
            view_frame(&ctx, &mut app, vec![], view);
            let painted = view_frame(&ctx, &mut app, vec![], view);
            let position = painted
                .iter()
                .find(|(text, _)| text == label)
                .unwrap()
                .1
                .center();
            view_frame(
                &ctx,
                &mut app,
                pointer_click(position, egui::PointerButton::Primary),
                view,
            );
            app.actions.clear();
            view_frame(&ctx, &mut app, vec![], view);
            assert!(
                app.actions
                    .iter()
                    .any(|action| matches!(action, Action::LoadMore(found) if *found == page))
            );
            app.actions.clear();
            app.library.albums.error = Some("Try again later".into());
            app.library.artists.error = Some("Try again later".into());
            app.library.shows.error = Some("Try again later".into());
            for _ in 0..3 {
                view_frame(&ctx, &mut app, vec![], view);
                assert!(
                    !app.actions
                        .iter()
                        .any(|action| matches!(action, Action::LoadMore(found) if *found == page)),
                    "failed pages must not retry every frame"
                );
            }
            app.backend.shutdown();
        }
    }

    #[test]
    fn slow_banner_closes_or_opens_personal_app_setup() {
        use egui::accesskit::{Action as AccessibleAction, Role};

        for setup in [false, true] {
            let (ctx, mut app) = accessible_app(&format!("slow-banner-{setup}"));
            app.slow_banner = true;
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let button = accessible_node(
                &tree,
                if setup {
                    "Register Spotifrost"
                } else {
                    "Close banner"
                },
                Role::Button,
            );
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(button, AccessibleAction::Click, None)],
            );
            assert!(!app.slow_banner);
            assert!(app.dialog.is_none());
            if setup {
                assert_eq!(app.page(), &Page::Settings);
                assert_eq!(app.settings.slow_app_banner.dismissals, 0);
            } else {
                assert_eq!(app.settings.slow_app_banner.dismissals, 1);
            }
            app.backend.shutdown();
        }
    }

    #[test]
    fn audio_key_banner_opens_the_device_picker_or_closes() {
        use egui::accesskit::{Action as AccessibleAction, Role};

        for other_device in [false, true] {
            let (ctx, mut app) = accessible_app(&format!("audio-key-banner-{other_device}"));
            app.audio_key_banner = true;
            app.slow_banner = true;
            app.show_devices = false;
            let title = "Spotify won't let this computer play this song right now.";
            let shows = |tree: &egui::accesskit::TreeUpdate, text: &str| {
                tree.nodes
                    .iter()
                    .any(|(_, node)| node.label() == Some(text) || node.value() == Some(text))
            };
            app.dialog = Some(Dialog::Shortcuts);
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            assert!(!shows(&tree, title), "not over a dialog");
            app.dialog = None;
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            assert!(shows(&tree, title));
            assert!(
                !shows(&tree, "Spotify is being slow on the shared sign-in."),
                "one banner at a time, the audio key's first"
            );
            let button = accessible_node(
                &tree,
                if other_device {
                    "Play on another device"
                } else {
                    "Close banner"
                },
                Role::Button,
            );
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(button, AccessibleAction::Click, None)],
            );
            if other_device {
                assert!(app.show_devices, "the device picker opens");
                assert!(app.audio_key_banner, "until a device is chosen");
            } else {
                assert!(!app.audio_key_banner);
                assert!(app.slow_banner, "the slow banner's turn comes next");
            }
            app.backend.shutdown();
        }
    }

    #[test]
    fn personal_app_intro_can_be_dismissed_or_open_setup_with_keyboard_focus() {
        use egui::accesskit::{Action as AccessibleAction, Role};

        for setup in [false, true] {
            let (ctx, mut app) = accessible_app(&format!("personal-app-intro-{setup}"));
            app.dialog = Some(Dialog::PersonalAppIntro);
            accessible_frame(&ctx, &mut app, vec![]);
            let tree = accessible_frame(&ctx, &mut app, vec![]);
            let button = accessible_node(
                &tree,
                if setup {
                    "Register Spotifrost"
                } else {
                    "Not now"
                },
                Role::Button,
            );
            accessible_frame(
                &ctx,
                &mut app,
                vec![accessible_action(button, AccessibleAction::Click, None)],
            );
            assert!(app.dialog.is_none());
            assert!(app.settings.personal_app_intro_seen);
            if setup {
                assert_eq!(app.page(), &Page::Settings);
                accessible_frame(&ctx, &mut app, vec![]);
                assert!(
                    ctx.memory(|memory| memory.has_focus(egui::Id::new("personal-web-client-id")))
                );
            }
            app.backend.shutdown();
        }
        let (ctx, mut app) = accessible_app("personal-app-intro-escape");
        app.dialog = Some(Dialog::PersonalAppIntro);
        accessible_frame(&ctx, &mut app, vec![]);
        accessible_frame(
            &ctx,
            &mut app,
            vec![keyboard(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(app.dialog.is_none());
        assert!(app.settings.personal_app_intro_seen);
        app.backend.shutdown();
    }

    fn settings_filter(ctx: &egui::Context) -> String {
        ctx.data_mut(|data| {
            data.get_temp::<String>(egui::Id::new("settings-filter"))
                .unwrap_or_default()
        })
    }

    #[test]
    fn the_cross_on_a_recent_search_forgets_only_that_query() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("search-history");
        app.open(Page::Search);
        // Recent searches stand in for results only while nothing is searched
        app.search.query.clear();
        app.search.committed.clear();
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let forget = accessible_node(&tree, "Remove ambient", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(forget, AccessibleAction::Click, None)],
        );
        assert_eq!(app.settings.search_history, ["Khruangbin", "Rework"]);
        assert!(
            app.search.committed.is_empty(),
            "the cross must forget a query without running it"
        );
        app.backend.shutdown();
    }

    /// #596: clearing the global search from another page empties the field
    /// without leaving that page.
    #[test]
    fn clearing_the_global_search_stays_on_the_current_page() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("global-search-clear");
        app.open(Page::Home);
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(!app.search.query.is_empty());
        let clear = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.label() == Some("Clear")
                    && node.role() == Role::Button
                    && node.bounds().is_some_and(|bounds| bounds.y1 < 80.0)
            })
            .expect("Clear beside the global search")
            .0;
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(clear, AccessibleAction::Click, None)],
        );
        for _ in 0..3 {
            accessible_frame(&ctx, &mut app, vec![]);
        }
        assert!(app.search.query.is_empty());
        assert!(
            matches!(app.page(), Page::Home),
            "clearing must not open the search page"
        );
        assert!(
            ctx.memory(|memory| memory.has_focus(egui::Id::new("global-search"))),
            "the field stays ready for the next query"
        );
        // Typing a new query still goes to the search page.
        accessible_frame(&ctx, &mut app, vec![egui::Event::Text("Rework".into())]);
        accessible_frame(&ctx, &mut app, vec![]);
        assert_eq!(app.search.query, "Rework");
        assert!(matches!(app.page(), Page::Search));
        app.backend.shutdown();
    }

    #[test]
    fn personal_app_setup_clears_a_saved_settings_search() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("settings-search-setup");
        app.open(Page::Settings);
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let field = accessible_node(&tree, "Search settings", Role::TextInput);
        // Search for something that hides the Account section...
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(field, AccessibleAction::Focus, None)],
        );
        accessible_frame(&ctx, &mut app, vec![egui::Event::Text("theme".into())]);
        assert_eq!(settings_filter(&ctx), "theme");
        // ...leave Settings...
        app.open(Page::Home);
        accessible_frame(&ctx, &mut app, vec![]);
        // ...and enter Personal App setup, which must reveal and focus
        // the Client ID field instead of landing on a filtered-out row.
        app.dialog = Some(Dialog::PersonalAppIntro);
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let setup = accessible_node(&tree, "Register Spotifrost", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(setup, AccessibleAction::Click, None)],
        );
        assert_eq!(app.page(), &Page::Settings);
        assert!(
            settings_filter(&ctx).is_empty(),
            "setup must drop the saved search so its row is visible"
        );
        accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            ctx.memory(|memory| memory.has_focus(egui::Id::new("personal-web-client-id"))),
            "the Client ID field takes focus once its row is visible"
        );
        app.backend.shutdown();
    }

    fn settings_text(ctx: &egui::Context, app: &mut App, query: &str) -> Vec<String> {
        ctx.data_mut(|data| {
            data.insert_temp(egui::Id::new("settings-filter"), query.to_owned());
        });
        view_frame(ctx, app, vec![], crate::ui::settings::show);
        view_frame(ctx, app, vec![], crate::ui::settings::show)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    /// The About card ends with the author's credit, and the name opens
    /// the author's website.
    #[test]
    fn the_about_card_credits_the_author() {
        let (ctx, mut app) = accessible_app("about-credit");
        settings_text(&ctx, &mut app, "for fun");
        let texts = view_frame(&ctx, &mut app, vec![], crate::ui::settings::show);
        assert!(
            texts
                .iter()
                .any(|(text, _)| text.contains("Built for fun by")),
            "{texts:?}"
        );
        let name = texts
            .iter()
            .find(|(text, _)| text == "spham2mail-star")
            .map(|(_, rect)| rect.center())
            .expect("the author's name");
        app.actions.clear();
        view_frame(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(name),
                egui::Event::PointerButton {
                    pos: name,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: name,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            crate::ui::settings::show,
        );
        assert!(
            app.actions.iter().any(|action| matches!(
                action,
                Action::OpenUrl(url) if url == crate::ui::widgets::AUTHOR_URL
            )),
            "{:?}",
            app.actions
        );
        app.backend.shutdown();
    }

    #[test]
    fn create_an_app_row_hides_once_the_personal_app_is_ready() {
        let (ctx, mut app) = accessible_app("settings-create-app");
        app.settings.web_client_id = None;
        app.web_app = None;
        let text = settings_text(&ctx, &mut app, "Account");
        assert!(text.iter().any(|text| text == "Register Spotifrost"));

        app.settings.web_client_id = Some("test-client".into());
        let text = settings_text(&ctx, &mut app, "Account");
        assert!(text.iter().any(|text| text == "Register Spotifrost"));
        assert!(text.iter().any(|text| text == "Authorize your Client ID"));

        app.web_app = Some("test-client".into());
        let text = settings_text(&ctx, &mut app, "Account");
        assert!(text.iter().any(|text| text == "Spotifrost registered"));
        assert!(!text.iter().any(|text| text == "Register Spotifrost"));
        assert!(!text.iter().any(|text| text == "Setup guide"));
        app.backend.shutdown();
    }

    #[test]
    fn settings_search_keeps_apply_available_after_a_playback_edit() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("settings-search-apply");
        app.open(Page::Settings);
        ctx.data_mut(|data| {
            data.insert_temp(egui::Id::new("settings-filter"), "normalize".to_string())
        });
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let toggle = accessible_node(&tree, "Normalize volume", Role::CheckBox);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(toggle, AccessibleAction::Click, None)],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        accessible_node(&tree, "Apply and restart playback", Role::Button);
        app.backend.shutdown();
    }

    #[test]
    fn accessible_sliders_accept_keyboard_and_screen_reader_values() {
        use crate::ui::widgets::{SliderEvent, thin_slider};
        use egui::accesskit::{Action, ActionData, Role};
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        crate::theme::install(&ctx);
        let palette = crate::theme::Palette::dark();
        let mut value = 0.5;
        let mut render = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    if let SliderEvent::Committed(next) = thin_slider(
                        ui,
                        &palette,
                        egui::Id::new("test-volume"),
                        "Volume (%)",
                        value,
                        200.0,
                        Some(0.05),
                    ) {
                        value = next;
                    }
                },
            );
            output.textures_delta.clear();
            (value, output.platform_output.accesskit_update.unwrap())
        };
        let (_, tree) = render(vec![]);
        let slider = accessible_node(&tree, "Volume (%)", Role::Slider);
        render(vec![accessible_action(slider, Action::Focus, None)]);
        let (value, _) = render(vec![keyboard(egui::Key::ArrowRight, egui::Modifiers::NONE)]);
        assert!((value - 0.55).abs() < 0.001);
        let (value, tree) = render(vec![accessible_action(
            slider,
            Action::SetValue,
            Some(ActionData::NumericValue(35.0)),
        )]);
        assert!((value - 0.35).abs() < 0.001);
        let node = &tree.nodes.iter().find(|(id, _)| *id == slider).unwrap().1;
        assert!((node.numeric_value().unwrap() - 35.0).abs() < 0.001);
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(100.0));
        let (value, _) = render(vec![accessible_action(
            slider,
            Action::SetValue,
            Some(ActionData::NumericValue(200.0)),
        )]);
        assert_eq!(value, 1.0);
        let (value, _) = render(vec![accessible_action(
            slider,
            Action::SetValue,
            Some(ActionData::NumericValue(f64::NAN)),
        )]);
        assert_eq!(value, 1.0);
    }

    #[test]
    fn accessible_switch_has_a_name_state_and_keyboard_activation() {
        use egui::accesskit::{Action, Role, Toggled};
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        crate::theme::install(&ctx);
        let palette = crate::theme::Palette::dark();
        let mut on = false;
        let mut render = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    crate::ui::widgets::switch(ui, &palette, "Autoplay", &mut on);
                },
            );
            output.textures_delta.clear();
            (on, output.platform_output.accesskit_update.unwrap())
        };
        let (_, tree) = render(vec![]);
        let switch = accessible_node(&tree, "Autoplay", Role::CheckBox);
        let node = &tree.nodes.iter().find(|(id, _)| *id == switch).unwrap().1;
        assert_eq!(node.toggled(), Some(Toggled::False));
        render(vec![accessible_action(switch, Action::Focus, None)]);
        let (on, tree) = render(vec![keyboard(egui::Key::Space, egui::Modifiers::NONE)]);
        assert!(on);
        let node = &tree.nodes.iter().find(|(id, _)| *id == switch).unwrap().1;
        assert_eq!(node.toggled(), Some(Toggled::True));
    }

    #[test]
    fn accessible_song_focus_survives_visible_row_changes_and_keeps_duplicates_distinct() {
        use crate::ui::widgets::{TrackRow, track_row};
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("rows");
        let item = app.queue.get().unwrap().queue[0].clone();
        let context = crate::model::RowContext::Uris(Arc::from(vec![item.uri().to_string(); 2]));
        let label = format!("Play {}, {}", item.name(), item.subtitle());
        let mut render = |first, events| {
            app.actions.clear();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    for index in first..2 {
                        track_row(
                            ui,
                            &mut app,
                            TrackRow {
                                index,
                                number: Some(index + 1),
                                item: &item,
                                context: &context,
                                show_cover: false,
                                show_album: false,
                                added_at: None,
                                added_by: None,
                                show_added_by: false,
                                show_plays: false,
                                compact: false,
                                thin: false,
                                shift: 0.0,
                                picked: false,
                                picked_songs: &[],
                                picked_positions: &[],
                                position: None,
                            },
                        );
                    }
                },
            );
            output.textures_delta.clear();
            (
                output.platform_output.accesskit_update.unwrap(),
                app.actions.clone(),
            )
        };
        let (tree, _) = render(0, vec![]);
        let mut positioned_rows: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some(label.as_str()) && node.role() == Role::Button)
            .map(|(id, node)| (*id, node.bounds().unwrap().y0))
            .collect();
        positioned_rows.sort_by(|a, b| a.1.total_cmp(&b.1));
        let rows: Vec<_> = positioned_rows.into_iter().map(|(id, _)| id).collect();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0], rows[1]);
        render(
            0,
            vec![accessible_action(rows[1], AccessibleAction::Focus, None)],
        );
        let (tree, _) = render(1, vec![]);
        assert_eq!(
            accessible_node(&tree, &label, Role::Button),
            rows[1],
            "scrolling must not give a song another row's identity"
        );
        assert_eq!(tree.focus, rows[1]);
        let (_, actions) = render(1, vec![keyboard(egui::Key::Enter, egui::Modifiers::NONE)]);
        assert!(matches!(
            actions.as_slice(),
            [crate::model::Action::PlayFromRow { index: 1, .. }]
        ));
        let (tree, _) = render(1, vec![]);
        let more = accessible_node(&tree, "More", Role::Button);
        render(
            1,
            vec![accessible_action(more, AccessibleAction::Focus, None)],
        );
        let (tree, _) = render(1, vec![]);
        assert_eq!(
            tree.focus, more,
            "More must remain reachable after focus leaves the song row"
        );
        render(1, vec![keyboard(egui::Key::Enter, egui::Modifiers::NONE)]);
        // The menu's first frame only measures it, and egui keeps widgets it
        // has not shown out of the screen-reader tree.
        let (tree, _) = render(1, vec![]);
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Add to queue")),
            "the keyboard opens the song menu"
        );
        app.backend.shutdown();
    }

    #[test]
    fn accessible_tab_reaches_songs_beyond_the_visible_list() {
        use crate::ui::widgets::{TrackRow, track_row, virtual_rows};
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("scroll");
        let item = app.queue.get().unwrap().queue[0].clone();
        let context = crate::model::RowContext::Uris(Arc::from(vec![item.uri().to_string(); 20]));
        let label = format!("Play {}, {}", item.name(), item.subtitle());
        let mut reached_last = false;
        let mut render = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(700.0, 200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical().animated(false).show(ui, |ui| {
                        virtual_rows(ui, 20, crate::theme::ROW_HEIGHT, |ui, index| {
                            if index == 19 && ui.cursor().top() < ui.clip_rect().bottom() {
                                reached_last = true;
                            }
                            track_row(
                                ui,
                                &mut app,
                                TrackRow {
                                    index,
                                    number: Some(index + 1),
                                    item: &item,
                                    context: &context,
                                    show_cover: false,
                                    show_album: false,
                                    added_at: None,
                                    added_by: None,
                                    show_added_by: false,
                                    show_plays: false,
                                    compact: false,
                                    thin: false,
                                    shift: 0.0,
                                    picked: false,
                                    picked_songs: &[],
                                    picked_positions: &[],
                                    position: None,
                                },
                            );
                        });
                    });
                },
            );
            output.textures_delta.clear();
            output.platform_output.accesskit_update.unwrap()
        };
        let tree = render(vec![]);
        let first = accessible_node(&tree, &label, Role::Button);
        render(vec![accessible_action(
            first,
            AccessibleAction::Focus,
            None,
        )]);
        for _ in 0..120 {
            render(vec![keyboard(egui::Key::Tab, egui::Modifiers::NONE)]);
        }
        assert!(
            reached_last,
            "Tab must scroll through the virtual list instead of trapping focus in its first visible rows"
        );
        app.backend.shutdown();
    }

    #[test]
    fn accessible_tab_reaches_cards_beyond_the_visible_grid() {
        use crate::ui::widgets::{CardCover, card, card_row_height, virtual_wrapped_cards};
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("card-grid");
        let mut reached_last = false;
        let mut render = |events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(400.0, 280.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical().animated(false).show(ui, |ui| {
                        let height = card_row_height(ui);
                        virtual_wrapped_cards(ui, 24, height, |ui, index| {
                            if index == 23 && ui.cursor().top() < ui.clip_rect().bottom() {
                                reached_last = true;
                            }
                            card(
                                ui,
                                &mut app,
                                None,
                                &format!("Album {index}"),
                                "Artist",
                                CardCover::default(),
                            );
                        });
                    });
                },
            );
            output.textures_delta.clear();
            output.platform_output.accesskit_update.unwrap()
        };
        let tree = render(vec![]);
        let first = accessible_node(&tree, "Album 0, Artist", Role::Button);
        render(vec![accessible_action(
            first,
            AccessibleAction::Focus,
            None,
        )]);
        for _ in 0..80 {
            render(vec![keyboard(egui::Key::Tab, egui::Modifiers::NONE)]);
        }
        assert!(
            reached_last,
            "Tab must scroll through the virtual card grid instead of trapping focus in its first visible row"
        );
        app.backend.shutdown();
    }

    #[test]
    fn accessible_playing_and_queued_copies_target_their_own_context() {
        use crate::model::RowContext;
        use crate::ui::widgets::{TrackRow, track_row};
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("queued-copy");
        let item = app.queue.get().unwrap().currently_playing.clone().unwrap();
        let contexts = [
            RowContext::Uris(Arc::from([item.uri().to_string()])),
            RowContext::Queue,
        ];
        let label = format!("Play {}, {}", item.name(), item.subtitle());
        let mut render = |events| {
            app.actions.clear();
            let mut output = ctx.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    for context in &contexts {
                        track_row(
                            ui,
                            &mut app,
                            TrackRow {
                                index: 0,
                                number: Some(1),
                                item: &item,
                                context,
                                show_cover: false,
                                show_album: false,
                                added_at: None,
                                added_by: None,
                                show_added_by: false,
                                show_plays: false,
                                compact: false,
                                thin: false,
                                shift: 0.0,
                                picked: false,
                                picked_songs: &[],
                                picked_positions: &[],
                                position: None,
                            },
                        );
                    }
                },
            );
            output.textures_delta.clear();
            (
                output.platform_output.accesskit_update.unwrap(),
                app.actions.clone(),
            )
        };
        let (tree, _) = render(vec![]);
        let mut rows: Vec<_> = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some(label.as_str()) && node.role() == Role::Button)
            .map(|(id, node)| (*id, node.bounds().unwrap().y0))
            .collect();
        rows.sort_by(|a, b| a.1.total_cmp(&b.1));
        assert_eq!(
            rows.len(),
            2,
            "the playing song and queued copy need separate controls"
        );
        let (_, actions) = render(vec![accessible_action(
            rows[1].0,
            AccessibleAction::Click,
            None,
        )]);
        assert!(matches!(
            actions.as_slice(),
            [crate::model::Action::PlayFromRow {
                context: RowContext::Queue,
                index: 0,
                ..
            }]
        ));
        let (_, actions) = render(vec![accessible_action(
            rows[0].0,
            AccessibleAction::Click,
            None,
        )]);
        assert!(matches!(
            actions.as_slice(),
            [crate::model::Action::PlayFromRow {
                context: RowContext::Uris(_),
                index: 0,
                ..
            }]
        ));
        app.backend.shutdown();
    }

    fn frame(ctx: &egui::Context, app: &mut App) {
        frame_events(ctx, app, Vec::new());
    }

    #[test]
    fn a_long_device_list_stays_in_the_window_and_scrolls_to_the_last_speaker() {
        let (ctx, mut app) = accessible_app("long-device-list");
        app.backend.set_offline(true);
        app.show_devices = true;
        app.local_ready = true;
        app.receivers.clear();
        app.devices = (0..40)
            .map(|index| Device {
                id: Some(format!("speaker-{index}")),
                name: format!("Speaker {index:02}"),
                kind: "speaker".into(),
                ..Default::default()
            })
            .collect();
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(760.0, 620.0));
        ctx.data_mut(|data| {
            data.insert_temp(
                egui::Id::new(crate::ui::devices::BUTTON_RECT_ID),
                egui::Rect::from_min_size(egui::pos2(680.0, 580.0), egui::vec2(32.0, 32.0)),
            )
        });
        let draw = |app: &mut App, events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(screen),
                    events,
                    ..Default::default()
                },
                |_| crate::ui::devices::popup(app, &ctx),
            );
            output.textures_delta.clear();
            output
        };
        // Open with only this computer, then let device discovery fill the list.
        let discovered = std::mem::take(&mut app.devices);
        draw(&mut app, vec![]);
        draw(&mut app, vec![]);
        app.devices = discovered;
        draw(&mut app, vec![]);
        let output = draw(&mut app, vec![]);
        let visible_speakers = output
            .shapes
            .iter()
            .filter(|shape| {
                matches!(&shape.shape, egui::epaint::Shape::Text(text)
                    if text.galley.job.text.starts_with("Speaker ")
                        && shape.clip_rect.contains_rect(text.visual_bounding_rect()))
            })
            .count();
        assert!(
            visible_speakers >= 5,
            "show several devices before scrolling, found {visible_speakers}"
        );
        let popup = egui::AreaState::load(&ctx, egui::Id::new("devices-popup"))
            .unwrap()
            .rect();
        assert!(screen.contains_rect(popup), "the popup must fit: {popup:?}");
        let cursor = popup.center();
        draw(
            &mut app,
            vec![
                egui::Event::PointerMoved(cursor),
                egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, -10_000.0),
                    modifiers: egui::Modifiers::NONE,
                    phase: egui::TouchPhase::Move,
                },
            ],
        );
        let mut found = None;
        for _ in 0..30 {
            let output = draw(&mut app, vec![]);
            for shape in output.shapes {
                if let egui::epaint::Shape::Text(text) = shape.shape
                    && text.galley.job.text == "Speaker 39"
                    && shape.clip_rect.contains_rect(text.visual_bounding_rect())
                {
                    found = Some(text.visual_bounding_rect().center());
                }
            }
        }
        let last = found.expect("scrolling reaches the last speaker");
        app.actions.clear();
        draw(&mut app, pointer_click(last, egui::PointerButton::Primary));
        assert!(app.actions.iter().any(
            |action| matches!(action, crate::model::Action::Transfer(id) if id == "speaker-39")
        ));
        app.devices.clear();
        draw(&mut app, vec![]);
        draw(&mut app, vec![]);
        let popup = egui::AreaState::load(&ctx, egui::Id::new("devices-popup"))
            .unwrap()
            .rect();
        assert!(
            popup.height() < 160.0,
            "a list with only this computer should shrink to fit: {popup:?}"
        );
        app.backend.shutdown();
    }

    #[test]
    fn playlist_filter_preserves_edit_permissions_and_keyboard_selection() {
        use egui::accesskit::Role;
        for count in [1, 2] {
            let (ctx, mut app) = accessible_app(&format!("playlist-filter-{count}"));
            app.backend.set_offline(true);
            let owner = app.user_id().unwrap().to_string();
            let make = |id: &str, name: &str, owned: bool, collaborative| Playlist {
                id: id.into(),
                uri: format!("spotify:playlist:{id}"),
                name: name.into(),
                owner: crate::api::models::Owner {
                    id: Some(if owned {
                        owner.clone()
                    } else {
                        "another-user".into()
                    }),
                    ..Default::default()
                },
                collaborative,
                ..Default::default()
            };
            app.library.playlists = Loadable::Loaded(vec![
                make("readonly", "Night locked", false, false),
                make("owned", "Night drive", true, false),
                make("shared", "Night together", false, true),
                make("day", "Daylight", true, false),
            ]);
            let items: Vec<_> = (0..count).map(|i| PlayableItem::Track(track(i))).collect();
            let mut query = String::new();
            let draw = |app: &mut App, query: &mut String, focus: bool, events| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(760.0, 620.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        let field =
                            crate::ui::widgets::playlist_picker(ui, app, &items, query, None);
                        if focus {
                            field.request_focus();
                        }
                    },
                );
                output.textures_delta.clear();
                output
            };
            draw(&mut app, &mut query, true, vec![]);
            let output = draw(
                &mut app,
                &mut query,
                false,
                vec![egui::Event::Text("  NiGhT  ".into())],
            );
            assert_eq!(query, "  NiGhT  ");
            let tree = output.platform_output.accesskit_update.unwrap();
            for name in ["Night locked", "Daylight"] {
                assert!(
                    !tree
                        .nodes
                        .iter()
                        .any(|(_, node)| node.label() == Some(name)),
                    "{name} must not be offered"
                );
            }
            let owned = accessible_node(&tree, "Night drive", Role::Button);
            accessible_node(&tree, "Night together", Role::Button);
            let mut reached = false;
            for _ in 0..6 {
                let output = draw(
                    &mut app,
                    &mut query,
                    false,
                    vec![keyboard(egui::Key::Tab, egui::Modifiers::NONE)],
                );
                if output.platform_output.accesskit_update.unwrap().focus == owned {
                    reached = true;
                    break;
                }
            }
            assert!(
                reached,
                "Tab must reach the filtered playlist from the search field"
            );
            app.actions.clear();
            draw(
                &mut app,
                &mut query,
                false,
                vec![keyboard(egui::Key::Enter, egui::Modifiers::NONE)],
            );
            assert!(
                matches!(app.actions.as_slice(), [crate::model::Action::AddToPlaylist { playlist_id, items: selected, .. }] if playlist_id == "owned" && selected == &items)
            );
            query = "no such playlist".into();
            let output = draw(&mut app, &mut query, false, vec![]);
            let tree = output.platform_output.accesskit_update.unwrap();
            accessible_node(&tree, "New playlist", Role::Button);
            assert!(output.shapes.iter().any(|shape| matches!(&shape.shape, egui::epaint::Shape::Text(text) if text.galley.job.text == "No matching playlists")));
            app.backend.shutdown();
        }
    }

    /// Typing in the playlist filter chooses the first match, so Enter adds
    /// to it at once; Up and Down move the choice first.
    #[test]
    fn typing_in_the_playlist_filter_chooses_the_first_match() {
        let (ctx, mut app) = accessible_app("playlist-filter-choice");
        app.backend.set_offline(true);
        let owner = app.user_id().unwrap().to_string();
        let make = |id: &str, name: &str| Playlist {
            id: id.into(),
            uri: format!("spotify:playlist:{id}"),
            name: name.into(),
            owner: crate::api::models::Owner {
                id: Some(owner.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        app.library.playlists = Loadable::Loaded(vec![
            make("drive", "Night drive"),
            make("day", "Daylight"),
            make("walk", "Night walk"),
            make("owls", "Night owls"),
        ]);
        let items = vec![PlayableItem::Track(track(0))];
        let draw = |app: &mut App, query: &mut String, focus: bool, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let field = crate::ui::widgets::playlist_picker(ui, app, &items, query, None);
                    if focus {
                        field.request_focus();
                    }
                },
            );
            output.textures_delta.clear();
        };
        let added = |app: &App| match app.actions.as_slice() {
            [crate::model::Action::AddToPlaylist { playlist_id, .. }] => Some(playlist_id.clone()),
            [] => None,
            other => panic!("expected one addition, got {other:?}"),
        };
        let key = |key| vec![keyboard(key, egui::Modifiers::NONE)];

        // #given the filter has focus
        let mut query = String::new();
        draw(&mut app, &mut query, true, vec![]);

        // #when a filter is typed and Enter pressed
        draw(
            &mut app,
            &mut query,
            false,
            vec![egui::Event::Text("night".into())],
        );
        draw(&mut app, &mut query, false, key(egui::Key::Enter));

        // #then the first match is added
        assert_eq!(added(&app).as_deref(), Some("drive"));

        // #when Down moves the choice twice and Up once, past both ends
        app.actions.clear();
        let mut query = String::new();
        draw(&mut app, &mut query, true, vec![]);
        draw(
            &mut app,
            &mut query,
            false,
            vec![egui::Event::Text("night".into())],
        );
        draw(&mut app, &mut query, false, key(egui::Key::ArrowUp));
        for _ in 0..3 {
            draw(&mut app, &mut query, false, key(egui::Key::ArrowDown));
        }
        draw(&mut app, &mut query, false, key(egui::Key::ArrowUp));
        assert_eq!(query, "night", "the arrows do not edit the filter");
        draw(&mut app, &mut query, false, key(egui::Key::Enter));

        // #then the choice it reached is added
        assert_eq!(added(&app).as_deref(), Some("walk"));

        // #when the filter changes after a choice was moved
        app.actions.clear();
        let mut query = String::new();
        draw(&mut app, &mut query, true, vec![]);
        draw(
            &mut app,
            &mut query,
            false,
            vec![egui::Event::Text("night".into())],
        );
        draw(&mut app, &mut query, false, key(egui::Key::ArrowDown));
        draw(
            &mut app,
            &mut query,
            false,
            vec![egui::Event::Text(" o".into())],
        );
        draw(&mut app, &mut query, false, key(egui::Key::Enter));

        // #then the choice starts again at the first match
        assert_eq!(added(&app).as_deref(), Some("owls"));

        // #when Enter is pressed with nothing typed
        app.actions.clear();
        let mut query = String::new();
        draw(&mut app, &mut query, true, vec![]);
        draw(&mut app, &mut query, false, key(egui::Key::Enter));

        // #then nothing is chosen for the listener
        assert_eq!(added(&app), None);
        app.backend.shutdown();
    }

    /// In the heart's checklist a click checks a playlist instead of adding
    /// to it, a playlist holding the song starts checked, one not read in
    /// full is marked as unknown rather than lacking it, and Done adds to
    /// the newly checked and takes the song out of the unchecked.
    #[test]
    fn the_heart_checklist_adds_and_removes_only_on_done() {
        use crate::ui::widgets::Mark;
        use egui::accesskit::{Action as AccessibleAction, Role};

        let (ctx, mut app) = accessible_app("playlist-checklist");
        app.backend.set_offline(true);
        let owner = app.user_id().unwrap().to_string();
        let make = |id: &str, name: &str| Playlist {
            id: id.into(),
            uri: format!("spotify:playlist:{id}"),
            name: name.into(),
            owner: crate::api::models::Owner {
                id: Some(owner.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        app.library.playlists =
            Loadable::Loaded(vec![make("drive", "Night drive"), make("day", "Daylight")]);
        let item = PlayableItem::Track(track(0));
        app.playlist_pages.insert(
            "day".into(),
            PlaylistPage {
                local_additions: [item.uri().to_string()].into(),
                ..PlaylistPage::default()
            },
        );
        let items = vec![item.clone()];
        let mut checks = crate::ui::widgets::PlaylistChecks::new(&app, &item);
        assert_eq!(checks.mark("drive"), Mark::Unknown, "never read");
        let mut query = String::new();
        let mut draw = |app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    crate::ui::widgets::playlist_picker(
                        ui,
                        app,
                        &items,
                        &mut query,
                        Some(&mut checks),
                    );
                },
            );
            output.textures_delta.clear();
            output.platform_output.accesskit_update.unwrap()
        };
        let tree = draw(&mut app, vec![]);
        let checked = |tree: &egui::accesskit::TreeUpdate, name: &str| {
            let id = accessible_node(tree, name, Role::CheckBox);
            let node = &tree.nodes.iter().find(|(node, _)| *node == id).unwrap().1;
            node.toggled() == Some(egui::accesskit::Toggled::True)
        };
        assert!(checked(&tree, "Liked Songs"));
        assert!(checked(&tree, "Daylight"), "it holds the song");
        assert!(!checked(&tree, "Night drive"));

        for name in ["Night drive", "Daylight"] {
            let row = accessible_node(&tree, name, Role::CheckBox);
            draw(
                &mut app,
                vec![accessible_action(row, AccessibleAction::Click, None)],
            );
        }
        let tree = draw(&mut app, vec![]);
        assert!(checked(&tree, "Night drive"));
        assert!(!checked(&tree, "Daylight"));
        assert!(app.actions.is_empty(), "nothing changes before Done");

        let changes = checks.changes(&app, &item);
        assert!(matches!(
            changes,
            Some(crate::model::Action::UpdatePlaylists { item: changed, add, remove })
                if changed.uri() == item.uri()
                    && add == [("drive".to_string(), "Night drive".to_string())]
                    && remove == ["day"]
        ));

        app.playlist_pages.insert(
            "drive".into(),
            PlaylistPage {
                items: PagedList {
                    loaded_once: true,
                    next_offset: None,
                    ..Default::default()
                },
                ..PlaylistPage::default()
            },
        );
        let checks = crate::ui::widgets::PlaylistChecks::new(&app, &item);
        assert_eq!(checks.mark("drive"), Mark::Unchecked, "all rows read");
        app.backend.shutdown();
    }

    /// The player bar's heart applies its checks on Done, asking about each
    /// playlist it adds to, and a click outside drops them unapplied.
    #[test]
    fn the_hearts_done_applies_its_checks_and_a_click_outside_drops_them() {
        use crate::backend::ApiRequest;
        use egui::accesskit::{Action as AccessibleAction, Role};

        let (ctx, mut app) = accessible_app("heart-checklist-done");
        app.backend.set_offline(true);
        let owner = app.user_id().unwrap().to_string();
        let make = |id: &str, name: &str| Playlist {
            id: id.into(),
            uri: format!("spotify:playlist:{id}"),
            name: name.into(),
            owner: crate::api::models::Owner {
                id: Some(owner.clone()),
                ..Default::default()
            },
            ..Default::default()
        };
        app.library.playlists =
            Loadable::Loaded(vec![make("drive", "Night drive"), make("day", "Daylight")]);
        let click = |app: &mut App, label: &str, role: Role| {
            let tree = accessible_frame(&ctx, app, vec![]);
            let node = accessible_node(&tree, label, role);
            accessible_frame(
                &ctx,
                app,
                vec![accessible_action(node, AccessibleAction::Click, None)],
            );
            // A menu's first frame only measures it.
            accessible_frame(&ctx, app, vec![]);
        };
        // Neither playlist was read in full, so both are checks to make.
        let check_both = |app: &mut App| {
            click(app, "Add to playlist", Role::Button);
            click(app, "Night drive", Role::CheckBox);
            click(app, "Daylight", Role::CheckBox);
        };
        let asked = |app: &App| -> Vec<String> {
            app.backend
                .take_playlist_add_requests()
                .into_iter()
                .filter_map(|request| match request {
                    ApiRequest::CheckPlaylistDuplicates { playlist_id, .. } => Some(playlist_id),
                    _ => None,
                })
                .collect()
        };
        accessible_frame(&ctx, &mut app, vec![]);
        let now = app.now_playing().unwrap().uri;
        assert_eq!(app.is_saved(&now), Some(true));

        check_both(&mut app);
        // The margin at the bar's left edge, beside the cover.
        let outside = egui::pos2(6.0, 796.0);
        accessible_frame(
            &ctx,
            &mut app,
            pointer_click(outside, egui::PointerButton::Primary),
        );
        accessible_frame(&ctx, &mut app, vec![]);
        assert!(asked(&app).is_empty(), "a click outside changes nothing");

        // Kept checks would uncheck both here, and Done would ask nothing.
        check_both(&mut app);
        click(&mut app, "Done", Role::Button);
        assert_eq!(asked(&app), ["drive", "day"]);
        assert_eq!(app.is_saved(&now), Some(true), "it stays liked");
        app.backend.shutdown();
    }

    #[test]
    fn play_next_menus_queue_the_chosen_rows_only_for_local_playback() {
        for multiple in [false, true] {
            for local in [false, true] {
                let (ctx, mut app) = accessible_app(&format!("play-next-{multiple}-{local}"));
                let queued = seed_queued_songs(&mut app);
                if !local {
                    app.local.playback = crate::player::Playback::Stopped;
                    app.local_ready = false;
                }
                let songs: Vec<_> = if multiple { vec![2, 1, 2] } else { vec![2] }
                    .into_iter()
                    .map(|index| PlayableItem::Track(track(index)))
                    .collect();
                let draw = |app: &mut App, events| {
                    let mut output = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(760.0, 620.0),
                            )),
                            events,
                            ..Default::default()
                        },
                        |ui| {
                            if multiple {
                                crate::ui::widgets::picked_menu(ui, app, &songs, &[], None);
                            } else {
                                crate::ui::widgets::item_menu(ui, app, &songs[0], None, None);
                            }
                        },
                    );
                    output.textures_delta.clear();
                    output
                };
                draw(&mut app, vec![]);
                let output = draw(&mut app, vec![]);
                let node = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some("Play next"))
                    .expect("Play next is present in both song menus")
                    .1
                    .clone();
                assert_eq!(node.is_disabled(), !local);
                let bounds = node.bounds().unwrap();
                let pos = egui::pos2(
                    (bounds.x0 + 20.0) as f32,
                    ((bounds.y0 + bounds.y1) / 2.0) as f32,
                );
                app.actions.clear();
                draw(&mut app, pointer_click(pos, egui::PointerButton::Primary));
                assert_eq!(app.manual_queue, queued, "drawing only emits an action");
                if local {
                    let actions = std::mem::take(&mut app.actions);
                    assert!(
                        matches!(actions.as_slice(), [Action::PlayNext { items }] if items == &songs)
                    );
                    for action in actions {
                        app.apply(action, &ctx);
                    }
                    let expected: Vec<_> = songs
                        .iter()
                        .map(|item| item.uri().to_string())
                        .chain(queued)
                        .collect();
                    assert_eq!(app.manual_queue, expected);
                } else {
                    assert!(
                        app.actions.is_empty(),
                        "a disabled entry must not append remotely"
                    );
                }
                app.backend.shutdown();
            }
        }
    }

    #[test]
    fn playlist_submenu_keeps_typing_and_resets_after_the_parent_closes() {
        use egui::accesskit::{Action, Role};
        let (ctx, mut app) = accessible_app("playlist-submenu");
        app.backend.set_offline(true);
        let songs = vec![PlayableItem::Track(track(0))];
        let draw = |app: &mut App, show_parent: bool, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    if show_parent {
                        crate::ui::widgets::picked_menu(ui, app, &songs, &[], None);
                    }
                },
            );
            output.textures_delta.clear();
            output
        };
        let tree = draw(&mut app, true, vec![])
            .platform_output
            .accesskit_update
            .unwrap();
        let add = accessible_node(&tree, "Add to playlist", Role::Button);
        let open = || vec![accessible_action(add, Action::Click, None)];
        draw(&mut app, true, open());
        draw(&mut app, true, vec![]);
        let tree = draw(&mut app, true, vec![egui::Event::Text("night".into())])
            .platform_output
            .accesskit_update
            .unwrap();
        assert!(egui::Popup::is_any_open(&ctx));
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.value() == Some("night"))
        );
        accessible_node(&tree, "Late night focus", Role::Button);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Sunday morning"))
        );
        draw(
            &mut app,
            true,
            vec![keyboard(egui::Key::Escape, egui::Modifiers::NONE)],
        );
        assert!(!egui::Popup::is_any_open(&ctx));
        // A closed outer context menu no longer draws its submenu at all.
        draw(&mut app, false, vec![]);
        draw(&mut app, true, vec![]);
        draw(&mut app, true, open());
        let tree = draw(&mut app, true, vec![])
            .platform_output
            .accesskit_update
            .unwrap();
        accessible_node(&tree, "Sunday morning", Role::Button);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.value() == Some("night"))
        );
        app.backend.shutdown();
    }

    #[test]
    fn playlist_submenu_stays_open_for_a_click_in_its_filter_and_grows_back_when_cleared() {
        use egui::accesskit::{Action, Role};
        let (ctx, mut app) = accessible_app("playlist-submenu-click");
        app.backend.set_offline(true);
        let songs = vec![PlayableItem::Track(track(0))];
        let draw = |app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::widgets::picked_menu(ui, app, &songs, &[], None),
            );
            output.textures_delta.clear();
            output.platform_output.accesskit_update.unwrap()
        };
        let tree = draw(&mut app, vec![]);
        let add = accessible_node(&tree, "Add to playlist", Role::Button);
        draw(&mut app, vec![accessible_action(add, Action::Click, None)]);
        draw(&mut app, vec![]);
        let tree = draw(&mut app, vec![]);
        let field = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Filter playlists"))
            .and_then(|(_, node)| node.bounds())
            .expect("the playlist filter");
        let center = egui::pos2(
            ((field.x0 + field.x1) / 2.0) as f32,
            ((field.y0 + field.y1) / 2.0) as f32,
        );
        let height = || {
            let layer = ctx.layer_id_at(center).expect("the submenu's layer");
            ctx.memory(|memory| memory.area_rect(layer.id))
                .expect("the submenu's area")
                .height()
        };
        let full = height();

        // #when the filter field is clicked
        draw(
            &mut app,
            pointer_click(center, egui::PointerButton::Primary),
        );
        let tree = draw(&mut app, vec![]);

        // #then the submenu stays open with every playlist
        assert!(egui::Popup::is_any_open(&ctx));
        accessible_node(&tree, "Sunday morning", Role::Button);

        // #when a filter narrows the list and is then cleared
        draw(&mut app, vec![egui::Event::Text("night".into())]);
        draw(&mut app, vec![]);
        assert!(height() < full, "the filter shrinks the submenu");
        draw(
            &mut app,
            vec![keyboard(egui::Key::Backspace, egui::Modifiers::NONE); 5],
        );
        draw(&mut app, vec![]);
        draw(&mut app, vec![]);

        // #then the submenu grows back to its full height
        assert_eq!(height(), full);
        app.backend.shutdown();
    }

    #[test]
    fn an_empty_search_half_is_not_reported_as_no_results_while_the_other_waits_or_fails() {
        let (ctx, mut app) = accessible_app("partial-search-status");
        app.search.committed = "new query".into();
        app.search.results = Loadable::Loaded(SearchResults::default());
        for pending in [true, false] {
            app.search.catalogue_pending = pending;
            app.search.error = (!pending).then(|| "Search: network error".into());
            search_frame(&ctx, &mut app, vec![]);
            let text = search_frame(&ctx, &mut app, vec![]);
            assert!(!text.iter().any(|(label, _)| label.contains("No results")));
            if !pending {
                assert!(
                    text.iter()
                        .any(|(label, _)| label.contains("network error"))
                );
            }
        }
        app.backend.shutdown();
    }

    fn search_frame(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
    ) -> Vec<(String, egui::Rect)> {
        view_frame(ctx, app, events, crate::ui::search::show)
    }

    #[test]
    fn playing_artist_links_work_before_web_metadata_and_after_focus_returns() {
        use crate::player::{LocalState, LocalTrack, Playback};

        let (ctx, mut app) = accessible_app("playing-artist-links");
        let artists = vec![
            ArtistRef {
                id: Some("first".into()),
                name: "Tyler, the Creator".into(),
                uri: Some("spotify:artist:first".into()),
            },
            ArtistRef {
                id: Some("guest".into()),
                name: "Guest".into(),
                uri: Some("spotify:artist:guest".into()),
            },
        ];
        app.local = LocalState {
            playback: Playback::Playing,
            track: Some(LocalTrack {
                uri: "spotify:track:uncached".into(),
                title: "Song".into(),
                artists: artists.clone(),
                duration_ms: 200_000,
                ..LocalTrack::default()
            }),
            ..LocalState::default()
        };
        app.track_cache.clear();
        let view = crate::ui::player_bar::show;
        for cached in [false, true] {
            if cached {
                // A partial response must not turn working links back into text.
                app.track_cache.insert("uncached".into(), Track::default());
            }
            view_frame(&ctx, &mut app, vec![], view);
            let text = view_frame(&ctx, &mut app, vec![], view);
            for artist in &artists {
                let pos = text
                    .iter()
                    .find(|(text, _)| text == &artist.name)
                    .unwrap()
                    .1
                    .center();
                app.actions.clear();
                view_frame(
                    &ctx,
                    &mut app,
                    pointer_click(pos, egui::PointerButton::Primary),
                    view,
                );
                assert!(
                    matches!(app.actions.as_slice(), [Action::Open(Page::Artist(id))] if Some(id) == artist.id.as_ref())
                );
            }
            view_frame(
                &ctx,
                &mut app,
                vec![egui::Event::WindowFocused(false), egui::Event::PointerGone],
                view,
            );
            view_frame(&ctx, &mut app, vec![egui::Event::WindowFocused(true)], view);
        }
        app.backend.shutdown();
    }

    #[test]
    fn compact_track_rows_leave_a_gap_before_the_added_date_separator() {
        fn rows(app: &mut App, ui: &mut egui::Ui) {
            use crate::model::RowContext;
            use crate::ui::widgets::{TrackRow, track_row};
            ui.set_max_width(520.0);
            for count in 1..=2 {
                let mut song = track(count);
                song.artists = (0..count)
                    .map(|index| ArtistRef {
                        id: Some(format!("artist-{index}")),
                        name: format!("Artist {index}"),
                        uri: Some(format!("spotify:artist:artist-{index}")),
                    })
                    .collect();
                let item = PlayableItem::Track(song);
                let context = RowContext::Uris(std::sync::Arc::from([item.uri().to_owned()]));
                track_row(
                    ui,
                    app,
                    TrackRow {
                        index: count,
                        number: Some(count),
                        item: &item,
                        context: &context,
                        show_cover: false,
                        show_album: false,
                        added_at: Some("2026-01-01T00:00:00Z"),
                        added_by: None,
                        show_added_by: false,
                        show_plays: false,
                        compact: false,
                        thin: true,
                        shift: 0.0,
                        picked: false,
                        picked_songs: &[],
                        picked_positions: &[],
                        position: None,
                    },
                );
            }
        }
        let (ctx, mut app) = accessible_app("compact-artist-date-gap");
        for palette in [
            crate::theme::Palette::dark(),
            crate::theme::Palette::light(),
        ] {
            app.palette = palette;
            crate::theme::apply(&ctx, &palette);
            view_frame(&ctx, &mut app, vec![], rows);
            let text = view_frame(&ctx, &mut app, vec![], rows);
            for (label, artist) in text
                .iter()
                .filter(|(label, _)| label.starts_with("Artist "))
            {
                let separator = text.iter().find(|(label, rect)| {
                    label == "•"
                        && (rect.center().y - artist.center().y).abs() < 3.0
                        && rect.left() >= artist.right() - 0.1
                });
                // Only the last artist in each row borders the date separator.
                if let Some((_, separator)) = separator {
                    let gap = separator.left() - artist.right();
                    assert!(
                        gap >= 5.9,
                        "{label} needs a gap before the date bullet, got {gap}"
                    );
                }
            }
            assert_eq!(text.iter().filter(|(label, _)| label == "•").count(), 4);
        }
        app.backend.shutdown();
    }

    fn view_frame(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        view: fn(&mut App, &mut egui::Ui),
    ) -> Vec<(String, egui::Rect)> {
        view_frame_sized(ctx, app, events, view, egui::vec2(1280.0, 2200.0))
    }

    fn view_frame_sized(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        view: fn(&mut App, &mut egui::Ui),
        size: egui::Vec2,
    ) -> Vec<(String, egui::Rect)> {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                events,
                ..Default::default()
            },
            |ui| view(app, ui),
        );
        output.textures_delta.clear();
        fn walk(shape: &egui::epaint::Shape, text: &mut Vec<(String, egui::Rect)>) {
            match shape {
                egui::epaint::Shape::Text(shape) => text.push((
                    shape.galley.job.text.clone(),
                    shape.galley.rect.translate(shape.pos.to_vec2()),
                )),
                egui::epaint::Shape::Vec(shapes) => {
                    shapes.iter().for_each(|shape| walk(shape, text));
                }
                _ => {}
            }
        }
        let mut text = Vec::new();
        for shape in &output.shapes {
            walk(&shape.shape, &mut text);
        }
        text
    }

    #[test]
    fn loading_collections_keep_their_hero_layout() {
        fn playlist(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::collection::playlist(app, ui, "pl1");
        }
        fn album(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::collection::album(app, ui, "alb0");
        }
        fn artist(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::artist::show(app, ui, "art0");
        }
        fn show(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::show::show(app, ui, "sh0");
        }

        let (ctx, mut app) = accessible_app("loading-collections");
        app.playlist_pages.remove("pl1");
        app.album_pages.remove("alb0");
        app.artist_pages.remove("art0");
        app.show_pages.remove("sh0");
        app.library.playlists = Loadable::Loaded(Vec::new());
        app.library.albums.items.clear();
        app.library.artists.items.clear();
        app.library.shows.items.clear();
        app.search.results.get_mut().unwrap().albums = None;
        for (title, detail, view) in [
            (
                "Late night focus",
                "Carmine",
                playlist as fn(&mut App, &mut egui::Ui),
            ),
            ("Fragments", "Bonobo", album),
            ("Bonobo", "electronic, downtempo, ambient", artist),
            ("Rework", "37signals", show),
        ] {
            let painted = view_frame(&ctx, &mut app, vec![], view);
            let title = painted.iter().find(|(text, _)| text == title).unwrap().1;
            assert!(painted.iter().any(|(text, _)| text.contains(detail)));
            // The songs to come are skeleton rows, which screen readers
            // hear as Loading…, below the known header (#674).
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 2200.0),
                    )),
                    ..Default::default()
                },
                |ui| view(&mut app, ui),
            );
            output.textures_delta.clear();
            let tree = output.platform_output.accesskit_update.unwrap();
            let loading = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.label() == Some("Loading…") || node.value() == Some("Loading…")
                })
                .and_then(|(_, node)| node.bounds())
                .expect("a loading skeleton");
            assert!(loading.y0 as f32 > title.bottom());
            assert!(
                !painted.iter().any(|(text, _)| text == "Loading…"),
                "no spinner row"
            );
        }
        app.backend.shutdown();
    }

    /// Home's shelves wait in card-shaped skeletons, not a spinner row, each
    /// heard as Loading… (#674).
    #[test]
    fn loading_home_shelves_show_card_skeletons() {
        let (ctx, mut app) = accessible_app("loading-home");
        app.home.recently_played = Loadable::Loading;
        app.home.top_artists = Loadable::Loading;
        app.home.made_for_you = Loadable::Loading;
        let view = crate::ui::home::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        assert!(!painted.iter().any(|(text, _)| text == "Loading…"));
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1280.0, 2200.0),
                )),
                ..Default::default()
            },
            |ui| view(&mut app, ui),
        );
        output.textures_delta.clear();
        let tree = output.platform_output.accesskit_update.unwrap();
        let skeletons = tree
            .nodes
            .iter()
            .filter(|(_, node)| {
                node.value() == Some("Loading…") || node.label() == Some("Loading…")
            })
            .count();
        assert_eq!(skeletons, 3, "one per loading shelf");
        app.backend.shutdown();
    }

    #[test]
    fn failed_collections_keep_known_metadata_above_retry() {
        fn playlist(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::collection::playlist(app, ui, "pl1");
        }
        fn album(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::collection::album(app, ui, "alb0");
        }
        fn artist(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::artist::show(app, ui, "art0");
        }
        fn show(app: &mut App, ui: &mut egui::Ui) {
            crate::ui::show::show(app, ui, "sh0");
        }

        let (ctx, mut app) = accessible_app("failed-collections");
        app.playlist_pages.get_mut("pl1").unwrap().playlist =
            Loadable::Failed("Connection interrupted".into());
        app.album_pages.get_mut("alb0").unwrap().album =
            Loadable::Failed("Connection interrupted".into());
        app.artist_pages.get_mut("art0").unwrap().artist =
            Loadable::Failed("Connection interrupted".into());
        app.show_pages.get_mut("sh0").unwrap().show =
            Loadable::Failed("Connection interrupted".into());

        for (title, detail, view) in [
            (
                "Late night focus",
                "Carmine",
                playlist as fn(&mut App, &mut egui::Ui),
            ),
            ("Fragments", "Bonobo", album),
            ("Bonobo", "electronic, downtempo, ambient", artist),
            ("Rework", "37signals", show),
        ] {
            let painted = view_frame(&ctx, &mut app, vec![], view);
            let title = painted.iter().find(|(text, _)| text == title).unwrap().1;
            assert!(painted.iter().any(|(text, _)| text.contains(detail)));
            assert!(
                painted
                    .iter()
                    .any(|(text, _)| text == "Connection interrupted")
            );
            let retry = painted.iter().find(|(text, _)| text == "Retry").unwrap().1;
            assert!(retry.top() > title.bottom());
        }
        app.backend.shutdown();
    }

    fn pointer_click(pos: egui::Pos2, button: egui::PointerButton) -> Vec<egui::Event> {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerButton {
                pos,
                button,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    }

    /// Painted labels of a freshly drawn menu, for asserting which entries
    /// a context offers.
    fn menu_text(output: &egui::FullOutput) -> Vec<(String, egui::Rect)> {
        fn walk(shape: &egui::epaint::Shape, text: &mut Vec<(String, egui::Rect)>) {
            match shape {
                egui::epaint::Shape::Text(shape) => text.push((
                    shape.galley.job.text.clone(),
                    shape.galley.rect.translate(shape.pos.to_vec2()),
                )),
                egui::epaint::Shape::Vec(shapes) => {
                    shapes.iter().for_each(|shape| walk(shape, text));
                }
                _ => {}
            }
        }
        let mut text = Vec::new();
        for shape in &output.shapes {
            walk(&shape.shape, &mut text);
        }
        text
    }

    /// Settings offers Off, the window and the tray for a start at login,
    /// and a pick is an action; a run that cannot register shows no row.
    #[test]
    fn start_with_computer_offers_three_choices_and_asks_for_the_pick() {
        use fastframe_autostart::{Start, State};
        let (ctx, mut app) = accessible_app("start-with-computer");
        app.backend.shutdown();
        ctx.data_mut(|data| {
            data.insert_temp(egui::Id::new("settings-filter"), "Start with".to_string())
        });
        for _ in 0..3 {
            view_frame(&ctx, &mut app, vec![], crate::ui::settings::show);
        }
        let painted = view_frame(&ctx, &mut app, vec![], crate::ui::settings::show);
        sidebar_text(&painted, "Start with your computer");
        sidebar_text(&painted, "Off");
        sidebar_text(&painted, "With the window");
        let (in_tray, starting_in_tray) = if cfg!(target_os = "macos") {
            ("In the menu bar", "Starting in the menu bar")
        } else {
            ("In the tray", "Starting in the tray")
        };
        let tray = sidebar_text(&painted, in_tray).center();
        view_frame(
            &ctx,
            &mut app,
            pointer_click(tray, egui::PointerButton::Primary),
            crate::ui::settings::show,
        );
        assert!(matches!(
            app.actions.as_slice(),
            [Action::SetStartWithComputer(Some(Start::Hidden))]
        ));
        app.actions.clear();

        // Off keeps the tray out of reach: the row says the window opens.
        app.autostart = Some(State::On(Start::Hidden));
        app.settings.keep_playing_in_background = false;
        let painted = view_frame(&ctx, &mut app, vec![], crate::ui::settings::show);
        assert!(painted.iter().any(|(text, _)| {
            text.starts_with(&format!(
                "{starting_in_tray} needs Keep music playing when the window closes."
            ))
        }));

        app.autostart = Some(State::Unsupported);
        let painted = view_frame(&ctx, &mut app, vec![], crate::ui::settings::show);
        assert!(
            !painted.iter().any(|(text, _)| text == in_tray),
            "nothing to offer where nothing can be registered"
        );
    }

    /// The painted rect of a sidebar label, for pointer tests against the
    /// sidebar's private rows.
    fn sidebar_text(painted: &[(String, egui::Rect)], label: &str) -> egui::Rect {
        painted
            .iter()
            .find(|(text, _)| text == label)
            .map(|(_, rect)| *rect)
            .unwrap_or_else(|| panic!("missing sidebar text {label:?}"))
    }

    fn double_click(pos: egui::Pos2) -> [Vec<egui::Event>; 2] {
        [
            pointer_click(pos, egui::PointerButton::Primary),
            pointer_click(pos, egui::PointerButton::Primary),
        ]
    }

    fn played_contexts(app: &App) -> Vec<String> {
        app.actions
            .iter()
            .filter_map(|action| match action {
                Action::PlayContext { uri, .. } => Some(uri.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn song_menu_copies_the_title_and_artists_from_a_submenu() {
        let (ctx, mut app) = accessible_app("song-menu-copy-details");
        let mut item = track(0);
        item.artists = vec![
            ArtistRef {
                id: Some("first".into()),
                name: "Ween".into(),
                uri: Some("spotify:artist:first".into()),
            },
            ArtistRef {
                id: None,
                name: "Guest".into(),
                uri: None,
            },
        ];
        let title = item.name.clone();
        app.search.results = Loadable::Loaded(SearchResults {
            tracks: Some(page(vec![item])),
            ..Default::default()
        });
        for (label, expected) in [
            ("Song title", title.clone()),
            ("Artist", "Ween, Guest".to_string()),
            ("Artist and song title", format!("Ween, Guest - {title}")),
        ] {
            search_frame(&ctx, &mut app, vec![]);
            let text = search_frame(&ctx, &mut app, vec![]);
            let pos = text
                .iter()
                .find(|(text, _)| text == &title)
                .expect("top result title")
                .1
                .center();
            search_frame(
                &ctx,
                &mut app,
                pointer_click(pos, egui::PointerButton::Secondary),
            );
            let text = search_frame(&ctx, &mut app, vec![]);
            assert!(
                !text.iter().any(|(text, _)| text == "Song title"),
                "the copy entries stay inside the submenu"
            );
            let details = text
                .iter()
                .find(|(text, _)| text == "Copy details")
                .expect("the song menu offers Copy details")
                .1
                .center();
            search_frame(
                &ctx,
                &mut app,
                pointer_click(details, egui::PointerButton::Primary),
            );
            search_frame(&ctx, &mut app, vec![]);
            let output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 2200.0),
                    )),
                    ..Default::default()
                },
                |ui| crate::ui::search::show(&mut app, ui),
            );
            let text = menu_text(&output);
            let tree = output.platform_output.accesskit_update.unwrap();
            // A menu item spans its menu, so its width is the menu's.
            let width = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label() == Some("Song title")
                })
                .and_then(|(_, node)| node.bounds())
                .expect("the submenu's first entry")
                .width();
            assert!(width <= 300.0, "the submenu is {width} points wide");
            let entry = text
                .iter()
                .rev()
                .find(|(text, _)| text == label)
                .unwrap_or_else(|| panic!("Copy details is missing {label}"))
                .1
                .center();
            app.actions.clear();
            search_frame(
                &ctx,
                &mut app,
                pointer_click(entry, egui::PointerButton::Primary),
            );
            assert!(
                matches!(app.actions.as_slice(), [Action::CopyText(text)] if text == &expected),
                "{label}: {:?}",
                app.actions
            );
        }
        app.backend.shutdown();
    }

    /// A local file's title comes from its URI and can be empty, so each
    /// entry needs only what it copies.
    #[test]
    fn copy_details_offers_only_what_the_song_has() {
        for (case, title, artist, offered) in [
            ("no-title", "", "Ween", &["Artist"][..]),
            ("no-artist", "Mutilated Lips", "", &["Song title"][..]),
            ("neither", "", "", &[][..]),
        ] {
            let (ctx, mut app) = accessible_app(&format!("song-menu-copy-{case}"));
            let mut song = track(0);
            song.name = title.into();
            song.artists = (!artist.is_empty())
                .then(|| ArtistRef {
                    name: artist.into(),
                    ..Default::default()
                })
                .into_iter()
                .collect();
            let song = PlayableItem::Track(song);
            let draw = |app: &mut App, events| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1280.0, 800.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| crate::ui::widgets::item_menu(ui, app, &song, None, None),
                );
                output.textures_delta.clear();
                menu_text(&output)
            };
            draw(&mut app, vec![]);
            let text = draw(&mut app, vec![]);
            let details = text.iter().find(|(text, _)| text == "Copy details");
            let Some((_, details)) = details else {
                assert!(offered.is_empty(), "{case}: Copy details is missing");
                app.backend.shutdown();
                continue;
            };
            assert!(
                !offered.is_empty(),
                "{case}: Copy details has nothing to copy"
            );
            draw(
                &mut app,
                pointer_click(details.center(), egui::PointerButton::Primary),
            );
            draw(&mut app, vec![]);
            let text = draw(&mut app, vec![]);
            for label in ["Song title", "Artist", "Artist and song title"] {
                assert_eq!(
                    text.iter().any(|(text, _)| text == label),
                    offered.contains(&label),
                    "{case}: {label}"
                );
            }
            let entry = text
                .iter()
                .find(|(text, _)| text == offered[0])
                .expect("the submenu's entry")
                .1
                .center();
            app.actions.clear();
            draw(&mut app, pointer_click(entry, egui::PointerButton::Primary));
            let expected = if title.is_empty() { artist } else { title };
            assert!(
                matches!(app.actions.as_slice(), [Action::CopyText(text)] if text == expected),
                "{case}: {:?}",
                app.actions
            );
            app.backend.shutdown();
        }
    }

    #[test]
    fn song_top_result_artist_name_opens_the_available_profile_or_the_album() {
        for artist_id in [Some("ween"), None] {
            let (ctx, mut app) = accessible_app("top-result-song-artist");
            let mut item = track(0);
            let album_id = item.album.as_ref().unwrap().id.clone();
            item.artists = vec![ArtistRef {
                id: artist_id.map(str::to_string),
                name: "Ween".into(),
                uri: artist_id.map(|id| format!("spotify:artist:{id}")),
            }];
            app.search.results = Loadable::Loaded(SearchResults {
                tracks: Some(page(vec![item])),
                ..Default::default()
            });
            search_frame(&ctx, &mut app, vec![]);
            let text = search_frame(&ctx, &mut app, vec![]);
            let songs_left = text
                .iter()
                .filter(|(text, _)| text == "Songs")
                .max_by(|(_, left), (_, right)| left.left().total_cmp(&right.left()))
                .expect("songs heading")
                .1
                .left();
            let artist = text
                .iter()
                .find(|(text, rect)| text == "Ween" && rect.left() < songs_left)
                .expect("top result artist name");
            app.actions.clear();
            search_frame(
                &ctx,
                &mut app,
                pointer_click(
                    egui::pos2(artist.1.right() - 4.0, artist.1.center().y),
                    egui::PointerButton::Primary,
                ),
            );
            if artist_id.is_some() {
                assert!(
                    matches!(app.actions.as_slice(), [Action::Open(Page::Artist(id))] if id == "ween")
                );
            } else {
                assert!(
                    matches!(app.actions.as_slice(), [Action::Open(Page::Album(id))] if id == &album_id)
                );
            }
            app.backend.shutdown();
        }
    }

    fn check_card_menu(
        app: &mut App,
        ctx: &egui::Context,
        view: fn(&mut App, &mut egui::Ui),
        section: &str,
        title: &str,
        uri: &str,
        labels: &[&str],
    ) {
        view_frame(ctx, app, vec![], view);
        let text = view_frame(ctx, app, vec![], view);
        let below = text
            .iter()
            .find(|(text, _)| text == section)
            .unwrap()
            .1
            .bottom();
        let pos = text
            .iter()
            .find(|(text, rect)| text == title && rect.top() >= below)
            .unwrap_or_else(|| panic!("{title} in {section}"))
            .1
            .center();
        let earlier_card = text
            .iter()
            .find(|(_, rect)| (rect.center().y - pos.y).abs() < 1.0 && rect.center().x < pos.x)
            .map(|(_, rect)| rect.center());
        app.actions.clear();
        view_frame(
            ctx,
            app,
            pointer_click(pos, egui::PointerButton::Secondary),
            view,
        );
        // A hovered Play button must not change which item owns the open menu.
        if let Some(pos) = earlier_card {
            view_frame(ctx, app, vec![egui::Event::PointerMoved(pos)], view);
        }
        let text = view_frame(ctx, app, vec![], view);
        // Drawing may request page metadata, independently of the menu gesture.
        app.actions
            .retain(|action| !matches!(action, Action::RequestPlayCounts(_)));
        assert!(
            app.actions.is_empty(),
            "right-click must not navigate or play"
        );
        for label in labels {
            assert!(
                text.iter().any(|(text, _)| text == label),
                "{section}: missing {label}"
            );
        }
        let copy = text
            .iter()
            .find(|(text, _)| text == "Copy link")
            .unwrap_or_else(|| panic!("{section}: no menu for {title}"));
        view_frame(
            ctx,
            app,
            pointer_click(copy.1.center(), egui::PointerButton::Primary),
            view,
        );
        app.actions
            .retain(|action| !matches!(action, Action::RequestPlayCounts(_)));
        assert!(matches!(app.actions.as_slice(), [Action::CopyLink(link)] if link == uri));
    }

    #[test]
    fn library_and_discography_cards_open_their_own_menus() {
        for (page, section, title, uri, labels) in [
            (
                Page::Albums,
                "Albums",
                album(1).name,
                album(1).uri,
                vec!["Add to queue", "Shuffle play"],
            ),
            (
                Page::Artists,
                "Artists",
                artist(1).name,
                artist(1).uri,
                vec!["Follow"],
            ),
            (
                Page::Podcasts,
                "Podcasts",
                show(1).name,
                show(1).uri,
                vec!["Add to Your Library"],
            ),
            (
                Page::Artist("art0".into()),
                "Discography",
                album(1).name,
                album(1).uri,
                vec!["Add to queue", "Shuffle play"],
            ),
            (
                Page::Artist("art0".into()),
                "Fans also like",
                artist(2).name,
                artist(2).uri,
                vec!["Follow"],
            ),
        ] {
            let (ctx, mut app) = accessible_app(&format!("library-card-{section}"));
            app.open(page);
            fn view(app: &mut App, ui: &mut egui::Ui) {
                if let Page::Artist(id) = app.page().clone() {
                    crate::ui::artist::show(app, ui, &id);
                } else {
                    crate::ui::library::show(app, ui, app.page().clone());
                }
            }
            check_card_menu(&mut app, &ctx, view, section, &title, &uri, &labels);
            app.backend.shutdown();
        }
    }

    #[test]
    fn a_search_result_song_plays_on_its_own_not_the_other_results() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("search-song-plays-alone");
        app.open(Page::Search);
        app.search.filter = SearchFilter::Songs;
        let Loadable::Loaded(results) = &app.search.results else {
            panic!("demo search results");
        };
        let songs = results.tracks.as_ref().expect("demo songs").items.clone();
        assert!(songs.len() > 2, "the demo needs several results");
        let chosen = PlayableItem::Track(songs[1].clone());
        let label = format!("Play {}, {}", chosen.name(), chosen.subtitle());
        // Only the search view: its actions stay queued instead of being applied.
        let render = |app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 2200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::search::show(app, ui),
            );
            output.textures_delta.clear();
            output
                .platform_output
                .accesskit_update
                .expect("screen-reader tree")
        };
        render(&mut app, vec![]);
        let tree = render(&mut app, vec![]);
        let row = accessible_node(&tree, &label, Role::Button);
        app.actions.clear();
        render(
            &mut app,
            vec![accessible_action(row, AccessibleAction::Click, None)],
        );
        let played = app.actions.iter().find_map(|action| match action {
            crate::model::Action::PlayFromRow {
                context: RowContext::Uris(uris),
                uri,
                ..
            } => Some((uris.to_vec(), uri.clone())),
            _ => None,
        });
        assert_eq!(
            played,
            Some((vec![chosen.uri().to_string()], chosen.uri().to_string())),
            "a search result is not a queue of the other results"
        );
        app.backend.shutdown();
    }

    #[test]
    fn search_shelves_and_filtered_grids_open_item_menus() {
        for (filter, title, uri, labels) in [
            (
                SearchFilter::Artists,
                artist(1).name,
                artist(1).uri,
                vec!["Follow"],
            ),
            (
                SearchFilter::Albums,
                album(0).name,
                album(0).uri,
                vec!["Add to queue", "Add to Your Library"],
            ),
            (
                SearchFilter::Playlists,
                playlist(1).name,
                playlist(1).uri,
                vec!["Edit details", "Delete"],
            ),
            (
                SearchFilter::Podcasts,
                show(0).name,
                show(0).uri,
                vec!["Add to Your Library"],
            ),
        ] {
            for selected in [SearchFilter::All, filter] {
                let (ctx, mut app) =
                    accessible_app(&format!("search-card-{selected:?}-{filter:?}"));
                app.saved.clear();
                app.search.filter = selected;
                // Keep matching artist subtitles in song rows out of the card lookup.
                if let Loadable::Loaded(results) = &mut app.search.results {
                    results.tracks = None;
                    results.episodes = None;
                }
                let section = if selected == SearchFilter::All {
                    filter.label(crate::i18n::Locale::English)
                } else {
                    "All".into()
                };
                check_card_menu(
                    &mut app,
                    &ctx,
                    crate::ui::search::show,
                    &section,
                    &title,
                    &uri,
                    &labels,
                );
                app.backend.shutdown();
            }
        }
    }

    #[test]
    fn home_json_hides_only_the_chosen_recommendation_shelves() {
        let (ctx, mut app) = accessible_app("home-json-visibility");
        let view = crate::ui::home::show;
        for (made_for_you, recommendations) in
            [(true, true), (false, true), (true, false), (false, false)]
        {
            app.settings.home = serde_json::from_str(&format!(
                r#"{{"made_for_you":{{"visible":{made_for_you}}},"recommendations":{{"visible":{recommendations}}}}}"#
            )).unwrap();
            let size = egui::vec2(1280.0, 3200.0);
            view_frame_sized(&ctx, &mut app, vec![], view, size);
            let text = view_frame_sized(&ctx, &mut app, vec![], view, size);
            for (label, expected) in [
                ("Made for you", made_for_you),
                ("Your top mixes", made_for_you),
                ("New releases", true),
                ("Recommended for you", recommendations),
                ("Liked Songs", true),
                ("Recently played", true),
                ("Your top artists", true),
                ("Your top songs", true),
            ] {
                assert_eq!(
                    text.iter().any(|(text, _)| text == label),
                    expected,
                    "{label}, made_for_you={made_for_you}, recommendations={recommendations}"
                );
            }
        }
        app.backend.shutdown();
    }

    /// #907: the New releases page lists the releases' songs, newest
    /// release first, under a Play button.
    #[test]
    fn the_new_releases_page_lists_the_songs_of_the_releases() {
        let (ctx, mut app) = accessible_app("new-releases-page");
        let view = crate::ui::new_releases::page;
        view_frame(&ctx, &mut app, vec![], view);
        let text = view_frame(&ctx, &mut app, vec![], view);
        let first = app.home.new_release_tracks.get().unwrap()[0].clone();
        assert_eq!(
            first.album.as_ref().map(|album| album.uri.as_str()),
            app.home
                .new_releases
                .get()
                .unwrap()
                .first()
                .map(|album| album.uri.as_str())
        );
        for label in ["New releases", first.name.as_str()] {
            assert!(
                text.iter().any(|(text, _)| text == label),
                "{label} missing from {:?}",
                text.iter().map(|(text, _)| text).collect::<Vec<_>>()
            );
        }
        app.backend.shutdown();
    }

    #[test]
    fn home_liked_songs_tile_keeps_its_primary_click_only() {
        let (ctx, mut app) = accessible_app("home-liked-tile");
        let view = crate::ui::home::show;
        view_frame(&ctx, &mut app, vec![], view);
        let text = view_frame(&ctx, &mut app, vec![], view);
        let pos = text
            .iter()
            .find(|(text, _)| text == "Liked Songs")
            .unwrap()
            .1
            .center();
        app.actions.clear();
        view_frame(
            &ctx,
            &mut app,
            pointer_click(pos, egui::PointerButton::Secondary),
            view,
        );
        let text = view_frame(&ctx, &mut app, vec![], view);
        assert!(app.actions.is_empty());
        assert!(!text.iter().any(|(text, _)| text == "Copy link"));
        view_frame(
            &ctx,
            &mut app,
            pointer_click(pos, egui::PointerButton::Primary),
            view,
        );
        assert!(matches!(
            app.actions.as_slice(),
            [Action::Open(Page::LikedSongs)]
        ));
        app.backend.shutdown();
    }

    fn frame_events(ctx: &egui::Context, app: &mut App, events: Vec<egui::Event>) {
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            events,
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| {
            app.frame_ui(ui);
        });
        output.textures_delta.clear();
    }

    /// Every line of the sign-in card is drawn whole, without a scroll bar,
    /// in each state at the smallest main window and a usual one (#782).
    #[test]
    fn the_sign_in_card_fits_what_it_shows_without_scrolling() {
        let (ctx, mut app) = accessible_app("sign-in-card");
        app.backend.set_offline(true);
        let states = [
            AuthStatus::SignedOut,
            AuthStatus::WaitingForBrowser {
                url: "https://accounts.spotify.com/authorize".into(),
            },
            AuthStatus::Connecting,
            // The longest explanations, beside the setup steps.
            AuthStatus::Refused(crate::backend::SignInProblem::SharedRetired),
            AuthStatus::Refused(crate::backend::SignInProblem::AccountNotAllowed),
            AuthStatus::Refused(crate::backend::SignInProblem::MalformedClientId),
        ];
        for size in [
            egui::vec2(
                crate::window::MAIN_MIN_SIZE[0],
                crate::window::MAIN_MIN_SIZE[1],
            ),
            egui::vec2(1280.0, 800.0),
        ] {
            let draw = |app: &mut App, events: Vec<egui::Event>| {
                let mut events = Some(events);
                let mut output = None;
                for _ in 0..6 {
                    let mut frame = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                            events: events.take().unwrap_or_default(),
                            ..Default::default()
                        },
                        |ui| app.frame_ui(ui),
                    );
                    frame.textures_delta.clear();
                    output = Some(frame);
                }
                output.unwrap()
            };
            // The card's title, and that no text in the card is cut off.
            let title = |output: &egui::FullOutput, state: &AuthStatus| {
                let mut title = None;
                for clipped in &output.shapes {
                    if let egui::epaint::Shape::Text(text) = &clipped.shape {
                        let rect = text.galley.rect.translate(text.pos.to_vec2());
                        assert!(
                            clipped.clip_rect.expand(0.5).contains_rect(rect),
                            "{:?} is cut off at {size:?} in {state:?}",
                            text.galley.job.text
                        );
                        if text.galley.job.text == "Spotifrost" {
                            title = Some(rect);
                        }
                    }
                }
                title.unwrap_or_else(|| panic!("the card was drawn in {state:?}"))
            };
            for state in &states {
                app.auth = state.clone();
                app.user = None;
                let before = title(&draw(&mut app, Vec::new()), state);
                // A card with nothing hidden has nothing to scroll, and so
                // shows no scroll bar.
                let wheel = vec![
                    egui::Event::PointerMoved(before.center()),
                    egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, -400.0),
                        modifiers: egui::Modifiers::NONE,
                        phase: egui::TouchPhase::Move,
                    },
                ];
                draw(&mut app, wheel);
                let after = title(&draw(&mut app, Vec::new()), state);
                assert_eq!(before, after, "the card scrolled at {size:?} in {state:?}");
                // Back to the top for the next state.
                let wheel = vec![egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0.0, 400.0),
                    modifiers: egui::Modifiers::NONE,
                    phase: egui::TouchPhase::Move,
                }];
                draw(&mut app, wheel);
                draw(&mut app, Vec::new());
            }
        }
        app.backend.shutdown();
    }

    /// A toast is wide enough to avoid wrapping every word.
    #[test]
    fn a_toast_is_wide_enough_to_read() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-toast-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 800.0),
            )),
            ..Default::default()
        };
        // A short toast first: the toasts area remembers its size, and a
        // long toast used to inherit the narrow width and wrap inside it.
        app.toast("Saved");
        for _ in 0..2 {
            frame(&ctx, &mut app);
        }
        app.toasts.clear();
        app.toast("Wish You Were Here added to queue");
        // Two frames: an area sizes itself on its first one.
        let mut first = ctx.run_ui(input.clone(), |ui| app.frame_ui(ui));
        first.textures_delta.clear();
        let mut output = ctx.run_ui(input, |ui| app.frame_ui(ui));
        output.textures_delta.clear();

        fn widest_toast_text(shape: &egui::epaint::Shape) -> Option<f32> {
            match shape {
                egui::epaint::Shape::Text(text)
                    if text.galley.job.text.contains("Wish You Were Here") =>
                {
                    Some(text.galley.rect.width())
                }
                egui::epaint::Shape::Vec(shapes) => {
                    shapes.iter().filter_map(widest_toast_text).next()
                }
                _ => None,
            }
        }
        let width = output
            .shapes
            .iter()
            .filter_map(|clipped| widest_toast_text(&clipped.shape))
            .next()
            .expect("the toast's text is painted");
        assert!(
            width > 150.0,
            "one word per line again: the toast text is only {width}px wide"
        );
        app.backend.shutdown();
    }

    /// The shortcuts are longer than a small window is tall, so the
    /// dialog scrolls them rather than running off the bottom with the
    /// Done button somewhere past the edge of the screen.
    #[test]
    fn the_shortcuts_dialog_fits_a_small_window() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-shortcuts-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        app.dialog = Some(Dialog::Shortcuts);

        let height = 420.0;
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(900.0, height),
            )),
            ..Default::default()
        };
        // Two frames: the dialog sizes itself on the first.
        let mut first = ctx.run_ui(input.clone(), |ui| app.frame_ui(ui));
        first.textures_delta.clear();
        let mut output = ctx.run_ui(input, |ui| app.frame_ui(ui));
        output.textures_delta.clear();

        let dialog = app.dialog_rect.expect("the dialog drew itself");
        let bottom = dialog.max.y;
        assert!(
            bottom <= height + 1.0,
            "the dialog runs {} pixels past the bottom of a {height}-tall window",
            bottom - height
        );
        app.backend.shutdown();
    }

    /// Virtual queue rows and library cards still draw a long list, and the
    /// library still asks for the next page when the end is near.
    #[test]
    fn a_long_virtual_queue_and_library_still_draw() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-virtual-long-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        if let Loadable::Loaded(queue) = &mut app.queue {
            let seed = queue.queue.clone();
            queue.queue = seed.iter().cloned().cycle().take(200).collect();
            app.manual_queue = queue
                .queue
                .iter()
                .take(80)
                .map(|item| item.uri().to_string())
                .collect();
        }
        app.show_queue_panel = true;
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }
        app.library.albums.items = (0..80)
            .map(|index| SavedAlbum {
                added_at: None,
                album: album(index % 8),
            })
            .collect();
        app.library.albums.next_offset = Some(80);
        app.library.albums.loaded_once = true;
        app.open(Page::Albums);
        app.actions.clear();
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    /// A drag in flight renders, and releasing it over an owned playlist
    /// row lands in the same add-to-playlist plumbing the row menu uses.
    #[test]
    fn dropping_selected_songs_on_a_sidebar_playlist_adds_them_all() {
        drop_songs_on_sidebar(2);
    }

    #[test]
    fn dropping_a_song_on_a_sidebar_playlist_adds_it() {
        drop_songs_on_sidebar(1);
    }

    fn drop_songs_on_sidebar(count: usize) {
        let root = std::env::temp_dir().join(format!(
            "spotifrost-drag-test-{}-{count}",
            std::process::id()
        ));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        app.library
            .playlists
            .get_mut()
            .expect("the demo library")
            .retain(|playlist| playlist.id == "pl1");
        app.open(Page::Playlist("pl1".into()));
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }
        let before = app.playlist_pages["pl1"].items.items.len();
        let mut dragged = vec![
            PlayableItem::Track(Track {
                id: Some("not-in-demo-playlists".into()),
                uri: "spotify:track:not-in-demo-playlists".into(),
                name: "A new song".into(),
                ..Default::default()
            }),
            PlayableItem::Track(Track {
                id: Some("also-not-in-demo-playlists".into()),
                uri: "spotify:track:also-not-in-demo-playlists".into(),
                name: "Another new song".into(),
                ..Default::default()
            }),
        ];

        dragged.truncate(count);

        // Sweep a held track down the sidebar; somewhere along the sweep
        // the pointer crosses an owned playlist row, and releasing there
        // must mark the playlist edit busy through the existing plumbing.
        // Where exactly the rows sit depends on the loaded fonts, so the
        // sweep does not hardcode a row position.
        let mut dropped = false;
        for step in 0..40 {
            let pos = egui::pos2(120.0, 120.0 + step as f32 * 15.0);
            egui::DragAndDrop::set_payload(
                &ctx,
                DragTrack {
                    title: "A new song".into(),
                    image: None,
                    items: dragged.clone(),
                    from: None,
                    source_playlist: None,
                },
            );
            frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            assert!(!egui::DragAndDrop::has_any_payload(&ctx));
            if app.playlist_busy {
                dropped = true;
                break;
            }
        }
        assert!(dropped, "no sweep position landed on an owned playlist row");
        assert_eq!(app.playlist_pages["pl1"].items.items.len(), before + count);
        assert_eq!(
            app.playlist_pages["pl1"].items.items[before..]
                .iter()
                .map(|row| row.playable().unwrap().uri())
                .collect::<Vec<_>>(),
            dragged.iter().map(PlayableItem::uri).collect::<Vec<_>>()
        );
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    /// A single Library row click keeps navigating and never starts playback.
    #[test]
    fn single_clicking_a_sidebar_row_only_navigates() {
        let (ctx, mut app) = accessible_app("sidebar-single-click");
        let view = crate::ui::sidebar::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let name = sidebar_text(&painted, "Sunday morning").center();
        app.actions.clear();
        view_frame(
            &ctx,
            &mut app,
            pointer_click(name, egui::PointerButton::Primary),
            view,
        );
        assert!(played_contexts(&app).is_empty());
        assert!(
            app.actions
                .iter()
                .any(|action| matches!(action, Action::Open(Page::Playlist(id)) if id == "pl2"))
        );
        app.backend.shutdown();
    }

    /// A Liked Songs row plays the collection while its rows are still
    /// loading. It used to play only the loaded rows as a nameless list,
    /// which stopped after the last of them and gave Spotify no context.
    #[test]
    fn a_liked_row_plays_the_collection_before_all_rows_load() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("liked-row-plays-collection");
        app.open(Page::LikedSongs);
        let liked = &mut app.library.liked;
        liked.next_offset = Some(liked.items.len() as u32);
        assert!(!app.library.liked.is_complete(), "more rows to load");
        let chosen = PlayableItem::Track(app.library.liked.items[1].track.clone());
        let label = format!("Play {}, {}", chosen.name(), chosen.subtitle());
        // Only the page: its actions stay queued instead of being applied.
        let render = |app: &mut App, events| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 2200.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| crate::ui::collection::liked(app, ui),
            );
            output.textures_delta.clear();
            output
                .platform_output
                .accesskit_update
                .expect("screen-reader tree")
        };
        render(&mut app, vec![]);
        let tree = render(&mut app, vec![]);
        let row = accessible_node(&tree, &label, Role::Button);
        app.actions.clear();
        render(
            &mut app,
            vec![accessible_action(row, AccessibleAction::Click, None)],
        );
        let played = app.actions.iter().find_map(|action| match action {
            Action::PlayFromRow {
                context: RowContext::Context { uri: context, .. },
                uri,
                ..
            } => Some((context.clone(), uri.clone())),
            _ => None,
        });
        assert_eq!(
            played,
            Some((
                "spotify:user:demo:collection".to_string(),
                chosen.uri().to_string()
            ))
        );
        app.backend.shutdown();
    }

    #[test]
    fn double_clicking_a_sidebar_liked_row_plays_the_collection() {
        let (ctx, mut app) = accessible_app("sidebar-double-click-liked");
        let view = crate::ui::sidebar::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let name = sidebar_text(&painted, "Liked Songs").center();
        app.actions.clear();
        let [first, second] = double_click(name);
        view_frame(&ctx, &mut app, first, view);
        view_frame(&ctx, &mut app, second, view);
        assert_eq!(played_contexts(&app), ["spotify:user:demo:collection"]);
        app.backend.shutdown();
    }

    #[test]
    fn double_clicking_a_sidebar_folder_plays_nothing() {
        use crate::player::RootlistEntry;
        let (ctx, mut app) = accessible_app("sidebar-double-click-folder");
        app.rootlist = vec![
            RootlistEntry::FolderStart {
                id: "f1".into(),
                name: "Focus".into(),
            },
            RootlistEntry::Playlist("spotify:playlist:pl1".into()),
            RootlistEntry::Playlist("spotify:playlist:pl2".into()),
            RootlistEntry::FolderEnd,
        ];
        let view = crate::ui::sidebar::show;
        view_frame(&ctx, &mut app, vec![], view);
        let painted = view_frame(&ctx, &mut app, vec![], view);
        let name = sidebar_text(&painted, "Focus").center();
        app.actions.clear();
        let [first, second] = double_click(name);
        view_frame(&ctx, &mut app, first, view);
        view_frame(&ctx, &mut app, second, view);
        assert!(played_contexts(&app).is_empty(), "folders must not play");
        app.backend.shutdown();
    }

    /// The cover and title in the bottom-left player are a song source, not
    /// just links. The sidebar can therefore receive the same complete row it
    /// receives when a table song is dragged.
    #[test]
    fn dragging_the_now_playing_song_supplies_a_playlist_row() {
        let root = std::env::temp_dir().join(format!(
            "spotifrost-now-playing-drag-test-{}",
            std::process::id()
        ));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }

        let start = egui::pos2(40.0, 755.0);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
        );

        let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
            .expect("dragging the bottom-left song should create a song payload");
        assert_eq!(payload.items.len(), 1);
        assert_eq!(payload.items[0].uri(), "spotify:track:trk0");
        assert_eq!(payload.from, None, "this is an add, not a playlist move");

        egui::DragAndDrop::clear_payload(&ctx);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    /// Dropping a dragged song on the queue button in the player bar queues
    /// it, the same as the "Add to queue" menu item.
    #[test]
    fn dropping_a_dragged_song_on_the_queue_button_queues_it() {
        let (ctx, mut app) = accessible_app("queue-button-drop");
        let source_uri = app
            .queue
            .get()
            .unwrap()
            .currently_playing
            .clone()
            .unwrap()
            .uri()
            .to_string();

        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let button = accessible_node(&tree, "Queue", egui::accesskit::Role::Button);
        let bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == button)
            .unwrap()
            .1
            .bounds()
            .unwrap();
        let end = egui::pos2(
            (bounds.x0 + bounds.x1) as f32 / 2.0,
            (bounds.y0 + bounds.y1) as f32 / 2.0,
        );

        // Drag the now-playing song from the bottom-left player, same
        // starting point as the equivalent playlist-insert test.
        let start = egui::pos2(40.0, 755.0);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
        );
        let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
            .expect("dragging the now-playing song should create a payload");
        assert_eq!(payload.items[0].uri(), source_uri);

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(app.manual_queue, vec![source_uri]);
        assert!(
            app.toasts
                .iter()
                .any(|toast| toast.message == "1 song added to queue"),
            "{:?}",
            app.toasts
        );
        app.backend.shutdown();
    }

    /// Dropping a dragged song anywhere on the open queue list (side panel
    /// or full page) queues it. The queue has no positional drop slots, so
    /// any point in the list works, not just the toggle button.
    #[test]
    fn dropping_a_dragged_song_on_the_open_queue_list_queues_it() {
        let (ctx, mut app) = accessible_app("queue-list-drop");
        app.show_queue_panel = true;
        let source_uri = app
            .queue
            .get()
            .unwrap()
            .currently_playing
            .clone()
            .unwrap()
            .uri()
            .to_string();
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }

        // A point well inside the queue side panel's body, clear of its
        // close/save buttons and tab chips.
        let end = egui::pos2(1100.0, 300.0);

        let start = egui::pos2(40.0, 755.0);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
        );
        let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
            .expect("dragging the now-playing song should create a payload");
        assert_eq!(payload.items[0].uri(), source_uri);

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(app.manual_queue, vec![source_uri]);
        assert!(
            app.toasts
                .iter()
                .any(|toast| toast.message == "1 song added to queue"),
            "{:?}",
            app.toasts
        );
        app.backend.shutdown();
    }

    /// Seeds three manually queued songs at the front of "Playing next",
    /// named "Queued 0".."Queued 2", for the queue-reorder tests below.
    /// Also makes the local player the active target, the only case where
    /// the queue can be reordered or inserted into positionally.
    fn seed_queued_songs(app: &mut App) -> Vec<String> {
        app.local_ready = true;
        app.local.track = Some(crate::player::LocalTrack {
            uri: app.now_playing().unwrap().uri,
            ..Default::default()
        });
        app.local.playback = crate::player::Playback::Paused;
        let songs: Vec<Track> = (0..3)
            .map(|index| {
                let mut t = track(index);
                t.uri = format!("spotify:track:queued{index}");
                t.id = Some(format!("queued{index}"));
                t.name = format!("Queued {index}");
                t
            })
            .collect();
        if let Loadable::Loaded(queue) = &mut app.queue {
            for (offset, song) in songs.iter().enumerate() {
                queue
                    .queue
                    .insert(offset, PlayableItem::Track(song.clone()));
            }
        }
        let uris: Vec<String> = songs.iter().map(|song| song.uri.clone()).collect();
        app.manual_queue = uris.clone();
        uris
    }

    /// Dragging a song already in "Playing next" and dropping it elsewhere
    /// in the queue moves it there instead of adding a duplicate.
    #[test]
    fn dragging_a_queued_row_within_the_queue_reorders_it() {
        let (ctx, mut app) = accessible_app("queue-reorder");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let row_rect = |name: &str| {
            let prefix = format!("Play {name},");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label().is_some_and(|text| text.starts_with(&prefix))
                })
                .unwrap_or_else(|| panic!("missing row {name}"))
                .1
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            )
        };
        let start_row = row_rect("Queued 0");
        let start = egui::pos2(start_row.left() + 80.0, start_row.center().y);
        // Dropped past the last queued row: moves to the end of "Playing next".
        let last_row = row_rect("Queued 2");
        let end = egui::pos2(last_row.left() + 130.0, last_row.bottom() - 1.0);

        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(15.0, -10.0))],
        );
        let payload =
            egui::DragAndDrop::payload::<DragTrack>(&ctx).expect("a reorderable queue row drags");
        assert_eq!(
            payload.from,
            Some(("queue".to_string(), 0)),
            "the queue row must tag itself as the move source"
        );

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(
            app.manual_queue,
            vec![uris[1].clone(), uris[2].clone(), uris[0].clone()],
            "moved, not duplicated"
        );
        app.backend.shutdown();
    }

    /// Right-clicking a song under Playing next offers to take it out of
    /// the queue, and doing so removes that row; a Next up row, which
    /// plays from the context, offers no such entry (#721).
    #[test]
    fn a_queued_rows_menu_removes_it_from_the_queue() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("queue-remove");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        // The queue panel's own row: some of its songs are also on the
        // page behind it, and the panel opens against the right edge.
        let row_center = |tree: &egui::accesskit::TreeUpdate, name: &str| {
            let prefix = format!("Play {name},");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == Role::Button
                        && node.label().is_some_and(|text| text.starts_with(&prefix))
                        && node
                            .bounds()
                            .is_some_and(|bounds| bounds.x0 >= 900.0 && bounds.y1 <= 800.0)
                })
                .unwrap_or_else(|| panic!("missing queue row {name}"))
                .1
                .bounds()
                .unwrap();
            egui::pos2(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        };
        let has_remove = |tree: &egui::accesskit::TreeUpdate| {
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Remove from queue"))
        };

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let next_up = row_center(&tree, "Otomo");
        accessible_frame(
            &ctx,
            &mut app,
            pointer_click(next_up, egui::PointerButton::Secondary),
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        accessible_node(&tree, "Add to queue", Role::Button);
        assert!(!has_remove(&tree), "a Next up row cannot be removed");
        accessible_frame(
            &ctx,
            &mut app,
            vec![keyboard(egui::Key::Escape, egui::Modifiers::NONE)],
        );

        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let queued = row_center(&tree, "Queued 1");
        accessible_frame(
            &ctx,
            &mut app,
            pointer_click(queued, egui::PointerButton::Secondary),
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let remove = accessible_node(&tree, "Remove from queue", Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(
                remove,
                egui::accesskit::Action::Click,
                None,
            )],
        );
        assert_eq!(app.manual_queue, [uris[0].clone(), uris[2].clone()]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            !tree.nodes.iter().any(|(_, node)| node
                .label()
                .is_some_and(|text| text.starts_with("Play Queued 1,"))),
            "the removed row leaves the queue at once"
        );
        app.backend.shutdown();
    }

    /// If the playing song advances (consuming the front queued row) while a
    /// queued row is mid-drag, the drop must still act on the song that was
    /// actually picked up, not on whatever now sits at the drag's recorded
    /// start index.
    #[test]
    fn dragging_a_queued_row_while_next_advances_moves_the_dragged_song() {
        let (ctx, mut app) = accessible_app("queue-reorder-mid-drag-advance");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let row_rect = |name: &str| {
            let prefix = format!("Play {name},");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label().is_some_and(|text| text.starts_with(&prefix))
                })
                .unwrap_or_else(|| panic!("missing row {name}"))
                .1
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            )
        };
        // "Queued 1" is dragged; row 0's slot is the drop target, captured
        // now so the drop still lands there once row 0 shifts up.
        let start_row = row_rect("Queued 1");
        let start = egui::pos2(start_row.left() + 80.0, start_row.center().y);
        let front_slot = row_rect("Queued 0");
        let end = egui::pos2(front_slot.left() + 130.0, front_slot.top() + 1.0);

        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(15.0, -10.0))],
        );
        let payload =
            egui::DragAndDrop::payload::<DragTrack>(&ctx).expect("a reorderable queue row drags");
        assert_eq!(
            payload.from,
            Some(("queue".to_string(), 1)),
            "the queue row must tag itself as the move source"
        );

        // "Next" fires mid-drag: the front queued row is consumed and every
        // later row's index shifts down by one, so index 1 (the recorded
        // drag source) now names "Queued 2" instead of the dragged song.
        app.manual_queue.remove(0);
        if let Loadable::Loaded(queue) = &mut app.queue {
            queue.queue.remove(0);
        }

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        // The dragged song ("Queued 1") had already shifted to the front on
        // its own; dropping it back on the front slot is a no-op. Trusting
        // the stale index 1 would instead have moved "Queued 2" (the wrong
        // song) to the front.
        assert_eq!(
            app.manual_queue,
            vec![uris[1].clone(), uris[2].clone()],
            "the dragged song stays put instead of the wrong row moving"
        );
        app.backend.shutdown();
    }

    /// "Next up" plays from the current context, not from a list Spotifrost
    /// can rewrite, so it is never a drop target: dropping a queued row on
    /// it must not move or insert anything, even though the row sits inside
    /// the same scrollable list as "Playing next".
    #[test]
    fn dropping_a_queued_row_on_next_up_does_nothing() {
        let (ctx, mut app) = accessible_app("queue-reorder-next-up-not-a-target");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        // "Otomo" is also a demo library track shown elsewhere on the page
        // behind the queue side panel, so match on the queue panel's own
        // on-screen column (it opens flush against the right edge) rather
        // than the first node with a matching label.
        let row_rect = |name: &str| {
            let prefix = format!("Play {name},");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label().is_some_and(|text| text.starts_with(&prefix))
                        && node
                            .bounds()
                            .is_some_and(|bounds| bounds.x0 >= 900.0 && bounds.y1 <= 800.0)
                })
                .unwrap_or_else(|| panic!("missing on-screen queue row {name}"))
                .1
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            )
        };
        let start_row = row_rect("Queued 0");
        let start = egui::pos2(start_row.left() + 80.0, start_row.center().y);
        // "Otomo" is the first row of "Next up", from the default demo
        // queue past the three manually queued rows seeded above.
        let next_up_row = row_rect("Otomo");
        // The "Next up" heading sits just below Playing next's last row.
        let heading = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Next up") || node.value() == Some("Next up"))
            .and_then(|(_, node)| node.bounds())
            .expect("the Next up heading");
        let heading = egui::Rect::from_min_max(
            egui::pos2(heading.x0 as f32, heading.y0 as f32),
            egui::pos2(heading.x1 as f32, heading.y1 as f32),
        );

        for end in [
            egui::pos2(next_up_row.left() + 130.0, next_up_row.center().y),
            egui::pos2(heading.left() + 130.0, heading.center().y),
            egui::pos2(heading.left() + 130.0, heading.top() + 1.0),
        ] {
            frame_events(
                &ctx,
                &mut app,
                vec![
                    egui::Event::PointerMoved(start),
                    egui::Event::PointerButton {
                        pos: start,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerMoved(start + egui::vec2(15.0, -10.0))],
            );
            egui::DragAndDrop::payload::<DragTrack>(&ctx).expect("a reorderable queue row drags");

            frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            frame_events(&ctx, &mut app, vec![]);

            assert_eq!(
                app.manual_queue, uris,
                "Next up is never a drop target: nothing moves or gets inserted (dropped at {end:?})"
            );
        }
        app.backend.shutdown();
    }

    /// Holding a dragged queue row at the bottom edge of the open queue
    /// scrolls it, as the playlist table does, so a row can be moved past
    /// the rows that fit on screen.
    #[test]
    fn dragging_a_queued_row_scrolls_a_long_playing_next() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("queue-drag-scroll");
        app.show_queue_panel = true;
        seed_queued_songs(&mut app);
        let songs: Vec<Track> = (3..40)
            .map(|index| {
                let mut t = track(index);
                t.uri = format!("spotify:track:queued{index}");
                t.id = Some(format!("queued{index}"));
                t.name = format!("Queued {index}");
                t
            })
            .collect();
        if let Loadable::Loaded(queue) = &mut app.queue {
            for (offset, song) in songs.iter().enumerate() {
                queue
                    .queue
                    .insert(3 + offset, PlayableItem::Track(song.clone()));
            }
        }
        app.manual_queue
            .extend(songs.iter().map(|song| song.uri.clone()));
        let uris = app.manual_queue.clone();
        let on_screen = |tree: &egui::accesskit::TreeUpdate, name: &str| {
            let prefix = format!("Play {name},");
            tree.nodes.iter().find_map(|(_, node)| {
                (node.role() == Role::Button
                    && node.label().is_some_and(|text| text.starts_with(&prefix)))
                .then(|| node.bounds())
                .flatten()
                .filter(|bounds| bounds.x0 >= 900.0)
            })
        };
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            on_screen(&tree, "Queued 39").is_none(),
            "the last queued row starts out of view"
        );
        let first = on_screen(&tree, "Queued 0").expect("the first queued row is shown");
        let start = egui::pos2(first.x0 as f32 + 80.0, ((first.y0 + first.y1) / 2.0) as f32);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(15.0, -10.0))],
        );
        egui::DragAndDrop::payload::<DragTrack>(&ctx).expect("a reorderable queue row drags");
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(egui::pos2(
                start.x,
                800.0 - crate::theme::PLAYER_BAR_HEIGHT - 14.0,
            ))],
        );
        for _ in 0..400 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            on_screen(&tree, "Queued 39").is_some(),
            "holding the drag at the bottom edge scrolls Playing next into view"
        );
        assert_eq!(
            app.manual_queue, uris,
            "scrolling alone must not reorder rows"
        );
        egui::DragAndDrop::clear_payload(&ctx);
        app.backend.shutdown();
    }

    /// With nothing manually queued yet, every row the open queue shows
    /// belongs to "Next up", so the panel offers no drop target at all:
    /// neither a Next up row nor its heading takes a dragged song. The
    /// player bar's Queue button still does.
    #[test]
    fn dropping_a_song_on_next_up_with_an_empty_playing_next_does_nothing() {
        use egui::accesskit::Role;
        let (ctx, mut app) = accessible_app("queue-empty-playing-next-not-a-target");
        app.show_queue_panel = true;
        // Make the local player the active target, like `seed_queued_songs`,
        // but leave "Playing next" empty.
        app.local_ready = true;
        app.local.track = Some(crate::player::LocalTrack {
            uri: app.now_playing().unwrap().uri,
            ..Default::default()
        });
        app.local.playback = crate::player::Playback::Paused;
        assert!(app.manual_queue.is_empty());
        assert!(app.queue_locally_reorderable());

        let source_uri = app
            .queue
            .get()
            .unwrap()
            .currently_playing
            .clone()
            .unwrap()
            .uri()
            .to_string();

        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let bounds_of = |id: egui::accesskit::NodeId| {
            let bounds = tree
                .nodes
                .iter()
                .find(|(node, _)| *node == id)
                .unwrap()
                .1
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            )
        };
        // "Otomo" is the first row of the default demo queue's "Next up".
        // It is also a demo library track shown elsewhere on the page
        // behind the queue side panel, so match on the queue panel's own
        // on-screen column (it opens flush against the right edge) rather
        // than the first node with a matching label.
        let row = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Button
                    && node
                        .label()
                        .is_some_and(|text| text.starts_with("Play Otomo,"))
                    && node
                        .bounds()
                        .is_some_and(|bounds| bounds.x0 >= 900.0 && bounds.y1 <= 800.0)
            })
            .unwrap_or_else(|| panic!("missing on-screen queue row Otomo"))
            .0;
        let row = bounds_of(row);
        let heading = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Next up") || node.value() == Some("Next up"))
            .expect("the Next up heading")
            .0;
        let heading = bounds_of(heading);
        // The player bar's Queue button, not the queue panel's Queue tab.
        let button = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == Role::Button
                    && node.label() == Some("Queue")
                    && node.bounds().is_some_and(|bounds| {
                        bounds.y0 >= f64::from(800.0 - crate::theme::PLAYER_BAR_HEIGHT)
                    })
            })
            .expect("the player bar's Queue button")
            .0;
        let button = bounds_of(button);

        let drag_to = |app: &mut App, end: egui::Pos2| {
            let start = egui::pos2(40.0, 755.0);
            frame_events(
                &ctx,
                app,
                vec![
                    egui::Event::PointerMoved(start),
                    egui::Event::PointerButton {
                        pos: start,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
            frame_events(
                &ctx,
                app,
                vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
            );
            let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
                .expect("dragging the now-playing song should create a payload");
            assert_eq!(payload.items[0].uri(), source_uri);
            frame_events(&ctx, app, vec![egui::Event::PointerMoved(end)]);
            frame_events(
                &ctx,
                app,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            frame_events(&ctx, app, vec![]);
        };

        for end in [
            egui::pos2(row.left() + 130.0, row.center().y),
            egui::pos2(row.left() + 130.0, row.top() + 2.0),
            heading.center(),
            egui::pos2(heading.left() + 130.0, heading.top() - 2.0),
        ] {
            drag_to(&mut app, end);
            assert!(
                app.manual_queue.is_empty(),
                "Next up is never a drop target, even when Playing next has no rows of its own (dropped at {end:?})"
            );
        }

        drag_to(&mut app, button.center());
        assert_eq!(
            app.manual_queue,
            vec![source_uri],
            "the Queue button still queues the dropped song"
        );
        app.backend.shutdown();
    }

    /// Dropping a song from elsewhere at a specific row in "Playing next"
    /// inserts it there instead of always appending at the end.
    #[test]
    fn dropping_a_new_song_at_a_queue_position_inserts_it_there() {
        let (ctx, mut app) = accessible_app("queue-insert-position");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let row_rect = |name: &str| {
            let prefix = format!("Play {name},");
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.role() == egui::accesskit::Role::Button
                        && node.label().is_some_and(|text| text.starts_with(&prefix))
                })
                .unwrap_or_else(|| panic!("missing row {name}"))
                .1
                .bounds()
                .unwrap();
            egui::Rect::from_min_max(
                egui::pos2(bounds.x0 as f32, bounds.y0 as f32),
                egui::pos2(bounds.x1 as f32, bounds.y1 as f32),
            )
        };
        // Drop on top of "Queued 1": inserts before it.
        let target_row = row_rect("Queued 1");
        let end = egui::pos2(target_row.left() + 130.0, target_row.top() + 1.0);

        let start = egui::pos2(40.0, 755.0);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
        );
        let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
            .expect("dragging the now-playing song should create a payload");
        let dropped_uri = payload.items[0].uri().to_string();

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(
            app.manual_queue,
            vec![
                uris[0].clone(),
                dropped_uri,
                uris[1].clone(),
                uris[2].clone(),
            ],
        );
        app.backend.shutdown();
    }

    /// Without local playback active, neither the Web API nor librespot can
    /// reorder or insert into the live queue, so a drop still just appends,
    /// exactly like before this position-aware behavior existed.
    #[test]
    fn dropping_on_the_queue_without_local_playback_still_just_appends() {
        let (ctx, mut app) = accessible_app("queue-remote-fallback");
        app.show_queue_panel = true;
        let uris = seed_queued_songs(&mut app);
        app.local_ready = false;
        assert!(!app.queue_locally_reorderable());
        for _ in 0..3 {
            frame_events(&ctx, &mut app, vec![]);
        }
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let bounds = tree
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button
                    && node
                        .label()
                        .is_some_and(|text| text.starts_with("Play Queued 0,"))
            })
            .unwrap()
            .1
            .bounds()
            .unwrap();
        // Drop right on the first queued row; without local playback this
        // still appends at the end, ignoring the row it landed on.
        let end = egui::pos2(bounds.x0 as f32 + 130.0, bounds.y0 as f32 + 2.0);

        let start = egui::pos2(40.0, 755.0);
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(start),
                egui::Event::PointerButton {
                    pos: start,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerMoved(start + egui::vec2(20.0, -10.0))],
        );
        let payload = egui::DragAndDrop::payload::<DragTrack>(&ctx)
            .expect("dragging the now-playing song should create a payload");
        assert_eq!(payload.from, None, "not reorderable, so not tagged as one");
        let dropped_uri = payload.items[0].uri().to_string();

        frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
        frame_events(
            &ctx,
            &mut app,
            vec![egui::Event::PointerButton {
                pos: end,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            }],
        );

        assert_eq!(
            app.manual_queue,
            vec![
                uris[0].clone(),
                uris[1].clone(),
                uris[2].clone(),
                dropped_uri
            ],
        );
        app.backend.shutdown();
    }

    /// Pins are pins: dropping a pinned row at the top of the block
    /// reorders the pins themselves, and the rest of the shelf stays in
    /// its automatic order.
    #[test]
    fn dragging_within_the_pinned_block_reorders_it() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-reorder-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        app.settings.pinned_contexts =
            vec!["spotify:playlist:pl2".into(), "spotify:playlist:pl4".into()];
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }

        // Sweep from the top: the first slot inside the list drops the
        // dragged row above Liked Songs. Where the list begins
        // depends on the loaded fonts, so the sweep does not hardcode it.
        let mut dropped = false;
        for step in 0..40 {
            let pos = egui::pos2(120.0, 100.0 + step as f32 * 10.0);
            egui::DragAndDrop::set_payload(
                &ctx,
                DragEntry {
                    uri: "spotify:playlist:pl4".into(),
                    title: "Release Radar".into(),
                    image: None,
                },
            );
            frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            egui::DragAndDrop::clear_payload(&ctx);
            if app.settings.pinned_contexts.first().map(String::as_str)
                == Some("spotify:playlist:pl4")
            {
                dropped = true;
                break;
            }
        }
        assert!(dropped, "no sweep position landed in the pinned block");
        assert_eq!(
            app.settings.pinned_contexts,
            vec![
                "spotify:playlist:pl4".to_string(),
                crate::settings::LIKED_SONGS_KEY.to_string(),
                "spotify:playlist:pl2".to_string(),
            ],
        );
        assert!(app.settings.sidebar_order.is_empty());
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    /// Reordering unpinned playlists creates a custom sidebar order.
    #[test]
    fn dropping_between_unpinned_playlists_creates_the_custom_order() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-unpinned-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        assert!(app.settings.pinned_contexts.is_empty());
        assert!(app.settings.sidebar_order.is_empty());
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }

        // Sweep from the top; the first slot inside the list is the one
        // right under Liked Songs, between what were the first two
        // unpinned playlists.
        let mut dropped = false;
        for step in 0..40 {
            let pos = egui::pos2(120.0, 100.0 + step as f32 * 10.0);
            egui::DragAndDrop::set_payload(
                &ctx,
                DragEntry {
                    uri: "spotify:playlist:pl4".into(),
                    title: "Release Radar".into(),
                    image: None,
                },
            );
            frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            egui::DragAndDrop::clear_payload(&ctx);
            if !app.settings.sidebar_order.is_empty() {
                dropped = true;
                break;
            }
        }
        assert!(dropped, "no sweep position landed below Liked Songs");
        let expected: Vec<String> = [0, 4]
            .into_iter()
            .chain((1..PLAYLISTS.len()).filter(|index| *index != 4))
            .map(|index| format!("spotify:playlist:pl{index}"))
            .collect();
        assert_eq!(app.settings.sidebar_order, expected);
        assert_eq!(
            app.settings.library_pins(),
            [crate::settings::LIKED_SONGS_KEY]
        );
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn playlist_drop_targets_cover_empty_lists_and_preserve_editing_boundaries() {
        for mode in [
            "foreign",
            "empty",
            "multiple",
            "own-multiple",
            "readonly",
            "sorted",
            "filtered",
        ] {
            let (ctx, mut app) = accessible_app(&format!("playlist-drop-target-{mode}"));
            let mut target = track(0);
            target.name = "Drop target".into();
            target.uri = "spotify:track:destination".into();
            let rows = vec![(PlayableItem::Track(target), None, None)];
            if mode == "sorted" {
                app.table_sorts.insert(
                    Page::Playlist("pl1".into()),
                    crate::model::TableSort {
                        column: crate::model::SortColumn::Title,
                        ascending: true,
                    },
                );
            }
            let draw = |app: &mut App, events, empty| {
                let mut output = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(760.0, 620.0),
                        )),
                        events,
                        ..Default::default()
                    },
                    |ui| {
                        crate::ui::collection::table(
                            app,
                            ui,
                            crate::ui::collection::Table {
                                pagination: None,
                                items: if empty { &[] } else { &rows },
                                row_offset: if mode == "empty" { 0 } else { 100 },
                                context: crate::model::RowContext::Context {
                                    uri: "spotify:playlist:pl1".into(),
                                    editable_playlist: (mode != "readonly")
                                        .then(|| ("pl1".into(), None)),
                                },
                                show_album: false,
                                show_cover: true,
                                show_added: false,
                                show_added_by: false,
                                show_plays: false,
                                page: Page::Playlist("pl1".into()),
                                loading: false,
                                error: None,
                                can_load_more: false,
                                filter: if mode == "filtered" {
                                    "Drop target"
                                } else {
                                    ""
                                },
                                items_revision: u64::from(empty),
                                plays_shown_rows: false,
                            },
                        );
                    },
                );
                output.textures_delta.clear();
                output.platform_output.accesskit_update.unwrap()
            };
            draw(&mut app, vec![], false);
            let tree = draw(&mut app, vec![], false);
            let bounds = tree
                .nodes
                .iter()
                .find(|(_, node)| {
                    node.label()
                        .is_some_and(|label| label.starts_with("Play Drop target,"))
                })
                .unwrap()
                .1
                .bounds()
                .unwrap();
            let end = egui::pos2(
                bounds.x0 as f32 + 140.0,
                if mode == "empty" {
                    400.0
                } else {
                    bounds.y0 as f32 + 2.0
                },
            );
            let dragged =
                app.queue.get().unwrap().queue[..if matches!(mode, "multiple" | "own-multiple") {
                    2
                } else {
                    1
                }]
                    .to_vec();
            let expected: Vec<_> = dragged.iter().map(|item| item.uri().to_string()).collect();
            egui::DragAndDrop::set_payload(
                &ctx,
                DragTrack {
                    title: dragged[0].name().into(),
                    image: None,
                    items: dragged,
                    from: (mode == "foreign").then(|| ("pl2".into(), 5)),
                    source_playlist: match mode {
                        "foreign" => Some("pl2".into()),
                        "own-multiple" => Some("pl1".into()),
                        _ => None,
                    },
                },
            );
            draw(
                &mut app,
                vec![
                    egui::Event::PointerMoved(end),
                    egui::Event::PointerButton {
                        pos: end,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                mode == "empty",
            );
            app.actions.clear();
            draw(
                &mut app,
                vec![egui::Event::PointerButton {
                    pos: end,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                mode == "empty",
            );
            if matches!(mode, "foreign" | "empty" | "multiple") {
                assert!(
                    matches!(app.actions.as_slice(), [Action::InsertInPlaylist { playlist_id, position, items }] if playlist_id == "pl1" && *position == if mode == "empty" { 0 } else { 100 } && items.iter().map(PlayableItem::uri).collect::<Vec<_>>() == expected),
                    "{mode}: {:?}",
                    app.actions
                );
            } else {
                assert!(
                    !app.actions.iter().any(|action| matches!(
                        action,
                        Action::InsertInPlaylist { .. } | Action::MoveInPlaylist { .. }
                    )),
                    "{mode}"
                );
            }
            app.backend.shutdown();
        }
    }

    /// Dragging a row within an owned playlist's table moves it through
    /// the same MoveInPlaylist action the menu's move items use: the slot
    /// is Spotify's insert-before, which the handler mirrors locally
    /// before asking the server.
    #[test]
    fn dragging_a_row_within_a_playlist_reorders_it() {
        let root =
            std::env::temp_dir().join(format!("spotifrost-move-test-{}", std::process::id()));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);
        app.open(Page::Playlist("pl1".into()));
        for _ in 0..3 {
            frame(&ctx, &mut app);
        }
        let order = |app: &App| -> Vec<String> {
            app.playlist_pages["pl1"]
                .items
                .items
                .iter()
                .filter_map(|item| item.playable().map(|playable| playable.uri().to_string()))
                .collect()
        };
        let original = order(&app);
        let from = 5usize;
        let held = |from: usize, uri: &str| DragTrack {
            title: "Closer".into(),
            image: None,
            items: vec![PlayableItem::Track(Track {
                uri: uri.to_string(),
                name: "Closer".into(),
                ..Default::default()
            })],
            from: Some(("pl1".into(), from as u32)),
            source_playlist: Some("pl1".into()),
        };

        // Sweep the held row down the page; above the table nothing
        // bites, and the first slot inside it lands the row above its old
        // place. Where the table begins depends on the loaded fonts, so
        // the sweep does not hardcode it.
        let mut landed = None;
        for step in 0..45 {
            let pos = egui::pos2(700.0, 120.0 + step as f32 * 15.0);
            egui::DragAndDrop::set_payload(&ctx, held(from, &original[from]));
            frame_events(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
            frame_events(
                &ctx,
                &mut app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            egui::DragAndDrop::clear_payload(&ctx);
            if app.playlist_busy {
                landed = Some(pos);
                break;
            }
        }
        let landed = landed.expect("no sweep position landed inside the table");
        let drop_at = |ctx: &egui::Context, app: &mut App, payload: DragTrack| {
            egui::DragAndDrop::set_payload(ctx, payload);
            frame_events(ctx, app, vec![egui::Event::PointerMoved(landed)]);
            frame_events(
                ctx,
                app,
                vec![egui::Event::PointerButton {
                    pos: landed,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
            egui::DragAndDrop::clear_payload(ctx);
        };
        // The handler mirrored the move locally: the dragged row moved
        // up, everything else kept its order.
        let now = order(&app);
        let to = now
            .iter()
            .position(|uri| *uri == original[from])
            .expect("the dragged row vanished");
        assert!(to < from, "the row should have moved up, not to {to}");
        let mut expected = original.clone();
        let moved = expected.remove(from);
        expected.insert(to, moved);
        assert_eq!(now, expected);

        // Dropping the row on the same slot again moves nothing: the slot
        // is insert-before, so a row's own edges are a no-op. A slot sent
        // one row out would move it here.
        app.playlist_busy = false;
        drop_at(&ctx, &mut app, held(to, &expected[to]));
        assert!(!app.playlist_busy, "a row dropped on its own slot moved");
        assert_eq!(order(&app), expected);

        // A sorted view refuses the move: positions on screen no longer
        // match the server's.
        app.table_sorts.insert(
            Page::Playlist("pl1".into()),
            TableSort {
                column: SortColumn::Title,
                ascending: true,
            },
        );
        frame(&ctx, &mut app);
        drop_at(&ctx, &mut app, held(to, &expected[to]));
        assert!(!app.playlist_busy, "a sorted view accepted a move");
        assert_eq!(order(&app), expected);
        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn sorted_playlist_view_keeps_remove_but_not_moves() {
        let (ctx, mut app) = accessible_app("sorted-remove-menu");
        app.backend.set_offline(true);
        let item = PlayableItem::Track(track(0));
        let uri = item.uri().to_string();
        let editable = Some(("pl1".to_string(), None));
        let unsorted = RowContext::Context {
            uri: "spotify:playlist:pl1".into(),
            editable_playlist: editable.clone(),
        };
        let sorted = RowContext::View {
            uris: Arc::from([uri.clone()]),
            context_uri: "spotify:playlist:pl1".into(),
            editable_playlist: editable.clone(),
        };
        let paint = |ctx: &egui::Context,
                     app: &mut App,
                     item: &PlayableItem,
                     context: &RowContext,
                     events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    crate::ui::widgets::item_menu(ui, app, item, Some(context), Some(1));
                },
            );
            output.textures_delta.clear();
            menu_text(&output)
        };
        // The unsorted menu offers positional moves and removal.
        let painted = paint(&ctx, &mut app, &item, &unsorted, vec![]);
        for label in ["Move up", "Move down", "Remove from this playlist"] {
            assert!(
                painted.iter().any(|(text, _)| text == label),
                "unsorted menu is missing {label}"
            );
        }
        // A sorted view keeps the URI-based removal but drops the
        // positional moves: screen positions no longer match the server's.
        let painted = paint(&ctx, &mut app, &item, &sorted, vec![]);
        assert!(
            painted
                .iter()
                .any(|(text, _)| text == "Remove from this playlist"),
            "sorted menu lost its removal"
        );
        for label in ["Move up", "Move down"] {
            assert!(
                !painted.iter().any(|(text, _)| text == label),
                "sorted menu must not offer {label}"
            );
        }
        // Clicking the sorted removal removes that song by URI.
        let remove = painted
            .iter()
            .find(|(text, _)| text == "Remove from this playlist")
            .unwrap()
            .1
            .center();
        app.actions.clear();
        paint(
            &ctx,
            &mut app,
            &item,
            &sorted,
            pointer_click(remove, egui::PointerButton::Primary),
        );
        assert!(
            matches!(app.actions.as_slice(), [Action::RemoveFromPlaylist { playlist_id, uris, .. }] if playlist_id == "pl1" && uris == std::slice::from_ref(&uri))
        );
        // A view without edit rights offers no removal at all.
        let readonly = RowContext::View {
            uris: Arc::from([uri.clone()]),
            context_uri: "spotify:playlist:pl1".into(),
            editable_playlist: None,
        };
        let painted = paint(&ctx, &mut app, &item, &readonly, vec![]);
        assert!(
            !painted
                .iter()
                .any(|(text, _)| text == "Remove from this playlist"),
            "a read-only view must not offer removal"
        );
        // The multi-select menu removes the whole selection by URI too.
        let songs = vec![PlayableItem::Track(track(0)), PlayableItem::Track(track(1))];
        let frame = |ctx: &egui::Context,
                     app: &mut App,
                     songs: &[PlayableItem],
                     events: Vec<egui::Event>| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(760.0, 620.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ui| {
                    crate::ui::widgets::picked_menu(
                        ui,
                        app,
                        songs,
                        &[],
                        Some(&("pl1".to_string(), None)),
                    );
                },
            );
            output.textures_delta.clear();
            menu_text(&output)
        };
        let text = frame(&ctx, &mut app, &songs, vec![]);
        let remove = text
            .iter()
            .find(|(text, _)| text == "Remove from this playlist")
            .expect("multi-select menu lost its removal")
            .1
            .center();
        app.actions.clear();
        frame(
            &ctx,
            &mut app,
            &songs,
            pointer_click(remove, egui::PointerButton::Primary),
        );
        let expected: Vec<String> = songs.iter().map(|song| song.uri().to_string()).collect();
        assert!(
            matches!(app.actions.as_slice(), [Action::RemoveFromPlaylist { playlist_id, uris, .. }] if playlist_id == "pl1" && *uris == expected)
        );
        app.backend.shutdown();
    }

    /// The custom order is a setting like any other: it survives the trip
    /// through the settings file, and older files without it stay in the
    /// automatic order.
    #[test]
    fn custom_sidebar_order_round_trips_through_settings() {
        let settings = Settings {
            sidebar_order: vec![
                "spotify:playlist:pl4".to_string(),
                "spotify:playlist:pl0".to_string(),
            ],
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.sidebar_order, settings.sidebar_order);
        let older: Settings = serde_json::from_str("{}").unwrap();
        assert!(older.sidebar_order.is_empty());
    }

    /// Clicking the search icon in the library header reveals and focuses
    /// the sidebar search field.
    #[test]
    fn clicking_search_in_library_shelf_focuses_search_field() {
        let root = std::env::temp_dir().join(format!(
            "spotifrost-sidebar-search-focus-test-{}",
            std::process::id()
        ));
        let dirs = AppDirs {
            config: root.join("config"),
            state: root.join("state"),
            cache: root.join("cache"),
        };
        let ctx = egui::Context::default();
        ctx.enable_accesskit();
        let waker = crate::backend::Waker::default();
        waker.attach(&ctx);
        let mut app = App::new(
            &waker,
            dirs,
            Settings::default(),
            AppOptions {
                media_controls: false,
                restore_sign_in: false,
                tray: false,
                autostart: false,
            },
        );
        app.attach(&ctx);
        populate(&mut app);

        // Use the button's actual bounds: the header can gain controls
        // without changing which button this pointer test exercises.
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let search = accessible_node(&tree, "Search Your Library", egui::accesskit::Role::Button);
        let bounds = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == search)
            .and_then(|(_, node)| node.bounds())
            .expect("Search Your Library bounds");
        let search_pos = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );

        // Click on the search button in the Library shelf header.
        frame_events(
            &ctx,
            &mut app,
            vec![
                egui::Event::PointerMoved(search_pos),
                egui::Event::PointerButton {
                    pos: search_pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: search_pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        );

        // Advance one frame so the focused widget processes events.
        frame(&ctx, &mut app);

        // Verify the search field is shown and has keyboard focus.
        let search_id = egui::Id::new("sidebar-search");
        let has_focus = ctx.memory(|m| m.has_focus(search_id));
        assert!(
            has_focus,
            "sidebar-search must have keyboard focus after clicking the search icon"
        );

        app.backend.shutdown();
        let _ = std::fs::remove_dir_all(root);
    }

    /// The device popup lists this computer's audio outputs on request,
    /// by name, and writes the chosen one's id to `audio_device`; System
    /// default clears it (#963).
    #[test]
    fn the_device_popup_chooses_this_computers_audio_output() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("audio-outputs");
        app.show_devices = true;
        app.settings.audio_device = None;
        app.output_devices = Loadable::Loaded(demo_outputs()[..2].to_vec());
        let click = |ctx: &egui::Context, app: &mut App, label: &str| {
            let tree = accessible_frame(ctx, app, vec![]);
            let node = accessible_node(&tree, label, Role::Button);
            accessible_frame(
                ctx,
                app,
                vec![accessible_action(node, AccessibleAction::Click, None)],
            );
        };
        let listed = |ctx: &egui::Context, app: &mut App, label: &str| {
            accessible_frame(ctx, app, vec![])
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some(label))
        };
        let voicemeeter = "VoiceMeeter Input (VB-Audio VoiceMeeter VAIO)";
        accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            !listed(&ctx, &mut app, voicemeeter),
            "listed only on request"
        );
        click(&ctx, &mut app, "Audio output");
        assert!(listed(&ctx, &mut app, "System default"));
        assert!(
            app.output_devices.get().is_some(),
            "an earlier list stays while it is read again"
        );
        click(&ctx, &mut app, voicemeeter);
        accessible_frame(&ctx, &mut app, vec![]);
        let voicemeeter_id = demo_outputs()[1].id.as_str().to_string();
        assert_eq!(app.settings.audio_device, Some(voicemeeter_id.clone()));
        assert!(listed(&ctx, &mut app, voicemeeter), "shown by its name");
        assert!(!listed(&ctx, &mut app, &voicemeeter_id));
        click(&ctx, &mut app, "System default");
        accessible_frame(&ctx, &mut app, vec![]);
        assert_eq!(app.settings.audio_device, None);
        app.backend.shutdown();
    }

    /// A name an older build saved shows as that device, chosen, and gives
    /// way to its id once playback opened on it; a name no output has, such
    /// as an ALSA one from before PipeWire's host, stays listed as saved.
    #[test]
    fn an_old_saved_output_name_shows_and_becomes_its_id() {
        let (ctx, mut app) = accessible_app("audio-outputs");
        app.show_devices = true;
        app.output_devices = Loadable::Loaded(demo_outputs());
        let speakers = demo_outputs()[0].clone();
        app.settings.audio_device = Some(speakers.name.clone());
        accessible_frame(&ctx, &mut app, vec![]);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let node = accessible_node(&tree, "Audio output", egui::accesskit::Role::Button);
        accessible_frame(
            &ctx,
            &mut app,
            vec![accessible_action(
                node,
                egui::accesskit::Action::Click,
                None,
            )],
        );
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        let rows = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.label() == Some(speakers.name.as_str()))
            .count();
        assert_eq!(
            rows, 1,
            "the saved name is the listed device, not a row of its own"
        );

        // Another choice made meanwhile is kept.
        app.settings.audio_device = Some("other".into());
        app.on_audio_device_found(&speakers.name, speakers.id.as_str().into());
        assert_eq!(app.settings.audio_device.as_deref(), Some("other"));
        app.settings.audio_device = Some(speakers.name.clone());
        app.on_audio_device_found(&speakers.name, speakers.id.as_str().into());
        assert_eq!(
            app.settings.audio_device.as_deref(),
            Some(speakers.id.as_str())
        );

        app.settings.audio_device = Some("PipeWire Sound Server".into());
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            tree.nodes
                .iter()
                .any(|(_, node)| node.label() == Some("PipeWire Sound Server"))
        );
        app.backend.shutdown();
    }

    /// Clicking a page's header cover shows Spotify's largest artwork, and a
    /// click on it, a click beside it, or Esc closes it (#586). Liked Songs'
    /// drawn cover has nothing to enlarge.
    #[test]
    fn clicking_a_header_cover_enlarges_it_until_any_click_or_escape() {
        use egui::accesskit::{Action as AccessibleAction, Role};
        let (ctx, mut app) = accessible_app("cover-art");
        let open = |ctx: &egui::Context, app: &mut App| {
            let tree = accessible_frame(ctx, app, vec![]);
            let cover = accessible_node(&tree, "Enlarge cover", Role::Button);
            accessible_frame(
                ctx,
                app,
                vec![accessible_action(cover, AccessibleAction::Click, None)],
            );
            let Some(Dialog::CoverArt { url, title, .. }) = &app.dialog else {
                panic!("the cover opens enlarged: {:?}", app.dialog);
            };
            assert!(url.ends_with("/640/640"), "the largest artwork: {url}");
            assert_eq!(title, "Late night focus");
            // The modal's first frame only measures it.
            accessible_frame(ctx, app, vec![]);
            accessible_frame(ctx, app, vec![])
        };
        app.open(Page::Playlist("pl1".into()));
        accessible_frame(&ctx, &mut app, vec![]);
        let centre = egui::pos2(640.0, 400.0);
        for close in [
            pointer_click(centre, egui::PointerButton::Primary),
            pointer_click(egui::pos2(8.0, 8.0), egui::PointerButton::Primary),
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
        ] {
            let tree = open(&ctx, &mut app);
            accessible_node(&tree, "Late night focus", Role::Image);
            accessible_frame(&ctx, &mut app, close);
            accessible_frame(&ctx, &mut app, vec![]);
            assert!(app.dialog.is_none(), "it closes");
            assert_eq!(app.page(), &Page::Playlist("pl1".into()), "on its page");
        }
        app.open(Page::LikedSongs);
        let tree = accessible_frame(&ctx, &mut app, vec![]);
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Enlarge cover")),
            "Liked Songs has no artwork to enlarge"
        );
        app.backend.shutdown();
    }

    /// Discography placeholders fill the old grid, including any space
    /// reserved for pagination, without relying on an outer minimum height.
    #[test]
    fn discography_placeholders_fill_the_previous_grid_height() {
        use crate::ui::widgets;

        let (ctx, mut app) = accessible_app("discography-placeholder-height");
        for light in [false, true] {
            ctx.set_theme(if light {
                egui::Theme::Light
            } else {
                egui::Theme::Dark
            });
            for width in [760.0, 1280.0] {
                for count in [1, 8, 20] {
                    for footer in [0.0, 32.0] {
                        let mut loaded_height = 0.0;
                        let mut output = ctx.run_ui(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 6000.0),
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                ui.set_width(width - 298.0);
                                loaded_height = ui
                                    .scope(|ui| {
                                        widgets::grid(ui, |ui| {
                                            for index in 0..count {
                                                let album = super::album(index);
                                                widgets::card(
                                                    ui,
                                                    &mut app,
                                                    None,
                                                    &album.name,
                                                    "2026 • Album",
                                                    widgets::CardCover::square(false),
                                                );
                                            }
                                        });
                                        ui.add_space(footer);
                                    })
                                    .response
                                    .rect
                                    .height();
                            },
                        );
                        output.textures_delta.clear();
                        let palette = app.palette;
                        let locale = app.locale;
                        let mut skeleton_height = 0.0;
                        let mut output = ctx.run_ui(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 6000.0),
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                ui.set_width(width - 298.0);
                                skeleton_height = ui
                                    .scope(|ui| {
                                        widgets::grid(ui, |ui| {
                                            widgets::skeleton_cards_filling(
                                                ui,
                                                &palette,
                                                locale,
                                                count,
                                                false,
                                                loaded_height,
                                            );
                                        });
                                    })
                                    .response
                                    .rect
                                    .height();
                            },
                        );
                        output.textures_delta.clear();
                        assert!(
                            (skeleton_height - loaded_height).abs() <= 1.0,
                            "{width}px, light={light}, {count} cards, footer={footer}: \
                             loaded {loaded_height}, skeletons {skeleton_height}"
                        );
                    }
                }
            }
        }
        app.backend.shutdown();
    }
}
