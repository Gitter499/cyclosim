use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use velo_ffi::VeloHandle;

fn write_test_png(path: &PathBuf, rgb: [u8; 3]) {
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
fn ffi_bike_import_list_and_active() {
    let bikes_dir = TempDir::new().unwrap();
    let packs_dir = TempDir::new().unwrap();
    let handle = VeloHandle::with_dirs_for_tests(
        packs_dir.path().to_path_buf(),
        bikes_dir.path().to_path_buf(),
    );

    let img = bikes_dir.path().join("side.png");
    write_test_png(&img, [120, 40, 40]);

    handle
        .import_bike_from_images(
            vec![img.display().to_string()],
            "ffi-bike".into(),
            Some("FFI Bike".into()),
        )
        .unwrap();

    let bikes = handle.list_bikes().unwrap();
    assert_eq!(bikes.len(), 1);
    assert_eq!(bikes[0].bike_id, "ffi-bike");
    assert_eq!(handle.active_bike_id(), Some("ffi-bike".into()));

    handle.clear_active_bike();
    assert_eq!(handle.active_bike_id(), None);

    handle.set_active_bike("ffi-bike".into()).unwrap();
    assert_eq!(handle.active_bike_id(), Some("ffi-bike".into()));
}
