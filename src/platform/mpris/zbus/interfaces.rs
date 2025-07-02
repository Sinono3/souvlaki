use std::convert::From;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use zbus::interface;

use super::super::{MetadataDict, ServiceState};
use crate::{MediaControlEvent, MediaPlayback, MediaPosition, Repeat, SeekDirection};

pub(super) struct AppInterface {
    pub friendly_name: String,
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
        true
    }

    #[zbus(property)]
    fn can_raise(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn has_tracklist(&self) -> bool {
        false
    }

    #[zbus(property)]
    fn identity(&self) -> &str {
        &self.friendly_name
    }

    #[zbus(property)]
    fn supported_uri_schemes(&self) -> &[&str] {
        &[]
    }

    #[zbus(property)]
    fn supported_mime_types(&self) -> &[&str] {
        &[]
    }
}

impl AppInterface {
    fn send_event(&self, event: MediaControlEvent) {
        (self.event_handler.lock().unwrap())(event);
    }
}

pub(super) struct PlayerInterface {
    pub state: ServiceState,
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

        // NOTE: Should the `Seeked` signal be called when calling this method?
    }

    fn set_position(&self, _track_id: zbus::zvariant::ObjectPath, position: i64) {
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

    #[zbus(property)]
    fn playback_status(&self) -> &'static str {
        self.state.playback_status.to_dbus_value()
    }

    #[zbus(property)]
    fn loop_status(&self) -> &'static str {
        self.state.loop_status.to_dbus_value()
    }

    #[zbus(property)]
    fn set_loop_status(&self, loop_status: &str) {
        if let Some(repeat) = Repeat::from_dbus_value(loop_status) {
            self.send_event(MediaControlEvent::SetRepeat(repeat));
        }
    }

    #[zbus(property)]
    fn rate(&self) -> f64 {
        self.state.rate
    }

    #[zbus(property)]
    fn set_rate(&self, rate: f64) {
        self.send_event(MediaControlEvent::SetRate(rate));
    }

    #[zbus(property)]
    fn shuffle(&self) -> bool {
        self.state.shuffle
    }

    #[zbus(property)]
    fn set_shuffle(&self, shuffle: bool) {
        self.send_event(MediaControlEvent::SetShuffle(shuffle));
    }

    #[zbus(property)]
    fn metadata(&self) -> MetadataDict {
        self.state.metadata_dict.clone()
    }

    #[zbus(property)]
    fn volume(&self) -> f64 {
        self.state.volume
    }

    #[zbus(property)]
    fn set_volume(&self, volume: f64) {
        self.send_event(MediaControlEvent::SetVolume(volume));
    }

    #[zbus(property)]
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

    #[zbus(property)]
    fn maximum_rate(&self) -> f64 {
        self.state.maximum_rate
    }

    #[zbus(property)]
    fn minimum_rate(&self) -> f64 {
        self.state.minimum_rate
    }

    #[zbus(property)]
    fn can_go_next(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_go_previous(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_play(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_pause(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_seek(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn can_control(&self) -> bool {
        true
    }
}
