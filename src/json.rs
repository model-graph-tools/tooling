//! Serializable types for JSON output of container commands.

use crate::error::{MgtError, MgtErrorCode};
use serde::Serialize;
use wildfly_meta::{UpdateResult as MetaUpdateResult, UpdateStatus as MetaUpdateStatus};

/// Running container info for `mgt ps --json`.
#[derive(Serialize)]
pub struct ContainerInfo {
    pub identifier: String,
    pub source_type: String,
    pub name: String,
    pub container_name: String,
    pub bolt: u16,
    pub http: u16,
    pub status: String,
    pub id: String,
}

/// Resolved identifier for `mgt resolve --json`.
#[derive(Serialize)]
pub struct ResolveResult {
    pub identifier: String,
    pub source_type: String,
    pub name: String,
}

/// Command result for `mgt start --json` and `mgt stop --json`.
#[derive(Serialize)]
pub struct CommandResult {
    pub identifier: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bolt: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<MgtErrorCode>,
}

impl CommandResult {
    pub fn success(identifier: String, bolt: Option<u16>, http: Option<u16>) -> Self {
        Self {
            identifier,
            success: true,
            bolt,
            http,
            error: None,
            error_code: None,
        }
    }

    pub fn error(identifier: String, err: &anyhow::Error) -> Self {
        Self {
            identifier,
            success: false,
            bolt: None,
            http: None,
            error_code: Some(MgtError::error_code(err)),
            error: Some(err.to_string()),
        }
    }
}

/// JSON output for a single update status (WildFly images or feature packs).
#[derive(Serialize)]
pub struct UpdateStatusResult {
    pub status: String,
    pub version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_version: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub added: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<String>,
}

impl From<&MetaUpdateStatus> for UpdateStatusResult {
    fn from(status: &MetaUpdateStatus) -> Self {
        match status {
            MetaUpdateStatus::Downloaded { version, .. } => Self {
                status: "downloaded".into(),
                version: *version,
                from_version: None,
                added: Vec::new(),
                removed: Vec::new(),
            },
            MetaUpdateStatus::Updated {
                from_version,
                to_version,
                diff,
            } => Self {
                status: "updated".into(),
                version: *to_version,
                from_version: Some(*from_version),
                added: diff.added.clone(),
                removed: diff.removed.clone(),
            },
            MetaUpdateStatus::AlreadyUpToDate(version) => Self {
                status: "up_to_date".into(),
                version: *version,
                from_version: None,
                added: Vec::new(),
                removed: Vec::new(),
            },
        }
    }
}

/// Combined JSON output for `mgt update --json`.
#[derive(Serialize)]
pub struct UpdateResult {
    pub wildfly_images: UpdateStatusResult,
    pub feature_packs: UpdateStatusResult,
}

impl From<&MetaUpdateResult> for UpdateResult {
    fn from(result: &MetaUpdateResult) -> Self {
        Self {
            wildfly_images: UpdateStatusResult::from(&result.wildfly_images),
            feature_packs: UpdateStatusResult::from(&result.feature_packs),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_info_serializes_all_fields() {
        let info = ContainerInfo {
            identifier: "39.0".into(),
            source_type: "wildfly".into(),
            name: "WildFly 39.0".into(),
            container_name: "mgt-neo4j-390".into(),
            bolt: 6390,
            http: 7390,
            status: "Up 2 hours".into(),
            id: "abc123".into(),
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&info).unwrap()).unwrap();
        assert_eq!(json["identifier"], "39.0");
        assert_eq!(json["source_type"], "wildfly");
        assert_eq!(json["name"], "WildFly 39.0");
        assert_eq!(json["container_name"], "mgt-neo4j-390");
        assert_eq!(json["bolt"], 6390);
        assert_eq!(json["http"], 7390);
        assert_eq!(json["status"], "Up 2 hours");
        assert_eq!(json["id"], "abc123");
    }

    #[test]
    fn container_info_array_serializes_as_json_array() {
        let infos = vec![
            ContainerInfo {
                identifier: "39.0".into(),
                source_type: "wildfly".into(),
                name: "WildFly 39.0".into(),
                container_name: "mgt-neo4j-390".into(),
                bolt: 6390,
                http: 7390,
                status: "Up 2 hours".into(),
                id: "abc123".into(),
            },
            ContainerInfo {
                identifier: "ai:1.0.0".into(),
                source_type: "feature-pack".into(),
                name: "AI Feature Pack 1.0.0".into(),
                container_name: "mgt-neo4j-ai-1-0-0".into(),
                bolt: 6100,
                http: 7100,
                status: "Up 5 minutes".into(),
                id: "def456".into(),
            },
        ];
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&infos).unwrap()).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["identifier"], "39.0");
        assert_eq!(json[1]["identifier"], "ai:1.0.0");
    }

    #[test]
    fn command_result_success_omits_optional_fields() {
        let result = CommandResult {
            identifier: "39.0".into(),
            success: true,
            bolt: None,
            http: None,
            error: None,
            error_code: None,
        };
        let json_str = serde_json::to_string(&result).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(json["identifier"], "39.0");
        assert_eq!(json["success"], true);
        assert!(json.get("bolt").is_none());
        assert!(json.get("http").is_none());
        assert!(json.get("error").is_none());
        assert!(json.get("error_code").is_none());
    }

    #[test]
    fn command_result_start_success_includes_ports() {
        let result = CommandResult {
            identifier: "39.0".into(),
            success: true,
            bolt: Some(6390),
            http: Some(7390),
            error: None,
            error_code: None,
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["bolt"], 6390);
        assert_eq!(json["http"], 7390);
        assert!(json.get("error").is_none());
    }

    #[test]
    fn command_result_error_includes_message() {
        let result = CommandResult {
            identifier: "99.0".into(),
            success: false,
            bolt: None,
            http: None,
            error: Some("Failed to pull image: not found".into()),
            error_code: Some(MgtErrorCode::ImagePullFailed),
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "Failed to pull image: not found");
        assert_eq!(json["error_code"], "IMAGE_PULL_FAILED");
        assert!(json.get("bolt").is_none());
    }

    #[test]
    fn command_result_array_serializes_mixed_results() {
        let results = vec![
            CommandResult {
                identifier: "39.0".into(),
                success: true,
                bolt: Some(6390),
                http: Some(7390),
                error: None,
                error_code: None,
            },
            CommandResult {
                identifier: "99.0".into(),
                success: false,
                bolt: None,
                http: None,
                error: Some("not found".into()),
                error_code: Some(MgtErrorCode::Internal),
            },
        ];
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&results).unwrap()).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["success"], true);
        assert_eq!(json[0]["bolt"], 6390);
        assert_eq!(json[1]["success"], false);
        assert_eq!(json[1]["error"], "not found");
    }

    #[test]
    fn empty_array_serializes_to_empty_json_array() {
        let results: Vec<CommandResult> = vec![];
        let json_str = serde_json::to_string(&results).unwrap();
        assert_eq!(json_str, "[]");
    }

    #[test]
    fn resolve_result_serializes_wildfly() {
        let result = ResolveResult {
            identifier: "39.0".into(),
            source_type: "wildfly".into(),
            name: "WildFly 39.0".into(),
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["identifier"], "39.0");
        assert_eq!(json["source_type"], "wildfly");
        assert_eq!(json["name"], "WildFly 39.0");
    }

    #[test]
    fn resolve_result_serializes_feature_pack() {
        let result = ResolveResult {
            identifier: "ai:0.9.1".into(),
            source_type: "feature-pack".into(),
            name: "AI 0.9.1".into(),
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["identifier"], "ai:0.9.1");
        assert_eq!(json["source_type"], "feature-pack");
        assert_eq!(json["name"], "AI 0.9.1");
    }

    #[test]
    fn resolve_result_array_serializes() {
        let results = vec![
            ResolveResult {
                identifier: "39.0".into(),
                source_type: "wildfly".into(),
                name: "WildFly 39.0".into(),
            },
            ResolveResult {
                identifier: "ai:0.9.1".into(),
                source_type: "feature-pack".into(),
                name: "AI 0.9.1".into(),
            },
        ];
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&results).unwrap()).unwrap();
        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 2);
        assert_eq!(json[0]["identifier"], "39.0");
        assert_eq!(json[1]["identifier"], "ai:0.9.1");
    }

    #[test]
    fn update_result_up_to_date() {
        let meta = MetaUpdateResult {
            wildfly_images: MetaUpdateStatus::AlreadyUpToDate(2),
            feature_packs: MetaUpdateStatus::AlreadyUpToDate(4),
        };
        let result = UpdateResult::from(&meta);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["wildfly_images"]["status"], "up_to_date");
        assert_eq!(json["wildfly_images"]["version"], 2);
        assert!(json["wildfly_images"].get("from_version").is_none());
        assert_eq!(json["feature_packs"]["status"], "up_to_date");
        assert_eq!(json["feature_packs"]["version"], 4);
    }

    #[test]
    fn update_result_downloaded() {
        let meta = MetaUpdateResult {
            wildfly_images: MetaUpdateStatus::Downloaded {
                version: 1,
                count: 10,
            },
            feature_packs: MetaUpdateStatus::Downloaded {
                version: 1,
                count: 5,
            },
        };
        let result = UpdateResult::from(&meta);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["wildfly_images"]["status"], "downloaded");
        assert_eq!(json["wildfly_images"]["version"], 1);
        assert_eq!(json["feature_packs"]["status"], "downloaded");
    }

    #[test]
    fn update_result_updated_with_diff() {
        use wildfly_meta::UpdateDiff;
        let meta = MetaUpdateResult {
            wildfly_images: MetaUpdateStatus::Updated {
                from_version: 1,
                to_version: 2,
                diff: UpdateDiff {
                    added: vec!["WildFly 40".into()],
                    removed: vec![],
                },
            },
            feature_packs: MetaUpdateStatus::Updated {
                from_version: 3,
                to_version: 4,
                diff: UpdateDiff {
                    added: vec!["AI 1.0.0".into()],
                    removed: vec!["AI 0.8.0".into()],
                },
            },
        };
        let result = UpdateResult::from(&meta);
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["wildfly_images"]["status"], "updated");
        assert_eq!(json["wildfly_images"]["version"], 2);
        assert_eq!(json["wildfly_images"]["from_version"], 1);
        assert_eq!(json["wildfly_images"]["added"][0], "WildFly 40");
        assert_eq!(json["feature_packs"]["added"][0], "AI 1.0.0");
        assert_eq!(json["feature_packs"]["removed"][0], "AI 0.8.0");
    }
}
