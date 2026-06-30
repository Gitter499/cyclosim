use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{Result, RideStoreError};
use crate::schema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishStatus {
    Local,
    Strava,
    Failed,
}

impl PublishStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Strava => "strava",
            Self::Failed => "failed",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(Self::Local),
            "strava" => Some(Self::Strava),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RideRecord {
    pub id: String,
    pub started_at_unix: u64,
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub avg_power_w: Option<f64>,
    pub max_power_w: Option<f64>,
    pub fit_path: String,
    pub screenshot_path: Option<String>,
    pub strava_activity_id: Option<String>,
    pub publish_status: PublishStatus,
    pub route_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewRideRecord {
    pub started_at_unix: u64,
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub avg_power_w: Option<f64>,
    pub max_power_w: Option<f64>,
    pub fit_path: String,
    pub screenshot_path: Option<String>,
    pub strava_activity_id: Option<String>,
    pub publish_status: PublishStatus,
    pub route_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RideArtifacts {
    pub ride_id: String,
    pub fit_path: PathBuf,
    pub screenshot_path: Option<PathBuf>,
    pub ride_dir: PathBuf,
}

pub struct RideLibrary {
    conn: Arc<Mutex<Connection>>,
    artifacts_base: PathBuf,
}

impl RideLibrary {
    pub fn open(db_path: impl AsRef<Path>, artifacts_base: impl AsRef<Path>) -> Result<Self> {
        if let Some(parent) = db_path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&artifacts_base)?;
        let conn = Connection::open(db_path)?;
        schema::migrate(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            artifacts_base: artifacts_base.as_ref().to_path_buf(),
        })
    }

    pub fn open_in_memory(artifacts_base: impl AsRef<Path>) -> Result<Self> {
        fs::create_dir_all(&artifacts_base)?;
        let conn = Connection::open_in_memory()?;
        schema::migrate(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            artifacts_base: artifacts_base.as_ref().to_path_buf(),
        })
    }

    pub fn artifacts_base(&self) -> &Path {
        &self.artifacts_base
    }

    /// Write FIT (+ optional PNG) under `{artifacts_base}/{uuid}/`.
    pub fn save_ride_artifacts(
        &self,
        fit_bytes: &[u8],
        screenshot_png: Option<&[u8]>,
    ) -> Result<RideArtifacts> {
        let ride_id = Uuid::new_v4().to_string();
        let ride_dir = self.artifacts_base.join(&ride_id);
        fs::create_dir_all(&ride_dir)?;

        let fit_path = ride_dir.join("ride.fit");
        fs::write(&fit_path, fit_bytes)?;

        let screenshot_path = screenshot_png.map(|png| {
            let path = ride_dir.join("screenshot.png");
            fs::write(&path, png).expect("screenshot write");
            path
        });

        Ok(RideArtifacts {
            ride_id,
            fit_path,
            screenshot_path,
            ride_dir,
        })
    }

    pub fn insert_ride(&self, record: NewRideRecord) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO rides (
                id, started_at_unix, elapsed_s, distance_m,
                avg_power_w, max_power_w, fit_path, screenshot_path,
                strava_activity_id, publish_status, route_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                id,
                record.started_at_unix as i64,
                record.elapsed_s,
                record.distance_m,
                record.avg_power_w,
                record.max_power_w,
                record.fit_path,
                record.screenshot_path,
                record.strava_activity_id,
                record.publish_status.as_str(),
                record.route_id,
            ],
        )?;
        Ok(id)
    }

    /// Insert a ride with a pre-assigned id (used when artifacts dir uses the same uuid).
    pub fn insert_ride_with_id(&self, id: &str, record: NewRideRecord) -> Result<()> {
        validate_ride_id(id)?;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"
            INSERT INTO rides (
                id, started_at_unix, elapsed_s, distance_m,
                avg_power_w, max_power_w, fit_path, screenshot_path,
                strava_activity_id, publish_status, route_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            params![
                id,
                record.started_at_unix as i64,
                record.elapsed_s,
                record.distance_m,
                record.avg_power_w,
                record.max_power_w,
                record.fit_path,
                record.screenshot_path,
                record.strava_activity_id,
                record.publish_status.as_str(),
                record.route_id,
            ],
        )?;
        Ok(())
    }

    pub fn list_rides(&self) -> Result<Vec<RideRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, started_at_unix, elapsed_s, distance_m,
                   avg_power_w, max_power_w, fit_path, screenshot_path,
                   strava_activity_id, publish_status, route_id
            FROM rides
            ORDER BY started_at_unix DESC, created_at_unix DESC
            "#,
        )?;
        let rows = stmt.query_map([], row_to_record)?;
        rows.collect::<std::result::Result<Vec<_>, _>>()
            .map_err(RideStoreError::from)
    }

    pub fn get_ride(&self, id: &str) -> Result<Option<RideRecord>> {
        validate_ride_id(id)?;
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"
            SELECT id, started_at_unix, elapsed_s, distance_m,
                   avg_power_w, max_power_w, fit_path, screenshot_path,
                   strava_activity_id, publish_status, route_id
            FROM rides WHERE id = ?1
            "#,
        )?;
        let mut rows = stmt.query([id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row_to_record(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_ride(&self, id: &str) -> Result<bool> {
        validate_ride_id(id)?;
        let ride = match self.get_ride(id)? {
            Some(r) => r,
            None => return Ok(false),
        };

        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM rides WHERE id = ?1", [id])?;

        if deleted > 0 {
            if let Some(parent) = Path::new(&ride.fit_path).parent() {
                let _ = fs::remove_dir_all(parent);
            }
        }

        Ok(deleted > 0)
    }
}

fn validate_ride_id(id: &str) -> Result<()> {
    if Uuid::parse_str(id).is_err() {
        return Err(RideStoreError::InvalidId(id.to_string()));
    }
    Ok(())
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<RideRecord> {
    let status_str: String = row.get(9)?;
    let publish_status = PublishStatus::from_str(&status_str).unwrap_or(PublishStatus::Local);
    Ok(RideRecord {
        id: row.get(0)?,
        started_at_unix: row.get::<_, i64>(1)? as u64,
        elapsed_s: row.get(2)?,
        distance_m: row.get(3)?,
        avg_power_w: row.get(4)?,
        max_power_w: row.get(5)?,
        fit_path: row.get(6)?,
        screenshot_path: row.get(7)?,
        strava_activity_id: row.get(8)?,
        publish_status,
        route_id: row.get(10)?,
    })
}
