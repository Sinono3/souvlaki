#[cfg(platform_mpris)]
pub mod mpris;
#[cfg(platform_mpris)]
pub use mpris::{Mpris as OsImpl, MprisError as OsError};

// Both macOS and iOS are under the Apple platform
#[cfg(platform_apple)]
pub mod macos;
#[cfg(platform_apple)]
pub use macos::{Macos as OsImpl, MacosError as OsError};

#[cfg(platform_windows)]
pub mod windows;
#[cfg(platform_windows)]
pub use windows::{Windows as OsImpl, WindowsError as OsError};

/// Dummy platform in case is not supported. All media control operations are simply no-ops.
#[cfg(not(any(platform_mpris, platform_macos, platform_windows)))]
#[path = "empty/mod.rs"]
pub mod empty;
#[cfg(not(any(platform_mpris, platform_macos, platform_windows)))]
pub use empty::{Empty as OsImpl, EmptyError as OsError};
