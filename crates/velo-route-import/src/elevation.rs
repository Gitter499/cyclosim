use velo_core::RoutePoint;

/// Fill missing elevations by linear interpolation; apply light moving average.
pub fn smooth_elevation(points: &[RoutePoint]) -> Vec<RoutePoint> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<RoutePoint> = points.to_vec();

    // Fill zeros/missing with interpolation along distance
    let has_ele: Vec<bool> = out.iter().map(|p| p.elevation_m != 0.0).collect();
    if has_ele.iter().any(|&h| h) && has_ele.iter().any(|&h| !h) {
        for i in 0..out.len() {
            if has_ele[i] {
                continue;
            }
            let prev = (0..i).rev().find(|&j| has_ele[j]);
            let next = (i + 1..out.len()).find(|&j| has_ele[j]);
            out[i].elevation_m = match (prev, next) {
                (Some(p), Some(n)) => {
                    let a = &out[p];
                    let b = &out[n];
                    let t = (out[i].distance_m - a.distance_m) / (b.distance_m - a.distance_m);
                    a.elevation_m + t * (b.elevation_m - a.elevation_m)
                }
                (Some(p), None) => out[p].elevation_m,
                (None, Some(n)) => out[n].elevation_m,
                _ => 0.0,
            };
        }
    }

    // 3-point moving average
    if out.len() >= 3 {
        let raw: Vec<f64> = out.iter().map(|p| p.elevation_m).collect();
        for i in 1..out.len() - 1 {
            out[i].elevation_m = (raw[i - 1] + raw[i] + raw[i + 1]) / 3.0;
        }
    }

    out
}
