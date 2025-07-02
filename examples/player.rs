use std::{sync::mpsc, thread::sleep, time::Duration};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

struct TestApp {
    songs: Vec<souvlaki::MediaMetadata>,
    song_index: usize,
    status: TestAppStatus,
}

/// Position in millis
enum TestAppStatus {
    Playing { position: u64 },
    Paused { position: u64 },
    Stopped,
}

impl TestAppStatus {
    pub fn play(&mut self) {
        *self = match *self {
            TestAppStatus::Playing { position } => TestAppStatus::Playing { position },
            TestAppStatus::Paused { position } => TestAppStatus::Playing { position },
            TestAppStatus::Stopped => TestAppStatus::Playing { position: 0 },
        };
    }
    pub fn pause(&mut self) {
        *self = match *self {
            TestAppStatus::Playing { position } => TestAppStatus::Paused { position },
            TestAppStatus::Paused { position } => TestAppStatus::Paused { position },
            TestAppStatus::Stopped => TestAppStatus::Paused { position: 0 },
        };
    }
    pub fn toggle(&mut self) {
        *self = match *self {
            TestAppStatus::Playing { position } => TestAppStatus::Paused { position },
            TestAppStatus::Paused { position } => TestAppStatus::Playing { position },
            TestAppStatus::Stopped => TestAppStatus::Stopped,
        };
    }
    pub fn go_to(&mut self, millis: u64) {
        *self = match *self {
            TestAppStatus::Playing { .. } => TestAppStatus::Playing { position: millis },
            TestAppStatus::Paused { .. } => TestAppStatus::Paused { position: millis },
            TestAppStatus::Stopped => TestAppStatus::Stopped,
        };
    }
    pub fn seek_fwd(&mut self, offset: u64) {
        *self = match *self {
            TestAppStatus::Playing { position } => TestAppStatus::Playing {
                position: position + offset,
            },
            TestAppStatus::Paused { position } => TestAppStatus::Paused {
                position: position + offset,
            },
            TestAppStatus::Stopped => TestAppStatus::Stopped,
        };
    }
    pub fn seek_bwd(&mut self, offset: u64) {
        *self = match *self {
            TestAppStatus::Playing { position } => TestAppStatus::Playing {
                position: position.saturating_sub(offset),
            },
            TestAppStatus::Paused { position } => TestAppStatus::Paused {
                position: position.saturating_sub(offset),
            },
            TestAppStatus::Stopped => TestAppStatus::Stopped,
        };
    }
    pub fn to_souvlaki(&self) -> souvlaki::MediaPlayback {
        match *self {
            TestAppStatus::Playing { position } => souvlaki::MediaPlayback::Playing {
                progress: Some(souvlaki::MediaPosition(Duration::from_millis(position))),
            },
            TestAppStatus::Paused { position } => souvlaki::MediaPlayback::Paused {
                progress: Some(souvlaki::MediaPosition(Duration::from_millis(position))),
            },
            TestAppStatus::Stopped => souvlaki::MediaPlayback::Stopped,
        }
    }
}

mod sample_data;

fn main() {
    let event_loop = EventLoop::new();
    #[allow(unused_variables)]
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(512, 512))
        .with_title("Souvlaki Player")
        .build(&event_loop)
        .unwrap();

    // MPRIS platform
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let config = souvlaki::platform::mpris::MprisConfig {
        display_name: "Souvlaki Player".to_owned(),
        dbus_name: "souvlaki_player".to_owned(),
    };

    // macOS/iOS platform
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let config = ();

    // Windows platform
    #[cfg(target_os = "windows")]
    let config = {
        use raw_window_handle::{HasRawWindowHandle, Win32WindowHandle};

        let handle: Win32WindowHandle = window.raw_window_handle();
        souvlaki::platform::windows::WindowsConfig { hwnd: handle.hwnd }
    };

    // Dummy platform (for unsupported OSes)
    #[cfg(any(
        not(any(unix, target_os = "macos", target_os = "ios", target_os = "windows")),
        target_os = "android",
    ))]
    let config = ();

    let mut app = TestApp {
        songs: sample_data::album().to_vec(),
        song_index: 0,
        status: TestAppStatus::Stopped,
    };
    let cover = sample_data::cover();
    // NOTE: Uncomment this if you want test out loading image covers from bytes.
    // Also check out the function if you want to see how it works under the hood.
    // let cover = sample_data::cover_bytes();

    let (tx, rx) = mpsc::sync_channel(32);
    let mut controls = souvlaki::OsMediaControls::new(config).unwrap();
    // Attach event handlers to our controls
    controls.attach(move |e| tx.send(e).unwrap()).unwrap();

    // Set playback status
    controls
        .set_playback(souvlaki::MediaPlayback::Playing { progress: None })
        .unwrap();

    // Set metadata
    controls
        .set_metadata(app.songs[app.song_index].clone())
        .unwrap();
    // Set cover image (the value differs depending on the OS)
    // (To see how these differences are handled in application code, please
    // see the implementation of [`sample_data::cover`].)
    controls.set_cover(cover.clone()).unwrap();
    // Set playback status.
    controls.set_playback(app.status.to_souvlaki()).unwrap();

    let mut t = 0u64;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                for event in rx.try_iter() {
                    use souvlaki::MediaControlEvent::*;
                    match event {
                        Toggle => {
                            app.status.toggle();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Play => {
                            app.status.play();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Pause => {
                            app.status.pause();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Next => {
                            app.song_index = app.song_index.saturating_add(1);
                            if app.song_index == app.songs.len() {
                                // The album has ended, restart it.
                                app.song_index = 0;
                                app.status = TestAppStatus::Stopped;
                            } else {
                                // The album has not ended yet
                                app.status.go_to(0);
                            }

                            controls
                                .set_metadata(app.songs[app.song_index].clone())
                                .unwrap();
                            controls.set_cover(cover.clone()).unwrap();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Previous => {
                            app.song_index = app.song_index.saturating_sub(1).min(app.songs.len());
                            app.status.go_to(0);

                            controls
                                .set_metadata(app.songs[app.song_index].clone())
                                .unwrap();
                            controls.set_cover(cover.clone()).unwrap();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Stop => {
                            app.status = TestAppStatus::Stopped;
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        SetPosition(position) => {
                            app.status.go_to(position.0.as_millis() as u64);
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                        Seek(direction) => match direction {
                            souvlaki::SeekDirection::Forward => {
                                app.status.seek_fwd(5000);
                                controls.set_playback(app.status.to_souvlaki()).unwrap();
                            }
                            souvlaki::SeekDirection::Backward => {
                                app.status.seek_bwd(5000);
                                controls.set_playback(app.status.to_souvlaki()).unwrap();
                            }
                        },
                        SeekBy(direction, offset) => match direction {
                            souvlaki::SeekDirection::Forward => {
                                app.status.seek_fwd(offset.as_millis() as u64);
                                controls.set_playback(app.status.to_souvlaki()).unwrap();
                            }
                            souvlaki::SeekDirection::Backward => {
                                app.status.seek_bwd(offset.as_millis() as u64);
                                controls.set_playback(app.status.to_souvlaki()).unwrap();
                            }
                        },
                        SetVolume(_) => todo!(),
                        SetRate(_) => todo!(),
                        SetShuffle(_) => todo!(),
                        SetRepeat(_) => todo!(),
                        OpenUri(_) => {
                            eprintln!("This example player does not support opening URIs");
                        }
                        Raise => {
                            window.request_user_attention(Some(
                                winit::window::UserAttentionType::Informational,
                            ));
                            window.focus_window();
                        }
                        Quit => {
                            eprintln!("Quitting...");
                            return;
                        }
                    }
                }

                let cur_song = &app.songs[app.song_index];
                let duration = cur_song.duration.unwrap();

                // Advance
                // (Not how one should actually do it in a music player. This *will* cause desyncs.)
                sleep(Duration::from_millis(1));
                match app.status {
                    TestAppStatus::Playing { position } => {
                        app.status = TestAppStatus::Playing {
                            position: (position + 1),
                        };

                        // Go to next song if it has finished
                        if position >= duration.as_millis() as u64 {
                            app.song_index = app.song_index.saturating_add(1);
                            if app.song_index == app.songs.len() {
                                // The album has ended, restart it.
                                app.song_index = 0;
                                app.status = TestAppStatus::Stopped;
                            } else {
                                // The album has not ended yet
                                app.status.go_to(0);
                            }

                            controls
                                .set_metadata(app.songs[app.song_index].clone())
                                .unwrap();
                            controls.set_cover(cover.clone()).unwrap();
                            controls.set_playback(app.status.to_souvlaki()).unwrap();
                        }
                    }
                    _ => (),
                }

                // Print every second
                t += 1;
                if (t % 1000) == 0 {
                    let track_number = cur_song.track_number.unwrap();
                    let artist = cur_song.artist.as_ref().unwrap();
                    let title = cur_song.title.as_ref().unwrap();

                    match app.status {
                        TestAppStatus::Playing { position } => eprintln!(
                            "Playing: {}. {} - {} ({:02}:{:02})",
                            track_number,
                            artist,
                            title,
                            (position / 1000) / 60,
                            (position / 1000) % 60,
                        ),
                        TestAppStatus::Paused { position } => eprintln!(
                            "Paused: {}. {} - {} ({:02}:{:02})",
                            track_number,
                            artist,
                            title,
                            (position / 1000) / 60,
                            (position / 1000) % 60,
                        ),
                        TestAppStatus::Stopped => {
                            eprintln!("Stopped: {}. {} - {}", track_number, artist, title,)
                        }
                    }
                }
            }
            _ => (),
        }
    });
}
