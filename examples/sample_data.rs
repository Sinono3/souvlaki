use std::time::Duration;

use souvlaki::{MediaMetadata, MediaTypeApple, MediaTypeWindows};

#[allow(dead_code)]
fn main() {
    println!(
        "This file is a library, which is not meant to be executed. Please use the other examples"
    )
}

pub fn album() -> [MediaMetadata; 10] {
    [
        (1, "Alison", 232_411_429),
        (2, "Machine Gun", 268_042_449),
        (3, "40 Days", 195_735_510),
        (4, "Sing", 289_306_122),
        (5, "Here She Comes", 141_008_980),
        (6, "Souvlaki Space Station", 359_131_429),
        (7, "When the Sun Hits", 287_451_429),
        (8, "Altogether", 222_458_776),
        (9, "Melon Yellow", 233_586_939),
        (10, "Dagger", 214_674_286),
    ]
    .map(|(track_number, title, microseconds)| MediaMetadata {
        title: Some(title.to_string()),
        track_number: Some(track_number),
        duration: Some(Duration::from_micros(microseconds)),
        ..base()
    })
}

pub fn base() -> MediaMetadata {
    MediaMetadata {
        artist: Some("Slowdive".to_owned()),
        artists: Some(vec!["Slowdive".to_owned()]),
        album_title: Some("Souvlaki".to_owned()),
        album_artist: Some("Slowdive".to_owned()),
        album_artists: Some(vec!["Slowdive".to_owned()]),
        genre: Some("Shoegaze".to_owned()),
        genres: Some(vec!["Shoegaze".to_owned()]),
        album_track_count: Some(10),
        disc_number: Some(1),
        disc_count: Some(1),
        lyricists: Some(vec!["Halstead".to_owned()]),
        user_rating_01: Some(0.8),
        user_rating_05: Some(4),
        auto_rating: Some(0.7),
        play_count: Some(108),
        skip_count: Some(23),
        media_url: Some("https://www.discogs.com/master/9478-Slowdive-Souvlaki".to_owned()),
        media_type_apple: Some(MediaTypeApple {
            music: true,
            ..Default::default()
        }),
        media_type_windows: Some(MediaTypeWindows::Music),
        // There are more metadata properties...
        ..Default::default()
    }
}

#[allow(dead_code)]
type Cover = <souvlaki::platform::OsImpl as souvlaki::MediaControls>::Cover;
#[allow(dead_code)]
pub fn cover() -> Option<Cover> {
    #[allow(dead_code)]
    const SOUVLAKI_COVER_URL: &'static str = "https://i.discogs.com/i7xH4rv3WwaRaG_ky3mlJCkCQZ18YnczTcNQs9aYpQ0/rs:fit/g:sm/q:90/h:600/w:589/czM6Ly9kaXNjb2dz/LWRhdGFiYXNlLWlt/YWdlcy9SLTgyOTIw/MC0xNTk1MzY1NzA2/LTUwNzUuanBlZw.jpeg";

    // MPRIS platform
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    {
        Some(souvlaki::platform::mpris::MprisCover::Url(
            SOUVLAKI_COVER_URL.to_owned(),
        ))
    }

    // macOS platform
    #[cfg(target_os = "macos")]
    {
        Some(souvlaki::platform::apple::AppleCover::Url(
            SOUVLAKI_COVER_URL.to_owned(),
        ))
    }

    // iOS platform (can't use URLs, only local files or bytes)
    #[cfg(target_os = "ios")]
    {
        todo!();
    }

    // Windows platform
    #[cfg(target_os = "windows")]
    {
        Some(souvlaki::platform::apple::AppleCover::Url(
            SOUVLAKI_COVER_URL.to_owned(),
        ))
    };

    // Dummy platform (for unsupported OSes)
    #[cfg(any(
        not(any(unix, target_os = "macos", target_os = "ios", target_os = "windows")),
        target_os = "android",
    ))]
    {
        None
    }
}

#[allow(dead_code)]
pub fn cover_bytes() -> Option<Cover> {
    const COVER_BYTES: &'static [u8] = include_bytes!("./cover.png");

    // MPRIS platform
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    {
        Some(souvlaki::platform::mpris::MprisCover::from_bytes(COVER_BYTES).unwrap())
    }

    // macOS/iOS platform
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        Some(souvlaki::platform::apple::AppleCover::Bytes(
            COVER_BYTES.to_vec(),
        ))
    }

    // Windows platform
    #[cfg(target_os = "windows")]
    {
        Some(souvlaki::platform::apple::WindowsCover::Bytes(
            COVER_BYTES.to_vec(),
        ))
    };

    // Dummy platform (for unsupported OSes)
    #[cfg(any(
        not(any(unix, target_os = "macos", target_os = "ios", target_os = "windows")),
        target_os = "android",
    ))]
    {
        None
    }
}
