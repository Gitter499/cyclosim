//! Attribution strings required by tile providers (ToS).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileProvider {
    /// Google Photorealistic 3D Tiles (production Tier B).
    GooglePhotorealistic,
    /// Cesium ion sample / dev tilesets.
    CesiumIonDev,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileAttribution {
    pub provider: TileProvider,
    pub text: String,
}

impl TileAttribution {
    pub fn google() -> Self {
        Self {
            provider: TileProvider::GooglePhotorealistic,
            text: "© Google".to_string(),
        }
    }

    pub fn cesium_ion_dev() -> Self {
        Self {
            provider: TileProvider::CesiumIonDev,
            text: "© Cesium ion / OpenStreetMap contributors".to_string(),
        }
    }
}

pub fn attribution_for_provider(provider: TileProvider) -> TileAttribution {
    match provider {
        TileProvider::GooglePhotorealistic => TileAttribution::google(),
        TileProvider::CesiumIonDev => TileAttribution::cesium_ion_dev(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn google_attribution_non_empty() {
        let a = TileAttribution::google();
        assert!(a.text.contains("Google"));
    }
}
