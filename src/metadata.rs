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
/// - **macOS**: [MPMediaItem](https://developer.apple.com/documentation/mediaplayer/mpmediaitem)
/// - **Windows**: [SystemMediaTransportControlsDisplayUpdater](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrolsdisplayupdater?view=winrt-26100), [SystemMediaTransportControls.UpdateTimelineProperties](https://learn.microsoft.com/en-us/uwp/api/windows.media.systemmediatransportcontrols.updatetimelineproperties?view=winrt-26100#windows-media-systemmediatransportcontrols-updatetimelineproperties(windows-media-systemmediatransportcontrolstimelineproperties))
///
/// Platform-specific fields aren't gated because they are cheap to construct or ignore.
/// They're just strings.
#[derive(Clone, Debug)]
pub struct MediaMetadata {
    /// Track/media title
    /// - **MPRIS:** `xesam:title`
    /// - **macOS:** `MPMediaItem.title`
    /// - **Windows:**
    ///     - `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Title`
    ///     - `SystemMediaTransportControlsDisplayUpdater.VideoProperties.Title`
    ///     - `SystemMediaTransportControlsDisplayUpdater.ImageProperties.Title`
    pub title: Option<String>,

    /// Track/song artist (single string, if multiple artists they are expected to all appear in this string)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.artist`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Artist`
    pub artist: Option<String>,

    /// Track/song artists (list of individual artists)
    /// - **MPRIS:** `xesam:artist`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub artists: Option<Vec<String>>,

    /// Album title
    /// - **MPRIS:** `xesam:album`
    /// - **macOS:** `MPMediaItem.albumTitle`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumTitle`
    pub album_title: Option<String>,

    /// Album artist (single string, if multiple artists they are expected to all appear in this string)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.albumArtist`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumArtist`
    pub album_artist: Option<String>,

    /// Album artists (list of individual artists)
    /// - **MPRIS:** `xesam:albumArtist`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub album_artists: Option<Vec<String>>,

    /// Genre (single string)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.genre`
    /// - **Windows:** Unsupported
    pub genre: Option<String>,

    /// Genres (list of genre names)
    /// - **MPRIS:** `xesam:genre`
    /// - **macOS:** Unsupported
    /// - **Windows:**
    ///     - `SystemMediaTransportControlsDisplayUpdater.MusicProperties.Genres`
    ///     - `SystemMediaTransportControlsDisplayUpdater.VideoProperties.Genres`
    pub genres: Option<Vec<String>>,

    /// Track number on the album/disc
    /// - **MPRIS:** `xesam:trackNumber`
    /// - **macOS:** `MPMediaItem.albumTrackNumber`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.TrackNumber`
    pub track_number: Option<i32>,

    /// Total number of tracks on the album
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.albumTrackCount`
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.MusicProperties.AlbumTrackCount`
    pub album_track_count: Option<i32>,

    /// Disc number of current track (for multi-disc albums)
    /// - **MPRIS:** `xesam:discNumber`
    /// - **macOS:** `MPMediaItem.discNumber`
    /// - **Windows:** Unsupported
    pub disc_number: Option<i32>,

    /// Total number of discs for the album
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.discCount`
    /// - **Windows:** Unsupported
    pub disc_count: Option<i32>,

    /// Track duration
    /// - **MPRIS:** `mpris:length` (converted to microseconds)
    /// - **macOS:** `MPMediaItem.playbackDuration` (converted to `TimeInterval`)
    /// - **Windows:** `SystemMediaTransportControlsTimelineProperties.MaxSeekTime`
    ///    (converted to `TimeSpan`. `MinSeekTime` is set to 0. `Position` is handled
    ///     by the [`MediaControls`] struct)
    pub duration: Option<Duration>,

    /// Composer(s) of the track
    /// - **MPRIS:** `xesam:composer`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub composers: Option<Vec<String>>,

    /// Composer (single string)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.composer`
    /// - **Windows:** Unsupported
    pub composer: Option<String>,

    /// Lyricist(s) of the track
    /// - **MPRIS:** `xesam:lyricist`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub lyricists: Option<Vec<String>>,

    /// Track lyrics
    /// - **MPRIS:** `xesam:asText`
    /// - **macOS:** `MPMediaItem.lyrics`
    /// - **Windows:** Unsupported
    pub lyrics: Option<String>,

    /// Comments about the media item
    /// - **MPRIS:** `xesam:comment`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub comments: Option<Vec<String>>,

    /// Comments about the media item (single string)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.comments`
    /// - **Windows:** Unsupported
    pub comment: Option<String>,

    /// Beats per minute (music tempo)
    /// - **MPRIS:** `xesam:audioBPM`
    /// - **macOS:** `MPMediaItem.beatsPerMinute`
    /// - **Windows:** Unsupported
    pub beats_per_minute: Option<i32>,

    /// User-specified rating of 0.0 to 1.0, inclusive
    /// - **MPRIS:** `xesam:userRating`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub user_rating_01: Option<f64>,

    /// User-specified rating of 0 to 5, inclusive
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.rating`
    /// - **Windows:** Unsupported
    pub user_rating_05: Option<i32>,

    /// Automatically-generated rating based on things such as how often it has been played, of 0.0 to 1.0, inclusive
    /// - **MPRIS:** `xesam:autoRating`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub auto_rating: Option<f64>,

    /// Number of times the track has been played
    /// - **MPRIS:** `xesam:useCount`
    /// - **macOS:** `MPMediaItem.playCount`
    /// - **Windows:** Unsupported
    pub play_count: Option<i32>,

    /// Number of times the track has been skipped
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.skipCount`
    /// - **Windows:** Unsupported
    pub skip_count: Option<i32>,

    /// When the track was created (usually only year is useful)
    /// - **MPRIS:** `xesam:contentCreated`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub content_created: Option<Date>,

    /// When the track was first played
    /// - **MPRIS:** `xesam:firstUsed`
    /// - **macOS:** Unsupported
    /// - **Windows:** Unsupported
    pub first_played: Option<Date>,

    /// When the track was last played
    /// - **MPRIS:** `xesam:lastUsed`
    /// - **macOS:** `MPMediaItem.lastPlayedDate`
    /// - **Windows:** Unsupported
    pub last_played: Option<Date>,

    /// Date the media item was added to the library
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.dateAdded`
    /// - **Windows:** Unsupported
    pub date_added: Option<Date>,

    /// Release date of the media item
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.releaseDate`
    /// - **Windows:** Unsupported
    pub release_date: Option<Date>,

    /// Location of the media file
    /// - **MPRIS:** `xesam:url`
    /// - **macOS:** `MPMediaItem.assetURL`
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
    // /// - **macOS:** Unsupported
    // /// - **Windows:** Unsupported
    // pub mpris_track_id: Option<String>,
    /// Persistent identifier for the media item
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.persistentID`
    /// - **Windows:** Unsupported
    pub media_persistent_id: Option<String>,

    /// Persistent identifier for the artist
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.artistPersistentID`
    /// - **Windows:** Unsupported
    pub artist_persistent_id: Option<String>,

    /// Persistent identifier for the album
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.albumPersistentID`
    /// - **Windows:** Unsupported
    pub album_persistent_id: Option<String>,

    /// Persistent identifier for the album artist
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.albumArtistPersistentID`
    /// - **Windows:** Unsupported
    pub album_artist_persistent_id: Option<String>,

    /// Persistent identifier for the composer
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.composerPersistentID`
    /// - **Windows:** Unsupported
    pub composer_persistent_id: Option<String>,

    /// Persistent identifier for the genre
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.genrePersistentID`
    /// - **Windows:** Unsupported
    pub genre_persistent_id: Option<String>,

    /// Persistent identifier for a podcast
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.podcastPersistentID`
    /// - **Windows:** Unsupported
    pub podcast_persistent_id: Option<String>,

    /// Media type for macOS (not an enum, but bitflags)
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.mediaType`
    /// - **Windows:** Unsupported
    pub media_type_macos: Option<MediaTypeMacos>,

    /// Bookmark time for user's most recent interaction
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.bookmarkTime` (converted to seconds)
    /// - **Windows:** Unsupported
    pub bookmark_time: Option<Duration>,

    /// Whether the media item is a cloud/streaming item
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.isCloudItem`
    /// - **Windows:** Unsupported
    pub is_cloud_item: Option<bool>,

    /// Whether the media item is part of a compilation
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.isCompilation`
    /// - **Windows:** Unsupported
    pub is_compilation: Option<bool>,

    /// Whether the media item is a preorder
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.isPreorder`
    /// - **Windows:** Unsupported
    pub is_preorder: Option<bool>,

    /// Whether the media item has explicit content
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.isExplicitItem`
    /// - **Windows:** Unsupported
    pub is_explicit: Option<bool>,

    /// "When the value is true, the media item has DRM protection."
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.hasProtectedAsset`
    /// - **Windows:** Unsupported
    pub has_protected_asset: Option<bool>,

    /// "The ID of a media item from the Apple Music catalog."
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.playbackStoreID`
    /// - **Windows:** Unsupported
    pub playback_store_id: Option<String>,

    /// Podcast title
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.podcastTitle`
    /// - **Windows:** Unsupported
    pub podcast_title: Option<String>,

    /// "Corresponds to the “Grouping” field in the Info tab in the Get Info dialog in iTunes."
    /// - **MPRIS:** Unsupported
    /// - **macOS:** `MPMediaItem.userGrouping`
    /// - **Windows:** Unsupported
    pub user_grouping: Option<String>,

    /// Media type for Windows
    /// - **MPRIS:** Unsupported
    /// - **macOS:** Unsupported
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.Type`
    pub media_type_windows: Option<MediaTypeWindows>,

    /// "Gets or sets the media id of the app."
    /// - **MPRIS:** Unsupported
    /// - **macOS:** Unsupported
    /// - **Windows:** `SystemMediaTransportControlsDisplayUpdater.AppMediaId`
    pub app_media_id: Option<String>,

    /// Video/image subtitle
    /// - **MPRIS:** Unsupported
    /// - **macOS:** Unsupported
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
            composer: None,
            lyricists: None,
            lyrics: None,
            comments: None,
            comment: None,
            beats_per_minute: None,
            user_rating_01: None,
            user_rating_05: None,
            auto_rating: None,
            play_count: None,
            skip_count: None,
            content_created: None,
            first_played: None,
            last_played: None,
            date_added: None,
            release_date: None,
            media_url: None,
            // For compatibility reasons
            // mpris_track_id: Some("/".to_owned()),
            media_persistent_id: None,
            artist_persistent_id: None,
            album_persistent_id: None,
            album_artist_persistent_id: None,
            composer_persistent_id: None,
            genre_persistent_id: None,
            podcast_persistent_id: None,
            media_type_macos: None,
            bookmark_time: None,
            is_cloud_item: None,
            is_compilation: None,
            is_preorder: None,
            is_explicit: None,
            has_protected_asset: None,
            playback_store_id: None,
            podcast_title: None,
            user_grouping: None,
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

/// Converted to a bitflag in the macOS implementation
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MediaTypeMacos {
    pub music: bool,
    pub podcast: bool,
    pub audio_book: bool,
    pub audio_itunes_u: bool,
    pub any_audio: bool,
    pub movie: bool,
    pub tv_show: bool,
    pub video_podcast: bool,
    pub music_video: bool,
    pub video_itunes_u: bool,
    pub home_video: bool,
    pub any_video: bool,
    pub any: bool,
}
