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
