use std::time::Duration;

use souvlaki::{
    MediaControlEvent, MediaMetadata, MediaTypeMacos, MediaTypeWindows, OsMediaControls,
    PlatformConfig,
};

fn main() {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let (hwnd, _dummy_window) = {
        let dummy_window = windows::DummyWindow::new().unwrap();
        let handle = Some(dummy_window.handle.0 as _);
        (handle, dummy_window)
    };

    let config = PlatformConfig {
        dbus_name: "my_player",
        display_name: "My Player",
        hwnd,
    };

    let mut controls = OsMediaControls::new(config).unwrap();

    // The closure must be Send and have a static lifetime.
    controls
        .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
        .unwrap();

    // Update the media metadata.
    controls
        .set_metadata(MediaMetadata {
            title: Some("Souvlaki Space Station".to_owned()),
            artist: Some("Slowdive".to_owned()),
            artists: Some(vec!["Slowdive".to_owned()]),
            album_title: Some("Souvlaki".to_owned()),
            album_artist: Some("Slowdive".to_owned()),
            album_artists: Some(vec!["Slowdive".to_owned()]),
            genre: Some("Shoegaze".to_owned()),
            genres: Some(vec!["Shoegaze".to_owned()]),
            track_number: Some(6),
            album_track_count: Some(10),
            disc_number: Some(1),
            disc_count: Some(1),
            duration: Some(Duration::from_micros(359_131_429)),
            lyricists: Some(vec!["Halstead".to_owned()]),
            user_rating_01: Some(0.8),
            user_rating_05: Some(4),
            auto_rating: Some(0.7),
            play_count: Some(108),
            skip_count: Some(23),
            media_url: Some("https://www.discogs.com/master/9478-Slowdive-Souvlaki".to_owned()),
            beats_per_minute: Some(138),
            media_type_macos: Some(MediaTypeMacos {
                music: true,
                ..Default::default()
            }),
            media_type_windows: Some(MediaTypeWindows::Music),
            ..Default::default()
        })
        .unwrap();

    // Your actual logic goes here.
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1));

        // this must be run repeatedly by your program to ensure
        // the Windows event queue is processed by your application
        #[cfg(target_os = "windows")]
        windows::pump_event_queue();
    }

    // The controls automatically detach on drop.
}

// demonstrates how to make a minimal window to allow use of media keys on the command line
#[cfg(target_os = "windows")]
mod windows {
    use std::io::Error;
    use std::mem;

    use windows::core::PCWSTR;
    use windows::w;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor,
        IsDialogMessageW, PeekMessageW, RegisterClassExW, TranslateMessage, GA_ROOT, MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
    };

    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| (format!("Getting module handle failed: {e}")))?;

                let wnd_class = WNDCLASSEXW {
                    cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                    hInstance: instance,
                    lpszClassName: PCWSTR::from(class_name),
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };

                if RegisterClassExW(&wnd_class) == 0 {
                    return Err(format!(
                        "Registering class failed: {}",
                        Error::last_os_error()
                    ));
                }

                let handle = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!(""),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    instance,
                    None,
                );

                if handle.0 == 0 {
                    Err(format!(
                        "Message only window creation failed: {}",
                        Error::last_os_error()
                    ))
                } else {
                    Ok(handle)
                }
            };

            handle_result.map(|handle| DummyWindow { handle })
        }
        extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    impl Drop for DummyWindow {
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.handle);
            }
        }
    }

    pub fn pump_event_queue() -> bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            while msg.message != WM_QUIT && has_message {
                if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            }

            msg.message == WM_QUIT
        }
    }
}
