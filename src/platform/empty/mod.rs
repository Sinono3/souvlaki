use crate::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};

/// A platform-specific error.
#[derive(Debug)]
pub struct EmptyError;

impl std::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Error")
    }
}

impl std::error::Error for EmptyError {}

/// A handle to OS media controls.
pub struct Empty;

impl MediaControls for Empty {
    type Error = EmptyError;

    fn new(_config: PlatformConfig) -> Result<Self, EmptyError> {
        Ok(Self)
    }

    fn attach<F>(&mut self, _event_handler: F) -> Result<(), EmptyError>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        Ok(())
    }

    fn detach(&mut self) -> Result<(), EmptyError> {
        Ok(())
    }

    fn set_playback(&mut self, _playback: MediaPlayback) -> Result<(), EmptyError> {
        Ok(())
    }

    fn set_metadata(&mut self, _metadata: MediaMetadata) -> Result<(), EmptyError> {
        Ok(())
    }
}
