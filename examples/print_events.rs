use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};

fn main() {
    #[cfg(not(target_os = "windows"))]
    let hwnd = None;

    #[cfg(target_os = "windows")]
    let hwnd = {
        use raw_window_handle::windows::WindowsHandle;

        let handle: WindowsHandle = unimplemented!();
        Some(handle.hwnd)
    };

    let config = PlatformConfig {
        dbus_name: "my_player",
        display_name: "My Player",
        hwnd,
    };

    let mut controls = MediaControls::new(config).unwrap();

    // The closure must be Send and have a static lifetime.
    controls
        .attach(|event: MediaControlEvent| println!("Event received: {:?}", event))
        .unwrap();

    // Update the media metadata.
    controls
        .set_metadata(MediaMetadata {
            title: Some("Souvlaki Space Station"),
            artist: Some("Slowdive"),
            album: Some("Souvlaki"),
            ..Default::default()
        })
        .unwrap();

    // Your actual logic goes here.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    // The controls automatically detach on drop.
}
