fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icon.ico");
    if let Err(e) = res.compile() {
        println!("cargo:warning=Failed to compile resource file: {e}");
    };

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let target_dir = std::path::Path::new(&out_dir).ancestors().nth(3).unwrap();
    let source_path = std::env::current_dir().unwrap().join("resources");

    let link_path = target_dir.join("resources");
    if link_path.exists() || link_path.is_symlink() {
        if let Err(e) = std::fs::remove_file(&link_path) {
            eprintln!("Warning: Failed to remove existing symlink: {e}");
        }
    }

    #[cfg(unix)]
    {
        if let Err(e) = std::os::unix::fs::symlink(source_path, &link_path) {
            eprintln!("Failed to create symbolic link: {}", e);
            std::process::exit(1);
        }
    }

    #[cfg(windows)]
    {
        if let Err(e) = std::os::windows::fs::symlink_dir(&source_path, &link_path) {
            eprintln!("Failed to create directory symbolic link: {e}");
            eprintln!("Source: {}, Target: {}", source_path.display(), link_path.display());
            std::process::exit(1);
        }
    }

    println!("Created symbolic link: {} -> {}", link_path.display(), source_path.display());
}
