[![Crates.io](https://img.shields.io/crates/v/souvlaki.svg)](https://crates.io/crates/souvlaki)
[![Docs](https://docs.rs/souvlaki/badge.svg)](https://docs.rs/souvlaki)
[![CI](https://github.com/Sinono3/souvlaki/actions/workflows/build.yml/badge.svg)](https://github.com/Sinono3/souvlaki/actions/workflows/build.yml)

<sub>DISCLAIMER: the project is still in an early state. All parts may be subject to change.</sub>

# Souvlaki

A cross-platform solution to OS media controls and metadata. One abstraction for Linux, MacOS and Windows.

## Supported platforms

- Linux (via MPRIS)
- MacOS
- Windows

## Windows

- Update metadata:\
![image](https://user-images.githubusercontent.com/8389938/106080661-4a515e80-60f6-11eb-81e0-81ab0eda5188.png)
- Play and pause polling.\
![play_pause](https://user-images.githubusercontent.com/8389938/106080917-bdf36b80-60f6-11eb-98b5-f3071ae3eab6.gif)

## MacOS

Screenshots coming soon.

## Linux

Coming soon.

## Example

The main struct is `MediaControls`. Each platform has to initialize it in a different way:

- MacOS: `MediaControls::new()`. No arguments needed.
- Linux: `MediaControls::new_with_name(dbus_name, fancy_name)`. `dbus_name` should follow [the specifications](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-names-bus). The fancy name could be however you want. It represents what could be shown to the users.
- Windows: `MediaControls::for_window(hwnd: WindowsHandle)`. Unfortunately in this case, a window needs to be opened to allow media controls. The argument required is a `WindowsHandle` found in the `raw-window-handle` crate.

So, an example full cross-platform app would look like this:

```rust
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
```
