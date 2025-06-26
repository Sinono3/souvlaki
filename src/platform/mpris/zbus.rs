use std::collections::HashMap;
use std::convert::From;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use zbus::{dbus_interface, ConnectionBuilder, SignalContext};
use zvariant::{ObjectPath, Value};

use super::insert_if_some;
use super::InternalEvent;
use super::MprisError;
use super::ServiceState;
use super::ServiceThreadHandle;
use crate::Loop;
use crate::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
    SeekDirection,
};

/// A handle to OS media controls.
pub struct Zbus {
    thread: Option<ServiceThreadHandle>,
    dbus_name: String,
    friendly_name: String,
}

impl Zbus {
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

impl MediaControls for Zbus {
    type Error = MprisError;

    fn new(config: PlatformConfig) -> Result<Self, Self::Error> {
        let PlatformConfig {
            dbus_name,
            display_name,
            ..
        } = config;

        Ok(Self {
            thread: None,
            dbus_name: dbus_name.to_string(),
            friendly_name: display_name.to_string(),
        })
    }

    fn attach<F>(&mut self, event_handler: F) -> Result<(), Self::Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.detach()?;

        let dbus_name = self.dbus_name.clone();
        let friendly_name = self.friendly_name.clone();
        let event_handler = Arc::new(Mutex::new(event_handler));
        let (event_channel, rx) = mpsc::channel();

        self.thread = Some(ServiceThreadHandle {
            event_channel,
            thread: thread::spawn(move || {
                pollster::block_on(run_service(dbus_name, friendly_name, event_handler, rx))
                    .map_err(|e| e.into())
            }),
        });
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
        self.send_internal_event(InternalEvent::SetMetadata(metadata.into()))
    }
}

struct AppInterface {
    friendly_name: String,
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
}

#[dbus_interface(name = "org.mpris.MediaPlayer2")]
impl AppInterface {
    fn raise(&self) {
        self.send_event(MediaControlEvent::Raise);
    }
    fn quit(&self) {
        self.send_event(MediaControlEvent::Quit);
    }

    #[dbus_interface(property)]
    fn can_quit(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn has_tracklist(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    fn identity(&self) -> &str {
        &self.friendly_name
    }

    #[dbus_interface(property)]
    fn supported_uri_schemes(&self) -> &[&str] {
        &[]
    }

    #[dbus_interface(property)]
    fn supported_mime_types(&self) -> &[&str] {
        &[]
    }
}

impl AppInterface {
    fn send_event(&self, event: MediaControlEvent) {
        (self.event_handler.lock().unwrap())(event);
    }
}

struct PlayerInterface {
    state: ServiceState,
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
}

impl PlayerInterface {
    fn send_event(&self, event: MediaControlEvent) {
        (self.event_handler.lock().unwrap())(event);
    }
}

#[dbus_interface(name = "org.mpris.MediaPlayer2.Player")]
impl PlayerInterface {
    fn next(&self) {
        self.send_event(MediaControlEvent::Next);
    }
    fn previous(&self) {
        self.send_event(MediaControlEvent::Previous);
    }
    fn pause(&self) {
        self.send_event(MediaControlEvent::Pause);
    }
    fn play_pause(&self) {
        self.send_event(MediaControlEvent::Toggle);
    }
    fn stop(&self) {
        self.send_event(MediaControlEvent::Stop);
    }
    fn play(&self) {
        self.send_event(MediaControlEvent::Play);
    }

    fn seek(&self, offset: i64) {
        let abs_offset = offset.unsigned_abs();
        let direction = if offset > 0 {
            SeekDirection::Forward
        } else {
            SeekDirection::Backward
        };

        self.send_event(MediaControlEvent::SeekBy(
            direction,
            Duration::from_micros(abs_offset),
        ));

        // NOTE: Should the `Seeked` signal be called when calling this method?
    }

    fn set_position(&self, _track_id: zvariant::ObjectPath, position: i64) {
        if let Ok(micros) = position.try_into() {
            if let Some(duration) = self.state.metadata.duration {
                // If the Position argument is greater than the track length, do nothing.
                if position > duration.as_micros().try_into().unwrap() {
                    return;
                }
            }

            let position = Duration::from_micros(micros);
            self.send_event(MediaControlEvent::SetPosition(MediaPosition(position)));
        }
    }

    fn open_uri(&self, uri: String) {
        // NOTE: we should check if the URI is in the `SupportedUriSchemes` list.
        self.send_event(MediaControlEvent::OpenUri(uri));
    }

    // TODO: Seeked signal missing

    #[dbus_interface(property)]
    fn playback_status(&self) -> &'static str {
        self.state.playback_status.to_dbus_value()
    }

    #[dbus_interface(property)]
    fn loop_status(&self) -> &'static str {
        self.state.loop_status.to_dbus_value()
    }

    #[dbus_interface(property)]
    fn set_loop_status(&self, loop_status: &str) {
        if let Some(loop_status) = Loop::from_dbus_value(loop_status) {
            self.send_event(MediaControlEvent::SetLoop(loop_status));
        }
    }

    #[dbus_interface(property)]
    fn rate(&self) -> f64 {
        self.state.rate
    }

    #[dbus_interface(property)]
    fn set_rate(&self, rate: f64) {
        self.send_event(MediaControlEvent::SetPlaybackRate(rate));
    }

    #[dbus_interface(property)]
    fn shuffle(&self) -> bool {
        self.state.shuffle
    }

    #[dbus_interface(property)]
    fn set_shuffle(&self, shuffle: bool) {
        self.send_event(MediaControlEvent::SetShuffle(shuffle));
    }

    #[dbus_interface(property)]
    fn metadata(&self) -> HashMap<&str, Value> {
        // TODO: this should be stored in a cache inside the state.
        let mut dict = HashMap::<&str, Value>::new();

        let MediaMetadata {
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
        } = self.state.metadata;

        // TODO: Workaround to enable SetPosition.
        dict.insert(
            "mpris:trackid",
            Value::new(ObjectPath::try_from("/").unwrap()),
        );

        #[rustfmt::skip]
        insert_if_some!(|k, v| dict.insert(k, v), Value,
            // Cover URL missing here
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
        #[rustfmt::skip]
        insert_if_some!(|k, v| dict.insert(k, v), Value, no_clone,
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
            } = metadata;
            // TODO: handle date types
            todo!();
        }
        dict
    }

    #[dbus_interface(property)]
    fn volume(&self) -> f64 {
        self.state.volume
    }

    #[dbus_interface(property)]
    fn set_volume(&self, volume: f64) {
        self.send_event(MediaControlEvent::SetVolume(volume));
    }

    #[dbus_interface(property)]
    fn position(&self) -> i64 {
        let position = match self.state.playback_status {
            MediaPlayback::Playing {
                progress: Some(pos),
            }
            | MediaPlayback::Paused {
                progress: Some(pos),
            } => pos.0.as_micros(),
            _ => 0,
        };

        position.try_into().unwrap_or(0)
    }

    #[dbus_interface(property)]
    fn maximum_rate(&self) -> f64 {
        self.state.maximum_rate
    }

    #[dbus_interface(property)]
    fn minimum_rate(&self) -> f64 {
        self.state.minimum_rate
    }

    #[dbus_interface(property)]
    fn can_go_next(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_seek(&self) -> bool {
        true
    }

    #[dbus_interface(property)]
    fn can_control(&self) -> bool {
        true
    }
}

async fn run_service(
    dbus_name: String,
    friendly_name: String,
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> zbus::Result<()> {
    let app = AppInterface {
        friendly_name,
        event_handler: event_handler.clone(),
    };

    let player = PlayerInterface {
        state: ServiceState {
            playback_status: MediaPlayback::Stopped,
            loop_status: Loop::None,
            rate: 1.0,
            shuffle: false,
            metadata: Default::default(),
            volume: 1.0,
            maximum_rate: 1.0,
            minimum_rate: 1.0,
        },
        event_handler,
    };

    let name = format!("org.mpris.MediaPlayer2.{dbus_name}");
    let path = ObjectPath::try_from("/org/mpris/MediaPlayer2")?;
    let connection = ConnectionBuilder::session()?
        .serve_at(&path, app)?
        .serve_at(&path, player)?
        .name(name.as_str())?
        .build()
        .await?;

    loop {
        if let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let interface_ref = connection
                .object_server()
                .interface::<_, PlayerInterface>(&path)
                .await?;
            let mut interface = interface_ref.get_mut().await;
            let ctxt = SignalContext::new(&connection, &path)?;

            match event {
                InternalEvent::SetMetadata(metadata) => {
                    interface.state.metadata = metadata;
                    interface.metadata_changed(&ctxt).await?;
                }
                InternalEvent::SetPlayback(playback) => {
                    interface.state.playback_status = playback;
                    interface.playback_status_changed(&ctxt).await?;
                }
                InternalEvent::SetLoopStatus(loop_status) => {
                    interface.state.loop_status = loop_status;
                    interface.loop_status_changed(&ctxt).await?;
                }
                InternalEvent::SetVolume(volume) => {
                    interface.state.volume = volume;
                    interface.volume_changed(&ctxt).await?;
                }
                InternalEvent::SetRate(rate) => {
                    interface.state.rate = rate;
                    interface.rate_changed(&ctxt).await?;
                }
                InternalEvent::SetMaximumRate(rate) => {
                    interface.state.maximum_rate = rate;
                    interface.maximum_rate_changed(&ctxt).await?;
                }
                InternalEvent::SetMinimumRate(rate) => {
                    interface.state.minimum_rate = rate;
                    interface.minimum_rate_changed(&ctxt).await?;
                }
                InternalEvent::SetShuffle(shuffle) => {
                    interface.state.shuffle = shuffle;
                    interface.shuffle_changed(&ctxt).await?;
                }
                InternalEvent::Kill => break,
            }
        }
    }

    Ok(())
}
