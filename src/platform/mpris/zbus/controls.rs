use std::convert::TryFrom;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use zbus::{ConnectionBuilder, SignalContext};
use zvariant::ObjectPath;

use super::super::{
    create_metadata_dict, InternalEvent, MprisCover, MprisError, ServiceState, ServiceThreadHandle,
};
use super::{AppInterface, PlayerInterface};
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
    Ok(ServiceThreadHandle {
        event_channel,
        thread: thread::spawn(move || {
            pollster::block_on(run_service(dbus_name, friendly_name, event_handler, rx))
                .map_err(|e| e.into())
        }),
    })
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
