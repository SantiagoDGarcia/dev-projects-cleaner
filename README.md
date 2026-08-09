# DevProjectsCleaner

> Professional project cleaner — removes temporary files, build artifacts, and caches across many programming languages using Rust for the best perfomance.

`DevProjectsCleaner` scans any project directory for the cruft that accumulates during development (`node_modules`, `target`, `.dart_tool`, `__pycache__`, `.DS_Store`, logs, editor swap files, and much more), shows you a **simple hierarchy** of what it found, and — after your explicit confirmation — moves it to the **recycle bin** so nothing is ever truly lost.

It is designed for developers who juggle multiple projects and want one safe, cross-platform command to tidy them up.

## ✨ Features

- **Interactive by default** — asks the **path first**, then shows a compact directory tree (not a wall of file names), and asks before touching anything.
- **SAFE MODE built-in** — by default items go to the recycle bin (recoverable). Opt out to permanently delete.
- **Parameterizable patterns** — folders, file names and name patterns are configurable through a TOML config file or the `--add` / `--ignore` flags. No code changes needed.
- **Automatic detection** — recognizes Python, Node, Dart/Flutter, Rust, Go, C/C++, Java, iOS, Jupyter, Terraform, IDE caches, OS metadata, logs, and more (full list below).
- **Full-list preview** — at the confirmation prompt, press `l` to see the complete uncollapsed hierarchy before deciding.
- **Run log** — every run writes a log file (path, mode, deleted items, space freed, errors) that you can keep for audit.
- **Dry run** — preview everything that would be cleaned without touching a single byte.
- **Verbose mode** — when you _do_ want the full list of every item.
- **Cross-platform** — macOS (arm64/x86_64), Windows, and Linux (arm64/x86_64).
- **Safe defaults** — never touches your `.git`, `.svn`, or `.hg` directories.
- **Fast** — parallel-free, dependency-light Rust binary with no runtime.

## 🚀 Installation

### Prebuilt binaries

Prebuilt binaries for macOS, Windows and Linux are attached in [Release](https://github.com/<your-user>/dev-projects-cleaner/releases):

| Platform              | File                                                   |
| --------------------- | ------------------------------------------------------ |
| macOS (Apple Silicon) | `DevProjectsCleaner-macos-arm64`                       |
| macOS (Intel)         | `DevProjectsCleaner-macos-x86_64`                      |
| Linux (x86_64)        | `DevProjectsCleaner-linux-x86_64`                      |
| Linux (ARM64)         | `DevProjectsCleaner-linux-arm64`                       |
| Windows (x86_64)      | `DevProjectsCleaner-windows-x86_64.exe`                |
| Windows (ARM64)       | `DevProjectsCleaner-windows-arm64.exe` _(built by CI)_ |

#### macOS Gatekeeper bypass

macOS (including Tahoe 26.6) may block the downloaded binary because it is not digitally signed. If you see a warning stating that Apple could not verify the app, you must remove the quarantine attribute.

Open your Terminal and execute the following command, replacing the path with the actual location of your downloaded file:

````bash
xattr -cr /path/to/downloaded/DevProjectsCleaner-macos-arm64

## 📖 Usage

```bash
DevProjectsCleaner [OPTIONS] [PATH]
````

Run it from inside a project, or point it at one:

```bash
DevProjectsCleaner                 # prompts for a path, then cleans it
DevProjectsCleaner ~/code/my-api   # clean a specific project directly
```

Run without arguments and the first thing it asks is the **project path** (press `Enter` to use the current directory):

```
? Project path to clean [.] >
```

### The interactive flow

```
? Project path to clean [.] > ~/code/my-api
DevProjectsCleaner v1.0.0 — Clean your project artifacts
Target: /Users/you/code/my-api

Scanning /Users/you/code/my-api …

✓ found 10 items (35.0 B), scanned in 0.01s

Breakdown by category:
  Logs:        ████████████████████    5 items  25.0 B
  Rust:        ████                    1 item    2.0 B
  Node:        ████                    1 item    2.0 B

Where they are:
  ├── .DS_Store      [OS]  2.0 B  macOS folder metadata
  ├── node_modules/  [Node]  2.0 B  Node.js dependencies
  ├── src/           5 items  25.0 B
  │   ├── app1.log   [Logs]  5.0 B  Log files
  │   └── app2.log   [Logs]  5.0 B  Log files
  ├── target/        [Rust]  2.0 B  Build artifacts

? The items above will be cleaned. Continue? [y/N/l] > l

Full hierarchy (no collapsing):
  ├── a/                     1 item  2.0 B
  │   └── b/                 1 item  2.0 B
  │       └── c/             1 item  2.0 B
  │           └── d/         1 item  2.0 B
  │               └── x.log  [Logs]  2.0 B  Log files
  └── src/                   1 item  2.0 B
      └── y.log              [Logs]  2.0 B  Log files

? The items above will be cleaned. Continue? [y/N/l] > y

? Run in SAFE MODE? Items go to the recycle bin (recoverable). [Y/n] >

Mode: SAFE MODE (recycle bin)

═══════════════════════════════════════
              SUMMARY
═══════════════════════════════════════
  Items removed:  10
  Space freed:    35.0 B
  Mode:           Recycle Bin
  Time taken:     0.42s
  Log written:    ~/Library/Application Support/dev-projects-cleaner/clean-2026-08-08_13_41_36_UTC.log
```

1. **Scan** — shows a category breakdown and a _hierarchy_ of where items live (collapsed for readability).
2. **Continue?** — press `n` to abort and touch nothing; press **`l`** to print the full uncollapsed hierarchy and get asked again.
3. **SAFE MODE?** — press `Enter` (yes) to send items to the recycle bin, or `n` to permanently delete.

Use `--log <path>` to write it somewhere else, or `--no-log` to disable.

### Command-line options

| Flag              | Description                                                                    |
| ----------------- | ------------------------------------------------------------------------------ |
| `-f`, `--force`   | Permanently delete. Skips the SAFE MODE question.                              |
| `-v`, `--verbose` | List every single item instead of the compact hierarchy.                       |
| `-n`, `--dry-run` | Preview what would be cleaned without deleting anything.                       |
| `-y`, `--yes`     | Skip all confirmation prompts (uses SAFE MODE unless combined with `--force`). |
| `--add <NAME>`    | Add an extra pattern to clean (repeatable).                                    |
| `--ignore <NAME>` | Never clean a name — even built-in patterns are skipped (repeatable).          |
| `--config <PATH>` | Use a custom TOML config file.                                                 |
| `--log <PATH>`    | Write the run log to a custom path.                                            |
| `--no-log`        | Do not write a run log file.                                                   |
| `-h`, `--help`    | Show help.                                                                     |
| `-V`, `--version` | Show version.                                                                  |

### Common examples

```bash
DevProjectsCleaner                   # ask for a path, then clean it interactively
DevProjectsCleaner -n                # preview what would be cleaned
DevProjectsCleaner -n -v             # preview with every item listed
DevProjectsCleaner -y                # clean now, no prompts (safe mode)
DevProjectsCleaner -yf               # clean now, permanently, no prompts
DevProjectsCleaner ~/code -f         # permanent clean of a directory
DevProjectsCleaner --add "*.bak"     # also clean *.bak files
DevProjectsCleaner --ignore node_modules   # never touch node_modules
```

## ⚙️ Parameterizing patterns

Every folder, file name or name pattern is data-driven and can be customized
without touching code. Sources are merged, in order of precedence:

1. **Built-in defaults** — [`patterns/defaults.json`](patterns/defaults.json), embedded into the binary at compile time.
2. **Global config** — `~/.config/dev-projects-cleaner/config.toml`.
3. **Project config** — `dev-projects-cleaner.toml` inside the target directory (auto-detected).
4. **Explicit config** — `--config <path>`.
5. **CLI flags** — `--add` / `--ignore`.

Excludes always win: a name that is ignored is never entered nor cleaned.

### Config file format

```toml
# Add your own patterns (names support * prefix/suffix wildcards)
[[patterns]]
names = ["myplugin", "*.blend5"]
kind = "dir"          # "dir", "file" or "any"
scope = "any"         # "any" (any depth) or "root" (only at the project root)
description = "My custom plugin folder"
category = "Custom"

[[patterns]]
names = ["*.tmp"]
kind = "file"
description = "Temp files"
category = "Custom"

# Names that must never be cleaned or descended into
[exclude]
names = [".git", "node_modules"]
```

A `dev-projects-cleaner.toml` placed in a project is picked up automatically when you
clean that directory, which makes it easy to share per-project rules.

## 🧹 What it cleans

The built-in patterns live in [`patterns/defaults.json`](patterns/defaults.json)
(embedded into the binary), using the same JSON format as the config file.
Anything easy to add:

| Category        | Examples                                                                                                                                               |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Python          | `__pycache__`, `.pytest_cache`, `.mypy_cache`, `.ruff_cache`, `.tox`, `venv`, `.venv`, `*.egg-info`, `.coverage`, `htmlcov`, `*.pyc`, `*.pyo`, `*.pyd` |
| Node            | `node_modules`, `.next`, `.nuxt`, `.eslintcache`, `.parcel-cache`, `*.tsbuildinfo`                                                                     |
| Dart/Flutter    | `.dart_tool`, `*.dart_tool`, `.packages`, `.flutter-plugins`, `.flutter-plugins-dependencies`                                                          |
| Rust            | `target`                                                                                                                                               |
| Go              | `vendor` _(only at the project root — avoids Django's `static/_/vendor`)\*                                                                             |
| C/C++           | `*.o`, `*.obj`                                                                                                                                         |
| Java/JVM        | `.gradle`, `*.class`                                                                                                                                   |
| iOS/Swift       | `Pods`, `.build`                                                                                                                                       |
| Jupyter         | `.ipynb_checkpoints`                                                                                                                                   |
| 3D/Blender      | `*.blend1` … `*.blend4`                                                                                                                                |
| Infra/Terraform | `.terraform`                                                                                                                                           |
| IDE/Editor      | `.idea`, `.history`, `*~`, `*.swp`, `*.swo`                                                                                                            |
| OS metadata     | `.DS_Store`, `Thumbs.db`, `desktop.ini`                                                                                                                |
| Logs            | `*.log`                                                                                                                                                |
| CSS/Sass        | `.sass-cache`                                                                                                                                          |
| General build   | `dist`, `build`, `out`                                                                                                                                 |

> **Never touched:** `.git`, `.svn`, `.hg`.

## 🔒 Safety model

- **Default: recycle bin.** Nothing is permanently destroyed without you asking for it.
- **Explicit confirmation.** The tool always shows you what it found before doing anything.
- **Dry run** to inspect without side effects.
- **`.git` and friends are always skipped**, so you can't accidentally nuke version history.

## 🤝 Contributing

- Found a pattern missing from the defaults? Open an issue, or add it to
  [`patterns/defaults.json`](patterns/defaults.json) — it's one entry.
- Need a pattern only for your projects? Use the [config file](#parameterizing-patterns),
  no code required.
- Bug or feature request? Open an issue.
