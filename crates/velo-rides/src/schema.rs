use rusqlite::Connection;

use crate::error::Result;

pub const SCHEMA_VERSION: i32 = 1;

pub const CREATE_RIDES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS rides (
    id TEXT PRIMARY KEY NOT NULL,
    started_at_unix INTEGER NOT NULL,
    elapsed_s REAL NOT NULL,
    distance_m REAL NOT NULL,
    avg_power_w REAL,
    max_power_w REAL,
    fit_path TEXT NOT NULL,
    screenshot_path TEXT,
    strava_activity_id TEXT,
    publish_status TEXT NOT NULL,
    route_id TEXT,
    created_at_unix INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
"#;

pub const CREATE_INDEX_STARTED: &str = r#"
CREATE INDEX IF NOT EXISTS idx_rides_started_at ON rides(started_at_unix DESC);
"#;

pub const CREATE_SCHEMA_VERSION: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);
"#;

/// Apply schema migrations idempotently.
pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(CREATE_SCHEMA_VERSION)?;
    let current: i32 = conn
        .query_row(
            "SELECT COALESCE((SELECT version FROM schema_version LIMIT 1), 0)",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch(CREATE_RIDES_TABLE)?;
        conn.execute_batch(CREATE_INDEX_STARTED)?;
        conn.execute("DELETE FROM schema_version", [])?;
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?1)",
            [SCHEMA_VERSION],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_rides_table() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='rides'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
