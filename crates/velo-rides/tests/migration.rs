#[path = "common/mod.rs"]
mod common;

use rusqlite::Connection;
use velo_rides::schema::migrate;

#[test]
fn migrate_from_version_zero_applies_schema() {
    let conn = Connection::open_in_memory().unwrap();
    migrate(&conn).unwrap();
    let tables: Vec<String> = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(tables.contains(&"rides".to_string()));
    assert!(tables.contains(&"schema_version".to_string()));
}

#[test]
fn second_open_reuses_existing_db() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("rides.db");
    let artifacts = dir.path().join("artifacts");

    {
        let lib = velo_rides::RideLibrary::open(&db, &artifacts).unwrap();
        let arts = lib.save_ride_artifacts(b"fit", None).unwrap();
        lib.insert_ride_with_id(
            &arts.ride_id,
            common::sample_record(&arts.fit_path.display().to_string()),
        )
        .unwrap();
    }

    let lib2 = velo_rides::RideLibrary::open(&db, &artifacts).unwrap();
    assert_eq!(lib2.list_rides().unwrap().len(), 1);
}
