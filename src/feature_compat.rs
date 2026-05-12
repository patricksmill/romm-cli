use serde::Serialize;

use crate::openapi::EndpointRegistry;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct RequiredEndpoint {
    pub method: &'static str,
    pub path: &'static str,
}

impl RequiredEndpoint {
    pub fn label(self) -> String {
        format!("{} {}", self.method, self.path)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FeatureCompatibility {
    pub feature: &'static str,
    pub supported: bool,
    pub missing: Vec<RequiredEndpoint>,
    unsupported_message: &'static str,
}

impl FeatureCompatibility {
    pub fn from_registry(
        feature: &'static str,
        unsupported_message: &'static str,
        required: &[RequiredEndpoint],
        registry: &EndpointRegistry,
    ) -> Self {
        let missing = required
            .iter()
            .copied()
            .filter(|ep| !registry.has_endpoint(ep.method, ep.path))
            .collect::<Vec<_>>();
        Self {
            feature,
            supported: missing.is_empty(),
            missing,
            unsupported_message,
        }
    }

    pub fn supported(feature: &'static str, unsupported_message: &'static str) -> Self {
        Self {
            feature,
            supported: true,
            missing: Vec::new(),
            unsupported_message,
        }
    }

    pub fn unsupported_message(&self) -> String {
        if self.missing.is_empty() {
            self.unsupported_message.to_string()
        } else {
            format!(
                "{} Missing endpoint(s): {}",
                self.unsupported_message,
                self.missing
                    .iter()
                    .map(|ep| ep.label())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

pub const SAVE_SYNC_FEATURE: &str = "save-sync";

pub const SAVE_SYNC_UNSUPPORTED_MESSAGE: &str =
    "This RomM server does not expose save-sync endpoints; upgrade RomM to use romm-cli sync.";

pub const SAVE_SYNC_REQUIRED_ENDPOINTS: [RequiredEndpoint; 6] = [
    RequiredEndpoint {
        method: "GET",
        path: "/api/devices",
    },
    RequiredEndpoint {
        method: "POST",
        path: "/api/devices",
    },
    RequiredEndpoint {
        method: "GET",
        path: "/api/sync/sessions",
    },
    RequiredEndpoint {
        method: "POST",
        path: "/api/sync/negotiate",
    },
    RequiredEndpoint {
        method: "POST",
        path: "/api/sync/sessions/{session_id}/complete",
    },
    RequiredEndpoint {
        method: "POST",
        path: "/api/sync/devices/{device_id}/push-pull",
    },
];

pub type SaveSyncCompatibility = FeatureCompatibility;

pub fn save_sync_compatibility(registry: &EndpointRegistry) -> SaveSyncCompatibility {
    FeatureCompatibility::from_registry(
        SAVE_SYNC_FEATURE,
        SAVE_SYNC_UNSUPPORTED_MESSAGE,
        &SAVE_SYNC_REQUIRED_ENDPOINTS,
        registry,
    )
}

pub fn supported_save_sync_compatibility() -> SaveSyncCompatibility {
    FeatureCompatibility::supported(SAVE_SYNC_FEATURE, SAVE_SYNC_UNSUPPORTED_MESSAGE)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry_for(paths: &[(&str, &str)]) -> EndpointRegistry {
        let mut paths_json = serde_json::Map::new();
        for (method, path) in paths {
            let entry = paths_json
                .entry((*path).to_string())
                .or_insert_with(|| serde_json::json!({}));
            entry.as_object_mut().unwrap().insert(
                method.to_ascii_lowercase(),
                serde_json::json!({ "responses": { "200": { "description": "ok" } } }),
            );
        }
        EndpointRegistry::from_openapi_json(
            &serde_json::json!({
                "openapi": "3.0.0",
                "paths": paths_json
            })
            .to_string(),
        )
        .expect("parse")
    }

    #[test]
    fn feature_compatibility_supported_when_all_required_endpoints_exist() {
        let required = [
            RequiredEndpoint {
                method: "GET",
                path: "/api/a",
            },
            RequiredEndpoint {
                method: "POST",
                path: "/api/b/{id}",
            },
        ];
        let compat = FeatureCompatibility::from_registry(
            "demo",
            "Demo feature unsupported.",
            &required,
            &registry_for(&[("GET", "/api/a"), ("POST", "/api/b/{id}")]),
        );

        assert!(compat.supported);
        assert!(compat.missing.is_empty());
    }

    #[test]
    fn feature_compatibility_reports_missing_endpoints() {
        let required = [
            RequiredEndpoint {
                method: "GET",
                path: "/api/a",
            },
            RequiredEndpoint {
                method: "POST",
                path: "/api/b",
            },
        ];
        let compat = FeatureCompatibility::from_registry(
            "demo",
            "Demo feature unsupported.",
            &required,
            &registry_for(&[("GET", "/api/a")]),
        );

        assert!(!compat.supported);
        assert_eq!(
            compat.missing,
            vec![RequiredEndpoint {
                method: "POST",
                path: "/api/b"
            }]
        );
        assert_eq!(
            compat.unsupported_message(),
            "Demo feature unsupported. Missing endpoint(s): POST /api/b"
        );
    }

    #[test]
    fn save_sync_compatibility_supported_when_all_required_endpoints_exist() {
        let paths = SAVE_SYNC_REQUIRED_ENDPOINTS
            .iter()
            .map(|ep| (ep.method, ep.path))
            .collect::<Vec<_>>();
        let compat = save_sync_compatibility(&registry_for(&paths));

        assert_eq!(compat.feature, SAVE_SYNC_FEATURE);
        assert!(compat.supported);
        assert!(compat.missing.is_empty());
    }

    #[test]
    fn save_sync_compatibility_reports_partial_missing_endpoints() {
        let compat = save_sync_compatibility(&registry_for(&[("GET", "/api/devices")]));

        assert!(!compat.supported);
        assert_eq!(compat.missing.len(), SAVE_SYNC_REQUIRED_ENDPOINTS.len() - 1);
        assert!(compat
            .unsupported_message()
            .contains("POST /api/sync/negotiate"));
    }

    #[test]
    fn save_sync_compatibility_empty_registry_is_unsupported() {
        let compat = save_sync_compatibility(&EndpointRegistry::default());

        assert!(!compat.supported);
        assert_eq!(compat.missing.len(), SAVE_SYNC_REQUIRED_ENDPOINTS.len());
    }
}
