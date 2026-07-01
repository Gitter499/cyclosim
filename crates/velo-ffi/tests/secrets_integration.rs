//! FFI round-trip for runtime secrets → tile/bikegen status.

use velo_ffi::{RuntimeSecretsDto, VeloHandle};

#[test]
fn configure_runtime_secrets_updates_status() {
    let handle = VeloHandle::with_dirs_for_tests(
        std::env::temp_dir().join("velo-secrets-packs"),
        std::env::temp_dir().join("velo-secrets-bikes"),
    );

    assert!(handle.tiles_provider_status().contains("dev tileset"));

    handle.configure_runtime_secrets(RuntimeSecretsDto {
        google_map_tiles_api_key: Some("test-google-key".into()),
        cesium_ion_access_token: None,
        meshy_api_key: None,
        prefer_hosted_bike_generation: false,
    });

    assert!(handle.tiles_provider_status().contains("Google"));

    handle.configure_runtime_secrets(RuntimeSecretsDto {
        google_map_tiles_api_key: None,
        cesium_ion_access_token: None,
        meshy_api_key: None,
        prefer_hosted_bike_generation: true,
    });

    assert!(handle.bikegen_mode_status().contains("Meshy API key"));
}
