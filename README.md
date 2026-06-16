<div align="center">

# wsup

### *A beautiful TUI localhost process manager with real-time graphs*

[![Crates.io](https://img.shields.io/crates/v/wsup?style=for-the-badge&logo=rust&color=orange)](https://crates.io/crates/wsup)
[![Downloads](https://img.shields.io/crates/d/wsup?style=for-the-badge&logo=rust&color=blue)](https://crates.io/crates/wsup)
[![License](https://img.shields.io/badge/license-MIT-green.svg?style=for-the-badge)](LICENSE)
[![Stars](https://img.shields.io/github/stars/NotKiwy/wsup?style=for-the-badge&logo=github&color=yellow)](https://github.com/NotKiwy/wsup/stargazers)
[![Issues](https://img.shields.io/github/issues/NotKiwy/wsup?style=for-the-badge&logo=github&color=red)](https://github.com/NotKiwy/wsup/issues)

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Built with Ratatui](https://img.shields.io/badge/built%20with-ratatui-blue?style=flat-square)](https://github.com/ratatui-org/ratatui)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](CONTRIBUTING.md)

---

**wsup** (pronounced "what's up") is a terminal UI for monitoring and managing processes running on localhost ports. Built with Rust and [ratatui](https://github.com/ratatui-org/ratatui).

[Features](#features) •
[Installation](#installation) •
[Usage](#usage) •
[Demo](#demo) •
[Contributing](#contributing)

</div>

## Demo

### Basic Navigation & Sorting
Press `s` to cycle through sort modes, `o` to reverse order

![Basic Demo](preview/demo-basic.gif)

### CLI Flag: --sort
Start with a specific sort mode

![Sort Flag Demo](preview/demo-sort.gif)

### CLI Flag: --filter  
Start with processes pre-filtered, or use `/` to search

![Filter Flag Demo](preview/demo-filter.gif)

### Detail View with Live Graphs
Press `Enter` on any process to see real-time metrics

![Detail Demo](preview/demo-detail.gif)

## Quick Start

```bash
# Install
cargo install wsup

# Run
wsup

# Run with options
wsup --sort cpu --filter node
```

## Features

- Live CPU/Memory/Connection graphs with 60-point history
- Sort by Port, CPU, Memory, Connections, or Name
- Filter processes as you type
- Color-coded ports (HTTP, databases, dev servers)
- Kill processes with confirmation
- Vim keybindings (j/k) support
- Auto-refresh every 2 seconds

## Installation

### Via Cargo
```bash
cargo install wsup
```

### From Source
```bash
git clone https://github.com/NotKiwy/wsup.git
cd wsup
cargo install --path .
```

## Usage

### Basic Usage
```bash
# Start wsup
wsup

# Start with specific sort mode
wsup --sort cpu

# Start with a filter
wsup --filter node

# Both together
wsup --sort memory --filter 3000

# Quick kill process on port (without TUI)
wsup --kill 3000

# Show help
wsup --help
```

### Keyboard Shortcuts

#### List View
| Key | Action |
|-----|--------|
| `↑/k` | Move up |
| `↓/j` | Move down |
| `/` | Search/filter |
| `s` | Cycle sort mode |
| `o` | Toggle sort order (ascending/descending) |
| `Enter` | Show process details |
| `x` or `d` | Kill selected process |
| `r` | Refresh processes |
| `q` | Quit |

#### Detail View
| Key | Action |
|-----|--------|
| `Esc` | Back to list |
| `x` | Kill process |

#### Search Mode
| Key | Action |
|-----|--------|
| `Type` | Filter processes |
| `Backspace` | Delete character |
| `Esc` or `Enter` | Exit search mode |

## Features Breakdown

### Process List View
- **Port Number** - Color-coded by service type
- **PID** - Process identifier
- **CPU %** - Current CPU usage
- **Memory** - RAM consumption (formatted)
- **Connections** - Active network connections
- **Name** - Process name
- **Command** - Full command with arguments

### Detail View
When you press `Enter` on a process, you get:
- **Process Info** - Name, port, PID, current stats
- **CPU History Graph** - Sparkline showing last 60 data points
- **Memory History Graph** - Visual memory usage trend
- **Connections Graph** - Active connections over time
- **Command Display** - Full command with all arguments

### Port Color Coding
- **Cyan** - HTTP/HTTPS (80, 443, 8080, 8443)
- **Green** - Dev servers (3000-3999)
- **Magenta** - Application servers (5000-5999)
- **Red** - Redis (6379)
- **Blue** - MongoDB (27017, 27018)
- **Light Blue** - PostgreSQL (5432, 5433)
- **Light Magenta** - MySQL (3306)
- **Yellow** - Various servers (8000-8999)

## Technical Details

### Built With
- **[ratatui](https://github.com/ratatui-org/ratatui)** - Terminal UI framework
- **[crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation
- **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - System information
- **[clap](https://github.com/clap-rs/clap)** - CLI argument parsing

### Architecture
- **Rust Edition 2021**
- **Zero unsafe code** - Memory-safe by design
- **250ms event polling, 2s auto-refresh**
- **Process detection** - Uses `lsof` for port mapping
- **CPU metrics** - Direct `ps` command integration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Contributors

<a href="https://github.com/NotKiwy/wsup/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=NotKiwy/wsup" />
</a>

## Stats

<div align="center">

![GitHub Stats](https://img.shields.io/github/commit-activity/m/NotKiwy/wsup?style=flat-square&logo=github&label=Commits)
![GitHub last commit](https://img.shields.io/github/last-commit/NotKiwy/wsup?style=flat-square&logo=github)
![GitHub repo size](https://img.shields.io/github/repo-size/NotKiwy/wsup?style=flat-square&logo=github)

</div>

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [ratatui](https://github.com/ratatui-org/ratatui)
- Inspired by [htop](https://htop.dev/), [bottom](https://github.com/ClementTsang/bottom), and [glances](https://nicolargo.github.io/glances/)
- Terminal recording with [VHS](https://github.com/charmbracelet/vhs)

## Why "wsup"?

**wsup** = "what's up" - A casual way to check on your localhost processes. Quick to type, easy to remember, shows you what's up on your ports.

---

If you find **wsup** useful:
- ⭐ [Star this repository](https://github.com/NotKiwy/wsup/stargazers)
- 🐛 [Report bugs](https://github.com/NotKiwy/wsup/issues/new)
- 💡 [Suggest features](https://github.com/NotKiwy/wsup/issues/new)
- 🔀 [Contribute code](CONTRIBUTING.md)

---

<div align="center">

Made with ❤️ and Rust

[![GitHub](https://img.shields.io/badge/GitHub-NotKiwy%2Fwsup-181717?style=for-the-badge&logo=github)](https://github.com/NotKiwy/wsup)
[![X Follow](https://img.shields.io/badge/FOLLOW-@0KIWY-000000?style=for-the-badge&logo=x)](https://x.com/0kiwy)

</div>
