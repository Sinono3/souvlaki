use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};

#[derive(Debug)]
pub struct Error;

pub struct MediaControls;

impl MediaControls {
    pub fn new() -> Self {
        Self
    }

    pub fn attach<F>(&mut self, _event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        Ok(())
    }

    pub fn set_playback(&mut self, _playback: MediaPlayback) -> Result<(), Error> {
        Ok(())
    }

    pub fn set_metadata(&mut self, _metadata: MediaMetadata) -> Result<(), Error> {
        Ok(())
    }
}
