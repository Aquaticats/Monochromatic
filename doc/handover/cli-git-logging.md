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

## What changed, with the test and the red plant of each

Every plant below was applied to the source, the named test run red, and the source restored (the working tree was clean
after each).

### Service journal transport (defect 1)

- `journald_native.rs` (new): `Sink::choose` uses the native socket only when `$JOURNAL_STREAM` names the device and inode
  of standard error (the check `systemd.exec(5)` asks for); the socket is unconnected and non-blocking, one `sendto` per
  entry, no lock; any entry the socket does not take goes to standard error, prefixed `<priority>` when standard error is
  the journal's stream.
  Entry: `PRIORITY`, `SYSLOG_IDENTIFIER=cli-git-service`, `MESSAGE` (the line, `at_ms` kept), `CLI_GIT_AT_MS`,
  `CLI_GIT_RELEASE`, `CLI_GIT_INSTANCE` and per-line fields.
  Every value is one line (control characters become spaces, cut at 16 KiB for the message and 1 KiB for a field), so the
  simple `NAME=value` form is enough; the binary form for values with newlines is not needed and not built.
- `service_timing::log_entry` is the one writer; `log` and `service_log!` keep their signatures (info), `service_warn!` and
  `service_error!` are new; the userinfo of every URL is masked in the message and in field values.
- The signal handler sends its line as a datagram with `sendto` on a descriptor and address made in `install()`, and writes
  to standard error only when that fails (so the exit is in the journal once).
- Tests: `journald_native::tests::*` (10 unit), `service_timing::tests::the_userinfo_of_a_url_never_reaches_an_entry`,
  `the_macros_write_at_the_priority_their_name_says`, `a_message_with_a_newline_is_one_line`,
  `service_signal::tests::the_native_entry_carries_the_message_the_priority_and_the_time`; binary
  `commit_timing::a_service_on_the_journal_stream_sends_native_entries_that_join_the_clients_record`,
  `a_service_stopped_by_a_signal_sends_its_exit_on_the_native_socket`,
  `a_journal_that_is_gone_leaves_the_lines_on_standard_error_with_their_priority`.
  They use abstract sockets bound by the test (a fixture path is longer than a socket path may be) and a stand-in for
  `$JOURNAL_STREAM` (the device and inode of the log file the service gets as standard error); the real journal socket, the
  user manager and the production service are never touched.
- Red plants: control characters kept in `one_line` (`a_newline_in_a_message_or_a_value_cannot_start_a_field` red);
  `&&` weakened to `||` in `stream_matches` (`the_stream_is_the_journal_only_when_device_and_inode_both_match` and
  `standard_error_that_is_not_the_journal_keeps_the_plain_line` red); `set_nonblocking` removed
  (`a_full_journal_queue_falls_back_instead_of_waiting` hangs and is killed at the bound, 230 s, no pass); masking removed
  from `log_entry` (`the_userinfo_of_a_url_never_reaches_an_entry` red); the handler's descriptor never stored
  (`a_service_stopped_by_a_signal_sends_its_exit_on_the_native_socket` red).
- Cost: native entry about 11 microseconds per entry including the test's own drain, debug build, one CPU (a plain write is
  0.5 microseconds); a commit makes two to four entries; no fsync.

### Join client and service (defect 2)

- The client timing line has `requestId` (the id the request carries, the `request=` of the `answered` line and the
  `CLI_GIT_REQUEST_ID` field), plus `time` (end) and `level`.
- Tests: `commit_timing::tests::the_line_carries_the_request_identity_that_joins_it_to_the_service`,
  `the_line_states_its_end_time_and_a_level`; the binary join test above compares the datagram's field with the timing
  record.
- Red plant: `timing.request = None` in `service_commit.rs` (binary join test red).

### Push trigger names its worktree (defect 3)

- Lines have `worktree`, `gitDir`, `instance`, `level`; the `command-failed` and "watch could not start" lines went through
  `record_decision` with no branch, now the failed command names the branch and its tip before the command (the watch holds
  both); the detail is masked.
  `record_decision` takes one `Decision` value (it had six parameters).
- Tests: `push_trigger_log::tests::*` (4), extended binary `a_failed_command_is_recorded_as_such` and
  `a_watch_that_could_not_start_is_recorded`.
- Red plants: branch dropped from the `command-failed` decision (binary test red); masking removed from the detail
  (`a_url_in_the_detail_is_stored_without_its_credentials` red).

### More to read (the "log more" half)

- The service now logs every push it runs (`pushed branch=.. oid=.. items=.. outcome=.. push_ms=..`, a failure at warning with
  Git's words, credentials masked), automatic maintenance that ran, a finished review, and each batch a restart settled.
  Before, a push that succeeded left nothing and a settled batch was computed and dropped.
- Tests: `service_work::tests::every_push_leaves_an_entry_that_names_the_branch_and_how_it_ended`,
  `upkeep_and_review_that_ran_leave_entries`, `service_server::tests::a_restart_says_each_batch_it_settled`, through a
  test-only capture of entries (`service_timing::capture`, not compiled into the release).
- Red plant: the push entry call removed (`every_push_leaves_an_entry..` red).

### The appender of the three JSONL files

- `discard_audit::append_line` (used by timing, push trigger and audit): opens read+append, moves the file aside only when the
  path still names the file it opened (`move_aside_if_still_named`), and starts its line on a fresh one when the file ends
  without a newline.
  The audit line gained `time`, `level`, `release`, `gitDir`.
- Tests: `a_torn_last_line_does_not_swallow_the_next_record`,
  `a_second_rotation_does_not_replace_the_history_with_the_new_file`, `concurrent_appenders_lose_no_line_and_tear_none`,
  audit line field controls.
- Red plants: inode check removed (rotation test red); torn-tail prefix disabled (torn test red).
- Cost: 12 microseconds per line before and after (3 rounds of 3000 appends, debug build: old 12.4 to 13.1, new 11.0 to 11.9).
- Limits, not closed: the check and the rename are two calls, so two writers can still both rename in a window of
  microseconds; the loss is then the older generation `.1`, not the live file.
  A writer that reads the last byte while another is half-way through a page-crossing write adds an empty line (seen in the
  concurrency test); readers skip empty lines.
  A full disk can still cut a line short; the next record then starts on a fresh line and only the torn line is lost.

## Decisions not to change

- `at_ms` stays on every line and in `MESSAGE`: it costs nothing, the stderr fallback needs it, and the coordinator's other
  agent's test and the README describe it.
- The JSONL event stream, `diagnostic_log.rs`, the install log, intent journals, the standards-review shadow log and the
  work notices are unchanged (see the inventory).
- The conflict copies under `cli-git/discarded` are unbounded on purpose: they are the only copy of a replaced file.
- `lock_*` fields and the index-lock holder fields are untouched; they ride in `MESSAGE` as before.

## Verification

- `native:check`, `native:clippy` (warnings denied) clean; whole library suite: 2869 of 2870 pass.
  The one failure, `child_watch::tests::a_command_past_the_bound_is_stopped_with_the_processes_it_started`, fails the same way
  alone inside the capped `systemd-run --scope` I ran everything in; `child_watch.rs` and its tests are not changed on this
  branch. I did not run it on a clean main, so "pre-existing" is not proven.
- Binary tests run on the host: `trigger` (18), `timing` (12), `audit`/`conflict_side` (14); the container gate was not run.
