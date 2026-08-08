mod cleaner;
mod cli;
mod config;
mod logger;
mod patterns;
mod ui;

use std::collections::HashMap;
use std::time::Instant;

use clap::Parser;
use colored::Colorize;

use crate::cleaner::{
    collect_matches, display_path, format_size, print_summary, run_clean, CleanMode, CleanOptions,
};
use crate::cli::CliArgs;
use crate::config::Config;
use crate::logger::{write_clean_log, LogOptions};
use crate::ui::{
    ask_path, confirm, confirm_or_list, format_duration, render_hierarchy, ContinueAnswer,
    HierarchyOptions,
};

fn pluralize(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

/// Welcome banner shown when the program opens.
fn print_intro() {
    println!(
        "{} v{} — {}",
        "DevProjectsCleaner".bold().cyan(),
        env!("CARGO_PKG_VERSION"),
        "Clean your project artifacts".dimmed()
    );
    println!();
    let description = [
        "A cross-platform CLI that scans any project for the junk that accumulates",
        "during development: temporary files, build artifacts and caches",
        "(node_modules, target, .dart_tool, __pycache__, logs, IDE/OS metadata, ...).",
        "Interactive by default: it shows a simple hierarchy of what was found and",
        "asks for confirmation before touching anything. Items go to the recycle bin",
        "(SAFE MODE) unless you use --force to permanently delete them.",
    ];
    for line in description {
        println!("  {}", line.dimmed());
    }
    println!();
}

fn main() {
    let args = CliArgs::parse();
    let started = Instant::now();

    print_intro();

    // ── Resolve the path (prompted first if not given as an argument) ──────
    let given_path = match &args.path {
        Some(p) => p.clone(),
        None => match ask_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}: {}", "Error".red().bold(), e);
                std::process::exit(1);
            }
        },
    };

    let canonical_path = std::fs::canonicalize(&given_path).unwrap_or_else(|_| given_path.clone());

    if !canonical_path.is_dir() {
        eprintln!(
            "{}: '{}' is not a valid directory",
            "Error".red().bold(),
            display_path(&canonical_path)
        );
        std::process::exit(1);
    }

    println!(
        "{} {}",
        "Target:".bright_black(),
        display_path(&canonical_path).yellow()
    );

    let config = Config::load(&args, &canonical_path);
    if let Some(file) = &config.loaded_file {
        println!(
            "{} {}",
            "Config:".bright_black(),
            file.display().to_string().magenta()
        );
    }
    println!();

    // ── Scan ────────────────────────────────────────────────────────────────
    let scan_started = Instant::now();
    println!(
        "{} {} …",
        "Scanning".cyan().bold(),
        display_path(&canonical_path).yellow()
    );
    let items = collect_matches(&canonical_path, &config.patterns, &config.excluded_names);
    let elapsed = scan_started.elapsed();
    let total_size: u64 = items.iter().map(|i| i.size).sum();

    if items.is_empty() {
        println!(
            "{} nothing to clean — your project is tidy. Done in {}.",
            "✓".green().bold(),
            format_duration(elapsed).dimmed()
        );
        return;
    }
    println!(
        "{} found {} ({}), scanned in {}",
        "✓".green().bold(),
        pluralize(items.len(), "item"),
        format_size(total_size).bold(),
        format_duration(elapsed).dimmed()
    );
    println!();

    // ── Breakdown by category ───────────────────────────────────────────────
    println!("{}", "Breakdown by category:".underline());
    let mut by_category: HashMap<String, (usize, u64)> = HashMap::new();
    for item in &items {
        let entry = by_category
            .entry(item.pattern.category.clone())
            .or_insert((0, 0));
        entry.0 += 1;
        entry.1 += item.size;
    }
    let mut cats: Vec<_> = by_category.into_iter().collect();
    cats.sort_by_key(|(_, (_, size))| std::cmp::Reverse(*size));
    let max_count = cats.iter().map(|(_, (c, _))| *c).max().unwrap_or(1);
    for (cat, (count, size)) in cats {
        let bar_len = (count * 20).max(1) / max_count;
        let bar: String = "█".repeat(bar_len);
        let label = if count == 1 {
            "1 item".to_string()
        } else {
            format!("{count} items")
        };
        println!(
            "  {:<12} {}  {:>10}  {}",
            format!("{cat}:"),
            bar.cyan(),
            label,
            format_size(size)
        );
    }

    // ── Where the items are ─────────────────────────────────────────────────
    println!();
    if args.verbose {
        println!("{}", "All matching items:".underline());
        for item in &items {
            let size_str = format_size(item.size);
            println!(
                "  {}  {}  ({})",
                format!("[{}]", item.pattern.category).cyan(),
                display_path(&item.path),
                size_str
            );
        }
    } else {
        println!("{}", "Where they are:".underline());
        for line in render_hierarchy(&items, &canonical_path, &HierarchyOptions::default()) {
            println!("{line}");
        }
    }

    // ── Dry run short-circuits here ─────────────────────────────────────────
    if args.dry_run {
        println!();
        println!(
            "{} nothing was touched — run without --dry-run to actually clean.",
            "DRY RUN:".yellow().bold()
        );
        return;
    }

    // ── Confirmation & SAFE MODE ────────────────────────────────────────────
    let mut mode = if args.force {
        CleanMode::Force
    } else {
        CleanMode::Trash
    };

    if !args.yes {
        println!();
        // Loop so the user can ask to see the full, uncollapsed hierarchy
        // before deciding (press "l").
        loop {
            match confirm_or_list("The items above will be cleaned. Continue?") {
                Ok(ContinueAnswer::No) | Err(_) => {
                    println!("{}", "Cancelled. Nothing was touched.".dimmed());
                    std::process::exit(0);
                }
                Ok(ContinueAnswer::Yes) => break,
                Ok(ContinueAnswer::List) => {
                    println!();
                    println!("{}", "Full hierarchy (no collapsing):".underline());
                    for line in render_hierarchy(
                        &items,
                        &canonical_path,
                        &HierarchyOptions {
                            max_depth: 100,
                            max_leaves: usize::MAX,
                            plain: false,
                        },
                    ) {
                        println!("{line}");
                    }
                    println!();
                }
            }
        }
        println!();

        if !args.force {
            let safe = confirm(
                "Run in SAFE MODE? Items go to the recycle bin (recoverable).",
                true,
            )
            .unwrap_or(true);
            mode = if safe {
                CleanMode::Trash
            } else {
                CleanMode::Force
            };
            let mode_str = if safe {
                format!("{} (recycle bin)", "SAFE MODE".green())
            } else {
                format!("{} (permanent delete)", "FULL MODE".red())
            };
            println!("{} {}", "Mode:".bright_black(), mode_str);
            println!();
        }
    } else if mode == CleanMode::Force {
        println!();
        println!(
            "{}",
            "Running in FULL MODE — items will be permanently deleted."
                .red()
                .bold()
        );
        println!();
    }

    // ── Execute ─────────────────────────────────────────────────────────────
    let options = CleanOptions {
        mode,
        verbose: args.verbose,
    };
    let summary = run_clean(&options, &items);

    let log_options = LogOptions {
        enabled: !args.no_log,
        custom_path: args.log.clone(),
    };
    let log_path = write_clean_log(
        &summary,
        &canonical_path,
        config.loaded_file.as_deref(),
        args.dry_run,
        started.elapsed(),
        &log_options,
    );

    print_summary(&summary, started.elapsed(), log_path.as_deref());

    if !summary.errors.is_empty() {
        std::process::exit(1);
    }
}
