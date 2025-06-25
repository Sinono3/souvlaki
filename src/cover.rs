use std::path::PathBuf;

/// A way of specifying cover art/artwork/thumbnail.
/// This enum exists because the OSes handle this task differently in a fundamental way.
/// - **MPRIS:** Receives a `cover_url` metadata field, nothing more than this.
///     This could be an HTTP URL, a local file, or even a data URL.
pub enum MediaCover {
    /// Supported in MPRIS, macOS, Windows.
    #[cfg(any(platform_mpris, platform_macos, platform_windows))]
    HttpUrl(String),
    /// Supported in MPRIS, macOS, Windows.
    #[cfg(any(platform_mpris, platform_macos, platform_windows))]
    DataUrl(String),
    /// Supported in MPRIS, macOS/iOS, Windows.
    #[cfg(any(platform_mpris, platform_apple, platform_windows))]
    LocalFile(PathBuf),
    /// Supported in macOS, Windows.
    #[cfg(any(platform_apple, platform_windows))]
    Bytes(Vec<u8>),
}

// /// Location of artwork/album art image
// /// - **MPRIS:** `mpris:artUrl`
// /// - **macOS:** Unsupported (use `artwork` field)
// /// - **Windows:** Unsupported (use `thumbnail` field)
// pub art_url: Option<String>,

// /// Artwork image data/object
// /// - **MPRIS:** Unsupported (use `art_url` field)
// /// - **macOS:** `MPMediaItem.artwork` (MPMediaItemArtwork)
// /// - **Windows:** Unsupported (use `thumbnail` field)
// pub artwork: Option<Vec<u8>>, // Platform-specific artwork data

// /// Thumbnail image data/object
// /// - **MPRIS:** Unsupported (use `art_url` field)
// /// - **macOS:** Unsupported (use `artwork` field)
// /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.Thumbnail`
// pub thumbnail: Option<Vec<u8>>, // Platform-specific thumbnail data
