#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use velo_ffi::{
    ActivityPublisherCallback, AudioDirectorCallback, MediaCaptureCallback, PlaybackIntentDto,
    PublishResultDto, RideSummaryDto, SegmentEnergyDto, SensorSourceCallback,
    SteerStateDto, SteeringInputCallback, TelemetrySampleDto, TrainerControlCallback,
};

pub struct MockPublisher {
    pub saved_locally: bool,
    pub activity_url: String,
}

impl ActivityPublisherCallback for MockPublisher {
    fn publish_ride(
        &self,
        _fit_bytes: Vec<u8>,
        _screenshot_png: Option<Vec<u8>>,
        _summary: RideSummaryDto,
    ) -> PublishResultDto {
        PublishResultDto {
            activity_url: self.activity_url.clone(),
            saved_locally: self.saved_locally,
            ride_id: String::new(),
            highlight_clip_path: None,
        }
    }
}

pub struct MockMedia;

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
        std::fs::write(&output_path, b"mock-mp4").is_ok()
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
