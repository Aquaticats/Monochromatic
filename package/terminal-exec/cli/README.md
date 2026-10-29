# terminal-exec

Cross-platform terminal emulator launcher.
On Linux/FreeBSD,
 implements the proposed [Default Terminal Execution Specification][term-spec]
with a KDE `kdeglobals` fallback.

[term-spec]: https://gitlab.freedesktop.org/terminal-wg/specifications/-/merge_requests/3
On Windows,
 detects Windows Terminal or falls back to `cmd.exe`.

## Motivation

The reference `xdg-terminal-exec` shell script ignores desktop environment settings.
On KDE Plasma,
 users configure their default terminal in System Settings,
which writes `TerminalService` to `$XDG_CONFIG_HOME/kdeglobals` or,
 with the
current fallback implementation,
 `$HOME/config/kdeglobals` when `XDG_CONFIG_HOME`
is unset.
The reference script does not read `kdeglobals`,
 falling back to scanning all `.desktop` entries
with `Categories=TerminalEmulator`:
 often selecting the wrong one.

On Windows,
 no equivalent of `xdg-terminal-exec` exists at all.

## Resolution order

### Linux / FreeBSD (XDG)

1. Explicit entries from `xdg-terminals.list` config files (desktop-specific variants checked first)
2. KDE `TerminalService` from `$XDG_CONFIG_HOME/kdeglobals`,
    falling back to `$HOME/config/kdeglobals`
   when `XDG_CONFIG_HOME` is unset (when no explicit entries exist)
3. All `TerminalEmulator`-category desktop entries as fallback

### Windows

1. Windows Terminal (`wt.exe`):
    default on Windows 11,
    widely installed on Windows 10
2. `cmd.exe`:
    always available

## Configured launch arguments

After desktop-entry tokenization,
the launcher preserves the configured `Exec` arguments in their original order.
It does not remove or rewrite `--gtk-single-instance=false`,
`--gtk-single-instance=true`,
or other terminal-specific switches.
The desktop entry owns that policy;
terminal-specific consequences of selecting an existing instance remain the terminal's responsibility.

## Usage

```sh
# Command syntax.
terminal-exec [options] [--] [command [args...]]
```

### Options

- `--app-id=VALUE`: set the terminal window app ID / class
- `--title=VALUE`: set the terminal window title
- `--dir=VALUE`: set the working directory
- `--hold`: keep the terminal open after the command finishes
- `--` or `-e`: delimiter between options and the command

### Examples

```sh
# Open default terminal
terminal-exec

# Open terminal running a command
terminal-exec bash -l

# Open terminal with title and working directory
terminal-exec --title="Build logs" --dir=/tmp -- tail -f build.log
```

## Configuration (Linux)

Create `~/.config/xdg-terminals.list` to explicitly set preferred terminals:

```text
com.mitchellh.ghostty.desktop
org.kde.konsole.desktop
```

First valid entry wins.
 Entries listed here bypass `OnlyShowIn`/`NotShowIn` checks.

### Excluding entries from fallback

Prefix an entry ID with `-` to prevent it from appearing in the fallback scan:

```text
-vscodium-pod-shell.desktop
```

## Desktop entry X-TerminalArg fields

Terminal `.desktop` files declare how to pass arguments via `X-TerminalArg*` keys:

- `X-TerminalArgExec`:
   how to pass a command (`-e`,
   `--`,
   `execute`)
- `X-TerminalArgTitle`:
   how to set window title (`--title=` or `--title`)
- `X-TerminalArgAppId`:
   how to set app ID (`--class=` or `--class`)
- `X-TerminalArgDir`:
   how to set working directory (`--working-directory=`)
- `X-TerminalArgHold`:
   how to keep terminal open (`--wait-after-command`)

A trailing `=` means the value is concatenated as one argument (`--title=My Title`).
No trailing `=` means two separate arguments (`--title` `My Title`).
