//! A cross-platform solution to OS media controls and metadata. One abstraction for Linux, MacOS and Windows.
//!
//! ## Example
//! 
//! The main struct is `MediaControls`. In order to create this struct you need a `PlatformConfig`. This struct contains all of the platform-specific requirements for spawning media controls. Here are the differences between the platforms:
//! 
//! - MacOS: No config needed.
//! - Linux: 
//! 	- `dbus_name`: The way your player will appear on D-Bus. It should follow [the D-Bus specification](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-names-bus). 
//! 	- `display_name`: This could be however you want. It's the name that will be shown to the users.
//! - Windows: 
//! 	- `hwnd`: In this platform, a window needs to be opened to create media controls. The argument required is an `HWND`, a value of type `*mut c_void`. This value can be extracted when you open a window in your program, for example using the `raw_window_handle` in winit.
//! 
//! A full cross-platform app would look like this:
//! 
//! ```rust
//! use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
//! 
//! fn main() {
//!     #[cfg(not(target_os = "windows"))]
//!     let hwnd = None;
//! 
//!     #[cfg(target_os = "windows")]
//!     let hwnd = {
//!         use raw_window_handle::windows::WindowsHandle;
//! 
//!         let handle: WindowsHandle = unimplemented!();
//!         Some(handle.hwnd)
//!     };
//! 
//!     let config = PlatformConfig {
//!         dbus_name: "my_player",
//!         display_name: "My Player",
//!         hwnd,
//!     };
//! 
//!     let mut controls = MediaControls::new(config);
//! 
//!     // The closure must be Send and have a static lifetime.
//!     controls
//!         .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
//!         .unwrap();
//! 
//!     // Update the media metadata.
//!     controls
//!         .set_metadata(MediaMetadata {
//!             title: Some("Souvlaki Space Station"),
//!             artist: Some("Slowdive"),
//!             album: Some("Souvlaki"),
//!             ..Default::default()
//!         })
//!         .unwrap();
//! 
//!     // Your actual logic goes here.
//!     loop {
//!         std::thread::sleep(std::time::Duration::from_secs(1));
//!     }
//! 
//!     // The controls automatically detach on drop.
//! }
//! ```
//! 
//! [Check out this example here.](https://github.com/Sinono3/souvlaki/blob/master/examples/print_events.rs)

mod config;
mod platform;

use std::time::Duration;

pub use config::*;
pub use platform::{Error, MediaControls};

/// The status of media playback.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaPlayback {
    Stopped,
    Paused { progress: Option<MediaPosition> },
    Playing { progress: Option<MediaPosition> },
}

/// The metadata of a media item.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct MediaMetadata<'a> {
    pub title: Option<&'a str>,
    pub album: Option<&'a str>,
    pub artist: Option<&'a str>,
    pub cover_url: Option<&'a str>,
    pub duration: Option<Duration>,
}

/// Events sent by the OS media controls.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,

    /// Seek forward or backward by an undetermined amount.
    Seek(SeekDirection),
    /// Seek forward or backward by a certain amount.
    SeekBy(SeekDirection, Duration),
    /// Set the position/progress of the currently playing media item.
    SetPosition(MediaPosition),
    /// Open the URI in the media player.
    OpenUri(String),

    /// Bring the media player's user interface to the front using any appropriate mechanism available.
    Raise,
    /// Shut down the media player.
    Quit,
}

/// An instant in a media item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MediaPosition(pub Duration);

/// The direction to seek in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

impl Drop for MediaControls {
    fn drop(&mut self) {
        // Ignores errors if there are any.
        self.detach().ok();
    }
}
