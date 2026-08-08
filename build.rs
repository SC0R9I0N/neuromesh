fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "NeuroMesh");
        res.set("FileDescription", "NeuroMesh — local knowledge graph");
        res.set("OriginalFilename", "neuromesh.exe");
        if let Err(e) = res.compile() {
            // Missing rc.exe/windres shouldn't break `cargo check` on a bare machine.
            println!("cargo:warning=failed to embed Windows icon resource: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
