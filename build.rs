#[cfg(any(target_os = "macos", target_os = "ios"))]
fn build_apple() {
    if std::env::var("TARGET").unwrap().contains("-apple") {
        println!("cargo:rustc-link-lib=framework=MediaPlayer");
    }
}
use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Platforms
        platform_mpris: { all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android"))) },
        platform_mpris_dbus: { all(platform_mpris, feature = "use_dbus") },
        platform_mpris_zbus: { all(platform_mpris, feature = "use_zbus") },
        platform_macos: { target_os = "macos" },
        platform_ios: { target_os = "ios" },
        platform_apple: { any(platform_macos, platform_ios) },
        platform_windows: { target_os = "windows" },
        platform_dummy: { not(any(platform_linux, platform_macos, platform_windows)) },
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    build_apple();
}
