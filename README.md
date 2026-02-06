# git-schedule

Schedule git commits for later. Like "delayed send" for your code.

```bash
git add feature.rs
git-schedule "feat: add awesome feature" --in 2h
# ✓ Scheduled commit for 3:00 PM (in 2 hours)
# Files are captured and unstaged. Commit happens automatically later.
```

## Why?

- **Commit during work hours** - Write code at night, commit at 9 AM
- **Batch commits** - Stage changes throughout the day, commit them together later
- **Time-based workflows** - Schedule commits to align with your team's timezone
- **"I'll commit this after the tests pass"** - Schedule now, let it run later

## Features

- Schedule commits up to 24 hours in advance
- Relative time (`--in 2h`, `--in 30m`) or absolute time (`--at 9am`, `--at 14:00`)
- Optional auto-push after commit (`--push`)
- View, edit, and cancel scheduled commits
- Interactive file selection if nothing is staged
- System notifications on success/failure
- Automatic retry queue for failed commits
- Lightweight background daemon (auto-starts when needed)

## Installation

### macOS / Linux

#### From Releases

Download the latest release for your platform from [GitHub Releases](https://github.com/mafex11/Git-Schedule/releases):

- **macOS (Apple Silicon):** `git-schedule-macos-aarch64.tar.gz`
- **Linux (x86_64):** `git-schedule-linux-x86_64.tar.gz`

```bash
# Extract and install (example for macOS)
tar -xzf git-schedule-macos-aarch64.tar.gz
sudo mv git-schedule git-schedule-daemon /usr/local/bin/
```

#### From Source (requires Rust)

```bash
# Clone the repository
git clone https://github.com/mafex11/git-schedule.git
cd git-schedule

# Build release binaries
cargo build --release

# Install to your PATH
cargo install --path cli
cargo install --path daemon

# Or copy manually
cp target/release/git-schedule /usr/local/bin/
cp target/release/git-schedule-daemon /usr/local/bin/
```

### Windows

**One-liner install (PowerShell):**

```powershell
irm https://raw.githubusercontent.com/mafex11/git-schedule/main/install.ps1 | iex
```

**Or manually:**

1. Download `git-schedule-windows-x86_64.zip` from [GitHub Releases](https://github.com/mafex11/Git-Schedule/releases)
2. Extract to a folder (e.g., `C:\Program Files\git-schedule\`)
3. Add that folder to your PATH
4. Open a new terminal and verify: `git-schedule --help`

**Or build from source with Rust:**

```powershell
git clone https://github.com/mafex11/git-schedule.git
cd git-schedule
cargo build --release
# Copy target\release\git-schedule.exe and git-schedule-daemon.exe to your PATH
```

### Verify Installation

```bash
git-schedule --version
git-schedule --help
```

## Quick Start

```bash
# 1. Stage your changes (normal git workflow)
git add src/feature.rs

# 2. Schedule the commit
git-schedule "feat: add new feature" --in 2h

# 3. That's it! Files are captured and will be committed in 2 hours
#    You can continue working - the staged files are now unstaged
```

## Usage

### Schedule a Commit

```bash
# Relative time
git-schedule "commit message" --in 30m      # in 30 minutes
git-schedule "commit message" --in 2h       # in 2 hours
git-schedule "commit message" --in 1h30m    # in 1 hour 30 minutes

# Absolute time
git-schedule "commit message" --at 9am      # at 9:00 AM today (or tomorrow if passed)
git-schedule "commit message" --at 9:30am   # at 9:30 AM
git-schedule "commit message" --at 14:00    # at 2:00 PM (24-hour format)
git-schedule "commit message" --at "2:30 PM"

# With auto-push
git-schedule "feat: ready to ship" --in 1h --push
```

### View Scheduled Commits

```bash
# List all pending commits
git-schedule list

# Example output:
# Scheduled Commits
# ────────────────────────────────────────────────────────────
# ○ a1b2c3d4 03:00 PM (1h 30m) feat: add new feature
#     my-project @ main
# ○ e5f6g7h8 05:00 PM (3h 30m) fix: resolve bug
#     my-project @ main [push]
```

### Check Status

```bash
git-schedule status

# Example output:
# git-schedule Status
#
# ● Daemon running (PID: 12345)
#   Uptime: 2h 15m
#
# Pending: 2
# Failed: 0
#
# Next Commit
#   Time: 03:00 PM (1h 30m)
#   Message: feat: add new feature
#   Repo: my-project @ main
```

### View a Scheduled Diff

```bash
git-schedule show a1b2c3d4

# Shows the full diff that will be committed
```

### Edit a Schedule

```bash
# Change the message
git-schedule edit a1b2c3d4 --message "feat: better message"

# Reschedule the time
git-schedule edit a1b2c3d4 --in 3h
git-schedule edit a1b2c3d4 --at 5pm

# Change both
git-schedule edit a1b2c3d4 --message "new message" --in 4h
```

### Cancel a Schedule

```bash
git-schedule cancel a1b2c3d4
# ✓ Cancelled schedule a1b2c3d4
```

### Handle Failed Commits

If a commit fails (e.g., branch was deleted, merge conflict), it moves to the failed queue:

```bash
# View failed commits
git-schedule failed

# Retry a failed commit (re-stages the files so you can fix and reschedule)
git-schedule retry a1b2c3d4
# ✓ Files from schedule a1b2c3d4 have been re-staged
# Run: git-schedule "message" --in <time>
```

### Daemon Management

The daemon starts automatically when you schedule something. Manual control:

```bash
git-schedule daemon start    # Start the daemon
git-schedule daemon stop     # Stop the daemon
git-schedule daemon restart  # Restart the daemon
```

## How It Works

```
┌─────────────────────────────────────────────────────────────┐
│  git add file.rs                                            │
│  git-schedule "message" --in 2h                             │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  CLI captures staged changes as a patch file                │
│  Unstages the files (git reset)                             │
│  Sends schedule to daemon                                   │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  Daemon stores schedule in ~/.git-schedule/                 │
│  Waits until scheduled time                                 │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼ (when time arrives)
┌─────────────────────────────────────────────────────────────┐
│  Daemon applies patch to repo                               │
│  Creates the commit                                         │
│  Optionally pushes to remote                                │
│  Sends system notification                                  │
└─────────────────────────────────────────────────────────────┘
```

### Storage

All data is stored in:
- **macOS/Linux:** `~/.git-schedule/`
- **Windows:** `%LOCALAPPDATA%\git-schedule\`

```
# Unix
~/.git-schedule/
├── schedules.json      # Schedule metadata
├── patches/            # Captured diffs
│   └── abc123.patch
├── logs/               # Daemon logs
│   └── daemon.log.2024-01-15
├── daemon.pid          # Daemon process ID
└── daemon.sock         # Unix socket for IPC

# Windows
%LOCALAPPDATA%\git-schedule\
├── schedules.json
├── patches\
├── logs\
└── daemon.pid
# (Windows uses TCP localhost:7392 for IPC instead of socket)
```

## Configuration

### Limits

- **Maximum schedule time:** 24 hours
- **Maximum pending schedules:** 10

### What Happens If...

| Scenario | Behavior |
|----------|----------|
| Machine sleeps through scheduled time | Commit marked as "missed", moved to failed queue |
| Branch is deleted before commit | Commit fails, moved to failed queue with error |
| You're on a different branch | Commit fails (branch mismatch), moved to failed queue |
| Merge conflict when applying patch | Commit fails, files re-staged for manual resolution |
| Daemon crashes | Restarts automatically on next `git-schedule` command |

## Command Reference

| Command | Description |
|---------|-------------|
| `git-schedule "msg" --in TIME` | Schedule commit in relative time |
| `git-schedule "msg" --at TIME` | Schedule commit at absolute time |
| `git-schedule "msg" --in TIME --push` | Schedule commit + push |
| `git-schedule list` | List pending schedules |
| `git-schedule status` | Show daemon status and next commit |
| `git-schedule show ID` | View scheduled diff |
| `git-schedule edit ID [options]` | Edit message or time |
| `git-schedule cancel ID` | Cancel a schedule |
| `git-schedule failed` | List failed/missed commits |
| `git-schedule retry ID` | Re-stage files from failed commit |
| `git-schedule daemon start` | Start daemon manually |
| `git-schedule daemon stop` | Stop daemon |
| `git-schedule daemon restart` | Restart daemon |

### Time Formats

**Relative (`--in`):**
- `30m` - 30 minutes
- `2h` - 2 hours
- `1h30m` - 1 hour 30 minutes

**Absolute (`--at`):**
- `9am`, `9:30am`, `9:30 AM` - 12-hour format
- `14:00`, `9:30` - 24-hour format

## Troubleshooting

### "Daemon not running"

```bash
git-schedule daemon start
```

### "Queue full"

You have 10 pending schedules. Cancel some:

```bash
git-schedule list
git-schedule cancel <id>
```

### "Branch mismatch"

You scheduled a commit on `main` but switched to `feature`. Either:
- Switch back to `main` and wait for the commit
- Cancel the schedule: `git-schedule cancel <id>`

### View Logs

```bash
cat ~/.git-schedule/logs/daemon.log.*
```

### Reset Everything

```bash
# macOS/Linux
git-schedule daemon stop
rm -rf ~/.git-schedule

# Windows (PowerShell)
git-schedule daemon stop
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\git-schedule"
```

## Development

### Project Structure

```
git-schedule/
├── cli/                 # CLI binary
│   └── src/
│       ├── main.rs      # Entry point, clap setup
│       ├── commands/    # Command implementations
│       ├── git.rs       # Git operations
│       ├── client.rs    # Daemon communication
│       └── time_parser.rs
├── daemon/              # Background daemon
│   └── src/
│       ├── main.rs      # Daemon entry point
│       ├── scheduler.rs # Timer loop
│       ├── executor.rs  # Commit execution
│       ├── server.rs    # Unix socket server
│       └── storage.rs   # JSON persistence
└── shared/              # Shared library
    └── src/
        ├── types.rs     # Schedule, ScheduleStatus
        ├── protocol.rs  # IPC messages
        └── config.rs    # Paths, constants
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test
```

### Running Locally

```bash
# Run CLI (debug)
cargo run --bin git-schedule -- --help

# Run daemon (debug)
cargo run --bin git-schedule-daemon
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- CLI parsing by [clap](https://github.com/clap-rs/clap)
- Git operations by [git2](https://github.com/rust-lang/git2-rs)
- Notifications by [notify-rust](https://github.com/hoodie/notify-rust)
