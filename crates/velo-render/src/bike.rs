//! Foreground bike glTF pass (chase-camera relative).

use std::fs;
use std::path::Path;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use velo_bikegen::AnchorTransform;
use velo_cesium::decode_gltf_bytes;
use wgpu::util::DeviceExt;

use crate::scene::ChaseCamera;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BikeGpuVertex {
    position: [f32; 3],
    color: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct BikeUniforms {
    mvp: [[f32; 4]; 4],
}

struct GpuBikeMesh {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    index_count: u32,
}

pub struct BikeScene {
    mesh: GpuBikeMesh,
    bind_group: wgpu::BindGroup,
    uniforms: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    anchor: AnchorTransform,
}

impl BikeScene {
    pub fn from_gltf_path(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        scene_bind_layout: &wgpu::BindGroupLayout,
        path: &Path,
        anchor: AnchorTransform,
    ) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| e.to_string())?;
        Self::from_gltf_bytes(device, format, scene_bind_layout, &bytes, anchor)
    }

    pub fn from_gltf_bytes(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        scene_bind_layout: &wgpu::BindGroupLayout,
        bytes: &[u8],
        anchor: AnchorTransform,
    ) -> Result<Self, String> {
        let decoded = decode_gltf_bytes(bytes, "bike").map_err(|e| e.to_string())?;
        let tint = [0.72, 0.22, 0.18];
        let vertices: Vec<BikeGpuVertex> = decoded
            .vertices
            .iter()
            .map(|v| BikeGpuVertex {
                position: v.position,
                color: tint,
            })
            .collect();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bike-vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("bike-indices"),
            contents: bytemuck::cast_slice(&decoded.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bike-uniforms"),
            size: std::mem::size_of::<BikeUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bike-bind-group"),
            layout: scene_bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bike-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/bike.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bike-pipeline-layout"),
            bind_group_layouts: &[scene_bind_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BikeGpuVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bike-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            mesh: GpuBikeMesh {
                vertex_buffer,
                index_buffer,
                index_count: decoded.indices.len() as u32,
            },
            bind_group,
            uniforms,
            pipeline,
            anchor,
        })
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        queue: &wgpu::Queue,
        aspect: f32,
        camera: &ChaseCamera,
        rider: Vec3,
        forward: Vec3,
    ) {
        let view_proj = camera.view_proj_at(aspect, rider + Vec3::Y * 1.5, forward);
        let model = bike_model_matrix(rider, forward, self.anchor);
        let mvp = view_proj * model;
        let uniforms = BikeUniforms {
            mvp: mvp.to_cols_array_2d(),
        };
        queue.write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        pass.set_index_buffer(
            self.mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint32,
        );
        pass.draw_indexed(0..self.mesh.index_count, 0, 0..1);
    }
}

fn bike_model_matrix(rider: Vec3, forward: Vec3, anchor: AnchorTransform) -> Mat4 {
    let fwd = if forward.length_squared() < 1e-6 {
        Vec3::Z
    } else {
        forward.normalize()
    };
    let yaw = fwd.x.atan2(fwd.z) - std::f32::consts::FRAC_PI_2;
    let orient = Mat4::from_rotation_y(yaw);
    let at_rider = Mat4::from_translation(rider);
    at_rider * orient * anchor.to_mat4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bike_model_faces_forward() {
        let m = bike_model_matrix(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), AnchorTransform::default());
        let fwd = m.transform_vector3(Vec3::X);
        assert!(fwd.z.abs() > 0.5);
    }
}
