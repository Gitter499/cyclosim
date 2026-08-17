//! Runtime bikegen provider credentials (shell → FFI).

use std::sync::{Mutex, OnceLock};

static BIKEGEN_CREDENTIALS: OnceLock<Mutex<BikegenCredentials>> = OnceLock::new();

fn store() -> &'static Mutex<BikegenCredentials> {
    BIKEGEN_CREDENTIALS.get_or_init(|| Mutex::new(BikegenCredentials::default()))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BikegenCredentials {
    pub meshy_api_key: Option<String>,
    /// When true, import requires a configured hosted API key (Meshy scaffold).
    pub prefer_hosted_generation: bool,
}

impl BikegenCredentials {
    pub fn meshy_key(&self) -> Option<String> {
        self.meshy_api_key
            .clone()
            .filter(|k| !k.trim().is_empty())
            .or_else(|| {
                std::env::var("MESHY_API_KEY")
                    .ok()
                    .filter(|k| !k.trim().is_empty())
            })
    }
}

pub fn set_bikegen_credentials(creds: BikegenCredentials) {
    *store().lock().unwrap() = creds;
}

pub fn bikegen_credentials() -> BikegenCredentials {
    store().lock().unwrap().clone()
}

/// Status string for shell UI.
pub fn bikegen_mode_status() -> String {
    let creds = bikegen_credentials();
    if creds.prefer_hosted_generation {
        if creds.meshy_key().is_some() {
            return "Hosted generation (Meshy key set — API wiring deferred; using placeholder)"
                .into();
        }
        return "Hosted generation requires Meshy API key — configure in Settings".into();
    }
    "Offline placeholder (1–4 photos → synthetic glTF)".into()
}

/// Returns an error message when hosted mode is enabled but no key is configured.
pub fn hosted_import_gate_error() -> Option<String> {
    let creds = bikegen_credentials();
    if creds.prefer_hosted_generation && creds.meshy_key().is_none() {
        Some("Meshy API key required for hosted bike generation — open Settings".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hosted_gate_blocks_without_key() {
        set_bikegen_credentials(BikegenCredentials {
            meshy_api_key: None,
            prefer_hosted_generation: true,
        });
        assert!(hosted_import_gate_error().is_some());
        set_bikegen_credentials(BikegenCredentials::default());
    }

    #[test]
    fn placeholder_mode_always_allowed() {
        set_bikegen_credentials(BikegenCredentials::default());
        assert!(hosted_import_gate_error().is_none());
    }
}
