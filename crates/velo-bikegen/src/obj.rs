//! Minimal OBJ → colored-triangle converter for the vendored CC0 bike mesh
//! (Quaternius "LowPoly Public Transport", assets/quaternius_bicycle.obj).
//!
//! Parses `v` positions and `f` faces (any arity, fan-triangulated), tracks
//! `usemtl` for per-part tinting, and bakes soft sun/hemisphere lighting into
//! vertex colors per face (game-graphics skill §2) so the unlit bike pipeline
//! still shows low-poly facets. No vt/vn support — the source mesh's normals
//! are recomputed from the transformed triangles.

/// One corner-expanded triangle mesh: positions with matching colors.
pub struct ObjMesh {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 3]>,
}

/// Convert the bicycle OBJ into renderer space: forward = +X (front wheel at
/// +0.5, rear at -0.5, matching the placeholder's wheelbase convention),
/// up = +Y with tires touching y = 0.
pub fn bike_mesh_from_obj(text: &str, frame_color: [f32; 3]) -> ObjMesh {
    // Source-space constants measured from the vendored mesh: ground plane
    // at y = -0.95, wheel centers at z = -1.68 (front) / 2.25 (rear).
    const GROUND_Y: f32 = -0.95;
    const FWD_MID: f32 = -0.285; // midpoint of (-z) wheel centers
    const SCALE: f32 = 0.2545; // wheelbase 3.93 units -> 1.0 m

    let color_for = |material: &str| -> [f32; 3] {
        match material {
            "Bike" => frame_color,
            "Handle" => [0.16, 0.16, 0.18],
            "Wheel" => [0.10, 0.10, 0.12],
            // Saddle exports as an unnamed Blender material.
            m if m.starts_with("Material") => [0.12, 0.12, 0.14],
            _ => [
                frame_color[0] * 0.85,
                frame_color[1] * 0.85,
                frame_color[2] * 0.85,
            ],
        }
    };

    let mut src_positions: Vec<[f32; 3]> = Vec::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut current: [f32; 3] = frame_color;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("v ") {
            let mut it = rest.split_whitespace();
            let (x, y, z) = (
                it.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0),
                it.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0),
                it.next().and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0),
            );
            // Rotate Blender's -Z-forward into +X-forward, recenter the
            // wheelbase midpoint on the origin, drop the ground to y = 0.
            src_positions.push([
                (-z - FWD_MID) * SCALE,
                (y - GROUND_Y) * SCALE,
                x * SCALE,
            ]);
        } else if let Some(rest) = line.strip_prefix("usemtl ") {
            current = color_for(rest.trim());
        } else if let Some(rest) = line.strip_prefix("f ") {
            let idx: Vec<usize> = rest
                .split_whitespace()
                .filter_map(|tok| {
                    tok.split('/')
                        .next()
                        .and_then(|v| v.parse::<isize>().ok())
                        .map(|v| {
                            if v < 0 {
                                (src_positions.len() as isize + v) as usize
                            } else {
                                (v - 1) as usize
                            }
                        })
                })
                .collect();
            for k in 1..idx.len().saturating_sub(1) {
                let tri = [idx[0], idx[k], idx[k + 1]];
                if tri.iter().any(|&i| i >= src_positions.len()) {
                    continue;
                }
                let (a, b, c) = (
                    src_positions[tri[0]],
                    src_positions[tri[1]],
                    src_positions[tri[2]],
                );
                let shaded = shade(current, a, b, c);
                positions.extend_from_slice(&[a, b, c]);
                colors.extend_from_slice(&[shaded, shaded, shaded]);
            }
        }
    }
    ObjMesh { positions, colors }
}

/// Bake soft directional + hemisphere light into a face color so the unlit
/// pipeline shows facets. Deliberately gentle: the bike yaws with the route
/// at runtime while this bake is static, so direction error must stay subtle.
fn shade(base: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let mut n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 1e-8 {
        n = [n[0] / len, n[1] / len, n[2] / len];
    } else {
        n = [0.0, 1.0, 0.0];
    }
    // Same sun bearing as the terrain shader, normalized.
    let sun = [0.4787, 0.7447, -0.3723_f32];
    let ndl = (n[0] * sun[0] + n[1] * sun[1] + n[2] * sun[2]).max(0.0);
    let l = (0.70 + 0.26 * ndl + 0.08 * n[1]).clamp(0.0, 1.15);
    [base[0] * l, base[1] * l, base[2] * l]
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const BIKE_OBJ: &str = include_str!("../assets/quaternius_bicycle.obj");

    #[test]
    fn bike_obj_converts_to_renderer_space() {
        let mesh = bike_mesh_from_obj(BIKE_OBJ, [0.8, 0.2, 0.2]);
        assert!(!mesh.positions.is_empty());
        assert_eq!(mesh.positions.len(), mesh.colors.len());
        assert_eq!(mesh.positions.len() % 3, 0);
        let (mut min, mut max) = ([f32::MAX; 3], [f32::MIN; 3]);
        for p in &mesh.positions {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        // Tires on the ground, wheelbase ~1.0 centered on the origin.
        assert!(min[1] > -0.01 && min[1] < 0.05, "ground: {:?}", min);
        assert!(max[1] > 0.6 && max[1] < 1.0, "height: {:?}", max);
        assert!(min[0] > -0.85 && max[0] < 0.85, "length: {min:?} {max:?}");
    }
}
