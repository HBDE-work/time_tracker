
# Tracker - Track Worktime Today

- I was so annoyed by the time tracking tool of my company that I built my own
- tracker is capable to track time within multiple sequential sessions with pauses per day
    - time per session can be sliced further into tasks (optionally assigned to number `0 - 9`)
- optionally autotrack via (un-) plugging a Smartcard
- Data is stored as TOMLs per day under `~/.config/time_tracking/` (Linux) or `%APPDATA%/time_tracking/` (Windows)
- Run `tracker go` when you start, `tracker pause` for breaks, `tracker stop` when you're done
- Check your hours with `tracker status` or look up past days like `tracker status monday 23`

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
`tracker [OPTIONS] <COMMAND>`

### Commands

#### `go`:                  Start or resume tracking (optionally for a named task)
- args:
    - `-t, --task <TASK>`:    Track a named task (e.g. --task "Code Review")
    - `-h, --help`:    Print help

#### `pause`:               Pause the current tracking
- args:
    - `-h, --help`:    Print help

#### `stop`:                Stop tracking for today
- args:
    - `-h, --help`:    Print help

#### `status`:              Show tracked hours (today by default, or a specific weekday of a given week/year)
- args:
    - `-h, --help`:    Print help

#### `tui`:                 Launch interactive terminal UI
- args:
    - `-h, --help`:    Print help

#### `help`:                Print this message or the help of the given subcommand(s)


### Global Options

- `-h, --help`:              Print help
- `-V, --version`:           Print version

<!-- CLAP_DOC_GEN_END -->

---

## Development

### Building

- this repo uses [cargo make](https://github.com/sagiegurari/cargo-make) as build helper
- the app can be crosscompiled for both Linux and Windows using `cargo make deploy`
    - cargo will crash if necessary toolchains are not installed
- a zip package can be created by running `cargo make package`

---
