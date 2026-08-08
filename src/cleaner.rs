use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use colored::Colorize;
use walkdir::WalkDir;

use crate::patterns::{is_excluded, matches_pattern, Pattern};
use crate::ui::format_duration;

pub struct FoundItem {
    pub path: PathBuf,
    pub pattern: Pattern,
    pub size: u64,
}

pub struct DeletedItem {
    pub path: PathBuf,
    pub category: String,
    pub size: u64,
}

pub struct CleanSummary {
    pub total_items: usize,
    pub total_size: u64,
    pub by_category: HashMap<String, usize>,
    pub deleted: Vec<DeletedItem>,
    pub errors: Vec<String>,
    pub mode: CleanMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CleanMode {
    Trash,
    Force,
}

pub struct CleanOptions {
    pub mode: CleanMode,
    pub verbose: bool,
}

pub fn collect_matches(root: &Path, patterns: &[Pattern], excluded: &[String]) -> Vec<FoundItem> {
    let mut items = Vec::new();
    let mut dirs_to_scan: Vec<(PathBuf, usize)> = vec![(root.to_path_buf(), 0)];
    let mut scanned = std::collections::HashSet::new();

    while let Some((dir, depth)) = dirs_to_scan.pop() {
        if !scanned.insert(dir.clone()) {
            continue;
        }

        let read_dir = match fs::read_dir(&dir) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for entry in read_dir.flatten() {
            let path = entry.path();
            let name = match entry.file_name().to_str() {
                Some(n) => n.to_string(),
                None => continue,
            };

            if is_excluded(&name, excluded) {
                continue;
            }

            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let is_file = !is_dir;

            if let Some(pattern) = matches_pattern(&name, is_dir, is_file, depth, patterns) {
                let size = if is_dir {
                    calculate_dir_size(&path)
                } else {
                    path.metadata().map(|m| m.len()).unwrap_or(0)
                };

                items.push(FoundItem {
                    path,
                    pattern,
                    size,
                });
                continue;
            }

            if is_dir {
                dirs_to_scan.push((path, depth + 1));
            }
        }
    }

    items.sort_by(|a, b| a.path.cmp(&b.path));
    items
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

pub fn delete_item(path: &Path, mode: CleanMode) -> io::Result<()> {
    match mode {
        CleanMode::Trash => trash::delete(path).map_err(|e| io::Error::other(e.to_string())),
        CleanMode::Force => {
            if path.is_dir() {
                fs::remove_dir_all(path)
            } else {
                fs::remove_file(path)
            }
        }
    }
}

pub fn run_clean(options: &CleanOptions, items: &[FoundItem]) -> CleanSummary {
    let total_size: u64 = items.iter().map(|i| i.size).sum();

    let mut by_category: HashMap<String, usize> = HashMap::new();
    for item in items {
        *by_category
            .entry(item.pattern.category.clone())
            .or_insert(0) += 1;
    }

    let mut errors = Vec::new();
    let mut deleted = Vec::new();

    for item in items {
        match delete_item(&item.path, options.mode) {
            Ok(()) => {
                deleted.push(DeletedItem {
                    path: item.path.clone(),
                    category: item.pattern.category.clone(),
                    size: item.size,
                });
                if options.verbose {
                    let size_str = format_size(item.size);
                    let action = if options.mode == CleanMode::Trash {
                        "Trashed"
                    } else {
                        "Deleted"
                    };
                    println!(
                        "  {}  {}  ({})",
                        action.green(),
                        display_path(&item.path),
                        size_str
                    );
                }
            }
            Err(e) => {
                if options.verbose {
                    println!("  {}  {}  ({})", "ERROR".red(), display_path(&item.path), e);
                }
                errors.push(format!("{}: {}", display_path(&item.path), e));
            }
        }
    }

    CleanSummary {
        total_items: items.len(),
        total_size,
        by_category,
        deleted,
        errors,
        mode: options.mode,
    }
}

pub fn display_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.to_string()
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

pub fn print_summary(summary: &CleanSummary, elapsed: Duration, log_path: Option<&Path>) {
    println!();
    println!(
        "{}",
        "═══════════════════════════════════════".bright_black()
    );
    println!("{}", "              SUMMARY".bold());
    println!(
        "{}",
        "═══════════════════════════════════════".bright_black()
    );

    let mode_str = match summary.mode {
        CleanMode::Trash => "Recycle Bin",
        CleanMode::Force => "Permanently Deleted",
    };

    println!(
        "  Items removed:  {}",
        summary.total_items.to_string().bold()
    );
    println!(
        "  Space freed:    {}",
        format_size(summary.total_size).bold()
    );
    println!("  Mode:           {}", mode_str.cyan());
    println!("  Time taken:     {}", format_duration(elapsed).bold());
    if let Some(path) = log_path {
        println!("  Log written:    {}", path.display().to_string().green());
    }

    if !summary.by_category.is_empty() {
        println!();
        println!("{}", "By category:".underline());
        let mut cats: Vec<_> = summary.by_category.iter().collect();
        cats.sort_by(|a, b| b.1.cmp(a.1));
        for (cat, count) in cats {
            println!("  {:18}  {}", format!("{cat}:"), count);
        }
    }

    if !summary.errors.is_empty() {
        println!();
        println!("{}", "Errors:".red().bold());
        for err in &summary.errors {
            println!("  {} {}", "•".red(), err);
        }
    }

    println!();
}
