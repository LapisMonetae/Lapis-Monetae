fn main() {
    // Embed .ico as Windows executable icon
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app.ico");
        res.set("ProductName", "Lapis Monetae Wallet");
        res.set("FileDescription", "Lapis Monetae Wallet GUI");
        res.set("ProductVersion", "1.0.1");
        res.compile().expect("Failed to compile Windows resources");
    }
}
