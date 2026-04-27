use anyhow::bail;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturePack {
    pub shortcut: &'static str,
    pub group_id: &'static str,
    pub artifact_id: &'static str,
    pub id: u16,
    pub version: String,
}

struct FeaturePackDefinition {
    shortcut: &'static str,
    group_id: &'static str,
    artifact_id: &'static str,
    id: u16,
}

static FEATURE_PACKS: &[FeaturePackDefinition] = &[
    FeaturePackDefinition {
        shortcut: "wildfly",
        group_id: "org.wildfly",
        artifact_id: "wildfly-galleon-pack",
        id: 1,
    },
    FeaturePackDefinition {
        shortcut: "cloud",
        group_id: "org.wildfly.cloud",
        artifact_id: "wildfly-cloud-galleon-pack",
        id: 2,
    },
    FeaturePackDefinition {
        shortcut: "datasources",
        group_id: "org.wildfly",
        artifact_id: "wildfly-datasources-galleon-pack",
        id: 3,
    },
    FeaturePackDefinition {
        shortcut: "keycloak",
        group_id: "org.keycloak",
        artifact_id: "keycloak-saml-adapter-galleon-pack",
        id: 4,
    },
    FeaturePackDefinition {
        shortcut: "grpc",
        group_id: "org.wildfly.extras.grpc",
        artifact_id: "wildfly-grpc-feature-pack",
        id: 5,
    },
    FeaturePackDefinition {
        shortcut: "myfaces",
        group_id: "org.wildfly",
        artifact_id: "wildfly-myfaces-feature-pack",
        id: 6,
    },
    FeaturePackDefinition {
        shortcut: "graphql",
        group_id: "org.wildfly.extras.graphql",
        artifact_id: "wildfly-microprofile-graphql-feature-pack",
        id: 7,
    },
];

impl FeaturePack {
    pub fn parse(input: &str) -> anyhow::Result<FeaturePack> {
        let Some((shortcut, version)) = input.split_once(':') else {
            bail!(
                "Invalid feature pack '{}'. Expected format: <shortcut>:<version> (e.g. cloud:9.0.0.Final). \
                 Known shortcuts: {}",
                input,
                known_shortcuts().join(", ")
            );
        };
        if version.is_empty() {
            bail!(
                "Missing version for feature pack '{}'. Expected format: {}:<version>",
                shortcut,
                shortcut
            );
        }
        let Some(def) = FEATURE_PACKS.iter().find(|fp| fp.shortcut == shortcut) else {
            bail!(
                "Unknown feature pack '{}'. Known shortcuts: {}",
                shortcut,
                known_shortcuts().join(", ")
            );
        };
        Ok(FeaturePack {
            shortcut: def.shortcut,
            group_id: def.group_id,
            artifact_id: def.artifact_id,
            id: def.id,
            version: version.to_string(),
        })
    }

    pub fn from_shortcut(shortcut: &str) -> Option<FeaturePack> {
        FEATURE_PACKS.iter().find(|fp| fp.shortcut == shortcut).map(|def| FeaturePack {
            shortcut: def.shortcut,
            group_id: def.group_id,
            artifact_id: def.artifact_id,
            id: def.id,
            version: String::new(),
        })
    }

    pub fn download_url(&self) -> String {
        let group_path = self.group_id.replace('.', "/");
        format!(
            "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}-doc.zip",
            group_path, self.artifact_id, self.version, self.artifact_id, self.version
        )
    }

    pub fn display_name(&self) -> String {
        format!("{} {}", self.shortcut, self.version)
    }
}

pub fn known_shortcuts() -> Vec<&'static str> {
    FEATURE_PACKS.iter().map(|fp| fp.shortcut).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_shortcut_version() {
        let fp = FeaturePack::parse("cloud:9.0.0.Final").unwrap();
        assert_eq!(fp.shortcut, "cloud");
        assert_eq!(fp.group_id, "org.wildfly.cloud");
        assert_eq!(fp.artifact_id, "wildfly-cloud-galleon-pack");
        assert_eq!(fp.id, 2);
        assert_eq!(fp.version, "9.0.0.Final");
    }

    #[test]
    fn parse_all_shortcuts() {
        let inputs = [
            ("wildfly:39.0.1.Final", 1),
            ("cloud:9.0.0.Final", 2),
            ("datasources:11.2.0.Final", 3),
            ("keycloak:26.4.0", 4),
            ("grpc:0.1.16.Final", 5),
            ("myfaces:2.0.2.Final", 6),
            ("graphql:2.6.0.Final", 7),
        ];
        for (input, expected_id) in inputs {
            let fp = FeaturePack::parse(input).unwrap();
            assert_eq!(fp.id, expected_id, "Wrong ID for {}", input);
        }
    }

    #[test]
    fn parse_unknown_shortcut() {
        let result = FeaturePack::parse("unknown:1.0.0");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown feature pack"));
        assert!(err.contains("cloud"));
    }

    #[test]
    fn parse_missing_version() {
        let result = FeaturePack::parse("cloud:");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing version"));
    }

    #[test]
    fn parse_no_colon() {
        let result = FeaturePack::parse("cloud");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected format"));
    }

    #[test]
    fn download_url_format() {
        let fp = FeaturePack::parse("cloud:9.0.0.Final").unwrap();
        assert_eq!(
            fp.download_url(),
            "https://repo1.maven.org/maven2/org/wildfly/cloud/wildfly-cloud-galleon-pack/9.0.0.Final/wildfly-cloud-galleon-pack-9.0.0.Final-doc.zip"
        );
    }

    #[test]
    fn download_url_nested_group() {
        let fp = FeaturePack::parse("grpc:0.1.16.Final").unwrap();
        assert_eq!(
            fp.download_url(),
            "https://repo1.maven.org/maven2/org/wildfly/extras/grpc/wildfly-grpc-feature-pack/0.1.16.Final/wildfly-grpc-feature-pack-0.1.16.Final-doc.zip"
        );
    }

    #[test]
    fn display_name_format() {
        let fp = FeaturePack::parse("cloud:9.0.0.Final").unwrap();
        assert_eq!(fp.display_name(), "cloud 9.0.0.Final");
    }

    #[test]
    fn unique_ids() {
        let ids: Vec<u16> = FEATURE_PACKS.iter().map(|fp| fp.id).collect();
        let mut deduped = ids.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(ids.len(), deduped.len(), "Feature pack IDs must be unique");
    }

    #[test]
    fn ids_in_valid_range() {
        for fp in FEATURE_PACKS {
            assert!(
                fp.id >= 1 && fp.id < 100,
                "Feature pack '{}' has ID {} outside range 1-99",
                fp.shortcut,
                fp.id
            );
        }
    }
}
