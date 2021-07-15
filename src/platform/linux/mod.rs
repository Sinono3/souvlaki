#![cfg(target_os = "linux")]

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback, MediaPosition, SeekDirection};
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::strings::Path as DbusPath;
use dbus::Error as DbusError;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use std::collections::HashMap;
use std::convert::From;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Error;

pub struct MediaControls {
    shared_data: Arc<Mutex<MprisData>>,
    thread: Option<DbusThread>,
}

struct DbusThread {
    kill_signal: mpsc::Sender<()>,
    thread: JoinHandle<()>,
}

struct MprisData {
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
            duration: other.duration.map(|d| d.as_micros().try_into().unwrap())
        }
    }
}

impl MediaControls {
    pub fn new_with_name<S>(dbus_name: S, friendly_name: S) -> Self
    where
        S: ToString,
    {
        let shared_data = Arc::new(Mutex::new(MprisData {
            dbus_name: dbus_name.to_string(),
            friendly_name: friendly_name.to_string(),
            metadata: Default::default(),
            playback_status: MediaPlayback::Stopped,
        }));

        Self {
            shared_data,
            thread: None,
        }
    }

    pub fn attach<F>(&mut self, event_handler: F) -> Result<(), Error>
    where
        F: Fn(MediaControlEvent) + Send + 'static,
    {
        self.detach()?;

        let shared_data = self.shared_data.clone();
        let event_handler = Arc::new(Mutex::new(event_handler));
        let (tx, rx) = mpsc::channel();

        self.thread = Some(DbusThread {
            kill_signal: tx,
            thread: thread::spawn(move || {
                mpris_run(event_handler, shared_data, rx).unwrap();
            }),
        });
        Ok(())
    }

    pub fn detach(&mut self) -> Result<(), Error> {
        if let Some(DbusThread {
            kill_signal,
            thread,
        }) = self.thread.take()
        {
            kill_signal.send(()).unwrap();
            thread.join().unwrap();
        }
        Ok(())
    }

    pub fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Error> {
        let mut data = self.shared_data.lock().unwrap();
        data.playback_status = playback;
        Ok(())
    }

    pub fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Error> {
        if let Ok(mut data) = self.shared_data.lock() {
            data.metadata = metadata.into();
        }
        Ok(())
    }
}

// TODO: better errors
fn mpris_run(
    event_handler: Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    shared_data: Arc<Mutex<MprisData>>,
    kill_signal: mpsc::Receiver<()>,
) -> Result<(), DbusError> {
    let (dbus_name, friendly_name) = {
        let data = shared_data.lock().unwrap();
        (
            format!("org.mpris.MediaPlayer2.{}", data.dbus_name),
            data.friendly_name.clone(),
        )
    };

    let c = Connection::new_session()?;
    c.request_name(dbus_name, false, true, false)?;

    let mut cr = Crossroads::new();

    let media_player_2 = cr.register("org.mpris.MediaPlayer2", {
        let event_handler = event_handler.clone();

        move |b| {
            b.property("Identity")
                .get(move |_, _| Ok(friendly_name.clone()));

            register_method(b, &event_handler, "Raise", MediaControlEvent::Raise);
            register_method(b, &event_handler, "Quit", MediaControlEvent::Quit);

            // TODO: allow user to set these properties
            b.property("CanQuit").get(|_, _| Ok(true));
            b.property("CanRaise").get(|_, _| Ok(true));
            b.property("HasTracklist").get(|_, _| Ok(false));
            b.property("SupportedUriSchemes")
                .get(move |_, _| Ok(&[] as &[String]));
            b.property("SupportedMimeTypes")
                .get(move |_, _| Ok(&[] as &[String]));
        }
    });

    let player = cr.register("org.mpris.MediaPlayer2.Player", move |b| {
        use dbus::arg::{RefArg, Variant};

        // TODO: allow user to set these properties
        b.property("CanControl").get(|_, _| Ok(true));
        b.property("CanPlay").get(|_, _| Ok(true));
        b.property("CanPause").get(|_, _| Ok(true));
        b.property("CanGoNext").get(|_, _| Ok(true));
        b.property("CanGoPrevious").get(|_, _| Ok(true));
        b.property("CanSeek").get(|_, _| Ok(true));

        b.property("PlaybackStatus").get({
            let shared_data = shared_data.clone();
            move |_, _| {
                let data = shared_data.lock().unwrap();
                let status = match data.playback_status {
                    MediaPlayback::Playing { .. } => "Playing",
                    MediaPlayback::Paused { .. } => "Paused",
                    MediaPlayback::Stopped => "Stopped",
                };
                Ok(status.to_string())
            }
        });

        b.property("Position").get({
            let shared_data = shared_data.clone();
            move |_, _| {
                let data = shared_data.lock().unwrap();
                let progress: i64 = match data.playback_status {
                    MediaPlayback::Playing { progress: Some(progress) } |
                    MediaPlayback::Paused { progress: Some(progress) } => progress.0.as_micros(),
                    _ => 0,
                }.try_into().unwrap();
                Ok(progress)
            }
        });

        b.property("Metadata").get({
            let shared_data = shared_data.clone();

            move |_, _| {
                // TODO: this could be stored in a cache in `shared_data`.
                let mut dict = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

                let data = shared_data.lock().unwrap();
                let mut insert = |k: &str, v| dict.insert(k.to_string(), Variant(v));

                let OwnedMetadata {
                    ref title,
                    ref album,
                    ref artist,
                    ref cover_url,
                    ref duration,
                } = data.metadata;

                // TODO: For some reason the properties don't follow the order when
                // queried from the D-Bus. Probably because of the use of HashMap.
                // Can't use `dbus::arg::Dict` though, because it isn't Send.

                // MPRIS
                
                // TODO: this is just a workaround to enable SetPosition.
                insert("mpris:trackid", Box::new(DbusPath::new("/").unwrap()));

                if let Some(length) = duration {
                    insert("mpris:length", Box::new(*length));
                }

                if let Some(cover_url) = cover_url {
                    insert("mpris:artUrl", Box::new(cover_url.clone()));
                }

                // Xesam
                if let Some(title) = title {
                    insert("xesam:title", Box::new(title.clone()));
                }
                if let Some(artist) = artist {
                    insert("xesam:albumArtist", Box::new(artist.clone()));
                }
                if let Some(album) = album {
                    insert("xesam:album", Box::new(album.clone()));
                }

                Ok(dict)
            }
        });

        register_method(b, &event_handler, "Play", MediaControlEvent::Play);
        register_method(b, &event_handler, "Pause", MediaControlEvent::Pause);
        register_method(b, &event_handler, "PlayPause", MediaControlEvent::Toggle);
        register_method(b, &event_handler, "Next", MediaControlEvent::Next);
        register_method(b, &event_handler, "Previous", MediaControlEvent::Previous);
        register_method(b, &event_handler, "Stop", MediaControlEvent::Stop);

        b.method("Seek", ("Offset",), (), {
            let event_handler = event_handler.clone();

            move |_, _, (offset,): (i64,)| {
                let abs_offset = offset.abs() as u64;
                let direction = if offset > 0 {
                    SeekDirection::Forward
                } else {
                    SeekDirection::Backward
                };

                (event_handler.lock().unwrap())(MediaControlEvent::SeekBy(
                    direction,
                    Duration::from_micros(abs_offset),
                ));
                Ok(())
            }
        });

        b.method("SetPosition", ("TrackId", "Position"), (), {
            let shared_data = shared_data.clone();

            move |_, _, (_trackid, position): (DbusPath, i64)| {
                let data = shared_data.lock().unwrap();
                // According to the MPRIS specification:

                // 1.
                // If the TrackId argument is not the same as the current
                // trackid, the call is ignored as stale. So here we check that.
                // (Maybe it should be optional?)

                // TODO: the check. (We first need to store the TrackId somewhere)

                // 2.
                // If the Position argument is less than 0, do nothing.
                // If the Position argument is greater than the track length, do nothing.

                if position < 0 {
                    return Ok(());
                }

                if let Some(duration) = data.metadata.duration {
                    if position > duration {
                        return Ok(());
                    }
                }

                let position: u64 = position.try_into().unwrap();

                (event_handler.lock().unwrap())(MediaControlEvent::SetPosition(MediaPosition(
                    Duration::from_micros(position),
                )));
                Ok(())
            }
        });
    });

    cr.insert("/org/mpris/MediaPlayer2", &[media_player_2, player], ());

    c.start_receive(
        dbus::message::MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );

    // Start the server loop.
    loop {
        // If the kill signal was sent, then break the loop.
        if kill_signal.recv_timeout(Duration::from_millis(10)).is_ok() {
            break;
        }

        // Do the event processing.
        c.process(Duration::from_millis(1000))?;
    }
    Ok(())
}

fn register_method(
    b: &mut IfaceBuilder<()>,
    event_handler: &Arc<Mutex<dyn Fn(MediaControlEvent) + Send + 'static>>,
    name: &'static str,
    event: MediaControlEvent,
) {
    let event_handler = event_handler.clone();

    b.method(name, (), (), move |_, _, _: ()| {
        (event_handler.lock().unwrap())(event.clone());
        Ok(())
    });
}
