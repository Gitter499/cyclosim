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
                push_tree(
                    &mut verts,
                    e as f32,
                    ground_y(e, n),
                    n as f32,
                    2.8 + j as f32 * 2.2,
                    i,
                );
            }
        }
        i += 1;
        d += 30.0 + j * 22.0;
    }
    verts
}

fn push_tree(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32) {
    // Bark desaturated dark; canopy two-tone — cool-shaded base grading to a
    // warm sun-lit apex, baked into vertex colors (game-graphics skill §3).
    let bark = [0.32, 0.26, 0.20];
    let g = 0.30 + (seed % 5) as f32 * 0.04;
    let shade = [0.09, g * 0.72, 0.14];
    let lit = [0.30, (g * 1.30 + 0.12).min(0.66), 0.22];

    // Seeded yaw so rows don't read as copy-paste axis-aligned billboards.
    let yaw = (seed % 8) as f32 * 0.3927; // π/8 steps
    let (c, s) = (yaw.cos(), yaw.sin());
    let d1 = [c, s];
    let d2 = [-s, c];

    let tw = 0.16;
    let th = h * 0.35;
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
