/// Pattern matching is fully data-driven: every name, folder, file name or
/// name pattern lives in a `Pattern`. Built-in defaults are defined in
/// `patterns/defaults.json` (embedded into the binary at compile time); users
/// can add their own via a TOML config file or the `--add` / `--ignore` CLI
/// flags (see `config.rs`).
use std::sync::LazyLock;

use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchKind {
    File,
    Dir,
    #[default]
    Any,
}

/// Where a pattern is allowed to match.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MatchScope {
    /// Match at any depth (default).
    #[default]
    Any,
    /// Only as a direct child of the scanned root.
    Root,
}

/// A single match rule: one or more names (with optional `*` wildcards), a
/// kind, an optional scope and human-readable metadata. Shared by the embedded
/// JSON defaults and the TOML config files.
#[derive(Clone, Debug, Deserialize)]
pub struct Pattern {
    pub names: Vec<String>,
    #[serde(default)]
    pub kind: MatchKind,
    #[serde(default)]
    pub scope: MatchScope,
    #[serde(default = "default_description")]
    pub description: String,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_description() -> String {
    "Custom pattern".to_string()
}

fn default_category() -> String {
    "Custom".to_string()
}

/// Top-level shape of `patterns/defaults.json`.
#[derive(Deserialize)]
struct DefaultsFile {
    #[serde(default)]
    exclude: Vec<String>,
    #[serde(default)]
    patterns: Vec<Pattern>,
}

/// The built-in defaults, parsed once from the embedded JSON.
static DEFAULTS: LazyLock<DefaultsFile> = LazyLock::new(|| {
    let json = include_str!("../patterns/defaults.json");
    serde_json::from_str(json)
        .expect("patterns/defaults.json must be valid JSON matching DefaultsFile")
});

/// Owned copy of the built-in pattern table.
pub fn default_patterns() -> Vec<Pattern> {
    DEFAULTS.patterns.clone()
}

pub fn default_excluded_names() -> Vec<String> {
    DEFAULTS.exclude.clone()
}

/// Check whether `name` matches any of the given patterns at `depth` (0 = a
/// direct child of the scanned root).
///
/// Supports plain names (`node_modules`) and `*` wildcards as a prefix
/// (`*.log`) or suffix (`*~`).
pub fn matches_pattern(
    name: &str,
    is_dir: bool,
    is_file: bool,
    depth: usize,
    patterns: &[Pattern],
) -> Option<Pattern> {
    patterns.iter().find_map(|pattern| {
        if pattern.scope == MatchScope::Root && depth != 0 {
            return None;
        }

        let kind_ok = match pattern.kind {
            MatchKind::File => is_file,
            MatchKind::Dir => is_dir,
            MatchKind::Any => true,
        };
        if !kind_ok {
            return None;
        }

        let name_matches = pattern.names.iter().any(|pattern_name| {
            if let Some(suffix) = pattern_name.strip_prefix('*') {
                name.ends_with(suffix)
            } else if let Some(prefix) = pattern_name.strip_suffix('*') {
                name.starts_with(prefix)
            } else {
                name == *pattern_name
            }
        });

        name_matches.then(|| pattern.clone())
    })
}

/// True when `name` should never be entered nor cleaned.
pub fn is_excluded(name: &str, excluded: &[String]) -> bool {
    excluded.iter().any(|e| e == name)
}
