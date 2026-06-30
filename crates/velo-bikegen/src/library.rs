//! Bike asset library on disk (`~/Documents/VeloSim/bikes/<id>/`).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json;
use thiserror::Error;

use crate::asset::{BikeAsset, BikeMeta, GLTF_FILE, META_FILE, SOURCES_DIR};
use crate::placeholder::{
    default_placeholder_anchor, generate_placeholder_glb, PLACEHOLDER_GENERATOR, TARGET_WHEELBASE_M,
};

#[derive(Debug, Error)]
pub enum BikeImportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("import: {0}")]
    Import(#[from] crate::placeholder::PlaceholderError),
    #[error("need 1–4 source images, got {0}")]
    ImageCount(usize),
    #[error("bike not found: {0}")]
    NotFound(String),
    #[error("invalid bike id")]
    InvalidId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BikeSummary {
    pub bike_id: String,
    pub name: String,
}

/// Default user bike library: `~/Documents/VeloSim/bikes/`.
pub fn default_bikes_dir() -> PathBuf {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|h| h.join("Documents").join("VeloSim").join("bikes"))
        .unwrap_or_else(|| PathBuf::from("assets/bikes"))
}

pub fn bike_dir_for_id(bikes_dir: &Path, bike_id: &str) -> PathBuf {
    bikes_dir.join(bike_id)
}

pub fn list_bikes(bikes_dir: &Path) -> Result<Vec<BikeSummary>, BikeImportError> {
    let mut bikes = Vec::new();
    if !bikes_dir.is_dir() {
        return Ok(bikes);
    }
    for entry in fs::read_dir(bikes_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let meta_path = entry.path().join(META_FILE);
            if meta_path.is_file() {
                let data = fs::read_to_string(&meta_path)?;
                let meta: BikeMeta = serde_json::from_str(&data)?;
                bikes.push(BikeSummary {
                    bike_id: meta.bike_id,
                    name: meta.name,
                });
            }
        }
    }
    bikes.sort_by(|a, b| a.bike_id.cmp(&b.bike_id));
    Ok(bikes)
}

pub fn load_bike_asset(bikes_dir: &Path, bike_id: &str) -> Result<BikeAsset, BikeImportError> {
    let bike_dir = bike_dir_for_id(bikes_dir, bike_id);
    let meta_path = bike_dir.join(META_FILE);
    if !meta_path.is_file() {
        return Err(BikeImportError::NotFound(bike_id.to_string()));
    }
    let meta: BikeMeta = serde_json::from_str(&fs::read_to_string(meta_path)?)?;
    let gltf_path = meta.gltf_path(&bike_dir);
    if !gltf_path.is_file() {
        return Err(BikeImportError::NotFound(format!("{bike_id}/{}", GLTF_FILE)));
    }
    Ok(meta.to_asset(&bike_dir))
}

/// Import 1–4 images → placeholder glTF in the bike library (v1 offline path).
pub fn import_bike_from_images(
    bikes_dir: &Path,
    image_paths: &[PathBuf],
    bike_id: &str,
    name: Option<&str>,
) -> Result<BikeAsset, BikeImportError> {
    validate_bike_id(bike_id)?;
    if image_paths.is_empty() || image_paths.len() > 4 {
        return Err(BikeImportError::ImageCount(image_paths.len()));
    }
    for path in image_paths {
        if !path.is_file() {
            return Err(BikeImportError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("image not found: {}", path.display()),
            )));
        }
    }

    let bike_dir = bike_dir_for_id(bikes_dir, bike_id);
    let sources_dir = bike_dir.join(SOURCES_DIR);
    fs::create_dir_all(&sources_dir)?;

    let mut stored_sources = Vec::new();
    for (i, src) in image_paths.iter().enumerate() {
        let ext = src
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let dest_name = format!("{i}.{ext}");
        let dest = sources_dir.join(&dest_name);
        fs::copy(src, &dest)?;
        stored_sources.push(format!("{SOURCES_DIR}/{dest_name}"));
    }

    let glb = generate_placeholder_glb(image_paths)?;
    let gltf_path = bike_dir.join(GLTF_FILE);
    fs::write(&gltf_path, &glb)?;

    let anchor = normalize_anchor(&glb);
    let display_name = name.unwrap_or(bike_id);
    let meta = BikeMeta {
        bike_id: bike_id.to_string(),
        name: display_name.to_string(),
        gltf_file: GLTF_FILE.to_string(),
        anchor,
        source_images: stored_sources,
        generator: PLACEHOLDER_GENERATOR.to_string(),
    };
    fs::write(bike_dir.join(META_FILE), serde_json::to_vec_pretty(&meta)?)?;

    Ok(meta.to_asset(&bike_dir))
}

fn validate_bike_id(id: &str) -> Result<(), BikeImportError> {
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(BikeImportError::InvalidId);
    }
    Ok(())
}

/// Scale placeholder mesh so wheelbase matches [`TARGET_WHEELBASE_M`].
fn normalize_anchor(glb: &[u8]) -> crate::asset::AnchorTransform {
    let mut anchor = default_placeholder_anchor();
    if let Ok(mesh) = velo_cesium::decode_gltf_bytes(glb, "normalize") {
        let (min, max) = mesh_bbox(&mesh);
        let extent_x = (max[0] - min[0]).max(0.01);
        let scale = TARGET_WHEELBASE_M / extent_x;
        anchor.scale = scale;
        // Sit bottom of bbox on ground (y = 0 at rider feet).
        anchor.translation[1] = -min[1] * scale;
    }
    anchor
}

fn mesh_bbox(mesh: &velo_cesium::TileMesh) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for v in &mesh.vertices {
        for (i, c) in v.position.iter().enumerate() {
            min[i] = min[i].min(*c);
            max[i] = max[i].max(*c);
        }
    }
    (min, max)
}

#[cfg(feature = "hosted-api")]
pub mod hosted_api {
    //! Scaffold for Meshy/Tripo hosted image-to-3D APIs (not wired in v1).

    #[derive(Debug)]
    pub struct HostedApiConfig {
        pub provider: String,
        pub api_key_env: String,
    }

    pub fn meshy_config() -> HostedApiConfig {
        HostedApiConfig {
            provider: "meshy".into(),
            api_key_env: "MESHY_API_KEY".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_test_png(path: &Path, rgb: [u8; 3]) {
        let mut buf = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut buf, 2, 2);
            enc.set_color(png::ColorType::Rgb);
            enc.set_depth(png::BitDepth::Eight);
            let mut writer = enc.write_header().unwrap();
            let pixels: Vec<u8> = (0..4).flat_map(|_| rgb).collect();
            writer.write_image_data(&pixels).unwrap();
        }
        fs::write(path, buf).unwrap();
    }

    #[test]
    fn import_and_list_round_trip() {
        let dir = std::env::temp_dir().join("velo-bikegen-test");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let img = dir.join("photo.png");
        write_test_png(&img, [200, 40, 40]);

        let asset = import_bike_from_images(&dir, &[img], "test-bike", Some("My Bike")).unwrap();
        assert!(asset.gltf_path.is_file());

        let bikes = list_bikes(&dir).unwrap();
        assert_eq!(bikes.len(), 1);
        assert_eq!(bikes[0].bike_id, "test-bike");

        let loaded = load_bike_asset(&dir, "test-bike").unwrap();
        assert_eq!(loaded.bike_id, "test-bike");

        let _ = fs::remove_dir_all(&dir);
    }
}
