use std::collections::BTreeMap;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use colored::Colorize;

use crate::cleaner::{format_size, DeletedItem, FoundItem};

/// Minimal interface the hierarchy renderer needs from each item.
pub trait HierarchyItem {
    fn path(&self) -> &Path;
    fn size(&self) -> u64;
    fn category(&self) -> &str;
    fn description(&self) -> &str;
}

impl HierarchyItem for FoundItem {
    fn path(&self) -> &Path {
        &self.path
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn category(&self) -> &str {
        &self.pattern.category
    }
    fn description(&self) -> &str {
        &self.pattern.description
    }
}

impl HierarchyItem for DeletedItem {
    fn path(&self) -> &Path {
        &self.path
    }
    fn size(&self) -> u64 {
        self.size
    }
    fn category(&self) -> &str {
        &self.category
    }
    fn description(&self) -> &str {
        ""
    }
}

/// Ask the user for a project path. Empty input defaults to `.`.
/// Loops until a valid existing directory is given.
pub fn ask_path() -> io::Result<PathBuf> {
    loop {
        print!(
            "{} Project path to clean {} {} ",
            "?".cyan().bold(),
            "[.]".dimmed(),
            ">".bright_black()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        let n = io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            // Pressing Enter on a terminal keeps the "." default, but when stdin
            // is not a terminal (IDE runs, cron, CI) an empty/EOF input must not
            // silently scan the whole current directory (e.g. the home folder).
            if n == 0 && !io::stdin().is_terminal() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "no project path provided and stdin is not interactive — pass a path as an argument, e.g. `DevProjectsCleaner ./my-project`",
                ));
            }
            return Ok(PathBuf::from("."));
        }

        let path = PathBuf::from(trimmed);
        if path.is_dir() {
            return Ok(path);
        }
        println!(
            "{}  '{}' is not a directory.",
            "Invalid path.".yellow(),
            trimmed
        );
    }
}

/// Ask the user a yes/no question and return the answer.
///
/// Entering an empty line uses `default_yes`.
pub fn confirm(prompt: &str, default_yes: bool) -> io::Result<bool> {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        print!(
            "{} {} {} {} ",
            "?".cyan().bold(),
            prompt,
            hint.dimmed(),
            ">".bright_black()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_ascii_lowercase();

        match answer.as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" | "s" | "si" | "sí" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("{}  Please answer yes or no.", "Invalid answer.".yellow()),
        }
    }
}

/// What the user answered to a `confirm_or_list` prompt.
pub enum ContinueAnswer {
    Yes,
    No,
    List,
}

/// Ask "continue?" with a third option to print the full, uncollapsed list.
///
/// The caller is responsible for re-asking when `List` is returned.
pub fn confirm_or_list(prompt: &str) -> io::Result<ContinueAnswer> {
    loop {
        print!(
            "{} {} {} {} ",
            "?".cyan().bold(),
            prompt,
            "[y/N/l]".dimmed(),
            ">".bright_black()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_ascii_lowercase();

        match answer.as_str() {
            "" => return Ok(ContinueAnswer::No),
            "y" | "yes" => return Ok(ContinueAnswer::Yes),
            "n" | "no" => return Ok(ContinueAnswer::No),
            "l" | "list" | "full" | "v" => return Ok(ContinueAnswer::List),
            _ => println!(
                "{}  Enter y (yes), n (no) or l (list everything).",
                "Invalid answer.".yellow()
            ),
        }
    }
}

/// Format a duration in a human friendly way, e.g. "0.42s" or "1m 05s".
pub fn format_duration(d: Duration) -> String {
    if d.as_secs() >= 60 {
        format!("{}m {:02}s", d.as_secs() / 60, d.as_secs() % 60)
    } else {
        format!("{}.{:02}s", d.as_secs(), d.subsec_millis() / 10)
    }
}

/// Current UTC time as `YYYY-MM-DD HH:MM:SS UTC`.
pub fn utc_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let secs_of_day = secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02} UTC")
}

/// Days since 1970-01-01 -> (year, month, day). Proleptic Gregorian calendar.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { year + 1 } else { year }, month, day)
}

pub struct HierarchyOptions {
    pub max_depth: usize,
    pub max_leaves: usize,
    /// Render without ANSI colors (e.g. when writing the hierarchy to a log file).
    pub plain: bool,
}

impl Default for HierarchyOptions {
    fn default() -> Self {
        HierarchyOptions {
            max_depth: 3,
            max_leaves: 18,
            plain: false,
        }
    }
}

struct TreeNode<'a> {
    leaf: Option<&'a dyn HierarchyItem>,
    children: BTreeMap<String, TreeNode<'a>>,
    count: usize,
    size: u64,
}

struct TreeLine {
    left: String,
    right: String,
}

/// Render the matched items as a compact directory tree relative to `root`.
///
/// Instead of dumping every file name, this groups items by their parent
/// directories and collapses anything too deep or too numerous.
pub fn render_hierarchy<T: HierarchyItem>(
    items: &[T],
    root: &Path,
    opts: &HierarchyOptions,
) -> Vec<String> {
    let mut tree = TreeNode {
        leaf: None,
        children: BTreeMap::new(),
        count: 0,
        size: 0,
    };

    for item in items {
        let rel = item.path().strip_prefix(root).unwrap_or(item.path());
        let comps: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .filter(|c| !c.is_empty())
            .collect();

        let mut node = &mut tree;
        for (idx, comp) in comps.iter().enumerate() {
            node = node
                .children
                .entry(comp.clone())
                .or_insert_with(|| TreeNode {
                    leaf: None,
                    children: BTreeMap::new(),
                    count: 0,
                    size: 0,
                });
            if idx == comps.len() - 1 {
                node.leaf = Some(item);
            }
        }
    }

    finalize(&mut tree);

    let mut lines = Vec::new();
    let mut shown = 0usize;
    let mut overflow = 0usize;
    render_children(&tree, "", 0, opts, &mut lines, &mut shown, &mut overflow);

    if overflow > 0 {
        lines.push(TreeLine {
            left: format!("{} {} more items not shown", "…".yellow(), overflow),
            right: String::new(),
        });
    }

    // Pad the left column for a tidy two-column layout.
    let max_left = lines
        .iter()
        .map(|l| l.left.chars().count())
        .max()
        .unwrap_or(0);
    lines
        .into_iter()
        .map(|l| {
            let pad = max_left.saturating_sub(l.left.chars().count());
            let padding: String = " ".repeat(pad);
            if l.right.is_empty() {
                format!("  {}", l.left)
            } else {
                format!("  {}{}  {}", l.left, padding, l.right)
            }
        })
        .collect()
}

fn pluralize(n: usize, word: &str) -> String {
    if n == 1 {
        format!("{n} {word}")
    } else {
        format!("{n} {word}s")
    }
}

fn finalize(node: &mut TreeNode<'_>) {
    let mut count = usize::from(node.leaf.is_some());
    let mut size = node.leaf.map(|l| l.size()).unwrap_or(0);
    for child in node.children.values_mut() {
        finalize(child);
        count += child.count;
        size += child.size;
    }
    node.count = count;
    node.size = size;
}

fn render_children(
    node: &TreeNode<'_>,
    prefix: &str,
    depth: usize,
    opts: &HierarchyOptions,
    lines: &mut Vec<TreeLine>,
    shown: &mut usize,
    overflow: &mut usize,
) {
    let children: Vec<(&String, &TreeNode)> = node.children.iter().collect();
    let total = children.len();

    for (idx, (name, child)) in children.into_iter().enumerate() {
        let is_last = idx == total - 1;
        let branch = if is_last { "└── " } else { "├── " };

        let is_dir = child.leaf.map(|l| l.path().is_dir()).unwrap_or(true);
        let display_name = format!("{}{}", name, if is_dir { "/" } else { "" });

        if let Some(item) = child.leaf {
            if *shown >= opts.max_leaves {
                *overflow += 1;
                continue;
            }
            *shown += 1;
            let right = if opts.plain {
                format!(
                    "[{}]  {}  {}",
                    item.category(),
                    format_size(child.size),
                    item.description()
                )
            } else {
                format!(
                    "{}  {}  {}",
                    format!("[{}]", item.category()).cyan(),
                    format_size(child.size),
                    item.description().dimmed()
                )
            };
            lines.push(TreeLine {
                left: format!("{}{}{}", prefix, branch, display_name),
                right,
            });
        } else if depth + 1 >= opts.max_depth {
            let right = format!(
                "{}  {}  {}",
                pluralize(child.count, "item"),
                format_size(child.size),
                if opts.plain {
                    "(collapsed)".to_string()
                } else {
                    "(collapsed)".dimmed().to_string()
                }
            );
            lines.push(TreeLine {
                left: format!("{}{}{}", prefix, branch, display_name),
                right,
            });
        } else {
            let right = format!(
                "{}  {}",
                pluralize(child.count, "item"),
                format_size(child.size)
            );
            lines.push(TreeLine {
                left: format!("{}{}{}", prefix, branch, display_name),
                right,
            });
            let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            render_children(
                child,
                &child_prefix,
                depth + 1,
                opts,
                lines,
                shown,
                overflow,
            );
        }
    }
}
