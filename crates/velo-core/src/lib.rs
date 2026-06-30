pub mod app;
pub mod highlight;
pub mod packs;
pub mod physics;
pub mod ride;
pub mod ride_session;
pub mod route;
pub mod workout;
pub mod zwo;

pub use app::VeloApp;
pub use packs::{
    default_packs_dir, list_route_packs, load_route_pack, load_scenery_config, pack_dir_for_id,
    save_scenery_config, SceneryConfig, SCENERY_FILE,
};
pub use physics::{integrate_step, PhysicsConfig, RideSnapshot};
pub use ride::{RideMode, RideState};
pub use highlight::{plan_highlight_clips, HighlightClipRequest};
pub use ride_session::{RideSample, RideSession, RideSummary};
pub use route::{haversine_m, lat_lon_to_local, RouteError, RouteMeta, RouteModel, RouteOrigin, RoutePoint};
pub use workout::{
    Workout, WorkoutEngine, WorkoutInterval, WorkoutState, WorkoutTarget,
};
pub use zwo::{parse_zwo_xml, ZwoError};
