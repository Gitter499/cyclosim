use velo_ffi::{
    ActivityPublisherCallback, MediaCaptureCallback, PublishResultDto, PublishStatus,
    RideSummaryDto, SensorSourceCallback, TelemetrySampleDto, TrainerControlCallback, VeloHandle,
};

struct MockPublisher {
    saved_locally: bool,
    activity_url: String,
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
        }
    }
}

struct MockMedia;

impl MediaCaptureCallback for MockMedia {
    fn encode_png_rgba(&self, _width: u32, _height: u32, _rgba_pixels: Vec<u8>) -> Vec<u8> {
        vec![0x89, 0x50, 0x4E, 0x47]
    }
}

struct TickSensors {
    elapsed_ms: u64,
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

struct NoopTrainer;

impl TrainerControlCallback for NoopTrainer {
    fn set_target_power(&self, _watts: f64) {}
    fn set_simulation(&self, _grade: f64, _crr: f64, _cwa: f64) {}
    fn stop(&self) {}
}

#[test]
fn finish_ride_persists_to_library() {
    let dir = tempfile::TempDir::new().unwrap();
    let db = dir.path().join("rides.db");
    let artifacts = dir.path().join("artifacts");

    let handle = VeloHandle::new();
    handle
        .configure_ride_library(
            db.display().to_string(),
            artifacts.display().to_string(),
        )
        .unwrap();

    handle.start_ride();
    let mut elapsed_ms = 0u64;
    for _ in 0..8 {
        elapsed_ms += 33;
        handle.tick(
            Box::new(TickSensors { elapsed_ms }),
            Box::new(NoopTrainer),
        );
    }

    let result = handle
        .finish_ride_and_publish(Box::new(MockMedia), Box::new(MockPublisher {
            saved_locally: true,
            activity_url: "/tmp/ride-folder".into(),
        }))
        .expect("finish ride");

    assert!(!result.ride_id.is_empty());
    let rides = handle.list_rides().unwrap();
    assert_eq!(rides.len(), 1);
    assert_eq!(rides[0].id, result.ride_id);
    assert_eq!(rides[0].publish_status, PublishStatus::Local);
}

#[test]
fn strava_publish_records_activity_id() {
    let dir = tempfile::TempDir::new().unwrap();
    let handle = VeloHandle::new();
    handle
        .configure_ride_library(
            dir.path().join("rides.db").display().to_string(),
            dir.path().join("artifacts").display().to_string(),
        )
        .unwrap();

    handle.start_ride();
    for ms in (33..=264).step_by(33) {
        handle.tick(
            Box::new(TickSensors { elapsed_ms: ms }),
            Box::new(NoopTrainer),
        );
    }

    let result = handle
        .finish_ride_and_publish(
            Box::new(MockMedia),
            Box::new(MockPublisher {
                saved_locally: false,
                activity_url: "https://www.strava.com/activities/998877".into(),
            }),
        )
        .unwrap();

    let ride = handle.get_ride(result.ride_id).unwrap().unwrap();
    assert_eq!(ride.publish_status, PublishStatus::Strava);
    assert_eq!(ride.strava_activity_id.as_deref(), Some("998877"));
}
