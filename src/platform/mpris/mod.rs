#![cfg(platform_mpris)]

#[cfg(not(any(feature = "dbus", feature = "zbus")))]
compile_error!("either feature \"dbus\" or feature \"zbus\" are required");

#[cfg(all(feature = "dbus", feature = "zbus"))]
compile_error!("feature \"dbus\" and feature \"zbus\" are mutually exclusive");

#[cfg(feature = "zbus")]
mod zbus;
use std::{sync::mpsc, thread::JoinHandle};

use crate::{extensions::MprisPropertiesExt, Loop, MediaMetadata, MediaPlayback};

#[cfg(feature = "zbus")]
pub use self::zbus::Zbus as Mpris;
#[cfg(feature = "zbus")]
extern crate zbus as zbus_crate;

#[cfg(feature = "dbus")]
mod dbus;
#[cfg(feature = "dbus")]
pub use self::dbus::Dbus as Mpris;
#[cfg(feature = "dbus")]
extern crate dbus as dbus_crate;

/// A platform-specific error.
#[derive(thiserror::Error, Debug)]
pub enum MprisError {
    #[error("internal D-Bus error: {0}")]
    #[cfg(feature = "dbus")]
    DbusError(#[from] dbus_crate::Error),
    #[error("internal D-Bus error: {0}")]
    #[cfg(feature = "zbus")]
    DbusError(#[from] zbus_crate::Error),
    #[error("D-bus service thread not running. Run MediaControls::attach()")]
    ThreadNotRunning,
    // NOTE: For now this error is not very descriptive. For now we can't do much about it
    // since the panic message returned by JoinHandle::join does not implement Debug/Display,
    // thus we cannot print it, though perhaps there is another way. I will leave this error here,
    // to at least be able to catch it, but it is preferable to have this thread *not panic* at all.
    #[error("D-Bus service thread panicked")]
    ThreadPanicked,
}

struct ServiceThreadHandle {
    event_channel: mpsc::Sender<InternalEvent>,
    thread: JoinHandle<Result<(), MprisError>>,
}

#[derive(Clone, Debug)]
pub(crate) enum InternalEvent {
    SetMetadata(MediaMetadata),
    SetPlayback(MediaPlayback),
    SetLoopStatus(Loop),
    SetRate(f64),
    SetShuffle(bool),
    SetVolume(f64),
    SetMaximumRate(f64),
    SetMinimumRate(f64),
    Kill,
}

#[cfg(platform_mpris_dbus)]
use ::dbus::arg::{RefArg, Variant};
#[cfg(platform_mpris_dbus)]
use std::collections::HashMap;

// TODO: This is public only due to how rust modules work...
// should not actually be seen by the library user
#[derive(Debug)]
struct ServiceState {
    playback_status: MediaPlayback,
    loop_status: Loop,
    rate: f64,
    shuffle: bool,
    metadata: MediaMetadata,
    #[cfg(platform_mpris_dbus)]
    metadata_dict: HashMap<String, Variant<Box<dyn RefArg>>>,
    volume: f64,
    maximum_rate: f64,
    minimum_rate: f64,
}

impl MprisPropertiesExt for Mpris {
    fn set_loop_status(&mut self, loop_status: Loop) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetLoopStatus(loop_status))
    }

    fn set_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetRate(rate))
    }

    fn set_shuffle(&mut self, shuffle: bool) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetShuffle(shuffle))
    }

    fn set_volume(&mut self, volume: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetVolume(volume))
    }

    fn set_maximum_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMaximumRate(rate))
    }

    fn set_minimum_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMinimumRate(rate))
    }
}

// Macro for constructing metadata fields
macro_rules! insert_if_some {
    ($insert:expr, $wrapper:ident, $($key:literal, $value:expr),* $(,)?) => {
        $(
            if let Some(value) = $value {
                ($insert)($key, $wrapper::new(value.clone()));
            }
        )*
    };
    // Variant for values that don't need cloning
    ($insert:expr, $wrapper:ident, no_clone, $($key:literal, $value:expr),* $(,)?) => {
        $(
            if let Some(value) = $value {
                ($insert)($key, $wrapper::new(value));
            }
        )*
    };
}
pub(self) use insert_if_some;
