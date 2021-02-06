#![cfg(target_os = "windows")]
use souvlaki::{MediaControls, MediaControlEvent};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
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

    let handle = match window.raw_window_handle() {
        RawWindowHandle::Windows(h) => h,
        _ => panic!("Not Windows"),
    };

    let mut app = TestApp { playing: false };
    let mut controls = souvlaki::windows::WindowsMediaControls::create(handle).unwrap();

    controls.set_metadata(
        souvlaki::MediaMetadata {
            title: "When The Sun Hits",
            album: "Souvlaki",
            artist: "Slowdive",
        }
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
            Event::MainEventsCleared => {
                let mut change = false;

                controls.poll(|event| {
                    match event {
                        MediaControlEvent::Play => app.playing = true,
                        MediaControlEvent::Pause => app.playing = false,
                        _ => unimplemented!(),
                    }
                    change = true;
                });

                if change {
                    controls.set_playback(app.playing);
                    eprintln!("App is now: {}", if app.playing {
                        "playing"
                    } else {
                        "paused"
                    });
                }
            }
            _ => (),
        }
    });
}