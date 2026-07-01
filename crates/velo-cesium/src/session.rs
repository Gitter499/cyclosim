use std::collections::HashMap;

use thiserror::Error;
use url::Url;

use crate::attribution::{attribution_for_provider, TileAttribution, TileProvider};
use crate::credentials::tiles_credentials;
use crate::gltf::{decode_gltf_bytes, GltfDecodeError};
use crate::mesh::TileMesh;
use crate::policy::{OnlineOnlyPolicy, PolicyError};
use crate::synthetic::synthetic_triangle_glb;
use crate::tileset::{TilesetDocument, TilesetError};
use crate::DEV_ION_ASSET_ID;

const GOOGLE_ROOT_TILESET: &str = "https://tile.googleapis.com/v1/3dtiles/root.json";

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
    #[error("missing API key for configured tile provider")]
    MissingApiKey,
}

/// HTTP fetch options for tile requests (API keys via header or query).
#[derive(Debug, Clone, Default)]
struct FetchAuth {
    google_api_key: Option<String>,
    cesium_ion_token: Option<String>,
}

impl FetchAuth {
    fn from_credentials() -> Self {
        let creds = tiles_credentials();
        Self {
            google_api_key: creds.google_key(),
            cesium_ion_token: creds.cesium_ion_token(),
        }
    }

    fn google_root_url(&self) -> Option<String> {
        self.google_api_key
            .as_ref()
            .map(|key| format!("{GOOGLE_ROOT_TILESET}?key={}", urlencoding::encode(key)))
    }

    fn resolve_tile_url(&self, base: &Url, href: &str) -> Result<String, SessionError> {
        let joined = base
            .join(href)
            .map_err(|e| SessionError::Network(e.to_string()))?;
        let mut url = joined;
        if self.google_api_key.is_some() {
            if let Some(key) = &self.google_api_key {
                url.query_pairs_mut().append_pair("key", key);
            }
        }
        Ok(url.to_string())
    }
}

/// In-memory 3D Tiles session. Never persists tile bytes to disk.
pub struct TilesSession {
    policy: OnlineOnlyPolicy,
    provider: TileProvider,
    attribution: TileAttribution,
    fetch_auth: FetchAuth,
    /// In-memory tile payload cache (cleared on drop — not disk-backed).
    memory_cache: HashMap<String, Vec<u8>>,
    loaded_meshes: Vec<TileMesh>,
    online: bool,
    tileset_url: Option<String>,
    last_error: Option<String>,
}

impl TilesSession {
    /// Offline/synthetic session for CI and unit tests.
    pub fn synthetic() -> Self {
        let provider = TileProvider::CesiumIonDev;
        let mut session = Self {
            policy: OnlineOnlyPolicy::new(),
            provider,
            attribution: attribution_for_provider(provider),
            fetch_auth: FetchAuth::default(),
            memory_cache: HashMap::new(),
            loaded_meshes: Vec::new(),
            online: false,
            tileset_url: None,
            last_error: None,
        };
        let glb = synthetic_triangle_glb();
        if let Ok(mesh) = decode_gltf_bytes(&glb, "synthetic") {
            session.loaded_meshes.push(mesh);
        }
        session
    }

    /// Online session — Google tiles when key configured, else Cesium ion (token or dev asset).
    pub fn online_default() -> Result<Self, SessionError> {
        let auth = FetchAuth::from_credentials();
        let (provider, tileset_url) = if let Some(url) = auth.google_root_url() {
            (TileProvider::GooglePhotorealistic, Some(url))
        } else if auth.cesium_ion_token.is_some() {
            (
                TileProvider::CesiumIonDev,
                Some(format!(
                    "https://assets.ion.cesium.com/{}/tileset.json?access_token={}",
                    DEV_ION_ASSET_ID,
                    urlencoding::encode(auth.cesium_ion_token.as_ref().unwrap())
                )),
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
            fetch_auth: auth,
            memory_cache: HashMap::new(),
            loaded_meshes: Vec::new(),
            online: true,
            tileset_url,
            last_error: None,
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

    /// Last bootstrap/network error (for HUD / setup status).
    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// Update visible tiles for the current view. Fetches into memory only.
    pub fn tick(&mut self, view: ViewCorridor) -> Result<&[TileMesh], SessionError> {
        self.policy.check_no_disk_write()?;
        if !self.online {
            return Ok(&self.loaded_meshes);
        }

        if self.loaded_meshes.is_empty() {
            if let Err(e) = self.bootstrap_online_tiles(view) {
                self.last_error = Some(e.to_string());
                self.load_synthetic_fallback()?;
            }
        }
        Ok(&self.loaded_meshes)
    }

    fn bootstrap_online_tiles(&mut self, _view: ViewCorridor) -> Result<(), SessionError> {
        let Some(tileset_url) = self.tileset_url.clone() else {
            return Err(SessionError::MissingApiKey);
        };

        let tileset_json =
            fetch_bytes_in_memory(&tileset_url, &mut self.memory_cache, &self.fetch_auth)?;
        let doc = TilesetDocument::parse_json(
            std::str::from_utf8(&tileset_json).map_err(|e| SessionError::Network(e.to_string()))?,
        )?;

        let base = Url::parse(&tileset_url).map_err(|e| SessionError::Network(e.to_string()))?;

        for uri in doc.content_uris(2).into_iter().take(2) {
            let tile_url = self.fetch_auth.resolve_tile_url(&base, &uri)?;
            let bytes = fetch_bytes_in_memory(&tile_url, &mut self.memory_cache, &self.fetch_auth)?;
            let mesh = decode_gltf_bytes(&bytes, &uri)?;
            self.loaded_meshes.push(mesh);
        }

        if self.loaded_meshes.is_empty() {
            return Err(SessionError::Network(
                "tileset contained no decodable meshes".into(),
            ));
        }
        self.last_error = None;
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
    auth: &FetchAuth,
) -> Result<Vec<u8>, SessionError> {
    if let Some(bytes) = cache.get(url) {
        return Ok(bytes.clone());
    }
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(8))
        .build()
        .map_err(|e| SessionError::Network(e.to_string()))?;
    let mut req = client
        .get(url)
        .header("User-Agent", "VeloSim/0.1 (M3b spike)");
    if auth.google_api_key.is_some() && url.contains("tile.googleapis.com") {
        if let Some(key) = &auth.google_api_key {
            req = req.header("X-GOOG-API-KEY", key.as_str());
        }
    }
    let resp = req
        .send()
        .map_err(|e| SessionError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(SessionError::Network(format!(
            "HTTP {} for {}",
            resp.status(),
            redact_url(url)
        )));
    }
    let bytes = resp
        .bytes()
        .map_err(|e| SessionError::Network(e.to_string()))?
        .to_vec();
    cache.insert(url.to_string(), bytes.clone());
    Ok(bytes)
}

#[cfg(not(feature = "network"))]
fn fetch_bytes_in_memory(
    _url: &str,
    _cache: &mut HashMap<String, Vec<u8>>,
    _auth: &FetchAuth,
) -> Result<Vec<u8>, SessionError> {
    Err(SessionError::Offline)
}

fn redact_url(url: &str) -> String {
    url.split('?').next().unwrap_or(url).to_string()
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

    #[test]
    fn google_root_url_includes_key() {
        let auth = FetchAuth {
            google_api_key: Some("abc123".into()),
            ..Default::default()
        };
        let url = auth.google_root_url().unwrap();
        assert!(url.contains("tile.googleapis.com"));
        assert!(url.contains("key=abc123"));
    }
}
