#![cfg(target_os = "windows")]

use std::sync::{Arc, Mutex};
use std::time::Duration;
use windows::core::{Error as WindowsError, HSTRING};
use windows::Foundation::{TypedEventHandler, Uri};
use windows::Media::*;
use windows::Storage::Streams::RandomAccessStreamReference;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop;

use crate::{
    MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig, SeekDirection,
};

/// A handle to OS media controls.
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

/// A platform-specific error.
#[derive(Debug)]
pub struct Error(WindowsError);

impl From<WindowsError> for Error {
    fn from(other: WindowsError) -> Error {
        Error(other)
    }
}

impl MediaControls {
    /// Create media controls with the specified config.
    pub fn new(config: PlatformConfig) -> Result<Self, Error> {
        let interop: ISystemMediaTransportControlsInterop = windows::core::factory::<
            SystemMediaTransportControls,
            ISystemMediaTransportControlsInterop,
        >()?;
        let hwnd = config
            .hwnd
            .expect("Windows media controls require an HWND in MediaControlsOptions.");

        let controls: SystemMediaTransportControls =
            unsafe { interop.GetForWindow(HWND(hwnd as isize)) }?;
        let display_updater = controls.DisplayUpdater()?;
        let timeline_properties = SystemMediaTransportControlsTimelineProperties::new()?;

        Ok(Self {
            controls,
            display_updater,
            timeline_properties,
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
        self.controls.SetIsStopEnabled(true)?;
        self.controls.SetIsNextEnabled(true)?;
        self.controls.SetIsPreviousEnabled(true)?;
        self.controls.SetIsFastForwardEnabled(true)?;
        self.controls.SetIsRewindEnabled(true)?;

        // TODO: allow changing this
        self.display_updater.SetType(MediaPlaybackType::Music)?;

        let event_handler = Arc::new(Mutex::new(event_handler));

        let button_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();

            move |_, args: &Option<_>| {
                let args: &SystemMediaTransportControlsButtonPressedEventArgs =
                    args.as_ref().unwrap();
                let button = args.Button()?;

                let event = if button == SystemMediaTransportControlsButton::Play {
                    MediaControlEvent::Play
                } else if button == SystemMediaTransportControlsButton::Pause {
                    MediaControlEvent::Pause
                } else if button == SystemMediaTransportControlsButton::Stop {
                    MediaControlEvent::Stop
                } else if button == SystemMediaTransportControlsButton::Next {
                    MediaControlEvent::Next
                } else if button == SystemMediaTransportControlsButton::Previous {
                    MediaControlEvent::Previous
                } else if button == SystemMediaTransportControlsButton::FastForward {
                    MediaControlEvent::Seek(SeekDirection::Forward)
                } else if button == SystemMediaTransportControlsButton::Rewind {
                    MediaControlEvent::Seek(SeekDirection::Backward)
                } else {
                    // Ignore unknown events
                    return Ok(());
                };

                (event_handler.lock().unwrap())(event);
                Ok(())
            }
        });
        self.controls.ButtonPressed(&button_handler)?;

        let position_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();

            move |_, args: &Option<_>| {
                let args: &PlaybackPositionChangeRequestedEventArgs = args.as_ref().unwrap();
                let position = Duration::from(args.RequestedPlaybackPosition()?);

                (event_handler.lock().unwrap())(MediaControlEvent::SetPosition(MediaPosition(position)));
                Ok(())
            }
        });
        self.controls
            .PlaybackPositionChangeRequested(&position_handler)?;

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

        let progress = match playback {
            MediaPlayback::Playing {
                progress: Some(progress),
            }
            | MediaPlayback::Paused {
                progress: Some(progress),
            } => progress.0,
            _ => Duration::default(),
        };
        self.timeline_properties.SetPosition(progress.into())?;

        self.controls
            .UpdateTimelineProperties(&self.timeline_properties)?;
        Ok(())
    }

    /// Set the metadata of the currently playing media item.
    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        let properties = self.display_updater.MusicProperties()?;

        if let Some(title) = metadata.title {
            properties.SetTitle(&HSTRING::from(title))?;
        }
        if let Some(artist) = metadata.artist {
            properties.SetArtist(&HSTRING::from(artist))?;
        }
        if let Some(album) = metadata.album {
            properties.SetAlbumTitle(&HSTRING::from(album))?;
        }
        if let Some(url) = metadata.cover_url {
            let uri = Uri::CreateUri(&HSTRING::from(url))?;
            let stream = RandomAccessStreamReference::CreateFromUri(&uri)?;
            self.display_updater.SetThumbnail(&stream)?;
        }
        let duration = metadata.duration.unwrap_or_default();
        self.timeline_properties
            .SetStartTime(Duration::default().into())?;
        self.timeline_properties
            .SetMinSeekTime(Duration::default().into())?;
        self.timeline_properties.SetEndTime(duration.into())?;
        self.timeline_properties.SetMaxSeekTime(duration.into())?;

        self.controls
            .UpdateTimelineProperties(&self.timeline_properties)?;
        self.display_updater.Update()?;
        Ok(())
    }
}
