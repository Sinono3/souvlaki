#![cfg(target_os = "linux")]

use crate::{
    MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig, SeekDirection,
};
use std::collections::HashMap;
use std::convert::From;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use zbus::{dbus_interface, SignalContext};
use zvariant::{ObjectPath, Value};

/// A platform-specific error.
#[derive(Debug)]
pub struct Error;

/// A handle to OS media controls.
pub struct MediaControls {
    thread: Option<ServiceThreadHandle>,
    default_service_state: ServiceState,
}

struct ServiceThreadHandle {
    event_channel: mpsc::Sender<InternalEvent>,
    thread: JoinHandle<()>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum InternalEvent {
    ChangeMetadata(OwnedMetadata),
    ChangePlayback(MediaPlayback),
    Kill,
}

#[derive(Clone, Debug)]
struct ServiceState {
    dbus_name: String,
    friendly_name: String,
    metadata: OwnedMetadata,
    playback_status: MediaPlayback,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct OwnedMetadata {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<i64>,
}

impl From<MediaMetadata<'_>> for OwnedMetadata {
    fn from(other: MediaMetadata) -> Self {
        OwnedMetadata {
            title: other.title.map(|s| s.to_string()),
            artist: other.artist.map(|s| s.to_string()),
            album: other.album.map(|s| s.to_string()),
            cover_url: other.cover_url.map(|s| s.to_string()),
            duration: other.duration.map(|d| d.as_micros().try_into().unwrap()),
        }
    }
}

impl MediaControls {
    /// Create media controls with the specified config.
    pub fn new(config: PlatformConfig) -> Result<Self, Error> {
        let PlatformConfig {
            dbus_name,
            display_name,
            ..
        } = config;

        let default_service_state = ServiceState {
            dbus_name: dbus_name.to_string(),
            friendly_name: display_name.to_string(),
            metadata: Default::default(),
            playback_status: MediaPlayback::Stopped,
        };

        Ok(Self {
            thread: None,
            default_service_state,
        })
    }

    /// Attach the media control events to a handler.
    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.detach()?;

        let initial_state = self.default_service_state.clone();
        let event_handler = Arc::new(Mutex::new(event_handler));
        let (event_channel, rx) = mpsc::channel();

        self.thread = Some(ServiceThreadHandle {
            event_channel,
            thread: thread::spawn(move || {
                pollster::block_on(run_service(initial_state, event_handler, rx)).unwrap();
            }),
        });
        Ok(())
    }
    /// Detach the event handler.
    pub fn detach(&mut self) -> Result<(), Error> {
        if let Some(ServiceThreadHandle {
            event_channel,
            thread,
        }) = self.thread.take()
        {
            event_channel.send(InternalEvent::Kill).unwrap();
            thread.join().unwrap();
        }
        Ok(())
    }

    /// Set the current playback status.
    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        self.send_internal_event(InternalEvent::ChangePlayback(playback));
        Ok(())
    }

    /// Set the metadata of the currently playing media item.
    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        self.send_internal_event(InternalEvent::ChangeMetadata(metadata.into()));
        Ok(())
    }

    // TODO: result
    fn send_internal_event(&mut self, event: InternalEvent) {
        let channel = &self.thread.as_ref().unwrap().event_channel;
        channel.send(event).unwrap();
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

    #[dbus_interface(signal)]
    async fn seeked(&self, ctxt: &zbus::SignalContext<'_>) -> zbus::Result<()>;

    async fn seek(&self, offset: i64) {
        let abs_offset = offset.abs() as u64;
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
                if position > duration {
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

    #[dbus_interface(property)]
    fn playback_status(&self) -> &'static str {
        match self.state.playback_status {
            MediaPlayback::Playing { .. } => "Playing",
            MediaPlayback::Paused { .. } => "Paused",
            MediaPlayback::Stopped => "Stopped",
        }
    }

    #[dbus_interface(property)]
    fn rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn metadata(&self) -> HashMap<&str, Value> {
        // TODO: this should be stored in a cache inside the state.
        let mut dict = HashMap::<&str, Value>::new();

        let OwnedMetadata {
            ref title,
            ref album,
            ref artist,
            ref cover_url,
            ref duration,
        } = self.state.metadata;

        // MPRIS
        dict.insert(
            "mpris:trackid",
            // TODO: this is just a workaround to enable SetPosition.
            Value::new(ObjectPath::try_from("/").unwrap()),
        );

        if let Some(length) = duration {
            dict.insert("mpris:length", Value::new(*length));
        }

        if let Some(cover_url) = cover_url {
            dict.insert("mpris:artUrl", Value::new(cover_url.clone()));
        }

        // Xesam
        if let Some(title) = title {
            dict.insert("xesam:title", Value::new(title.clone()));
        }
        if let Some(artist) = artist {
            dict.insert("xesam:albumArtist", Value::new(artist.clone()));
        }
        if let Some(album) = album {
            dict.insert("xesam:album", Value::new(album.clone()));
        }
        dict
    }

    #[dbus_interface(property)]
    fn volume(&self) -> f64 {
        1.0
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
    fn minimum_rate(&self) -> f64 {
        1.0
    }

    #[dbus_interface(property)]
    fn maximum_rate(&self) -> f64 {
        1.0
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

use zbus::Connection;

async fn run_service(
    initial_state: ServiceState,
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> zbus::Result<()> {
    let name = format!("org.mpris.MediaPlayer2.{}", initial_state.dbus_name);
    let connection = Connection::session().await?;

    let app = AppInterface {
        friendly_name: initial_state.friendly_name.clone(),
        event_handler: event_handler.clone(),
    };

    let player = PlayerInterface {
        state: initial_state,
        event_handler,
    };

    let path = ObjectPath::try_from("/org/mpris/MediaPlayer2")?;
    connection.object_server_mut().await.at(&path, app)?;
    connection.object_server_mut().await.at(&path, player)?;
    connection.request_name(name.as_str()).await?;

    loop {
        if let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            if event == InternalEvent::Kill {
                break;
            }

            let obj = connection.object_server_mut().await;
            let mut i = obj.get_interface_mut::<_, PlayerInterface>(&path).await?;
            let ctxt = SignalContext::new(&connection, &path)?;

            match event {
                InternalEvent::ChangeMetadata(metadata) => {
                    i.state.metadata = metadata;
                    i.metadata_changed(&ctxt).await?;
                }
                InternalEvent::ChangePlayback(playback) => {
                    i.state.playback_status = playback;
                    i.playback_status_changed(&ctxt).await?;
                }
                _ => (),
            }
        }
    }

    Ok(())
}
