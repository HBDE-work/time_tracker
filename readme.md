
# Tracker - Track your Worktime Today

- I was so annoyed by the time tracking tool of my company that I built my own
- `tracker` is capable to track time within multiple sequential sessions with pauses per day
    - time per session can be sliced further into tasks (optionally assigned to number `0 - 9`)
    - supports configurable time limits
- Data is stored as TOMLs per day under `~/.config/time_tracking/` (Linux) or `%APPDATA%/time_tracking/` (Windows)
- Run `tracker go` when you start, `tracker pause` for breaks, `tracker stop` when you're done
- Check Status of your hours
    -  `tracker status` shows the current day
    - `tracker status <dayname> <calendarweek> <year>` looks up past days where the arguments are optional and if omitted default to `current`
        - example: `tracker status monday 23` shows the status of monday of calendarweek 23 of the current year
- or use the TUI Mode...

---

## Installation

### Install directly from crates-io

`cargo install time-tracker-cli`

This installs the binary as `tracker` in your Cargo bin directory

---

## TUI Mode

- Just run `tracker` without any subcommand and the TUI launches

![TUI Mode](https://github.com/HBDE-work/time_tracker/blob/HEAD/docs/images/tui_mode.png?raw=true)

### Eventkeys
- send status updates through the `[g]o`, `[p]ause` and `[s]top` buttons or edit the displayed data via `[e]dit`

### Menus

#### Task Menu
- `[F1]` Task menu: Configure Task names and maximum worktimes
- When in Task menu, press a number between `0 - 9` and enter a task name
- a time limit for that task can be set by adding `:<time_limit>`
    - e.g. `Code Review:2.5`
- a globally daytime limit is configured through `m` when in Task Menu

#### Smartcard Toggle
- `[F2]` de- / activate SmartCard control if hardware supports it

#### Record History
- `[F3]` activates viewing different days within the TUI
- use `right` and `left` arrow keys to rotate through dayrecords

#### Switch display Unit
- `[F12]` Toggle between `Hours:Minutes:Seconds` and `Decimal` notation throughout the TUI

#### Exit
- Press `q` or `Esc` to quit

---

## CLI Reference

<!-- CLAP_DOC_GEN_START -->
**Usage:** `tracker <COMMAND>`

Simple CLI Timetracker

#### Commands

| Command | Description |
|---------|-------------|
| `go` | Start or resume tracking (optionally for a named task) |
| `pause` | Pause the current tracking |
| `stop` | Stop tracking for today |
| `status` | Show tracked hours (today by default, or a specific weekday of a given week/year) |
| `tui` | Launch interactive terminal UI |

### `tracker go`

Start or resume tracking (optionally for a named task)

#### Options

| Options | Description |
|------|------|
| `-t, --task <TASK>` | Track a named task (e.g. --task "Code Review") |

### `tracker pause`

Pause the current tracking

### `tracker stop`

Stop tracking for today

### `tracker status`

Show tracked hours (today by default, or a specific weekday of a given week/year)

#### Arguments

| Arguments | Description |
|------|------|
| `<DAY>` | Weekday name (e.g. "monday", "tue") |
| `<WEEK>` | Calendar week number (1-53, defaults to current week) |
| `<YEAR>` | Year (defaults to current year) |

#### Options

| Options | Description |
|------|------|
| `-d, --decimal` | Display time in decimal hours format |

### `tracker tui`

Launch interactive terminal UI

<!-- CLAP_DOC_GEN_END -->

---

## Development

### Building

- this repo uses [cargo make](https://github.com/sagiegurari/cargo-make) as build helper
- the app can be crosscompiled for both Linux and Windows using `cargo make deploy`
    - cargo will crash if necessary toolchains are not installed
- a zip package can be created by running `cargo make package`

---
