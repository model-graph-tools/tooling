//! Structured error types for the `mgt` CLI.
//!
//! Every error carries a stable [`MgtErrorCode`] for machine consumption and a
//! human-readable message for terminal users. When `--json` is active, top-level
//! errors are emitted as a [`JsonErrorEnvelope`] on stdout.

use serde::Serialize;

/// Stable, machine-parseable error codes emitted in JSON mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MgtErrorCode {
    ContainerRuntimeNotFound,
    ContainerCommandFailed,
    NetworkCreateFailed,
    ImagePullFailed,
    ImageListFailed,
    ContainerListFailed,
    HealthcheckFailed,
    ContainerStartFailed,
    RegistryInitFailed,
    UnknownIdentifier,
    ClapParseError,
    Internal,
}

/// A typed error that carries both a stable code and a human-readable message.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct MgtError {
    pub code: MgtErrorCode,
    pub message: String,
}

impl MgtError {
    pub fn container_runtime_not_found() -> Self {
        Self {
            code: MgtErrorCode::ContainerRuntimeNotFound,
            message: "podman or docker not found".into(),
        }
    }

    pub fn network_create_failed(name: &str, stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::NetworkCreateFailed,
            message: format!("Failed to create network {name}: {stderr}"),
        }
    }

    pub fn container_command_failed(context: &str, stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::ContainerCommandFailed,
            message: format!("{context}: {stderr}"),
        }
    }

    pub fn container_list_failed(stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::ContainerListFailed,
            message: format!("Failed to list containers: {stderr}"),
        }
    }

    pub fn image_list_failed(stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::ImageListFailed,
            message: format!("Failed to list images: {stderr}"),
        }
    }

    pub fn image_pull_failed(image: &str, stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::ImagePullFailed,
            message: format!("Failed to pull image {image}: {stderr}"),
        }
    }

    pub fn healthcheck_failed(url: &str, attempts: u32) -> Self {
        Self {
            code: MgtErrorCode::HealthcheckFailed,
            message: format!("Healthcheck failed after {attempts} attempts: {url}"),
        }
    }

    pub fn container_start_failed(stderr: &str) -> Self {
        Self {
            code: MgtErrorCode::ContainerStartFailed,
            message: format!("Failed to start Neo4J: {stderr}"),
        }
    }

    pub fn registry_init_failed(details: &str) -> Self {
        Self {
            code: MgtErrorCode::RegistryInitFailed,
            message: format!("Failed to initialize registries: {details}"),
        }
    }

    pub fn unknown_identifier(input: &str) -> Self {
        Self {
            code: MgtErrorCode::UnknownIdentifier,
            message: format!(
                "\"{input}\" is not a known WildFly version or feature pack. \
                 Use 'mgt versions' and 'mgt feature-packs' to list available identifiers."
            ),
        }
    }

    pub fn clap_parse_error(details: &str) -> Self {
        Self {
            code: MgtErrorCode::ClapParseError,
            message: details.to_string(),
        }
    }

    pub fn internal(details: &str) -> Self {
        Self {
            code: MgtErrorCode::Internal,
            message: details.to_string(),
        }
    }

    pub fn error_code(err: &anyhow::Error) -> MgtErrorCode {
        err.downcast_ref::<MgtError>()
            .map(|e| e.code)
            .unwrap_or(MgtErrorCode::Internal)
    }
}

/// Top-level JSON error envelope emitted on stdout when `--json` is active.
#[derive(Serialize)]
pub struct JsonErrorEnvelope {
    pub error: JsonErrorBody,
}

/// Body of a [`JsonErrorEnvelope`].
#[derive(Serialize)]
pub struct JsonErrorBody {
    pub code: MgtErrorCode,
    pub message: String,
}

impl JsonErrorEnvelope {
    pub fn from_mgt_error(err: &MgtError) -> Self {
        Self {
            error: JsonErrorBody {
                code: err.code,
                message: err.message.clone(),
            },
        }
    }

    pub fn from_anyhow(err: &anyhow::Error) -> Self {
        match err.downcast_ref::<MgtError>() {
            Some(mgt) => Self::from_mgt_error(mgt),
            None => Self {
                error: JsonErrorBody {
                    code: MgtErrorCode::Internal,
                    message: err.to_string(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_serializes_as_screaming_snake_case() {
        let json = serde_json::to_string(&MgtErrorCode::ContainerRuntimeNotFound).unwrap();
        assert_eq!(json, "\"CONTAINER_RUNTIME_NOT_FOUND\"");
    }

    #[test]
    fn error_code_unknown_identifier() {
        let json = serde_json::to_string(&MgtErrorCode::UnknownIdentifier).unwrap();
        assert_eq!(json, "\"UNKNOWN_IDENTIFIER\"");
    }

    #[test]
    fn mgt_error_display_uses_message() {
        let err = MgtError::container_runtime_not_found();
        assert_eq!(err.to_string(), "podman or docker not found");
    }

    #[test]
    fn mgt_error_parameterized_message() {
        let err = MgtError::image_pull_failed("quay.io/mgt/model:99", "manifest unknown");
        assert_eq!(
            err.to_string(),
            "Failed to pull image quay.io/mgt/model:99: manifest unknown"
        );
    }

    #[test]
    fn json_error_envelope_from_mgt_error() {
        let err = MgtError::unknown_identifier("99");
        let envelope = JsonErrorEnvelope::from_mgt_error(&err);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(json["error"]["code"], "UNKNOWN_IDENTIFIER");
        assert!(json["error"]["message"].as_str().unwrap().contains("99"));
    }

    #[test]
    fn json_error_envelope_from_anyhow_with_mgt_error() {
        let err: anyhow::Error = MgtError::registry_init_failed("network timeout").into();
        let envelope = JsonErrorEnvelope::from_anyhow(&err);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(json["error"]["code"], "REGISTRY_INIT_FAILED");
        assert!(json["error"]["message"]
            .as_str()
            .unwrap()
            .contains("network timeout"));
    }

    #[test]
    fn json_error_envelope_from_anyhow_without_mgt_error() {
        let err = anyhow::anyhow!("something unexpected");
        let envelope = JsonErrorEnvelope::from_anyhow(&err);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&envelope).unwrap()).unwrap();
        assert_eq!(json["error"]["code"], "INTERNAL");
        assert_eq!(json["error"]["message"], "something unexpected");
    }

    #[test]
    fn error_code_extracts_from_anyhow() {
        let err: anyhow::Error = MgtError::healthcheck_failed("http://localhost:7474", 120).into();
        assert_eq!(MgtError::error_code(&err), MgtErrorCode::HealthcheckFailed);
    }

    #[test]
    fn error_code_falls_back_to_internal() {
        let err = anyhow::anyhow!("plain error");
        assert_eq!(MgtError::error_code(&err), MgtErrorCode::Internal);
    }
}
