pub mod app;
pub mod packs;
pub mod physics;
pub mod ride;
pub mod ride_session;
pub mod route;

pub use app::VeloApp;
pub use packs::{default_packs_dir, list_route_packs, load_route_pack, pack_dir_for_id};
pub use physics::{integrate_step, PhysicsConfig, RideSnapshot};
pub use ride::{RideMode, RideState};
pub use ride_session::{RideSample, RideSession, RideSummary};
pub use route::{haversine_m, lat_lon_to_local, RouteError, RouteMeta, RouteModel, RouteOrigin, RoutePoint};
