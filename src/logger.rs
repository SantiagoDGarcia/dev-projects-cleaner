/// Writes a text log of every cleaning run: parameters, what was removed,
/// space freed, timing and errors.
///
/// By default each run gets its own timestamped file next to the executable
/// (so the log always lands in a known folder, even when the app is launched
/// from Finder or another directory). A custom location can be given with
/// `--log <path>`, and logging can be disabled with `--no-log`.
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::cleaner::{display_path, format_size, CleanMode, CleanSummary};
use crate::ui::{format_duration, render_hierarchy, utc_timestamp, HierarchyOptions};

pub struct LogOptions {
    pub enabled: bool,
    pub custom_path: Option<PathBuf>,
}

/// Write the run log and return the path it was written to, if any.
pub fn write_clean_log(
    summary: &CleanSummary,
    target: &Path,
    config_file: Option<&Path>,
    dry_run: bool,
    elapsed: Duration,
    opts: &LogOptions,
) -> Option<PathBuf> {
    if !opts.enabled || dry_run {
        return None;
    }

    let path = match &opts.custom_path {
        Some(p) => p.clone(),
        None => default_log_path()?,
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let content = build_log(summary, target, config_file, elapsed);
    match fs::write(&path, content) {
        Ok(()) => Some(path),
        Err(_) => None,
    }
}

fn default_log_path() -> Option<PathBuf> {
    let stamp = utc_timestamp().replace([' ', ':'], "_");
    Some(
        std::env::current_exe()
            .ok()?
            .parent()?
            .join(format!("clean-{stamp}.log")),
    )
}

fn build_log(
    summary: &CleanSummary,
    target: &Path,
    config_file: Option<&Path>,
    elapsed: Duration,
) -> String {
    let mode_str = match summary.mode {
        CleanMode::Trash => "Trash (recycle bin)",
        CleanMode::Force => "Force (permanent delete)",
    };

    let mut out = String::new();
    out.push_str(&format!(
        "DevProjectsCleaner v{} — Clean log\n",
        env!("CARGO_PKG_VERSION")
    ));
    out.push_str(&"=".repeat(60));
    out.push('\n');
    out.push_str(&format!("Generated:      {}\n", utc_timestamp()));
    out.push_str(&format!("Target:         {}\n", display_path(target)));
    out.push_str(&format!("Mode:           {mode_str}\n"));
    if let Some(cfg) = config_file {
        out.push_str(&format!("Config:         {}\n", cfg.display()));
    }
    out.push('\n');
    out.push_str(&format!("Items removed:  {}\n", summary.total_items));
    out.push_str(&format!(
        "Space freed:    {}\n",
        format_size(summary.total_size)
    ));
    out.push_str(&format!("Time taken:     {}\n", format_duration(elapsed)));
    out.push('\n');

    if summary.deleted.is_empty() {
        out.push_str("Deleted items:  none\n");
    } else {
        out.push_str(&format!("Deleted items ({}):\n", summary.deleted.len()));
        for item in &summary.deleted {
            out.push_str(&format!(
                "  [{}]  {}  ({})\n",
                item.category,
                display_path(&item.path),
                format_size(item.size)
            ));
        }
    }
    out.push('\n');

    if !summary.deleted.is_empty() {
        out.push_str("Cleaned hierarchy (full, no collapsing):\n");
        for line in render_hierarchy(
            &summary.deleted,
            target,
            &HierarchyOptions {
                max_depth: 100,
                max_leaves: usize::MAX,
                plain: true,
            },
        ) {
            out.push_str(&format!("{line}\n"));
        }
        out.push('\n');
    }

    if summary.errors.is_empty() {
        out.push_str("Errors:         none\n");
    } else {
        out.push_str(&format!("Errors ({}):\n", summary.errors.len()));
        for e in &summary.errors {
            out.push_str(&format!("  {e}\n"));
        }
    }
    out
}
