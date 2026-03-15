fn main() {
    // Embed the application icon into the Windows PE binary so that Explorer,
    // taskbar, and .lnk shortcuts display it without a separate icon file.
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
