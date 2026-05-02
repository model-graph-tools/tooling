//! Serializable types for JSON output of container commands.

use serde::Serialize;

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
        };
        let json_str = serde_json::to_string(&result).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(json["identifier"], "39.0");
        assert_eq!(json["success"], true);
        assert!(json.get("bolt").is_none());
        assert!(json.get("http").is_none());
        assert!(json.get("error").is_none());
    }

    #[test]
    fn command_result_start_success_includes_ports() {
        let result = CommandResult {
            identifier: "39.0".into(),
            success: true,
            bolt: Some(6390),
            http: Some(7390),
            error: None,
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
        };
        let json: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&result).unwrap()).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["error"], "Failed to pull image: not found");
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
            },
            CommandResult {
                identifier: "99.0".into(),
                success: false,
                bolt: None,
                http: None,
                error: Some("not found".into()),
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
}
