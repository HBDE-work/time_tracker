
# Tracker - Track Worktime Today

- I was so annoyed by the time tracking tool of my company that I built my own.
- It just tracks a single time with pauses per day, nothing more nothing less.
- Data is stored as JSON per day under `~/.config/time_tracking/` (Linux) or `%APPDATA%/time_tracking/` (Windows).
- Run `tracker go` when you start, `tracker pause` for breaks, `tracker stop` when you're done.
- Check your hours with `tracker status` or look up past days like `tracker status monday 23`.

## Installation

### Install directly from crates-io

`cargo install time-tracker-cli`

This installs the binary as `tracker` in your Cargo bin directory.

## TUI Mode

- Just run `tracker` without any subcommand and the TUI launches.
- Upper panel shows the available commands: `[g]` Go, `[p]` Pause, `[s]` Stop.
- Lower panel shows today's status and auto-refreshes every minute.
- Press `q` or `Esc` to quit.

![TUI Mode](https://github.com/HBDE-work/time_tracker/blob/master/docs/images/tui_mode.png?raw=true)

## CLI Reference


<!-- CLAP_DOC_GEN_START -->
`tracker [OPTIONS] <COMMAND>`

### Commands

#### `go`:                  Start or resume tracking
- args:
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
