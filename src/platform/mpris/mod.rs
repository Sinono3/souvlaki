#![cfg(all(unix, not(target_os = "macos")))]

#[cfg(not(any(feature = "dbus", feature = "zbus")))]
compile_error!("either feature \"dbus\" or feature \"zbus\" are required");

#[cfg(all(feature = "dbus", feature = "zbus"))]
compile_error!("feature \"dbus\" and feature \"zbus\" are mutually exclusive");

#[cfg(feature = "zbus")]
mod zbus;
#[cfg(feature = "zbus")]
pub use self::zbus::*;

#[cfg(feature = "dbus")]
mod dbus;
#[cfg(feature = "dbus")]
pub use self::dbus::*;
