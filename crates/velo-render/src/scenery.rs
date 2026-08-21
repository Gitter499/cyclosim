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
        // rock so the corridor doesn't read as one repeated asset.
        match i % 7 {
            3 => {
                let lateral = (5.5 + j * 2.0) * side;
                push_bush(
                    &mut verts,
                    (east + px * lateral) as f32,
                    up as f32,
                    (north + pz * lateral) as f32,
                    0.6 + j as f32 * 0.5,
                    i,
                );
            }
            5 => {
                let lateral = (6.0 + j * 8.0) * side;
                push_rock(
                    &mut verts,
                    (east + px * lateral) as f32,
                    up as f32,
                    (north + pz * lateral) as f32,
                    0.4 + j as f32 * 0.4,
                    i,
                );
            }
            _ => {
                // Clear of the road band + shoulder (3 m + fringe).
                let lateral = (8.0 + j * 9.0) * side;
                push_tree(
                    &mut verts,
                    (east + px * lateral) as f32,
                    up as f32,
                    (north + pz * lateral) as f32,
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
    let trunk = [0.36, 0.25, 0.16];
    // Vary canopy green per tree so a row doesn't read as copy-paste.
    let g = 0.30 + (seed % 5) as f32 * 0.05;
    let canopy = [0.12, g, 0.14];

    let tw = 0.16;
    let th = h * 0.35;
    // Trunk: crossed vertical quads.
    quad(
        v,
        [x - tw, y, z],
        [x + tw, y, z],
        [x + tw, y + th, z],
        [x - tw, y + th, z],
        trunk,
    );
    quad(
        v,
        [x, y, z - tw],
        [x, y, z + tw],
        [x, y + th, z + tw],
        [x, y + th, z - tw],
        trunk,
    );
    // Canopy: crossed triangles forming a conifer silhouette.
    let cw = h * 0.40;
    tri(
        v,
        [x - cw, y + th, z],
        [x + cw, y + th, z],
        [x, y + h, z],
        canopy,
    );
    tri(
        v,
        [x, y + th, z - cw],
        [x, y + th, z + cw],
        [x, y + h, z],
        canopy,
    );
}

/// Low crossed dome of foliage; slightly yellower than the tree canopy.
fn push_bush(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32) {
    let g = 0.34 + (seed % 4) as f32 * 0.04;
    let leaf = [0.18, g, 0.12];
    let w = h * 1.3;
    // Crossed squat trapezoids (narrow top) read as a rounded shrub.
    trapezoid(v, x, y, z, w, h, 0.55, leaf, true);
    trapezoid(v, x, y, z, w, h, 0.55, leaf, false);
}

/// Squat gray crossed boulder.
fn push_rock(v: &mut Vec<SceneVertex>, x: f32, y: f32, z: f32, h: f32, seed: u32) {
    let shade = 0.42 + (seed % 3) as f32 * 0.06;
    let stone = [shade, shade, shade * 1.04];
    let w = h * 1.4;
    trapezoid(v, x, y, z, w, h, 0.45, stone, true);
    trapezoid(v, x, y, z, w, h, 0.45, stone, false);
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
    v.push(SceneVertex { position: a, color });
    v.push(SceneVertex { position: b, color });
    v.push(SceneVertex { position: c, color });
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
