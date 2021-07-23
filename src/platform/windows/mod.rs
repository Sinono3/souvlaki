#![cfg(target_os = "windows")]

mod bindings {
    ::windows::include_bindings!();
}

use self::bindings::Windows as win;
use win::Foundation::{TypedEventHandler, Uri};
use win::Media::*;
use win::Storage::Streams::RandomAccessStreamReference;
use win::Win32::Foundation::HWND;
use win::Win32::System::WinRT::ISystemMediaTransportControlsInterop;
use windows::HSTRING;

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback, PlatformConfig};

/// A handle to OS media controls.
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

/// A platform-specific error.
#[derive(Debug)]
pub struct Error(windows::Error);

impl From<windows::Error> for Error {
    fn from(other: windows::Error) -> Error {
        Error(other)
    }
}

impl MediaControls {
    /// Create media controls with the specified config.
    pub fn new(config: PlatformConfig) -> Result<Self, Error> {
        let interop: ISystemMediaTransportControlsInterop =
            windows::factory::<SystemMediaTransportControls, ISystemMediaTransportControlsInterop>(
            )?;
        let hwnd = config
            .hwnd
            .expect("Windows media controls require an HWND in MediaControlsOptions.");

        let controls: SystemMediaTransportControls =
            unsafe { interop.GetForWindow(HWND(hwnd as isize)) }?;
        let display_updater = controls.DisplayUpdater()?;

        Ok(Self {
            controls,
            display_updater,
        })
    }

    /// Attach the media control events to a handler.
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

    /// Detach the event handler.
    pub fn detach(&mut self) -> Result<(), Error> {
        self.controls.SetIsEnabled(false)?;
        self.controls.ButtonPressed(None)?;
        Ok(())
    }

    /// Set the current playback status.
    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let status = match playback {
            MediaPlayback::Playing { .. } => SmtcPlayback::Playing as i32,
            MediaPlayback::Paused { .. } => SmtcPlayback::Paused as i32,
            MediaPlayback::Stopped => SmtcPlayback::Stopped as i32,
        };
        self.controls
            .SetPlaybackStatus(MediaPlaybackStatus(status))?;
        Ok(())
    }

    /// Set the metadata of the currently playing media item.
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
                RandomAccessStreamReference::CreateFromUri(Uri::CreateUri(HSTRING::from(url))?)?;
            self.display_updater.SetThumbnail(stream)?;
        }

        self.display_updater.Update()?;
        Ok(())
    }
}
