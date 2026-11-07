# Logging of the Git policy wrapper

Note 2026-11-04T22:23:58-05:00: the program `cli-git-native` is renamed `monochromatic-git-policy-cli`,
the command word `git cli-git` becomes `git monochromatic-git-policy-cli`,
the settings file `cli-git.config.jsonc` becomes `monochromatic-git-policy-cli.config.jsonc`,
and the folders, logs and service units that carry `cli-git` carry `monochromatic-git-policy-cli`
(the list is in `doc/planning/git-policy-cli-program-rename.md`).
The new names arrive with the first release after this date;
this repository switches its installed program and its settings file when it installs that release.
This record keeps the names of its time.

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
(`journald-native.c:564` sets `SO_TIMESTAMP`, `journald-manager.c:1178` records it), while a line on the stream socket
is
stamped by journald when it processes it (`journald-stream.c:309` passes no time).

### Documents read today

- `man systemd.exec` (systemd 262): `$JOURNAL_STREAM` holds the device and inode of the stream connected to the journal,
  and "The device and inode numbers of the file descriptors should be compared with the values set in the environment
  variable"; setting is not enough to check.
- https://systemd.io/JOURNAL_NATIVE_PROTOCOL/: simple form `KEY=value` and a newline; binary form (key, newline,
  little-endian unsigned 64-bit length, value, newline) for values holding a newline; keys may not be empty or hold
  `=` or
  a newline; keys starting with `_` are ignored when a client sends them; a client "may then opt to upgrade to the
  native
  Journal protocol" and "cease to use STDERR" when the stream matches; a datagram too large for the socket fails with
  `EMSGSIZE` and is retried through a sealed `memfd`.
  `journald-native.c` (copy in `/var/home/user/temp/agent/systemd-src/`) calls
  `journal_field_valid(..., allow_protected=false)`.

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
- Fields: `schemaVersion`, `startMillis`, `pid`, `release`, `instance`, `gitDir`, `branch`, `outcome`, `exitCode`,
  phases.
- Bounding: moves to `.1` at one mebibyte, replacing the older copy.
- Verdict: no request identity, so it joins the service's `answered` line only by time; no end time; no level.

### Push trigger `<common git dir>/cli-git-push-trigger.jsonl` (writer: wrapper, `push_trigger_log.rs`)

- One line per decision after a command that adds commits; the file is shared by every worktree.
- Fields: `schemaVersion`, `time`, `pid`, `release`, `command`, `outcome`, `reason`, `detail`, optional `branch`.
- Verdict: names no worktree; the `command-failed` and watch-could-not-start lines name no branch even though the
  caller holds one; `detail` is not masked for URL userinfo.

### Discard audit `<common git dir>/cli-git-audit.jsonl` (writer: wrapper, `discard_audit.rs`)

- One line per decision about a guarded form; same appender, but each rotation keeps its file under a new number
  (`.1`, `.2`, and on) and generations that name no saved copy on disk are kept up to 4 MiB
  (`doc/planning/cli-git-shipped-limits-audit.md`, "Audit log rotation").
- Fields: `schemaVersion`, `timeMillis` (a quoted string), `pid`, `parentPid`, `worktree` (a kind word, not a path),
  `form`, `outcome`, `dirty`, `cwd`, `arguments`, optional `copies`.
- Verdict: no ISO time, no release, no Git directory; arguments are not masked for URL userinfo.

### Saved conflict copies `<common git dir>/cli-git/discarded/<UTC time>/` (writer: wrapper, `conflict_sides.rs`)

- File copies, not log lines; each is named by its audit line (`copies`).
- Unbounded on purpose: they are the only remaining content of a file the command replaced.
- Verdict: unchanged; pruning user data is the owner's decision (see "Left for the owner").

### Standards review shadow log `<common git dir>/cli-git/standards-review-shadow.md` (writer: service,
`standards_review_append.rs`)

- Markdown entries, one whole `O_APPEND` write each, private mode, no lock; a review record, not a diagnostic log.
- Verdict: unchanged.

### Work notices for the person (`push_failure_notice.rs`)

- Files in the Git directory that the next wrapper command shows once; credentials are masked in them.
- Verdict: unchanged.

### Worktree copy install log, journals (`worktree_copy_install_log.rs`, `service_journal.rs`, `subvolume_journal.rs`,
`merge_isolation_journal.rs`)

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

- `journald_native.rs` (new): `Sink::choose` uses the native socket only when `$JOURNAL_STREAM` names the device and
  inode
  of standard error (the check `systemd.exec(5)` asks for); the socket is unconnected and non-blocking, one `sendto` per
  entry, no lock; any entry the socket does not take goes to standard error, prefixed `<priority>` when standard error
  is
  the journal's stream.
  Entry: `PRIORITY`, `SYSLOG_IDENTIFIER=cli-git-service`, `MESSAGE` (the line, `at_ms` kept), `CLI_GIT_AT_MS`,
  `CLI_GIT_RELEASE`, `CLI_GIT_INSTANCE` and per-line fields.
  Every value is one line (control characters become spaces, cut at 16 KiB for the message and 1 KiB for a field), so
  the
  simple `NAME=value` form is enough; the binary form for values with newlines is not needed and not built.
- `service_timing::log_entry` is the one writer; `log` and `service_log!` keep their signatures (info),
  `service_warn!` and
  `service_error!` are new; the userinfo of every URL is masked in the message and in field values.
- The signal handler sends its line as a datagram with `sendto` on a descriptor and address made in `install()`, and
  writes
  to standard error only when that fails (so the exit is in the journal once).
- Tests: `journald_native::tests::*` (10 unit), `service_timing::tests::the_userinfo_of_a_url_never_reaches_an_entry`,
  `the_macros_write_at_the_priority_their_name_says`, `a_message_with_a_newline_is_one_line`,
  `service_signal::tests::the_native_entry_carries_the_message_the_priority_and_the_time`; binary
  `commit_timing::a_service_on_the_journal_stream_sends_native_entries_that_join_the_clients_record`,
  `a_service_stopped_by_a_signal_sends_its_exit_on_the_native_socket`,
  `a_journal_that_is_gone_leaves_the_lines_on_standard_error_with_their_priority`.
  They use abstract sockets bound by the test (a fixture path is longer than a socket path may be) and a stand-in for
  `$JOURNAL_STREAM` (the device and inode of the log file the service gets as standard error); the real journal
  socket, the
  user manager and the production service are never touched.
- Red plants: control characters kept in `one_line` (`a_newline_in_a_message_or_a_value_cannot_start_a_field` red);
  `&&` weakened to `||` in `stream_matches` (`the_stream_is_the_journal_only_when_device_and_inode_both_match` and
  `standard_error_that_is_not_the_journal_keeps_the_plain_line` red); `set_nonblocking` removed
  (`a_full_journal_queue_falls_back_instead_of_waiting` hangs and is killed at the bound, 230 s, no pass); masking
  removed
  from `log_entry` (`the_userinfo_of_a_url_never_reaches_an_entry` red); the handler's descriptor never stored
  (`a_service_stopped_by_a_signal_sends_its_exit_on_the_native_socket` red).
- Cost: native entry about 11 microseconds per entry including the test's own drain, debug build, one CPU (a plain
  write is
  0.5 microseconds); a commit makes two to four entries; no fsync.

### Join client and service (defect 2)

- The client timing line has `requestId` (the id the request carries, the `request=` of the `answered` line and the
  `CLI_GIT_REQUEST_ID` field), plus `time` (end) and `level`.
- Tests: `commit_timing::tests::the_line_carries_the_request_identity_that_joins_it_to_the_service`,
  `the_line_states_its_end_time_and_a_level`; the binary join test above compares the datagram's field with the timing
  record.
- Red plant: `timing.request = None` in `service_commit.rs` (binary join test red).

### Push trigger names its worktree (defect 3)

- Lines have `worktree`, `gitDir`, `instance`, `level`; the `command-failed` and "watch could not start" lines went
  through
  `record_decision` with no branch, now the failed command names the branch and its tip before the command (the watch
  holds
  both); the detail is masked.
  `record_decision` takes one `Decision` value (it had six parameters).
- Tests: `push_trigger_log::tests::*` (4), extended binary `a_failed_command_is_recorded_as_such` and
  `a_watch_that_could_not_start_is_recorded`.
- Red plants: branch dropped from the `command-failed` decision (binary test red); masking removed from the detail
  (`a_url_in_the_detail_is_stored_without_its_credentials` red).

### More to read (the "log more" half)

- The service now logs every push it runs (`pushed branch=.. oid=.. items=.. outcome=.. push_ms=..`, a failure at
  warning with
  Git's words, credentials masked), automatic maintenance that ran, a finished review, and each batch a restart settled.
  Before, a push that succeeded left nothing and a settled batch was computed and dropped.
- Tests: `service_work::tests::every_push_leaves_an_entry_that_names_the_branch_and_how_it_ended`,
  `upkeep_and_review_that_ran_leave_entries`, `service_server::tests::a_restart_says_each_batch_it_settled`, through a
  test-only capture of entries (`service_timing::capture`, not compiled into the release).
- Red plant: the push entry call removed (`every_push_leaves_an_entry..` red).

### The appender of the three JSONL files

- `discard_audit::append_line` (used by timing, push trigger and audit): opens read+append, moves the file aside only
  when the
  path still names the file it opened (`move_aside_if_still_named`), and starts its line on a fresh one when the file
  ends
  without a newline.
  The audit line gained `time`, `level`, `release`, `gitDir`.
- Tests: `a_torn_last_line_does_not_swallow_the_next_record`,
  `a_second_rotation_does_not_replace_the_history_with_the_new_file`, `concurrent_appenders_lose_no_line_and_tear_none`,
  audit line field controls.
- Red plants: inode check removed (rotation test red); torn-tail prefix disabled (torn test red).
- Cost: 12 microseconds per line before and after (3 rounds of 3000 appends, debug build: old 12.4 to 13.1, new 11.0
  to 11.9).
- Limits, not closed: the check and the rename are two calls, so two writers can still both rename in a window of
  microseconds; the loss is then the older generation `.1`, not the live file.
  A writer that reads the last byte while another is half-way through a page-crossing write adds an empty line (seen
  in the
  concurrency test); readers skip empty lines.
  A full disk can still cut a line short; the next record then starts on a fresh line and only the torn line is lost.

## Decisions not to change

- `at_ms` stays on every line and in `MESSAGE`: it costs nothing, the stderr fallback needs it, and the coordinator's
  other
  agent's test and the README describe it.
- The JSONL event stream, `diagnostic_log.rs`, the install log, intent journals, the standards-review shadow log and the
  work notices are unchanged (see the inventory).
- The conflict copies under `cli-git/discarded` are unbounded on purpose: they are the only copy of a replaced file.
- `lock_*` fields and the index-lock holder fields are untouched; they ride in `MESSAGE` as before.

## Real journal check (defect 1)

A one-off transient user unit ran the real debug `cli-git-service` (capped: `MemoryMax=512M`, `TasksMax=64`,
`CPUQuota=100%`, `IOWeight=30`, instance `probeinstance`, own socket and journal directory under
`/var/home/user/temp/agent/logging/`), once to its idle exit and once stopped with `systemctl --user stop`.
`journalctl --user --unit=cli-git-logprobe --output=json` gave, for the service's own entries:

- `_TRANSPORT=journal` (the native socket, not `stdout`), `SYSLOG_IDENTIFIER=cli-git-service`, `PRIORITY=6`,
  `_SOURCE_REALTIME_TIMESTAMP` present (a stream entry has none), and `CLI_GIT_RELEASE=0.5.13`,
  `CLI_GIT_INSTANCE=probeinstance`, `CLI_GIT_AT_MS` kept, so the journal accepts the field names.
- One entry per line: no duplicate from standard error.
- On the stop: the service's `exiting reason=signal-15 uptime_ms=992` is dated `_SOURCE_REALTIME_TIMESTAMP`
  1793652238213682, and systemd's `Stopping cli-git-logprobe.service` 1793652238213730,
  48 microseconds later, in that order.
  On the 0.5.13 upgrade the same pair had read the same millisecond for two processes six seconds after systemd's lines.
- The unit is gone afterwards (`systemctl --user list-units --all 'cli-git-logprobe*'` is empty).

`SyslogLevelPrefix=` (`man systemd.exec`): lines written to standard error that are prefixed with a log level
are processed with that level and the prefix removed, "Defaults to true".
The `<n>` prefix of the fallback therefore sets the priority.

## Known limits

- The three lines written before `set_context` runs
  (`--journal <directory> is required`,
  `cannot create the journal directory`,
  `cannot listen`)
  carry no `CLI_GIT_RELEASE` or `CLI_GIT_INSTANCE`;
  the signal handler's entry carries neither either (it builds its bytes without allocating).
- Standard-error fallback lines have no fields, only the text and `at_ms`.
- Two writers crossing the size limit at the same microsecond can still both move the file aside; the older `.1` is
  lost.
- Not changed in the retagged files of the index-lock agent: only the macro name (`service_log!` to `service_error!`) at
  `service_index_lock.rs:104,240`, `service_index_hold.rs:79,149,157,182`, `service_index_placement.rs:213,261`, and
  `service_land.rs:234` now calls `log_entry` with a priority and the field `CLI_GIT_BATCH`; the message text is the
  same.
  New `lock_*` fields in a line keep working: they are part of `MESSAGE`.

## Decided by the owner (2026-11-02)

### Pruning of `cli-git/discarded/`: "Prune by both age and size."

Implemented in `discarded_prune.rs`, called from `conflict_side_guard.rs` after a save is complete and synced.
The coordinator's defaults; the settings are keys of `cli-git.config.jsonc`:

- Age: saves older than 30 days go. `discardedCopies.maxAgeDays` (whole days, at least 1).
- Size: while the folder is over 1 GiB the oldest saves go first. `discardedCopies.maxSize` (bytes, or text such as `"256m"`
  with `k`, `m`, `g` as Git's integer settings).
- Where the settings live: the coordinator corrected the first brief, which had them in Git config. They are a new section
  of `cli-git.config.jsonc` (`config_discarded.rs`, `CliGitConfig.discarded`), beside `policies`, `hooks`, `indexLock`
  and `landing`, with the file's validation; there is no Git config read left.
- Floor: nothing younger than 24 hours is removed by either limit, so a burst is never cut by size, and the save just
  written is never chosen (a save made with an old clock survives its own pass).
- A value that is no limit (zero, negative, a fraction, text that is not a size) is a configuration failure that names the
  key (exit 2 before any policy runs), as every bad value of the file is, and nothing is removed. This replaces the first
  design, which named the value on standard error and kept the default: the coordinator's correction asked for the file's
  own validation, and the failure is what that validation does. If the owner prefers the soft handling, it is a change
  in `config_discarded.rs` only. There is no "off": a large number does the same.
- The age of a save is the time in its directory name, so a touched or edited copy does not look young; a directory
  whose name is not a save time is never touched.
- Moment: after the new copy is durable, so a kill while copying costs no older copy; a `checkout --theirs`
  family command is not a commit, and the pass reads the small folder once.
  (Review fixes: the folder is opened once without following a link
  and removal goes through descriptors, `discarded_store.rs`;
  a non-blocking exclusive lock admits one pass;
  names are real dates;
  the 24-hour floor and the age limit count from the end of the named second;
  sums are 128-bit;
  what a pass cannot do is kept and said in the notice and the doctor.)
- Audit: one line per removed save directory, after the directory is gone (`form` `PruneDiscarded`, `outcome` `pruned`,
  `reason`, `limit`, `folderBytes`, `path`, `bytes`, `copiesTotal`, `copies` up to a hundred names), through the shared
  appender.
- Doctor: `git cli-git doctor` writes size, copy count, save count, oldest save and the limits in force (the file's
  `discardedCopies`, else the defaults) to standard error when the folder holds any save.
- Limits of the design: the pass is the only remover; a kill between two removals leaves a line for each directory that
  went and a whole directory for each that did not.
- Tests: unit `discarded_prune::tests::*` (the plan, the real-folder pass with audit lines, a writer saving 40 times while a
  pass runs with the tightest limits and the old directory removed as the positive control, the doctor report),
  `config_discarded::tests::*` (defaults, accepted spellings, every value that is no limit, unknown key); binary in
  `conflict_side` (age cut with audit line, the file's limits with the floor, a longer age keeps, a zero limit is a
  configuration failure that removes nothing, doctor report and the limits it names).
- Red plants (each restored): floor removed; keep guard removed; age cut removed; size order reversed; audit write
  removed; foreign names accepted; age key ignored; size key ignored; zero age accepted; zero size accepted; section not
  applied; limits not handed to the checks; doctor ignores the file; `prune_after_save` call removed.
- Not measured: a folder of many thousand saves takes a walk per pass; the pass runs only on the rare conflict-side
  command.

### `at_ms` in the message or only in a field

`at_ms` is in the text and in `CLI_GIT_AT_MS`. If the owner wants the text without it on the native path (the journal
already dates the entry), the stderr fallback still needs it; removing it from the native `MESSAGE` only is one line in
`log_entry`. Ranking: keep both (now) > field only on the native path, because the text stays readable in a plain
`journalctl` and in a copied log; field only saves about thirty characters a line.

### A dependency for the native protocol

No crate was added. The `sd-journal` style crates would add sealed `memfd` support for entries over the datagram limit;
the entries here are cut at 16 KiB, below it, so the gain is nothing today. Ranking: no crate (now) > a journal crate.

## Cross-vendor review of this branch (openai/gpt-6.1-sol, read the files and the diff)

Output: `/var/home/user/temp/agent/logging/review/sol-review.txt`. Findings and what was done:

- P1 rotation check and rename not atomic (`discard_audit.rs`): accepted, not fixed. The reviewer's fix is a
  cross-process
  lock, which the owner ruled out; the loss is the older `.1` only, documented.
- P1 stderr fallback blocks the service (`journald_native.rs`, `service_signal.rs`): fixed. On the journal's stream the
  fallback uses `send(MSG_DONTWAIT)`; a line that finds no room is dropped whole, a line the socket took in part is
  finished within three waits of at most 100 ms. Test `a_full_journal_stream_drops_the_line_instead_of_waiting` (hangs
  without `MSG_DONTWAIT`).
- P1 append needs read permission: fixed. An unreadable file is opened for appending alone and the tail check is
  skipped.
  Test `a_file_that_cannot_be_read_is_still_appended_to` (red with the fallback removed).
- P1 redaction after truncation: fixed, the text is masked, then cut. Test
  `a_cut_just_before_the_at_sign_leaves_no_password` (red with the old order).
- P2 torn-line check races and `write_all` is not one write: the write is now one `write` that errors on a short count
  (`write_once`); the check race is kept (the reviewer's alternative, a leading newline on every record, doubles the
  lines
  and makes `tail --lines=5` show two or three records), documented as a limit.
- P2 signal fallback loses the priority: fixed, the handler's fallback carries `<6>` on the journal's stream. Test
  `the_line_on_the_journal_stream_carries_the_info_priority`.
- P2 longest abstract name rejected: fixed. Test
  `the_longest_abstract_name_fits_the_raw_address_and_one_more_byte_does_not`
  (red with `>=` back).
- P3 datagram-size claim: reworded as best effort; an entry the socket refuses goes to standard error.

## Verification

- `native:check` and `native:clippy` (warnings denied) clean after the last merge of main.
  Whole library suite after the merge: 2869 of 2870 pass.
  The one failure, `child_watch::tests::a_command_past_the_bound_is_stopped_with_the_processes_it_started`, fails the
  same
  way on a clean checkout of `main` (a throwaway worktree, same capped scope), so it is not from this branch.
- Binary tests on the host: `timing`, `trigger`, `audit` and `conflict_side`; the container gate was not run.
