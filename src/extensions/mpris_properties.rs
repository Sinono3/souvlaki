use crate::{Loop, MediaControls};

/// Allows setting additional MPRIS properties to reflect the
/// current state of the media player, e.g. volume, minimum rate,
/// maximum rate, playback rate
pub trait MprisPropertiesExt: MediaControls {
    /// Set the loop/repeat status (none, track, playlist)
    fn set_loop_status(&mut self, loop_status: Loop) -> Result<(), Self::Error>;
    /// Set the playback rate, e.g. 0.5x, 1.0x, 2.0x
    fn set_rate(&mut self, rate: f64) -> Result<(), Self::Error>;
    /// As in the MPRIS2 specification:
    /// > A value of false indicates that playback is progressing linearly through a playlist, while true means playback is progressing through a playlist in some other order.
    fn set_shuffle(&mut self, shuffle: bool) -> Result<(), Self::Error>;
    /// Set the volume level (0.0-1.0)
    fn set_volume(&mut self, volume: f64) -> Result<(), Self::Error>;
    /// Set the maximum playback rate (should always be 1.0 or more)
    fn set_maximum_rate(&mut self, rate: f64) -> Result<(), Self::Error>;
    /// Set the minimum playback rate (should always be 1.0 or less)
    fn set_minimum_rate(&mut self, rate: f64) -> Result<(), Self::Error>;

    // TODO: Control capabilities
    // CanGoNext	b	Read only
    // CanGoPrevious	b	Read only
    // CanPlay	b	Read only
    // CanPause	b	Read only
    // CanSeek	b	Read only
    // CanControl	b	Read only
}
