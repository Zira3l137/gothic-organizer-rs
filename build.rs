fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("resources/icon.ico");
    if let Err(e) = res.compile() {
        println!("cargo:warning=Failed to compile resource file: {e}");
    };
}
