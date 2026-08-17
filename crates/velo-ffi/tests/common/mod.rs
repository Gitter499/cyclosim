#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use velo_ffi::{
    ActivityPublisherCallback, AudioDirectorCallback, MediaCaptureCallback, PlaybackIntentDto,
    PublishResultDto, RideSummaryDto, SegmentEnergyDto, SensorSourceCallback,
    SteerStateDto, SteeringInputCallback, TelemetrySampleDto, TrainerControlCallback,
};

/// GPX fixture shared by route scenario tests.
pub fn fixture_gpx_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../velo-route-import/tests/fixtures/simple_climb.gpx")
}

pub struct MockPublisher {
    pub saved_locally: bool,
    pub activity_url: String,
    pub publish_count: Arc<Mutex<usize>>,
}

impl MockPublisher {
    pub fn local(activity_url: impl Into<String>) -> Self {
        Self {
            saved_locally: true,
            activity_url: activity_url.into(),
            publish_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl ActivityPublisherCallback for MockPublisher {
    fn publish_ride(
        &self,
        _fit_bytes: Vec<u8>,
        _screenshot_png: Option<Vec<u8>>,
        _summary: RideSummaryDto,
    ) -> PublishResultDto {
        *self.publish_count.lock().unwrap() += 1;
        PublishResultDto {
            activity_url: self.activity_url.clone(),
            saved_locally: self.saved_locally,
            ride_id: String::new(),
            highlight_clip_path: None,
        }
    }
}

pub struct MockMedia {
    pub highlight_encode_count: Arc<Mutex<usize>>,
}

impl Default for MockMedia {
    fn default() -> Self {
        Self {
            highlight_encode_count: Arc::new(Mutex::new(0)),
        }
    }
}

impl MediaCaptureCallback for MockMedia {
    fn encode_png_rgba(&self, _width: u32, _height: u32, _rgba_pixels: Vec<u8>) -> Vec<u8> {
        vec![0x89, 0x50, 0x4E, 0x47]
    }

    fn encode_highlight_clip(
        &self,
        clips: Vec<velo_ffi::HighlightClipRequestDto>,
        output_path: String,
    ) -> bool {
        if clips.is_empty() {
            return false;
        }
        *self.highlight_encode_count.lock().unwrap() += 1;
        std::fs::write(&output_path, b"mock-mp4").is_ok()
    }
}

/// Replay sensors with monotonic elapsed time and configurable power.
pub struct ReplaySensors {
    pub tick: Arc<Mutex<u64>>,
    pub power_w: f64,
    pub step_ms: u64,
}

impl ReplaySensors {
    pub fn at_180w() -> Self {
        Self {
            tick: Arc::new(Mutex::new(0)),
            power_w: 180.0,
            step_ms: 10,
        }
    }
}

impl SensorSourceCallback for ReplaySensors {
    fn poll_samples(&self) -> Vec<TelemetrySampleDto> {
        let mut t = self.tick.lock().unwrap();
        *t += self.step_ms;
        vec![TelemetrySampleDto {
            elapsed_ms: *t,
            power_w: Some(self.power_w),
            cadence_rpm: Some(90.0),
            heart_rate_bpm: Some(140.0),
            wheel_speed_mps: None,
        }]
    }
}

pub struct TickSensors {
    pub elapsed_ms: u64,
}

impl SensorSourceCallback for TickSensors {
    fn poll_samples(&self) -> Vec<TelemetrySampleDto> {
        vec![TelemetrySampleDto {
            elapsed_ms: self.elapsed_ms,
            power_w: Some(180.0),
            cadence_rpm: Some(90.0),
            heart_rate_bpm: Some(140.0),
            wheel_speed_mps: None,
        }]
    }
}

pub struct NoopTrainer;

impl TrainerControlCallback for NoopTrainer {
    fn set_target_power(&self, _watts: f64) {}
    fn set_simulation(&self, _grade: f64, _crr: f64, _cwa: f64) {}
    fn stop(&self) {}
}

pub struct NoopSteering;

impl SteeringInputCallback for NoopSteering {
    fn poll(&self) -> SteerStateDto {
        SteerStateDto {
            axis: 0.0,
            recenter: false,
        }
    }
}

pub struct MockSteering {
    pub axis: f32,
}

impl SteeringInputCallback for MockSteering {
    fn poll(&self) -> SteerStateDto {
        SteerStateDto {
            axis: self.axis,
            recenter: false,
        }
    }
}

pub struct RecordingAudioDirectorCallback {
    pub calls: Arc<Mutex<Vec<(SegmentEnergyDto, PlaybackIntentDto)>>>,
}

impl AudioDirectorCallback for RecordingAudioDirectorCallback {
    fn on_segment(&self, energy: SegmentEnergyDto, intent: PlaybackIntentDto) {
        self.calls.lock().unwrap().push((energy, intent));
    }
}

pub struct NoopAudioDirector;

impl AudioDirectorCallback for NoopAudioDirector {
    fn on_segment(&self, _energy: SegmentEnergyDto, _intent: PlaybackIntentDto) {}
}

/// Records the last ERG target and SIM grade forwarded through FFI callbacks.
pub struct RecordingTrainerCallback {
    pub last_power: Arc<Mutex<Option<f64>>>,
    pub last_sim: Arc<Mutex<Option<(f64, f64, f64)>>>,
}

impl TrainerControlCallback for RecordingTrainerCallback {
    fn set_target_power(&self, watts: f64) {
        *self.last_power.lock().unwrap() = Some(watts);
    }

    fn set_simulation(&self, grade: f64, crr: f64, cwa: f64) {
        *self.last_sim.lock().unwrap() = Some((grade, crr, cwa));
    }

    fn stop(&self) {}
}
