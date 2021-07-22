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

The main struct is `MediaControls`. In order to create this struct you need a `PlatformConfig`. This struct contains all of the platform-specific requirements for spawning media controls. Here are the differences between the platforms:

- MacOS: No config needed.
- Linux: 
	- `dbus_name`: The way your player will appear on D-Bus. It should follow [the D-Bus specification](https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-names-bus). 
	- `display_name`: This could be however you want. It's the name that will be shown to the users.
- Windows: 
	- `hwnd`: In this platform, a window needs to be opened to create media controls. The argument required is an `HWND`, a value of type `*mut c_void`. This value can be extracted when you open a window in your program, for example using the `raw_window_handle` in winit.

A full cross-platform app would look like this:

```rust
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

    let mut controls = MediaControls::new(config);

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

[Check out this example here.](https://github.com/Sinono3/souvlaki/blob/master/examples/print_events.rs)
