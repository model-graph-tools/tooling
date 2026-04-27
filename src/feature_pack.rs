use anyhow::bail;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturePack {
    pub shortcut: &'static str,
    pub group_id: &'static str,
    pub artifact_id: &'static str,
    pub id: u16,
    pub version: &'static str,
    pub maven_version: &'static str,
}

static FEATURE_PACKS: &[FeaturePack] = &[
    FeaturePack {
        shortcut: "ai",
        group_id: "org.wildfly.generative-ai",
        artifact_id: "wildfly-ai-feature-pack",
        id: 1,
        version: "0.9.0",
        maven_version: "0.9.0",
    },
    FeaturePack {
        shortcut: "graphql",
        group_id: "org.wildfly.extras.graphql",
        artifact_id: "wildfly-microprofile-graphql-feature-pack",
        id: 2,
        version: "2.7.0",
        maven_version: "2.7.0.Final",
    },
    FeaturePack {
        shortcut: "grpc",
        group_id: "org.wildfly.extras.grpc",
        artifact_id: "wildfly-grpc-feature-pack",
        id: 3,
        version: "0.1.16",
        maven_version: "0.1.16.Final",
    },
    FeaturePack {
        shortcut: "keycloak",
        group_id: "org.keycloak",
        artifact_id: "keycloak-saml-adapter-galleon-pack",
        id: 4,
        version: "26.6.1",
        maven_version: "26.6.1",
    },
    FeaturePack {
        shortcut: "myfaces",
        group_id: "org.wildfly",
        artifact_id: "wildfly-myfaces-feature-pack",
        id: 5,
        version: "2.0.3",
        maven_version: "2.0.3.Final",
    },
];

impl FeaturePack {
    pub fn parse(input: &str) -> anyhow::Result<FeaturePack> {
        let Some(fp) = FEATURE_PACKS.iter().find(|fp| fp.shortcut == input) else {
            bail!(
                "Unknown feature pack '{}'. Known feature packs: {}",
                input,
                known_shortcuts().join(", ")
            );
        };
        Ok(fp.clone())
    }

    pub fn from_shortcut(shortcut: &str) -> Option<&'static FeaturePack> {
        FEATURE_PACKS.iter().find(|fp| fp.shortcut == shortcut)
    }

    pub fn download_url(&self) -> String {
        let group_path = self.group_id.replace('.', "/");
        format!(
            "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}-doc.zip",
            group_path, self.artifact_id, self.maven_version, self.artifact_id, self.maven_version
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
    fn parse_all_shortcuts() {
        let inputs = [
            ("ai", 1, "0.9.0"),
            ("graphql", 2, "2.7.0"),
            ("grpc", 3, "0.1.16"),
            ("keycloak", 4, "26.6.1"),
            ("myfaces", 5, "2.0.3"),
        ];
        for (input, expected_id, expected_version) in inputs {
            let fp = FeaturePack::parse(input).unwrap();
            assert_eq!(fp.id, expected_id, "Wrong ID for {}", input);
            assert_eq!(fp.version, expected_version, "Wrong version for {}", input);
        }
    }

    #[test]
    fn parse_unknown_shortcut() {
        let result = FeaturePack::parse("unknown");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown feature pack"));
        assert!(err.contains("ai"));
    }

    #[test]
    fn download_url_without_final() {
        let fp = FeaturePack::parse("ai").unwrap();
        assert_eq!(
            fp.download_url(),
            "https://repo1.maven.org/maven2/org/wildfly/generative-ai/wildfly-ai-feature-pack/0.9.0/wildfly-ai-feature-pack-0.9.0-doc.zip"
        );
    }

    #[test]
    fn download_url_with_final() {
        let fp = FeaturePack::parse("graphql").unwrap();
        assert_eq!(
            fp.download_url(),
            "https://repo1.maven.org/maven2/org/wildfly/extras/graphql/wildfly-microprofile-graphql-feature-pack/2.7.0.Final/wildfly-microprofile-graphql-feature-pack-2.7.0.Final-doc.zip"
        );
    }

    #[test]
    fn display_name_without_final() {
        let fp = FeaturePack::parse("ai").unwrap();
        assert_eq!(fp.display_name(), "ai 0.9.0");
    }

    #[test]
    fn display_name_strips_final() {
        let fp = FeaturePack::parse("grpc").unwrap();
        assert_eq!(fp.display_name(), "grpc 0.1.16");
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

    #[test]
    fn from_shortcut_found() {
        let fp = FeaturePack::from_shortcut("ai");
        assert!(fp.is_some());
        assert_eq!(fp.unwrap().id, 1);
    }

    #[test]
    fn from_shortcut_not_found() {
        assert!(FeaturePack::from_shortcut("unknown").is_none());
    }
}
