use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ::windows::core::HSTRING;
use ::windows::Foundation::{EventRegistrationToken, TimeSpan, TypedEventHandler, Uri};
use ::windows::Media::*;
use ::windows::Storage::Streams::RandomAccessStreamReference;
use ::windows::Win32::Foundation::HWND;
use ::windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop;

use crate::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, SeekDirection,
};

pub use ::windows::core::Error as WindowsError;

/// A handle to Windows' SystemMediaTransportControls
pub struct Windows {
    controls: SystemMediaTransportControls,
    button_handler_token: Option<EventRegistrationToken>,
    display_updater: SystemMediaTransportControlsDisplayUpdater,
    timeline_properties: SystemMediaTransportControlsTimelineProperties,
}

/// Windows-specific configuration needed to create media controls.
#[derive(Debug)]
pub struct WindowsConfig {
    /// HWND. A window handle specific to Windows.
    pub hwnd: *mut c_void,
}

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum SmtcPlayback {
    Stopped = 2,
    Playing = 3,
    Paused = 4,
}

impl MediaControls for Windows {
    type Error = WindowsError;
    type PlatformConfig = WindowsConfig;

    fn new(config: Self::PlatformConfig) -> Result<Self, Self::Error> {
        let interop: ISystemMediaTransportControlsInterop = windows::core::factory::<
            SystemMediaTransportControls,
            ISystemMediaTransportControlsInterop,
        >()?;
        let controls: SystemMediaTransportControls =
            unsafe { interop.GetForWindow(HWND(config.hwnd as isize)) }?;
        let display_updater = controls.DisplayUpdater()?;
        let timeline_properties = SystemMediaTransportControlsTimelineProperties::new()?;

        Ok(Self {
            controls,
            display_updater,
            timeline_properties,
            button_handler_token: None,
        })
    }

    fn attach<F>(&mut self, event_handler: F) -> Result<(), Self::Error>
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
        self.button_handler_token = Some(self.controls.ButtonPressed(&button_handler)?);

        let position_handler = TypedEventHandler::new({
            move |_, args: &Option<_>| {
                let args: &PlaybackPositionChangeRequestedEventArgs = args.as_ref().unwrap();
                let position = Duration::from(args.RequestedPlaybackPosition()?);

                (event_handler.lock().unwrap())(MediaControlEvent::SetPosition(MediaPosition(
                    position,
                )));
                Ok(())
            }
        });
        self.controls
            .PlaybackPositionChangeRequested(&position_handler)?;

        Ok(())
    }

    fn detach(&mut self) -> Result<(), Self::Error> {
        self.controls.SetIsEnabled(false)?;
        if let Some(button_handler_token) = self.button_handler_token {
            self.controls.RemoveButtonPressed(button_handler_token)?;
        }
        Ok(())
    }

    fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Self::Error> {
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
            } => TimeSpan::from(progress.0),
            _ => TimeSpan::default(),
        };
        self.timeline_properties.SetPosition(progress)?;

        self.controls
            .UpdateTimelineProperties(&self.timeline_properties)?;
        Ok(())
    }

    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Self::Error> {
        let properties = self.display_updater.MusicProperties()?;

        if let Some(title) = metadata.title {
            properties.SetTitle(&HSTRING::from(title))?;
        }
        if let Some(artist) = metadata.artist {
            properties.SetArtist(&HSTRING::from(artist))?;
        }
        if let Some(album_title) = metadata.album_title {
            properties.SetAlbumTitle(&HSTRING::from(album_title))?;
        }
        // TODO:
        // if let Some(url) = metadata.cover_url {
        //     let stream = if url.starts_with("file://") {
        //         // url is a file, load it manually
        //         let path = url.trim_start_matches("file://");
        //         let loader =
        //             windows::Storage::StorageFile::GetFileFromPathAsync(&HSTRING::from(path))?;
        //         let results = loader.get()?;
        //         loader.Close()?;

        //         RandomAccessStreamReference::CreateFromFile(&results)?
        //     } else {
        //         RandomAccessStreamReference::CreateFromUri(&Uri::CreateUri(&HSTRING::from(url))?)?
        //     };
        //     self.display_updater.SetThumbnail(&stream)?;
        // }
        let duration = metadata.duration.unwrap_or_default();
        self.timeline_properties.SetStartTime(TimeSpan::default())?;
        self.timeline_properties
            .SetMinSeekTime(TimeSpan::default())?;
        self.timeline_properties
            .SetEndTime(TimeSpan::from(duration))?;
        self.timeline_properties
            .SetMaxSeekTime(TimeSpan::from(duration))?;

        self.controls
            .UpdateTimelineProperties(&self.timeline_properties)?;
        self.display_updater.Update()?;
        Ok(())
    }
}
