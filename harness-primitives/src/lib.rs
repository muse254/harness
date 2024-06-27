//pub mod device;
pub mod error;
pub mod harness_os;
pub mod http;
pub mod internals;
pub mod program;

pub use harness_os::{HarnessOs, ProgramId};

/// Way easier to have a static path in our system that holds all files
/// that we need to run the harness system instead of having to pass env variable for this.
pub fn get_harness_path() -> error::Result<String> {
    let path = {
        if cfg!(unix) {
            // `~/.config/MyApp` for unix systems
            format!("{}/.config/harness", std::env::var("HOME").unwrap())
        } else if cfg!(windows) {
            // `%USERPROFILE%\AppData\Local\MyApp\` for windows
            format!(
                "{}\\AppData\\Local\\harness",
                std::env::var("USERPROFILE").unwrap()
            )
        } else {
            return Err(error::Error::internal::<anyhow::Error>(
                "Unsupported OS. Please open an issue on the repo to add support for this OS",
                None,
            ));
        }
    };

    if !std::path::Path::new(&path).exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    Ok(path)
}
