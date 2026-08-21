//! Tier A roadside scenery: crossed-billboard trees along the route corridor.
//!
//! Deterministic (seeded by tree index) so eval renders stay byte-stable.
//! Geometry is world-space `SceneVertex` triangles drawn with the existing
//! colored-vertex pipeline; crossed planes read from every camera angle the
//! same way the placeholder rider does.

use velo_core::RouteModel;

use crate::scene::SceneVertex;

/// Build tree geometry along both sides of the route.
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
        // Deterministic 0..1 jitter from the tree index.
        let j = (i.wrapping_mul(2_654_435_761) >> 8) as f64 / (1u64 << 24) as f64;
        // Clear of the road band + shoulder (3 m + 1.5 m).
        let lateral = (8.0 + j * 9.0) * side;

        push_tree(
            &mut verts,
            (east + px * lateral) as f32,
            up as f32,
            (north + pz * lateral) as f32,
            2.8 + j as f32 * 2.2,
            i,
        );
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
