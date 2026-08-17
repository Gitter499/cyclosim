//! Runtime tile provider credentials (set by shell via FFI; env vars as fallback).

use std::sync::{Mutex, OnceLock};

static TILES_CREDENTIALS: OnceLock<Mutex<TilesCredentials>> = OnceLock::new();

fn store() -> &'static Mutex<TilesCredentials> {
    TILES_CREDENTIALS.get_or_init(|| Mutex::new(TilesCredentials::default()))
}

/// API keys/tokens for 3D Tiles providers. Secrets are never logged or persisted to disk by this crate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TilesCredentials {
    pub google_map_tiles_api_key: Option<String>,
    pub cesium_ion_access_token: Option<String>,
}

impl TilesCredentials {
    pub fn google_key(&self) -> Option<String> {
        self.google_map_tiles_api_key
            .clone()
            .filter(|k| !k.trim().is_empty())
            .or_else(|| {
                std::env::var("GOOGLE_MAP_TILES_API_KEY")
                    .ok()
                    .filter(|k| !k.trim().is_empty())
            })
    }

    pub fn cesium_ion_token(&self) -> Option<String> {
        self.cesium_ion_access_token
            .clone()
            .filter(|k| !k.trim().is_empty())
            .or_else(|| {
                std::env::var("CESIUM_ION_ACCESS_TOKEN")
                    .ok()
                    .filter(|k| !k.trim().is_empty())
            })
    }
}

/// Replace runtime tile credentials (called from shell on launch and when Settings saves).
pub fn set_tiles_credentials(creds: TilesCredentials) {
    *store().lock().unwrap() = creds;
}

/// Current tile credentials (runtime overrides + env fallback).
pub fn tiles_credentials() -> TilesCredentials {
    store().lock().unwrap().clone()
}

/// Human-readable provider readiness for UI.
pub fn tiles_provider_status() -> String {
    let creds = tiles_credentials();
    if creds.google_key().is_some() {
        return "Google Photorealistic 3D Tiles (API key configured)".into();
    }
    if creds.cesium_ion_token().is_some() {
        return "Cesium ion (access token configured)".into();
    }
    "Cesium ion dev tileset (no API keys — add Google key in Settings for photorealistic tiles)"
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_overrides_env() {
        set_tiles_credentials(TilesCredentials {
            google_map_tiles_api_key: Some("test-google".into()),
            cesium_ion_access_token: None,
        });
        assert_eq!(
            tiles_credentials().google_key().as_deref(),
            Some("test-google")
        );
        set_tiles_credentials(TilesCredentials::default());
    }

    #[test]
    fn status_reflects_google_key() {
        set_tiles_credentials(TilesCredentials {
            google_map_tiles_api_key: Some("k".into()),
            ..Default::default()
        });
        assert!(tiles_provider_status().contains("Google"));
        set_tiles_credentials(TilesCredentials::default());
    }
}
