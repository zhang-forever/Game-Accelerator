fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // On Windows, embed the application manifest so the exe requests administrator
    // privileges via UAC on launch. Most optimizations (registry, services,
    // process priority) require elevation, so this removes the need to manually
    // "Run as administrator". The manifest also declares Windows 10/11 support.
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/app.rc");
        println!("cargo:rerun-if-changed=assets/app.manifest");
        embed_resource::compile("assets/app.rc", embed_resource::NONE);
    }
}
