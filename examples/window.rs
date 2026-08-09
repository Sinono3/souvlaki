use std::{sync::mpsc, thread::sleep, time::Duration};

use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

#[derive(Default)]
struct TestApp {
    playing: bool,
    song_index: u8,
    window: Option<Window>,
    controls: Option<MediaControls>,
    rx: Option<mpsc::Receiver<MediaControlEvent>>,
}

impl ApplicationHandler for TestApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasWindowHandle, RawWindowHandle};

            let handle = match window.window_handle().unwrap().as_raw() {
                RawWindowHandle::Win32(h) => h,
                _ => unreachable!(),
            };
            Some(handle.hwnd.get() as *mut std::ffi::c_void)
        };

        let config = PlatformConfig {
            dbus_name: "my_player",
            display_name: "My Player",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

        let (tx, rx) = mpsc::sync_channel(32);
        controls.attach(move |e| tx.send(e).unwrap()).unwrap();
        controls
            .set_playback(MediaPlayback::Playing { progress: None })
            .unwrap();
        controls
            .set_metadata(MediaMetadata {
                title: Some("When The Sun Hits"),
                album: Some("Souvlaki"),
                artist: Some("Slowdive"),
                duration: Some(Duration::from_secs_f64(4.0 * 60.0 + 50.0)),
                cover_url: Some("https://c.pxhere.com/photos/34/c1/souvlaki_authentic_greek_greek_food_mezes-497780.jpg!d"),
            })
            .unwrap();

        self.playing = true;
        self.window = Some(window);
        self.controls = Some(controls);
        self.rx = Some(rx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        if event == WindowEvent::CloseRequested {
            event_loop.exit();
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let (Some(rx), Some(controls)) = (self.rx.as_ref(), self.controls.as_mut()) else {
            return;
        };

        let mut change = false;

        for event in rx.try_iter() {
            match event {
                MediaControlEvent::Toggle => self.playing = !self.playing,
                MediaControlEvent::Play => self.playing = true,
                MediaControlEvent::Pause => self.playing = false,
                MediaControlEvent::Next => self.song_index = self.song_index.wrapping_add(1),
                MediaControlEvent::Previous => self.song_index = self.song_index.wrapping_sub(1),
                MediaControlEvent::Stop => self.playing = false,
                _ => (),
            }
            change = true;
        }
        sleep(Duration::from_millis(50));

        if change {
            controls
                .set_playback(if self.playing {
                    MediaPlayback::Playing { progress: None }
                } else {
                    MediaPlayback::Paused { progress: None }
                })
                .unwrap();

            eprintln!(
                "{} (song {})",
                if self.playing { "Playing" } else { "Paused" },
                self.song_index
            );
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut TestApp::default()).unwrap();
}
