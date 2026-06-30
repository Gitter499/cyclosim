fn main() {
    println!("cargo:rerun-if-env-changed=CESIUM_NATIVE_DIR");
    println!("cargo:rerun-if-changed=cpp/bridge.h");
    println!("cargo:rerun-if-changed=cpp/bridge.cpp");

    if std::env::var("CARGO_FEATURE_CESIUM_NATIVE").is_ok() {
        if let Ok(cesium_dir) = std::env::var("CESIUM_NATIVE_DIR") {
            println!("cargo:rustc-env=CESIUM_NATIVE_DIR={cesium_dir}");
        } else {
            eprintln!(
                "warning: cesium-native feature enabled but CESIUM_NATIVE_DIR is unset; \
                 building stub bridge only"
            );
        }
        cxx_build::bridge("src/bridge.rs")
            .file("cpp/bridge.cpp")
            .std("c++17")
            .compile("velo-cesium-bridge");
    }
}
