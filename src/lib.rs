#![doc = include_str!("../README.md")]

use std::{fmt::Debug, time::Duration};

mod controls;
mod cover;
/// Contains traits which extend MediaControls by adding additional methods
/// available only in specific OSes.
pub mod extensions;
mod metadata;
/// The platform-specific implementations of the media controls.
pub mod platform;

pub use controls::{MediaControls, MediaControlsWrapper};
pub use cover::MediaCover;
pub use metadata::*;

pub type OsMediaControls = controls::MediaControlsWrapper<crate::platform::OsImpl>;

/// Events caused by the user interacting with the OS media controls.
#[derive(Clone, PartialEq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,

    /// Seek forward or backward by an undetermined amount.
    Seek(SeekDirection),
    /// Seek forward or backward by a certain amount.
    SeekBy(SeekDirection, Duration),
    /// Set the position/progress of the currently playing media item.
    SetPosition(MediaPosition),
    /// Set the volume. The value is intended to be from 0.0 to 1.0.
    /// But other values are also accepted. **It is up to the user to
    /// set constraints on this value.**
    /// **NOTE**: If the event was received and correctly handled,
    /// [`MediaControls::set_volume`] must be called. Note that
    /// this must be done only with the MPRIS backend.
    SetVolume(f64),
    /// Set the playback rate.
    /// **NOTE**: If the event was received and correctly handled,
    /// [`MediaControls::set_rate`] must be called. Note that
    /// this must be done only with the MPRIS backend.
    SetPlaybackRate(f64),
    /// Set shuffle on or off.
    /// **NOTE**: If the event was received and correctly handled,
    /// [`MediaControls::set_shuffle`] must be called. Note that
    /// this must be done only with the MPRIS backend.
    SetShuffle(bool),
    /// Set loop status of the media item: (none, loop track, loop playlist)
    /// **NOTE**: If the event was received and correctly handled,
    /// [`MediaControls::set_loop`] must be called. Note that
    /// this must be done only with the MPRIS backend.
    SetLoop(Loop),

    /// Open the URI in the media player.
    OpenUri(String),
    /// Bring the media player's user interface to the front using any appropriate mechanism available.
    Raise,
    /// Shut down the media player.
    Quit,
}
/// An instant in a media item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MediaPosition(pub Duration);

/// The status of media playback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused { progress: Option<MediaPosition> },
    Playing { progress: Option<MediaPosition> },
}

impl MediaPlayback {
    pub fn to_dbus_value(&self) -> &'static str {
        use MediaPlayback::*;
        match self {
            Playing { .. } => "Playing",
            Paused { .. } => "Paused",
            Stopped => "Stopped",
        }
    }
}

/// A repeat/loop status
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Loop {
    /// - **MPRIS**: The playback will stop when there are no more tracks to play
    None,
    /// - **MPRIS**: The current track will start again from the begining once it has finished playing
    Track,
    /// - **MPRIS**: The playback loops through a list of tracks
    Playlist,
}

impl Loop {
    pub fn to_dbus_value(self) -> &'static str {
        use Loop::*;
        match self {
            None => "None",
            Track => "Track",
            Playlist => "Playlist",
        }
    }
    pub fn from_dbus_value(x: &str) -> Option<Self> {
        use Loop::*;
        match x {
            "None" => Some(None),
            "Track" => Some(Track),
            "Playlist" => Some(Playlist),
            _ => Option::None,
        }
    }
}

/// The direction to seek in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}
