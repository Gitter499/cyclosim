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
fn migrate_v1_db_adds_highlight_clip_path() {
    use velo_rides::schema::{CREATE_INDEX_STARTED, CREATE_RIDES_TABLE, CREATE_SCHEMA_VERSION};

    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(CREATE_SCHEMA_VERSION).unwrap();
    conn.execute_batch(CREATE_RIDES_TABLE).unwrap();
    conn.execute_batch(CREATE_INDEX_STARTED).unwrap();
    conn.execute("INSERT INTO schema_version (version) VALUES (1)", [])
        .unwrap();
    conn.execute(
        "INSERT INTO rides (id, started_at_unix, elapsed_s, distance_m, fit_path, publish_status, created_at_unix)
         VALUES ('ride-v1', 1_700_000_000, 60.0, 1500.0, '/tmp/ride.fit', 'local', 1_700_000_000)",
        [],
    )
    .unwrap();

    migrate(&conn).unwrap();

    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 2);

    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(rides)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(cols.contains(&"highlight_clip_path".to_string()));

    let elapsed: f64 = conn
        .query_row(
            "SELECT elapsed_s FROM rides WHERE id = 'ride-v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!((elapsed - 60.0).abs() < f64::EPSILON);

    conn.execute(
        "UPDATE rides SET highlight_clip_path = '/tmp/highlight.mp4' WHERE id = 'ride-v1'",
        [],
    )
    .unwrap();
    let clip: Option<String> = conn
        .query_row(
            "SELECT highlight_clip_path FROM rides WHERE id = 'ride-v1'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(clip.as_deref(), Some("/tmp/highlight.mp4"));
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
