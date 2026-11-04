fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .cpp(true)
            .file("src/safe_call.mm")
            .flag("-fobjc-exceptions")
            .flag("-fexceptions")
            .compile("vv_plugin_safe_call");
        println!("cargo:rerun-if-changed=src/safe_call.mm");
    }

    #[cfg(target_os = "linux")]
    {
        cc::Build::new()
            .cpp(true)
            .file("src/safe_call.cpp")
            .compile("vv_plugin_safe_call");
        println!("cargo:rerun-if-changed=src/safe_call.cpp");
    }
}
