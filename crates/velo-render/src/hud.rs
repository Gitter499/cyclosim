use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{MultisampleState, Queue, TextureFormat};

const LINE_HEIGHT_PX: f32 = 22.0;
const MARGIN_PX: f32 = 16.0;
const PANEL_PAD_PX: f32 = 10.0;

/// Backdrop quad rect + fill color, in NDC. Keeps HUD text legible over any
/// scene (sky, snow, bright terrain).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PanelUniform {
    rect: [f32; 4],
    color: [f32; 4],
}

const PANEL_SHADER: &str = r#"
struct PanelUniform {
    rect: vec4<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0) var<uniform> panel: PanelUniform;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let use_x1 = vi == 1u || vi == 3u || vi == 4u;
    let use_y1 = vi == 2u || vi == 4u || vi == 5u;
    let x = select(panel.rect.x, panel.rect.z, use_x1);
    let y = select(panel.rect.y, panel.rect.w, use_y1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return panel.color;
}
"#;

/// Stats overlay drawn over the 3D view.
#[derive(Debug, Clone, Default)]
pub struct HudSnapshot {
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub speed_mps: f64,
    pub distance_m: f64,
    pub elapsed_s: f64,
    pub grade: f64,
    pub mode: &'static str,
    /// Active structured-workout interval label (M5).
    pub workout_interval: Option<String>,
    /// Resolved ERG target for the current interval (None = free ride).
    pub workout_target_w: Option<f64>,
    /// Shown when Tier B 3D Tiles mode is active (ToS attribution).
    pub attribution: Option<String>,
}

impl HudSnapshot {
    fn format_line(label: &str, value: Option<f64>, suffix: &str) -> String {
        match value {
            Some(v) => format!("{label}: {:.0} {suffix}", v),
            None => format!("{label}: —"),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let speed_kmh = self.speed_mps * 3.6;
        let mins = (self.elapsed_s / 60.0).floor() as u32;
        let secs = (self.elapsed_s % 60.0).floor() as u32;
        let mut lines = Vec::new();
        if let Some(interval) = &self.workout_interval {
            let target = match self.workout_target_w {
                Some(w) => format!("{:.0} W", w),
                None => "Free ride".into(),
            };
            lines.push(format!("Interval: {interval}  Target: {target}"));
        }
        lines.extend([
            format!("Mode: {}  Grade: {:.1}%", self.mode, self.grade * 100.0),
            Self::format_line("Power", self.power_w, "W"),
            Self::format_line("Cadence", self.cadence_rpm, "rpm"),
            Self::format_line("HR", self.heart_rate_bpm, "bpm"),
            format!("Speed: {:.1} km/h", speed_kmh),
            format!("Distance: {:.0} m", self.distance_m),
            format!("Time: {mins:02}:{secs:02}"),
        ]);
        if let Some(attr) = &self.attribution {
            lines.push(attr.clone());
        }
        lines
    }
}

pub struct HudRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache, // kept alive for Viewport
    viewport: Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    buffer: Buffer,
    panel_pipeline: wgpu::RenderPipeline,
    panel_uniform: wgpu::Buffer,
    panel_bind_group: wgpu::BindGroup,
}

impl HudRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: TextureFormat) -> Self {
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut text_atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer = TextRenderer::new(
            &mut text_atlas,
            device,
            MultisampleState::default(),
            None,
        );
        let metrics = Metrics::new(18.0, LINE_HEIGHT_PX);
        let buffer = Buffer::new(&mut font_system, metrics);

        let (panel_pipeline, panel_uniform, panel_bind_group) =
            Self::create_panel(device, format);

        Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            text_atlas,
            text_renderer,
            buffer,
            panel_pipeline,
            panel_uniform,
            panel_bind_group,
        }
    }

    fn create_panel(
        device: &wgpu::Device,
        format: TextureFormat,
    ) -> (wgpu::RenderPipeline, wgpu::Buffer, wgpu::BindGroup) {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud-panel-shader"),
            source: wgpu::ShaderSource::Wgsl(PANEL_SHADER.into()),
        });

        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud-panel-uniform"),
            size: std::mem::size_of::<PanelUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hud-panel-bind-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("hud-panel-bind-group"),
            layout: &bind_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud-panel-pipeline-layout"),
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud-panel-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        (pipeline, uniform, bind_group)
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &Queue,
        hud: &HudSnapshot,
        width: u32,
        height: u32,
    ) -> Result<(), glyphon::PrepareError> {
        self.viewport.update(
            queue,
            Resolution {
                width,
                height,
            },
        );

        let text = hud.lines().join("\n");
        self.buffer.set_size(
            &mut self.font_system,
            Some(width as f32),
            Some(height as f32),
        );
        self.buffer.set_text(
            &mut self.font_system,
            &text,
            Attrs::new().family(Family::Monospace),
            Shaping::Advanced,
        );
        self.buffer
            .shape_until_scroll(&mut self.font_system, false);

        // Measure the shaped text so the block anchors to the bottom-left
        // whatever the line count (workout + attribution lines vary).
        let mut line_count = 0u32;
        let mut max_line_w = 0.0f32;
        for run in self.buffer.layout_runs() {
            line_count += 1;
            max_line_w = max_line_w.max(run.line_w);
        }
        let block_h = line_count as f32 * LINE_HEIGHT_PX;
        let top = (height as f32 - MARGIN_PX - block_h).max(0.0);

        self.write_panel(
            queue,
            width as f32,
            height as f32,
            MARGIN_PX - PANEL_PAD_PX,
            top - PANEL_PAD_PX,
            MARGIN_PX + max_line_w + PANEL_PAD_PX,
            top + block_h + PANEL_PAD_PX,
        );

        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: MARGIN_PX,
                top,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: top as i32,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: Color::rgb(235, 240, 245),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn write_panel(
        &self,
        queue: &Queue,
        width: f32,
        height: f32,
        x0_px: f32,
        y0_px: f32,
        x1_px: f32,
        y1_px: f32,
    ) {
        let to_ndc_x = |px: f32| px / width * 2.0 - 1.0;
        let to_ndc_y = |px: f32| 1.0 - px / height * 2.0;
        let uniform = PanelUniform {
            rect: [
                to_ndc_x(x0_px),
                to_ndc_y(y0_px),
                to_ndc_x(x1_px),
                to_ndc_y(y1_px),
            ],
            color: [0.03, 0.05, 0.08, 0.78],
        };
        queue.write_buffer(&self.panel_uniform, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError> {
        pass.set_pipeline(&self.panel_pipeline);
        pass.set_bind_group(0, &self.panel_bind_group, &[]);
        pass.draw(0..6, 0..1);
        self.text_renderer
            .render(&self.text_atlas, &self.viewport, pass)
    }

    pub fn trim(&mut self) {
        self.text_atlas.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workout_interval_line_prepended_when_active() {
        let hud = HudSnapshot {
            workout_interval: Some("Warmup".into()),
            workout_target_w: Some(137.5),
            mode: "ERG",
            ..Default::default()
        };
        let lines = hud.lines();
        assert!(lines[0].contains("Warmup"));
        assert!(lines[0].contains("138 W"));
    }
}
