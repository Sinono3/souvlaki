#[cfg(target_os = "macos")]
fn build_macos() {
    if std::env::var("TARGET").unwrap().contains("-apple") {
        println!("cargo:rustc-link-lib=framework=MediaPlayer");
    }
}

fn main() {
    #[cfg(target_os = "macos")]
    build_macos();
}
