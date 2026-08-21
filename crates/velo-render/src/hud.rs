//! Glyphon + quad HUD for the 3D view — capture/eval path.
//!
//! **Single HUD path:** live ride metrics are drawn by the Swift shell
//! (`RideHUDOverlay`). This renderer is disabled during normal riding
//! (`hud_draw_enabled = false` at init) and retained for screenshot/clip/eval
//! renders that need baked-in stats. Layout, type scale, zone palette, and
//! contrast rules follow `.claude/skills/hud-design/SKILL.md` — the same
//! design language as the SwiftUI overlay, approximated with text runs and
//! colored quads.

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextBounds, TextRenderer, TextAtlas, Viewport, Weight,
};
use wgpu::{MultisampleState, Queue, TextureFormat};

const MARGIN_PX: f32 = 18.0;
const PAD_PX: f32 = 12.0;

/// Type scale (px @ 1080p-ish; regions scale positions, not glyphs).
const HERO_SIZE: f32 = 58.0;
const METRIC_SIZE: f32 = 21.0;
const LABEL_SIZE: f32 = 12.0;
const SMALL_SIZE: f32 = 13.0;

/// Panel scrim — survives snow/sky per the contrast doctrine.
const PANEL_RGBA: [f32; 4] = [0.03, 0.05, 0.08, 0.78];
const TRACK_RGBA: [f32; 4] = [1.0, 1.0, 1.0, 0.12];
const TEXT_PRIMARY: Color = Color::rgb(235, 240, 245);
const TEXT_SECONDARY: Color = Color::rgb(168, 178, 188);

/// 3 s rolling display smoothing for power (universal head-unit standard).
const POWER_SMOOTH_S: f64 = 3.0;

/// Stats overlay input for one frame.
#[derive(Debug, Clone, Default)]
pub struct HudSnapshot {
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub speed_mps: f64,
    pub distance_m: f64,
    pub elapsed_s: f64,
    pub grade: f64,
    pub elevation_m: Option<f64>,
    pub mode: &'static str,
    pub workout_interval: Option<String>,
    pub workout_target_w: Option<f64>,
    pub interval_duration_s: Option<f64>,
    pub interval_elapsed_s: Option<f64>,
    /// Rider FTP for power-zone tinting (None → neutral zone color).
    pub ftp_w: Option<f64>,
    pub attribution: Option<String>,
    /// Upcoming interval name for the workout bar's "next" hint.
    pub workout_next_interval: Option<String>,
    /// Downsampled route elevations for the top elevation bar (empty → hidden).
    pub elevation_profile: Vec<f32>,
    /// Route length backing the elevation bar's rider-position dot.
    pub route_total_m: Option<f64>,
}

impl HudSnapshot {
    pub fn interval_fraction(&self) -> Option<f64> {
        let duration = self.interval_duration_s?;
        let elapsed = self.interval_elapsed_s?;
        if duration > 0.0 {
            Some((elapsed / duration).clamp(0.0, 1.0))
        } else {
            None
        }
    }

    pub fn interval_remaining_s(&self) -> Option<f64> {
        let duration = self.interval_duration_s?;
        let elapsed = self.interval_elapsed_s?;
        if duration > 0.0 {
            Some((duration - elapsed).max(0.0))
        } else {
            None
        }
    }

    /// Text summary for logs/eval JSON (not the rendered layout).
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
        lines.push(format!(
            "POWER {}  |  HR {}  |  CAD {}",
            self.power_w
                .map(|w| format!("{:.0} W", w))
                .unwrap_or_else(|| "—".into()),
            self.heart_rate_bpm
                .map(|b| format!("{:.0}", b))
                .unwrap_or_else(|| "—".into()),
            self.cadence_rpm
                .map(|c| format!("{:.0}", c))
                .unwrap_or_else(|| "—".into()),
        ));
        lines.push(format!(
            "Speed: {:.1} km/h  Dist: {:.0} m  Time: {mins:02}:{secs:02}",
            speed_kmh, self.distance_m
        ));
        match self.elevation_m {
            Some(e) => lines.push(format!("Elev: {:.0} m  Grade: {:.1}%", e, self.grade * 100.0)),
            None => lines.push(format!("Grade: {:.1}%", self.grade * 100.0)),
        }
        lines.push(format!("Mode: {}", self.mode));
        if let Some(attr) = &self.attribution {
            lines.push(attr.clone());
        }
        lines
    }
}

/// Coggan 7-zone palette (%FTP boundaries 55/75/90/105/120/150), tuned for
/// ~0.40 alpha tints under white text on the dark panel.
fn zone_color(power_w: f64, ftp_w: f64) -> [f32; 3] {
    let pct = if ftp_w > 0.0 { power_w / ftp_w * 100.0 } else { 0.0 };
    let hex = if pct < 55.0 {
        0x9AA5B1 // Z1 recovery — grey
    } else if pct < 75.0 {
        0x3D9BE9 // Z2 endurance — blue
    } else if pct < 90.0 {
        0x3FBE58 // Z3 tempo — green
    } else if pct < 105.0 {
        0xF5C542 // Z4 threshold — yellow
    } else if pct < 120.0 {
        0xF07F2E // Z5 vo2max — orange
    } else if pct < 150.0 {
        0xE43F4F // Z6 anaerobic — red
    } else {
        0xB05CE0 // Z7 neuromuscular — purple
    };
    [
        ((hex >> 16) & 0xFF) as f32 / 255.0,
        ((hex >> 8) & 0xFF) as f32 / 255.0,
        (hex & 0xFF) as f32 / 255.0,
    ]
}

// ---------------------------------------------------------------------------
// Quad batch pass (panels, zone block, gauges)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UiVertex {
    pos: [f32; 2],
    /// Pixel offset of this corner from the rect center.
    local: [f32; 2],
    /// Rect half-extents in pixels.
    half: [f32; 2],
    color: [f32; 4],
    /// Corner radius in pixels.
    radius: f32,
}

const MAX_QUADS: usize = 192;

/// Antialiased rounded-rectangle panels via a signed-distance field, so HUD
/// surfaces read as cards and pills instead of hard-edged slabs.
const QUAD_SHADER: &str = r#"
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) half_ext: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) radius: f32,
};

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) half_ext: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) radius: f32,
) -> VsOut {
    var out: VsOut;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.local = local;
    out.half_ext = half_ext;
    out.color = color;
    out.radius = radius;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let r = min(in.radius, min(in.half_ext.x, in.half_ext.y));
    let q = abs(in.local) - (in.half_ext - vec2<f32>(r, r));
    let d = length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
    let aa = 1.0 - smoothstep(-0.75, 0.75, d);
    return vec4<f32>(in.color.rgb, in.color.a * aa);
}
"#;

/// Text run roles, one glyphon buffer each (real type hierarchy).
enum Role {
    Hero,
    Metric,
    Label,
    Small,
}

struct Run {
    buffer: Buffer,
    left: f32,
    top: f32,
    color: Color,
}

pub struct HudRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    #[allow(dead_code)]
    cache: Cache, // kept alive for Viewport
    viewport: Viewport,
    text_atlas: TextAtlas,
    text_renderer: TextRenderer,
    runs: Vec<Run>,
    quad_pipeline: wgpu::RenderPipeline,
    quad_vertices: wgpu::Buffer,
    quad_count: usize,
    quads: Vec<UiVertex>,
    /// (elapsed_s, watts) ring for 3 s display smoothing.
    power_window: std::collections::VecDeque<(f64, f64)>,
}

impl HudRenderer {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: TextureFormat) -> Self {
        // Embed Inter (SIL OFL, fonts/LICENSE-Inter.txt) so both the shipped
        // app and headless eval renders use the same typeface everywhere.
        let mut font_system = FontSystem::new();
        font_system
            .db_mut()
            .load_font_data(include_bytes!("fonts/InterVariable.ttf").to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut text_atlas = TextAtlas::new(device, queue, &cache, format);
        let text_renderer =
            TextRenderer::new(&mut text_atlas, device, MultisampleState::default(), None);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hud-quad-shader"),
            source: wgpu::ShaderSource::Wgsl(QUAD_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("hud-quad-pipeline-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("hud-quad-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2, 1 => Float32x2, 2 => Float32x2,
                        3 => Float32x4, 4 => Float32
                    ],
                }],
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

        let quad_vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("hud-quad-vertices"),
            size: (MAX_QUADS * 6 * std::mem::size_of::<UiVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            text_atlas,
            text_renderer,
            runs: Vec::new(),
            quad_pipeline,
            quad_vertices,
            quad_count: 0,
            quads: Vec::new(),
            power_window: std::collections::VecDeque::new(),
        }
    }

    fn metrics_for(role: &Role) -> Metrics {
        match role {
            Role::Hero => Metrics::new(HERO_SIZE, HERO_SIZE * 1.02),
            Role::Metric => Metrics::new(METRIC_SIZE, METRIC_SIZE * 1.25),
            Role::Label => Metrics::new(LABEL_SIZE, LABEL_SIZE * 1.3),
            Role::Small => Metrics::new(SMALL_SIZE, SMALL_SIZE * 1.3),
        }
    }

    /// Shape a run, returning (index, width) so callers can align it.
    fn shape(&mut self, role: Role, text: &str, color: Color, width: f32, height: f32) -> (usize, f32) {
        // Per-role Inter weights: real type hierarchy instead of one mono face.
        // Numerals stay jitter-free because every live run is right-aligned
        // against a fixed edge (hud-design skill §3).
        let attrs = match role {
            Role::Hero => Attrs::new()
                .family(Family::Name("Inter Variable"))
                .weight(Weight::BOLD),
            Role::Metric => Attrs::new()
                .family(Family::Name("Inter Variable"))
                .weight(Weight::SEMIBOLD),
            Role::Label => Attrs::new()
                .family(Family::Name("Inter Variable"))
                .weight(Weight::BOLD),
            Role::Small => Attrs::new()
                .family(Family::Name("Inter Variable"))
                .weight(Weight::MEDIUM),
        };
        let mut buffer = Buffer::new(&mut self.font_system, Self::metrics_for(&role));
        buffer.set_size(&mut self.font_system, Some(width), Some(height));
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);
        let w = buffer
            .layout_runs()
            .map(|r| r.line_w)
            .fold(0.0f32, f32::max);
        self.runs.push(Run {
            buffer,
            left: 0.0,
            top: 0.0,
            color,
        });
        (self.runs.len() - 1, w)
    }

    fn place(&mut self, idx: usize, left: f32, top: f32) {
        self.runs[idx].left = left;
        self.runs[idx].top = top;
    }

    /// Rounded-rect panel; `radius` in px (half the height makes a pill).
    #[allow(clippy::too_many_arguments)]
    fn quad(
        &mut self,
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        radius: f32,
        color: [f32; 4],
        w: f32,
        h: f32,
    ) {
        if self.quads.len() / 6 >= MAX_QUADS {
            return;
        }
        let cx = (x0 + x1) / 2.0;
        let cy = (y0 + y1) / 2.0;
        let half = [(x1 - x0).abs() / 2.0, (y1 - y0).abs() / 2.0];
        let nx = |px: f32| px / w * 2.0 - 1.0;
        let ny = |py: f32| 1.0 - py / h * 2.0;
        let v = |x: f32, y: f32| UiVertex {
            pos: [nx(x), ny(y)],
            local: [x - cx, y - cy],
            half,
            color,
            radius,
        };
        self.quads.extend_from_slice(&[
            v(x0, y0),
            v(x1, y0),
            v(x0, y1),
            v(x1, y0),
            v(x1, y1),
            v(x0, y1),
        ]);
    }

    /// 3 s-smoothed display power (head-unit standard); also feeds zone tint.
    fn smoothed_power(&mut self, hud: &HudSnapshot) -> Option<f64> {
        if let Some(p) = hud.power_w {
            self.power_window.push_back((hud.elapsed_s, p));
        }
        while let Some(&(t, _)) = self.power_window.front() {
            if t < hud.elapsed_s - POWER_SMOOTH_S {
                self.power_window.pop_front();
            } else {
                break;
            }
        }
        if self.power_window.is_empty() {
            return None;
        }
        Some(
            self.power_window.iter().map(|&(_, p)| p).sum::<f64>()
                / self.power_window.len() as f64,
        )
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &Queue,
        hud: &HudSnapshot,
        width: u32,
        height: u32,
    ) -> Result<(), glyphon::PrepareError> {
        let w = width as f32;
        let h = height as f32;
        self.viewport.update(queue, Resolution { width, height });
        self.runs.clear();
        self.quads.clear();

        let display_power = self.smoothed_power(hud);

        // ---- Top strip: TIME · SPEED · DIST · GRADE (ambient context) ----
        let hrs = (hud.elapsed_s / 3600.0).floor() as u32;
        let mins = ((hud.elapsed_s % 3600.0) / 60.0).floor() as u32;
        let secs = (hud.elapsed_s % 60.0).floor() as u32;
        let time = if hrs > 0 {
            format!("{hrs}:{mins:02}:{secs:02}")
        } else {
            format!("{mins:02}:{secs:02}")
        };
        let strip = format!(
            "{time}   {:5.1} km/h   {:6.2} km   {:+5.1}%",
            hud.speed_mps * 3.6,
            hud.distance_m / 1000.0,
            hud.grade * 100.0,
        );
        let (strip_idx, strip_w) = self.shape(Role::Metric, &strip, TEXT_PRIMARY, w, h);
        let strip_x = (w - strip_w) / 2.0;
        let strip_y = MARGIN_PX;
        self.quad(
            strip_x - PAD_PX - 6.0,
            strip_y - 6.0,
            strip_x + strip_w + PAD_PX + 6.0,
            strip_y + METRIC_SIZE * 1.25 + 6.0,
            (METRIC_SIZE * 1.25 + 12.0) / 2.0,
            PANEL_RGBA,
            w,
            h,
        );
        self.place(strip_idx, strip_x, strip_y);

        // ---- Elevation bar (top, under the strip): route silhouette + dot ----
        // A map, not a metric (hud-design skill §1): low-contrast column fill,
        // one accent dot, never taller than ~40 px.
        if !hud.elevation_profile.is_empty() {
            if let Some(total) = hud.route_total_m.filter(|t| *t > 0.0) {
                let bar_w = (w * 0.34).clamp(240.0, 420.0);
                let bar_h = 34.0;
                let bar_x = (w - bar_w) / 2.0;
                let bar_y = MARGIN_PX + METRIC_SIZE * 1.25 + 6.0 + 8.0;
                self.quad(bar_x, bar_y, bar_x + bar_w, bar_y + bar_h, 10.0, PANEL_RGBA, w, h);

                let n = hud.elevation_profile.len();
                let (mut min_e, mut max_e) = (f32::MAX, f32::MIN);
                for &e in &hud.elevation_profile {
                    min_e = min_e.min(e);
                    max_e = max_e.max(e);
                }
                let range = (max_e - min_e).max(1.0);
                let pad = 6.0;
                let usable_h = bar_h - pad * 2.0 - 3.0;
                let col_w = (bar_w - pad * 2.0) / n as f32;
                let col_top = |e: f32| -> f32 {
                    let frac = (e - min_e) / range;
                    bar_y + bar_h - pad - (3.0 + frac * usable_h)
                };
                // Columns overlap slightly so the profile reads as one
                // continuous silhouette, not an equalizer.
                for (i, &e) in hud.elevation_profile.iter().enumerate() {
                    let x0 = bar_x + pad + i as f32 * col_w;
                    self.quad(
                        x0,
                        col_top(e),
                        x0 + col_w + 0.5,
                        bar_y + bar_h - pad,
                        0.0,
                        [1.0, 1.0, 1.0, 0.20],
                        w,
                        h,
                    );
                }

                // Rider position dot (radius = half-size → SDF circle).
                let p = (hud.distance_m / total).clamp(0.0, 1.0) as f32;
                let idx = ((p * (n - 1) as f32).round() as usize).min(n - 1);
                let cx = bar_x + pad + p * (bar_w - pad * 2.0);
                let cy = col_top(hud.elevation_profile[idx]);
                let r_dot = 4.5;
                self.quad(
                    cx - r_dot,
                    cy - r_dot,
                    cx + r_dot,
                    cy + r_dot,
                    r_dot,
                    [1.0, 1.0, 1.0, 0.95],
                    w,
                    h,
                );
            }
        }

        // ---- Primary block (bottom-left): hero power + CAD/HR row ----
        // Inner grid: everything (label, band, CAD/HR) shares the PAD_PX
        // inset; the zone band gets real breathing room around the numeral so
        // digits never touch its edges.
        let block_w = 248.0;
        let hero_h = HERO_SIZE * 1.02;
        let band_pad_y = 9.0;
        let band_h = hero_h + band_pad_y * 2.0;
        let label_gap = 6.0;
        let row_gap = 10.0;
        let row_h = METRIC_SIZE * 1.25 + LABEL_SIZE * 1.3;
        let block_h =
            PAD_PX + LABEL_SIZE * 1.3 + label_gap + band_h + row_gap + row_h + PAD_PX;
        let block_x = MARGIN_PX;
        let block_y = h - MARGIN_PX - block_h;
        self.quad(
            block_x,
            block_y,
            block_x + block_w,
            block_y + block_h,
            16.0,
            PANEL_RGBA,
            w,
            h,
        );

        let (pl_idx, _) = self.shape(Role::Label, "POWER · 3s", TEXT_SECONDARY, w, h);
        self.place(pl_idx, block_x + PAD_PX, block_y + PAD_PX);

        // Zone-tinted surface behind the hero numeral, on the inner grid.
        let ftp = hud.ftp_w.unwrap_or(0.0);
        let zone = display_power
            .map(|p| zone_color(p, ftp.max(1.0)))
            .unwrap_or([0.35, 0.38, 0.42]);
        let band_top = block_y + PAD_PX + LABEL_SIZE * 1.3 + label_gap;
        self.quad(
            block_x + PAD_PX,
            band_top,
            block_x + block_w - PAD_PX,
            band_top + band_h,
            10.0,
            [zone[0], zone[1], zone[2], 0.42],
            w,
            h,
        );

        // Fixed-slot right-aligned hero numeral + unit, both fully inside the
        // band with an inner margin (the unit must never straddle the edge).
        let hero_top = band_top + band_pad_y;
        let hero_text = match display_power {
            Some(p) => format!("{:>4.0}", p),
            None => "   —".into(),
        };
        let (unit_idx, unit_w) = self.shape(Role::Metric, "W", TEXT_SECONDARY, w, h);
        let unit_x = block_x + block_w - PAD_PX - 12.0 - unit_w;
        let (hero_idx, hero_w) = self.shape(Role::Hero, &hero_text, TEXT_PRIMARY, w, h);
        self.place(hero_idx, unit_x - 8.0 - hero_w, hero_top);
        // Baseline-align the unit with the numeral (both bottom-anchored).
        self.place(unit_idx, unit_x, hero_top + hero_h - METRIC_SIZE * 1.25);

        // Secondary row: CAD · HR (labels above values, one row).
        let row_top = band_top + band_h + row_gap;
        let half = (block_w - PAD_PX * 2.0) / 2.0;
        let cad = hud
            .cadence_rpm
            .map(|c| format!("{:>3.0}", c))
            .unwrap_or_else(|| "  —".into());
        let hr = hud
            .heart_rate_bpm
            .map(|b| format!("{:>3.0}", b))
            .unwrap_or_else(|| "  —".into());
        for (i, (label, value)) in [("CAD", cad), ("HR", hr)].into_iter().enumerate() {
            let cell_x = block_x + PAD_PX + i as f32 * half;
            let (li, _) = self.shape(Role::Label, label, TEXT_SECONDARY, w, h);
            self.place(li, cell_x, row_top);
            let (vi, _) = self.shape(Role::Metric, &value, TEXT_PRIMARY, w, h);
            self.place(vi, cell_x, row_top + LABEL_SIZE * 1.3);
        }

        // ---- Workout bar (bottom-center) ----
        if let Some(name) = hud.workout_interval.clone() {
            let bar_w = (w * 0.42).clamp(320.0, 560.0);
            let bar_h = 58.0;
            let bar_x = (w - bar_w) / 2.0;
            let bar_y = h - MARGIN_PX - bar_h;
            self.quad(bar_x, bar_y, bar_x + bar_w, bar_y + bar_h, 14.0, PANEL_RGBA, w, h);

            let target = match hud.workout_target_w {
                Some(t) => format!("{:>4.0} W", t),
                None => "Free".into(),
            };
            let remaining = hud
                .interval_remaining_s()
                .map(|r| {
                    format!("-{}:{:02}", (r / 60.0).floor() as u32, (r % 60.0).floor() as u32)
                })
                .unwrap_or_default();

            let (ni, name_w) = self.shape(Role::Small, &name, TEXT_PRIMARY, w, h);
            self.place(ni, bar_x + PAD_PX, bar_y + 8.0);
            if let Some(next) = hud.workout_next_interval.clone() {
                let (nx, _) = self.shape(
                    Role::Small,
                    &format!("next · {next}"),
                    TEXT_SECONDARY,
                    w,
                    h,
                );
                self.place(nx, bar_x + PAD_PX + name_w + 14.0, bar_y + 8.0);
            }
            let (ti, ti_w) = self.shape(Role::Small, &target, TEXT_PRIMARY, w, h);
            let (ri, ri_w) = self.shape(Role::Small, &remaining, TEXT_SECONDARY, w, h);
            self.place(ri, bar_x + bar_w - PAD_PX - ri_w, bar_y + 8.0);
            self.place(ti, bar_x + bar_w - PAD_PX - ri_w - 12.0 - ti_w, bar_y + 8.0);

            // Progress gauge: track + zone-of-target fill (quads, not ASCII).
            let g_y0 = bar_y + bar_h - 18.0;
            let g_y1 = bar_y + bar_h - 10.0;
            self.quad(bar_x + PAD_PX, g_y0, bar_x + bar_w - PAD_PX, g_y1, 4.0, TRACK_RGBA, w, h);
            if let Some(frac) = hud.interval_fraction() {
                let fill_zone = hud
                    .workout_target_w
                    .map(|t| zone_color(t, ftp.max(1.0)))
                    .unwrap_or([0.4, 0.6, 0.9]);
                let g_x1 = bar_x + PAD_PX + (bar_w - PAD_PX * 2.0) * frac as f32;
                self.quad(
                    bar_x + PAD_PX,
                    g_y0,
                    g_x1,
                    g_y1,
                    4.0,
                    [fill_zone[0], fill_zone[1], fill_zone[2], 0.9],
                    w,
                    h,
                );
            }
        }

        // ---- Attribution (bottom-right, dim) ----
        if let Some(attr) = hud.attribution.clone() {
            let (ai, aw) = self.shape(Role::Small, &attr, TEXT_SECONDARY, w, h);
            self.place(ai, w - MARGIN_PX - aw, h - MARGIN_PX - SMALL_SIZE * 1.3);
        }

        // Upload quads.
        self.quad_count = self.quads.len();
        if self.quad_count > 0 {
            queue.write_buffer(&self.quad_vertices, 0, bytemuck::cast_slice(&self.quads));
        }

        // Prepare all text areas in one pass.
        let areas: Vec<TextArea> = self
            .runs
            .iter()
            .map(|r| TextArea {
                buffer: &r.buffer,
                left: r.left,
                top: r.top,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: r.color,
                custom_glyphs: &[],
            })
            .collect();

        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            areas,
            &mut self.swash_cache,
        )
    }

    pub fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError> {
        if self.quad_count > 0 {
            pass.set_pipeline(&self.quad_pipeline);
            pass.set_vertex_buffer(0, self.quad_vertices.slice(..));
            pass.draw(0..self.quad_count as u32, 0..1);
        }
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

    #[test]
    fn zone_palette_boundaries() {
        let ftp = 200.0;
        assert_eq!(zone_color(100.0, ftp), zone_color(0.4 * ftp, ftp)); // Z1
        assert_ne!(zone_color(0.5 * ftp, ftp), zone_color(0.6 * ftp, ftp)); // Z1→Z2
        assert_ne!(zone_color(0.8 * ftp, ftp), zone_color(1.0 * ftp, ftp)); // Z3→Z4
        assert_ne!(zone_color(1.1 * ftp, ftp), zone_color(1.3 * ftp, ftp)); // Z5→Z6
        assert_ne!(zone_color(1.3 * ftp, ftp), zone_color(1.6 * ftp, ftp)); // Z6→Z7
    }

    #[test]
    fn interval_progress_math() {
        let hud = HudSnapshot {
            interval_duration_s: Some(120.0),
            interval_elapsed_s: Some(30.0),
            ..Default::default()
        };
        assert!((hud.interval_fraction().unwrap() - 0.25).abs() < 1e-9);
        assert!((hud.interval_remaining_s().unwrap() - 90.0).abs() < 1e-9);
    }
}
