use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use dbus::{
    arg::{RefArg, Variant},
    Path,
};
use dbus_crossroads::{Context, Crossroads};

use crate::{MediaControlEvent, MediaPosition, Repeat, SeekDirection};

use super::super::{MprisConfig, ServiceState};

// TODO: This type is super messed up, but it's the only way to get seeking working properly
// on graphical media controls using dbus-crossroads.
pub type SeekedSignal = Box<dyn Fn(&Path<'_>, &(i64,)) -> dbus::Message + Send + Sync>;

pub(super) fn register_methods<F>(
    state: &Arc<Mutex<ServiceState>>,
    event_handler: &Arc<Mutex<F>>,
    config: MprisConfig,
    seeked_signal_tx: mpsc::Sender<SeekedSignal>,
) -> Crossroads
where
    F: Fn(MediaControlEvent) + Send + 'static,
{
    macro_rules! method_const {
        ($b:ident, $name:expr, $event:expr) => {
            let event_handler = event_handler.clone();
            $b.method($name, (), (), move |_, _, _: ()| {
                (event_handler.lock().unwrap())($event);
                Ok(())
            });
        };
    }

    macro_rules! method {
        ($b:ident, $name:expr, $args:expr, $out:expr, $handle:expr) => {
            let event_handler = event_handler.clone();
            $b.method($name, $args, $out, move |ctx, _, value| {
                let event = ($handle)(ctx, value);
                if let Some(event) = event {
                    (event_handler.lock().unwrap())(event);
                }
                Ok(())
            });
        };
    }

    macro_rules! prop_const {
        // Return a constant value
        ($b:ident, $name:expr, $value:expr) => {
            let value = $value.to_owned();
            $b.property($name)
                .get({ move |_, _| Ok(value.clone()) })
                .emits_changed_true();
        };
    }
    macro_rules! prop {
        // Get (retrieve from state)
        ($b:ident, $name:expr, $get:expr) => {
            $b.property($name)
                .get({
                    let state = state.clone();
                    move |_, _| {
                        let state: &ServiceState = &*state.lock().unwrap();
                        Ok(($get)(state))
                    }
                })
                .emits_changed_true();
        };
        // Get (retrieve from state) and set (send media control event)
        ($b:ident, $name:expr, $get:expr, $set:expr) => {
            $b.property($name)
                .get({
                    let state = state.clone();
                    move |_, _| {
                        let state: &ServiceState = &*state.lock().unwrap();
                        Ok(($get)(state))
                    }
                })
                .set({
                    let event_handler = event_handler.clone();
                    move |_, _, value| {
                        let event = $set(value);
                        if let Some(event) = event {
                            (event_handler.lock().unwrap())(event);
                        }
                        Ok(None)
                    }
                })
                .emits_changed_true();
        };
    }

    let mut cr = Crossroads::new();
    let app_interface = cr.register("org.mpris.MediaPlayer2", {
        move |b| {
            method_const!(b, "Raise", MediaControlEvent::Raise);
            method_const!(b, "Quit", MediaControlEvent::Quit);

            prop!(b, "CanQuit", |state: &ServiceState| state
                .permissions
                .can_quit);
            prop!(
                b,
                "Fullscreen",
                |state: &ServiceState| state.fullscreen,
                |fullscreen: bool| Some(MediaControlEvent::SetFullscreen(fullscreen))
            );
            prop!(b, "CanSetFullscreen", |state: &ServiceState| state
                .permissions
                .can_set_fullscreen);
            prop!(b, "CanRaise", |state: &ServiceState| state
                .permissions
                .can_raise);
            prop!(b, "HasTrackList", |_state: &ServiceState| false);
            prop_const!(b, "Identity", config.identity);
            prop_const!(b, "DesktopEntry", config.desktop_entry);
            prop!(b, "SupportedUriSchemes", |state: &ServiceState| state
                .permissions
                .supported_uri_schemes
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>());
            prop!(b, "SupportedMimeTypes", |state: &ServiceState| state
                .permissions
                .supported_mime_types
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>());
        }
    });

    let player_interface = cr.register("org.mpris.MediaPlayer2.Player", |b| {
        method_const!(b, "Next", MediaControlEvent::Next);
        method_const!(b, "Previous", MediaControlEvent::Previous);
        method_const!(b, "Pause", MediaControlEvent::Pause);
        method_const!(b, "PlayPause", MediaControlEvent::Toggle);
        method_const!(b, "Stop", MediaControlEvent::Stop);
        method_const!(b, "Play", MediaControlEvent::Play);

        method!(
            b,
            "Seek",
            ("Offset",),
            (),
            move |_ctx: &mut Context, (offset,): (i64,)| {
                let abs_offset = offset.unsigned_abs();
                let direction = if offset > 0 {
                    SeekDirection::Forward
                } else {
                    SeekDirection::Backward
                };
                Some(MediaControlEvent::SeekBy(
                    direction,
                    Duration::from_micros(abs_offset),
                ))
            }
        );
        method!(b, "SetPosition", ("TrackId", "Position"), (), {
            move |_ctx: &mut Context, (_trackid, position): (Path, i64)| {
                // if the `Position` argument is less than 0, do nothing.
                if let Ok(position) = u64::try_from(position) {
                    let position = Duration::from_micros(position);

                    Some(MediaControlEvent::SetPosition(MediaPosition(position)))
                } else {
                    None
                }
            }
        });
        method!(b, "OpenUri", ("Uri",), (), {
            move |_ctx: &mut Context, (uri,): (String,)| Some(MediaControlEvent::OpenUri(uri))
        });

        // need to send this signature to our caller... clunky but it's what can be done
        seeked_signal_tx
            .send(b.signal::<(i64,), _>("Seeked", ("x",)).msg_fn())
            .unwrap();

        prop!(b, "PlaybackStatus", |state: &ServiceState| state
            .playback_status
            .to_dbus_value()
            .to_owned());
        prop!(
            b,
            "LoopStatus",
            |state: &ServiceState| state.loop_status.to_dbus_value().to_owned(),
            |loop_status_dbus: String| {
                // If invalid, just ignore it
                Repeat::from_dbus_value(&loop_status_dbus)
                    .map(|repeat| MediaControlEvent::SetRepeat(repeat))
            }
        );
        prop!(b, "Rate", |state: &ServiceState| state.rate, |rate: f64| {
            Some(MediaControlEvent::SetRate(rate))
        });
        prop!(
            b,
            "Shuffle",
            |state: &ServiceState| state.shuffle,
            |shuffle: bool| { Some(MediaControlEvent::SetShuffle(shuffle)) }
        );
        prop!(b, "Metadata", |state: &ServiceState| {
            state
                .metadata_dict
                .iter()
                .map(|(k, v)| (k.to_owned(), Variant(v.box_clone())))
                .collect::<HashMap<_, _>>()
        });
        prop!(
            b,
            "Volume",
            |state: &ServiceState| state.volume,
            |volume: f64| { Some(MediaControlEvent::SetVolume(volume)) }
        );
        prop!(b, "Position", |state: &ServiceState| state
            .playback_status
            .to_micros());
        prop!(b, "MinimumRate", |state: &ServiceState| state
            .permissions
            .min_rate);
        prop!(b, "MaximumRate", |state: &ServiceState| state
            .permissions
            .max_rate);
        prop!(b, "CanGoNext", |state: &ServiceState| state
            .permissions
            .can_go_next);
        prop!(b, "CanGoPrevious", |state: &ServiceState| state
            .permissions
            .can_go_previous);
        prop!(b, "CanPlay", |state: &ServiceState| state
            .permissions
            .can_play);
        prop!(b, "CanPause", |state: &ServiceState| state
            .permissions
            .can_pause);
        prop!(b, "CanSeek", |state: &ServiceState| state
            .permissions
            .can_seek);
        prop!(b, "CanControl", |state: &ServiceState| state
            .permissions
            .can_control);
    });

    cr.insert(
        "/org/mpris/MediaPlayer2",
        &[app_interface, player_interface],
        (),
    );
    cr
}
