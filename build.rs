fn main() {
    slint_build::compile("ui/app.slint").unwrap();
    embed_windows_icon();
}

#[cfg(windows)]
fn embed_windows_icon() {
    if !std::path::Path::new("assets/AppIcon.ico").exists() {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/AppIcon.ico");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn embed_windows_icon() {}
