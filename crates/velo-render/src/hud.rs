//! Glyphon text overlay for the 3D view.
//!
//! **Single HUD path:** live ride metrics are drawn by the Swift shell (`RideHUDOverlay`).
//! This renderer is disabled during normal riding (`hud_draw_enabled = false` at init) and
//! retained for screenshot/capture paths that need baked-in stats. See
//! `docs/VeloSim-UI-and-Zwift-Parity-Guide.md` §2 and `shell-macos/.../HUD/RideHUDOverlay.swift`.

use glyphon::{
    Attrs, Buffer, Cache, Color, Family, FontSystem, Metrics, Resolution, Shaping, SwashCache,
    TextArea, TextAtlas, TextBounds, TextRenderer, Viewport,
};
use wgpu::{MultisampleState, Queue, TextureFormat};

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
    pub elevation_m: Option<f64>,
    pub mode: &'static str,
    pub workout_interval: Option<String>,
    pub workout_target_w: Option<f64>,
    pub interval_duration_s: Option<f64>,
    pub interval_elapsed_s: Option<f64>,
    pub attribution: Option<String>,
}

impl HudSnapshot {
    pub fn interval_fraction(&self) -> Option<f64> {
        let duration = self.interval_duration_s?;
        let elapsed = self.interval_elapsed_s?;
        if duration > 0.0 { Some((elapsed / duration).clamp(0.0, 1.0)) } else { None }
    }

    pub fn interval_remaining_s(&self) -> Option<f64> {
        let duration = self.interval_duration_s?;
        let elapsed = self.interval_elapsed_s?;
        if duration > 0.0 { Some((duration - elapsed).max(0.0)) } else { None }
    }

    fn format_interval_bar(&self) -> Option<String> {
        let name = self.workout_interval.as_ref()?;
        let remaining = self.interval_remaining_s()?;
        let mins = (remaining / 60.0).floor() as u32;
        let secs = (remaining % 60.0).floor() as u32;
        let target = match self.workout_target_w {
            Some(w) => format!("{:.0} W", w),
            None => "Free".into(),
        };
        let filled = (self.interval_fraction().unwrap_or(0.0) * 20.0).round() as usize;
        let bar: String = (0..20).map(|i| if i < filled { '█' } else { '░' }).collect();
        Some(format!("{name} · {target} · {mins:02}:{secs:02}  [{bar}]"))
    }

    fn format_grade_elevation(&self) -> String {
        let grade_pct = self.grade * 100.0;
        match self.elevation_m {
            Some(elev) => format!("Elev: {:.0} m  Grade: {grade_pct:.1}%", elev),
            None => format!("Grade: {grade_pct:.1}%"),
        }
    }

    pub fn lines(&self) -> Vec<String> {
        let speed_kmh = self.speed_mps * 3.6;
        let mins = (self.elapsed_s / 60.0).floor() as u32;
        let secs = (self.elapsed_s % 60.0).floor() as u32;
        let mut lines = Vec::new();
        if let Some(bar) = self.format_interval_bar() {
            lines.push(bar);
        } else if let Some(interval) = &self.workout_interval {
            let target = match self.workout_target_w {
                Some(w) => format!("{:.0} W", w),
                None => "Free ride".into(),
            };
            lines.push(format!("Interval: {interval}  Target: {target}"));
        }
        lines.push(format!(
            "POWER {}  |  HR {}  |  CAD {}",
            self.power_w.map(|w| format!("{:.0} W", w)).unwrap_or_else(|| "—".into()),
            self.heart_rate_bpm.map(|b| format!("{:.0}", b)).unwrap_or_else(|| "—".into()),
            self.cadence_rpm.map(|c| format!("{:.0}", c)).unwrap_or_else(|| "—".into()),
        ));
        lines.push(format!("Speed: {:.1} km/h  Dist: {:.0} m  Time: {mins:02}:{secs:02}", speed_kmh, self.distance_m));
        lines.push(self.format_grade_elevation());
        lines.push(format!("Mode: {}", self.mode));
        if let Some(attr) = &self.attribution { lines.push(attr.clone()); }
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
        let metrics = Metrics::new(18.0, 22.0);
        let buffer = Buffer::new(&mut font_system, metrics);

        Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            text_atlas,
            text_renderer,
            buffer,
        }
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

        let bottom = height.saturating_sub(180) as i32;
        self.text_renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.text_atlas,
            &self.viewport,
            [TextArea {
                buffer: &self.buffer,
                left: 16.0,
                top: bottom as f32,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: bottom,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: Color::rgb(230, 235, 240),
                custom_glyphs: &[],
            }],
            &mut self.swash_cache,
        )
    }

    pub fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), glyphon::RenderError> {
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
    fn interval_fraction_and_remaining() {
        let hud = HudSnapshot {
            interval_duration_s: Some(120.0),
            interval_elapsed_s: Some(30.0),
            ..Default::default()
        };
        assert!((hud.interval_fraction().unwrap() - 0.25).abs() < f64::EPSILON);
        assert!((hud.interval_remaining_s().unwrap() - 90.0).abs() < f64::EPSILON);
    }

    #[test]
    fn interval_bar_line_includes_name_and_remaining() {
        let hud = HudSnapshot {
            workout_interval: Some("Block 1".into()),
            workout_target_w: Some(250.0),
            interval_duration_s: Some(120.0),
            interval_elapsed_s: Some(30.0),
            ..Default::default()
        };
        let lines = hud.lines();
        let line = lines.first().expect("interval bar line");
        assert!(line.contains("Block 1"));
        assert!(line.contains("250 W"));
        assert!(line.contains("01:30"));
        assert!(line.contains('█'));
    }

    #[test]
    fn grade_elevation_line_when_route_elev_present() {
        let hud = HudSnapshot {
            grade: 0.052,
            elevation_m: Some(842.6),
            ..Default::default()
        };
        let lines = hud.lines();
        let line = lines
            .iter()
            .find(|l| l.contains("Grade"))
            .expect("grade line");
        assert!(line.contains("843 m"));
        assert!(line.contains("5.2%"));
    }
}
