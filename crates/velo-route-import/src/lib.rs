//! Parse activity files and produce a resampled [`RouteModel`].

mod elevation;
mod gpx;
mod grade;
mod resample;
mod tcx;

pub use elevation::smooth_elevation;
pub use gpx::parse_gpx;
pub use grade::compute_grades;
pub use resample::resample_route;
pub use tcx::parse_tcx_stub;

use thiserror::Error;
use velo_core::{RouteModel, RoutePoint};

pub const DEFAULT_SPACING_M: f64 = 8.0;
pub const DEFAULT_GRADE_WINDOW_M: f64 = 50.0;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("xml: {0}")]
    Xml(String),
    #[error("route: {0}")]
    Route(#[from] velo_core::RouteError),
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    #[error("no track points in file")]
    NoPoints,
}

/// Full import pipeline: parse → resample → smooth elevation → grade → RouteModel.
pub fn import_gpx(
    data: &[u8],
    route_id: &str,
    name: &str,
    spacing_m: f64,
    grade_window_m: f64,
) -> Result<RouteModel, ImportError> {
    let raw = parse_gpx(data)?;
    if raw.is_empty() {
        return Err(ImportError::NoPoints);
    }
    let resampled = resample_route(&raw, spacing_m);
    let smoothed = smooth_elevation(&resampled);
    let graded = compute_grades(&smoothed, grade_window_m);
    RouteModel::new(route_id, name, graded).map_err(ImportError::from)
}

/// Detect format from extension and import.
pub fn import_file(
    path: &std::path::Path,
    route_id: &str,
    name: Option<&str>,
    spacing_m: f64,
    grade_window_m: f64,
) -> Result<RouteModel, ImportError> {
    let data = std::fs::read(path)?;
    let display_name = name.unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(route_id)
    });
    match path.extension().and_then(|e| e.to_str()) {
        Some("gpx") | Some("GPX") => import_gpx(&data, route_id, display_name, spacing_m, grade_window_m),
        Some("tcx") | Some("TCX") => {
            let raw = parse_tcx_stub(&data)?;
            if raw.is_empty() {
                return Err(ImportError::NoPoints);
            }
            let resampled = resample_route(&raw, spacing_m);
            let smoothed = smooth_elevation(&resampled);
            let graded = compute_grades(&smoothed, grade_window_m);
            RouteModel::new(route_id, display_name, graded).map_err(ImportError::from)
        }
        Some("fit") | Some("FIT") => Err(ImportError::UnsupportedFormat(
            "FIT import stubbed — use GPX for now".into(),
        )),
        ext => Err(ImportError::UnsupportedFormat(format!(
            "unknown extension: {:?}",
            ext
        ))),
    }
}

/// Re-export for tests.
pub type RawPoint = RoutePoint;
