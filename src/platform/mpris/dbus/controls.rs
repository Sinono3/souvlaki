use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use dbus::channel::{MatchingReceiver, Sender};
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::message::SignalArgs;
use dbus::Path;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::super::MprisError;
use crate::extensions::MprisPropertiesExt;
use crate::{
    LoopStatus, MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig,
};

/// A handle to OS media controls.
pub struct Dbus {
    thread: Option<ServiceThreadHandle>,
    dbus_name: String,
    friendly_name: String,
}

struct ServiceThreadHandle {
    event_channel: mpsc::Sender<InternalEvent>,
    thread: JoinHandle<Result<(), MprisError>>,
}

#[derive(Clone, Debug)]
enum InternalEvent {
    SetMetadata(MediaMetadata),
    SetPlayback(MediaPlayback),
    SetLoopStatus(LoopStatus),
    SetRate(f64),
    SetShuffle(bool),
    SetVolume(f64),
    SetMaximumRate(f64),
    SetMinimumRate(f64),
    Kill,
}

#[derive(Debug)]
pub struct ServiceState {
    pub playback_status: MediaPlayback,
    pub loop_status: LoopStatus,
    pub rate: f64,
    pub shuffle: bool,
    pub metadata: MediaMetadata,
    pub metadata_dict: HashMap<String, Variant<Box<dyn RefArg>>>,
    pub volume: f64,
    pub maximum_rate: f64,
    pub minimum_rate: f64,
}

// TODO: This can be refactored to use macros
pub fn create_metadata_dict(metadata: &MediaMetadata) -> HashMap<String, Variant<Box<dyn RefArg>>> {
    let mut dict = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

    let mut insert = |k: &str, v| dict.insert(k.to_string(), Variant(v));

    let &MediaMetadata {
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
    } = metadata;

    // MPRIS
    // TODO: Workaround to enable SetPosition.
    insert("mpris:trackid", Box::new(Path::new("/").unwrap()));

    if let Some(length) = duration {
        insert(
            "mpris:length",
            Box::new(i64::try_from(length.as_micros()).unwrap()),
        );
    }

    // TODO: set cover URL
    // if let Some(cover_url) = cover_url {
    //     insert("mpris:artUrl", Box::new(cover_url.clone()));
    // }

    // Xesam
    if let Some(title) = title {
        insert("xesam:title", Box::new(title.clone()));
    }
    if let Some(artists) = artists {
        insert("xesam:artist", Box::new(artists.clone()));
    }
    if let Some(album_title) = album_title {
        insert("xesam:album", Box::new(album_title.clone()));
    }
    if let Some(album_artists) = album_artists {
        insert("xesam:albumArtist", Box::new(album_artists.clone()));
    }
    if let Some(genres) = genres {
        insert("xesam:genre", Box::new(genres.clone()));
    }
    if let Some(track_number) = track_number {
        insert("xesam:trackNumber", Box::new(track_number));
    }
    if let Some(disc_number) = disc_number {
        insert("xesam:discNumber", Box::new(disc_number));
    }
    if let Some(composers) = composers {
        insert("xesam:composer", Box::new(composers.clone()));
    }
    if let Some(lyricists) = lyricists {
        insert("xesam:lyricist", Box::new(lyricists.clone()));
    }
    if let Some(lyrics) = lyrics {
        insert("xesam:asText", Box::new(lyrics.clone()));
    }
    if let Some(comments) = comments {
        insert("xesam:comment", Box::new(comments.clone()));
    }
    if let Some(beats_per_minute) = beats_per_minute {
        insert("xesam:audioBPM", Box::new(beats_per_minute));
    }
    if let Some(user_rating_01) = user_rating_01 {
        insert("xesam:userRating", Box::new(user_rating_01));
    }
    if let Some(auto_rating) = auto_rating {
        insert("xesam:autoRating", Box::new(auto_rating));
    }
    if let Some(play_count) = play_count {
        insert("xesam:playCount", Box::new(play_count));
    }
    if let Some(media_url) = media_url {
        insert("xesam:url", Box::new(media_url.clone()));
    }

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

impl MediaControls for Dbus {
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
        let (event_channel, rx) = mpsc::channel();

        // Check if the connection can be created BEFORE spawning the new thread
        let conn = Connection::new_session()?;
        let name = format!("org.mpris.MediaPlayer2.{}", dbus_name);
        conn.request_name(name, false, true, false)?;

        self.thread = Some(ServiceThreadHandle {
            event_channel,
            thread: thread::spawn(move || run_service(conn, friendly_name, event_handler, rx)),
        });
        Ok(())
    }

    fn detach(&mut self) -> Result<(), Self::Error> {
        if let Some(ServiceThreadHandle {
            event_channel,
            thread,
        }) = self.thread.take()
        {
            // We don't care about the result of this event, since we immedieately
            // check if the thread has panicked on the next line.
            event_channel.send(InternalEvent::Kill).ok();
            // One error in case the thread panics, and the other one in case the
            // thread has returned an error.
            thread.join().map_err(|_| MprisError::ThreadPanicked)??;
        }
        Ok(())
    }

    fn set_playback(&mut self, playback: MediaPlayback) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetPlayback(playback))
    }

    fn set_metadata(&mut self, metadata: MediaMetadata) -> Result<(), Self::Error> {
        self.send_internal_event(InternalEvent::SetMetadata(metadata))
    }
}

impl Dbus {
    fn send_internal_event(&mut self, event: InternalEvent) -> Result<(), MprisError> {
        let thread = &self.thread.as_ref().ok_or(MprisError::ThreadNotRunning)?;
        thread
            .event_channel
            .send(event)
            .map_err(|_| MprisError::ThreadPanicked)
    }
}

impl MprisPropertiesExt for Dbus {
    fn set_loop_status(&mut self, loop_status: LoopStatus) -> Result<(), Self::Error> {
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

fn run_service<F>(
    conn: Connection,
    friendly_name: String,
    event_handler: F,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> Result<(), MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    let state = Arc::new(Mutex::new(ServiceState {
        playback_status: MediaPlayback::Stopped,
        loop_status: LoopStatus::None,
        rate: 1.0,
        shuffle: false,
        metadata: Default::default(),
        metadata_dict: create_metadata_dict(&Default::default()),
        volume: 1.0,
        maximum_rate: 1.0,
        minimum_rate: 1.0,
    }));
    let event_handler = Arc::new(Mutex::new(event_handler));
    let seeked_signal = Arc::new(Mutex::new(None));

    let mut cr =
        super::interfaces::register_methods(&state, &event_handler, friendly_name, seeked_signal);

    conn.start_receive(
        dbus::message::MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );

    loop {
        if let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let mut changed_properties = HashMap::new();

            match event {
                InternalEvent::SetMetadata(metadata) => {
                    let mut state = state.lock().unwrap();
                    state.metadata_dict = create_metadata_dict(&metadata);
                    state.metadata = metadata;
                    changed_properties.insert(
                        "Metadata".to_owned(),
                        Variant(state.metadata_dict.box_clone()),
                    );
                }
                InternalEvent::SetPlayback(playback) => {
                    let mut state = state.lock().unwrap();
                    state.playback_status = playback;
                    changed_properties.insert(
                        "PlaybackStatus".to_owned(),
                        Variant(Box::new(state.playback_status.to_dbus_value().to_string())),
                    );
                }
                InternalEvent::SetLoopStatus(loop_status) => {
                    let mut state = state.lock().unwrap();
                    state.loop_status = loop_status;
                    changed_properties.insert(
                        "LoopStatus".to_owned(),
                        Variant(Box::new(loop_status.to_dbus_value().to_owned())),
                    );
                }
                InternalEvent::SetRate(rate) => {
                    let mut state = state.lock().unwrap();
                    state.rate = rate;
                    changed_properties.insert("Rate".to_owned(), Variant(Box::new(rate)));
                }
                InternalEvent::SetShuffle(shuffle) => {
                    let mut state = state.lock().unwrap();
                    state.shuffle = shuffle;
                    changed_properties.insert("Shuffle".to_owned(), Variant(Box::new(shuffle)));
                }
                InternalEvent::SetVolume(volume) => {
                    let mut state = state.lock().unwrap();
                    state.volume = volume;
                    changed_properties.insert("Volume".to_owned(), Variant(Box::new(volume)));
                }
                InternalEvent::SetMaximumRate(rate) => {
                    let mut state = state.lock().unwrap();
                    state.maximum_rate = rate;
                    changed_properties.insert("MaximumRate".to_owned(), Variant(Box::new(rate)));
                }
                InternalEvent::SetMinimumRate(rate) => {
                    let mut state = state.lock().unwrap();
                    state.minimum_rate = rate;
                    changed_properties.insert("MinimumRate".to_owned(), Variant(Box::new(rate)));
                }
                InternalEvent::Kill => break,
            }

            let properties_changed = PropertiesPropertiesChanged {
                interface_name: "org.mpris.MediaPlayer2.Player".to_owned(),
                changed_properties,
                invalidated_properties: Vec::new(),
            };

            conn.send(
                properties_changed.to_emit_message(&Path::new("/org/mpris/MediaPlayer2").unwrap()),
            )
            .ok();
        }
        conn.process(Duration::from_millis(1000))?;
    }

    Ok(())
}
