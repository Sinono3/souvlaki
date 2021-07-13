use souvlaki::MediaControls;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    {
        #[cfg(target_os = "linux")]
        let mut controls = MediaControls::new_with_name("my_player", "My Player");
        #[cfg(target_os = "macos")]
        let mut controls = MediaControls::new();
        #[cfg(target_os = "windows")]
        let mut controls = {
            use raw_window_handle::windows::WindowsHandle;

            let handle: WindowsHandle = unimplemented!();
            MediaControls::for_window(handle).unwrap()
        };

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
