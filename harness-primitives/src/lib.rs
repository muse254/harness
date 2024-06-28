//pub mod device;
pub mod error;
pub mod harness_os;
pub mod http;
pub mod internals;
pub mod program;

pub use harness_os::HarnessOs;

// todo: abstract the `HARNESS_PATH` and builds gen dir to be more configurable

/// Way easier to have a static path in our system that holds all files
/// that we need to run the harness system instead of having to pass env variable for this.
#[cfg(target_os = "windows")]
pub const HARNESS_PATH: &str =
    const_format::concatcp!(std::env!("USERPROFILE"), "\\AppData\\Local\\harness");

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const HARNESS_PATH: &str = const_format::concatcp!(std::env!("HOME"), "/.config/harness");

pub fn ensure_path_created<'a>() -> error::Result<&'a str> {
    if !cfg!(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )) {
        return Err(error::Error::internal::<anyhow::Error>(
            "Unsupported OS. Please open an issue on the repo to add support for this OS",
            None,
        ));
    }

    if !std::path::Path::new(HARNESS_PATH).exists() {
        std::fs::create_dir_all(HARNESS_PATH).unwrap();
    }

    Ok(HARNESS_PATH)
}
