//! Offline bike asset pipeline: 1–4 photos → normalized glTF in the bike library.

pub mod asset;
pub mod library;
pub mod placeholder;

pub use asset::{AnchorTransform, BikeAsset, BikeMeta};
pub use library::{
    bike_dir_for_id, default_bikes_dir, import_bike_from_images, list_bikes, load_bike_asset,
    BikeImportError, BikeSummary,
};
