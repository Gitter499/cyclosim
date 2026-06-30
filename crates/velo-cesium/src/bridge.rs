#[cfg(feature = "cesium-native")]
#[cxx::bridge(namespace = "velo_cesium")]
mod ffi {
    extern "C++" {
        include!("velo-cesium/cpp/bridge.h");

        fn velo_cesium_native_version() -> &'static str;
        fn velo_cesium_native_build_ok() -> u32;
    }
}

#[cfg(feature = "cesium-native")]
pub fn native_build_ok() -> bool {
    ffi::velo_cesium_native_build_ok() != 0
}

#[cfg(feature = "cesium-native")]
pub fn native_version() -> &'static str {
    ffi::velo_cesium_native_version()
}
