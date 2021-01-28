use midyakis::MediaControls;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
struct TestApp {
    playing: bool,
}

impl midyakis::MediaPlayer for TestApp {
    fn play(&mut self) {
        self.playing = true;
    }
    fn pause(&mut self) {
        self.playing = false;
    }
    fn playing(&self) -> bool {
        self.playing
    }
    fn metadata(&self) -> midyakis::MediaMetadata {
        midyakis::MediaMetadata {
            title: "When The Sun Hits".to_string(),
            album: "Souvlaki".to_string(),
            artist: "Slowdive".to_string(),
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let handle = match window.raw_window_handle() {
        RawWindowHandle::Windows(h) => h,
        _ => panic!("Not Windows"),
    };

    let mut test_app = TestApp { playing: false };
    let mut controls = midyakis::windows::WindowsMediaControls::new(&test_app, handle).unwrap();

    event_loop.run(move |event, _, control_flow| {
        // In this example, poll is needed.
        // N
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
            Event::MainEventsCleared => {
                controls.poll(&mut test_app);
            }
            _ => (),
        }
    });
}