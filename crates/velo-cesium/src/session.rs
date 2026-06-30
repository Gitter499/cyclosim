use std::collections::HashMap;

use thiserror::Error;
use url::Url;

use crate::attribution::{attribution_for_provider, TileAttribution, TileProvider};
use crate::gltf::{decode_gltf_bytes, GltfDecodeError};
use crate::mesh::TileMesh;
use crate::policy::{OnlineOnlyPolicy, PolicyError};
use crate::synthetic::synthetic_triangle_glb;
use crate::tileset::{TilesetDocument, TilesetError};
use crate::DEV_ION_ASSET_ID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewCorridor {
    pub lat: f64,
    pub lon: f64,
    pub radius_m: f64,
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("policy: {0}")]
    Policy(#[from] PolicyError),
    #[error("tileset: {0}")]
    Tileset(#[from] TilesetError),
    #[error("gltf: {0}")]
    Gltf(#[from] GltfDecodeError),
    #[error("network: {0}")]
    Network(String),
    #[error("session offline — 3D Tiles require network during ride")]
    Offline,
}

/// In-memory 3D Tiles session. Never persists tile bytes to disk.
pub struct TilesSession {
    policy: OnlineOnlyPolicy,
    provider: TileProvider,
    attribution: TileAttribution,
    /// In-memory tile payload cache (cleared on drop — not disk-backed).
    memory_cache: HashMap<String, Vec<u8>>,
    loaded_meshes: Vec<TileMesh>,
    online: bool,
    tileset_url: Option<String>,
}

impl TilesSession {
    /// Offline/synthetic session for CI and unit tests.
    pub fn synthetic() -> Self {
        let provider = TileProvider::CesiumIonDev;
        let mut session = Self {
            policy: OnlineOnlyPolicy::new(),
            provider,
            attribution: attribution_for_provider(provider),
            memory_cache: HashMap::new(),
            loaded_meshes: Vec::new(),
            online: false,
            tileset_url: None,
        };
        let glb = synthetic_triangle_glb();
        if let Ok(mesh) = decode_gltf_bytes(&glb, "synthetic") {
            session.loaded_meshes.push(mesh);
        }
        session
    }

    /// Online session — uses Google tiles when `GOOGLE_MAP_TILES_API_KEY` is set,
    /// otherwise the public Cesium ion dev asset.
    pub fn online_default() -> Result<Self, SessionError> {
        let google_key = std::env::var("GOOGLE_MAP_TILES_API_KEY").ok();
        let (provider, tileset_url) = if google_key.is_some() {
            (
                TileProvider::GooglePhotorealistic,
                None, // Google root URL resolved at fetch time in follow-up PR
            )
        } else {
            (
                TileProvider::CesiumIonDev,
                Some(format!(
                    "https://assets.ion.cesium.com/{}/tileset.json",
                    DEV_ION_ASSET_ID
                )),
            )
        };

        Ok(Self {
            policy: OnlineOnlyPolicy::new(),
            attribution: attribution_for_provider(provider),
            provider,
            memory_cache: HashMap::new(),
            loaded_meshes: Vec::new(),
            online: true,
            tileset_url,
        })
    }

    pub fn attribution(&self) -> &TileAttribution {
        &self.attribution
    }

    pub fn provider(&self) -> TileProvider {
        self.provider
    }

    pub fn is_online(&self) -> bool {
        self.online
    }

    pub fn meshes(&self) -> &[TileMesh] {
        &self.loaded_meshes
    }

    /// Update visible tiles for the current view. Fetches into memory only.
    pub fn tick(&mut self, view: ViewCorridor) -> Result<&[TileMesh], SessionError> {
        self.policy.check_no_disk_write()?;
        if !self.online {
            return Ok(&self.loaded_meshes);
        }

        if self.loaded_meshes.is_empty() {
            self.bootstrap_online_tiles(view)?;
        }
        Ok(&self.loaded_meshes)
    }

    fn bootstrap_online_tiles(&mut self, _view: ViewCorridor) -> Result<(), SessionError> {
        let Some(tileset_url) = self.tileset_url.clone() else {
            // Google path: load synthetic placeholder until full API wiring lands.
            return self.load_synthetic_fallback();
        };

        let tileset_json = fetch_bytes_in_memory(&tileset_url, &mut self.memory_cache)?;
        let doc = TilesetDocument::parse_json(std::str::from_utf8(&tileset_json).map_err(|e| {
            SessionError::Network(e.to_string())
        })?)?;

        let base = Url::parse(&tileset_url)
            .map_err(|e| SessionError::Network(e.to_string()))?;

        for uri in doc.content_uris(1).into_iter().take(1) {
            let tile_url = base.join(&uri).map_err(|e| SessionError::Network(e.to_string()))?;
            let bytes = fetch_bytes_in_memory(tile_url.as_str(), &mut self.memory_cache)?;
            let mesh = decode_gltf_bytes(&bytes, uri)?;
            self.loaded_meshes.push(mesh);
        }

        if self.loaded_meshes.is_empty() {
            self.load_synthetic_fallback()?;
        }
        Ok(())
    }

    fn load_synthetic_fallback(&mut self) -> Result<(), SessionError> {
        let glb = synthetic_triangle_glb();
        let mesh = decode_gltf_bytes(&glb, "fallback")?;
        self.loaded_meshes.push(mesh);
        Ok(())
    }

    /// Inject tile bytes directly (tests only).
    #[doc(hidden)]
    pub fn inject_glb_for_test(&mut self, bytes: &[u8], tile_id: &str) -> Result<(), SessionError> {
        self.policy.check_no_disk_write()?;
        let mesh = decode_gltf_bytes(bytes, tile_id)?;
        self.loaded_meshes.push(mesh);
        Ok(())
    }
}

impl Drop for TilesSession {
    fn drop(&mut self) {
        self.memory_cache.clear();
        self.loaded_meshes.clear();
    }
}

#[cfg(feature = "network")]
fn fetch_bytes_in_memory(
    url: &str,
    cache: &mut HashMap<String, Vec<u8>>,
) -> Result<Vec<u8>, SessionError> {
    if let Some(bytes) = cache.get(url) {
        return Ok(bytes.clone());
    }
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|e| SessionError::Network(e.to_string()))?;
    let resp = client
        .get(url)
        .header("User-Agent", "VeloSim/0.1 (M3b spike)")
        .send()
        .map_err(|e| SessionError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(SessionError::Network(format!("HTTP {}", resp.status())));
    }
    let bytes = resp
        .bytes()
        .map_err(|e| SessionError::Network(e.to_string()))?
        .to_vec();
    cache.insert(url.to_string(), bytes.clone());
    Ok(bytes)
}

#[cfg(not(feature = "network"))]
fn fetch_bytes_in_memory(_url: &str, _cache: &mut HashMap<String, Vec<u8>>) -> Result<Vec<u8>, SessionError> {
    Err(SessionError::Offline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_session_has_mesh() {
        let session = TilesSession::synthetic();
        assert!(!session.meshes().is_empty());
        assert_eq!(session.provider(), TileProvider::CesiumIonDev);
    }

    #[test]
    fn tick_offline_returns_meshes() {
        let mut session = TilesSession::synthetic();
        let view = ViewCorridor {
            lat: 37.7749,
            lon: -122.4194,
            radius_m: 500.0,
        };
        let meshes = session.tick(view).unwrap();
        assert!(!meshes.is_empty());
    }

    #[test]
    fn inject_glb_for_test() {
        let glb = synthetic_triangle_glb();
        let mut session = TilesSession::synthetic();
        let before = session.meshes().len();
        session.inject_glb_for_test(&glb, "extra").unwrap();
        assert_eq!(session.meshes().len(), before + 1);
    }
}
