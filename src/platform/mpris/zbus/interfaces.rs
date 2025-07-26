use std::convert::From;
use std::convert::{TryInto, TryFrom};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use zbus::interface;
use zbus::object_server::SignalEmitter;

use super::super::{MetadataDict, MprisConfig, ServiceState};
use crate::{MediaControlEvent, MediaPlayback, MediaPosition, Repeat, SeekDirection};

pub(super) struct AppInterface {
    pub config: MprisConfig,
    pub state: Arc<Mutex<ServiceState>>,
    pub event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
}

#[interface(name = "org.mpris.MediaPlayer2")]
impl AppInterface {
    fn raise(&self) {
        self.send_event(MediaControlEvent::Raise);
    }
    fn quit(&self) {
        self.send_event(MediaControlEvent::Quit);
    }

    #[zbus(property)]
    fn can_quit(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_quit
    }

    #[zbus(property)]
    fn fullscreen(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.fullscreen
    }

    #[zbus(property)]
    fn set_fullscreen(&self, fullscreen: bool) {
        self.send_event(MediaControlEvent::SetFullscreen(fullscreen));
    }

    #[zbus(property)]
    fn can_set_fullscreen(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_set_fullscreen
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_raise
    }

    #[zbus(property)]
    fn has_tracklist(&self) -> bool {
        // TODO: check issue #73
        false
    }

    #[zbus(property)]
    fn identity(&self) -> &str {
        &self.config.identity
    }

    #[zbus(property)]
    fn desktop_entry(&self) -> &str {
        &self.config.desktop_entry
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> Vec<&str> {
        let state = self.state.lock().unwrap();
        state.permissions.supported_uri_schemes.to_vec()
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> Vec<&str> {
        let state = self.state.lock().unwrap();
        state.permissions.supported_mime_types.to_vec()
    }
}

impl AppInterface {
    fn send_event(&self, event: MediaControlEvent) {
        (self.event_handler.lock().unwrap())(event);
    }
}

pub(super) struct PlayerInterface {
    pub state: Arc<Mutex<ServiceState>>,
    pub event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
}

impl PlayerInterface {
    pub fn send_event(&self, event: MediaControlEvent) {
        (self.event_handler.lock().unwrap())(event);
    }
}

#[interface(name = "org.mpris.MediaPlayer2.Player")]
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
    }

    fn set_position(&self, _track_id: zbus::zvariant::ObjectPath, position: i64) {
        if let Ok(position) = u64::try_from(position) {
            self.send_event(MediaControlEvent::SetPosition(MediaPosition(Duration::from_micros(position))));
        }
    }

    fn open_uri(&self, uri: String) {
        self.send_event(MediaControlEvent::OpenUri(uri));
    }

    #[zbus(signal)]
    pub async fn seeked(&self, _emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property)]
    fn playback_status(&self) -> &'static str {
        let state = self.state.lock().unwrap();
        state.playback_status.to_dbus_value()
    }

    #[zbus(property)]
    fn loop_status(&self) -> &'static str {
        let state = self.state.lock().unwrap();
        state.loop_status.to_dbus_value()
    }

    #[zbus(property)]
    fn set_loop_status(&self, loop_status: &str) {
        if let Some(repeat) = Repeat::from_dbus_value(loop_status) {
            self.send_event(MediaControlEvent::SetRepeat(repeat));
        }
    }

    #[zbus(property)]
    fn rate(&self) -> f64 {
        let state = self.state.lock().unwrap();
        state.rate
    }

    #[zbus(property)]
    fn set_rate(&self, rate: f64) {
        self.send_event(MediaControlEvent::SetRate(rate));
    }

    #[zbus(property)]
    fn shuffle(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.shuffle
    }

    #[zbus(property)]
    fn set_shuffle(&self, shuffle: bool) {
        self.send_event(MediaControlEvent::SetShuffle(shuffle));
    }

    #[zbus(property)]
    fn metadata(&self) -> MetadataDict {
        let state = self.state.lock().unwrap();
        state.metadata_dict.clone()
    }

    #[zbus(property)]
    fn volume(&self) -> f64 {
        let state = self.state.lock().unwrap();
        state.volume
    }

    #[zbus(property)]
    fn set_volume(&self, volume: f64) {
        self.send_event(MediaControlEvent::SetVolume(volume));
    }

    #[zbus(property)]
    fn position(&self) -> i64 {
        let state = self.state.lock().unwrap();
        let position = match state.playback_status {
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

    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        let state = self.state.lock().unwrap();
        state.permissions.max_rate
    }

    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        let state = self.state.lock().unwrap();
        state.permissions.min_rate
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_go_next
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_go_previous
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_play
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_pause
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_seek
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.permissions.can_control
    }
}
