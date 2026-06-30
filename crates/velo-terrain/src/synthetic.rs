use velo_core::{lat_lon_to_local, RouteModel};

use crate::heightfield::Heightfield;

/// Build a synthetic DEM around the route corridor for offline dev/CI.
pub fn synthetic_heightfield_for_route(
    route: &RouteModel,
    corridor_m: f64,
    cell_m: f64,
) -> Heightfield {
    let half = corridor_m / 2.0;
    let mut min_e = f64::MAX;
    let mut max_e = f64::MIN;
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_z = f64::MAX;
    let mut max_z = f64::MIN;

    for p in &route.points {
        let (east, north) =
            lat_lon_to_local(route.meta.origin.lat, route.meta.origin.lon, p.lat, p.lon);
        min_e = min_e.min(p.elevation_m);
        max_e = max_e.max(p.elevation_m);
        min_x = min_x.min(east - half);
        max_x = max_x.max(east + half);
        min_z = min_z.min(north - half);
        max_z = max_z.max(north + half);
    }

    let cols = ((max_x - min_x) / cell_m).ceil() as usize + 1;
    let rows = ((max_z - min_z) / cell_m).ceil() as usize + 1;
    let base_elev = route.meta.origin.elevation_m as f32;

    let mut elevations = vec![base_elev; cols * rows];
    for row in 0..rows {
        for col in 0..cols {
            let east = min_x + col as f64 * cell_m;
            let north = min_z + row as f64 * cell_m;
            let elev = sample_route_elevation(route, east, north, base_elev);
            // Add gentle undulation for visual interest
            let undulate = ((east * 0.02).sin() * (north * 0.015).cos() * 2.0) as f32;
            elevations[row * cols + col] = elev + undulate;
        }
    }

    Heightfield {
        cols,
        rows,
        cell_m,
        origin_east_m: min_x,
        origin_north_m: min_z,
        elevations,
    }
}

fn sample_route_elevation(route: &RouteModel, east: f64, north: f64, fallback: f32) -> f32 {
    let mut best_dist = f64::MAX;
    let mut best_elev = fallback;
    for p in &route.points {
        let (pe, pn) = lat_lon_to_local(route.meta.origin.lat, route.meta.origin.lon, p.lat, p.lon);
        let d = (pe - east).hypot(pn - north) as f64;
        if d < best_dist {
            best_dist = d;
            best_elev = p.elevation_m as f32;
        }
    }
    best_elev
}

/// Procedural earth-tone texture (RGBA8).
pub fn procedural_texture(cols: usize, rows: usize) -> Vec<u8> {
    let w = cols.max(4);
    let h = rows.max(4);
    let mut rgba = vec![0u8; w * h * 4];
    for row in 0..h {
        for col in 0..w {
            let u = col as f32 / w as f32;
            let v = row as f32 / h as f32;
            let noise = ((u * 12.7 + v * 8.3).sin() * 0.5 + 0.5) * 20.0;
            let r = (90.0 + noise + v * 30.0) as u8;
            let g = (120.0 + noise * 0.8 + u * 20.0) as u8;
            let b = (50.0 + noise * 0.5) as u8;
            let i = (row * w + col) * 4;
            rgba[i] = r;
            rgba[i + 1] = g;
            rgba[i + 2] = b;
            rgba[i + 3] = 255;
        }
    }
    rgba
}
