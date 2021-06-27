#![cfg(target_os = "macos")]
#![allow(non_upper_case_globals)]

use std::sync::Arc;

use block::ConcreteBlock;
use cocoa::{
    base::{id, nil, NO, YES},
    foundation::{NSInteger, NSString, NSUInteger},
};
use objc::{class, msg_send, sel, sel_impl};

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};

#[derive(Debug)]
pub struct Error;

pub struct MediaControls;

impl MediaControls {
    pub fn new() -> Self {
        Self
    }

    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        unsafe { attach_command_handlers(Arc::new(event_handler)) };
        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        unsafe { detach_command_handlers() };
        Ok(())
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        unsafe { set_playback_status(playback) };
        Ok(())
    }

    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        unsafe { set_playback_metadata(metadata) };
        Ok(())
    }
}

// MPNowPlayingPlaybackState
const MPNowPlayingPlaybackStatePlaying: NSUInteger = 1;
const MPNowPlayingPlaybackStatePaused: NSUInteger = 2;
const MPNowPlayingPlaybackStateStopped: NSUInteger = 3;

// MPRemoteCommandHandlerStatus
const MPRemoteCommandHandlerStatusSuccess: NSInteger = 0;

extern "C" {
    static MPMediaItemPropertyTitle: id; // NSString
    static MPMediaItemPropertyArtist: id; // NSString
    static MPMediaItemPropertyAlbumTitle: id; // NSString
}

unsafe fn set_playback_status(playback: MediaPlayback) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let state = match playback {
        MediaPlayback::Stopped => MPNowPlayingPlaybackStateStopped,
        MediaPlayback::Paused => MPNowPlayingPlaybackStatePaused,
        MediaPlayback::Playing => MPNowPlayingPlaybackStatePlaying,
    };
    let _: () = msg_send!(media_center, setPlaybackState: state);
}

unsafe fn set_playback_metadata(metadata: MediaMetadata) {
    let media_center: id = msg_send!(class!(MPNowPlayingInfoCenter), defaultCenter);
    let now_playing: id = msg_send!(class!(NSMutableDictionary), dictionary);
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
    let _: () = msg_send!(media_center, setNowPlayingInfo: now_playing);
}

unsafe fn attach_command_handlers(handler: Arc<dyn Fn(MediaControlEvent)>) {
    let command_center: id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

    // togglePlayPauseCommand
    let play_pause_handler = ConcreteBlock::new({
        let handler = handler.clone();
        move |_event: id| -> NSInteger {
            (handler)(MediaControlEvent::Toggle);
            MPRemoteCommandHandlerStatusSuccess
        }
    })
    .copy();
    let cmd: id = msg_send!(command_center, togglePlayPauseCommand);
    let _: () = msg_send!(cmd, setEnabled: YES);
    let _: () = msg_send!(cmd, addTargetWithHandler: play_pause_handler);

    // playCommand
    let play_handler = ConcreteBlock::new({
        let handler = handler.clone();
        move |_event: id| -> NSInteger {
            (handler)(MediaControlEvent::Play);
            MPRemoteCommandHandlerStatusSuccess
        }
    })
    .copy();
    let cmd: id = msg_send!(command_center, playCommand);
    let _: () = msg_send!(cmd, setEnabled: YES);
    let _: () = msg_send!(cmd, addTargetWithHandler: play_handler);

    // pauseCommand
    let pause_handler = ConcreteBlock::new({
        let handler = handler.clone();
        move |_event: id| -> NSInteger {
            (handler)(MediaControlEvent::Pause);
            MPRemoteCommandHandlerStatusSuccess
        }
    })
    .copy();
    let cmd: id = msg_send!(command_center, pauseCommand);
    let _: () = msg_send!(cmd, setEnabled: YES);
    let _: () = msg_send!(cmd, addTargetWithHandler: pause_handler);

    // previousTrackCommand
    let previous_track_handler = ConcreteBlock::new({
        let handler = handler.clone();
        move |_event: id| -> NSInteger {
            (handler)(MediaControlEvent::Previous);
            MPRemoteCommandHandlerStatusSuccess
        }
    })
    .copy();
    let cmd: id = msg_send!(command_center, previousTrackCommand);
    let _: () = msg_send!(cmd, setEnabled: YES);
    let _: () = msg_send!(cmd, addTargetWithHandler: previous_track_handler);

    // nextTrackCommand
    let next_track_handler = ConcreteBlock::new({
        let handler = handler.clone();
        move |_event: id| -> NSInteger {
            (handler)(MediaControlEvent::Next);
            MPRemoteCommandHandlerStatusSuccess
        }
    })
    .copy();
    let cmd: id = msg_send!(command_center, nextTrackCommand);
    let _: () = msg_send!(cmd, setEnabled: YES);
    let _: () = msg_send!(cmd, addTargetWithHandler: next_track_handler);
}

unsafe fn detach_command_handlers() {
    let command_center: id = msg_send!(class!(MPRemoteCommandCenter), sharedCommandCenter);

    let cmd: id = msg_send!(command_center, togglePlayPauseCommand);
    let _: () = msg_send!(cmd, setEnabled: NO);
    let _: () = msg_send!(cmd, removeTarget: nil);

    let cmd: id = msg_send!(command_center, playCommand);
    let _: () = msg_send!(cmd, setEnabled: NO);
    let _: () = msg_send!(cmd, removeTarget: nil);

    let cmd: id = msg_send!(command_center, pauseCommand);
    let _: () = msg_send!(cmd, setEnabled: NO);
    let _: () = msg_send!(cmd, removeTarget: nil);

    let cmd: id = msg_send!(command_center, previousTrackCommand);
    let _: () = msg_send!(cmd, setEnabled: NO);
    let _: () = msg_send!(cmd, removeTarget: nil);

    let cmd: id = msg_send!(command_center, nextTrackCommand);
    let _: () = msg_send!(cmd, setEnabled: NO);
    let _: () = msg_send!(cmd, removeTarget: nil);
}

unsafe fn ns_string(value: &str) -> id {
    NSString::alloc(nil).init_str(value)
}
