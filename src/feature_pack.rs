use std::collections::BTreeMap;

use anyhow::bail;
use lazy_static::lazy_static;

lazy_static! {
    static ref FEATURE_PACKS: BTreeMap<(&'static str, &'static str), FeaturePack> = {
        let mut m = BTreeMap::new();
        // ai (shortcut_index=0)
        m.insert(("ai", "0.9.0"), FeaturePack {
            shortcut: "ai",
            group_id: "org.wildfly.generative-ai",
            artifact_id: "wildfly-ai-feature-pack",
            shortcut_index: 0,
            version_index: 0,
            version: "0.9.0",
            maven_version: "0.9.0",
        });
        // graphql (shortcut_index=1)
        m.insert(("graphql", "2.7.0"), FeaturePack {
            shortcut: "graphql",
            group_id: "org.wildfly.extras.graphql",
            artifact_id: "wildfly-microprofile-graphql-feature-pack",
            shortcut_index: 1,
            version_index: 0,
            version: "2.7.0",
            maven_version: "2.7.0.Final",
        });
        // grpc (shortcut_index=2)
        m.insert(("grpc", "0.1.16"), FeaturePack {
            shortcut: "grpc",
            group_id: "org.wildfly.extras.grpc",
            artifact_id: "wildfly-grpc-feature-pack",
            shortcut_index: 2,
            version_index: 0,
            version: "0.1.16",
            maven_version: "0.1.16.Final",
        });
        // keycloak (shortcut_index=3)
        m.insert(("keycloak", "26.6.1"), FeaturePack {
            shortcut: "keycloak",
            group_id: "org.keycloak",
            artifact_id: "keycloak-saml-adapter-galleon-pack",
            shortcut_index: 3,
            version_index: 0,
            version: "26.6.1",
            maven_version: "26.6.1",
        });
        // myfaces (shortcut_index=4)
        m.insert(("myfaces", "2.0.3"), FeaturePack {
            shortcut: "myfaces",
            group_id: "org.wildfly",
            artifact_id: "wildfly-myfaces-feature-pack",
            shortcut_index: 4,
            version_index: 0,
            version: "2.0.3",
            maven_version: "2.0.3.Final",
        });
        m
    };
}

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

    pub fn parse(input: &str) -> anyhow::Result<FeaturePack> {
        if let Some((shortcut, version)) = input.split_once(':') {
            match FEATURE_PACKS.get(&(shortcut, version)) {
                Some(fp) => Ok(fp.clone()),
                None => {
                    let versions = known_versions(shortcut);
                    if versions.is_empty() {
                        bail!(
                            "Unknown feature pack '{}'. Known feature packs: {}",
                            shortcut,
                            known_shortcuts().join(", ")
                        );
                    } else {
                        bail!(
                            "Unknown version '{}' for feature pack '{}'. Known versions: {}",
                            version,
                            shortcut,
                            versions.join(", ")
                        );
                    }
                }
            }
        } else {
            let latest = FEATURE_PACKS
                .iter()
                .filter(|((s, _), _)| *s == input)
                .map(|(_, fp)| fp)
                .next_back();
            match latest {
                Some(fp) => Ok(fp.clone()),
                None => bail!(
                    "Unknown feature pack '{}'. Known feature packs: {}",
                    input,
                    known_shortcuts().join(", ")
                ),
            }
        }
    }

    pub fn from_container_id(container_id: &str) -> Option<FeaturePack> {
        FEATURE_PACKS
            .values()
            .find(|fp| fp.container_id() == container_id)
            .cloned()
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
    let mut shortcuts: Vec<&str> = FEATURE_PACKS.keys().map(|(s, _)| *s).collect();
    shortcuts.dedup();
    shortcuts
}

pub fn known_versions(shortcut: &str) -> Vec<&'static str> {
    FEATURE_PACKS
        .keys()
        .filter(|(s, _)| *s == shortcut)
        .map(|(_, v)| *v)
        .collect()
}

pub fn all_feature_pack_identifiers() -> Vec<String> {
    let mut ids: Vec<String> = known_shortcuts().iter().map(|s| s.to_string()).collect();
    for ((shortcut, version), _) in FEATURE_PACKS.iter() {
        ids.push(format!("{}:{}", shortcut, version));
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_shortcut_returns_latest() {
        let inputs = [
            ("ai", 0, "0.9.0"),
            ("graphql", 1, "2.7.0"),
            ("grpc", 2, "0.1.16"),
            ("keycloak", 3, "26.6.1"),
            ("myfaces", 4, "2.0.3"),
        ];
        for (input, expected_si, expected_version) in inputs {
            let fp = FeaturePack::parse(input).unwrap();
            assert_eq!(fp.shortcut_index, expected_si, "Wrong shortcut_index for {}", input);
            assert_eq!(fp.version, expected_version, "Wrong version for {}", input);
        }
    }

    #[test]
    fn parse_versioned_shortcut() {
        let fp = FeaturePack::parse("ai:0.9.0").unwrap();
        assert_eq!(fp.shortcut, "ai");
        assert_eq!(fp.version, "0.9.0");
        assert_eq!(fp.shortcut_index, 0);
        assert_eq!(fp.version_index, 0);
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
    fn parse_unknown_version() {
        let result = FeaturePack::parse("ai:9.9.9");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown version"));
        assert!(err.contains("0.9.0"));
    }

    #[test]
    fn parse_versioned_unknown_shortcut() {
        let result = FeaturePack::parse("unknown:1.0.0");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown feature pack"));
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
        assert_eq!(FeaturePack::parse("ai").unwrap().port_offset(), 1000);
        assert_eq!(FeaturePack::parse("graphql").unwrap().port_offset(), 1100);
        assert_eq!(FeaturePack::parse("grpc").unwrap().port_offset(), 1200);
        assert_eq!(FeaturePack::parse("keycloak").unwrap().port_offset(), 1300);
        assert_eq!(FeaturePack::parse("myfaces").unwrap().port_offset(), 1400);
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
        let mut offsets: Vec<u16> = FEATURE_PACKS.values().map(|fp| fp.port_offset()).collect();
        let len = offsets.len();
        offsets.sort();
        offsets.dedup();
        assert_eq!(len, offsets.len(), "Feature pack port offsets must be unique");
    }

    #[test]
    fn port_offsets_start_at_1000() {
        for fp in FEATURE_PACKS.values() {
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

    #[test]
    fn known_shortcuts_are_unique() {
        let shortcuts = known_shortcuts();
        let mut deduped = shortcuts.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(shortcuts.len(), deduped.len());
    }

    #[test]
    fn known_versions_for_shortcut() {
        let versions = known_versions("ai");
        assert!(versions.contains(&"0.9.0"));
    }

    #[test]
    fn known_versions_for_unknown_shortcut() {
        let versions = known_versions("unknown");
        assert!(versions.is_empty());
    }

    #[test]
    fn all_identifiers_include_shortcuts_and_versioned() {
        let ids = all_feature_pack_identifiers();
        assert!(ids.contains(&"ai".to_string()));
        assert!(ids.contains(&"ai:0.9.0".to_string()));
        assert!(ids.contains(&"grpc".to_string()));
        assert!(ids.contains(&"grpc:0.1.16".to_string()));
    }
}
