use std::ffi::OsStr;

use crate::feature_pack::known_shortcuts;
use clap_complete::engine::CompletionCandidate;
use semver::Version;
use wildfly_container_versions::{VERSIONS, WildFlyContainer};

pub fn complete_identifiers(current: &OsStr) -> Vec<CompletionCandidate> {
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
    known_shortcuts()
        .iter()
        .map(|s| format!("{}:", s))
        .collect()
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
