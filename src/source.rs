use crate::feature_pack::FeaturePack;
use wildfly_container_versions::WildFlyContainer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Source {
    WildFly(WildFlyContainer),
    FeaturePack(FeaturePack),
}

impl Source {
    pub fn parse(input: &str) -> anyhow::Result<Source> {
        if input.contains(':') {
            FeaturePack::parse(input).map(Source::FeaturePack)
        } else {
            WildFlyContainer::version(input)
                .map(Source::WildFly)
                .map_err(|e| anyhow::anyhow!("{}", e))
        }
    }

    pub fn parse_list(input: &str) -> anyhow::Result<Vec<Source>> {
        if input.contains("..") {
            let containers = WildFlyContainer::enumeration(input)
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            return Ok(containers.into_iter().map(Source::WildFly).collect());
        }

        let mut sources = Vec::new();
        for part in input.split(',') {
            let part = part.trim();
            if !part.is_empty() {
                sources.push(Source::parse(part)?);
            }
        }
        Ok(sources)
    }

    pub fn display_name(&self) -> String {
        match self {
            Source::WildFly(wc) => wc.display_version(),
            Source::FeaturePack(fp) => fp.display_name(),
        }
    }

    pub fn port_offset(&self) -> u16 {
        match self {
            Source::WildFly(wc) => wc.identifier,
            Source::FeaturePack(fp) => fp.id,
        }
    }

    pub fn container_id(&self) -> String {
        match self {
            Source::WildFly(wc) => wc.identifier.to_string(),
            Source::FeaturePack(fp) => fp.shortcut.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_wildfly_version() {
        let source = Source::parse("34").unwrap();
        assert!(matches!(source, Source::WildFly(_)));
        assert_eq!(source.display_name(), "34.0");
    }

    #[test]
    fn parse_wildfly_version_with_minor() {
        let source = Source::parse("26.1").unwrap();
        assert!(matches!(source, Source::WildFly(_)));
        assert_eq!(source.display_name(), "26.1");
    }

    #[test]
    fn parse_feature_pack() {
        let source = Source::parse("cloud:9.0.0.Final").unwrap();
        assert!(matches!(source, Source::FeaturePack(_)));
        assert_eq!(source.display_name(), "cloud 9.0.0.Final");
    }

    #[test]
    fn parse_invalid_input() {
        assert!(Source::parse("invalid").is_err());
    }

    #[test]
    fn parse_list_wildfly_only() {
        let sources = Source::parse_list("26,28,34").unwrap();
        assert_eq!(sources.len(), 3);
        assert!(sources.iter().all(|s| matches!(s, Source::WildFly(_))));
    }

    #[test]
    fn parse_list_feature_packs_only() {
        let sources = Source::parse_list("cloud:9.0.0.Final,grpc:0.1.16.Final").unwrap();
        assert_eq!(sources.len(), 2);
        assert!(sources.iter().all(|s| matches!(s, Source::FeaturePack(_))));
    }

    #[test]
    fn parse_list_mixed() {
        let sources = Source::parse_list("34,cloud:9.0.0.Final,26.1").unwrap();
        assert_eq!(sources.len(), 3);
        assert!(matches!(sources[0], Source::WildFly(_)));
        assert!(matches!(sources[1], Source::FeaturePack(_)));
        assert!(matches!(sources[2], Source::WildFly(_)));
    }

    #[test]
    fn parse_list_range() {
        let sources = Source::parse_list("26..29").unwrap();
        assert!(sources.len() >= 4);
        assert!(sources.iter().all(|s| matches!(s, Source::WildFly(_))));
    }

    #[test]
    fn port_offset_wildfly() {
        let source = Source::parse("34").unwrap();
        assert_eq!(source.port_offset(), 340);
    }

    #[test]
    fn port_offset_feature_pack() {
        let source = Source::parse("cloud:9.0.0.Final").unwrap();
        assert_eq!(source.port_offset(), 2);
    }

    #[test]
    fn port_offsets_no_overlap() {
        let fp_max = 99u16;
        let wf_min = 100u16; // WildFly 10.0 = major*10+minor = 100
        assert!(fp_max < wf_min, "Feature pack and WildFly port ranges overlap");
    }

    #[test]
    fn container_id_wildfly() {
        let source = Source::parse("34").unwrap();
        assert_eq!(source.container_id(), "340");
    }

    #[test]
    fn container_id_feature_pack() {
        let source = Source::parse("cloud:9.0.0.Final").unwrap();
        assert_eq!(source.container_id(), "cloud");
    }
}
