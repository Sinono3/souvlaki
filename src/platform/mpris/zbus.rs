use std::convert::From;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use zbus::{dbus_interface, ConnectionBuilder, SignalContext};
use zvariant::ObjectPath;

use super::{
    create_metadata_dict, InternalEvent, MprisCover, MprisError, ServiceState, ServiceThreadHandle,
};
use crate::platform::mpris::MetadataDict;
use crate::Loop;
use crate::{MediaControlEvent, MediaPlayback, MediaPosition, SeekDirection};

pub(super) fn spawn_thread<F>(
    event_handler: F,
    dbus_name: String,
    friendly_name: String,
    event_channel: mpsc::Sender<InternalEvent>,
    rx: mpsc::Receiver<InternalEvent>,
) -> Result<ServiceThreadHandle, MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    Ok(ServiceThreadHandle {
        event_channel,
        thread: thread::spawn(move || {
            pollster::block_on(run_service(dbus_name, friendly_name, event_handler, rx))
                .map_err(|e| e.into())
        }),
    })
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
    fn metadata(&self) -> MetadataDict {
        self.state.metadata_dict.clone()
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

async fn run_service<F>(
    dbus_name: String,
    friendly_name: String,
    event_handler: F,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> Result<(), MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    let event_handler = Arc::new(Mutex::new(event_handler));
    let app = AppInterface {
        friendly_name,
        event_handler: event_handler.clone(),
    };

    let player = PlayerInterface {
        state: ServiceState::default(),
        event_handler,
    };

    let name = format!("org.mpris.MediaPlayer2.{dbus_name}");
    let path = ObjectPath::try_from("/org/mpris/MediaPlayer2").unwrap();
    let connection = ConnectionBuilder::session()?
        .serve_at(&path, app)?
        .serve_at(&path, player)?
        .name(name.as_str())?
        .build()
        .await?;

    loop {
        while let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let interface_ref = connection
                .object_server()
                .interface::<_, PlayerInterface>(&path)
                .await?;
            let mut interface = interface_ref.get_mut().await;
            let ctxt = SignalContext::new(&connection, &path)?;

            match event {
                InternalEvent::SetMetadata(metadata) => {
                    interface.state.metadata_dict =
                        create_metadata_dict(&metadata, &interface.state.cover_url);
                    interface.state.metadata = metadata;
                    interface.metadata_changed(&ctxt).await?;
                }
                InternalEvent::SetCover(cover) => {
                    let cover_url = if let Some(MprisCover::Url(cover_url)) = cover {
                        Some(cover_url)
                    } else {
                        None
                    };

                    interface.state.metadata_dict =
                        create_metadata_dict(&interface.state.metadata, &cover_url);
                    interface.state.cover_url = cover_url;
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
                InternalEvent::SetRate(rate) => {
                    interface.state.rate = rate;
                    interface.rate_changed(&ctxt).await?;
                }
                InternalEvent::SetShuffle(shuffle) => {
                    interface.state.shuffle = shuffle;
                    interface.shuffle_changed(&ctxt).await?;
                }
                InternalEvent::SetVolume(volume) => {
                    interface.state.volume = volume;
                    interface.volume_changed(&ctxt).await?;
                }
                InternalEvent::SetMaximumRate(rate) => {
                    interface.state.maximum_rate = rate;
                    interface.maximum_rate_changed(&ctxt).await?;
                }
                InternalEvent::SetMinimumRate(rate) => {
                    interface.state.minimum_rate = rate;
                    interface.minimum_rate_changed(&ctxt).await?;
                }
                InternalEvent::Kill => return Ok(()),
            }
        }
    }
}
