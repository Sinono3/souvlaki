#![cfg(target_os = "windows")]

mod bindings;

use self::bindings::windows as win;
use raw_window_handle::windows::WindowsHandle;
use win::foundation::TypedEventHandler;
use win::media::*;
use win::win32::media_transport::ISystemMediaTransportControlsInterop;
use win::win32::windows_and_messaging::HWND;
use win::{Abi, Interface};

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};

pub struct MediaControls {
    controls: SystemMediaTransportControls,
    display_updater: SystemMediaTransportControlsDisplayUpdater,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum WindowsMediaPlaybackStatus {
    Stopped = 2,
    Playing = 3,
    Paused = 4,
}

#[derive(Debug)]
pub struct Error(win::Error);

impl From<win::Error> for Error {
    fn from(other: win::Error) -> Error {
        Error(other)
    }
}

impl MediaControls {
    pub fn for_window(window_handle: WindowsHandle) -> Result<Self, Error> {
        let interop: ISystemMediaTransportControlsInterop =
            win::factory::<SystemMediaTransportControls, ISystemMediaTransportControlsInterop>()?;

        let mut smtc: Option<SystemMediaTransportControls> = None;
        unsafe {
            interop.GetForWindow(
                HWND(window_handle.hwnd as isize),
                &SystemMediaTransportControls::IID as *const _,
                smtc.set_abi(),
            )
        }
        .unwrap();
        let controls = smtc.unwrap();
        let display_updater = controls.display_updater()?;

        Ok(Self {
            controls,
            display_updater,
        })
    }

    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.controls.set_is_enabled(true)?;
        self.controls.set_is_play_enabled(true)?;
        self.controls.set_is_pause_enabled(true)?;
        self.controls.set_is_next_enabled(true)?;
        self.controls.set_is_previous_enabled(true)?;

        self.display_updater.set_type(MediaPlaybackType::Music)?;

        let handler = TypedEventHandler::new(move |_, args: &Option<_>| {
            let args: &SystemMediaTransportControlsButtonPressedEventArgs = args.as_ref().unwrap();
            match args.button()? {
                SystemMediaTransportControlsButton::Play => {
                    (event_handler)(MediaControlEvent::Play);
                }
                SystemMediaTransportControlsButton::Pause => {
                    (event_handler)(MediaControlEvent::Pause);
                }
                SystemMediaTransportControlsButton::Next => {
                    (event_handler)(MediaControlEvent::Next);
                }
                SystemMediaTransportControlsButton::Previous => {
                    (event_handler)(MediaControlEvent::Previous);
                }
                _ => {
                    // Ignore unknown events.
                }
            }
            Ok(())
        });
        self.controls.button_pressed(handler)?;

        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        self.controls.set_is_enabled(false)?;
        self.controls.button_pressed(None)?;
        Ok(())
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let status = match playback {
            MediaPlayback::Playing => WindowsMediaPlaybackStatus::Playing as i32,
            MediaPlayback::Paused => WindowsMediaPlaybackStatus::Paused as i32,
            MediaPlayback::Stopped => WindowsMediaPlaybackStatus::Stopped as i32,
        };
        self.controls
            .set_playback_status(MediaPlaybackStatus(status))
            .unwrap();
        Ok(())
    }

    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        let properties = self.display_updater.music_properties().unwrap();

        properties.set_title(metadata.title).unwrap();
        properties.set_artist(metadata.artist).unwrap();
        properties.set_album_title(metadata.album).unwrap();

        self.display_updater.update().unwrap();
        Ok(())
    }
}
