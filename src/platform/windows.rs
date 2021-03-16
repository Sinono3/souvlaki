#![cfg(target_os = "windows")]

use raw_window_handle::windows::WindowsHandle;

use crate::platform_impl::{MediaControls as WindowsMediaControls, OsError};
use crate::MediaControls;

pub trait MediaControlsExtWindows {
    fn create_for_window(window_handle: WindowsHandle) -> Result<MediaControls, OsError>;
}

impl MediaControlsExtWindows for MediaControls {
    fn create_for_window(window_handle: WindowsHandle) -> Result<Self, OsError> {
        let controls = WindowsMediaControls::create_for_window(window_handle)?;

        Ok(MediaControls { controls })
    }
}
