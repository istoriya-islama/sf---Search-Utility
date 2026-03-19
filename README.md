# sf - Simple File and Folder Search Utility

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)

A fast and convenient command-line utility for searching files and directories, written in Rust.

## 🚀 Features

- Search files and folders by substring
- Support for glob patterns (`*`, `?`, `[]`)
- Case-sensitive and case-insensitive search
- Configurable recursion depth
- Custom starting directory for search
- Filter by file size (`+100M`, `-1G`, `500K`)
- Filter by modification date (`-7` days, `+30` days)
- Exclude paths from search (`--exclude node_modules`)
- JSON output for scripting (`--json`)
- Statistics mode with total size (`--stats`)

## 📦 Installation

### From Source

```bash
git clone https://github.com/istoriya-islama/sf---Search-Utility.git
cd sf
cargo build --release
```

The compiled binary will be located at `sf.exe` (Windows) or `sf` (Linux/macOS).

### Adding to PATH

For convenience, add the directory with the executable to your PATH or copy `sf.exe` to a directory already in your PATH.

## 🔧 Usage

![Search example](screenshots/scren1.png)
![Search example](screenshots/scren2.png)
![Search example](screenshots/scren3.png)

### Basic Syntax

```bash
sf [OPTIONS] <PATTERN>
```

### Examples

**Searching for files:**
```bash
sf "config"                    # Find all files containing "config" in name
sf "*.txt"                     # Find all .txt files (requires -g)
sf -g "*.rs"                   # Find all Rust files (glob pattern)
```

**Searching for folders:**
```bash
sf -d "src"                    # Find all folders containing "src"
sf -d "test"                   # Find folders with "test" in name
```

**Case-insensitive search:**
```bash
sf -i "README"                 # Will find readme, README, ReadMe, etc.
```

**Specifying search directory:**
```bash
sf -s "C:\Projects" "main"     # Search in C:\Projects
sf -s "../" -d "docs"          # Search in parent directory
```

**Limiting search depth:**
```bash
sf -r 2 "config"               # Search only up to 2 levels deep
sf -r 0 "readme.md"            # Search only in current directory
```

**Filtering by size:**
```bash
sf -g "*.mp4" --size +100M     # Find videos larger than 100 MB
sf -g "*.log" --size -1M       # Find logs smaller than 1 MB
sf -g "*.iso" --size +4G       # Find ISOs larger than 4 GB
```

**Filtering by modification date:**
```bash
sf -g "*.log" --mtime -7       # Modified in the last 7 days
sf -g "*.bak" --mtime +30      # Older than 30 days
```

**Excluding paths:**
```bash
sf -g "*.js" --exclude node_modules          # Skip node_modules
sf -g "*.rs" --exclude target --exclude .git  # Skip multiple dirs
```

**JSON output:**
```bash
sf -g "*.rs" --json            # Output results as JSON
sf -g "*.rs" --json | jq '.count'  # Count via jq
```

**Statistics mode:**
```bash
sf -g "*.jpg" --stats          # Show file sizes + total
sf -g "*.mp4" --size +100M --stats  # Big videos with total size
```

**Combining options:**
```bash
sf -d -i -s "C:\Projects" "test"                    # Folders with "test", case-insensitive
sf -g -r 3 "*.json"                                  # JSON files up to 3 levels deep
sf -g "*.log" --mtime -7 --exclude node_modules      # Recent logs, skip node_modules
sf -g "*.mp4" --size +500M --stats --json            # Big videos, stats, JSON output
```

## ⚙️ Options

| Option | Long Form | Description |
|--------|-----------|-------------|
| `-d` | `--dir` | Search for directories instead of files |
| `-i` | `--ignore-case` | Ignore case when searching |
| `-s <PATH>` | `--start <PATH>` | Starting directory for search (default `.`) |
| `-r <NUM>` | `--max-depth <NUM>` | Maximum recursion depth (-1 = unlimited) |
| `-g` | `--glob` | Use glob patterns (`*`, `?`, `[]`) |
| | `--size <FILTER>` | Filter by size: `+100M` (greater), `-1G` (less), `500K` (exact) |
| | `--mtime <DAYS>` | Filter by modification date: `-7` (last 7 days), `+30` (older than 30 days) |
| | `--exclude <STR>` | Exclude paths containing this substring (repeatable) |
| | `--json` | Output results as JSON |
| | `--stats` | Show file sizes and total size summary |
| `-h` | `--help` | Show help information |
| `-V` | `--version` | Show version |

## 📝 Notes

- By default, search starts in the current directory (`.`)
- Default recursion depth: `-1` (unlimited)
- To use glob patterns, you must specify the `-g` flag
- In PowerShell/CMD, it's recommended to use quotes for patterns with spaces
- Size suffixes: `K` / `KB`, `M` / `MB`, `G` / `GB`
- `--exclude` can be specified multiple times to skip several paths
- `--json` and `--stats` can be combined freely

## 🛠️ Dependencies

- [clap](https://github.com/clap-rs/clap) - Command-line argument parsing
- [walkdir](https://github.com/BurntSushi/walkdir) - Recursive directory traversal
- [globset](https://github.com/BurntSushi/ripgrep/tree/master/crates/globset) - Glob pattern support
- [colored](https://github.com/mackwic/colored) - Terminal color output
- [serde](https://github.com/serde-rs/serde) + [serde_json](https://github.com/serde-rs/json) - JSON serialization

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Suggestions and pull requests are welcome!