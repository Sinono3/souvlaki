use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata};

fn main() {
    #[cfg(target_os = "linux")]
    let mut controls = MediaControls::new_with_name("my_player", "My Player");
    #[cfg(target_os = "macos")]
    let mut controls = MediaControls::new();
    #[cfg(target_os = "windows")]
    let mut controls = {
        use raw_window_handle::windows::WindowsHandle;

        // No window creation in this example for the sake of simplicity
        let handle: WindowsHandle = unimplemented!();
        MediaControls::for_window(handle).unwrap()
    };

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
