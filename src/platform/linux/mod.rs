#![cfg(target_os = "linux")]

use crate::{MediaControlEvent, MediaMetadata, MediaPlayback};
use dbus::blocking::Connection;
use dbus::channel::MatchingReceiver;
use dbus::Error as DbusError;
use dbus_crossroads::{Crossroads, IfaceBuilder};
use std::collections::HashMap;
use std::convert::From;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct OwnedMetadata {
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    pub cover_url: Option<String>,
}

impl From<MediaMetadata<'_>> for OwnedMetadata {
    fn from(other: MediaMetadata) -> Self {
        OwnedMetadata {
            title: other.title.map(|s| s.to_string()),
            artist: other.artist.map(|s| s.to_string()),
            album: other.album.map(|s| s.to_string()),
            cover_url: other.cover_url.map(|s| s.to_string()),
        }
    }
}

impl MediaControls {
    pub fn new_with_name<S>(dbus_name: S, friendly_name: S) -> Self 
    where 
        S: ToString
    {
        let shared_data = Arc::new(Mutex::new(MprisData {
            dbus_name: dbus_name.to_string(),
            friendly_name: friendly_name.to_string(),
            metadata: Default::default(),
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

    pub fn set_playback(&mut self, _playback: MediaPlayback) -> Result<(), Error> {
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

    let media_player_2 = cr.register("org.mpris.MediaPlayer2", move |b| {
        b.property("Identity")
            .get(move |_, _| Ok(friendly_name.clone()));

        // TODO: Everything in here is placeholder
        b.method("Raise", (), (), move |_, _, _: ()| Ok(()));
        b.method("Quit", (), (), move |_, _, _: ()| Ok(()));
        b.property("CanQuit").get(|_, _| Ok(false));
        b.property("CanRaise").get(|_, _| Ok(false));
        b.property("HasTracklist").get(|_, _| Ok(false));
        b.property("SupportedUriSchemes")
            .get(move |_, _| Ok(&[] as &[String]));
        b.property("SupportedMimeTypes")
            .get(move |_, _| Ok(&[] as &[String]));
    });

    let player = cr.register("org.mpris.MediaPlayer2.Player", move |b| {
        use dbus::arg::{RefArg, Variant};

        // TODO: allow user to set these properties
        b.property("CanControl").get(|_, _| Ok(true));
        b.property("CanPlay").get(|_, _| Ok(true));
        b.property("CanPause").get(|_, _| Ok(true));
        b.property("CanGoNext").get(|_, _| Ok(true));
        b.property("CanGoPrevious").get(|_, _| Ok(true));

        // TODO: placeholder, seek unimplemented
        b.property("CanSeek").get(|_, _| Ok(false));

        b.property("Metadata").get(move |_, _| {
            // TODO: this could be stored in a cache in `shared_data`.
            let mut dict = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

            if let Ok(data) = shared_data.lock() {
                let mut insert = |k: &str, v| dict.insert(k.to_string(), Variant(v));

                let OwnedMetadata {
                    ref title,
                    ref album,
                    ref artist,
                    ref cover_url,
                } = data.metadata;

                // TODO: For some reason the properties don't follow an order.
                // Probably because of the use of HashMap. Can't use `dbus::arg::Dict`
                // though, because it isn't Send.

                if let Some(title) = title {
                    insert("xesam:title", Box::new(title.clone()));
                }
                if let Some(artist) = artist {
                    insert("xesam:albumArtist", Box::new(artist.clone()));
                }
                if let Some(album) = album {
                    insert("xesam:album", Box::new(album.clone()));
                }
                if let Some(cover_url) = cover_url {
                    insert("mpris:artUrl", Box::new(cover_url.clone()));
                }
            }

            Ok(dict)
        });

        register_method(b, &event_handler, "Play", MediaControlEvent::Play);
        register_method(b, &event_handler, "Pause", MediaControlEvent::Pause);
        register_method(b, &event_handler, "PlayPause", MediaControlEvent::Toggle);
        register_method(b, &event_handler, "Next", MediaControlEvent::Next);
        register_method(b, &event_handler, "Previous", MediaControlEvent::Previous);
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
        (event_handler.lock().unwrap())(event);
        Ok(())
    });
}
