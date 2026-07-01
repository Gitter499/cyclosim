//! wgpu renderer — terrain mesh or flat ground plane, chase camera, HUD overlay.

mod bike;
mod capture;
mod hud;
mod scene;
mod terrain;
mod tiles;

use std::path::Path;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use thiserror::Error;
use velo_bikegen::AnchorTransform;
use velo_cesium::{TilesSession, ViewCorridor};
use wgpu::util::DeviceExt;

pub use bike::BikeScene;
pub use capture::{bgra_to_rgba, FramebufferRgba, PNG_MAGIC};

pub use hud::{HudRenderer, HudSnapshot};
pub use scene::{ChaseCamera, GroundMesh, SceneVertex};
pub use terrain::{forward_from_enu, TerrainScene};
pub use tiles::TilesScene;

/// Rider position in local ENU for chase camera along a loaded route.
#[derive(Debug, Clone, Copy)]
pub struct RouteFollow {
    pub east: f64,
    pub up: f64,
    pub north: f64,
    pub forward: Vec3,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("renderer not initialized")]
    NotInitialized,
    #[error("wgpu: {0}")]
    Wgpu(String),
    #[error("hud: {0}")]
    Hud(String),
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    mvp: [[f32; 4]; 4],
}

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    grid_pipeline: wgpu::RenderPipeline,
    fill_pipeline: wgpu::RenderPipeline,
    scene_bind_layout: wgpu::BindGroupLayout,
    scene_bind_group: wgpu::BindGroup,
    scene_uniforms: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    grid_vertex_count: u32,
    fill_vertex_start: u32,
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    camera: ChaseCamera,
    hud: HudRenderer,
    terrain: Option<TerrainScene>,
    tiles: Option<TilesScene>,
    tiles_session: Option<TilesSession>,
    tiles_mode: bool,
    tiles_attribution: String,
    rider_z: f32,
    bike: Option<BikeScene>,
}

impl Renderer {
    /// Create a renderer from a raw CAMetalLayer pointer (Swift passes `Unmanaged.passRetained`).
    #[cfg(target_os = "macos")]
    pub fn from_metal_layer(
        layer_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        if layer_ptr.is_null() {
            return Err(RenderError::Wgpu("null CAMetalLayer".into()));
        }

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::METAL,
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::CoreAnimationLayer(layer_ptr))
                .map_err(|e| RenderError::Wgpu(e.to_string()))?
        };

        Self::init(instance, surface, width, height)
    }

    #[cfg(not(target_os = "macos"))]
    pub fn from_metal_layer(
        _layer_ptr: *mut std::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let _ = (width, height);
        Err(RenderError::Wgpu(
            "Metal layer rendering is macOS-only".into(),
        ))
    }

    fn init(
        instance: wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Result<Self, RenderError> {
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .ok_or_else(|| RenderError::Wgpu("no adapter".into()))?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("velo-render"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| RenderError::Wgpu(e.to_string()))?;

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/scene.wgsl").into()),
        });

        let scene_uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene-uniforms"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene-bind-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene-bind-group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniforms.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("scene-pipeline-layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SceneVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
        };

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let color_target = wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        };

        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("grid-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout.clone()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(color_target.clone())],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                ..Default::default()
            },
            depth_stencil: depth_stencil.clone(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let fill_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("fill-pipeline"),
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
                targets: &[Some(color_target)],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let mesh = GroundMesh::grid(40, 2.0);
        let total = mesh.vertices.len() as u32;
        let fill_vertex_start = total - 6;
        let grid_vertex_count = fill_vertex_start;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ground-vertices"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let (depth_texture, depth_view) = create_depth(&device, config.width, config.height);
        let hud = HudRenderer::new(&device, &queue, format);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            grid_pipeline,
            fill_pipeline,
            scene_bind_layout: bind_layout,
            scene_bind_group,
            scene_uniforms,
            vertex_buffer,
            grid_vertex_count,
            fill_vertex_start,
            depth_texture,
            depth_view,
            camera: ChaseCamera::default(),
            hud,
            terrain: None,
            tiles: None,
            tiles_session: None,
            tiles_mode: false,
            tiles_attribution: String::new(),
            rider_z: 0.0,
            bike: None,
        })
    }

    /// Load rider bike glTF for the foreground-object pass.
    pub fn load_bike_gltf(
        &mut self,
        gltf_path: &Path,
        anchor: AnchorTransform,
    ) -> Result<(), RenderError> {
        let bike = BikeScene::from_gltf_path(
            &self.device,
            self.config.format,
            &self.scene_bind_layout,
            gltf_path,
            anchor,
        )
        .map_err(|e| RenderError::Wgpu(e))?;
        self.bike = Some(bike);
        Ok(())
    }

    pub fn clear_bike(&mut self) {
        self.bike = None;
    }

    pub fn has_bike(&self) -> bool {
        self.bike.is_some()
    }

    /// Load textured terrain from a route pack directory.
    pub fn load_terrain_pack(&mut self, pack_dir: &Path) -> Result<(), RenderError> {
        let pack = velo_terrain::TerrainPack::load_from_dir(pack_dir)
            .map_err(|e| RenderError::Wgpu(e.to_string()))?;
        let terrain = TerrainScene::from_pack(
            &self.device,
            &self.queue,
            self.config.format,
            &self.scene_bind_layout,
            &pack,
        );
        self.terrain = Some(terrain);
        Ok(())
    }

    pub fn clear_terrain(&mut self) {
        self.terrain = None;
    }

    pub fn has_terrain(&self) -> bool {
        self.terrain.is_some()
    }

    /// Enable or disable Tier B 3D Tiles overlay (online-only during ride).
    pub fn set_tiles_mode(&mut self, enabled: bool) {
        if enabled && self.tiles_session.is_none() {
            let session =
                TilesSession::online_default().unwrap_or_else(|_| TilesSession::synthetic());
            self.tiles_attribution = session.attribution().text.clone();
            self.tiles_session = Some(session);
        }
        if !enabled {
            self.tiles_session = None;
            self.tiles = None;
            self.tiles_attribution.clear();
        }
        self.tiles_mode = enabled;
    }

    pub fn tiles_mode(&self) -> bool {
        self.tiles_mode
    }

    pub fn tiles_attribution(&self) -> &str {
        &self.tiles_attribution
    }

    pub fn tiles_provider_status(&self) -> String {
        velo_cesium::tiles_provider_status()
    }

    pub fn tiles_last_error(&self) -> Option<String> {
        self.tiles_session
            .as_ref()
            .and_then(|s| s.last_error().map(str::to_string))
    }

    /// Recreate the online tile session (after API keys change in Settings).
    pub fn refresh_tiles_session(&mut self) {
        if self.tiles_mode {
            self.tiles_session = None;
            self.tiles = None;
            self.set_tiles_mode(true);
        }
    }

    /// Refresh visible tile meshes for the current rider position (ENU origin frame).
    pub fn update_tiles_view(&mut self, lat: f64, lon: f64, radius_m: f64) {
        if !self.tiles_mode {
            return;
        }
        let Some(session) = self.tiles_session.as_mut() else {
            return;
        };
        let view = ViewCorridor { lat, lon, radius_m };
        if session.tick(view).is_ok() {
            let meshes = session.meshes();
            if !meshes.is_empty() {
                self.tiles = Some(TilesScene::from_meshes(
                    &self.device,
                    &self.queue,
                    self.config.format,
                    &self.scene_bind_layout,
                    meshes,
                ));
            }
        }
    }

    pub fn render_frame(
        &mut self,
        hud: &HudSnapshot,
        distance_m: f64,
        follow: Option<RouteFollow>,
        steer_yaw_rad: f32,
    ) -> Result<(), RenderError> {
        self.draw_scene(hud, distance_m, follow, steer_yaw_rad)?;
        Ok(())
    }

    fn draw_scene(
        &mut self,
        hud: &HudSnapshot,
        distance_m: f64,
        follow: Option<RouteFollow>,
        steer_yaw_rad: f32,
    ) -> Result<(), RenderError> {
        self.rider_z = distance_m as f32 * 0.05;

        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| RenderError::Wgpu(e.to_string()))?;
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;
        let mvp = self.scene_mvp(aspect, follow, steer_yaw_rad);
        let uniforms = SceneUniforms {
            mvp: mvp.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.scene_uniforms, 0, bytemuck::bytes_of(&uniforms));

        self.hud
            .prepare(
                &self.device,
                &self.queue,
                hud,
                self.config.width,
                self.config.height,
            )
            .map_err(|e| RenderError::Hud(e.to_string()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("velo-frame"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.45,
                            g: 0.62,
                            b: 0.82,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(terrain) = &self.terrain {
                terrain.draw(&mut pass, &self.scene_bind_group);
            } else {
                pass.set_bind_group(0, &self.scene_bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.set_pipeline(&self.fill_pipeline);
                pass.draw(self.fill_vertex_start..self.fill_vertex_start + 6, 0..1);
                pass.set_pipeline(&self.grid_pipeline);
                pass.draw(0..self.grid_vertex_count, 0..1);
            }

            if let Some(tiles) = &self.tiles {
                tiles.draw(&mut pass, &self.scene_bind_group);
            }

            if let Some(bike) = &self.bike {
                let (rider, forward) = rider_pose(follow, self.rider_z);
                bike.draw(&mut pass, &self.queue, aspect, &self.camera, rider, forward);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.hud
                .render(&mut pass)
                .map_err(|e| RenderError::Hud(e.to_string()))?;
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        self.hud.trim();
        frame.present();
        Ok(())
    }

    fn scene_mvp(
        &self,
        aspect: f32,
        follow: Option<RouteFollow>,
        steer_yaw_rad: f32,
    ) -> glam::Mat4 {
        if let Some(f) = follow {
            let rider = Vec3::new(f.east as f32, f.up as f32 + 1.5, f.north as f32);
            let forward = scene::apply_steer_yaw_for_camera(f.forward, steer_yaw_rad);
            self.camera.view_proj_at(aspect, rider, forward)
        } else {
            self.camera.view_proj(aspect, self.rider_z, steer_yaw_rad)
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            let (depth_texture, depth_view) = create_depth(&self.device, width, height);
            self.depth_texture = depth_texture;
            self.depth_view = depth_view;
        }
    }

    pub fn render_frame_legacy(
        &mut self,
        hud: &HudSnapshot,
        distance_m: f64,
    ) -> Result<(), RenderError> {
        self.render_frame(hud, distance_m, None, 0.0)
    }

    /// Grab the current framebuffer as raw RGBA8 pixels from the framebuffer.
    pub fn capture_framebuffer_rgba(
        &mut self,
        hud: &HudSnapshot,
        distance_m: f64,
        follow: Option<RouteFollow>,
        steer_yaw_rad: f32,
    ) -> Result<FramebufferRgba, RenderError> {
        self.rider_z = distance_m as f32 * 0.05;

        let frame = self
            .surface
            .get_current_texture()
            .map_err(|e| RenderError::Wgpu(e.to_string()))?;
        let texture = &frame.texture;
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;
        let mvp = self.scene_mvp(aspect, follow, steer_yaw_rad);
        let uniforms = SceneUniforms {
            mvp: mvp.to_cols_array_2d(),
        };
        self.queue
            .write_buffer(&self.scene_uniforms, 0, bytemuck::bytes_of(&uniforms));

        self.hud
            .prepare(
                &self.device,
                &self.queue,
                hud,
                self.config.width,
                self.config.height,
            )
            .map_err(|e| RenderError::Hud(e.to_string()))?;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("velo-capture"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.45,
                            g: 0.62,
                            b: 0.82,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(terrain) = &self.terrain {
                terrain.draw(&mut pass, &self.scene_bind_group);
            } else {
                pass.set_bind_group(0, &self.scene_bind_group, &[]);
                pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                pass.set_pipeline(&self.fill_pipeline);
                pass.draw(self.fill_vertex_start..self.fill_vertex_start + 6, 0..1);
                pass.set_pipeline(&self.grid_pipeline);
                pass.draw(0..self.grid_vertex_count, 0..1);
            }

            if let Some(tiles) = &self.tiles {
                tiles.draw(&mut pass, &self.scene_bind_group);
            }

            if let Some(bike) = &self.bike {
                let (rider, forward) = rider_pose(follow, self.rider_z);
                bike.draw(&mut pass, &self.queue, aspect, &self.camera, rider, forward);
            }
        }

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hud"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.hud
                .render(&mut pass)
                .map_err(|e| RenderError::Hud(e.to_string()))?;
        }

        let width = self.config.width;
        let height = self.config.height;
        let bytes_per_row = align_to_256(width * 4);
        let buffer_size = bytes_per_row as u64 * height as u64;

        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));
        self.hud.trim();

        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = sender.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        receiver
            .recv()
            .map_err(|_| RenderError::Wgpu("readback channel".into()))?
            .map_err(|e| RenderError::Wgpu(e.to_string()))?;

        let mapped = slice.get_mapped_range();
        let mut pixels = vec![0u8; (width * height * 4) as usize];
        for row in 0..height {
            let src_start = row as usize * bytes_per_row as usize;
            let dst_start = row as usize * width as usize * 4;
            let row_bgra = &mapped[src_start..src_start + (width * 4) as usize];
            bgra_to_rgba(
                row_bgra,
                &mut pixels[dst_start..dst_start + (width * 4) as usize],
            );
        }
        drop(mapped);
        readback.unmap();

        frame.present();

        Ok(FramebufferRgba {
            width,
            height,
            pixels,
        })
    }
}

fn rider_pose(follow: Option<RouteFollow>, rider_z: f32) -> (Vec3, Vec3) {
    if let Some(f) = follow {
        (
            Vec3::new(f.east as f32, f.up as f32, f.north as f32),
            f.forward,
        )
    } else {
        (Vec3::new(0.0, 0.0, rider_z), Vec3::Z)
    }
}

fn align_to_256(n: u32) -> u32 {
    (n + 255) & !255
}

fn create_depth(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    (depth_texture, depth_view)
}

/// Headless placeholder for CI on non-macOS hosts.
pub fn headless_ok() -> bool {
    true
}
