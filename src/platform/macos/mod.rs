#![cfg(any(target_os = "macos", target_os = "ios"))]
#![allow(non_upper_case_globals)]

#[cfg(target_os = "ios")]
use std::fs;

use std::{
    ffi::CString,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use block::ConcreteBlock;
use core_graphics::geometry::CGSize;

use dispatch::{Queue, QueuePriority};
use objc::{
    class, msg_send,
    runtime::{BOOL, NO, Object, YES},
    sel, sel_impl,
};

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};

type Id = *mut Object;

/// A platform-specific error.
#[derive(Debug)]
pub struct Error;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

/// A handle to OS media controls.
pub struct MediaControls;

impl MediaControls {
    /// Create media controls with the specified config.
    pub fn new(_config: PlatformConfig) -> Result<Self, Error> {
        Ok(Self)
    }

    /// Attach the media control events to a handler.
    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        unsafe { attach_command_handlers(Arc::new(event_handler)) };
        Ok(())
    }

    /// Detach the event handler.
    pub fn detach(&mut self) -> Result<(), Error> {
        unsafe { detach_command_handlers() };
        Ok(())
    }

    /// Set the current playback status.
    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        unsafe { set_playback_status(playback) };
        Ok(())
    }

    /// Set the metadata of the currently playing media item.
    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        unsafe { set_playback_metadata(metadata) };
        Ok(())
    }

    /// Publish the current shuffle state and enable its remote command.
    ///
    /// Available on macOS/iOS. Call after [`Self::attach`]. `None` disables
    /// the command for a player that does not support shuffle. After a
    /// [`MediaControlEvent::SetShuffle`] request, publish the state the
    /// player actually accepted; the callback does not change it itself.
    pub fn set_shuffle(&mut self, shuffle: Option<bool>) -> Result<(), Error> {
        unsafe {
            let command_center: Id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);
            let command: Id = msg_send!(command_center, changeShuffleModeCommand);
            let _: () = msg_send!(command, setCurrentShuffleType: shuffle_type(shuffle));
            let _: () = msg_send!(command, setEnabled: if shuffle.is_some() { YES } else { NO });
        }
        Ok(())
    }

    /// Publish whether the current media item is liked and enable its command.
    ///
    /// Available on macOS/iOS. Call after [`Self::attach`]. `None` disables
    /// the command and clears its active state. After a
    /// [`MediaControlEvent::SetLike`] request, publish the state the player
    /// accepted. Receiving a request does not update the published state.
    pub fn set_like(&mut self, liked: Option<bool>) -> Result<(), Error> {
        unsafe {
            let command_center: Id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);
            let command: Id = msg_send!(command_center, likeCommand);
            let _: () = msg_send!(command, setActive: if liked == Some(true) { YES } else { NO });
            let _: () = msg_send!(command, setEnabled: if liked.is_some() { YES } else { NO });
        }
        Ok(())
    }
}

// MPNowPlayingPlaybackState
const MPNowPlayingPlaybackStatePlaying: usize = 1;
const MPNowPlayingPlaybackStatePaused: usize = 2;
const MPNowPlayingPlaybackStateStopped: usize = 3;

// MPRemoteCommandHandlerStatus
const MPRemoteCommandHandlerStatusSuccess: isize = 0;
const MPRemoteCommandHandlerStatusCommandFailed: isize = 200;

// MPShuffleType, as declared by MediaPlayer/MPRemoteControlTypes.h.
const MPShuffleTypeOff: isize = 0;
const MPShuffleTypeItems: isize = 1;
const MPShuffleTypeCollections: isize = 2;

fn shuffle_type(shuffle: Option<bool>) -> isize {
    if shuffle == Some(true) {
        MPShuffleTypeItems
    } else {
        MPShuffleTypeOff
    }
}

fn shuffle_event(enabled: bool, shuffle_type: isize) -> Option<MediaControlEvent> {
    if !enabled {
        return None;
    }
    match shuffle_type {
        MPShuffleTypeOff => Some(MediaControlEvent::SetShuffle(false)),
        MPShuffleTypeItems | MPShuffleTypeCollections => Some(MediaControlEvent::SetShuffle(true)),
        _ => None,
    }
}

fn like_event(enabled: bool, negative: bool) -> Option<MediaControlEvent> {
    enabled.then_some(MediaControlEvent::SetLike(!negative))
}

unsafe extern "C" {
    static MPMediaItemPropertyTitle: Id; // NSString
    static MPMediaItemPropertyArtist: Id; // NSString
    static MPMediaItemPropertyAlbumTitle: Id; // NSString
    static MPMediaItemPropertyArtwork: Id; // NSString
    static MPMediaItemPropertyPlaybackDuration: Id; // NSString
    static MPNowPlayingInfoPropertyElapsedPlaybackTime: Id; // NSString
}

unsafe fn set_playback_status(playback: MediaPlayback) {
    unsafe {
        let media_center: Id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
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
}

static GLOBAL_METADATA_COUNTER: AtomicUsize = AtomicUsize::new(1);

unsafe fn set_playback_metadata(metadata: MediaMetadata) {
    unsafe {
        let prev_counter = GLOBAL_METADATA_COUNTER.fetch_add(1, Ordering::SeqCst);
        let media_center: Id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
        let now_playing: Id = msg_send!(class!(NSMutableDictionary), dictionary);
        if metadata.cover_url.is_some() {
            let previous: Id = msg_send!(media_center, nowPlayingInfo);
            if !previous.is_null() {
                let artwork: Id = msg_send!(previous, objectForKey: MPMediaItemPropertyArtwork);
                if !artwork.is_null() {
                    let _: () = msg_send!(now_playing, setObject: artwork
                                                       forKey: MPMediaItemPropertyArtwork);
                }
            }
        }
        if let Some(title) = metadata.title {
            let _: () = msg_send!(now_playing, setObject: ns_string(title)
                                                  forKey: MPMediaItemPropertyTitle);
        }
        if let Some(artist) = metadata.artist {
            let _: () = msg_send!(now_playing, setObject: ns_string(artist)
                                                  forKey: MPMediaItemPropertyArtist);
        }
        if let Some(album) = metadata.album {
            let _: () = msg_send!(now_playing, setObject: ns_string(album)
                                                  forKey: MPMediaItemPropertyAlbumTitle);
        }
        if let Some(duration) = metadata.duration {
            let _: () = msg_send!(now_playing, setObject: ns_number(duration.as_secs_f64())
                                                  forKey: MPMediaItemPropertyPlaybackDuration);
        }
        let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
        if let Some(cover_url) = metadata.cover_url {
            let cover_url = cover_url.to_owned();
            Queue::global(QueuePriority::Default).exec_async(move || {
                load_and_set_playback_artwork(cover_url, prev_counter + 1);
            });
        }
    }
}

unsafe fn load_and_set_playback_artwork(url: String, for_counter: usize) {
    unsafe {
        let (image, size) = load_image_from_url(&url);
        let artwork = if image.is_null() {
            std::ptr::null_mut()
        } else {
            mp_artwork(image, size)
        };
        if GLOBAL_METADATA_COUNTER.load(Ordering::SeqCst) == for_counter {
            set_playback_artwork(artwork);
        }
    }
}

unsafe fn set_playback_artwork(artwork: Id) {
    unsafe {
        let media_center: Id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
        let now_playing: Id = msg_send!(class!(NSMutableDictionary), dictionary);
        let prev_now_playing: Id = msg_send!(media_center, nowPlayingInfo);
        let _: () = msg_send!(now_playing, addEntriesFromDictionary: prev_now_playing);
        if artwork.is_null() {
            let _: () = msg_send!(now_playing, removeObjectForKey: MPMediaItemPropertyArtwork);
        } else {
            let _: () = msg_send!(now_playing, setObject: artwork
                                                  forKey: MPMediaItemPropertyArtwork);
        }
        let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
    }
}

unsafe fn set_playback_progress(progress: Duration) {
    unsafe {
        let media_center: Id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
        let now_playing: Id = msg_send!(class!(NSMutableDictionary), dictionary);
        let prev_now_playing: Id = msg_send!(media_center, nowPlayingInfo);
        let _: () = msg_send!(now_playing, addEntriesFromDictionary: prev_now_playing);
        let _: () = msg_send!(now_playing, setObject: ns_number(progress.as_secs_f64())
                                              forKey: MPNowPlayingInfoPropertyElapsedPlaybackTime);
        let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
    }
}

unsafe fn attach_command_handlers(handler: Arc<dyn Fn(MediaControlEvent)>) {
    unsafe {
        let command_center: Id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

        // togglePlayPauseCommand
        let play_pause_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |_event: Id| -> isize {
                (handler)(MediaControlEvent::Toggle);
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, togglePlayPauseCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: play_pause_handler);

        // playCommand
        let play_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |_event: Id| -> isize {
                (handler)(MediaControlEvent::Play);
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, playCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: play_handler);

        // pauseCommand
        let pause_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |_event: Id| -> isize {
                (handler)(MediaControlEvent::Pause);
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, pauseCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: pause_handler);

        // previousTrackCommand
        let previous_track_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |_event: Id| -> isize {
                (handler)(MediaControlEvent::Previous);
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, previousTrackCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: previous_track_handler);

        // nextTrackCommand
        let next_track_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |_event: Id| -> isize {
                (handler)(MediaControlEvent::Next);
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, nextTrackCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: next_track_handler);

        // changePlaybackPositionCommand
        let position_handler = ConcreteBlock::new({
            let handler = handler.clone();
            // event of type MPChangePlaybackPositionCommandEvent
            move |event: Id| -> isize {
                let position = *event.as_ref().unwrap().get_ivar::<f64>("_positionTime");
                (handler)(MediaControlEvent::SetPosition(MediaPosition(
                    Duration::from_secs_f64(position),
                )));
                MPRemoteCommandHandlerStatusSuccess
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, changePlaybackPositionCommand);
        let _: () = msg_send!(cmd, setEnabled: YES);
        let _: () = msg_send!(cmd, addTargetWithHandler: position_handler);

        // changeShuffleModeCommand. Only players that publish a shuffle
        // state opt in; receiving a request never publishes unconfirmed state.
        let shuffle_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |event: Id| -> isize {
                // MediaRemote setters can still invoke disabled commands.
                // Keep the opt-in contract even for those callers.
                let command_center: Id =
                    msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);
                let command: Id = msg_send!(command_center, changeShuffleModeCommand);
                let enabled: BOOL = msg_send!(command, isEnabled);
                let shuffle_type: isize = msg_send!(event, shuffleType);
                match shuffle_event(enabled == YES, shuffle_type) {
                    Some(event) => {
                        (handler)(event);
                        MPRemoteCommandHandlerStatusSuccess
                    }
                    None => MPRemoteCommandHandlerStatusCommandFailed,
                }
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, changeShuffleModeCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, addTargetWithHandler: shuffle_handler);

        // likeCommand. MPFeedbackCommandEvent's negative operation removes
        // an existing like; it must not be interpreted as a dislike or toggle.
        let like_handler = ConcreteBlock::new({
            let handler = handler.clone();
            move |event: Id| -> isize {
                let command_center: Id =
                    msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);
                let command: Id = msg_send!(command_center, likeCommand);
                let enabled: BOOL = msg_send!(command, isEnabled);
                let negative: BOOL = msg_send!(event, isNegative);
                match like_event(enabled == YES, negative == YES) {
                    Some(event) => {
                        (handler)(event);
                        MPRemoteCommandHandlerStatusSuccess
                    }
                    None => MPRemoteCommandHandlerStatusCommandFailed,
                }
            }
        })
        .copy();
        let cmd: Id = msg_send!(command_center, likeCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, setActive: NO);
        let _: () = msg_send!(cmd, addTargetWithHandler: like_handler);
    }
}

unsafe fn detach_command_handlers() {
    unsafe {
        let command_center: Id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

        let cmd: Id = msg_send!(command_center, togglePlayPauseCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, changeShuffleModeCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, likeCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, setActive: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, playCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, pauseCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, previousTrackCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, nextTrackCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());

        let cmd: Id = msg_send!(command_center, changePlaybackPositionCommand);
        let _: () = msg_send!(cmd, setEnabled: NO);
        let _: () = msg_send!(cmd, removeTarget: std::ptr::null_mut::<Object>());
    }
}

unsafe fn ns_string(value: &str) -> Id {
    unsafe {
        let sanitized = value.replace('\0', "");
        let c_string = CString::new(sanitized).expect("sanitized strings contain no interior NUL");
        let string: Id = msg_send!(class!(NSString), stringWithUTF8String: c_string.as_ptr());
        string
    }
}

unsafe fn ns_number(value: f64) -> Id {
    unsafe {
        let number: Id = msg_send!(class!(NSNumber), numberWithDouble: value);
        number
    }
}

unsafe fn ns_url(value: &str) -> Id {
    unsafe {
        let url: Id = msg_send!(class!(NSURL), URLWithString: ns_string(value));
        url
    }
}

#[cfg(target_os = "ios")]
unsafe fn load_image_from_url(url: &str) -> (Id, CGSize) {
    unsafe {
        // iOS can only load artwork from local files, not HTTP.
        let path = url.strip_prefix("file://").unwrap_or(url);
        let Ok(image_data) = fs::read(path) else {
            return (std::ptr::null_mut::<Object>(), CGSize::new(0.0, 0.0));
        };
        let base64_data = base64::encode(image_data);
        let base64_ns_string = ns_string(&base64_data);

        let ns_data: Id = msg_send!(class!(NSData), alloc);
        let ns_data: Id = msg_send!(ns_data, initWithBase64EncodedString: base64_ns_string
                                              options: 0);
        if ns_data.is_null() {
            return (std::ptr::null_mut::<Object>(), CGSize::new(0.0, 0.0));
        }
        let image: Id = msg_send!(class!(UIImage), imageWithData: ns_data);
        if image.is_null() {
            return (std::ptr::null_mut::<Object>(), CGSize::new(0.0, 0.0));
        }
        let size: CGSize = msg_send!(image, size);
        (image, size)
    }
}

#[cfg(target_os = "macos")]
unsafe fn load_image_from_url(url: &str) -> (Id, CGSize) {
    unsafe {
        let url = ns_url(url);
        let image: Id = msg_send!(class!(NSImage), alloc);
        let image: Id = msg_send!(image, initWithContentsOfURL: url);
        if image.is_null() {
            return (std::ptr::null_mut(), CGSize::new(0.0, 0.0));
        }
        let size: CGSize = msg_send!(image, size);
        (image, CGSize::new(size.width, size.height))
    }
}

#[cfg(target_os = "ios")]
unsafe fn mp_artwork(image: Id, bounds: CGSize) -> Id {
    unsafe {
        let artwork: Id = msg_send!(class!(MPMediaItemArtwork), alloc);
        let artwork: Id = msg_send!(artwork, initWithImage: image);
        artwork
    }
}

#[cfg(target_os = "macos")]
unsafe fn mp_artwork(image: Id, bounds: CGSize) -> Id {
    unsafe {
        let handler = ConcreteBlock::new(move |_size: CGSize| -> Id { image }).copy();
        let artwork: Id = msg_send!(class!(MPMediaItemArtwork), alloc);
        let artwork: Id = msg_send!(artwork, initWithBoundsSize: bounds
                                             requestHandler: handler);
        artwork
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_requests_set_and_remove_likes_without_toggling() {
        assert_eq!(
            like_event(true, false),
            Some(MediaControlEvent::SetLike(true))
        );
        assert_eq!(
            like_event(true, false),
            Some(MediaControlEvent::SetLike(true))
        );
        assert_eq!(
            like_event(true, true),
            Some(MediaControlEvent::SetLike(false))
        );
    }

    #[test]
    fn disabled_like_does_not_forward_positive_or_negative_requests() {
        assert_eq!(like_event(false, false), None);
        assert_eq!(like_event(false, true), None);
    }

    #[test]
    fn native_shuffle_requests_preserve_off_and_both_shuffle_types() {
        assert_eq!(
            shuffle_event(true, 0),
            Some(MediaControlEvent::SetShuffle(false))
        );
        assert_eq!(
            shuffle_event(true, 1),
            Some(MediaControlEvent::SetShuffle(true))
        );
        assert_eq!(
            shuffle_event(true, 2),
            Some(MediaControlEvent::SetShuffle(true))
        );
        assert_eq!(shuffle_event(true, 3), None);
        assert_eq!(shuffle_event(true, -1), None);
        assert_eq!(shuffle_event(true, isize::MAX), None);
    }

    #[test]
    fn disabled_shuffle_does_not_forward_even_forced_media_remote_requests() {
        for mode in [-1, 0, 1, 2, 3, isize::MAX] {
            assert_eq!(shuffle_event(false, mode), None);
        }
    }

    #[test]
    fn published_shuffle_uses_items_and_clears_unsupported_state() {
        assert_eq!(shuffle_type(Some(true)), 1);
        assert_eq!(shuffle_type(Some(false)), 0);
        assert_eq!(shuffle_type(None), 0);
    }
}
