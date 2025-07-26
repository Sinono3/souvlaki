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
    create_metadata_dict, InternalEvent, MprisConfig, MprisCover, MprisError, ServiceState,
    ServiceThreadHandle,
};
use crate::MediaControlEvent;

pub(in super::super) fn spawn_thread<F>(
    event_handler: F,
    config: MprisConfig,
    event_channel: mpsc::Sender<InternalEvent>,
    rx: mpsc::Receiver<InternalEvent>,
) -> Result<ServiceThreadHandle, MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    // Check if the connection can be created BEFORE spawning the new thread
    let conn = Connection::new_session()?;
    let name = format!("org.mpris.MediaPlayer2.{}", config.dbus_name);
    conn.request_name(name, false, true, false)?;

    Ok(ServiceThreadHandle {
        event_channel,
        thread: thread::spawn(move || run_service(conn, config, event_handler, rx)),
    })
}

fn run_service<F>(
    conn: Connection,
    config: MprisConfig,
    event_handler: F,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> Result<(), MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    let state = Arc::new(Mutex::new(ServiceState::default()));
    let event_handler = Arc::new(Mutex::new(event_handler));
    let (seeked_signal_tx, seeked_signal_rx) = mpsc::channel();
    let mut cr =
        super::interfaces::register_methods(&state, &event_handler, config, seeked_signal_tx);
    let media_player2_path = Path::new("/org/mpris/MediaPlayer2").unwrap();

    conn.start_receive(
        dbus::message::MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );
    let seeked_signal = seeked_signal_rx.recv().unwrap();

    loop {
        while let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let mut player_properties_changed = HashMap::<String, Variant<Box<dyn RefArg>>>::new();
            let mut app_properties_changed = HashMap::<String, Variant<Box<dyn RefArg>>>::new();

            match event {
                InternalEvent::SetPermissions(permissions) => {
                    let mut state = state.lock().unwrap();
                    // Check this one-by-one
                    if state.permissions.can_quit != permissions.can_quit {
                        app_properties_changed.insert(
                            "CanQuit".to_owned(),
                            Variant(Box::new(permissions.can_quit)),
                        );
                    }
                    if state.permissions.can_set_fullscreen != permissions.can_set_fullscreen {
                        app_properties_changed.insert(
                            "CanSetFullscreen".to_owned(),
                            Variant(Box::new(permissions.can_set_fullscreen)),
                        );
                    }
                    if state.permissions.can_raise != permissions.can_raise {
                        app_properties_changed.insert(
                            "CanRaise".to_owned(),
                            Variant(Box::new(permissions.can_raise)),
                        );
                    }
                    if state.permissions.supported_uri_schemes != permissions.supported_uri_schemes
                    {
                        let owned: Vec<_> = permissions
                            .supported_uri_schemes
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                        app_properties_changed
                            .insert("SupportedUriSchemes".to_owned(), Variant(Box::new(owned)));
                    }
                    if state.permissions.supported_mime_types != permissions.supported_mime_types {
                        let owned: Vec<_> = permissions
                            .supported_mime_types
                            .iter()
                            .map(|s| s.to_string())
                            .collect();
                        app_properties_changed
                            .insert("SupportedMimeTypes".to_owned(), Variant(Box::new(owned)));
                    }
                    if state.permissions.can_go_next != permissions.can_go_next {
                        player_properties_changed.insert(
                            "CanGoNext".to_owned(),
                            Variant(Box::new(permissions.can_go_next)),
                        );
                    }
                    if state.permissions.can_go_previous != permissions.can_go_previous {
                        player_properties_changed.insert(
                            "CanGoPrevious".to_owned(),
                            Variant(Box::new(permissions.can_go_previous)),
                        );
                    }
                    if state.permissions.can_play != permissions.can_play {
                        player_properties_changed.insert(
                            "CanPlay".to_owned(),
                            Variant(Box::new(permissions.can_play)),
                        );
                    }
                    if state.permissions.can_pause != permissions.can_pause {
                        player_properties_changed.insert(
                            "CanPause".to_owned(),
                            Variant(Box::new(permissions.can_pause)),
                        );
                    }
                    if state.permissions.can_seek != permissions.can_seek {
                        player_properties_changed.insert(
                            "CanSeek".to_owned(),
                            Variant(Box::new(permissions.can_seek)),
                        );
                    }
                    if state.permissions.can_control != permissions.can_control {
                        player_properties_changed.insert(
                            "CanControl".to_owned(),
                            Variant(Box::new(permissions.can_control)),
                        );
                    }
                    if state.permissions.max_rate != permissions.max_rate {
                        player_properties_changed.insert(
                            "MaximumRate".to_owned(),
                            Variant(Box::new(permissions.max_rate)),
                        );
                    }
                    if state.permissions.min_rate != permissions.min_rate {
                        player_properties_changed.insert(
                            "MinimumRate".to_owned(),
                            Variant(Box::new(permissions.min_rate)),
                        );
                    }

                    state.permissions = permissions; // TODO: Check each manually
                }
                InternalEvent::SetMetadata(metadata) => {
                    let mut state = state.lock().unwrap();
                    state.metadata_dict = create_metadata_dict(&metadata, &state.cover_url);
                    state.metadata = metadata;
                    player_properties_changed.insert(
                        "Metadata".to_owned(),
                        Variant(state.metadata_dict.box_clone()),
                    );
                }
                InternalEvent::SetCover(cover) => {
                    let cover_url = MprisCover::to_url(cover);
                    let mut state = state.lock().unwrap();
                    state.metadata_dict = create_metadata_dict(&state.metadata, &cover_url);
                    state.cover_url = cover_url;
                    player_properties_changed.insert(
                        "Metadata".to_owned(),
                        Variant(state.metadata_dict.box_clone()),
                    );
                }
                InternalEvent::SetPlayback(playback) => {
                    let mut state = state.lock().unwrap();
                    state.playback_status = playback;

                    // Emit seeked signal
                    let micros = state.playback_status.to_micros();
                    conn.send(seeked_signal(&media_player2_path, &(micros,)))
                        .ok();
                }
                InternalEvent::SetLoopStatus(loop_status) => {
                    let mut state = state.lock().unwrap();
                    state.loop_status = loop_status;
                    player_properties_changed.insert(
                        "LoopStatus".to_owned(),
                        Variant(Box::new(loop_status.to_dbus_value().to_owned())),
                    );
                }
                InternalEvent::SetRate(rate) => {
                    let mut state = state.lock().unwrap();
                    state.rate = rate;
                    player_properties_changed.insert("Rate".to_owned(), Variant(Box::new(rate)));
                }
                InternalEvent::SetShuffle(shuffle) => {
                    let mut state = state.lock().unwrap();
                    state.shuffle = shuffle;
                    player_properties_changed
                        .insert("Shuffle".to_owned(), Variant(Box::new(shuffle)));
                }
                InternalEvent::SetVolume(volume) => {
                    let mut state = state.lock().unwrap();
                    state.volume = volume;
                    player_properties_changed
                        .insert("Volume".to_owned(), Variant(Box::new(volume)));
                }
                InternalEvent::SetFullscreen(fullscreen) => {
                    let mut state = state.lock().unwrap();
                    state.fullscreen = fullscreen;
                    player_properties_changed
                        .insert("Fullscreen".to_owned(), Variant(Box::new(fullscreen)));
                }
                InternalEvent::Kill => return Ok(()),
            }

            if app_properties_changed.len() > 0 {
                let app_properties_changed = PropertiesPropertiesChanged {
                    interface_name: "org.mpris.MediaPlayer2".to_owned(),
                    changed_properties: app_properties_changed,
                    invalidated_properties: Vec::new(),
                };
                conn.send(app_properties_changed.to_emit_message(&media_player2_path))
                    .ok();
            }

            if player_properties_changed.len() > 0 {
                let player_properties_changed = PropertiesPropertiesChanged {
                    interface_name: "org.mpris.MediaPlayer2.Player".to_owned(),
                    changed_properties: player_properties_changed,
                    invalidated_properties: Vec::new(),
                };
                conn.send(player_properties_changed.to_emit_message(&media_player2_path))
                    .ok();
            }
        }

        // NOTE: Arbitrary timeout duration...
        conn.process(Duration::from_millis(10))?;
    }
}
