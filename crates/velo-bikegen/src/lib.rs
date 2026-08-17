//! Offline bike asset pipeline: 1–4 photos → normalized glTF in the bike library.

pub mod asset;
pub mod credentials;
pub mod library;
pub mod placeholder;

pub use asset::{AnchorTransform, BikeAsset, BikeMeta};
pub use credentials::{
    bikegen_credentials, bikegen_mode_status, hosted_import_gate_error, set_bikegen_credentials,
    BikegenCredentials,
};
pub use library::{
    bike_dir_for_id, default_bikes_dir, import_bike_from_images, list_bikes, load_bike_asset,
    BikeImportError, BikeSummary,
};
