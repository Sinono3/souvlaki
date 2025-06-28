use souvlaki::{MediaControlEvent, OsMediaControls};

mod sample_data;

fn main() {
    // MPRIS platform
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let config = souvlaki::platform::mpris::MprisConfig {
        display_name: "Souvlaki Player".to_owned(),
        dbus_name: "souvlaki_player".to_owned(),
    };

    // macOS platform
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    let config = ();

    // Windows platform
    #[cfg(target_os = "windows")]
    let (config, hwnd, _dummy_window) = {
        let dummy_window = windows::DummyWindow::new().unwrap();
        let handle = Some(dummy_window.handle.0 as _);
        let config = souvlaki::platform::windows::WindowsConfig { hwnd: handle.hwnd };
        (config, handle, dummy_window)
    };

    // Dummy platform (for unsupported OSes)
    #[cfg(any(
        not(any(unix, target_os = "macos", target_os = "ios", target_os = "windows")),
        target_os = "android",
    ))]
    let config = ();

    let mut controls = OsMediaControls::new(config).unwrap();

    // The closure must be Send and have a static lifetime.
    controls
        .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
        .unwrap();

    // Set the cover art.
    controls.set_cover(Some(sample_data::cover())).unwrap();

    // Update the media metadata.
    controls
        .set_metadata(sample_data::album()[6].clone())
        .unwrap();

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
