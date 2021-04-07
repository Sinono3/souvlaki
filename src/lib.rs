pub mod platform;

pub use platform::{Error, MediaControls};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused,
    Playing,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MediaMetadata<'s> {
    pub title: Option<&'s str>,
    pub album: Option<&'s str>,
    pub artist: Option<&'s str>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
}
