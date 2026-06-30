use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SceneVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

pub struct GroundMesh {
    pub vertices: Vec<SceneVertex>,
}

impl GroundMesh {
    /// Flat XZ grid centered on the rider path (+Z forward).
    pub fn grid(size: i32, spacing: f32) -> Self {
        let mut vertices = Vec::new();
        let half = size as f32 * spacing;
        let major_every = 5;

        for i in -size..=size {
            let pos = i as f32 * spacing;
            let major = i.rem_euclid(major_every) == 0;
            let color = if major {
                [0.35, 0.38, 0.42]
            } else {
                [0.22, 0.24, 0.28]
            };

            // Line along Z at fixed X
            vertices.push(SceneVertex {
                position: [pos, 0.0, -half],
                color,
            });
            vertices.push(SceneVertex {
                position: [pos, 0.0, half],
                color,
            });

            // Line along X at fixed Z
            vertices.push(SceneVertex {
                position: [-half, 0.0, pos],
                color,
            });
            vertices.push(SceneVertex {
                position: [half, 0.0, pos],
                color,
            });
        }

        // Solid ground fill (two triangles, dark green-gray)
        let ground = [0.12, 0.16, 0.13];
        let y = -0.01;
        vertices.extend_from_slice(&[
            SceneVertex {
                position: [-half, y, -half],
                color: ground,
            },
            SceneVertex {
                position: [half, y, -half],
                color: ground,
            },
            SceneVertex {
                position: [half, y, half],
                color: ground,
            },
            SceneVertex {
                position: [-half, y, -half],
                color: ground,
            },
            SceneVertex {
                position: [half, y, half],
                color: ground,
            },
            SceneVertex {
                position: [-half, y, half],
                color: ground,
            },
        ]);

        Self { vertices }
    }
}

pub struct ChaseCamera {
    pub eye_height: f32,
    pub behind: f32,
    pub look_ahead: f32,
}

impl Default for ChaseCamera {
    fn default() -> Self {
        Self {
            eye_height: 1.8,
            behind: 7.0,
            look_ahead: 18.0,
        }
    }
}

impl ChaseCamera {
    pub fn view_proj(&self, aspect: f32, rider_z: f32, steer_yaw_rad: f32) -> Mat4 {
        let rider = Vec3::new(0.0, 0.0, rider_z);
        let forward = apply_steer_yaw_for_camera(Vec3::Z, steer_yaw_rad);
        self.view_proj_at(aspect, rider, forward)
    }

    pub fn view_proj_at(&self, aspect: f32, rider: Vec3, forward: Vec3) -> Mat4 {
        let fwd = if forward.length_squared() < 1e-6 {
            Vec3::Z
        } else {
            forward.normalize()
        };
        let behind = -fwd * self.behind;
        let eye = rider + Vec3::Y * self.eye_height + behind;
        let target = rider + fwd * self.look_ahead + Vec3::Y * 0.5;
        let view = Mat4::look_at_rh(eye, target, Vec3::Y);
        let proj = Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.1, 2000.0);
        proj * view
    }
}

/// Rotate a forward vector around world Y for steering look offset.
pub fn apply_steer_yaw_for_camera(forward: Vec3, yaw_rad: f32) -> Vec3 {
    if yaw_rad.abs() < 1e-6 {
        return forward;
    }
    let rotated = glam::Quat::from_rotation_y(yaw_rad) * forward;
    if rotated.length_squared() < 1e-6 {
        forward
    } else {
        rotated.normalize()
    }
}
