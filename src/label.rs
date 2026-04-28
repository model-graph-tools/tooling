//! OCI container label management for Neo4J container identification.

/// OCI container labels used to tag and filter `mgt`-managed containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Identifier,
}

impl Label {
    /// Returns the fully-qualified OCI label key.
    pub fn key(&self) -> &'static str {
        match self {
            Label::Identifier => "org.wildfly.mgt.identifier",
        }
    }

    /// Returns a `--filter` argument to match containers with this label.
    pub fn filter(&self) -> String {
        format!("label={}", self.key())
    }

    /// Returns a `--filter` argument matching this label with a specific value.
    pub fn filter_value(&self, value: &str) -> String {
        format!("label={}={}", self.key(), value)
    }

    /// Returns a `--label` argument for `container run` (e.g. `key=value`).
    pub fn run_arg(&self, value: &str) -> String {
        format!("{}={}", self.key(), value)
    }

    /// Returns a Go template expression to extract this label's value in `--format`.
    pub fn format_expr(&self) -> String {
        format!("{{{{index .Labels \"{}\"}}}}", self.key())
    }

    /// Parses a label value from container output, returning `None` for empty or `<no value>`.
    pub fn parse_value(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "<no value>" {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_returns_oci_label_key() {
        assert_eq!(Label::Identifier.key(), "org.wildfly.mgt.identifier");
    }

    #[test]
    fn filter_without_value() {
        assert_eq!(
            Label::Identifier.filter(),
            "label=org.wildfly.mgt.identifier"
        );
    }

    #[test]
    fn filter_with_value() {
        assert_eq!(
            Label::Identifier.filter_value("340"),
            "label=org.wildfly.mgt.identifier=340"
        );
    }

    #[test]
    fn run_arg_formats_key_equals_value() {
        assert_eq!(
            Label::Identifier.run_arg("ai-0.9.0"),
            "org.wildfly.mgt.identifier=ai-0.9.0"
        );
    }

    #[test]
    fn format_expr_produces_go_template() {
        assert_eq!(
            Label::Identifier.format_expr(),
            "{{index .Labels \"org.wildfly.mgt.identifier\"}}"
        );
    }

    #[test]
    fn parse_value_returns_none_for_empty() {
        assert_eq!(Label::Identifier.parse_value(""), None);
    }

    #[test]
    fn parse_value_returns_none_for_no_value_sentinel() {
        assert_eq!(Label::Identifier.parse_value("<no value>"), None);
    }

    #[test]
    fn parse_value_returns_some_for_real_value() {
        assert_eq!(
            Label::Identifier.parse_value("340"),
            Some("340".to_string())
        );
    }

    #[test]
    fn parse_value_trims_whitespace() {
        assert_eq!(
            Label::Identifier.parse_value("  ai-0.9.0  "),
            Some("ai-0.9.0".to_string())
        );
    }
}
