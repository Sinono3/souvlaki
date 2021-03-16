#![cfg(target_os = "macos")]

use crate::platform_impl::MediaControls as MacOsMediaControls;
use crate::MediaControls;

pub trait MediaControlsExtMacOs {
    fn create() -> Result<MediaControls, ()>;
}

impl MediaControlsExtMacOs for MediaControls {
    fn create() -> Result<Self, ()> {
        Ok(MediaControls {
            controls: MacOsMediaControls::new(),
        })
    }
}
