use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};

/// Defines fundamental operations needed for media controls.
pub trait MediaControls: Sized {
    type Error;
    type PlatformConfig;

    /// Create media controls with the specified config.
    fn new(config: Self::PlatformConfig) -> Result<Self, Self::Error>;
    /// Attach the media control events to a handler.
    fn attach<F>(&mut self, event_handler: F) -> Result<(), Self::Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static;
    /// Detach the event handler.
    fn detach(&mut self) -> Result<(), Self::Error>;
    /// Set the current playback status.
    fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Self::Error>;
    /// Set the metadata of the currently playing media item.
    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Self::Error>;
}

/// Wrapper around a specific OS implementation of media controls.
/// Needed due to how Rust traits work.
/// Automatically detaches on Drop.
pub struct MediaControlsWrapper<OsImpl: MediaControls> {
    inner: OsImpl,
}

impl<T: MediaControls> MediaControlsWrapper<T> {
    /// Create media controls with the specified config.
    pub fn new(config: T::PlatformConfig) -> Result<Self, T::Error> {
        Ok(Self {
            inner: T::new(config)?,
        })
    }
    /// Attach the media control events to a handler.
    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), T::Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.inner.attach(event_handler)
    }
    /// Detach the event handler.
    pub fn detach(&mut self) -> Result<(), T::Error> {
        self.inner.detach()
    }
    /// Set the current playback status.
    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), T::Error> {
        self.inner.set_playback(playback)
    }
    /// Set the metadata of the currently playing media item.
    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), T::Error> {
        self.inner.set_metadata(metadata)
    }
}

impl<T: MediaControls> Drop for MediaControlsWrapper<T> {
    fn drop(&mut self) {
        self.inner.detach().ok();
    }
}

impl<T: MediaControls> std::fmt::Debug for MediaControlsWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MediaControls")
    }
}
