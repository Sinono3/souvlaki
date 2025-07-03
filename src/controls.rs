use std::error::Error;
use std::fmt::Debug;

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback, Repeat};

/// Defines fundamental operations needed for media controls.
pub trait MediaControls: Sized + Debug {
    type Error: Error + Debug;
    type PlatformConfig: Debug;
    type Cover: Clone + Debug;

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
    /// Set the metadata of the current media item.
    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Self::Error>;
    /// Set the cover art/artwork/thumbnail of the current media item.
    fn set_cover(&mut self, cover: Option<Self::Cover>) -> Result<(), Self::Error>;
    /// Set the repeat/loop status (none, track, playlist).
    fn set_repeat(&mut self, repeat: Repeat) -> Result<(), Self::Error>;
    /// Set the shuffle status.self.inner.mut()
    fn set_shuffle(&mut self, shuffle: bool) -> Result<(), Self::Error>;
    /// Set the volume level (0.0-1.0).
    fn set_volume(&mut self, volume: f64) -> Result<(), Self::Error>;
    /// Set the playback rate, e.g. 0.5x, 1.0x, 2.0x.
    fn set_rate(&mut self, rate: f64) -> Result<(), Self::Error>;
    /// Set the maximum allowed playback rate.
    /// - max: should always be 1.0 or more
    /// - min: should always be 1.0 or less
    /// Only events received within these limits will be sent to the application handler.
    fn set_rate_limits(&mut self, min: f64, max: f64) -> Result<(), Self::Error>;
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
    /// Set the cover art/artwork/thumbnail of the current media item.
    pub fn set_cover(&mut self, cover: Option<T::Cover>) -> Result<(), T::Error> {
        self.inner.set_cover(cover)
    }
    /// Set the repeat/loop status (none, track, playlist).
    pub fn set_repeat(&mut self, repeat: Repeat) -> Result<(), T::Error> {
        self.inner.set_repeat(repeat)
    }
    /// Set the shuffle status.
    pub fn set_shuffle(&mut self, shuffle: bool) -> Result<(), T::Error> {
        self.inner.set_shuffle(shuffle)
    }
    /// Set the volume level (0.0-1.0).
    pub fn set_volume(&mut self, volume: f64) -> Result<(), T::Error> {
        self.inner.set_volume(volume)
    }
    /// Set the playback rate, e.g. 0.5x, 1.0x, 2.0x.
    pub fn set_rate(&mut self, rate: f64) -> Result<(), T::Error> {
        self.inner.set_rate(rate)
    }
    /// Set the maximum allowed playback rate.
    /// - max: should always be 1.0 or more
    /// - min: should always be 1.0 or less
    /// Only events received within these limits will be sent to the application handler.
    pub fn set_rate_limits(&mut self, min: f64, max: f64) -> Result<(), T::Error> {
        self.inner.set_rate_limits(min, max)
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
