fn main() {
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
            panic!("Unsupported OS. Please open an issue on the repo to add support for this OS")
        }
    };

    if !std::path::Path::new(&path).exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
}
