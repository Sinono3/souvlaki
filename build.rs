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

fn main() {
    #[cfg(target_os = "windows")]
    build_winapi();
}