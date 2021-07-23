#![cfg(target_os = "windows")]

mod bindings {
    ::windows::include_bindings!();
}

use self::bindings::Windows as win;
use raw_window_handle::windows::WindowsHandle;
use std::sync::Arc;
use std::time::Duration;
use win::Foundation::{TypedEventHandler, Uri};
use win::Media::*;
use win::Storage::Streams::RandomAccessStreamReference;
use win::Win32::Foundation::HWND;
use win::Win32::System::WinRT::ISystemMediaTransportControlsInterop;
use windows::HSTRING;

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, SeekDirection};

pub struct MediaControls {
    controls: SystemMediaTransportControls,
    display_updater: SystemMediaTransportControlsDisplayUpdater,
    timeline_properties: SystemMediaTransportControlsTimelineProperties,
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

        let controls: SystemMediaTransportControls =
            unsafe { interop.GetForWindow(HWND(window_handle.hwnd as isize)) }?;
        let display_updater = controls.DisplayUpdater()?;
        let timeline_properties = SystemMediaTransportControlsTimelineProperties::new()?;

        Ok(Self {
            controls,
            display_updater,
            timeline_properties,
        })
    }

    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.controls.SetIsEnabled(true)?;
        self.controls.SetIsPlayEnabled(true)?;
        self.controls.SetIsPauseEnabled(true)?;
        self.controls.SetIsStopEnabled(true)?;
        self.controls.SetIsNextEnabled(true)?;
        self.controls.SetIsPreviousEnabled(true)?;
        self.controls.SetIsFastForwardEnabled(true)?;
        self.controls.SetIsRewindEnabled(true)?;

        // TODO: allow changing this
        self.display_updater.SetType(MediaPlaybackType::Music)?;

        let event_handler = Arc::new(event_handler);

        let button_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();

            move |_, args: &Option<_>| {
                let args: &SystemMediaTransportControlsButtonPressedEventArgs =
                    args.as_ref().unwrap();
                match args.Button()? {
                    SystemMediaTransportControlsButton::Play => {
                        (event_handler)(MediaControlEvent::Play);
                    }
                    SystemMediaTransportControlsButton::Pause => {
                        (event_handler)(MediaControlEvent::Pause);
                    }
                    SystemMediaTransportControlsButton::Stop => {
                        (event_handler)(MediaControlEvent::Stop);
                    }
                    SystemMediaTransportControlsButton::Next => {
                        (event_handler)(MediaControlEvent::Next);
                    }
                    SystemMediaTransportControlsButton::Previous => {
                        (event_handler)(MediaControlEvent::Previous);
                    }
                    SystemMediaTransportControlsButton::FastForward => {
                        (event_handler)(MediaControlEvent::Seek(SeekDirection::Forward));
                    }
                    SystemMediaTransportControlsButton::Rewind => {
                        (event_handler)(MediaControlEvent::Seek(SeekDirection::Backward));
                    }
                    _ => {
                        // Ignore unknown events.
                    }
                }
                Ok(())
            }
        });
        self.controls.ButtonPressed(button_handler)?;

        let position_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();

            move |_, args: &Option<_>| {
                let args: &PlaybackPositionChangeRequestedEventArgs = args.as_ref().unwrap();
                let position = Duration::from(args.RequestedPlaybackPosition()?);

                (event_handler)(MediaControlEvent::SetPosition(MediaPosition(position)));
                Ok(())
            }
        });
        self.controls
            .PlaybackPositionChangeRequested(position_handler)?;

        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        self.controls.SetIsEnabled(false)?;
        self.controls.ButtonPressed(None)?;
        Ok(())
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let status = match playback {
            MediaPlayback::Playing { .. } => SmtcPlayback::Playing as i32,
            MediaPlayback::Paused { .. } => SmtcPlayback::Paused as i32,
            MediaPlayback::Stopped => SmtcPlayback::Stopped as i32,
        };
        self.controls
            .SetPlaybackStatus(MediaPlaybackStatus(status))?;

        let progress = match playback {
            MediaPlayback::Playing {
                progress: Some(progress),
            }
            | MediaPlayback::Paused {
                progress: Some(progress),
            } => progress.0,
            _ => Duration::default(),
        };
        self.timeline_properties.SetPosition(progress)?;

        self.controls
            .UpdateTimelineProperties(self.timeline_properties.clone())?;
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
                RandomAccessStreamReference::CreateFromUri(Uri::CreateUri(HSTRING::from(url))?)?;
            self.display_updater.SetThumbnail(stream)?;
        }
        let duration = metadata.duration.unwrap_or_default();
        self.timeline_properties.SetStartTime(Duration::default())?;
        self.timeline_properties
            .SetMinSeekTime(Duration::default())?;
        self.timeline_properties.SetEndTime(duration)?;
        self.timeline_properties.SetMaxSeekTime(duration)?;

        self.controls
            .UpdateTimelineProperties(self.timeline_properties.clone())?;
        self.display_updater.Update()?;
        Ok(())
    }
}
