#[cfg(target_os = "windows")]
pub mod windows;

pub trait MediaControls: Sized {
    type Args;
    type Error;

    fn create(args: Self::Args) -> Result<Self, Self::Error>;

    fn set_playback(&mut self, playing: bool);
    fn set_metadata(&mut self, metadata: MediaMetadata);
    
    fn poll<'f, F>(&mut self, handler: F)
    where
        F: 'f + FnMut(MediaControlEvent);
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MediaMetadata<'s> {
    pub title: &'s str,
    pub album: &'s str,
    pub artist: &'s str,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Next,
    Previous,
}
