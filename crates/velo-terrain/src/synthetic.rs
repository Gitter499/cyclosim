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
        let d = (pe - east).hypot(pn - north);
        if d < best_dist {
            best_dist = d;
            best_elev = p.elevation_m as f32;
        }
    }
    best_elev
}

/// Procedural earth-tone texture (RGBA8).
/// Elevation-, slope-, and route-aware terrain texture (Tier A placeholder
/// until splats): grass gradient with noise, rocky steeps, high-altitude
/// lightening, and an asphalt road band following the route corridor.
///
/// Returns (rgba, width, height) supersampled `texels_per_cell`× beyond the
/// heightfield grid so the ground doesn't look like colored cells.
pub fn terrain_texture(
    hf: &Heightfield,
    route: &RouteModel,
    texels_per_cell: usize,
) -> (Vec<u8>, u32, u32) {
    const ROAD_HALF_WIDTH_M: f64 = 3.0;
    const ROAD_EDGE_M: f64 = 1.5;

    // Stay under conservative GPU texture limits (8192 per axis).
    const MAX_TEXTURE_DIM: usize = 8192;
    let mut ss = texels_per_cell.clamp(1, 8);
    while ss > 1 && (hf.cols * ss > MAX_TEXTURE_DIM || hf.rows * ss > MAX_TEXTURE_DIM) {
        ss -= 1;
    }
    let w = (hf.cols * ss).max(4).min(MAX_TEXTURE_DIM);
    let h = (hf.rows * ss).max(4).min(MAX_TEXTURE_DIM);
    let texel_m = hf.cell_m / ss as f64;

    // Route polyline resampled into a spatial hash for fast distance lookup.
    // Each sample also carries its arc length so road markings (dashed
    // centerline) can follow the direction of travel.
    let bucket_m = (ROAD_HALF_WIDTH_M + ROAD_EDGE_M).max(hf.cell_m);
    let mut buckets: std::collections::HashMap<(i32, i32), Vec<(f64, f64, f64)>> =
        std::collections::HashMap::new();
    let step = (bucket_m / 2.0).max(1.0);
    let total = route.total_distance_m();
    let mut d = 0.0;
    while d <= total {
        let (east, _, north) = route.position_enu_at(d);
        let key = ((east / bucket_m).floor() as i32, (north / bucket_m).floor() as i32);
        buckets.entry(key).or_default().push((east, north, d));
        d += step;
    }
    // (perpendicular distance to route, arc length at the nearest sample)
    let route_dist = |east: f64, north: f64| -> (f64, f64) {
        let bx = (east / bucket_m).floor() as i32;
        let bz = (north / bucket_m).floor() as i32;
        let mut best = f64::MAX;
        let mut best_arc = 0.0;
        for dx in -1..=1 {
            for dz in -1..=1 {
                if let Some(pts) = buckets.get(&(bx + dx, bz + dz)) {
                    for &(pe, pn, arc) in pts {
                        let dist = (pe - east).hypot(pn - north);
                        if dist < best {
                            best = dist;
                            best_arc = arc;
                        }
                    }
                }
            }
        }
        (best, best_arc)
    };

    let elev_at = |col_f: f64, row_f: f64| -> f32 {
        let c = (col_f.floor() as usize).min(hf.cols - 1);
        let r = (row_f.floor() as usize).min(hf.rows - 1);
        hf.elevations[r * hf.cols + c]
    };
    let (mut min_e, mut max_e) = (f32::MAX, f32::MIN);
    for &e in &hf.elevations {
        min_e = min_e.min(e);
        max_e = max_e.max(e);
    }
    let range = (max_e - min_e).max(1.0);

    let mut rgba = vec![0u8; w * h * 4];
    for row in 0..h {
        for col in 0..w {
            let col_f = (col as f64 / ss as f64).min(hf.cols as f64 - 1.0);
            let row_f = (row as f64 / ss as f64).min(hf.rows as f64 - 1.0);
            let east = hf.origin_east_m + col_f * hf.cell_m;
            let north = hf.origin_north_m + row_f * hf.cell_m;

            let e = elev_at(col_f, row_f);
            let e_dx = elev_at((col_f + 1.0).min(hf.cols as f64 - 1.0), row_f);
            let e_dz = elev_at(col_f, (row_f + 1.0).min(hf.rows as f64 - 1.0));
            let slope = (((e_dx - e).abs() + (e_dz - e).abs()) as f64 / hf.cell_m).min(1.0) as f32;
            let alt = (e - min_e) / range;

            // Three-octave value noise for ground variation.
            let n1 = ((east * 0.11).sin() * (north * 0.13).cos()) as f32;
            let n2 = ((east * 0.031 + 1.7).sin() * (north * 0.027 + 0.4).cos()) as f32;
            let n3 = ((east * 0.47 + 0.9).sin() * (north * 0.53 + 2.1).cos()) as f32;
            let noise = n1 * 0.5 + n2 * 0.32 + n3 * 0.18; // -1..1

            // Grass base, gently drying with altitude.
            let mut r = 64.0 + alt * 26.0 + noise * 14.0;
            let mut g = 122.0 - alt * 10.0 + noise * 18.0;
            let mut b = 48.0 + alt * 10.0 + noise * 9.0;

            // Broad meadow patches drift toward a sunnier yellow-green.
            if n2 > 0.45 {
                let p = ((n2 - 0.45) * 2.2).min(1.0);
                r += 18.0 * p;
                g += 8.0 * p;
                b -= 6.0 * p;
            }

            // Steep faces turn rocky.
            let rockiness = ((slope - 0.25) * 2.5).clamp(0.0, 1.0);
            if rockiness > 0.0 {
                r = r + (128.0 - r) * rockiness;
                g = g + (118.0 - g) * rockiness;
                b = b + (104.0 - b) * rockiness;
            }

            // Road band along the route, with painted markings.
            let (dist, arc) = route_dist(east, north);
            if dist < ROAD_HALF_WIDTH_M + ROAD_EDGE_M {
                let asphalt = 70.0 + noise * 5.0;
                if dist <= ROAD_HALF_WIDTH_M {
                    r = asphalt;
                    g = asphalt;
                    b = asphalt + 4.0;

                    // Painted markings only when the bake can resolve them —
                    // at coarse texels a "line" floods the whole road, so
                    // skip (distant roads don't show markings anyway).
                    if texel_m <= 0.8 {
                        let line_w = 0.35;
                        let edge_c = ROAD_HALF_WIDTH_M - 0.45;
                        let on_edge = (dist - edge_c).abs() < line_w / 2.0;
                        // Dashed centerline: 6 m painted, 6 m gap.
                        let on_center = dist < line_w / 2.0 && (arc % 12.0) < 6.0;
                        if on_edge || on_center {
                            let paint = 205.0 + noise * 8.0;
                            r = paint;
                            g = paint;
                            b = paint - 8.0;
                        }
                    }
                } else {
                    // Dirt shoulder blend.
                    let t = ((dist - ROAD_HALF_WIDTH_M) / ROAD_EDGE_M).clamp(0.0, 1.0) as f32;
                    let (sr, sg, sb) = (124.0, 104.0, 74.0);
                    r = sr + (r - sr) * t;
                    g = sg + (g - sg) * t;
                    b = sb + (b - sb) * t;
                }
            }

            let i = (row * w + col) * 4;
            rgba[i] = r.clamp(0.0, 255.0) as u8;
            rgba[i + 1] = g.clamp(0.0, 255.0) as u8;
            rgba[i + 2] = b.clamp(0.0, 255.0) as u8;
            rgba[i + 3] = 255;
        }
    }
    (rgba, w as u32, h as u32)
}

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
