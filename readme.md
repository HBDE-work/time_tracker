
# Tracker - Track your Worktime Today

- I was so annoyed by the time tracking tool of my company that I built my own
- `tracker` is capable to track time within multiple sequential sessions with pauses per day
    - time per session can be sliced further into tasks (optionally assigned to number `0 - 9`)
- optionally autotrack via (un-) plugging a Smartcard
- Past Day Record Status can be retrieved
- Day Records can be edited either directly in the files or via the TUI
- Data is stored as TOMLs per day under `~/.config/time_tracking/` (Linux) or `%APPDATA%/time_tracking/` (Windows)
- Run `tracker go` when you start, `tracker pause` for breaks, `tracker stop` when you're done
- Check Status of your hours
    -  `tracker status` shows the current day
    - `tracker status <dayname> <calendarweek> <year>` looks up past days where the arguments are optional and if omitted default to `current`
        - example: `tracker status monday 23` shows the status of monday of calendarweek 23 of the current year

---

## Installation

### Install directly from crates-io

`cargo install time-tracker-cli`

This installs the binary as `tracker` in your Cargo bin directory

---

## TUI Mode

- Just run `tracker` without any subcommand and the TUI launches
- Press `q` or `Esc` to quit

![TUI Mode](https://github.com/HBDE-work/time_tracker/blob/HEAD/docs/images/tui_mode.png?raw=true)

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
