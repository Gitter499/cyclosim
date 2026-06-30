#[path = "common/mod.rs"]
mod common;

use rusqlite::Connection;
use velo_rides::schema::{migrate, SCHEMA_VERSION};

#[test]
fn schema_version_table_exists_after_migrate() {
    let conn = Connection::open_in_memory().unwrap();
    migrate(&conn).unwrap();
    let version: i32 = conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
}

#[test]
fn rides_table_has_expected_columns() {
    let conn = Connection::open_in_memory().unwrap();
    migrate(&conn).unwrap();
    let sql: String = conn
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='rides'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    for col in [
        "id",
        "started_at_unix",
        "elapsed_s",
        "distance_m",
        "avg_power_w",
        "max_power_w",
        "fit_path",
        "screenshot_path",
        "strava_activity_id",
        "publish_status",
        "route_id",
    ] {
        assert!(sql.contains(col), "missing column {col}");
    }
    assert!(sql.contains("PRIMARY KEY"));

    let cols: Vec<String> = conn
        .prepare("PRAGMA table_info(rides)")
        .unwrap()
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .filter_map(Result::ok)
        .collect();
    assert!(cols.contains(&"highlight_clip_path".to_string()));
}

#[test]
fn migrate_is_idempotent() {
    let conn = Connection::open_in_memory().unwrap();
    migrate(&conn).unwrap();
    migrate(&conn).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM rides", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}
