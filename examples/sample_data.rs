use std::time::Duration;

use souvlaki::{MediaCover, MediaMetadata, MediaTypeMacos, MediaTypeWindows};

pub fn metadata() -> MediaMetadata {
    MediaMetadata {
        title: Some("Souvlaki Space Station".to_owned()),
        artist: Some("Slowdive".to_owned()),
        artists: Some(vec!["Slowdive".to_owned()]),
        album_title: Some("Souvlaki".to_owned()),
        album_artist: Some("Slowdive".to_owned()),
        album_artists: Some(vec!["Slowdive".to_owned()]),
        genre: Some("Shoegaze".to_owned()),
        genres: Some(vec!["Shoegaze".to_owned()]),
        track_number: Some(6),
        album_track_count: Some(10),
        disc_number: Some(1),
        disc_count: Some(1),
        duration: Some(Duration::from_micros(359_131_429)),
        lyricists: Some(vec!["Halstead".to_owned()]),
        user_rating_01: Some(0.8),
        user_rating_05: Some(4),
        auto_rating: Some(0.7),
        play_count: Some(108),
        skip_count: Some(23),
        media_url: Some("https://www.discogs.com/master/9478-Slowdive-Souvlaki".to_owned()),
        beats_per_minute: Some(138),
        media_type_macos: Some(MediaTypeMacos {
            music: true,
            ..Default::default()
        }),
        media_type_windows: Some(MediaTypeWindows::Music),
        ..Default::default()
    }
}

#[allow(dead_code)]
pub fn cover() -> MediaCover {
    // TODO: Correctly handle this
    MediaCover::HttpUrl(
        "https://www.discogs.com/master/9478-Slowdive-Souvlaki/image/SW1hZ2U6NDc3NzMyODA="
            .to_owned(),
    )
}

#[allow(dead_code)]
fn main() {
    println!(
        "This file is a library, which is not meant to be executed. Please use the other examples"
    )
}
