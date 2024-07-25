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
        // make sure path is created and we have a stub file to defer errors
        std::fs::create_dir_all(&path).unwrap();
        std::fs::File::create(std::path::Path::new(&path).join("harness_code.wasm")).unwrap();
    }
}
