//! Tier A roadside scenery: crossed-billboard trees, bushes, and rocks
//! along the route corridor.
//!
//! Deterministic (seeded by placement index) so eval renders stay
//! byte-stable. Geometry is world-space `SceneVertex` triangles drawn with
//! the existing colored-vertex pipeline; crossed planes read from every
//! camera angle the same way the placeholder rider does.

use velo_core::RouteModel;

use crate::scene::SceneVertex;

/// Build scenery geometry along both sides of the route.
pub fn tree_vertices_for_route(route: &RouteModel) -> Vec<SceneVertex> {
    let mut verts = Vec::new();
    let total = route.total_distance_m();
    // Elevation span for altitude-driven species/palette shifts: valleys mix
    // round-crowned deciduous trees, high climbs go pure conifer with cooler
    // foliage — so an alpine route reads different from a flat one.
    let (mut min_elev, mut max_elev) = (f64::MAX, f64::MIN);
    for p in &route.points {
        min_elev = min_elev.min(p.elevation_m);
        max_elev = max_elev.max(p.elevation_m);
    }
    let elev_range = (max_elev - min_elev).max(1.0);
    let origin_elev = route.meta.origin.elevation_m;
    let mut i: u32 = 0;
    let mut d = 15.0;
    while d < total {
        let (east, up, north) = route.position_enu_at(d);
        let (ea, _, na) = route.position_enu_at((d + 5.0).min(total));
        let (dx, dz) = (ea - east, na - north);
        let len = (dx * dx + dz * dz).sqrt().max(1e-3);
        // Perpendicular to travel; alternate sides.
        let (px, pz) = (dz / len, -dx / len);
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        // Deterministic 0..1 jitter from the placement index.
        let j = (i.wrapping_mul(2_654_435_761) >> 8) as f64 / (1u64 << 24) as f64;

        // Mostly trees, with the occasional bush (nearer the road) and
        // rock so the corridor doesn't read as one repeated asset. Bases
        // sit on the baked terrain surface: route elevation + the same
        // deterministic undulation velo-terrain adds (keeps feet planted
        // and blob shadows visible instead of buried).
        let ground_y = |e: f64, n: f64| -> f32 {
            (up + (e * 0.02).sin() * (n * 0.015).cos() * 2.0) as f32
        };
        match i % 7 {
            3 => {
                let lateral = (5.5 + j * 2.0) * side;
                let (e, n) = (east + px * lateral, north + pz * lateral);
                push_bush(
                    &mut verts,
                    e as f32,
                    ground_y(e, n),
                    n as f32,
                    0.6 + j as f32 * 0.5,
                    i,
                );
            }
            5 => {
                let lateral = (6.0 + j * 8.0) * side;
                let (e, n) = (east + px * lateral, north + pz * lateral);
                push_rock(
                    &mut verts,
                    e as f32,
                    ground_y(e, n),
                    n as f32,
                    0.4 + j as f32 * 0.4,
                    i,
                );
            }
            _ => {
                // Clear of the road band + shoulder (3 m + fringe).
                let lateral = (8.0 + j * 9.0) * side;
                let (e, n) = (east + px * lateral, north + pz * lateral);
                // Real-tree scale (8-14 m): at 2.8-5 m they vanished to a
                // few pixels past 80 m and the corridor read as empty.
                let alt01 =
                    (((up + origin_elev) - min_elev) / elev_range).clamp(0.0, 1.0) as f32;
                push_tree(
                    &mut verts,
                    e as f32,
                    ground_y(e, n),
                    n as f32,
                    8.0 + j as f32 * 6.0,
                    i,
                    alt01,
                );
            }
        }
        i += 1;
        d += 21.0 + j * 15.0;
    }
    verts
}

/// Near-field ground detail: small grass tufts and flower speckles beside
/// the road. The terrain texture's texels are coarsest right where the
/// camera looks (near field), so billboards carry the detail instead
/// (game-graphics skill §3). Deterministic like everything else.
pub fn tuft_vertices_for_route(route: &RouteModel) -> Vec<SceneVertex> {
    let mut verts = Vec::new();
    let total = route.total_distance_m();
    let mut i: u32 = 0;
    let mut d = 6.0;
    while d < total {
        let j = (i.wrapping_mul(2_654_435_761) >> 8) as f64 / (1u64 << 24) as f64;
        let j2 = (i.wrapping_mul(0x9E37_79B9) >> 8) as f64 / (1u64 << 24) as f64;
        let (east, up, north) = route.position_enu_at(d);
        let (ea, _, na) = route.position_enu_at((d + 5.0).min(total));
        let (dx, dz) = (ea - east, na - north);
        let len = (dx * dx + dz * dz).sqrt().max(1e-3);
        let (px, pz) = (dz / len, -dx / len);
        let side = if i % 2 == 0 { 1.0 } else { -1.0 };
        let lateral = (4.0 + j2 * 5.0) * side;
        let (e, n) = (east + px * lateral, north + pz * lateral);
        let y = (up + (e * 0.02).sin() * (n * 0.015).cos() * 2.0) as f32;
        let (x, z) = (e as f32, n as f32);
        if i % 9 == 3 {
            // Flower cluster: three tiny bright quads just above the grass.
            let warm = i % 18 == 3;
            let col = if warm {
                [0.92, 0.72, 0.18]
            } else {
                [0.88, 0.88, 0.82]
            };
            for (ox, oz) in [(0.0_f32, 0.0_f32), (0.28, 0.14), (-0.2, 0.24)] {
                let s = 0.07;
                quad(
                    &mut verts,
                    [x + ox - s, y + 0.16, z + oz - s],
                    [x + ox + s, y + 0.16, z + oz + s],
                    [x + ox + s, y + 0.30, z + oz + s],
                    [x + ox - s, y + 0.30, z + oz - s],
                    col,
                );
            }
        } else {
            // Grass tuft: crossed triangles, darker than the ground so it
            // reads as a speckle, height/tone jittered.
            let h = 0.28 + j as f32 * 0.30;
            let w = h * 0.55;
            let g = 0.13 + (i % 4) as f32 * 0.02;
            let col = [0.06, g, 0.05];
            let tip = [0.12, g + 0.08, 0.08];
            for (dx1, dz1) in [(1.0_f32, 0.35_f32), (-0.35, 1.0)] {
                tri_grad(
                    &mut verts,
                    [x - w * dx1, y, z - w * dz1],
                    [x + w * dx1, y, z + w * dz1],
                    [x, y + h, z],
                    col,
                    col,
                    tip,
                );
            }
        }
        i += 1;
        d += 3.2 + j * 4.0;
    }
    verts
}

/// Distant layered ridge silhouettes flanking the route (Firewatch-style
/// value layering, game-graphics skill §1): two haze-softened walls per side
/// that follow the valley's elevation profile. At 800-1500 m the shared
/// atmosphere fog renders them as pale blue-green cutouts, so the world no
/// longer ends at the terrain corridor's edge.
pub fn ridge_vertices_for_route(route: &RouteModel) -> Vec<SceneVertex> {
    let mut verts = Vec::new();
    let total = route.total_distance_m();
    const STEP: f64 = 250.0;
    // (lateral offset m, base height, peak amplitude, bottom color, top color)
    // Close enough that fog leaves a readable silhouette (0.5-0.7 haze, not
    // 0.85+), with dark forested bases so value structure survives the mix.
    // Colors are linear (sRGB target brightens them ~1 stop) and the haze
    // mix floors anything past ~1 km near-white, so bases must be very dark
    // and layers close enough (in-view depth 550-1100 m) to keep a readable
    // two-value silhouette under the sky.
    let layers: [(f64, f32, f32, [f32; 3], [f32; 3]); 2] = [
        (420.0, 55.0, 110.0, [0.10, 0.18, 0.12], [0.20, 0.30, 0.22]),
        (900.0, 150.0, 220.0, [0.14, 0.20, 0.26], [0.22, 0.31, 0.38]),
    ];
    for (li, (lat, base, amp, cb, ct)) in layers.iter().enumerate() {
        for side in [-1.0_f64, 1.0] {
            // Ridge crest polyline; extrapolated past both ends so the
            // valley doesn't stop abruptly at the start/finish.
            let mut pts: Vec<([f32; 3], f32)> = Vec::new();
            let n = (total / STEP) as i64;
            for k in -4..=(n + 4) {
                let d = (k as f64 * STEP).clamp(0.0, total);
                let (e, up, nn) = route.position_enu_at(d);
                let (e2, _, n2) = route.position_enu_at((d + 10.0).min(total).max(10.0));
                let (dx, dz) = (e2 - e, n2 - nn);
                let len = (dx * dx + dz * dz).sqrt().max(1e-3);
                let (px, pz) = (dz / len, -dx / len);
                let over = k as f64 * STEP - d; // nonzero only beyond the ends
                let (fx, fz) = (dx / len, dz / len);
                let ex = e + px * lat * side + fx * over;
                let ez = nn + pz * lat * side + fz * over;
                let seed = ((k + 64) as u32)
                    .wrapping_add((li as u32) << 9)
                    .wrapping_add(((side > 0.0) as u32) << 17);
                let h1 = (seed.wrapping_mul(2_654_435_761) >> 8) as f32 / (1u32 << 24) as f32;
                // Two sine octaves + hash: distinct peaks and saddles instead
                // of a mesa-flat crest.
                let wave = ((k as f32) * 0.9 + li as f32 * 1.3).sin() * 0.5 + 0.5;
                let wave2 = ((k as f32) * 2.3 + li as f32 * 0.7).sin() * 0.5 + 0.5;
                let top = up as f32
                    + base
                    + amp * (0.22 + 0.78 * (0.40 * wave + 0.30 * wave2 + 0.30 * h1));
                pts.push(([ex as f32, up as f32 - 120.0, ez as f32], top));
            }
            for w in pts.windows(2) {
                let (b0, t0) = (w[0].0, w[0].1);
                let (b1, t1) = (w[1].0, w[1].1);
                tri_grad(&mut verts, b0, b1, [b1[0], t1, b1[2]], *cb, *cb, *ct);
                tri_grad(&mut verts, b0, [b1[0], t1, b1[2]], [b0[0], t0, b0[2]], *cb, *ct, *ct);
            }
        }
    }
    verts
}

fn push_tree(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32, alt01: f32) {
    // Bark desaturated dark; canopy two-tone — cool-shaded base grading to a
    // warm sun-lit apex, baked into vertex colors (game-graphics skill §3).
    let bark = [0.32, 0.26, 0.20];
    let g = 0.30 + (seed % 5) as f32 * 0.04;
    // Foliage cools and darkens with altitude (subalpine conifers).
    let cool = alt01 * 0.6;
    let shade = [0.09 * (1.0 - cool), g * (0.72 - 0.18 * cool), 0.14 + 0.05 * cool];
    let lit = [
        0.30 * (1.0 - cool * 0.7),
        (g * (1.30 - 0.25 * cool) + 0.12).min(0.66),
        0.22 + 0.04 * cool,
    ];

    // Seeded yaw so rows don't read as copy-paste axis-aligned billboards.
    let yaw = (seed % 8) as f32 * 0.3927; // π/8 steps
    let (c, s) = (yaw.cos(), yaw.sin());
    let d1 = [c, s];
    let d2 = [-s, c];

    // Valleys mix in round-crowned deciduous trees; conifers own the climbs.
    let deciduous = alt01 < 0.45 && seed % 3 == 0;
    let h = if deciduous { h * 0.80 } else { h };

    let tw = 0.08 + h * 0.02;
    let th = if deciduous { h * 0.42 } else { h * 0.35 };
    for d in [d1, d2] {
        quad(
            v,
            [x - tw * d[0], y, z - tw * d[1]],
            [x + tw * d[0], y, z + tw * d[1]],
            [x + tw * d[0], y + th, z + tw * d[1]],
            [x - tw * d[0], y + th, z - tw * d[1]],
            bark,
        );
    }
    if deciduous {
        // Round crown: crossed diamond billboards (narrow → wide → narrow),
        // warmer than conifer foliage.
        let shade = [0.14, g * 0.80, 0.10];
        let lit = [0.36, (g * 1.35 + 0.14).min(0.68), 0.18];
        let cw = h * 0.34;
        let mid_y = y + th + (h - th) * 0.45;
        for d in [d1, d2] {
            tri_grad(
                v,
                [x - cw * d[0], mid_y, z - cw * d[1]],
                [x + cw * d[0], mid_y, z + cw * d[1]],
                [x, y + th * 0.9, z],
                shade,
                shade,
                shade,
            );
            tri_grad(
                v,
                [x - cw * d[0], mid_y, z - cw * d[1]],
                [x + cw * d[0], mid_y, z + cw * d[1]],
                [x, y + h, z],
                shade,
                shade,
                lit,
            );
        }
    } else {
        // Canopy: crossed triangles, shade at the skirt → lit at the apex.
        let cw = h * 0.40;
        for d in [d1, d2] {
            tri_grad(
                v,
                [x - cw * d[0], y + th, z - cw * d[1]],
                [x + cw * d[0], y + th, z + cw * d[1]],
                [x, y + h, z],
                shade,
                shade,
                lit,
            );
        }
    }

    // Contact blob shadow, offset away from the sun (grounds the tree).
    blob_shadow(v, x - 0.5, y, z + 0.4, h * 0.36, [0.15, 0.24, 0.13]);
}

/// Flat octagon disc just above the ground — the classic blob shadow.
fn blob_shadow(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, r: f32, color: [f32; 3]) {
    let yy = y + 0.03;
    for i in 0..8u32 {
        let a0 = i as f32 * std::f32::consts::TAU / 8.0;
        let a1 = (i + 1) as f32 * std::f32::consts::TAU / 8.0;
        tri(
            v,
            [x, yy, z],
            [x + r * a0.cos(), yy, z + r * a0.sin()],
            [x + r * a1.cos(), yy, z + r * a1.sin()],
            color,
        );
    }
}

/// Low crossed dome of foliage; slightly yellower than the tree canopy.
fn push_bush(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32) {
    let g = 0.34 + (seed % 4) as f32 * 0.04;
    let shade = [0.13, g * 0.75, 0.11];
    let lit = [0.26, (g * 1.25 + 0.10).min(0.62), 0.16];
    let w = h * 1.3;
    // Crossed squat trapezoids (narrow top) read as a rounded shrub.
    trapezoid_grad(v, x, y, z, w, h, 0.55, shade, lit, true);
    trapezoid_grad(v, x, y, z, w, h, 0.55, shade, lit, false);
    blob_shadow(v, x - 0.2, y, z + 0.15, w * 0.9, [0.15, 0.24, 0.13]);
}

/// Squat gray crossed boulder.
fn push_rock(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32) {
    let base = 0.42 + (seed % 3) as f32 * 0.06;
    let shade = [base * 0.72, base * 0.74, base * 0.82];
    let lit = [base * 1.18, base * 1.15, base * 1.10];
    let w = h * 1.4;
    trapezoid_grad(v, x, y, z, w, h, 0.45, shade, lit, true);
    trapezoid_grad(v, x, y, z, w, h, 0.45, shade, lit, false);
    blob_shadow(v, x - 0.15, y, z + 0.12, w * 0.85, [0.15, 0.24, 0.13]);
}

/// One vertical trapezoid (bottom width `w`, top width `w * top_frac`)
/// centered at (x, z), in the XZ or ZX plane.
#[allow(clippy::too_many_arguments)]
fn trapezoid(
    v: &mut Vec<SceneVertex>,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    h: f32,
    top_frac: f32,
    color: [f32; 3],
    x_plane: bool,
) {
    let t = w * top_frac;
    if x_plane {
        quad(
            v,
            [x - w, y, z],
            [x + w, y, z],
            [x + t, y + h, z],
            [x - t, y + h, z],
            color,
        );
    } else {
        quad(
            v,
            [x, y, z - w],
            [x, y, z + w],
            [x, y + h, z + t],
            [x, y + h, z - t],
            color,
        );
    }
}

fn tri(v: &mut Vec<SceneVertex>, a: [f32; 3], b: [f32; 3], c: [f32; 3], color: [f32; 3]) {
    tri_grad(v, a, b, c, color, color, color);
}

fn tri_grad(
    v: &mut Vec<SceneVertex>,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    ca: [f32; 3],
    cb: [f32; 3],
    cc: [f32; 3],
) {
    v.push(SceneVertex {
        position: a,
        color: ca,
    });
    v.push(SceneVertex {
        position: b,
        color: cb,
    });
    v.push(SceneVertex {
        position: c,
        color: cc,
    });
}

/// Trapezoid with a bottom→top color gradient (baked two-tone lighting).
#[allow(clippy::too_many_arguments)]
fn trapezoid_grad(
    v: &mut Vec<SceneVertex>,
    x: f32,
    y: f32,
    z: f32,
    w: f32,
    h: f32,
    top_frac: f32,
    bottom: [f32; 3],
    top: [f32; 3],
    x_plane: bool,
) {
    let t = w * top_frac;
    let (a, b, c, d) = if x_plane {
        (
            [x - w, y, z],
            [x + w, y, z],
            [x + t, y + h, z],
            [x - t, y + h, z],
        )
    } else {
        (
            [x, y, z - w],
            [x, y, z + w],
            [x, y + h, z + t],
            [x, y + h, z - t],
        )
    };
    tri_grad(v, a, b, c, bottom, bottom, top);
    tri_grad(v, a, c, d, bottom, top, top);
}

fn quad(
    v: &mut Vec<SceneVertex>,
    a: [f32; 3],
    b: [f32; 3],
    c: [f32; 3],
    d: [f32; 3],
    color: [f32; 3],
) {
    tri(v, a, b, c, color);
    tri(v, a, c, d, color);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trees_are_deterministic() {
        let points: Vec<velo_core::RoutePoint> = (0..40)
            .map(|i| velo_core::RoutePoint {
                distance_m: i as f64 * 50.0,
                lat: 46.0 + i as f64 * 0.0004,
                lon: 6.0,
                elevation_m: 400.0 + i as f64,
                grade: 0.02,
            })
            .collect();
        let route = velo_core::RouteModel::new("t", "Test", points).unwrap();
        let a = tree_vertices_for_route(&route);
        let b = tree_vertices_for_route(&route);
        assert!(!a.is_empty());
        assert_eq!(a.len(), b.len());
        assert_eq!(a[0].position, b[0].position);
    }
}
