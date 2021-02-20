pub use self::platform::*;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;

#[cfg(all(
    not(target_os = "windows")
))]
compile_error!("The platform you're compiling for is not supported by souvlaki");