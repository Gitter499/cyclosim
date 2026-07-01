uniffi::setup_scaffolding!();

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use velo_bikegen::{
    bikegen_mode_status, default_bikes_dir, import_bike_from_images, list_bikes, load_bike_asset,
    set_bikegen_credentials, BikeSummary, BikegenCredentials,
};
use velo_cesium::{set_tiles_credentials, tiles_provider_status, TilesCredentials};
use velo_core::{
    default_packs_dir, list_route_packs, load_route_pack, load_scenery_config, pack_dir_for_id,
    parse_zwo_xml as parse_zwo_xml_core, save_scenery_config, SceneryConfig, VeloApp, Workout,
    WorkoutInterval, WorkoutTarget,
};
use velo_platform::{
    AudioDirector, PlaybackIntent, SegmentEnergy, SensorSource, SteeringInput, TelemetrySample,
    TrainerControl,
};
use velo_render::{forward_from_enu, Renderer, RouteFollow};
use velo_rides::{
    default_artifacts_base, default_db_path, NewRideRecord, PublishStatus as StorePublishStatus,
    RideLibrary, RideRecord,
};
use velo_route_import::import_file;
use velo_terrain::{bake_terrain_for_route, DEFAULT_CELL_M, DEFAULT_CORRIDOR_M};
use velo_units::{Bpm, Grade, MetersPerSecond, Rpm, Watts};

/// UniFFI telemetry pattern: Swift pushes samples via `poll_samples()` each tick;
/// Rust drains them through `SensorSource::drain_samples`. Do not expose `mpsc::Receiver` over FFI.
struct FfiSensorSource {
    callback: Box<dyn SensorSourceCallback>,
}

impl SensorSource for FfiSensorSource {
    fn drain_samples(&mut self) -> Vec<TelemetrySample> {
        self.callback
            .poll_samples()
            .into_iter()
            .map(|s| TelemetrySample {
                elapsed: Duration::from_millis(s.elapsed_ms),
                power: s.power_w.map(Watts::new),
                cadence: s.cadence_rpm.map(Rpm::new),
                heart_rate: s.heart_rate_bpm.map(Bpm::new),
                wheel_speed: s.wheel_speed_mps.map(MetersPerSecond::new),
            })
            .collect()
    }
}

struct FfiTrainerControl {
    callback: Box<dyn TrainerControlCallback>,
}

struct FfiSteeringInput {
    callback: Box<dyn SteeringInputCallback>,
}

impl SteeringInput for FfiSteeringInput {
    fn poll(&self) -> velo_platform::SteerState {
        let dto = self.callback.poll();
        velo_platform::SteerState {
            axis: dto.axis,
            recenter: dto.recenter,
        }
    }
}

struct FfiAudioDirector {
    callback: Arc<dyn AudioDirectorCallback>,
}

impl AudioDirector for FfiAudioDirector {
    fn on_segment(&self, energy: SegmentEnergy, intent: PlaybackIntent) {
        self.callback
            .on_segment(map_segment_energy(energy), map_playback_intent(intent));
    }
}

impl TrainerControl for FfiTrainerControl {
    fn set_target_power(&self, watts: Watts) {
        self.callback.set_target_power(watts.0);
    }

    fn set_simulation(&self, grade: Grade, crr: f32, cw_a: f32) {
        self.callback
            .set_simulation(grade.0, crr as f64, cw_a as f64);
    }

    fn stop(&self) {
        self.callback.stop();
    }

    fn capabilities(&self) -> velo_platform::TrainerCaps {
        velo_platform::TrainerCaps {
            erg: true,
            sim: true,
            max_watts: 2000,
        }
    }
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum VeloError {
    #[error("render error")]
    RenderError,
    #[error("ride error: {message}")]
    RideError { message: String },
    #[error("publish error: {message}")]
    PublishError { message: String },
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum RideMode {
    Free,
    Erg,
    Sim,
}

#[derive(uniffi::Record, Clone, Debug, Default)]
pub struct TelemetrySampleDto {
    pub elapsed_ms: u64,
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub wheel_speed_mps: Option<f64>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct RideStateDto {
    pub mode: RideMode,
    pub distance_m: f64,
    pub speed_mps: f64,
    pub elapsed_s: f64,
    pub grade: f64,
    pub elevation_m: Option<f64>,
    pub power_w: Option<f64>,
    pub cadence_rpm: Option<f64>,
    pub heart_rate_bpm: Option<f64>,
    pub steer_axis: f32,
    pub steer_yaw_rad: f32,
}

#[derive(uniffi::Record, Clone, Debug, Default)]
pub struct WorkoutLiveDto {
    pub active: bool,
    pub workout_name: String,
    pub interval_name: String,
    pub interval_elapsed_s: f64,
    pub interval_duration_s: f64,
    pub workout_elapsed_s: f64,
    pub target_watts: Option<f64>,
    pub finished: bool,
}

#[derive(uniffi::Enum, Clone, Debug, PartialEq)]
pub enum WorkoutTargetDto {
    ErgWatts { watts: f64 },
    FtpPercent { percent: f64 },
    FreeRide,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct WorkoutIntervalDto {
    pub name: String,
    pub duration_s: f64,
    pub target: WorkoutTargetDto,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct WorkoutDto {
    pub name: String,
    pub intervals: Vec<WorkoutIntervalDto>,
}

#[derive(uniffi::Record, Clone, Debug, Default)]
pub struct RideSummaryDto {
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub sample_count: u32,
    pub avg_power_w: Option<f64>,
    pub max_power_w: Option<f64>,
    pub started_at_unix: u64,
    pub highlight_clips: Vec<HighlightClipRequestDto>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct HighlightClipRequestDto {
    pub start_elapsed_s: f64,
    pub duration_s: f64,
    pub label: String,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct FramebufferDto {
    pub width: u32,
    pub height: u32,
    pub rgba_pixels: Vec<u8>,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct PublishResultDto {
    pub activity_url: String,
    pub saved_locally: bool,
    pub ride_id: String,
    pub highlight_clip_path: Option<String>,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublishStatus {
    Local,
    Strava,
    Failed,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct RouteInfoDto {
    pub route_id: String,
    pub name: String,
    pub total_distance_m: f64,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct BikeInfoDto {
    pub bike_id: String,
    pub name: String,
}

/// Runtime API keys/tokens injected by the macOS shell (Keychain → FFI). Never persisted by Rust.
#[derive(uniffi::Record, Clone, Debug, Default)]
pub struct RuntimeSecretsDto {
    pub google_map_tiles_api_key: Option<String>,
    pub cesium_ion_access_token: Option<String>,
    pub meshy_api_key: Option<String>,
    pub prefer_hosted_bike_generation: bool,
}

#[derive(uniffi::Record, Clone, Debug)]
pub struct RideRecordDto {
    pub id: String,
    pub started_at_unix: u64,
    pub elapsed_s: f64,
    pub distance_m: f64,
    pub avg_power_w: Option<f64>,
    pub max_power_w: Option<f64>,
    pub fit_path: String,
    pub screenshot_path: Option<String>,
    pub highlight_clip_path: Option<String>,
    pub strava_activity_id: Option<String>,
    pub publish_status: PublishStatus,
    pub route_id: Option<String>,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentEnergyDto {
    Warmup,
    Build,
    Threshold,
    Recovery,
    Cooldown,
}

#[derive(uniffi::Enum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackIntentDto {
    Start,
    Transition,
    Duck,
}

#[derive(uniffi::Record, Clone, Debug, Default)]
pub struct SteerStateDto {
    pub axis: f32,
    pub recenter: bool,
}

#[uniffi::export(callback_interface)]
pub trait SteeringInputCallback: Send + Sync {
    fn poll(&self) -> SteerStateDto;
}

/// Shell maps segment energy to MusicKit queues/playlists (playback control only).
#[uniffi::export(callback_interface)]
pub trait AudioDirectorCallback: Send + Sync {
    fn on_segment(&self, energy: SegmentEnergyDto, intent: PlaybackIntentDto);
}

#[uniffi::export(callback_interface)]
pub trait SensorSourceCallback: Send + Sync {
    fn poll_samples(&self) -> Vec<TelemetrySampleDto>;
}

#[uniffi::export(callback_interface)]
pub trait TrainerControlCallback: Send + Sync {
    fn set_target_power(&self, watts: f64);
    fn set_simulation(&self, grade: f64, crr: f64, cwa: f64);
    fn stop(&self);
}

/// Shell encodes RGBA → PNG and highlight clips (VideoToolbox H.264).
#[uniffi::export(callback_interface)]
pub trait MediaCaptureCallback: Send + Sync {
    fn encode_png_rgba(&self, width: u32, height: u32, rgba_pixels: Vec<u8>) -> Vec<u8>;

    /// Encode highlight reel from ring-buffer frames captured during the ride.
    /// Returns true when `output_path` contains a valid MP4.
    fn encode_highlight_clip(
        &self,
        clips: Vec<HighlightClipRequestDto>,
        output_path: String,
    ) -> bool;
}

/// Shell uploads FIT + optional screenshot (Strava OAuth) or saves locally.
#[uniffi::export(callback_interface)]
pub trait ActivityPublisherCallback: Send + Sync {
    fn publish_ride(
        &self,
        fit_bytes: Vec<u8>,
        screenshot_png: Option<Vec<u8>>,
        summary: RideSummaryDto,
    ) -> PublishResultDto;
}

fn map_segment_energy(energy: SegmentEnergy) -> SegmentEnergyDto {
    match energy {
        SegmentEnergy::Warmup => SegmentEnergyDto::Warmup,
        SegmentEnergy::Build => SegmentEnergyDto::Build,
        SegmentEnergy::Threshold => SegmentEnergyDto::Threshold,
        SegmentEnergy::Recovery => SegmentEnergyDto::Recovery,
        SegmentEnergy::Cooldown => SegmentEnergyDto::Cooldown,
    }
}

fn map_playback_intent(intent: PlaybackIntent) -> PlaybackIntentDto {
    match intent {
        PlaybackIntent::Start => PlaybackIntentDto::Start,
        PlaybackIntent::Transition => PlaybackIntentDto::Transition,
        PlaybackIntent::Duck => PlaybackIntentDto::Duck,
    }
}

fn map_ride_mode(mode: velo_core::ride::RideMode) -> RideMode {
    match mode {
        velo_core::ride::RideMode::Free => RideMode::Free,
        velo_core::ride::RideMode::Erg => RideMode::Erg,
        velo_core::ride::RideMode::Sim => RideMode::Sim,
    }
}

fn map_ride_mode_in(mode: RideMode) -> velo_core::ride::RideMode {
    match mode {
        RideMode::Free => velo_core::ride::RideMode::Free,
        RideMode::Erg => velo_core::ride::RideMode::Erg,
        RideMode::Sim => velo_core::ride::RideMode::Sim,
    }
}

fn map_highlight_clip(c: velo_core::HighlightClipRequest) -> HighlightClipRequestDto {
    HighlightClipRequestDto {
        start_elapsed_s: c.start_elapsed_s,
        duration_s: c.duration_s,
        label: c.label,
    }
}

fn map_summary(summary: velo_core::RideSummary) -> RideSummaryDto {
    RideSummaryDto {
        elapsed_s: summary.elapsed_s,
        distance_m: summary.distance_m,
        sample_count: summary.sample_count,
        avg_power_w: summary.avg_power_w,
        max_power_w: summary.max_power_w,
        started_at_unix: summary.started_at_unix,
        highlight_clips: summary
            .highlight_clips
            .into_iter()
            .map(map_highlight_clip)
            .collect(),
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1_700_000_000)
}

fn map_publish_status(status: StorePublishStatus) -> PublishStatus {
    match status {
        StorePublishStatus::Local => PublishStatus::Local,
        StorePublishStatus::Strava => PublishStatus::Strava,
        StorePublishStatus::Failed => PublishStatus::Failed,
    }
}

fn map_ride_record(record: RideRecord) -> RideRecordDto {
    RideRecordDto {
        id: record.id,
        started_at_unix: record.started_at_unix,
        elapsed_s: record.elapsed_s,
        distance_m: record.distance_m,
        avg_power_w: record.avg_power_w,
        max_power_w: record.max_power_w,
        fit_path: record.fit_path,
        screenshot_path: record.screenshot_path,
        highlight_clip_path: record.highlight_clip_path,
        strava_activity_id: record.strava_activity_id,
        publish_status: map_publish_status(record.publish_status),
        route_id: record.route_id,
    }
}

fn map_bike_summary(summary: BikeSummary) -> BikeInfoDto {
    BikeInfoDto {
        bike_id: summary.bike_id,
        name: summary.name,
    }
}

fn extract_strava_activity_id(url: &str) -> Option<String> {
    url.strip_prefix("https://www.strava.com/activities/")
        .map(|id| id.split('/').next().unwrap_or(id).to_string())
}

fn infer_publish_status(result: &PublishResultDto) -> StorePublishStatus {
    if result.saved_locally {
        if result.activity_url.starts_with("error:") {
            StorePublishStatus::Failed
        } else {
            StorePublishStatus::Local
        }
    } else {
        StorePublishStatus::Strava
    }
}

fn persist_finished_ride(
    library: &RideLibrary,
    summary: &RideSummaryDto,
    fit_bytes: &[u8],
    screenshot_png: Option<&[u8]>,
    publish: &PublishResultDto,
    route_id: Option<String>,
) -> Result<String, VeloError> {
    let artifacts = library
        .save_ride_artifacts(fit_bytes, screenshot_png)
        .map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;

    let strava_activity_id = if publish.saved_locally {
        None
    } else {
        extract_strava_activity_id(&publish.activity_url)
    };

    library
        .insert_ride_with_id(
            &artifacts.ride_id,
            NewRideRecord {
                started_at_unix: summary.started_at_unix,
                elapsed_s: summary.elapsed_s,
                distance_m: summary.distance_m,
                avg_power_w: summary.avg_power_w,
                max_power_w: summary.max_power_w,
                fit_path: artifacts.fit_path.display().to_string(),
                screenshot_path: artifacts
                    .screenshot_path
                    .as_ref()
                    .map(|p| p.display().to_string()),
                highlight_clip_path: None,
                strava_activity_id,
                publish_status: infer_publish_status(publish),
                route_id,
            },
        )
        .map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;

    Ok(artifacts.ride_id)
}

#[derive(uniffi::Object)]
pub struct RideLibraryHandle {
    inner: RideLibrary,
}

#[uniffi::export]
impl RideLibraryHandle {
    #[uniffi::constructor]
    pub fn with_defaults() -> Result<Self, VeloError> {
        Self::open_paths(
            default_db_path().display().to_string(),
            default_artifacts_base().display().to_string(),
        )
    }

    #[uniffi::constructor(name = "open")]
    pub fn open_paths(db_path: String, artifacts_base: String) -> Result<Self, VeloError> {
        let inner = RideLibrary::open(PathBuf::from(db_path), PathBuf::from(artifacts_base))
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })?;
        Ok(Self { inner })
    }

    pub fn list_rides(&self) -> Result<Vec<RideRecordDto>, VeloError> {
        self.inner
            .list_rides()
            .map(|rides| rides.into_iter().map(map_ride_record).collect())
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn get_ride(&self, id: String) -> Result<Option<RideRecordDto>, VeloError> {
        self.inner
            .get_ride(&id)
            .map(|opt| opt.map(map_ride_record))
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn delete_ride(&self, id: String) -> Result<bool, VeloError> {
        self.inner
            .delete_ride(&id)
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }
}

#[derive(Default)]
struct VeloHandleInner {
    app: VeloApp,
    renderer: Option<Renderer>,
    ride_library: Option<RideLibrary>,
    packs_dir: PathBuf,
    bikes_dir: PathBuf,
    active_bike_id: Option<String>,
    tiles_3d_enabled: bool,
    audio_director: Option<Arc<dyn AudioDirectorCallback>>,
}

#[derive(uniffi::Object)]
pub struct VeloHandle {
    inner: Mutex<VeloHandleInner>,
}

impl VeloHandle {
    fn with_dirs(packs_dir: PathBuf, bikes_dir: PathBuf) -> Self {
        let mut inner = VeloHandleInner::default();
        inner.app.set_clock_unix(unix_now());
        inner.packs_dir = packs_dir;
        inner.bikes_dir = bikes_dir;
        inner.ride_library = RideLibrary::open(default_db_path(), default_artifacts_base()).ok();
        Self {
            inner: Mutex::new(inner),
        }
    }

    /// Integration tests only — avoids writing under `~/Documents`.
    #[doc(hidden)]
    pub fn with_dirs_for_tests(packs_dir: PathBuf, bikes_dir: PathBuf) -> Self {
        Self::with_dirs(packs_dir, bikes_dir)
    }

    /// Integration tests only — avoids writing under `~/Documents`.
    #[doc(hidden)]
    pub fn with_packs_dir_for_tests(packs_dir: PathBuf) -> Self {
        Self::with_dirs(packs_dir, default_bikes_dir())
    }
}

#[uniffi::export]
impl VeloHandle {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self::with_dirs(default_packs_dir(), default_bikes_dir())
    }

    pub fn packs_dir(&self) -> String {
        self.inner.lock().unwrap().packs_dir.display().to_string()
    }

    pub fn bikes_dir(&self) -> String {
        self.inner.lock().unwrap().bikes_dir.display().to_string()
    }

    pub fn list_routes(&self) -> Result<Vec<RouteInfoDto>, VeloError> {
        let inner = self.inner.lock().unwrap();
        let ids = list_route_packs(&inner.packs_dir).map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;
        let mut routes = Vec::new();
        for id in ids {
            let pack_dir = pack_dir_for_id(&inner.packs_dir, &id);
            if let Ok(route) = load_route_pack(&pack_dir) {
                routes.push(RouteInfoDto {
                    route_id: route.meta.route_id.clone(),
                    name: route.meta.name.clone(),
                    total_distance_m: route.meta.total_distance_m,
                });
            }
        }
        Ok(routes)
    }

    pub fn import_gpx_route(
        &self,
        gpx_path: String,
        route_id: String,
        name: Option<String>,
    ) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let pack_dir = pack_dir_for_id(&inner.packs_dir, &route_id);
        std::fs::create_dir_all(&pack_dir).map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;
        let model = import_file(
            std::path::Path::new(&gpx_path),
            &route_id,
            name.as_deref(),
            velo_route_import::DEFAULT_SPACING_M,
            velo_route_import::DEFAULT_GRADE_WINDOW_M,
        )
        .map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;
        model
            .save_pack(&pack_dir)
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })?;
        bake_terrain_for_route(&model, &pack_dir, DEFAULT_CORRIDOR_M, DEFAULT_CELL_M).map_err(
            |e| VeloError::RideError {
                message: e.to_string(),
            },
        )?;
        inner.app.load_route(model);
        if let Some(renderer) = inner.renderer.as_mut() {
            let _ = renderer.load_terrain_pack(&pack_dir);
        }
        Ok(())
    }

    pub fn set_active_route(&self, route_id: String) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let pack_dir = pack_dir_for_id(&inner.packs_dir, &route_id);
        let route = load_route_pack(&pack_dir).map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;
        inner.app.load_route(route);
        let scenery = load_scenery_config(&pack_dir);
        inner.tiles_3d_enabled = scenery.tiles_3d_enabled;
        let tiles_on = inner.tiles_3d_enabled;
        let route_for_tiles = inner.app.route.clone();
        let distance_m = inner.app.ride.distance_m;
        if let Some(renderer) = inner.renderer.as_mut() {
            let _ = renderer.load_terrain_pack(&pack_dir);
            renderer.set_tiles_mode(tiles_on);
            if tiles_on {
                if let Some(route) = &route_for_tiles {
                    sync_tiles_view(route, distance_m, renderer);
                }
            }
        }
        Ok(())
    }

    pub fn clear_active_route(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.app.clear_route();
        inner.tiles_3d_enabled = false;
        if let Some(renderer) = inner.renderer.as_mut() {
            renderer.clear_terrain();
            renderer.set_tiles_mode(false);
        }
    }

    pub fn active_route_id(&self) -> Option<String> {
        self.inner
            .lock()
            .unwrap()
            .app
            .active_route_id()
            .map(|s| s.to_string())
    }

    pub fn list_bikes(&self) -> Result<Vec<BikeInfoDto>, VeloError> {
        let inner = self.inner.lock().unwrap();
        list_bikes(&inner.bikes_dir)
            .map(|bikes| bikes.into_iter().map(map_bike_summary).collect())
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn import_bike_from_images(
        &self,
        image_paths: Vec<String>,
        bike_id: String,
        name: Option<String>,
    ) -> Result<(), VeloError> {
        let paths: Vec<PathBuf> = image_paths.into_iter().map(PathBuf::from).collect();
        let mut inner = self.inner.lock().unwrap();
        let asset = import_bike_from_images(&inner.bikes_dir, &paths, &bike_id, name.as_deref())
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })?;
        inner.active_bike_id = Some(bike_id);
        if let Some(renderer) = inner.renderer.as_mut() {
            let _ = renderer.load_bike_gltf(&asset.gltf_path, asset.anchor);
        }
        Ok(())
    }

    pub fn set_active_bike(&self, bike_id: String) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let asset =
            load_bike_asset(&inner.bikes_dir, &bike_id).map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })?;
        inner.active_bike_id = Some(bike_id);
        if let Some(renderer) = inner.renderer.as_mut() {
            renderer
                .load_bike_gltf(&asset.gltf_path, asset.anchor)
                .map_err(|_| VeloError::RenderError)?;
        }
        Ok(())
    }

    pub fn clear_active_bike(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.active_bike_id = None;
        if let Some(renderer) = inner.renderer.as_mut() {
            renderer.clear_bike();
        }
    }

    pub fn active_bike_id(&self) -> Option<String> {
        self.inner.lock().unwrap().active_bike_id.clone()
    }

    /// Per-route Tier B toggle (online-only; persisted in pack `scenery.json`).
    pub fn set_route_tiles_3d(&self, enabled: bool) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let route_id = inner.app.active_route_id().ok_or(VeloError::RideError {
            message: "no active route".into(),
        })?;
        let pack_dir = pack_dir_for_id(&inner.packs_dir, route_id);
        let config = SceneryConfig {
            tiles_3d_enabled: enabled,
        };
        save_scenery_config(&pack_dir, &config).map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;
        inner.tiles_3d_enabled = enabled;
        let route_for_tiles = inner.app.route.clone();
        let distance_m = inner.app.ride.distance_m;
        if let Some(renderer) = inner.renderer.as_mut() {
            renderer.set_tiles_mode(enabled);
            if enabled {
                if let Some(route) = &route_for_tiles {
                    sync_tiles_view(route, distance_m, renderer);
                }
            }
        }
        Ok(())
    }

    pub fn route_tiles_3d_enabled(&self) -> bool {
        self.inner.lock().unwrap().tiles_3d_enabled
    }

    pub fn tiles_attribution(&self) -> String {
        self.inner
            .lock()
            .unwrap()
            .renderer
            .as_ref()
            .map(|r| r.tiles_attribution().to_string())
            .unwrap_or_default()
    }

    /// Apply shell Keychain secrets before ride / 3D Tiles use.
    pub fn configure_runtime_secrets(&self, secrets: RuntimeSecretsDto) {
        set_tiles_credentials(TilesCredentials {
            google_map_tiles_api_key: secrets.google_map_tiles_api_key,
            cesium_ion_access_token: secrets.cesium_ion_access_token,
        });
        set_bikegen_credentials(BikegenCredentials {
            meshy_api_key: secrets.meshy_api_key,
            prefer_hosted_generation: secrets.prefer_hosted_bike_generation,
        });
        let mut inner = self.inner.lock().unwrap();
        if let Some(renderer) = inner.renderer.as_mut() {
            renderer.refresh_tiles_session();
        }
    }

    pub fn tiles_provider_status(&self) -> String {
        tiles_provider_status()
    }

    pub fn bikegen_mode_status(&self) -> String {
        bikegen_mode_status()
    }

    pub fn tiles_last_error(&self) -> Option<String> {
        self.inner
            .lock()
            .unwrap()
            .renderer
            .as_ref()
            .and_then(|r| r.tiles_last_error())
    }

    /// Open or create the ride library at custom paths (for tests).
    pub fn configure_ride_library(
        &self,
        db_path: String,
        artifacts_base: String,
    ) -> Result<(), VeloError> {
        let library = RideLibrary::open(PathBuf::from(db_path), PathBuf::from(artifacts_base))
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })?;
        self.inner.lock().unwrap().ride_library = Some(library);
        Ok(())
    }

    pub fn list_rides(&self) -> Result<Vec<RideRecordDto>, VeloError> {
        let inner = self.inner.lock().unwrap();
        let library = inner.ride_library.as_ref().ok_or(VeloError::RideError {
            message: "ride library not configured".into(),
        })?;
        library
            .list_rides()
            .map(|rides| rides.into_iter().map(map_ride_record).collect())
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn get_ride(&self, id: String) -> Result<Option<RideRecordDto>, VeloError> {
        let inner = self.inner.lock().unwrap();
        let library = inner.ride_library.as_ref().ok_or(VeloError::RideError {
            message: "ride library not configured".into(),
        })?;
        library
            .get_ride(&id)
            .map(|opt| opt.map(map_ride_record))
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn delete_ride(&self, id: String) -> Result<bool, VeloError> {
        let inner = self.inner.lock().unwrap();
        let library = inner.ride_library.as_ref().ok_or(VeloError::RideError {
            message: "ride library not configured".into(),
        })?;
        library.delete_ride(&id).map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })
    }

    pub fn toggle(&self) -> u32 {
        self.inner.lock().unwrap().app.toggle()
    }

    pub fn toggle_count(&self) -> u32 {
        self.inner.lock().unwrap().app.toggle_count()
    }

    pub fn set_ride_mode(&self, mode: RideMode) {
        self.inner
            .lock()
            .unwrap()
            .app
            .set_ride_mode(map_ride_mode_in(mode));
    }

    pub fn set_target_power(&self, watts: f64) {
        self.inner.lock().unwrap().app.set_target_power(watts);
    }

    pub fn set_ftp(&self, ftp_w: f64) {
        self.inner.lock().unwrap().app.set_ftp(ftp_w);
    }

    pub fn ftp(&self) -> f64 {
        self.inner.lock().unwrap().app.ftp()
    }

    pub fn start_sample_workout(&self) {
        self.inner
            .lock()
            .unwrap()
            .app
            .start_workout(Workout::sample_threshold());
    }

    pub fn start_workout(&self, workout: WorkoutDto) -> Result<(), VeloError> {
        let workout = map_workout_dto(workout)?;
        workout
            .validate()
            .map_err(|message| VeloError::RideError { message })?;
        self.inner.lock().unwrap().app.start_workout(workout);
        Ok(())
    }

    pub fn clear_workout(&self) {
        self.inner.lock().unwrap().app.clear_workout();
    }

    pub fn workout_active(&self) -> bool {
        self.inner.lock().unwrap().app.workout_active()
    }

    pub fn workout_live(&self) -> WorkoutLiveDto {
        map_workout_live(&self.inner.lock().unwrap().app)
    }

    pub fn set_grade(&self, grade: f64) {
        self.inner.lock().unwrap().app.set_grade(grade);
    }

    pub fn target_power(&self) -> f64 {
        self.inner.lock().unwrap().app.target_power()
    }

    pub fn is_ride_recording(&self) -> bool {
        self.inner.lock().unwrap().app.is_ride_recording()
    }

    pub fn start_ride(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.app.set_clock_unix(unix_now());
        inner.app.start_ride();
    }

    pub fn stop_ride(&self) -> Option<RideSummaryDto> {
        self.inner.lock().unwrap().app.stop_ride().map(map_summary)
    }

    pub fn export_fit(&self) -> Result<Vec<u8>, VeloError> {
        self.inner
            .lock()
            .unwrap()
            .app
            .export_fit()
            .map_err(|e| VeloError::RideError {
                message: e.to_string(),
            })
    }

    pub fn last_ride_summary(&self) -> Option<RideSummaryDto> {
        self.inner
            .lock()
            .unwrap()
            .app
            .last_ride_summary()
            .map(map_summary)
    }

    pub fn set_segment_music_enabled(&self, enabled: bool) {
        self.inner
            .lock()
            .unwrap()
            .app
            .set_segment_music_enabled(enabled);
    }

    pub fn segment_music_enabled(&self) -> bool {
        self.inner.lock().unwrap().app.segment_music_enabled()
    }

    pub fn set_steering_enabled(&self, enabled: bool) {
        self.inner.lock().unwrap().app.set_steering_enabled(enabled);
    }

    pub fn steering_enabled(&self) -> bool {
        self.inner.lock().unwrap().app.steering_enabled()
    }

    /// Register MusicKit (or mock) segment playback handler for workout intervals.
    pub fn set_audio_director(&self, director: Box<dyn AudioDirectorCallback>) {
        self.inner.lock().unwrap().audio_director = Some(Arc::from(director));
    }

    pub fn clear_audio_director(&self) {
        self.inner.lock().unwrap().audio_director = None;
    }

    pub fn tick(
        &self,
        sensors: Box<dyn SensorSourceCallback>,
        trainer: Box<dyn TrainerControlCallback>,
        steering: Box<dyn SteeringInputCallback>,
    ) {
        let mut inner = self.inner.lock().unwrap();
        let mut sensor = FfiSensorSource { callback: sensors };
        let trainer = FfiTrainerControl { callback: trainer };
        let steer = FfiSteeringInput { callback: steering };
        if let Some(audio_cb) = inner.audio_director.clone() {
            let audio = FfiAudioDirector { callback: audio_cb };
            inner
                .app
                .tick(&mut sensor, &trainer, Some(&steer), Some(&audio));
        } else {
            inner.app.tick(
                &mut sensor,
                &trainer,
                Some(&steer),
                None::<&velo_platform::MockAudioDirector>,
            );
        }
    }

    pub fn recent_logs(&self, limit: u32) -> Vec<String> {
        self.inner.lock().unwrap().app.recent_logs(limit as usize)
    }

    pub fn ride_state(&self) -> RideStateDto {
        let app = &self.inner.lock().unwrap().app;
        let ride = &app.ride;
        let elevation_m = app.route.as_ref().map(|route| {
            let (_, _, elev) = route.lat_lon_elev_at(ride.distance_m);
            elev
        });
        RideStateDto {
            mode: map_ride_mode(ride.mode),
            distance_m: ride.distance_m,
            speed_mps: ride.speed_mps,
            elapsed_s: ride.elapsed_s,
            grade: ride.grade,
            elevation_m,
            power_w: ride.power_w,
            cadence_rpm: ride.cadence_rpm,
            heart_rate_bpm: ride.heart_rate_bpm,
            steer_axis: app.steer_axis(),
            steer_yaw_rad: app.steer_yaw_rad(),
        }
    }

    pub fn init_renderer(
        &self,
        metal_layer_ptr: u64,
        width: u32,
        height: u32,
    ) -> Result<(), VeloError> {
        let ptr = metal_layer_ptr as *mut std::ffi::c_void;
        let mut inner = self.inner.lock().unwrap();
        let mut renderer =
            Renderer::from_metal_layer(ptr, width, height).map_err(|_| VeloError::RenderError)?;
        if let Some(ref bike_id) = inner.active_bike_id {
            if let Ok(asset) = load_bike_asset(&inner.bikes_dir, bike_id) {
                let _ = renderer.load_bike_gltf(&asset.gltf_path, asset.anchor);
            }
        }
        inner.renderer = Some(renderer);
        Ok(())
    }

    pub fn resize_renderer(&self, width: u32, height: u32) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let renderer = inner.renderer.as_mut().ok_or(VeloError::RenderError)?;
        renderer.resize(width, height);
        Ok(())
    }

    /// When false, live frames skip the in-canvas HUD (shell draws Swift overlay).
    pub fn set_hud_draw_enabled(&self, enabled: bool) {
        if let Some(renderer) = self.inner.lock().unwrap().renderer.as_mut() {
            renderer.set_hud_draw_enabled(enabled);
        }
    }

    pub fn render_frame(&self) -> Result<(), VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let ride = inner.app.ride.clone();
        let tiles_on = inner.tiles_3d_enabled;
        let tiles_attr = inner
            .renderer
            .as_ref()
            .filter(|_| tiles_on)
            .map(|r| r.tiles_attribution().to_string());
        let hud = hud_snapshot(&inner.app, tiles_attr);
        let distance_m = ride.distance_m;
        let follow = route_follow(&inner.app);
        let steer_yaw = inner.app.steer_yaw_rad();
        let route_for_tiles = inner.app.route.clone();
        let renderer = inner.renderer.as_mut().ok_or(VeloError::RenderError)?;
        if tiles_on {
            if let Some(route) = &route_for_tiles {
                sync_tiles_view(route, distance_m, renderer);
            }
        }
        renderer
            .render_frame(&hud, distance_m, follow, steer_yaw)
            .map_err(|_| VeloError::RenderError)
    }

    /// Grab the current framebuffer as raw RGBA8 (shell encodes PNG).
    pub fn capture_framebuffer_rgba(&self) -> Result<FramebufferDto, VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let tiles_attr = inner
            .renderer
            .as_ref()
            .filter(|_| inner.tiles_3d_enabled)
            .map(|r| r.tiles_attribution().to_string());
        let hud = hud_snapshot(&inner.app, tiles_attr);
        let distance_m = inner.app.ride.distance_m;
        let follow = route_follow(&inner.app);
        let steer_yaw = inner.app.steer_yaw_rad();
        let renderer = inner.renderer.as_mut().ok_or(VeloError::RenderError)?;
        let fb = renderer
            .capture_framebuffer_rgba(&hud, distance_m, follow, steer_yaw)
            .map_err(|_| VeloError::RenderError)?;
        Ok(FramebufferDto {
            width: fb.width,
            height: fb.height,
            rgba_pixels: fb.pixels,
        })
    }

    /// Stop ride, capture screenshot, export FIT, publish via shell callback.
    pub fn finish_ride_and_publish(
        &self,
        media: Box<dyn MediaCaptureCallback>,
        publisher: Box<dyn ActivityPublisherCallback>,
    ) -> Result<PublishResultDto, VeloError> {
        let mut inner = self.inner.lock().unwrap();
        let summary = inner.app.stop_ride().ok_or(VeloError::RideError {
            message: "no active or completed ride".into(),
        })?;
        let summary_dto = map_summary(summary);

        let fit_bytes = inner.app.export_fit().map_err(|e| VeloError::RideError {
            message: e.to_string(),
        })?;

        let screenshot_png = if inner.renderer.is_some() {
            let ride = inner.app.ride.clone();
            let tiles_attr = inner
                .renderer
                .as_ref()
                .filter(|_| inner.tiles_3d_enabled)
                .map(|r| r.tiles_attribution().to_string());
            let hud = hud_snapshot(&inner.app, tiles_attr);
            let distance_m = ride.distance_m;
            let follow = route_follow(&inner.app);
            let steer_yaw = inner.app.steer_yaw_rad();
            let renderer = inner.renderer.as_mut().ok_or(VeloError::RenderError)?;
            let fb = renderer
                .capture_framebuffer_rgba(&hud, distance_m, follow, steer_yaw)
                .map_err(|_| VeloError::RenderError)?;
            Some(media.encode_png_rgba(fb.width, fb.height, fb.pixels))
        } else {
            None
        };

        let mut publish = publisher.publish_ride(
            fit_bytes.clone(),
            screenshot_png.clone(),
            summary_dto.clone(),
        );
        publish.highlight_clip_path = None;

        if let Some(library) = inner.ride_library.as_ref() {
            let route_id = inner.app.active_route_id().map(|s| s.to_string());
            let ride_id = persist_finished_ride(
                library,
                &summary_dto,
                &fit_bytes,
                screenshot_png.as_deref(),
                &publish,
                route_id,
            )?;
            publish.ride_id = ride_id.clone();

            if !summary_dto.highlight_clips.is_empty() {
                if let Ok(Some(ride)) = library.get_ride(&ride_id) {
                    if let Some(parent) = std::path::Path::new(&ride.fit_path).parent() {
                        let clip_path = parent.join("highlight.mp4");
                        let clip_path_str = clip_path.display().to_string();
                        if media.encode_highlight_clip(
                            summary_dto.highlight_clips.clone(),
                            clip_path_str.clone(),
                        ) {
                            let _ = library.update_highlight_clip_path(&ride_id, &clip_path_str);
                            publish.highlight_clip_path = Some(clip_path_str);
                        }
                    }
                }
            }

            if publish.saved_locally {
                if let Ok(Some(ride)) = library.get_ride(&ride_id) {
                    if let Some(parent) = std::path::Path::new(&ride.fit_path).parent() {
                        publish.activity_url = parent.display().to_string();
                    }
                }
            }
        } else {
            publish.ride_id = String::new();
        }

        Ok(publish)
    }
}

fn route_follow(app: &VeloApp) -> Option<RouteFollow> {
    let route = app.route.as_ref()?;
    let d = app.ride.distance_m;
    let (east, up, north) = route.position_enu_at(d);
    let ahead = 15.0_f64;
    let d2 = (d + ahead).min(route.total_distance_m());
    let (e2, _, n2) = route.position_enu_at(d2);
    let forward = forward_from_enu(east, up, north, e2, n2);
    Some(RouteFollow {
        east,
        up,
        north,
        forward,
    })
}

fn sync_tiles_view(route: &velo_core::RouteModel, distance_m: f64, renderer: &mut Renderer) {
    let (lat, lon, _) = route.lat_lon_elev_at(distance_m);
    renderer.update_tiles_view(lat, lon, 500.0);
}

fn hud_snapshot(app: &VeloApp, attribution: Option<String>) -> velo_render::HudSnapshot {
    let ride = &app.ride;
    let mode = match ride.mode {
        velo_core::ride::RideMode::Free => "Free",
        velo_core::ride::RideMode::Erg => "ERG",
        velo_core::ride::RideMode::Sim => "SIM",
    };
    let (workout_interval, workout_target_w, interval_duration_s, interval_elapsed_s) =
        match app.workout_engine.as_ref() {
            Some(engine) if !engine.state().finished => {
                let interval = engine.current_interval();
                (
                    interval.map(|i| i.name.clone()),
                    engine.target_watts().map(|w| w.0),
                    interval.map(|i| i.duration_s),
                    Some(engine.state().interval_elapsed_s),
                )
            }
            _ => (None, None, None, None),
        };
    let elevation_m = app.route.as_ref().map(|route| {
        let (_, _, elev) = route.lat_lon_elev_at(ride.distance_m);
        elev
    });
    velo_render::HudSnapshot {
        power_w: ride.power_w,
        cadence_rpm: ride.cadence_rpm,
        heart_rate_bpm: ride.heart_rate_bpm,
        speed_mps: ride.speed_mps,
        distance_m: ride.distance_m,
        elapsed_s: ride.elapsed_s,
        grade: ride.grade,
        elevation_m,
        mode,
        workout_interval,
        workout_target_w,
        interval_duration_s,
        interval_elapsed_s,
        attribution,
    }
}

fn map_workout_target_dto(target: WorkoutTargetDto) -> WorkoutTarget {
    match target {
        WorkoutTargetDto::ErgWatts { watts } => WorkoutTarget::ErgWatts(watts),
        WorkoutTargetDto::FtpPercent { percent } => WorkoutTarget::FtpPercent(percent),
        WorkoutTargetDto::FreeRide => WorkoutTarget::FreeRide,
    }
}

fn map_workout_dto(dto: WorkoutDto) -> Result<Workout, VeloError> {
    Ok(Workout {
        name: dto.name,
        intervals: dto
            .intervals
            .into_iter()
            .map(|i| WorkoutInterval {
                name: i.name,
                duration_s: i.duration_s,
                target: map_workout_target_dto(i.target),
            })
            .collect(),
    })
}

fn map_workout_to_dto(workout: Workout) -> WorkoutDto {
    WorkoutDto {
        name: workout.name,
        intervals: workout
            .intervals
            .into_iter()
            .map(|i| WorkoutIntervalDto {
                name: i.name,
                duration_s: i.duration_s,
                target: match i.target {
                    WorkoutTarget::ErgWatts(w) => WorkoutTargetDto::ErgWatts { watts: w },
                    WorkoutTarget::FtpPercent(p) => WorkoutTargetDto::FtpPercent { percent: p },
                    WorkoutTarget::FreeRide => WorkoutTargetDto::FreeRide,
                },
            })
            .collect(),
    }
}

#[uniffi::export]
pub fn parse_zwo_xml(xml: String) -> Result<WorkoutDto, VeloError> {
    let workout = parse_zwo_xml_core(&xml).map_err(|e| VeloError::RideError {
        message: e.to_string(),
    })?;
    workout
        .validate()
        .map_err(|message| VeloError::RideError { message })?;
    Ok(map_workout_to_dto(workout))
}

fn map_workout_live(app: &VeloApp) -> WorkoutLiveDto {
    let Some(engine) = app.workout_engine.as_ref() else {
        return WorkoutLiveDto::default();
    };
    let state = engine.state();
    let interval = engine.current_interval();
    WorkoutLiveDto {
        active: true,
        workout_name: engine.workout().name.clone(),
        interval_name: interval.map(|i| i.name.clone()).unwrap_or_default(),
        interval_elapsed_s: state.interval_elapsed_s,
        interval_duration_s: interval.map(|i| i.duration_s).unwrap_or(0.0),
        workout_elapsed_s: state.workout_elapsed_s,
        target_watts: engine.target_watts().map(|w| w.0),
        finished: state.finished,
    }
}

#[uniffi::export]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
