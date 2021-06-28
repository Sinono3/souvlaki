#![cfg(target_os = "windows")]

mod bindings {
    ::windows::include_bindings!();
}

use self::bindings::Windows as win;
use raw_window_handle::windows::WindowsHandle;
use win::Foundation::{TypedEventHandler, Uri};
use win::Media::*;
use win::Storage::Streams::RandomAccessStreamReference;
use win::Win32::MediaTransport::ISystemMediaTransportControlsInterop;
use win::Win32::WindowsAndMessaging::HWND;
use windows::{Abi, HString, Interface};

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};

pub struct MediaControls {
    controls: SystemMediaTransportControls,
    display_updater: SystemMediaTransportControlsDisplayUpdater,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum SmtcPlayback {
    Stopped = 2,
    Playing = 3,
    Paused = 4,
}

#[derive(Debug)]
pub struct Error(windows::Error);

impl From<windows::Error> for Error {
    fn from(other: windows::Error) -> Error {
        Error(other)
    }
}

impl MediaControls {
    pub fn for_window(window_handle: WindowsHandle) -> Result<Self, Error> {
        let interop: ISystemMediaTransportControlsInterop =
            windows::factory::<SystemMediaTransportControls, ISystemMediaTransportControlsInterop>(
            )?;

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
        let display_updater = controls.DisplayUpdater()?;

        Ok(Self {
            controls,
            display_updater,
        })
    }

    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.controls.SetIsEnabled(true)?;
        self.controls.SetIsPlayEnabled(true)?;
        self.controls.SetIsPauseEnabled(true)?;
        self.controls.SetIsNextEnabled(true)?;
        self.controls.SetIsPreviousEnabled(true)?;

        self.display_updater.SetType(MediaPlaybackType::Music)?;

        let handler = TypedEventHandler::new(move |_, args: &Option<_>| {
            let args: &SystemMediaTransportControlsButtonPressedEventArgs = args.as_ref().unwrap();
            match args.Button()? {
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
        self.controls.ButtonPressed(handler)?;

        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        self.controls.SetIsEnabled(false)?;
        self.controls.ButtonPressed(None)?;
        Ok(())
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let status = match playback {
            MediaPlayback::Playing => SmtcPlayback::Playing as i32,
            MediaPlayback::Paused => SmtcPlayback::Paused as i32,
            MediaPlayback::Stopped => SmtcPlayback::Stopped as i32,
        };
        self.controls
            .SetPlaybackStatus(MediaPlaybackStatus(status))?;
        Ok(())
    }

    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        let properties = self.display_updater.MusicProperties()?;

        if let Some(title) = metadata.title {
            properties.SetTitle(title)?;
        }
        if let Some(artist) = metadata.artist {
            properties.SetArtist(artist)?;
        }
        if let Some(album) = metadata.album {
            properties.SetAlbumTitle(album)?;
        }
        if let Some(url) = metadata.cover_url {
            let stream =
                RandomAccessStreamReference::CreateFromUri(Uri::CreateUri(HString::from(url))?)?;
            self.display_updater.SetThumbnail(stream)?;
        }

        self.display_updater.Update()?;
        Ok(())
    }
}
