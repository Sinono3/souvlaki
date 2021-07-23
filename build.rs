#[cfg(target_os = "windows")]
fn build_winapi() {
    windows::build!(
        Windows::Foundation::{TypedEventHandler, EventRegistrationToken, Uri},
        Windows::Media::{
            SystemMediaTransportControls,
            SystemMediaTransportControlsDisplayUpdater,
            SystemMediaTransportControlsButton,
            SystemMediaTransportControlsButtonPressedEventArgs,
            SystemMediaTransportControlsTimelineProperties,
            ISystemMediaTransportControls,
            MediaPlaybackType,
            MediaPlaybackStatus,
            MusicDisplayProperties,
            PlaybackPositionChangeRequestedEventArgs,
        },
        Windows::Win32::Foundation::HWND,
        Windows::Win32::System::WinRT::ISystemMediaTransportControlsInterop,
        Windows::Storage::Streams::RandomAccessStreamReference,
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
