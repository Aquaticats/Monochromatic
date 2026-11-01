# `systemd-run` with `StandardOutput=file:` keeps the old tail of a log when a rerun writes less (systemd 259.9)

## Symptom

A transient unit started with `systemd-run --user --property=StandardOutput=file:/path/run.log ...` writes its output
to `run.log`.
Started again with the same path, the second run overwrites the file from the first byte but never shortens it, so
when the second output is shorter, the tail of the first run stays in the file.
The log then shows a mix of two runs, for example a stale `FAILED` line below a fresh passing summary.
A subagent hit this twice, reading stale clippy and end-to-end results from a log that a shorter rerun had overwritten.

## Root cause

`file:path` opens the target without truncating it; `truncate:path` is the mode that truncates.
The manual says so (`man systemd.exec`, `StandardOutput=`, systemd 259.9-1.fc44):

```text
truncate:path is similar to file:path above, but it truncates the file when opening it.
```

The same page lists `append:path` as the append mode.

## Verification

The probe starts two transient units in a row against one path, the first with a long line and the second with a short
one, for each mode (`systemd-run --user --wait --collect --property=StandardOutput=<mode>:<log> echo <text>`):

```text
file:     "second\nun with a long line of output\n"
truncate: "second\n"
```

With `file:`, the first run's `first r` is overwritten by `second\n` and the rest of `un with a long line of output\n`
remains.
With `truncate:`, only the second run's output remains.

## Verified workarounds

- Use `StandardOutput=truncate:<log>` (and `StandardError=truncate:<log>` if the error stream goes to a file) for
  every unit whose log is read as the result of the latest run.
  Tradeoff: the manual notes that `truncate:` truncates on each command line of a unit, so it suits a unit with a
  single `ExecStart=` and no `ExecStartPost=` or `ExecStop=`, which a plain `systemd-run` command is.
- Or give each run its own log path (a timestamp or a run counter in the name).
  Tradeoff: old logs accumulate and need cleaning.

## What does not work

- `file:` with the expectation that a rerun replaces the log: it overwrites in place only.
- Reading the log's modification time as proof of freshness: a rerun updates it even when the content is stale below
  the new output.

## Upstream filing decision

Nothing to file: the behavior is documented in the manual page cited above, and `truncate:` is the documented remedy.
