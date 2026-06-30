use velo_core::RoutePoint;

/// Compute smoothed grade (rise/run) over a sliding distance window.
pub fn compute_grades(points: &[RoutePoint], window_m: f64) -> Vec<RoutePoint> {
    if points.is_empty() {
        return Vec::new();
    }
    let half = window_m / 2.0;
    let mut out = points.to_vec();

    for i in 0..out.len() {
        let center_d = out[i].distance_m;
        let d0 = center_d - half;
        let d1 = center_d + half;

        let elev0 = interpolate_elev(&out, d0);
        let elev1 = interpolate_elev(&out, d1);
        let rise = elev1 - elev0;
        let run = (d1 - d0).max(1.0);
        out[i].grade = (rise / run).clamp(-0.30, 0.30);
    }

    out
}

fn interpolate_elev(points: &[RoutePoint], distance_m: f64) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    if distance_m <= points[0].distance_m {
        return points[0].elevation_m;
    }
    let last = points.last().unwrap();
    if distance_m >= last.distance_m {
        return last.elevation_m;
    }
    let idx = points
        .partition_point(|p| p.distance_m <= distance_m)
        .saturating_sub(1);
    let a = &points[idx];
    let b = &points[(idx + 1).min(points.len() - 1)];
    if (b.distance_m - a.distance_m).abs() < f64::EPSILON {
        return a.elevation_m;
    }
    let t = (distance_m - a.distance_m) / (b.distance_m - a.distance_m);
    a.elevation_m + t * (b.elevation_m - a.elevation_m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_climb_grade() {
        let pts: Vec<RoutePoint> = (0..=10)
            .map(|i| RoutePoint {
                distance_m: i as f64 * 10.0,
                lat: 46.0,
                lon: 6.0,
                elevation_m: 400.0 + i as f64,
                grade: 0.0,
            })
            .collect();
        let graded = compute_grades(&pts, 20.0);
        // 1m rise per 10m run ≈ 10%
        assert!(graded[5].grade > 0.08 && graded[5].grade < 0.12);
    }
}
