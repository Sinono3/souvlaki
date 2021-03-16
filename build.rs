#[cfg(target_os = "windows")]
fn build_winapi() {
    windows::build!(
        windows::media::{
            SystemMediaTransportControls,
            ISystemMediaTransportControls
        },
        windows::win32::windows_and_messaging::HWND,
        windows::win32::media_transport::ISystemMediaTransportControlsInterop,
    );
}

#[cfg(target_os = "macos")]
fn build_macos() {
    if std::env::var("TARGET").unwrap().contains("-apple") {
        println!("cargo:rustc-link-lib=framework=MediaPlayer");
    }
}

fn main() {
    #[cfg(target_os = "windows")]
    build_winapi();
    #[cfg(target_os = "macos")]
    build_macos();
}
