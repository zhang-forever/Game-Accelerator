fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Use normal user privileges for monitoring. The UI requests elevation on
    // demand; per-user settings do not need a UAC prompt at startup.
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/app.rc");
        println!("cargo:rerun-if-changed=assets/app.manifest");
        println!("cargo:rerun-if-changed=assets/app.ico");
        embed_resource::compile("assets/app.rc", embed_resource::NONE)
            .manifest_required()
            .expect("Failed to embed the Windows application manifest");
    }
}
