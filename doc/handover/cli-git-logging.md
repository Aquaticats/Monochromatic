# Logging of the Git policy wrapper

Branch `logging-quality`, crate `package/git-policy/cli`.
This file is the inventory, the evidence and the decisions, written as the work goes so it survives a stop.

## Requirements (the owner's words, 2026-11-02 and earlier)

- "If we're logging badly, improve how we log."
- "If we're not logging enough, log more."
- "The JSONL event stream stays."
- "Use as few locks as possible."
- "commits vs native git should not take 5s longer."
- "Ignore Win and mac until I say so."

## Evidence

### The kernel stamps a datagram when it is sent, not when it is read

Probe: `/var/home/user/temp/agent/logging/probe/ts.py` binds an `AF_UNIX` datagram socket with `SO_TIMESTAMP`, sends one
datagram, sleeps one second, then reads it.
Result on this host (Linux 7.2.15): the stamp was 0.000 s after the send and 1.000 s before the read.
So a native-protocol datagram that journald reads late still carries the sender's time as `_SOURCE_REALTIME_TIMESTAMP`
(`journald-native.c:564` sets `SO_TIMESTAMP`, `journald-manager.c:1178` records it), while a line on the stream socket is
stamped by journald when it processes it (`journald-stream.c:309` passes no time).

### Documents read today

- `man systemd.exec` (systemd 262): `$JOURNAL_STREAM` holds the device and inode of the stream connected to the journal,
  and "The device and inode numbers of the file descriptors should be compared with the values set in the environment
  variable"; setting is not enough to check.
- https://systemd.io/JOURNAL_NATIVE_PROTOCOL/: simple form `KEY=value` and a newline; binary form (key, newline,
  little-endian unsigned 64-bit length, value, newline) for values holding a newline; keys may not be empty or hold `=` or
  a newline; keys starting with `_` are ignored when a client sends them; a client "may then opt to upgrade to the native
  Journal protocol" and "cease to use STDERR" when the stream matches; a datagram too large for the socket fails with
  `EMSGSIZE` and is retried through a sealed `memfd`.
  `journald-native.c` (copy in `/var/home/user/temp/agent/systemd-src/`) calls `journal_field_valid(..., allow_protected=false)`.

## Inventory of sinks

### Service journal lines (writer: `cli-git-service`)

- Writers: `service_timing::log` (the `started`, `answered`, retire and `exiting` lines), `service_log!` at every
  failure and notice site (`service_server.rs`, `service_work.rs`, `service_main.rs`, `service_index_*.rs`,
  `service_claim_renewal.rs`, `work_item*.rs`), and the signal handler `service_signal::on_signal`.
- Transport before this work: stderr, which the unit connects to a journald stream socket.
  Journald stamps a stream line when it processes it, so under load the times are wrong and out of order with systemd's.
  The merged commit `462b07f637` (index-lock agent) put `at_ms=<epoch ms>` at the end of every line, which corrects the
  text but not the journal's own timestamp or its ordering.
- Fields: free text with `key=value` words; no priority (every line is `info` to journald, failures included);
  the line repeats the prefix `cli-git-service:` after the syslog identifier.
- Bounding: the journal's own limits.
- Interleaving: one `write` per line; a stream write of a line shorter than the pipe buffer is not torn.
- Verdict: wrong time (fixed by transport), no priority, no structured fields to join on, no URL masking, and silence
  for pushes, upkeep and reviews that succeeded.

### Client commit timing `<common git dir>/cli-git-commit-timing.jsonl` (writer: wrapper, `commit_timing.rs`)

- Written once when a wrapped commit ends, one `write` on an `O_APPEND` descriptor (`discard_audit::append_line`).
- Fields: `schemaVersion`, `startMillis`, `pid`, `release`, `instance`, `gitDir`, `branch`, `outcome`, `exitCode`, phases.
- Bounding: moves to `.1` at one mebibyte, replacing the older copy.
- Verdict: no request identity, so it joins the service's `answered` line only by time; no end time; no level.

### Push trigger `<common git dir>/cli-git-push-trigger.jsonl` (writer: wrapper, `push_trigger_log.rs`)

- One line per decision after a command that adds commits; the file is shared by every worktree.
- Fields: `schemaVersion`, `time`, `pid`, `release`, `command`, `outcome`, `reason`, `detail`, optional `branch`.
- Verdict: names no worktree; the `command-failed` and watch-could-not-start lines name no branch even though the
  caller holds one; `detail` is not masked for URL userinfo.

### Discard audit `<common git dir>/cli-git-audit.jsonl` (writer: wrapper, `discard_audit.rs`)

- One line per decision about a guarded form; same appender and rotation.
- Fields: `schemaVersion`, `timeMillis` (a quoted string), `pid`, `parentPid`, `worktree` (a kind word, not a path),
  `form`, `outcome`, `dirty`, `cwd`, `arguments`, optional `copies`.
- Verdict: no ISO time, no release, no Git directory; arguments are not masked for URL userinfo.

### Saved conflict copies `<common git dir>/cli-git/discarded/<UTC time>/` (writer: wrapper, `conflict_sides.rs`)

- File copies, not log lines; each is named by its audit line (`copies`).
- Unbounded on purpose: they are the only remaining content of a file the command replaced.
- Verdict: unchanged; pruning user data is the owner's decision (see "Left for the owner").

### Standards review shadow log `<common git dir>/cli-git/standards-review-shadow.md` (writer: service, `standards_review_append.rs`)

- Markdown entries, one whole `O_APPEND` write each, private mode, no lock; a review record, not a diagnostic log.
- Verdict: unchanged.

### Work notices for the person (`push_failure_notice.rs`)

- Files in the Git directory that the next wrapper command shows once; credentials are masked in them.
- Verdict: unchanged.

### Worktree copy install log, journals (`worktree_copy_install_log.rs`, `service_journal.rs`, `subvolume_journal.rs`, `merge_isolation_journal.rs`)

- Transactional records that recovery reads, not logs.
- Verdict: unchanged.

### JSONL event stream on stderr or stdout (`diagnostics.rs`, `policy_events.rs`)

- Owner: stays as it is.

### Debug and warning lines on stderr (`diagnostic_log.rs`)

- `[level] [ISO time] [cli-git] [tag] message`, one line per call, control characters replaced, off unless
  `MONOCHROMATIC_VERBOSE=true` (debug) or `MONOCHROMATIC_WARN=false` silences warnings.
- Verdict: unchanged; it already has a level and a time.

### Notices on the wrapper's stderr for the person

- `panic_notice.rs`, `startup_gate.rs`, `entry.rs` and the others print user-facing sentences, not log lines.
- Verdict: unchanged.

## Decisions

Filled in as each fix lands, below.
