use std::convert::TryFrom;
use std::sync::{mpsc, Arc, Mutex, RwLock};
use std::time::Duration;
use std::{mem, thread};

use zbus::connection;
use zbus::zvariant::ObjectPath;

use super::super::{
    create_metadata_dict, InternalEvent, MprisConfig, MprisCover, MprisError, ServiceState,
    ServiceThreadHandle,
};
use super::{AppInterface, PlayerInterface};
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
    Ok(ServiceThreadHandle {
        event_channel,
        thread: thread::spawn(move || {
            let result = pollster::block_on(run_service(config, event_handler, rx));
            if let Err(ref error) = result {
                // TODO: Do this correctly.
                println!("MPRIS service thread errored: {error}");
            }
            result
        }),
    })
}

async fn run_service<F>(
    config: MprisConfig,
    event_handler: F,
    event_channel: mpsc::Receiver<InternalEvent>,
) -> Result<(), MprisError>
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    let event_handler = Arc::new(Mutex::new(event_handler));
    let state = Arc::new(RwLock::new(ServiceState::default()));

    let app = AppInterface {
        config: config.clone(),
        state: state.clone(),
        event_handler: event_handler.clone(),
    };

    let player = PlayerInterface {
        state: state.clone(),
        event_handler,
    };

    let name = format!("org.mpris.MediaPlayer2.{}", &config.dbus_name);
    let path = ObjectPath::try_from("/org/mpris/MediaPlayer2").unwrap();
    let connection = connection::Builder::session()?
        .serve_at(&path, app)?
        .serve_at(&path, player)?
        .name(name.as_str())?
        .build()
        .await?;

    loop {
        while let Ok(event) = event_channel.recv_timeout(Duration::from_millis(10)) {
            let player_ref = connection
                .object_server()
                .interface::<_, PlayerInterface>(&path)
                .await?;
            let player = player_ref.get_mut().await;
            let p_emit = player_ref.signal_emitter();
            let app_ref = connection
                .object_server()
                .interface::<_, AppInterface>(&path)
                .await?;
            let app = app_ref.get_mut().await;
            let a_emit = app_ref.signal_emitter();

            match event {
                InternalEvent::SetPermissions(new_permissions) => {
                    let old_permissions;
                    // Scoped to avoid deadlock
                    {
                        let mut state = state.write().unwrap();
                        old_permissions =
                            mem::replace(&mut state.permissions, new_permissions.clone());
                    }

                    // Check this one-by-one
                    if old_permissions.can_quit != new_permissions.can_quit {
                        app.can_quit_changed(a_emit).await?;
                    }

                    if old_permissions.can_set_fullscreen != new_permissions.can_set_fullscreen {
                        app.can_set_fullscreen_changed(a_emit).await?;
                    }
                    if old_permissions.can_raise != new_permissions.can_raise {
                        app.can_raise_changed(a_emit).await?;
                    }
                    if old_permissions.supported_uri_schemes
                        != new_permissions.supported_uri_schemes
                    {
                        app.supported_uri_schemes_changed(a_emit).await?;
                    }
                    if old_permissions.supported_mime_types != new_permissions.supported_mime_types
                    {
                        app.supported_mime_types_changed(a_emit).await?;
                    }
                    if old_permissions.can_go_next != new_permissions.can_go_next {
                        player.can_go_next_changed(p_emit).await?;
                    }
                    if old_permissions.can_go_previous != new_permissions.can_go_previous {
                        player.can_go_previous_changed(p_emit).await?;
                    }
                    if old_permissions.can_play != new_permissions.can_play {
                        player.can_play_changed(p_emit).await?;
                    }
                    if old_permissions.can_pause != new_permissions.can_pause {
                        player.can_pause_changed(p_emit).await?;
                    }
                    if old_permissions.can_seek != new_permissions.can_seek {
                        player.can_seek_changed(p_emit).await?;
                    }
                    if old_permissions.can_control != new_permissions.can_control {
                        player.can_control_changed(p_emit).await?;
                    }
                    if old_permissions.max_rate != new_permissions.max_rate {
                        player.maximum_rate_changed(p_emit).await?;
                    }
                    if old_permissions.min_rate != new_permissions.min_rate {
                        player.minimum_rate_changed(p_emit).await?;
                    }
                }
                InternalEvent::SetMetadata(metadata) => {
                    {
                        let mut state = state.write().unwrap();
                        state.metadata_dict = create_metadata_dict(&metadata, &state.cover_url);
                        state.metadata = metadata;
                    }
                    player.metadata_changed(p_emit).await?;
                }
                InternalEvent::SetCover(cover) => {
                    {
                        let cover_url = MprisCover::to_url(cover);
                        let mut state = state.write().unwrap();
                        state.metadata_dict = create_metadata_dict(&state.metadata, &cover_url);
                        state.cover_url = cover_url;
                    }
                    player.metadata_changed(p_emit).await?;
                }
                InternalEvent::SetPlayback(playback) => {
                    {
                        let mut state = state.write().unwrap();
                        state.playback_status = playback;
                        player.playback_status_changed(p_emit).await?;
                    }
                    player.seeked(p_emit).await?;
                }
                InternalEvent::SetLoopStatus(loop_status) => {
                    {
                        let mut state = state.write().unwrap();
                        state.loop_status = loop_status;
                    }
                    player.loop_status_changed(p_emit).await?;
                }
                InternalEvent::SetRate(rate) => {
                    {
                        let mut state = state.write().unwrap();
                        state.rate = rate;
                    }
                    player.rate_changed(p_emit).await?;
                }
                InternalEvent::SetShuffle(shuffle) => {
                    {
                        let mut state = state.write().unwrap();
                        state.shuffle = shuffle;
                    }
                    player.shuffle_changed(p_emit).await?;
                }
                InternalEvent::SetVolume(volume) => {
                    {
                        let mut state = state.write().unwrap();
                        state.volume = volume;
                    }
                    player.volume_changed(p_emit).await?;
                }
                InternalEvent::SetFullscreen(fullscreen) => {
                    {
                        let mut state = state.write().unwrap();
                        state.fullscreen = fullscreen;
                    }
                    app.fullscreen_changed(a_emit).await?;
                }
                InternalEvent::Kill => return Ok(()),
            }
        }
    }
}
