#[cfg(target_os = "windows")]
pub mod windows;

pub trait MediaPlayer {
    fn play(&mut self);
    fn pause(&mut self);
    fn playing(&self) -> bool;

    fn metadata(&self) -> MediaMetadata;
}

pub trait MediaControls<S: MediaPlayer> {
    type Args;
    type Error;

    fn new(state: &S, args: Self::Args) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn poll(&mut self, state: &mut S);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MediaMetadata {
    pub title: String,
    pub album: String,
    pub artist: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Next,
    Previous,
}
