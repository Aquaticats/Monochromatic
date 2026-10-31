# file-enforcer command line

The `file-enforcer` binary over [`monochromatic-file-enforcer`](../core-rust/README.md).
It generates and checks the files this repository derives from other files,
as described in
[`doc/decision/file-enforcer-rust-rewrite.md`](../../../doc/decision/file-enforcer-rust-rewrite.md).

## Commands

```sh
file-enforcer write                         # regenerate every drifted file
file-enforcer check                         # report drift, exit 1 when anything differs
file-enforcer check --expected-dir /tmp/d   # also write /tmp/d/actual and /tmp/d/expected for diff tools
file-enforcer watch                         # keep files current, warn on hand edits
file-enforcer explain CLAUDE.md             # which rules read or write this path
file-enforcer user write                    # rules that write outside the repository
file-enforcer user check
```

- `--config <path>` names the repository config;
  without it the nearest `file-enforcer.config.jsonc` above the current
  directory is used,
  and its directory is the worktree root.
- `--user-config <path>` names the per-user config;
  the default is
  `<config dir>/monochromatic-dev/file-enforcer/user.config.jsonc`.
- Unknown flags are rejected.

## Output

Standard output is JSON lines only,
including clap's help,
version and usage errors (event types `help`,
`version`,
`error` with code `usage-error`).
Each line has `schemaVersion`,
`sequence` (per process),
`type`,
`code`,
`message`,
and optionally `hint`,
`path`
and `rule`.
Exit codes:
0 clean,
1 drift or rule failure,
2 bad command line or config.

## The server

Each command is a client of a server that starts on demand,
one per worktree (and one for the per-user config),
and exits
when idle.
`FILE_ENFORCER_IDLE_SECONDS` sets the idle time of servers a command starts (default 120);
containers and CI set it low for
deterministic shutdown.
The socket lives under `$XDG_RUNTIME_DIR/file-enforcer`,
or `<cache dir>/file-enforcer/run` when that variable is unset
or unusable.
The hidden `serve` command is what the first client runs,
detached.

## Tests

```sh
mise run //package/file-enforcer/cli:test
```

The tests run the built binary in scratch worktrees with their own runtime directories,
and a guard stops any server a
test leaves.
