use velo_core::RoutePoint;

/// Resample route to roughly fixed spacing along the path.
pub fn resample_route(points: &[RoutePoint], spacing_m: f64) -> Vec<RoutePoint> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 || spacing_m <= 0.0 {
        return points.to_vec();
    }

    let total = points.last().unwrap().distance_m;
    let mut out = Vec::new();
    let mut seg = 0usize;

    let mut d = 0.0;
    while d <= total + f64::EPSILON {
        while seg + 1 < points.len() && points[seg + 1].distance_m < d {
            seg += 1;
        }
        let a = &points[seg];
        let b = &points[(seg + 1).min(points.len() - 1)];
        let span = b.distance_m - a.distance_m;
        let t = if span.abs() < f64::EPSILON {
            0.0
        } else {
            ((d - a.distance_m) / span).clamp(0.0, 1.0)
        };
        out.push(RoutePoint {
            distance_m: d,
            lat: a.lat + t * (b.lat - a.lat),
            lon: a.lon + t * (b.lon - a.lon),
            elevation_m: a.elevation_m + t * (b.elevation_m - a.elevation_m),
            grade: 0.0,
        });
        d += spacing_m;
    }

    if out.last().map(|p| p.distance_m) != Some(total) {
        out.push(points.last().unwrap().clone());
        out.last_mut().unwrap().distance_m = total;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use velo_core::haversine_m;

    #[test]
    fn spacing_produces_expected_count() {
        let pts = vec![
            RoutePoint {
                distance_m: 0.0,
                lat: 46.0,
                lon: 6.0,
                elevation_m: 100.0,
                grade: 0.0,
            },
            RoutePoint {
                distance_m: 100.0,
                lat: 46.001,
                lon: 6.0,
                elevation_m: 110.0,
                grade: 0.0,
            },
        ];
        let resampled = resample_route(&pts, 10.0);
        assert!(resampled.len() >= 10);
        assert!((resampled.last().unwrap().distance_m - 100.0).abs() < 0.01);
    }

    #[test]
    fn haversine_sanity() {
        let d = haversine_m(46.0, 6.0, 46.001, 6.0);
        assert!(d > 100.0 && d < 120.0);
    }
}
