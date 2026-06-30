use velo_cesium::{decode_gltf_bytes, synthetic_triangle_glb, TilesetDocument, TilesSession};

#[test]
fn tileset_parse_and_gltf_decode_offline() {
    let sample = r#"{
        "asset": { "version": "1.0" },
        "geometricError": 500,
        "root": { "content": { "uri": "tile.glb" } }
    }"#;
    let doc = TilesetDocument::parse_json(sample).unwrap();
    assert_eq!(doc.content_uris(0), vec!["tile.glb".to_string()]);

    let glb = synthetic_triangle_glb();
    let mesh = decode_gltf_bytes(&glb, "fixture").unwrap();
    assert_eq!(mesh.vertices.len(), 3);
}

#[test]
fn session_synthetic_integration() {
    let mut session = TilesSession::synthetic();
    let meshes = session
        .tick(velo_cesium::ViewCorridor {
            lat: 37.77,
            lon: -122.42,
            radius_m: 200.0,
        })
        .unwrap();
    assert!(!meshes.is_empty());
    assert!(session.attribution().text.contains("Cesium"));
}

#[test]
#[cfg(feature = "network")]
#[ignore = "requires network; run with --ignored --features network"]
fn session_online_dev_tileset() {
    let mut session = TilesSession::online_default().unwrap();
    let view = velo_cesium::ViewCorridor {
        lat: 37.7749,
        lon: -122.4194,
        radius_m: 500.0,
    };
    match session.tick(view) {
        Ok(meshes) => assert!(!meshes.is_empty()),
        Err(_) => assert!(!session.meshes().is_empty()),
    }
}
