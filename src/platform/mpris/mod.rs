#![cfg(platform_mpris)]

#[cfg(not(any(feature = "dbus", feature = "zbus")))]
compile_error!("either feature \"dbus\" or feature \"zbus\" are required");

#[cfg(all(feature = "dbus", feature = "zbus"))]
compile_error!("feature \"dbus\" and feature \"zbus\" are mutually exclusive");

#[cfg(feature = "zbus")]
mod zbus;
#[cfg(feature = "zbus")]
pub use self::zbus::Zbus as Mpris;

#[cfg(feature = "dbus")]
mod dbus;
#[cfg(feature = "dbus")]
pub use self::dbus::Dbus as Mpris;

/// A platform-specific error.
#[derive(thiserror::Error, Debug)]
pub enum MprisError {
    #[error("internal D-Bus error: {0}")]
    #[cfg(feature = "dbus")]
    DbusError(#[from] ::dbus::Error),
    #[error("internal D-Bus error: {0}")]
    #[cfg(feature = "zbus")]
    DbusError(#[from] ::zbus::Error),
    #[error("D-bus service thread not running. Run MediaControls::attach()")]
    ThreadNotRunning,
    // NOTE: For now this error is not very descriptive. For now we can't do much about it
    // since the panic message returned by JoinHandle::join does not implement Debug/Display,
    // thus we cannot print it, though perhaps there is another way. I will leave this error here,
    // to at least be able to catch it, but it is preferable to have this thread *not panic* at all.
    #[error("D-Bus service thread panicked")]
    ThreadPanicked,
}

use crate::{extensions::MprisPropertiesExt, Loop, MediaMetadata, MediaPlayback};
use std::collections::HashMap;
use std::{sync::mpsc, thread::JoinHandle};

struct ServiceThreadHandle {
    event_channel: mpsc::Sender<InternalEvent>,
    thread: JoinHandle<Result<(), MprisError>>,
}

#[derive(Clone, Debug)]
pub(crate) enum InternalEvent {
    SetMetadata(MediaMetadata),
    SetPlayback(MediaPlayback),
    SetLoopStatus(Loop),
    SetRate(f64),
    SetShuffle(bool),
    SetVolume(f64),
    SetMaximumRate(f64),
    SetMinimumRate(f64),
    Kill,
}

#[cfg(platform_mpris_dbus)]
type MetadataDict = HashMap<String, ::dbus::arg::Variant<Box<dyn ::dbus::arg::RefArg>>>;
#[cfg(platform_mpris_zbus)]
type MetadataDict = HashMap<String, zvariant::OwnedValue>;

// TODO: This is public only due to how rust modules work...
// should not actually be seen by the library user
#[derive(Debug)]
struct ServiceState {
    playback_status: MediaPlayback,
    loop_status: Loop,
    rate: f64,
    shuffle: bool,
    metadata: MediaMetadata,
    metadata_dict: MetadataDict,
    volume: f64,
    maximum_rate: f64,
    minimum_rate: f64,
}

impl Default for ServiceState {
    fn default() -> Self {
        let metadata = Default::default();
        let metadata_dict = create_metadata_dict(&metadata);

        Self {
            playback_status: MediaPlayback::Stopped,
            loop_status: Loop::None,
            rate: 1.0,
            shuffle: false,
            metadata,
            metadata_dict,
            volume: 1.0,
            maximum_rate: 1.0,
            minimum_rate: 1.0,
        }
    }
}

impl MprisPropertiesExt for Mpris {
    fn set_loop_status(&mut self, loop_status: Loop) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetLoopStatus(loop_status))
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

    fn set_maximum_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMaximumRate(rate))
    }

    fn set_minimum_rate(&mut self, rate: f64) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMinimumRate(rate))
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
        metadata: $metadata:expr
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

        #[rustfmt::skip]
        insert_if_some!(insert, $wrap,
            // TODO: Cover URL missing here
            // "mpris:artUrl", ...,
            "xesam:title", title,
            "xesam:artist", artists,
            "xesam:album", album_title,
            "xesam:albumArtist", album_artists,
            "xesam:genre", genres,
            "xesam:composer", composers,
            "xesam:lyricist", lyricists,
            "xesam:asText", lyrics,
            "xesam:comment", comments,
            "xesam:url", media_url,
                                                                                                );
                                                                                                use std::convert::TryFrom;
        #[rustfmt::skip]
        insert_if_some!(insert, $wrap, no_clone,
            "mpris:length", duration.map(|length| i64::try_from(length.as_micros()).unwrap()),
            "xesam:trackNumber", track_number,
            "xesam:discNumber", disc_number,
            "xesam:audioBPM", beats_per_minute,
            "xesam:userRating", user_rating_01,
            "xesam:autoRating", auto_rating,
            "xesam:playCount", play_count,
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

fn create_metadata_dict(metadata: &MediaMetadata) -> MetadataDict {
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
            metadata: metadata
        )
    }

    #[cfg(platform_mpris_zbus)]
    {
        use zvariant::{ObjectPath, Value};
        fn create_value<'a, T: Into<zvariant::Value<'a>> + zvariant::DynamicType>(
            x: T,
        ) -> zvariant::OwnedValue {
            Value::new(x).to_owned()
        }

        build_metadata_dict!(
            wrap: create_value,
            trackid_value: ObjectPath::try_from("/").unwrap(),
            metadata: metadata
        )
    }
}
// pub(self) use {build_metadata_dict, insert_if_some};
