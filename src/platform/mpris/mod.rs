#![cfg(platform_mpris)]

#[cfg(not(any(feature = "dbus", feature = "zbus")))]
compile_error!("either feature \"mpris_dbus\" or feature \"mpris_zbus\" are required");

#[cfg(all(feature = "dbus", feature = "zbus"))]
compile_error!("feature \"mpris_dbus\" and feature \"mpris_zbus\" are mutually exclusive");

#[cfg(platform_mpris_zbus)]
mod zbus;

#[cfg(platform_mpris_dbus)]
mod dbus;

/// MPRIS-specific configuration needed to create media controls.
#[derive(Clone, Debug)]
pub struct MprisConfig {
    /// Should follow [the D-Bus spec](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-names-bus).
    pub dbus_name: String,
    /// A friendly name to identify the media player to users.
    /// This should usually match the name found in .desktop files
    /// (eg: "VLC media player").
    pub identity: String,
    /// The basename of an installed .desktop file which complies with the Desktop entry specification, with the ".desktop" extension stripped.
    /// Example: The desktop entry file is "/usr/share/applications/vlc.desktop", and this property contains "vlc"
    pub desktop_entry: String,
}

/// A platform-specific error.
#[derive(thiserror::Error, Debug)]
pub enum MprisError {
    #[error("internal D-Bus error: {0}")]
    #[cfg(platform_mpris_dbus)]
    DbusError(#[from] ::dbus::Error),
    #[error("internal D-Bus error: {0}")]
    #[cfg(platform_mpris_zbus)]
    DbusError(#[from] ::zbus::Error),
    #[error("D-bus service thread not running. Run MediaControls::attach()")]
    ThreadNotRunning,
    // NOTE: For now this error is not very descriptive. For now we can't do much about it
    // since the panic message returned by JoinHandle::join does not implement Debug/Display,
    // thus we cannot print it, though perhaps there is another way. I will leave this error here,
    // to at least be able to catch it, but it is preferable to have this thread *not panic* at all.
    #[error("D-Bus service thread panicked")]
    ThreadPanicked,
    #[error("Couldnt't infer mimetype from cover image bytes. Make sure the image data is valid.")]
    InvalidCoverBytes,
}

/// Definition/reference to cover art for MPRIS platforms.
#[derive(Clone, Debug)]
pub enum MprisCover {
    /// Simply sets the metadata field `mpris:artUrl` to this string.
    /// It depends on the
    Url(String),
    // Even though it only has one option, it is an enum in case
    // we need further expansion in the future.
}

impl MprisCover {
    fn to_url(cover: Option<Self>) -> Option<String> {
        #[allow(clippy::manual_map)]
        match cover {
            Some(MprisCover::Url(cover_url)) => Some(cover_url),
            None => None,
        }
    }

    /// Sets the `mpris:artUrl` field to an Base64-encoded
    /// data URL of the provided image bytes.
    /// Can be inefficient, but it works and is easy to work with,
    /// in case you don't want to save to a file for the URL option.
    #[cfg(feature = "mpris_base64_data_url")]
    pub fn from_bytes(image_data: &[u8]) -> Result<Self, MprisError> {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE;
        let mimetype = infer::get(image_data).ok_or(MprisError::InvalidCoverBytes)?;
        let mut out = format!("data:{mimetype};base64,");
        engine.encode_string(image_data, &mut out);
        Ok(Self::Url(out))
    }
}

/// Permissions determining which actions can the user take.
/// Souvlaki does not actually enforce any of them, it just
/// displays them on the D-Bus interface.
/// We leave it up to the application code to enforcement.
#[derive(Clone, Debug, PartialEq)]
pub struct MprisPermissions {
    pub can_quit: bool,
    pub can_set_fullscreen: bool,
    pub can_raise: bool,
    pub supported_uri_schemes: Vec<&'static str>,
    pub supported_mime_types: Vec<&'static str>,
    pub can_go_next: bool,
    pub can_go_previous: bool,
    pub can_play: bool,
    pub can_pause: bool,
    pub can_seek: bool,
    pub can_control: bool,
    pub min_rate: f64,
    pub max_rate: f64,
}

impl MprisPermissions {
    pub fn none() -> Self {
        Self {
            can_quit: false,
            can_set_fullscreen: false,
            can_raise: false,
            supported_uri_schemes: vec![],
            supported_mime_types: vec![],
            can_go_next: false,
            can_go_previous: false,
            can_play: false,
            can_pause: false,
            can_seek: false,
            can_control: false,
            min_rate: 1.0,
            max_rate: 1.0,
        }
    }
}

use crate::{MediaControlEvent, MediaControls};
use crate::{MediaMetadata, MediaPlayback, Repeat};
use std::collections::HashMap;
use std::{sync::mpsc, thread::JoinHandle};

struct ServiceThreadHandle {
    event_channel: mpsc::Sender<InternalEvent>,
    thread: JoinHandle<Result<(), MprisError>>,
}

#[derive(Clone, Debug)]
pub(crate) enum InternalEvent {
    SetPermissions(MprisPermissions),
    SetMetadata(MediaMetadata),
    SetCover(Option<MprisCover>),
    SetPlayback(MediaPlayback),
    SetLoopStatus(Repeat),
    SetRate(f64),
    SetShuffle(bool),
    SetVolume(f64),
    SetFullscreen(bool),
    Kill,
}

#[cfg(platform_mpris_dbus)]
type MetadataDict = HashMap<String, ::dbus::arg::Variant<Box<dyn ::dbus::arg::RefArg>>>;
#[cfg(platform_mpris_zbus)]
type MetadataDict = HashMap<String, ::zbus::zvariant::OwnedValue>;

#[derive(Debug)]
struct ServiceState {
    permissions: MprisPermissions,
    fullscreen: bool,
    playback_status: MediaPlayback,
    loop_status: Repeat,
    rate: f64,
    shuffle: bool,
    metadata: MediaMetadata,
    metadata_dict: MetadataDict,
    cover_url: Option<String>,
    volume: f64,
}

impl Default for ServiceState {
    fn default() -> Self {
        let metadata = Default::default();
        let metadata_dict = create_metadata_dict(&metadata, &None);

        Self {
            permissions: MprisPermissions::none(),
            fullscreen: false,
            playback_status: MediaPlayback::Stopped,
            loop_status: Repeat::None,
            rate: 1.0,
            shuffle: false,
            metadata,
            metadata_dict,
            cover_url: None,
            volume: 1.0,
        }
    }
}

/// A handle to OS media controls.
pub struct Mpris {
    thread: Option<ServiceThreadHandle>,
    config: MprisConfig,
}

impl Mpris {
    pub(in super::super) fn send_internal_event(
        &mut self,
        event: InternalEvent,
    ) -> Result<(), MprisError> {
        let channel = &self
            .thread
            .as_ref()
            .ok_or(MprisError::ThreadNotRunning)?
            .event_channel;
        channel.send(event).map_err(|_| MprisError::ThreadPanicked)
    }
}

impl std::fmt::Debug for Mpris {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Mpris")?;
        Ok(())
    }
}

impl MediaControls for Mpris {
    type Error = MprisError;
    type PlatformConfig = MprisConfig;
    type Cover = MprisCover;
    type Permissions = MprisPermissions;

    fn new(config: Self::PlatformConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            thread: None,
            config,
        })
    }

    fn attach<F>(&mut self, event_handler: F) -> Result<(), Self::Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.detach()?;

        let (event_channel, rx) = mpsc::channel();

        #[cfg(platform_mpris_dbus)]
        let thread =
            self::dbus::spawn_thread(event_handler, self.config.clone(), event_channel, rx)?;
        #[cfg(platform_mpris_zbus)]
        let thread =
            self::zbus::spawn_thread(event_handler, self.config.clone(), event_channel, rx)?;

        self.thread = Some(thread);
        Ok(())
    }

    fn detach(&mut self) -> Result<(), Self::Error> {
        if let Some(ServiceThreadHandle {
            event_channel,
            thread,
        }) = self.thread.take()
        {
            event_channel.send(InternalEvent::Kill).ok();
            thread.join().map_err(|_| Self::Error::ThreadPanicked)??;
        }
        Ok(())
    }

    fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetPlayback(playback))
    }

    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMetadata(metadata))
    }

    fn set_cover(&mut self, cover: Option<Self::Cover>) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetCover(cover))
    }

    fn set_repeat(&mut self, repeat: Repeat) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetLoopStatus(repeat))
    }

    fn set_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetRate(rate))
    }

    fn set_shuffle(&mut self, shuffle: bool) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetShuffle(shuffle))
    }

    fn set_volume(&mut self, volume: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetVolume(volume))
    }

    fn set_permissions(&mut self, permissions: Self::Permissions) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetPermissions(permissions))
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetFullscreen(fullscreen))
    }
}

// Macro for constructing metadata fields
macro_rules! insert_if_some {
    ($insert:expr, $wrap:path, $($key:literal, $value:expr),* $(,)?) => {
        $(
            if let Some(value) = $value {
                ($insert)($key.to_string(), ($wrap)(value.clone()));
            }
        )*
    };
    // Variant for values that don't need cloning
    ($insert:expr, $wrap:path, no_clone, $($key:literal, $value:expr),* $(,)?) => {
        $(
            if let Some(value) = $value {
                ($insert)($key.to_string(), ($wrap)(value));
            }
        )*
    };
}

macro_rules! build_metadata_dict {
    (
        wrap: $wrap:path,
        trackid_value: $trackid_value:expr,
        metadata: $metadata:expr,
        cover_url: $cover_url:expr,
    ) => {{
        let mut dict = MetadataDict::new();

        let &$crate::MediaMetadata {
            ref title,
            ref album_title,
            ref artists,
            ref album_artists,
            ref genres,
            track_number,
            disc_number,
            ref composers,
            ref lyricists,
            ref lyrics,
            ref comments,
            beats_per_minute,
            user_rating_01,
            auto_rating,
            play_count,
            ref media_url,
            duration,
            ..
        } = $metadata;

        // TODO: Workaround to enable SetPosition.
        dict.insert("mpris:trackid".to_string(), ($wrap)($trackid_value));

        let mut insert = |k, v| dict.insert(k, v);

        insert_if_some!(
            insert,
            $wrap,
            "mpris:artUrl",
            $cover_url,
            "xesam:title",
            title,
            "xesam:artist",
            artists,
            "xesam:album",
            album_title,
            "xesam:albumArtist",
            album_artists,
            "xesam:genre",
            genres,
            "xesam:composer",
            composers,
            "xesam:lyricist",
            lyricists,
            "xesam:asText",
            lyrics,
            "xesam:comment",
            comments,
            "xesam:url",
            media_url,
        );

        use std::convert::TryFrom;
        insert_if_some!(
            insert,
            $wrap,
            no_clone,
            "mpris:length",
            duration.map(|length| i64::try_from(length.as_micros()).unwrap()),
            "xesam:trackNumber",
            track_number,
            "xesam:discNumber",
            disc_number,
            "xesam:audioBPM",
            beats_per_minute,
            "xesam:userRating",
            user_rating_01,
            "xesam:autoRating",
            auto_rating,
            "xesam:playCount",
            play_count
        );

        #[cfg(feature = "date")]
        {
            let &MediaMetadata {
                ref content_created,
                ref first_played,
                ref last_played,
            } = $metadata;
            // TODO: handle date types
            todo!();
        }

        dict
    }};
}

fn create_metadata_dict(metadata: &MediaMetadata, cover_url: &Option<String>) -> MetadataDict {
    #[cfg(platform_mpris_dbus)]
    {
        use ::dbus::arg::{RefArg, Variant};
        use ::dbus::Path;
        fn make_variant<T: RefArg + 'static>(x: T) -> Variant<Box<dyn RefArg + 'static>> {
            Variant(Box::new(x))
        }

        build_metadata_dict!(
            wrap: make_variant,
            trackid_value: Path::new("/").unwrap(),
            metadata: metadata,
            cover_url: cover_url,
        )
    }

    #[cfg(platform_mpris_zbus)]
    {
        use ::zbus::zvariant::{ObjectPath, Value};
        fn create_value<
            'a,
            T: Into<::zbus::zvariant::Value<'a>> + ::zbus::zvariant::DynamicType,
        >(
            x: T,
        ) -> ::zbus::zvariant::OwnedValue {
            Value::new(x).try_to_owned().unwrap()
        }

        build_metadata_dict!(
            wrap: create_value,
            trackid_value: ObjectPath::try_from("/").unwrap(),
            metadata: metadata,
            cover_url: cover_url,
        )
    }
}
