//! Route model — ordered points with distance, grade, and georeferencing.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RoutePoint {
    pub distance_m: f64,
    pub lat: f64,
    pub lon: f64,
    pub elevation_m: f64,
    pub grade: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteOrigin {
    pub lat: f64,
    pub lon: f64,
    pub elevation_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteBBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteMeta {
    pub route_id: String,
    pub name: String,
    pub origin: RouteOrigin,
    pub total_distance_m: f64,
    pub bbox: RouteBBox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteModel {
    pub meta: RouteMeta,
    pub points: Vec<RoutePoint>,
}

#[derive(Debug, Error)]
pub enum RouteError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("empty route")]
    Empty,
    #[error("pack missing {0}")]
    MissingFile(String),
}

impl RouteModel {
    pub fn new(route_id: impl Into<String>, name: impl Into<String>, points: Vec<RoutePoint>) -> Result<Self, RouteError> {
        if points.is_empty() {
            return Err(RouteError::Empty);
        }
        let origin = RouteOrigin {
            lat: points[0].lat,
            lon: points[0].lon,
            elevation_m: points[0].elevation_m,
        };
        let mut min_lat = points[0].lat;
        let mut max_lat = points[0].lat;
        let mut min_lon = points[0].lon;
        let mut max_lon = points[0].lon;
        for p in &points {
            min_lat = min_lat.min(p.lat);
            max_lat = max_lat.max(p.lat);
            min_lon = min_lon.min(p.lon);
            max_lon = max_lon.max(p.lon);
        }
        let total_distance_m = points.last().map(|p| p.distance_m).unwrap_or(0.0);
        Ok(Self {
            meta: RouteMeta {
                route_id: route_id.into(),
                name: name.into(),
                origin,
                total_distance_m,
                bbox: RouteBBox {
                    min_lat,
                    max_lat,
                    min_lon,
                    max_lon,
                },
            },
            points,
        })
    }

    pub fn total_distance_m(&self) -> f64 {
        self.meta.total_distance_m
    }

    /// Grade (rise/run) at distance along route; clamps to endpoints.
    pub fn grade_at(&self, distance_m: f64) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }
        if distance_m <= self.points[0].distance_m {
            return self.points[0].grade;
        }
        let last = self.points.last().unwrap();
        if distance_m >= last.distance_m {
            return last.grade;
        }
        let idx = self
            .points
            .partition_point(|p| p.distance_m <= distance_m)
            .saturating_sub(1);
        let a = &self.points[idx];
        let b = &self.points[(idx + 1).min(self.points.len() - 1)];
        if (b.distance_m - a.distance_m).abs() < f64::EPSILON {
            return a.grade;
        }
        let t = (distance_m - a.distance_m) / (b.distance_m - a.distance_m);
        a.grade + t * (b.grade - a.grade)
    }

    /// Local ENU position (east, up, north) relative to route origin.
    pub fn position_enu_at(&self, distance_m: f64) -> (f64, f64, f64) {
        let (lat, lon, elev) = self.lat_lon_elev_at(distance_m);
        let (east, north) = lat_lon_to_local(self.meta.origin.lat, self.meta.origin.lon, lat, lon);
        (east, elev - self.meta.origin.elevation_m, north)
    }

    pub fn lat_lon_elev_at(&self, distance_m: f64) -> (f64, f64, f64) {
        if self.points.is_empty() {
            return (0.0, 0.0, 0.0);
        }
        if distance_m <= self.points[0].distance_m {
            let p = &self.points[0];
            return (p.lat, p.lon, p.elevation_m);
        }
        let last = self.points.last().unwrap();
        if distance_m >= last.distance_m {
            return (last.lat, last.lon, last.elevation_m);
        }
        let idx = self
            .points
            .partition_point(|p| p.distance_m <= distance_m)
            .saturating_sub(1);
        let a = &self.points[idx];
        let b = &self.points[(idx + 1).min(self.points.len() - 1)];
        if (b.distance_m - a.distance_m).abs() < f64::EPSILON {
            return (a.lat, a.lon, a.elevation_m);
        }
        let t = (distance_m - a.distance_m) / (b.distance_m - a.distance_m);
        (
            a.lat + t * (b.lat - a.lat),
            a.lon + t * (b.lon - a.lon),
            a.elevation_m + t * (b.elevation_m - a.elevation_m),
        )
    }

    pub fn save_pack(&self, pack_dir: &Path) -> Result<(), RouteError> {
        fs::create_dir_all(pack_dir)?;
        fs::write(
            pack_dir.join("meta.json"),
            serde_json::to_vec_pretty(&self.meta)?,
        )?;
        fs::write(
            pack_dir.join("route.json"),
            serde_json::to_vec_pretty(self)?,
        )?;
        Ok(())
    }

    pub fn load_pack(pack_dir: &Path) -> Result<Self, RouteError> {
        let route_path = pack_dir.join("route.json");
        if !route_path.exists() {
            return Err(RouteError::MissingFile("route.json".into()));
        }
        let data = fs::read_to_string(route_path)?;
        let model: RouteModel = serde_json::from_str(&data)?;
        if model.points.is_empty() {
            return Err(RouteError::Empty);
        }
        Ok(model)
    }
}

pub fn lat_lon_to_local(origin_lat: f64, origin_lon: f64, lat: f64, lon: f64) -> (f64, f64) {
    let dlat = (lat - origin_lat).to_radians();
    let dlon = (lon - origin_lon).to_radians();
    let cos_lat = origin_lat.to_radians().cos();
    let east = dlon * cos_lat * EARTH_RADIUS_M;
    let north = dlat * EARTH_RADIUS_M;
    (east, north)
}

pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<RoutePoint> {
        vec![
            RoutePoint {
                distance_m: 0.0,
                lat: 46.0,
                lon: 6.0,
                elevation_m: 400.0,
                grade: 0.0,
            },
            RoutePoint {
                distance_m: 100.0,
                lat: 46.0009,
                lon: 6.0,
                elevation_m: 405.0,
                grade: 0.05,
            },
            RoutePoint {
                distance_m: 200.0,
                lat: 46.0018,
                lon: 6.0,
                elevation_m: 410.0,
                grade: 0.05,
            },
        ]
    }

    #[test]
    fn grade_interpolates() {
        let route = RouteModel::new("t", "Test", sample_points()).unwrap();
        assert!((route.grade_at(50.0) - 0.025).abs() < 0.01);
        assert!((route.grade_at(150.0) - 0.05).abs() < 0.01);
    }

    #[test]
    fn pack_round_trip() {
        let route = RouteModel::new("demo", "Demo", sample_points()).unwrap();
        let dir = std::env::temp_dir().join("velo-route-pack-test");
        let _ = fs::remove_dir_all(&dir);
        route.save_pack(&dir).unwrap();
        let loaded = RouteModel::load_pack(&dir).unwrap();
        assert_eq!(loaded.meta.route_id, "demo");
        assert_eq!(loaded.points.len(), 3);
        let _ = fs::remove_dir_all(&dir);
    }
}
