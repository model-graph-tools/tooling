use std::ffi::OsStr;

use crate::feature_pack::all_feature_pack_identifiers;
use clap_complete::engine::CompletionCandidate;
use semver::Version;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

pub fn complete_single_identifier(_current: &OsStr) -> Vec<CompletionCandidate> {
    let mut completions = all_simple_versions();
    completions.extend(feature_pack_completions());
    completions
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}

pub fn complete_multiple_identifiers(current: &OsStr) -> Vec<CompletionCandidate> {
    let input = current.to_str().unwrap_or("");
    let parameter = if input.is_empty() { None } else { Some(input) };
    let (prefix_0, prefix_1, suggestions) = find_suggestions(parameter);
    suggestions
        .iter()
        .map(|s| CompletionCandidate::new(format!("{}{}{}", prefix_0, prefix_1, s)))
        .collect()
}

fn find_suggestions(parameter: Option<&str>) -> (String, String, Vec<String>) {
    let (prefix, token) = parse_prefix_token(parameter);

    let (out_token, suggestions): (&str, Vec<String>) = if token == ".." {
        let versions = all_simple_versions().into_iter().skip(1).collect();
        (token, versions)
    } else if let Some(after) = token.strip_prefix("..") {
        (token, suggest_after_dots(after, &Version::new(0, 0, 0)))
    } else if let Some(before) = token.strip_suffix("..") {
        let versions = parse_version(before)
            .map(|v| versions_after(&v))
            .unwrap_or_default();
        (token, versions)
    } else if token.contains("..") {
        let (before, after) = token.split_once("..").unwrap_or(("", ""));
        let versions = parse_version(before)
            .map(|v| suggest_after_dots(after, &v))
            .unwrap_or_default();
        (token, versions)
    } else {
        let mut completions = all_simple_versions();
        completions.extend(feature_pack_completions());
        ("", completions)
    };

    (prefix.to_string(), out_token.to_string(), suggestions)
}

fn feature_pack_completions() -> Vec<String> {
    all_feature_pack_identifiers()
}

fn parse_prefix_token(parameter: Option<&str>) -> (&str, &str) {
    match parameter {
        Some(param) => match param.rfind(',') {
            Some(pos) if pos < param.len() - 1 => param.split_at(pos + 1),
            Some(_) => (param, ""),
            None => ("", param),
        },
        None => ("", ""),
    }
}

fn parse_version(input: &str) -> Option<Version> {
    WildFlyContainer::version(input).ok().map(|wfc| wfc.version)
}

fn versions_after(start: &Version) -> Vec<String> {
    all_versions()
        .iter()
        .filter(|v| {
            if v.major == start.major {
                v.minor > start.minor
            } else {
                v.major > start.major
            }
        })
        .map(simple_version)
        .collect()
}

fn suggest_after_dots(after_dots: &str, start_after: &Version) -> Vec<String> {
    if WildFlyContainer::version(after_dots).is_ok() {
        return vec![];
    }

    let major_number = after_dots
        .strip_suffix('.')
        .unwrap_or(after_dots)
        .parse::<u64>()
        .ok();

    if let Some(number) = major_number {
        let versions = all_versions();
        let filtered: Vec<String> = versions
            .iter()
            .skip_while(|v| v <= &start_after)
            .filter(|v| match number {
                1..=9 if !after_dots.ends_with('.') => {
                    v.major >= (number * 10) && v.major < ((number + 1) * 10)
                }
                _ => v.major == number && v.minor > 0,
            })
            .map(simple_version)
            .map(|v| v.strip_prefix(after_dots).unwrap_or(&v).to_string())
            .collect();
        filtered
    } else {
        vec![]
    }
}

fn all_versions() -> Vec<Version> {
    VERSIONS.values().map(|wfc| wfc.version.clone()).collect()
}

fn all_simple_versions() -> Vec<String> {
    all_versions().iter().map(simple_version).collect()
}

fn simple_version(version: &Version) -> String {
    if version.minor == 0 {
        format!("{}", version.major)
    } else {
        format!("{}.{}", version.major, version.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidates_to_strings(candidates: Vec<CompletionCandidate>) -> Vec<String> {
        candidates
            .iter()
            .map(|c| c.get_value().to_str().unwrap().to_string())
            .collect()
    }

    fn single(input: &str) -> Vec<String> {
        candidates_to_strings(complete_single_identifier(OsStr::new(input)))
    }

    fn multiple(input: &str) -> Vec<String> {
        candidates_to_strings(complete_multiple_identifiers(OsStr::new(input)))
    }

    // ------------------------------------------------------ single identifier

    #[test]
    fn single_empty_returns_versions_and_feature_packs() {
        let results = single("");
        assert!(results.contains(&"34".to_string()));
        assert!(results.contains(&"26.1".to_string()));
        assert!(results.contains(&"ai".to_string()));
        assert!(results.contains(&"grpc".to_string()));
        assert!(results.contains(&"ai:0.9.0".to_string()));
    }

    #[test]
    fn single_includes_all_feature_pack_shortcuts() {
        let results = single("");
        for shortcut in &["ai", "graphql", "grpc", "keycloak", "myfaces"] {
            assert!(
                results.contains(&shortcut.to_string()),
                "Missing shortcut: {}",
                shortcut
            );
        }
    }

    #[test]
    fn single_includes_versioned_feature_packs() {
        let results = single("");
        assert!(results.contains(&"ai:0.9.0".to_string()));
        assert!(results.contains(&"grpc:0.1.16".to_string()));
    }

    #[test]
    fn single_no_range_suggestions() {
        let results = single("");
        assert!(!results.iter().any(|r| r.contains("..")));
    }

    #[test]
    fn single_no_comma_handling() {
        let results = single("34,");
        assert!(!results.iter().any(|r| r.starts_with("34,")));
    }

    // ------------------------------------------------------ multiple identifiers

    #[test]
    fn multiple_empty_returns_versions_and_feature_packs() {
        let results = multiple("");
        assert!(results.contains(&"34".to_string()));
        assert!(results.contains(&"ai".to_string()));
        assert!(results.contains(&"ai:0.9.0".to_string()));
    }

    #[test]
    fn multiple_after_comma_returns_fresh_completions() {
        let results = multiple("34,");
        assert!(results.iter().any(|r| r.starts_with("34,")));
        assert!(results.iter().any(|r| r.ends_with("ai")));
    }

    #[test]
    fn multiple_after_comma_with_partial() {
        let results = multiple("34,2");
        assert!(results.iter().all(|r| r.starts_with("34,")));
    }

    #[test]
    fn multiple_range_start_suggests_versions_after() {
        let results = multiple("20..");
        assert!(!results.is_empty());
        for r in &results {
            assert!(r.starts_with("20.."), "Expected prefix '20..': {}", r);
        }
    }

    #[test]
    fn multiple_complete_range_returns_empty() {
        let results = multiple("20..25");
        assert!(results.is_empty());
    }

    #[test]
    fn multiple_bare_dots_suggests_all_but_first() {
        let results = multiple("..");
        assert!(!results.is_empty());
        let all = all_simple_versions();
        assert!(!results.iter().any(|r| r == &format!("..{}", all[0])));
    }

    #[test]
    fn multiple_comma_then_range() {
        let results = multiple("34,20..");
        assert!(results.iter().all(|r| r.starts_with("34,20..")));
        assert!(!results.is_empty());
    }

    // ------------------------------------------------------ internal helpers

    #[test]
    fn parse_prefix_token_no_input() {
        let (prefix, token) = parse_prefix_token(None);
        assert_eq!(prefix, "");
        assert_eq!(token, "");
    }

    #[test]
    fn parse_prefix_token_simple() {
        let (prefix, token) = parse_prefix_token(Some("34"));
        assert_eq!(prefix, "");
        assert_eq!(token, "34");
    }

    #[test]
    fn parse_prefix_token_after_comma() {
        let (prefix, token) = parse_prefix_token(Some("34,26"));
        assert_eq!(prefix, "34,");
        assert_eq!(token, "26");
    }

    #[test]
    fn parse_prefix_token_trailing_comma() {
        let (prefix, token) = parse_prefix_token(Some("34,"));
        assert_eq!(prefix, "34,");
        assert_eq!(token, "");
    }

    #[test]
    fn feature_pack_completions_include_both_forms() {
        let completions = feature_pack_completions();
        assert!(completions.contains(&"ai".to_string()));
        assert!(completions.contains(&"ai:0.9.0".to_string()));
    }

    #[test]
    fn all_simple_versions_no_duplicates() {
        let versions = all_simple_versions();
        let mut deduped = versions.clone();
        deduped.sort();
        deduped.dedup();
        assert_eq!(versions.len(), deduped.len());
    }
}
