#![cfg(platform_apple)]
#![allow(non_upper_case_globals)]

use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use block::ConcreteBlock;
use cocoa::{
    base::{id, nil, NO, YES},
    foundation::{NSInteger, NSString, NSUInteger},
};
use core_graphics::geometry::CGSize;

use dispatch::{Queue, QueuePriority};
use objc::{class, msg_send, sel, sel_impl};

use crate::{
    controls::MediaControls, MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition,
};

/// A platform-specific error.
#[derive(Debug, thiserror::Error)]
pub enum AppleError {
    // TODO: They *could* be supported, though, can't they?
    #[error("Non UTF-8 paths are not supported for cover art loading")]
    NonUtf8Path,
}

/// A handle to Apple's MPRemoteCommandCenter and the NowPlaying interface
#[derive(Debug)]
pub struct Apple;

pub type OsImpl = Apple;

/// Definition/reference to cover art for Apple platforms.
/// Differs depending on whether it's macOS or iOS.
#[cfg(platform_macos)]
#[derive(Clone, Debug)]
pub enum AppleCover {
    /// Available only on macOS.
    /// May work with HTTP URLs, data URLs, file URLs. Hasn't been tested with others.
    #[cfg(platform_macos)]
    Url(String),
    /// Available on macOS/iOS.
    /// If the file is not found, it will silently fail and display a blank
    /// image as the artwork.
    /// As of currently, receiving errors from
    /// async calls to macOS is not implemented.
    LocalFile(PathBuf),
    /// Available on macOS/iOS.
    /// If the bytes are not recognized as an image,
    /// it will silently fail and display a blank image as the artwork.
    /// As of currently, receiving errors from
    /// async calls to macOS is not implemented.
    Bytes(Vec<u8>),
}

impl MediaControls for Apple {
    type Error = AppleError;
    type PlatformConfig = ();
    type Cover = AppleCover;

    fn new(_config: Self::PlatformConfig) -> Result<Self, AppleError> {
        Ok(Self)
    }

    fn attach<F>(&mut self, event_handler: F) -> Result<(), AppleError>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        unsafe { attach_command_handlers(Arc::new(event_handler)) };
        Ok(())
    }

    fn detach(&mut self) -> Result<(), AppleError> {
        unsafe { detach_command_handlers() };
        Ok(())
    }

    fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), AppleError> {
        unsafe { set_playback_status(playback) };
        Ok(())
    }

    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), AppleError> {
        unsafe { set_playback_metadata(metadata) };
        Ok(())
    }

    fn set_cover(&mut self, cover: Option<Self::Cover>) -> Result<(), Self::Error> {
        let prev_counter = GLOBAL_METADATA_COUNTER.fetch_add(1, Ordering::SeqCst);

        match cover {
            // Available only on macOS
            #[cfg(platform_macos)]
            Some(AppleCover::Url(cover_url)) => {
                load_and_set_artwork(
                    move || unsafe { load_image_from_url(&cover_url) },
                    prev_counter + 1,
                );
            }
            // Available on macOS/iOS
            Some(AppleCover::LocalFile(cover_path)) => {
                let cover_path = cover_path
                    .to_str()
                    .ok_or(AppleError::NonUtf8Path)?
                    .to_owned();

                load_and_set_artwork(
                    move || unsafe { load_image_from_path(&cover_path) },
                    prev_counter + 1,
                );
            }
            // Available on macOS/iOS
            Some(AppleCover::Bytes(bytes)) => {
                println!("?");
                load_and_set_artwork(
                    move || unsafe { load_image_from_bytes(&bytes) },
                    prev_counter + 1,
                );
            }
            None => unsafe {
                set_playback_artwork(nil);
            },
        };

        Ok(())
    }
}

// MPNowPlayingPlaybackState
const MPNowPlayingPlaybackStatePlaying: NSUInteger = 1;
const MPNowPlayingPlaybackStatePaused: NSUInteger = 2;
const MPNowPlayingPlaybackStateStopped: NSUInteger = 3;

// MPRemoteCommandHandlerStatus
const MPRemoteCommandHandlerStatusSuccess: NSInteger = 0;

#[allow(dead_code)]
extern "C" {
    /// [NSString] The playback duration of the media item.
    static MPMediaItemPropertyPlaybackDuration: id;
    /// [NSString] The track number of the media item, for a media item that is part of an album.
    static MPMediaItemPropertyAlbumTrackNumber: id;
    /// [NSString] The number of tracks for the album that contains the media item.
    static MPMediaItemPropertyAlbumTrackCount: id;
    /// [NSString] The disc number of the media item, for a media item that is part of a multidisc album.
    static MPMediaItemPropertyDiscNumber: id;
    /// [NSString] The number of discs for the album that contains the media item.
    static MPMediaItemPropertyDiscCount: id;
    /// [NSString] The artwork image for the media item.
    static MPMediaItemPropertyArtwork: id;
    /// [NSString] The lyrics for the media item.
    static MPMediaItemPropertyLyrics: id;
    /// [NSString] The date of the media item’s first public release.
    static MPMediaItemPropertyReleaseDate: id;
    /// [NSString] The number of musical beats per minute for the media item.
    static MPMediaItemPropertyBeatsPerMinute: id;
    /// [NSString] Textual information about the media item.
    static MPMediaItemPropertyComments: id;
    /// [NSString] A URL that points to the media item.
    static MPMediaItemPropertyAssetURL: id;
    /// [NSString] A Boolean value that indicates whether the media item contains explicit (adult) lyrics or language.
    static MPMediaItemPropertyIsExplicit: id;
    /// [NSString] A Boolean value that indicates whether the media item is a preorder.
    static MPMediaItemPropertyIsPreorder: id;
    /// [NSString] The identifier for enqueueing store tracks.
    static MPMediaItemPropertyPlaybackStoreID: id;
    /// [NSString] The primary performing artist for an album.
    static MPMediaItemPropertyAlbumArtist: id;
    /// [NSString] The persistent identifier for an album artist.
    static MPMediaItemPropertyAlbumArtistPersistentID: id;
    /// [NSString] The key for the persistent identifier for an album.
    static MPMediaItemPropertyAlbumPersistentID: id;
    /// [NSString] The title of an album.
    static MPMediaItemPropertyAlbumTitle: id;
    /// [NSString] The performing artists for a media item — which may vary from the primary artist for the album that a media item belongs to.
    static MPMediaItemPropertyArtist: id;
    /// [NSString] The key for the persistent identifier for an artist.
    static MPMediaItemPropertyArtistPersistentID: id;
    /// [NSString] The musical composer for the media item.
    static MPMediaItemPropertyComposer: id;
    /// [NSString] The persistent identifier for a composer.
    static MPMediaItemPropertyComposerPersistentID: id;
    /// [NSString] The music or film genre of the media item.
    static MPMediaItemPropertyGenre: id;
    /// [NSString] The persistent identifier for a genre.
    static MPMediaItemPropertyGenrePersistentID: id;
    /// [NSString] A Boolean value that indicates the media item has DRM protection so it can’t play through a standard playback API.
    static MPMediaItemPropertyHasProtectedAsset: id;
    /// [NSString] A Boolean value that indicates whether the media item is part of a compilation.
    static MPMediaItemPropertyIsCompilation: id;
    /// [NSString] A Boolean value that indicates whether the media item is an iCloud item.
    static MPMediaItemPropertyIsCloudItem: id;
    /// [NSString] The media type of the media item.
    static MPMediaItemPropertyMediaType: id;
    /// [NSString] The key for the persistent identifier for the media item.
    static MPMediaItemPropertyPersistentID: id;
    /// [NSString] The number of times the user plays the media item.
    static MPMediaItemPropertyPlayCount: id;
    /// [NSString] The persistent identifier for an audio podcast.
    static MPMediaItemPropertyPodcastPersistentID: id;
    /// [NSString] The title of a podcast.
    static MPMediaItemPropertyPodcastTitle: id;
    /// [NSString] The title or name of the media item.
    static MPMediaItemPropertyTitle: id;
    /// [NSString] The number of times the user has skipped playing the item.
    static MPMediaItemPropertySkipCount: id;
    /// [NSString] The user-specified rating of the object in the range [0...5], where a value of 5 indicates the most favorable rating.
    static MPMediaItemPropertyRating: id;
    /// [NSString] The most recent calendar date on which the user played the media item.
    static MPMediaItemPropertyLastPlayedDate: id;
    /// [NSString] Corresponds to the “Grouping” field in the Info tab in the Get Info dialog in iTunes.
    static MPMediaItemPropertyUserGrouping: id;
    /// [NSString] The user’s place in the media item the most recent time it was played.
    static MPMediaItemPropertyBookmarkTime: id;
    /// [NSString] The date the media item was added to the user’s Media library.
    static MPMediaItemPropertyDateAdded: id;
    /// [NSString] The identifier of the collection the Now Playing item belongs to.
    static MPNowPlayingInfoCollectionIdentifier: id;
    /// [NSString] A list of ad breaks in the Now Playing item.
    static MPNowPlayingInfoPropertyAdTimeRanges: id;
    /// [NSString] The available language option groups for the Now Playing item.
    static MPNowPlayingInfoPropertyAvailableLanguageOptions: id;
    /// [NSString] The URL pointing to the Now Playing item’s underlying asset.
    static MPNowPlayingInfoPropertyAssetURL: id;
    /// [NSString] The total number of chapters in the Now Playing item.
    static MPNowPlayingInfoPropertyChapterCount: id;
    /// [NSString] The number corresponding to the currently playing chapter.
    static MPNowPlayingInfoPropertyChapterNumber: id;
    /// [NSString] The start time for the credits, in seconds, without ads, for the Now Playing item.
    static MPNowPlayingInfoPropertyCreditsStartTime: id;
    /// [NSString] The currently active language options for the Now Playing item.
    static MPNowPlayingInfoPropertyCurrentLanguageOptions: id;
    /// [NSString] The date associated with the current elapsed playback time.
    static MPNowPlayingInfoPropertyCurrentPlaybackDate: id;
    /// [NSString] The default playback rate for the Now Playing item.
    static MPNowPlayingInfoPropertyDefaultPlaybackRate: id;
    /// [NSString] The elapsed time of the Now Playing item, in seconds.
    static MPNowPlayingInfoPropertyElapsedPlaybackTime: id;
    /// [NSString] A number that denotes whether to exclude the Now Playing item from content suggestions.
    static MPNowPlayingInfoPropertyExcludeFromSuggestions: id;
    /// [NSString] The opaque identifier that uniquely identifies the Now Playing item, even through app relaunches.
    static MPNowPlayingInfoPropertyExternalContentIdentifier: id;
    /// [NSString] The opaque identifier that uniquely identifies the profile the Now Playing item plays from, even through app relaunches.
    static MPNowPlayingInfoPropertyExternalUserProfileIdentifier: id;
    /// [NSString] The International Standard Recording Code (ISRC) of the Now Playing item.
    static MPNowPlayingInfoPropertyInternationalStandardRecordingCode: id;
    /// [NSString] A number that denotes whether the Now Playing item is a live stream.
    static MPNowPlayingInfoPropertyIsLiveStream: id;
    /// [NSString] The media type of the Now Playing item.
    static MPNowPlayingInfoPropertyMediaType: id;
    /// [NSString] The current progress of the Now Playing item.
    static MPNowPlayingInfoPropertyPlaybackProgress: id;
    /// [NSString] The playback rate of the Now Playing item.
    static MPNowPlayingInfoPropertyPlaybackRate: id;
    /// [NSString] The total number of items in the app’s playback queue.
    static MPNowPlayingInfoPropertyPlaybackQueueCount: id;
    /// [NSString] The index of the Now Playing item in the app’s playback queue.
    static MPNowPlayingInfoPropertyPlaybackQueueIndex: id;
    /// [NSString] The service provider associated with the Now Playing item.
    static MPNowPlayingInfoPropertyServiceIdentifier: id;
}

unsafe fn set_playback_status(playback: MediaPlayback) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let state = match playback {
        MediaPlayback::Stopped => MPNowPlayingPlaybackStateStopped,
        MediaPlayback::Paused { .. } => MPNowPlayingPlaybackStatePaused,
        MediaPlayback::Playing { .. } => MPNowPlayingPlaybackStatePlaying,
    };
    let _: () = msg_send!(media_center, setPlaybackState: state);
    if let MediaPlayback::Paused {
        progress: Some(progress),
    }
    | MediaPlayback::Playing {
        progress: Some(progress),
    } = playback
    {
        set_playback_progress(progress.0);
    }
}

static GLOBAL_METADATA_COUNTER: AtomicUsize = AtomicUsize::new(1);

unsafe fn set_playback_metadata(metadata: MediaMetadata) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let now_playing: id = msg_send!(class!(NSMutableDictionary), dictionary);

    macro_rules! set_metadata {
        ($constructor:path, $val:expr, $key:expr) => {
            if let Some(value) = $val {
                let _: () = msg_send!(now_playing, setObject: ($constructor)(value) forKey: $key);
            }
        }
    }
    let MediaMetadata {
        ref title,
        ref artist,
        ref album_title,
        ref album_artist,
        ref genre,
        track_number,
        album_track_count,
        disc_number,
        disc_count,
        duration,
        ref composer,
        ref lyrics,
        ref comment,
        beats_per_minute,
        user_rating_05,
        play_count,
        skip_count,
        ref media_url,
        media_persistent_id,
        artist_persistent_id,
        album_persistent_id,
        album_artist_persistent_id,
        composer_persistent_id,
        genre_persistent_id,
        podcast_persistent_id,
        // TODO: media_type_macos,
        bookmark_time,
        is_cloud_item,
        is_compilation,
        is_preorder,
        is_explicit,
        has_protected_asset,
        ref playback_store_id,
        ref podcast_title,
        ref user_grouping,
        ..
    } = metadata;
    let duration = duration.map(|x| x.as_secs_f64());
    let bookmark_time = bookmark_time.map(|x| x.as_secs_f64());

    set_metadata!(ns_string, title, MPMediaItemPropertyTitle);
    set_metadata!(ns_string, artist, MPMediaItemPropertyArtist);
    set_metadata!(ns_string, album_title, MPMediaItemPropertyAlbumTitle);
    set_metadata!(ns_string, album_artist, MPMediaItemPropertyAlbumArtist);
    set_metadata!(ns_string, genre, MPMediaItemPropertyGenre);
    set_metadata!(
        ns_number_int,
        track_number,
        MPMediaItemPropertyAlbumTrackNumber
    );
    set_metadata!(
        ns_number_int,
        album_track_count,
        MPMediaItemPropertyAlbumTrackCount
    );
    set_metadata!(ns_number_int, disc_number, MPMediaItemPropertyDiscNumber);
    set_metadata!(ns_number_int, disc_count, MPMediaItemPropertyDiscCount);
    set_metadata!(ns_number_f64, duration, MPMediaItemPropertyPlaybackDuration);
    set_metadata!(ns_string, composer, MPMediaItemPropertyComposer);
    set_metadata!(ns_string, lyrics, MPMediaItemPropertyLyrics);
    set_metadata!(ns_string, comment, MPMediaItemPropertyComments);
    set_metadata!(
        ns_number_int,
        beats_per_minute,
        MPMediaItemPropertyBeatsPerMinute
    );
    set_metadata!(ns_number_int, user_rating_05, MPMediaItemPropertyRating);
    set_metadata!(ns_number_int, play_count, MPMediaItemPropertyPlayCount);
    set_metadata!(ns_number_int, skip_count, MPMediaItemPropertySkipCount);
    set_metadata!(ns_url, media_url, MPMediaItemPropertyAssetURL);
    set_metadata!(
        ns_number_u64,
        media_persistent_id,
        MPMediaItemPropertyPersistentID
    );
    set_metadata!(
        ns_number_u64,
        artist_persistent_id,
        MPMediaItemPropertyArtistPersistentID
    );
    set_metadata!(
        ns_number_u64,
        album_persistent_id,
        MPMediaItemPropertyAlbumPersistentID
    );
    set_metadata!(
        ns_number_u64,
        album_artist_persistent_id,
        MPMediaItemPropertyAlbumArtistPersistentID
    );
    set_metadata!(
        ns_number_u64,
        composer_persistent_id,
        MPMediaItemPropertyComposerPersistentID
    );
    set_metadata!(
        ns_number_u64,
        genre_persistent_id,
        MPMediaItemPropertyGenrePersistentID
    );
    set_metadata!(
        ns_number_u64,
        podcast_persistent_id,
        MPMediaItemPropertyPodcastPersistentID
    );
    // TODO:
    // set_metadata!(
    //     ns_u64,
    //     media_type_macos,
    //     MPMediaItemPropertyMediaTypeMacos
    // );
    set_metadata!(
        ns_number_f64,
        bookmark_time,
        MPMediaItemPropertyBookmarkTime
    );
    set_metadata!(ns_bool, is_cloud_item, MPMediaItemPropertyIsCloudItem);
    set_metadata!(ns_bool, is_compilation, MPMediaItemPropertyIsCompilation);
    set_metadata!(ns_bool, is_preorder, MPMediaItemPropertyIsPreorder);
    set_metadata!(ns_bool, is_explicit, MPMediaItemPropertyIsExplicit);
    set_metadata!(
        ns_bool,
        has_protected_asset,
        MPMediaItemPropertyHasProtectedAsset
    );
    set_metadata!(
        ns_string,
        playback_store_id,
        MPMediaItemPropertyPlaybackStoreID
    );
    set_metadata!(ns_string, podcast_title, MPMediaItemPropertyPodcastTitle);
    set_metadata!(ns_string, user_grouping, MPMediaItemPropertyUserGrouping);

    // TODO: date support
    // let MediaMetadata {
    //     last_played,
    //     date_added,
    //     release_date,
    //     ..
    // };

    // Update the NowPlaying info
    let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
}

unsafe fn set_playback_artwork(artwork: id) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let now_playing: id = msg_send!(class!(NSMutableDictionary), dictionary);
    let prev_now_playing: id = msg_send!(media_center, nowPlayingInfo);
    let _: () = msg_send!(now_playing, addEntriesFromDictionary: prev_now_playing);
    let _: () = msg_send!(now_playing, setObject: artwork
                                          forKey: MPMediaItemPropertyArtwork);
    let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
}

unsafe fn set_playback_progress(progress: Duration) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let now_playing: id = msg_send!(class!(NSMutableDictionary), dictionary);
    let prev_now_playing: id = msg_send!(media_center, nowPlayingInfo);
    let _: () = msg_send!(now_playing, addEntriesFromDictionary: prev_now_playing);
    let _: () = msg_send!(now_playing, setObject: ns_number_f64(progress.as_secs_f64())
                                          forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime);
    let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
}

unsafe fn attach_command_handlers(handler: Arc<dyn Fn(MediaControlEvent)>) {
    let command_center: id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

    macro_rules! attach {
        ($id:ident, $handler:expr) => {
            let cb_handler = ConcreteBlock::new($handler).copy();
            let cmd: id = msg_send!(command_center, $id);
            let _: () = msg_send!(cmd, setEnabled: YES);
            let _: () = msg_send!(cmd, addTargetWithHandler: cb_handler);
        };
    }
    macro_rules! attach_simple {
        ($id:ident, $event:expr) => {
            attach!($id, {
                let handler = handler.clone();
                move |_event: id| -> NSInteger {
                    (handler)($event);
                    MPRemoteCommandHandlerStatusSuccess
                }
            });
        };
    }

    attach_simple!(togglePlayPauseCommand, MediaControlEvent::Toggle);
    attach_simple!(playCommand, MediaControlEvent::Play);
    attach_simple!(pauseCommand, MediaControlEvent::Pause);
    attach_simple!(stopCommand, MediaControlEvent::Stop);
    attach_simple!(previousTrackCommand, MediaControlEvent::Previous);
    attach_simple!(nextTrackCommand, MediaControlEvent::Next);
    attach!(changePlaybackPositionCommand, {
        let handler = handler.clone();
        // event of type MPChangePlaybackPositionCommandEvent
        move |event: id| -> NSInteger {
            let position = *event.as_ref().unwrap().get_ivar::<f64>("_positionTime");
            (handler)(MediaControlEvent::SetPosition(MediaPosition(
                Duration::from_secs_f64(position),
            )));
            MPRemoteCommandHandlerStatusSuccess
        }
    });
}

unsafe fn detach_command_handlers() {
    let command_center: id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

    macro_rules! detach {
        ($id:ident) => {
            let cmd: id = msg_send!(command_center, $id);
            let _: () = msg_send!(cmd, setEnabled: NO);
            let _: () = msg_send!(cmd, removeTarget: nil);
        };
    }

    detach!(togglePlayPauseCommand);
    detach!(playCommand);
    detach!(pauseCommand);
    detach!(stopCommand);
    detach!(previousTrackCommand);
    detach!(nextTrackCommand);
    detach!(changePlaybackPositionCommand);
}

unsafe fn ns_string(value: &str) -> id {
    NSString::alloc(nil).init_str(value)
}

unsafe fn ns_number_f64(value: f64) -> id {
    msg_send!(class!(NSNumber), numberWithDouble: value)
}

unsafe fn ns_number_int(value: i32) -> id {
    msg_send!(class!(NSNumber), numberWithInteger: value)
}

unsafe fn ns_number_u64(value: u64) -> id {
    msg_send!(class!(NSNumber), numberWithUnsignedLong: value)
}

// TODO: is this okay?
unsafe fn ns_bool(value: bool) -> id {
    msg_send!(class!(NsNumber), numberWithBool: value)
}

unsafe fn ns_url(value: &str) -> id {
    msg_send!(class!(NSURL), URLWithString: ns_string(value))
}

fn load_and_set_artwork<F>(loader: F, for_counter: usize)
where
    F: FnOnce() -> (id, CGSize) + Send + Sync + 'static,
{
    Queue::global(QueuePriority::Default).exec_async(move || unsafe {
        let (image, size) = loader();
        let artwork = mp_artwork(image, size);
        if GLOBAL_METADATA_COUNTER.load(Ordering::SeqCst) == for_counter {
            set_playback_artwork(artwork);
        }
    });
}

#[cfg(platform_ios)]
unsafe fn load_image_from_path(path: &str) -> (id, CGSize) {
    use base64::Engine;
    use std::fs;
    let image_data = fs::read(&path).unwrap();
    let engine = base64::engine::general_purpose::URL_SAFE;
    let base64_data = engine.encode(image_data);
    let base64_ns_string = ns_string(&base64_data);
    let ns_data: id = msg_send!(class!(NSData), alloc);
    let ns_data: id = msg_send!(ns_data, initWithBase64EncodedString: base64_ns_string
                                          options: 0);
    if ns_data == nil {
        return (nil, CGSize::new(0.0, 0.0));
    }
    let image: id = msg_send!(class!(UIImage), imageWithData: ns_data);
    if image == nil {
        return (nil, CGSize::new(0.0, 0.0));
    }
    let size: CGSize = msg_send!(image, size);
    (image, size)
}

#[cfg(platform_macos)]
unsafe fn load_image_from_path(path: &str) -> (id, CGSize) {
    let path = ns_string(path);
    let image: id = msg_send!(class!(NSImage), alloc);
    let image: id = msg_send!(image, initWithContentsOfFile: path);
    let size: CGSize = msg_send!(image, size);
    (image, CGSize::new(size.width, size.height))
}

#[cfg(platform_macos)]
unsafe fn load_image_from_url(url: &str) -> (id, CGSize) {
    let url = ns_url(url);
    let image: id = msg_send!(class!(NSImage), alloc);
    let image: id = msg_send!(image, initWithContentsOfURL: url);
    let size: CGSize = msg_send!(image, size);
    (image, CGSize::new(size.width, size.height))
}

unsafe fn load_image_from_bytes(image_data: &[u8]) -> (id, CGSize) {
    // TODO: Change to use unsafe raw pointer
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    let base64_data = engine.encode(image_data);
    let base64_ns_string = ns_string(&base64_data);
    let ns_data: id = msg_send!(class!(NSData), alloc);
    let ns_data: id = msg_send!(ns_data, initWithBase64EncodedString: base64_ns_string
                                          options: 0);
    if ns_data == nil {
        return (nil, CGSize::new(0.0, 0.0));
    }

    #[cfg(platform_macos)]
    let image: id = {
        let image: id = msg_send!(class!(NSImage), alloc);
        let image: id = msg_send!(image, initWithData: ns_data);
        image
    };
    #[cfg(platform_ios)]
    let image: id = msg_send!(class!(UIImage), imageWithData: ns_data);

    if image == nil {
        return (nil, CGSize::new(0.0, 0.0));
    }
    let size: CGSize = msg_send!(image, size);
    (image, size)
}

unsafe fn mp_artwork(image: id, bounds: CGSize) -> id {
    let handler = ConcreteBlock::new(move |_size: CGSize| -> id { image }).copy();
    let artwork: id = msg_send!(class!(MPMediaItemArtwork), alloc);
    let artwork: id = msg_send!(artwork, initWithBoundsSize: bounds
                                         requestHandler: handler);
    artwork
}
