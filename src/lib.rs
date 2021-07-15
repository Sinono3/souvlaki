pub mod platform;

pub use platform::{Error, MediaControls};

use std::time::Duration;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused,
    Playing,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MediaMetadata<'a> {
    pub title: Option<&'a str>,
    pub album: Option<&'a str>,
    pub artist: Option<&'a str>,
    pub cover_url: Option<&'a str>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
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
    SetPosition(MediaPosition),
    OpenUri(String),

    /// Bring the media player's user interface to the front using any appropriate mechanism available.
    Raise,
    /// Shut down the media player.
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// An instant in a media item.
pub struct MediaPosition(pub Duration);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

impl Drop for MediaControls {
    fn drop(&mut self) {
        // Ignores errors if there are any.
        self.detach().ok();
    }
}
