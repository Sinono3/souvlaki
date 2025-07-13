use std::time::Duration;

/// The metadata of a media item.
/// These contains the simple text-based, easy-to-represent metadata fields.
/// Thumbnails/artwork/cover art are not set here. See instead [`MediaControls::set_cover`].
///
/// The philosophy for this struct is: give fine-grained control to the library user.
/// Instead of the library doing the work of e.g. concatenating the `artists` field `Vec<String>`
/// into a `String` to support platforms which don't have first-class support for
/// multiple artists, we give you the option of filling these fields yourself.
///
/// However, if there are identical fields in multiple platforms
/// (e.g. song title, which is present on all three major platforms)
/// we do provide an unified field.
///
/// Some exceptions to the rule (where we do some conversion between formats, but losslessly)
/// are: `duration`.
///
/// The sources for these can be found in:
/// - **MPRIS:** [FreeDesktop - MPRIS v2 Metadata Guidelines](https://www.freedesktop.org/wiki/Specifications/mpris-spec/metadata/)
/// - **macOS/iOS**: [MPMediaItem](https://developer.apple.com/documentation/mediaplayer/mpmediaitem)
/// - **Windows**: [SystemMediaTransportControlsDisplayUpdater](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsdisplayupdater?view=winrt-26100), [SystemMediaTransportControls.UpdateTimelineProperties](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols.updatetimelineproperties?view=winrt-26100#windows-media-systemmediatransportcontrols-updatetimelineproperties(windows-media-systemmediatransportcontrolstimelineproperties))
///
/// Platform-specific fields aren't gated because they are cheap to construct or ignore.
/// They're just strings.
#[derive(Clone, Debug)]
pub struct MediaMetadata {
    /// Track/media title
    /// - **MPRIS:** `xesam:title`
    /// - **macOS/iOS:** `MPMediaItem.title`
    /// - **Windows:**
    ///     - `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Title`
    ///     - `SystemMediaTransportControlsDisplayUpdater.VideoProperties.Title`
    ///     - `SystemMediaTransportControlsDisplayUpdater.ImageProperties.Title`
    pub title: Option<String>,

    /// Track/song artist (single string, if multiple artists they are expected to all appear in this string)
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.artist`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Artist`
    pub artist: Option<String>,

    /// Track/song artists (list of individual artists)
    /// - **MPRIS:** `xesam:artist`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub artists: Option<Vec<String>>,

    /// Album title
    /// - **MPRIS:** `xesam:album`
    /// - **macOS/iOS:** `MPMediaItem.albumTitle`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumTitle`
    pub album_title: Option<String>,

    /// Album artist (single string, if multiple artists they are expected to all appear in this string)
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.albumArtist`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumArtist`
    pub album_artist: Option<String>,

    /// Album artists (list of individual artists)
    /// - **MPRIS:** `xesam:albumArtist`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub album_artists: Option<Vec<String>>,

    /// Genre (single string)
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.genre`
    /// - **Windows:** Unsupported
    pub genre: Option<String>,

    /// Genres (list of genre names)
    /// - **MPRIS:** `xesam:genre`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:**
    ///     - `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Genres`
    ///     - `SystemMediaTransportControlsDisplayUpdater.VideoProperties.Genres`
    pub genres: Option<Vec<String>>,

    /// Track number on the album/disc
    /// - **MPRIS:** `xesam:trackNumber`
    /// - **macOS/iOS:** `MPMediaItem.albumTrackNumber`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.TrackNumber`
    pub track_number: Option<i32>,

    /// Total number of tracks on the album
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.albumTrackCount`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumTrackCount`
    pub album_track_count: Option<i32>,

    /// Disc number of current track (for multi-disc albums)
    /// - **MPRIS:** `xesam:discNumber`
    /// - **macOS/iOS:** `MPMediaItem.discNumber`
    /// - **Windows:** Unsupported
    pub disc_number: Option<i32>,

    /// Total number of discs for the album
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.discCount`
    /// - **Windows:** Unsupported
    pub disc_count: Option<i32>,

    /// Track duration
    /// - **MPRIS:** `mpris:length` (converted to microseconds)
    /// - **macOS/iOS:** `MPMediaItem.playbackDuration` (converted to `TimeInterval`)
    /// - **Windows:** `SystemMediaTransportControlsTimelineProperties.MaxSeekTime`
    ///    (converted to `TimeSpan`. `MinSeekTime` is set to 0. `Position` is handled
    ///     by the [`MediaControls`] struct)
    pub duration: Option<Duration>,

    /// Composer(s) of the track
    /// - **MPRIS:** `xesam:composer`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub composers: Option<Vec<String>>,

    /// Lyricist(s) of the track
    /// - **MPRIS:** `xesam:lyricist`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub lyricists: Option<Vec<String>>,

    /// Track lyrics
    /// - **MPRIS:** `xesam:asText`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub lyrics: Option<String>,

    /// Comments about the media item
    /// - **MPRIS:** `xesam:comment`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub comments: Option<Vec<String>>,

    /// Beats per minute (music tempo)
    /// - **MPRIS:** `xesam:audioBPM`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub beats_per_minute: Option<i32>,

    /// User-specified rating of 0.0 to 1.0, inclusive
    /// - **MPRIS:** `xesam:userRating`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub user_rating_01: Option<f64>,

    /// Automatically-generated rating based on things such as how often it has been played, of 0.0 to 1.0, inclusive
    /// - **MPRIS:** `xesam:autoRating`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub auto_rating: Option<f64>,

    /// Number of times the track has been played
    /// - **MPRIS:** `xesam:useCount`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub play_count: Option<i32>,

    /// When the track was created (usually only year is useful)
    /// - **MPRIS:** `xesam:contentCreated`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub content_created: Option<Date>,

    /// When the track was first played
    /// - **MPRIS:** `xesam:firstUsed`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub first_played: Option<Date>,

    /// Location of the media file
    /// - **MPRIS:** `xesam:url`
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** Unsupported
    pub media_url: Option<String>,

    // TODO: Enable mpris_track_id setting. For now we just set it to "/"
    // /// Unique track D-Bus path
    // /// Expected to be a D-Bus path. The default value is "/".
    // /// This is supposed to be for playlist support in MPRIS,
    // /// (where each track has it's own D-Bus path),
    // /// but souvlaki needs to implement the service under the hood
    // /// for that to work.
    // /// For now, it makes sense to use the default and not change it.
    // /// - **MPRIS:** `mpris:trackid`
    // /// - **macOS/iOS:** Unsupported
    // /// - **Windows:** Unsupported
    // pub mpris_track_id: Option<String>,
    /// Persistent identifier for the media item
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** `MPMediaItem.persistentID`
    /// - **Windows:** Unsupported
    pub media_persistent_id: Option<u64>,

    /// Media type for Windows
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.Type`
    ///
    /// From the Windows docs:
    ///
    /// > ### Note
    /// > Apps should set a value for the Type property even if they aren't supplying other
    /// > media metadata to be displayed by the System Media Transport Controls. This value
    /// > helps the system handle your media content correctly, including preventing the
    /// > screen saver from activating during playback.
    pub media_type_windows: Option<MediaTypeWindows>,

    /// "Gets or sets the media id of the app."
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.AppMediaId`
    pub app_media_id: Option<String>,

    /// Video/image subtitle
    /// - **MPRIS:** Unsupported
    /// - **macOS/iOS:** Unsupported
    /// - **Windows:**
    ///     - `SystemMediaTransportControlsDisplayUpdater.VideoProperties.Subtitle`
    ///     - `SystemMediaTransportControlsDisplayUpdater.ImageProperties.Subtitle`
    pub subtitle: Option<String>,
}

impl Default for MediaMetadata {
    fn default() -> Self {
        Self {
            title: None,
            artist: None,
            artists: None,
            album_title: None,
            album_artist: None,
            album_artists: None,
            genre: None,
            genres: None,
            track_number: None,
            album_track_count: None,
            disc_number: None,
            disc_count: None,
            duration: None,
            composers: None,

            lyricists: None,
            lyrics: None,
            comments: None,
            beats_per_minute: None,
            user_rating_01: None,
            auto_rating: None,
            play_count: None,
            content_created: None,
            first_played: None,
            media_url: None,
            // For compatibility reasons
            // mpris_track_id: Some("/".to_owned()),
            media_persistent_id: None,
            media_type_windows: None,
            app_media_id: None,
            subtitle: None,
        }
    }
}

#[cfg(not(feature = "date"))]
pub type Date = ();

// TODO: Actually use a date crate here
#[cfg(feature = "date")]
pub type Date = ();

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaTypeWindows {
    Unknown = 0,
    Music = 1,
    Video = 2,
    Image = 3,
}
