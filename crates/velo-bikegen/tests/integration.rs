use std::fs;
use std::path::PathBuf;

use velo_bikegen::{import_bike_from_images, list_bikes, load_bike_asset};
use velo_cesium::decode_gltf_bytes;

fn write_fixture_png(path: &PathBuf, rgb: [u8; 3]) {
    let mut buf = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut buf, 4, 4);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        let pixels: Vec<u8> = (0..16).flat_map(|_| rgb).collect();
        writer.write_image_data(&pixels).unwrap();
    }
    fs::write(path, buf).unwrap();
}

#[test]
fn bike_library_offline_pipeline() {
    let dir = tempfile::tempdir().unwrap();
    let fixtures = dir.path().join("fixtures");
    fs::create_dir_all(&fixtures).unwrap();

    let img1 = fixtures.join("side.png");
    let img2 = fixtures.join("front.png");
    write_fixture_png(&img1, [180, 30, 30]);
    write_fixture_png(&img2, [30, 80, 180]);

    let bikes_dir = dir.path().join("bikes");
    let asset = import_bike_from_images(
        &bikes_dir,
        &[img1, img2],
        "fixture-bike",
        Some("Fixture Bike"),
    )
    .unwrap();

    assert!(asset.gltf_path.exists());
    let bytes = fs::read(&asset.gltf_path).unwrap();
    assert!(bytes.starts_with(b"glTF"));
    let mesh = decode_gltf_bytes(&bytes, "fixture-bike").unwrap();
    assert!(mesh.vertices.len() > 3);
    assert!(!mesh.indices.is_empty());

    let bikes = list_bikes(&bikes_dir).unwrap();
    assert_eq!(bikes.len(), 1);
    assert_eq!(bikes[0].name, "Fixture Bike");

    let loaded = load_bike_asset(&bikes_dir, "fixture-bike").unwrap();
    assert_eq!(loaded.anchor.scale, asset.anchor.scale);
}
