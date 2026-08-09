
#[cfg(not(target_os = "windows"))]
fn main() {
    use souvlaki::{MediaControls, PlatformConfig};
    use std::thread::sleep;
    use std::time::Duration;

    {
        let hwnd = None;

        let config = PlatformConfig {
            dbus_name: "my_player",
            display_name: "My Player",
            hwnd,
        };

        let mut controls = MediaControls::new(config).unwrap();

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

#[cfg(target_os = "windows")]
fn main() {
    println!("This example is not implemented for Windows");
    unimplemented!()
}
