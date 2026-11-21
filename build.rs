//! Embeds the Windows icon and version resources.

fn main() {
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=packaging/windows/spotifrost.ico");
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("packaging/windows/spotifrost.ico")
            .set("ProductName", "Spotifrost")
            // Task Manager and the Windows mixer name the process by its description.
            .set("FileDescription", "Spotifrost");
        if let Err(error) = resource.compile() {
            println!("cargo:warning=Windows resources not embedded: {error}");
        }
    }
}
