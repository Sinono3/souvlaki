use souvlaki::OsMediaControls;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    {
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

        controls.attach(|_| println!("Received message")).unwrap();
        println!("Attached");

        for i in 0..5 {
            println!("Main thread sleeping:  {}/4", i);
            sleep(Duration::from_secs(1));
        }
    }
    println!("Dropped and detached");
    sleep(Duration::from_secs(2));
}
