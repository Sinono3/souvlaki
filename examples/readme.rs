fn main() {
    use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata};

    #[cfg(target_os = "linux")]
    let mut controls = MediaControls::new_with_name("my-player", "My Player");
    #[cfg(target_os = "macos")]
    let mut controls = MediaControls::new();
    #[cfg(target_os = "windows")]
    let mut controls = {
        use raw_window_handle::windows::WindowsHandle;

        let handle: WindowsHandle = unimplemented!();
        MediaControls::for_window(handle).unwrap()
    };

    let mut playing = false;
    let mut number = 100i32;

    // The closure must be Send and have a static lifetime.
    controls
        .attach(move |event| {
            match event {
                MediaControlEvent::Play => playing = true,
                MediaControlEvent::Pause => playing = false,
                MediaControlEvent::Toggle => playing = !playing,
                MediaControlEvent::Next => number += 1,
                MediaControlEvent::Previous => number -= 1,
                _ => (),
            }
            println!("playing: {}", playing);
            println!("number: {}", number);
        })
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
