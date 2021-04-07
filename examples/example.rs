use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use souvlaki::{MediaControlEvent, MediaControls};
use souvlaki::{MediaMetadata, MediaPlayback};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TestApp {
    playing: bool,
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_os = "windows")]
    let mut controls = {
        let handle = match window.raw_window_handle() {
            RawWindowHandle::Windows(h) => h,
            _ => unreachable!(),
        };
        MediaControls::create_for_window(handle).unwrap();
    };
    #[cfg(target_os = "macos")]
    let mut controls = MediaControls::new();
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    let mut controls = MediaControls::new();

    let pending_events = Arc::new(Mutex::new(VecDeque::new()));
    let mut app = TestApp { playing: true };

    controls
        .attach({
            let pending_events = pending_events.clone();
            move |event| {
                pending_events.lock().unwrap().push_back(event);
            }
        })
        .unwrap();
    controls.set_playback(MediaPlayback::Playing).unwrap();
    controls
        .set_metadata(MediaMetadata {
            title: Some("When The Sun Hits"),
            album: Some("Souvlaki"),
            artist: Some("Slowdive"),
        })
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let mut change = false;

                if let Ok(mut events) = pending_events.try_lock() {
                    while let Some(event) = events.pop_front() {
                        match event {
                            MediaControlEvent::Toggle => app.playing = !app.playing,
                            MediaControlEvent::Play => app.playing = true,
                            MediaControlEvent::Pause => app.playing = false,
                            _ => {}
                        }
                        change = true;
                    }
                }

                if change {
                    controls
                        .set_playback(if app.playing {
                            MediaPlayback::Playing
                        } else {
                            MediaPlayback::Paused
                        })
                        .unwrap();
                    eprintln!(
                        "App is now: {}",
                        if app.playing { "playing" } else { "paused" }
                    );
                }
            }
            _ => (),
        }
    });
}
