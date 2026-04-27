use anyhow::bail;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturePack {
    pub shortcut: &'static str,
    pub group_id: &'static str,
    pub artifact_id: &'static str,
    pub shortcut_index: u16,
    pub version_index: u16,
    pub version: &'static str,
    pub maven_version: &'static str,
}

impl FeaturePack {
    pub fn port_offset(&self) -> u16 {
        1000 + (self.shortcut_index * 100) + self.version_index
    }

    pub fn container_id(&self) -> String {
        format!("{}-{}", self.shortcut, self.version)
    }
}

static FEATURE_PACKS: &[FeaturePack] = &[
    FeaturePack {
        shortcut: "ai",
        group_id: "org.wildfly.generative-ai",
        artifact_id: "wildfly-ai-feature-pack",
        shortcut_index: 0,
        version_index: 0,
        version: "0.9.0",
        maven_version: "0.9.0",
    },
    FeaturePack {
        shortcut: "graphql",
        group_id: "org.wildfly.extras.graphql",
        artifact_id: "wildfly-microprofile-graphql-feature-pack",
        shortcut_index: 1,
        version_index: 0,
        version: "2.7.0",
        maven_version: "2.7.0.Final",
    },
    FeaturePack {
        shortcut: "grpc",
        group_id: "org.wildfly.extras.grpc",
        artifact_id: "wildfly-grpc-feature-pack",
        shortcut_index: 2,
        version_index: 0,
        version: "0.1.16",
        maven_version: "0.1.16.Final",
    },
    FeaturePack {
        shortcut: "keycloak",
        group_id: "org.keycloak",
        artifact_id: "keycloak-saml-adapter-galleon-pack",
        shortcut_index: 3,
        version_index: 0,
        version: "26.6.1",
        maven_version: "26.6.1",
    },
    FeaturePack {
        shortcut: "myfaces",
        group_id: "org.wildfly",
        artifact_id: "wildfly-myfaces-feature-pack",
        shortcut_index: 4,
        version_index: 0,
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

    pub fn from_container_id(container_id: &str) -> Option<&'static FeaturePack> {
        FEATURE_PACKS.iter().find(|fp| fp.container_id() == container_id)
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
            ("ai", 0, 0, "0.9.0"),
            ("graphql", 1, 0, "2.7.0"),
            ("grpc", 2, 0, "0.1.16"),
            ("keycloak", 3, 0, "26.6.1"),
            ("myfaces", 4, 0, "2.0.3"),
        ];
        for (input, expected_si, expected_vi, expected_version) in inputs {
            let fp = FeaturePack::parse(input).unwrap();
            assert_eq!(fp.shortcut_index, expected_si, "Wrong shortcut_index for {}", input);
            assert_eq!(fp.version_index, expected_vi, "Wrong version_index for {}", input);
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
    fn port_offset() {
        let fp = FeaturePack::parse("ai").unwrap();
        assert_eq!(fp.port_offset(), 1000);
        let fp = FeaturePack::parse("graphql").unwrap();
        assert_eq!(fp.port_offset(), 1100);
        let fp = FeaturePack::parse("grpc").unwrap();
        assert_eq!(fp.port_offset(), 1200);
        let fp = FeaturePack::parse("keycloak").unwrap();
        assert_eq!(fp.port_offset(), 1300);
        let fp = FeaturePack::parse("myfaces").unwrap();
        assert_eq!(fp.port_offset(), 1400);
    }

    #[test]
    fn container_id() {
        let fp = FeaturePack::parse("ai").unwrap();
        assert_eq!(fp.container_id(), "ai-0.9.0");
        let fp = FeaturePack::parse("graphql").unwrap();
        assert_eq!(fp.container_id(), "graphql-2.7.0");
    }

    #[test]
    fn unique_port_offsets() {
        let offsets: Vec<u16> = FEATURE_PACKS.iter().map(|fp| fp.port_offset()).collect();
        let mut deduped = offsets.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(offsets.len(), deduped.len(), "Feature pack port offsets must be unique");
    }

    #[test]
    fn port_offsets_start_at_1000() {
        for fp in FEATURE_PACKS {
            assert!(
                fp.port_offset() >= 1000,
                "Feature pack '{}' has port offset {} below 1000",
                fp.shortcut,
                fp.port_offset()
            );
        }
    }

    #[test]
    fn from_container_id_found() {
        let fp = FeaturePack::from_container_id("ai-0.9.0");
        assert!(fp.is_some());
        assert_eq!(fp.unwrap().shortcut_index, 0);
    }

    #[test]
    fn from_container_id_not_found() {
        assert!(FeaturePack::from_container_id("unknown-1.0.0").is_none());
    }
}
