#![doc = include_str!("../README.md")]

use std::{fmt::Debug, time::Duration};

mod controls;
mod metadata;
/// The platform-specific implementations of the media controls.
pub mod platform;

pub use controls::{MediaControls, MediaControlsWrapper};
pub use metadata::*;

/// The current OS's media controls.
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
    /// But other values are also accepted. **It is up to the
    /// application to set constraints on this value.**
    /// **NOTE**: If the request was handled, and the property
    /// was changed, [`MediaControls::set_volume`] must be called
    /// with the new value.
    SetVolume(f64),
    /// Set the playback rate.
    /// **NOTE**: If the request was handled, and the property
    /// was changed, [`MediaControls::set_rate`] must be called
    /// with the new value.
    SetRate(f64),
    /// Set shuffle on or off.
    /// **NOTE**: If the request was handled, and the property
    /// was changed, [`MediaControls::set_shuffle`] must be called
    /// with the new value.
    SetShuffle(bool),
    /// Set repeat mode of the media item: (none, track, playlist)
    /// **NOTE**: If the request was handled, and the property
    /// was changed, [`MediaControls::set_repeat`] must be called
    /// with the new value.
    SetRepeat(Repeat),

    /// MPRIS-specific
    /// Open the URI in the media player.
    OpenUri(String),
    /// Bring the media player's user interface to the front using any appropriate mechanism available.
    Raise,
    /// Shut down the media player.
    Quit,

    /// Windows-specific
    FastForward,
    Rewind,
    ChannelUp,
    ChannelDown,
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
pub enum Repeat {
    /// - **MPRIS**: The playback will stop when there are no more tracks to play
    None,
    /// - **MPRIS**: The current track will start again from the begining once it has finished playing
    Track,
    /// - **MPRIS**: The playback loops through a list of tracks
    Playlist,
}

impl Repeat {
    pub fn to_dbus_value(self) -> &'static str {
        use Repeat::*;
        match self {
            None => "None",
            Track => "Track",
            Playlist => "Playlist",
        }
    }
    pub fn from_dbus_value(x: &str) -> Option<Self> {
        use Repeat::*;
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
