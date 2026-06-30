#pragma once

#include <cstdint>

// Stub bridge header — full Cesium Native integration implements IAssetAccessor /
// ITaskProcessor / IPrepareRendererResources on the Rust side (see README).

inline const char* velo_cesium_native_version() {
    return "0.44.0";
}

inline std::uint32_t velo_cesium_native_build_ok() {
    return 1;
}
