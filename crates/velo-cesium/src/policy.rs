use std::path::Path;

use thiserror::Error;

/// ToS guardrails: Tier B tiles are online-only and must never touch disk.
#[derive(Debug, Default)]
pub struct OnlineOnlyPolicy {
    disk_writes: u32,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PolicyError {
    #[error("disk cache prohibited by 3D Tiles ToS")]
    DiskCacheProhibited,
}

impl OnlineOnlyPolicy {
    pub fn new() -> Self {
        Self { disk_writes: 0 }
    }

    /// Reject any attempt to persist tile bytes.
    pub fn check_no_disk_write(&self) -> Result<(), PolicyError> {
        if self.disk_writes > 0 {
            Err(PolicyError::DiskCacheProhibited)
        } else {
            Ok(())
        }
    }

    /// Guard a write path — always fails for tile payloads.
    pub fn guard_write_path(&self, path: &Path) -> Result<(), PolicyError> {
        let _ = path;
        Err(PolicyError::DiskCacheProhibited)
    }

    #[cfg(test)]
    pub(crate) fn record_disk_write_for_test(&mut self) {
        self.disk_writes += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disk_write_rejected() {
        let mut policy = OnlineOnlyPolicy::new();
        assert!(policy.check_no_disk_write().is_ok());
        policy.record_disk_write_for_test();
        assert_eq!(policy.check_no_disk_write(), Err(PolicyError::DiskCacheProhibited));
    }

    #[test]
    fn guard_write_path_always_fails() {
        let policy = OnlineOnlyPolicy::new();
        assert_eq!(
            policy.guard_write_path(std::path::Path::new("/tmp/tile.glb")),
            Err(PolicyError::DiskCacheProhibited)
        );
    }
}
