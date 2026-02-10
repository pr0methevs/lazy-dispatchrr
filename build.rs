fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set("ProductName", "Lazy-Dispatchrr");
        res.set("FileDescription", "A TUI app for dispatching GitHub Workflows");
        res.set("LegalCopyright", "Copyright © 2026 Artur Kaminski");
        res.set("CompanyName", "homelab-core");
        res.set(
            "FileVersion",
            env!("CARGO_PKG_VERSION"),
        );
        res.set(
            "ProductVersion",
            env!("CARGO_PKG_VERSION"),
        );
        // Uncomment and set the path to an .ico file to embed an icon:
        // res.set_icon("assets/app.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
