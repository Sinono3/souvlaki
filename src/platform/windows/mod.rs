use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use windows::core::{Interface, Ref, HSTRING};
use windows::Foundation::{TimeSpan, TypedEventHandler, Uri};
use windows::Media::*;
use windows::Storage::Streams::RandomAccessStreamReference;
use windows::Storage::Streams::{DataWriter, IRandomAccessStream, InMemoryRandomAccessStream};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop;

use crate::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition,
    MediaTypeWindows, Repeat,
};

pub use windows::core::Error as WindowsError;

#[derive(Debug)]
struct Handlers {
    button: i64,
    playback_position: i64,
    playback_rate: i64,
    shuffle: i64,
    repeat: i64,
}

/// A handle to Windows' SystemMediaTransportControls
#[derive(Debug)]
pub struct Windows {
    controls: SystemMediaTransportControls,
    handlers: Option<Handlers>,
    display_updater: SystemMediaTransportControlsDisplayUpdater,
    timeline_properties: SystemMediaTransportControlsTimelineProperties,
}

/// Windows-specific configuration needed to create media controls.
#[derive(Debug)]
pub struct WindowsConfig {
    /// HWND. A window handle specific to Windows.
    pub hwnd: *mut c_void,
}

// TODO: Implement debug properly
/// Definition/reference to cover art for Windows.
#[derive(Clone)]
pub enum WindowsCover {
    /// Loads the image via [`RandomAccessStreamReference.CreateFromUri`](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.randomaccessstreamreference.createfromuri?view=winrt-26100#windows-storage-streams-randomaccessstreamreference-createfromuri(windows-foundation-uri))
    Uri(String),
    /// Loads the image via [`RandomAccessStreamReference.CreateFromFile`](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.randomaccessstreamreference.createfromfile?view=winrt-26100#windows-storage-streams-randomaccessstreamreference-createfromfile(windows-storage-istoragefile))
    LocalFile(PathBuf),
    /// Loads the image via [`RandomAccessStreamReference.CreateFromStream`](https://learn.microsoft.com/en-us/uwp/api/windows.storage.streams.randomaccessstreamreference.createfromstream?view=winrt-26100#windows-storage-streams-randomaccessstreamreference-createfromstream(windows-storage-streams-irandomaccessstream))
    Bytes(Vec<u8>),
}

impl std::fmt::Debug for WindowsCover {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowsCover::Uri(uri) => f.write_fmt(format_args!("Uri({uri:?})")),
            WindowsCover::LocalFile(path) => f.write_fmt(format_args!("LocalFile({path:?})")),
            WindowsCover::Bytes(_) => f.write_str("Bytes(<binary>)"),
        }
    }
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
    type Cover = WindowsCover;

    fn new(config: Self::PlatformConfig) -> Result<Self, Self::Error> {
        let interop: ISystemMediaTransportControlsInterop = windows::core::factory::<
            SystemMediaTransportControls,
            ISystemMediaTransportControlsInterop,
        >()?;
        let controls: SystemMediaTransportControls =
            unsafe { interop.GetForWindow(HWND(config.hwnd)) }?;
        let display_updater = controls.DisplayUpdater()?;
        let timeline_properties = SystemMediaTransportControlsTimelineProperties::new()?;

        Ok(Self {
            controls,
            display_updater,
            timeline_properties,
            handlers: None,
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

        let event_handler = Arc::new(Mutex::new(event_handler));

        let button_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();

            move |_, args: Ref<SystemMediaTransportControlsButtonPressedEventArgs>| {
                let args = (*args).as_ref().unwrap();
                let button = args.Button()?;

                // We cannot match on these...
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
                    MediaControlEvent::FastForward
                } else if button == SystemMediaTransportControlsButton::Rewind {
                    MediaControlEvent::Rewind
                } else if button == SystemMediaTransportControlsButton::ChannelUp {
                    MediaControlEvent::ChannelUp
                } else if button == SystemMediaTransportControlsButton::ChannelDown {
                    MediaControlEvent::ChannelDown
                } else {
                    // Ignore unknown events
                    return Ok(());
                };

                (event_handler.lock().unwrap())(event);
                Ok(())
            }
        });

        let position_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();
            move |_, args: Ref<PlaybackPositionChangeRequestedEventArgs>| {
                let args = (*args).as_ref().unwrap();
                let position = Duration::from(args.RequestedPlaybackPosition()?);

                (event_handler.lock().unwrap())(MediaControlEvent::SetPosition(MediaPosition(
                    position,
                )));
                Ok(())
            }
        });

        let rate_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();
            move |_, args: Ref<PlaybackRateChangeRequestedEventArgs>| {
                let args = (*args).as_ref().unwrap();
                let rate = args.RequestedPlaybackRate()?;
                (event_handler.lock().unwrap())(MediaControlEvent::SetRate(rate));
                Ok(())
            }
        });

        let shuffle_handler = TypedEventHandler::new({
            let event_handler = event_handler.clone();
            move |_, args: Ref<ShuffleEnabledChangeRequestedEventArgs>| {
                let args = (*args).as_ref().unwrap();
                let shuffle = args.RequestedShuffleEnabled()?;
                (event_handler.lock().unwrap())(MediaControlEvent::SetShuffle(shuffle));
                Ok(())
            }
        });

        let repeat_handler = TypedEventHandler::new({
            move |_, args: Ref<AutoRepeatModeChangeRequestedEventArgs>| {
                let args = (*args).as_ref().unwrap();
                let repeat = args.RequestedAutoRepeatMode()?;
                if let Some(repeat) = Repeat::from_native(repeat) {
                    (event_handler.lock().unwrap())(MediaControlEvent::SetRepeat(repeat));
                }
                Ok(())
            }
        });

        self.handlers = Some(Handlers {
            button: self.controls.ButtonPressed(&button_handler)?,
            playback_position: self
                .controls
                .PlaybackPositionChangeRequested(&position_handler)?,
            playback_rate: self.controls.PlaybackRateChangeRequested(&rate_handler)?,
            shuffle: self
                .controls
                .ShuffleEnabledChangeRequested(&shuffle_handler)?,
            repeat: self
                .controls
                .AutoRepeatModeChangeRequested(&repeat_handler)?,
        });

        Ok(())
    }

    fn detach(&mut self) -> Result<(), Self::Error> {
        self.controls.SetIsEnabled(false)?;
        if let Some(ref handlers) = self.handlers {
            self.controls.RemoveButtonPressed(handlers.button)?;
            self.controls
                .RemovePlaybackPositionChangeRequested(handlers.playback_position)?;
            self.controls
                .RemovePlaybackRateChangeRequested(handlers.playback_rate)?;
            self.controls
                .RemoveShuffleEnabledChangeRequested(handlers.shuffle)?;
            self.controls
                .RemoveAutoRepeatModeChangeRequested(handlers.repeat)?;
        }
        self.handlers = None;
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
        macro_rules! meta {
            ($properties:ident, $method:ident, $value:expr, ref $wrap:expr) => {
                if let Some(ref value) = $value {
                    $properties.$method(&$wrap(value.clone()))?;
                }
            };

            ($properties:ident, $method:ident, $value:expr, $wrap:expr) => {
                if let Some(ref value) = $value {
                    $properties.$method($wrap(value.clone()))?;
                }
            };
        }

        let MediaMetadata {
            title,
            artist,
            album_title,
            album_artist,
            genres,
            track_number,
            album_track_count,
            duration,
            media_type_windows,
            app_media_id,
            subtitle,
            ..
        } = metadata;

        let display = &self.display_updater;
        let music = display.MusicProperties()?;
        let image = display.ImageProperties()?;
        let video = display.VideoProperties()?;
        let h = |x| HSTRING::from(x);
        let to_u32 = |x| (x as u32);

        display.SetType(
            media_type_windows
                .unwrap_or(MediaTypeWindows::Music)
                .into_native(),
        )?;
        meta!(display, SetAppMediaId, app_media_id, ref h);
        meta!(music, SetTitle, title, ref h);
        meta!(music, SetArtist, artist, ref h);
        meta!(music, SetAlbumTitle, album_title, ref h);
        meta!(music, SetAlbumArtist, album_artist, ref h);
        meta!(music, SetTrackNumber, track_number, to_u32);
        meta!(music, SetAlbumTrackCount, album_track_count, to_u32);
        meta!(video, SetTitle, title, ref h);
        meta!(video, SetSubtitle, subtitle, ref h);
        meta!(image, SetTitle, title, ref h);
        meta!(image, SetSubtitle, subtitle, ref h);

        // TODO: We should allow setting shared Image and Video genres separately.
        if let Some(genres) = genres {
            let genres = genres.into_iter().map(|x| HSTRING::from(x));
            let genres_windows_music = music.Genres()?;
            let genres_windows_video = video.Genres()?;

            // Kinda clunky, but ok.
            genres_windows_music.Clear()?;
            genres_windows_video.Clear()?;
            for genre in genres {
                genres_windows_music.Append(&genre)?;
                genres_windows_video.Append(&genre)?;
            }
        }

        let duration = duration.unwrap_or_default();
        self.timeline_properties.SetStartTime(TimeSpan::default())?;
        self.timeline_properties
            .SetMinSeekTime(TimeSpan::default())?;
        self.timeline_properties
            .SetEndTime(TimeSpan::from(duration))?;
        self.timeline_properties
            .SetMaxSeekTime(TimeSpan::from(duration))?;

        self.controls
            .UpdateTimelineProperties(&self.timeline_properties)?;
        display.Update()?;
        Ok(())
    }

    fn set_cover(&mut self, cover: Option<Self::Cover>) -> Result<(), Self::Error> {
        let stream = match cover {
            Some(WindowsCover::Uri(uri)) => {
                RandomAccessStreamReference::CreateFromUri(&Uri::CreateUri(&HSTRING::from(uri))?)?
            }
            Some(WindowsCover::LocalFile(path)) => {
                let loader = windows::Storage::StorageFile::GetFileFromPathAsync(&HSTRING::from(
                    path.as_path(),
                ))?;
                let results = loader.get()?;
                loader.Close()?;
                RandomAccessStreamReference::CreateFromFile(&results)?
            }
            // TODO: Verify if this works on Windows
            Some(WindowsCover::Bytes(bytes)) => {
                let stream = create_stream_from_bytes(bytes)?;
                RandomAccessStreamReference::CreateFromStream(&stream)?
            }
            None => todo!(),
        };
        self.display_updater.SetThumbnail(&stream)?;
        self.display_updater.Update()?;
        Ok(())
    }

    fn set_repeat(&mut self, repeat: Repeat) -> Result<(), Self::Error> {
        self.controls.SetAutoRepeatMode(repeat.into_native())
    }

    fn set_shuffle(&mut self, shuffle: bool) -> Result<(), Self::Error> {
        self.controls.SetShuffleEnabled(shuffle)
    }

    fn set_volume(&mut self, _volume: f64) -> Result<(), Self::Error> {
        // Unsupported by Windows. No-op.
        Ok(())
    }

    fn set_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.controls.SetPlaybackRate(rate)
    }

    fn set_rate_limits(&mut self, _min: f64, _max: f64) -> Result<(), Self::Error> {
        // Unsupported by Windows. No-op.
        Ok(())
    }
}

fn create_stream_from_bytes(data: Vec<u8>) -> Result<IRandomAccessStream, WindowsError> {
    let stream = InMemoryRandomAccessStream::new()?;

    let output_stream = stream.GetOutputStreamAt(0)?;
    let writer = DataWriter::CreateDataWriter(&output_stream)?;
    writer.WriteBytes(&data)?;
    writer.StoreAsync()?.get()?;
    writer.Close()?;
    output_stream.Close()?;

    Ok(stream.cast::<IRandomAccessStream>()?)
}

impl MediaTypeWindows {
    pub fn into_native(self) -> MediaPlaybackType {
        match self {
            MediaTypeWindows::Unknown => MediaPlaybackType::Unknown,
            MediaTypeWindows::Music => MediaPlaybackType::Music,
            MediaTypeWindows::Video => MediaPlaybackType::Video,
            MediaTypeWindows::Image => MediaPlaybackType::Image,
        }
    }
}

impl Repeat {
    fn from_native(mode: MediaPlaybackAutoRepeatMode) -> Option<Self> {
        if mode == MediaPlaybackAutoRepeatMode::None {
            Some(Repeat::None)
        } else if mode == MediaPlaybackAutoRepeatMode::Track {
            Some(Repeat::Track)
        } else if mode == MediaPlaybackAutoRepeatMode::List {
            Some(Repeat::Playlist)
        } else {
            None
        }
    }
    fn into_native(self) -> MediaPlaybackAutoRepeatMode {
        match self {
            Repeat::None => MediaPlaybackAutoRepeatMode::None,
            Repeat::Track => MediaPlaybackAutoRepeatMode::Track,
            Repeat::Playlist => MediaPlaybackAutoRepeatMode::List,
        }
    }
}
