use std::{sync::mpsc, thread::sleep, time::Duration};

use souvlaki::{MediaControlEvent, MediaPlayback, OsMediaControls};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TestApp {
    playing: bool,
    song_index: u8,
}

mod sample_data;

fn main() {
    let event_loop = EventLoop::new();
    #[allow(unused_variables)]
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // MPRIS platform
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let config = souvlaki::platform::mpris::MprisConfig {
        display_name: "My Player".to_owned(),
        dbus_name: "my_player".to_owned(),
    };

    // macOS platform
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let config = ();

    // Windows platform
    #[cfg(target_os = "windows")]
    let config = {
        use raw_window_handle::Win32WindowHandle;

        let handle: Win32WindowHandle = unimplemented!();
        souvlaki::platform::windows::WindowsConfig { hwnd: handle.hwnd }
    };

    // Dummy platform (for unsupported OSes)
    #[cfg(any(
        not(any(unix, target_os = "macos", target_os = "ios", target_os = "windows")),
        target_os = "android",
    ))]
    let config = ();

    let mut controls = OsMediaControls::new(config).unwrap();

    let (tx, rx) = mpsc::sync_channel(32);
    let mut app = TestApp {
        playing: true,
        song_index: 0,
    };

    controls.attach(move |e| tx.send(e).unwrap()).unwrap();
    controls
        .set_playback(MediaPlayback::Playing { progress: None })
        .unwrap();
    controls.set_metadata(sample_data::metadata()).unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let mut change = false;

                for event in rx.try_iter() {
                    match event {
                        MediaControlEvent::Toggle => app.playing = !app.playing,
                        MediaControlEvent::Play => app.playing = true,
                        MediaControlEvent::Pause => app.playing = false,
                        MediaControlEvent::Next => app.song_index = app.song_index.wrapping_add(1),
                        MediaControlEvent::Previous => {
                            app.song_index = app.song_index.wrapping_sub(1)
                        }
                        MediaControlEvent::Stop => app.playing = false,
                        _ => (),
                    }
                    change = true;
                }
                sleep(Duration::from_millis(1));

                if change {
                    controls
                        .set_playback(if app.playing {
                            MediaPlayback::Playing { progress: None }
                        } else {
                            MediaPlayback::Paused { progress: None }
                        })
                        .unwrap();

                    eprintln!(
                        "{} (song {})",
                        if app.playing { "Playing" } else { "Paused" },
                        app.song_index
                    );
                }
            }
            _ => (),
        }
    });
}
