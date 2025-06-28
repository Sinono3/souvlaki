use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;
use dbus::channel::{MatchingReceiver, Sender};
use dbus::ffidisp::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::message::SignalArgs;
use dbus::Path;

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::super::{
    create_metadata_dict, InternalEvent, MprisCover, MprisError, ServiceState, ServiceThreadHandle,
};
use crate::MediaControlEvent;

pub(in super::super) fn spawn_thread<F>(
    event_handler: F,
    dbus_name: String,
    friendly_name: String,
    event_channel: mpsc::Sender<InternalEvent>,
    rx: mpsc::Receiver<InternalEvent>,
) -> Result<ServiceThreadHandle, MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    // Check if the connection can be created BEFORE spawning the new thread
    let conn = Connection::new_session()?;
    let name = format!("org.mpris.MediaPlayer2.{}", dbus_name);
    conn.request_name(name, false, true, false)?;

    Ok(ServiceThreadHandle {
        event_channel,
        thread: thread::spawn(move || run_service(conn, friendly_name, event_handler, rx)),
    })
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
    let state = Arc::new(Mutex::new(ServiceState::default()));
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
        while let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let mut changed_properties = HashMap::new();

            match event {
                InternalEvent::SetMetadata(metadata) => {
                    let mut state = state.lock().unwrap();
                    state.metadata_dict = create_metadata_dict(&metadata, &state.cover_url);
                    state.metadata = metadata;
                    changed_properties.insert(
                        "Metadata".to_owned(),
                        Variant(state.metadata_dict.box_clone()),
                    );
                }
                InternalEvent::SetCover(cover) => {
                    let cover_url = if let Some(MprisCover::Url(cover_url)) = cover {
                        Some(cover_url)
                    } else {
                        None
                    };
                    let mut state = state.lock().unwrap();
                    state.metadata_dict = create_metadata_dict(&state.metadata, &cover_url);
                    state.cover_url = cover_url;
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
                InternalEvent::Kill => return Ok(()),
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
        // NOTE: Arbitrary timeout duration...
        conn.process(Duration::from_millis(10))?;
    }
}
