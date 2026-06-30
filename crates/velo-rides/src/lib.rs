//! SQLite ride library — metadata catalog with on-disk artifact paths.

mod error;
pub mod schema;
mod store;

pub use error::{Result, RideStoreError};
pub use store::{NewRideRecord, PublishStatus, RideArtifacts, RideLibrary, RideRecord};

/// Default database path: `~/Documents/VeloSim/rides.db`.
pub fn default_db_path() -> std::path::PathBuf {
    default_velosim_dir().join("rides.db")
}

/// Default artifacts base: `~/Documents/VeloSim/rides/`.
pub fn default_artifacts_base() -> std::path::PathBuf {
    default_velosim_dir().join("rides")
}

fn default_velosim_dir() -> std::path::PathBuf {
    documents_dir().join("VeloSim")
}

fn documents_dir() -> std::path::PathBuf {
    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join("Documents"))
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}
