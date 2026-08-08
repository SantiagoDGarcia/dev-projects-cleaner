/// Loads and merges pattern configuration from, in order of precedence:
///
///   1. built-in defaults              (`patterns::default_patterns`)
///   2. global config                  `~/.config/dev-projects-cleaner/config.toml`
///   3. project config                 `<target>/dev-projects-cleaner.toml`
///   4. explicit config                `--config <path>`
///   5. CLI flags                      `--ignore <name>` / `--add <name>`
///
/// Later sources append to earlier ones; excludes always win (a name that is
/// excluded is never entered nor cleaned).
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::cli::CliArgs;
use crate::patterns::{default_excluded_names, default_patterns, MatchKind, MatchScope, Pattern};

const PROJECT_CONFIG_FILE: &str = "dev-projects-cleaner.toml";

#[derive(Deserialize)]
struct FileConfig {
    exclude: Option<ExcludeConfig>,
    patterns: Option<Vec<Pattern>>,
}

#[derive(Deserialize)]
struct ExcludeConfig {
    /// Names that must never be entered nor cleaned.
    names: Option<Vec<String>>,
}

pub struct Config {
    pub patterns: Vec<Pattern>,
    pub excluded_names: Vec<String>,
    /// The config file that was actually loaded and used, if any.
    pub loaded_file: Option<PathBuf>,
}

impl Config {
    pub fn load(cli: &CliArgs, target: &Path) -> Config {
        let mut config = Config {
            patterns: default_patterns(),
            excluded_names: default_excluded_names(),
            loaded_file: None,
        };

        if let Some(path) = global_config_path() {
            if config.merge_file(&path) {
                config.loaded_file = Some(path);
            }
        }

        let project_config = target.join(PROJECT_CONFIG_FILE);
        if project_config.is_file() && config.merge_file(&project_config) {
            config.loaded_file = Some(project_config);
        }

        if let Some(path) = &cli.config {
            if config.merge_file(path) {
                config.loaded_file = Some(path.clone());
            } else {
                eprintln!("Warning: could not read config file '{}'", path.display());
            }
        }

        for name in &cli.ignore {
            if !config.excluded_names.contains(name) {
                config.excluded_names.push(name.clone());
            }
        }

        for name in &cli.add {
            config.patterns.push(Pattern {
                names: vec![name.clone()],
                kind: MatchKind::Any,
                scope: MatchScope::Any,
                description: "Custom pattern".to_string(),
                category: "Custom".to_string(),
            });
        }

        config
    }

    /// Append the patterns/excludes from a TOML file. Returns true on success.
    fn merge_file(&mut self, path: &Path) -> bool {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let file: FileConfig = match toml::from_str(&content) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Warning: invalid config '{}': {e}", path.display());
                return false;
            }
        };

        if let Some(exclude) = file.exclude {
            if let Some(names) = exclude.names {
                for name in names {
                    if !self.excluded_names.contains(&name) {
                        self.excluded_names.push(name);
                    }
                }
            }
        }

        if let Some(patterns) = file.patterns {
            for p in patterns {
                self.patterns.push(p);
            }
        }

        true
    }
}

/// `$XDG_CONFIG_HOME/dev-projects-cleaner/config.toml` or
/// `~/.config/dev-projects-cleaner/config.toml`.
fn global_config_path() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return Some(
                PathBuf::from(dir)
                    .join("dev-projects-cleaner")
                    .join("config.toml"),
            );
        }
    }
    dirs::home_dir().map(|home| {
        home.join(".config")
            .join("dev-projects-cleaner")
            .join("config.toml")
    })
}
