//pub mod device;
pub mod error;
pub mod harness_os;
pub mod http;
pub mod internals;
pub mod program;
pub mod result;

#[cfg(feature = "wasm-ext")]
pub use harness_os::HarnessOs;
pub use result::HarnessResult;

/// Way easier to have a static path in our system that holds all files
/// that we need to run the harness system instead of having to pass env variable for this.
#[cfg(target_os = "windows")]
pub const HARNESS_PATH: &str = const_format::concatcp!(
    std::env!("USERPROFILE"),
    "\\AppData\\Local\\harness\\harness_code.wasm"
);

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub const HARNESS_PATH: &str =
    const_format::concatcp!(std::env!("HOME"), "/.config/harness/harness_code.wasm");
