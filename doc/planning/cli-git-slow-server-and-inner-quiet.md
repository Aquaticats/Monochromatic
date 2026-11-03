# Git wrapper: no `--quiet` on its own runs, and a push that waits for a slow server

Status: design for two owner decisions answered through the question tool on 2026-11-03. Branch
`fix/git-policy-cli-no-inner-quiet-and-slow-server-waits`. Package `package/git-policy/cli`.

## Owner requirements this design follows

- Earlier rule, 2026-11-03T04:50:41-05:00: "No the wrapper should never append --quiet. Letting native git emit whatever
  it wants to is expected." It shipped for the person's own commands in 0.5.24.
- Inner `--quiet`, answered "Drop it everywhere (Recommended)": no Git process the wrapper or the service starts gets
  `--quiet` or `-q` (maintenance, the auto-push fetch, fast-forward merges, reset, clean, init).
- Coordinator correction: the rule is about output. A `--quiet` that changes Git's exit status or what is computed
  stays; only a `--quiet` whose documented effect is hiding messages or progress goes.
- Slow server check, answered "Wait while connected (Recommended)": count bytes on the push's network socket as
  progress (TCP counters); once the pack is sent, keep waiting while the connection to the remote is open, with a line
  naming the wait. The owner accepted the con: a connection that dies without a reset is noticed only when TCP gives up.

## Decision 1: every use of `--quiet` and `-q`, classified

The search was `rg --line-number '"--quiet"|"-q"' package/git-policy/cli/src/native`, uncapped, then a second search
over byte strings, `String::from`, `.into()` and `OsString::from` spellings. Each hit is read below against the
installed Git 2.56.0 documentation (`/usr/share/doc/git`).

### Dropped: the documented effect is hiding messages or progress

Each is read in the Git 2.56.0 source (tag `v2.56.0`), not only the manual: the option gates only printing.

- `maintenance run --auto --quiet` (`maintenance_decision.rs`): "Do not report progress or other information over
  `stderr`" (`git-maintenance.adoc`).
- `fetch --quiet` of the auto-push (`auto_push_reconcile.rs`): "silence any other internally used git commands.
  Progress is not reported to the standard error stream" (`fetch-options.adoc`).
- `merge --ff-only --quiet` of the auto-push (`auto_push_reconcile.rs`): "Operate quietly. Implies `--no-progress`"
  (`merge-options.adoc`).
- `merge --ff-only --quiet --no-edit` of the isolated merge (`merge_isolation.rs`): same entry. See the open question.
- `reset -q` of the landing's index update, three spellings (`service_land_follow.rs`): "Be quiet, only report errors"
  (`git-reset.adoc`).
- `clean --force -d --quiet` of the worktree restore (`worktree_dirty_restore.rs`): "Be quiet, only report errors, but
  not the files that are successfully removed" (`git-clean.adoc`).
- `init --bare --quiet` of the reftable shadow (`shadow_create.rs`): "Only print error and warning messages; all other
  output will be suppressed" (`git-init.adoc`).
- `checkout-index --force --quiet` of the merge recovery (`merge_isolation_recovery.rs`): "be quiet if files exist or
  are not in the index" (`git-checkout-index.adoc`); `builtin/checkout-index.c:121` gates only the message. Measured on 2026-11-03 against Git 2.56.0: a path that is not
  in the index exits 1 with and without it, so only the message differs.
- `fast-import --quiet` of the landing (`service_land.rs`, a file others are changing): "Disable the output shown by
  --stats, making fast-import usually be silent when it is successful" (`git-fast-import.adoc`); Git 2.56.0
  `builtin/fast-import.c:1670` gates only the report on `quiet`. The edit there is not one token: `checked(...)`
  discarded standard error, so the call became `run(...)`, the existing `classify` on failure, and one `record_words`.
- `rev-list --objects --quiet` (`service_land.rs`, the connectivity check of a commit Git built): the manual says it
  is meant for testing the exit status, but `builtin/rev-list.c` (`REV_LIST_QUIET`, in `show_object`) returns only
  after `finish_object` has made its missing-object check, so the status is the same without it and only the printing
  of object names goes. The captured standard output is a list of object IDs, not words, so it is not recorded.
- `check-ignore --quiet` (`worktree_trash.rs`, `worktree_copy_selection.rs`): Git 2.56.0 `builtin/check-ignore.c:83,118`
  gates only the output on `quiet`; "Don't output anything, just set exit
  status" (`git-check-ignore.adoc`). Measured: exit 0 for an ignored path and 1 for a path that is not, with and without
  the option; the only difference is the path on standard output, which both callers ignore.

### Kept: the option changes the exit status or what is computed

- `rev-parse --verify --quiet` (eight call sites): measured on 2026-11-03, an unknown name exits 128 with
  "fatal: Needed a single revision" without the option and 1 with it, and every caller reads that difference as
  "absent" against "broken". Git 2.56.0 `builtin/rev-parse.c:581-587` (`die_no_single_rev`): `exit(1)` when quiet, `die`
  otherwise.
- `symbolic-ref --quiet HEAD` (seven call sites): "Do not issue an error message if the <name> is not a symbolic ref
  but a detached HEAD; instead exit with non-zero status silently" (`git-symbolic-ref.adoc`); callers read exit 1 as
  detached and any other failure as an error (`builtin/symbolic-ref.c:28-31`: returns 1 when quiet, `die` otherwise).
- `diff-index --cached --quiet` and `diff-files --quiet` (three call sites): "Disable all output of the program.
  Implies `--exit-code`" (`diff-options.adoc`); the exit status is the whole answer, and `diff.c:5327-5332` also turns
  rename detection off, and `diff.c:7541` (`diff_can_quit_early`) lets the walk stop at the first change.

### Not the wrapper's own run: unchanged

- Tables and parsers of the person's own options: the `row(... "quiet" ...)` tables, `config_loading.rs`,
  `merge_options.rs`, `worktree_add_plan.rs`, `service_commit_args.rs`, `package_repair.rs`,
  `service_hold_retry.rs`, `command_conflict_side.rs`, `repo_read_query_refs.rs`.
- `worktree_ladder_finish.rs` adds `--quiet` to its `reset --hard` only when the person passed `--quiet` to
  `worktree add`; it forwards the person's option.
- Fixtures in `binary_support.rs`, `binary_fault_support.rs`, `replay_reverse_apply_test_support.rs` and the `*_tests.rs`
  files start real Git to build a test repository, not as a run of the wrapper.

### Where the words go

Dropping the option makes Git say what it says. Every run is captured (`run_git` pipes both streams into
`GitOutput`), so the words need a destination or they are discarded as before.

- Runs inside the service (maintenance, landing reset, fast-import, the auto-push fetch and merge): one entry in the
  service's log, which the service manager keeps in the journal of its unit (README, "The service writes to its
  standard error"), through `service_timing::log_entry` at priority `info`, with the userinfo of URLs masked as every
  entry is. This is how the service already records Git's words for a failed push (the `pushed ... outcome=failed` entry).
- Runs the wrapper makes while it carries out a person's command (the worktree restore's `clean`, the isolated merge's
  fast-forward, the merge recovery, a stacked push's fetch and merge): the words go to the wrapper's own entry in the
  journal (`client_journal`, `journalctl --identifier=cli-git`), through `inner_git_words.rs`, and not to the person's
  streams. The person's command prints what native Git's command prints: the isolated merge is compared with native
  Git's output byte for byte by `the_merge_equals_what_native_git_writes_on_a_clean_checkout`, and the `clean` of the
  restore says "Removing <path>" for untracked files of a stage that native `git worktree add` has no lines for
  (measured on 2026-11-03: three such lines on standard output, also under `--quiet`, before they were routed). What
  Git writes to standard error in the isolated merge's fast-forward (a warning) still reaches the person. No protocol
  channel is involved: hook standard output and RPC are never the destination of an inner run.
- The consequence for the journal: `git reset` without `-q` lists, as "Unstaged changes after reset:", every tracked path
  of the worktree whose file differs from the index (measured on 2026-11-03: `M a.txt` and `M b.txt` for two edited files
  after `git reset HEAD -- c.txt`; `builtin/reset.c:494` passes `REFRESH_IN_PORCELAIN` instead of `REFRESH_QUIET`). The
  landing's index update runs it for every commit, so the service's journal now names the paths of other uncommitted work.
  This follows from "Drop it everywhere" and is reported to the coordinator for the owner.
- No other output-hiding option replaces `--quiet`: no `--no-progress`, no redirect to `/dev/null`.

### Detached maintenance: what Git does with the output

`git maintenance run --detach` calls `daemonize()` (`setup.c`, Git 2.56.0), which forks, `setsid()`s, and does
`close(0); close(1); close(2); sanitize_stdfds();`, so the detached part of the run writes to `/dev/null` whatever the
option. `builtin/gc.c` calls `daemonize()` only after "Work that will be executed before detaching". So dropping the
option shows nothing more from the detached part; the words that can appear are those of the foreground tasks before
the fork, of the `--no-detach` form, and of the pre-2.47 retry. Cited from the tag `v2.56.0` of the Git source.

### Open question reported to the coordinator

The isolated merge builds the merge commit itself, then fast-forwards the checkout to it and prints the wrapper's own
"Merge made by the 'ort' strategy." and diffstat. Without `--quiet`, Git's fast-forward also prints
"Updating a..b", "Fast-forward" and its own diffstat on standard output. Implemented: those words go to the journal and
the person sees native Git's output. The alternative, the words on the person's terminal, breaks the byte-for-byte
comparison with native Git's merge and shows a fast-forward the person did not ask for; it is the coordinator's to
confirm or reverse.

## Decision 2: a push waiting for the server's checks

### What stops a push today

`push_progress.rs` samples the push's tree every 500 ms: `rchar` plus `wchar` of `/proc/<pid>/io`, processor time,
and tasks in state `R` or `D`. A tree showing none for the stall window (30 s by default) is stopped (`StopCause::Stalled`).
`rchar` and `wchar` count `read` and `write`, not `send` and `recv` (`push_progress.rs`, measured 102 bytes for 100000
bytes through `send` and `recv`), so a transfer done by `git-remote-http` through `libcurl` shows only through the pipe
that feeds it. After the pack is sent the tree is in state `S` waiting for the server, which is indistinguishable from a
dead peer, so a server check longer than the window ends in a stop and a retry.

### Sources for per-socket byte counters on Linux, and the choice

- `/proc/net/tcp` and `/proc/<pid>/net/tcp` list sockets with `tx_queue` and `rx_queue` (bytes waiting now) and the
  state, but no cumulative byte count (kernel `Documentation/networking/proc_net_tcp.rst`); a count that returns to
  zero says nothing about progress. The same document says these interfaces "are deprecated in favor of tcp_diag".
- `pidfd_getfd(2)` then `getsockopt(TCP_INFO)` needs ptrace permission over the target (Yama `ptrace_scope`, container
  seccomp); rejected, it fails on hosts the wrapper must also serve.
- `ss --info` spawns a program; the brief says to embed instead.
- Chosen: `NETLINK_SOCK_DIAG`, the interface `ss` itself reads (`man 7 sock_diag`). One dump request per address family
  with `idiag_states` set to the established state and `idiag_ext` asking for `INET_DIAG_INFO` returns, per socket, an
  `inet_diag_msg` (`idiag_state`, `idiag_inode`; `man 7 sock_diag`, "IPv4 and IPv6 sockets") followed by a `struct
  tcp_info` attribute holding `tcpi_bytes_acked` and `tcpi_bytes_received` (`include/uapi/linux/tcp.h`, the installed
  `/usr/include/linux/tcp.h`). The push tree's own sockets are the `socket:[<inode>]` links of `/proc/<pid>/fd`
  (`man 5 proc_pid_fd`: "socket:[2248868] will be a socket and its inode is 2248868"), matched to `idiag_inode`. It needs no privilege for one's own sockets, spawns nothing, and one reply
  carries state and counters together, so the connection test and the progress test read the same record. Cost:
  one `readdir` and a few `readlink` calls per tree member, and two netlink round trips, every 500 ms sample.
- Raw packing lives in one small module, `tcp_sockets.rs`, behind two functions; the rest of the code sees only
  "the established sockets of these processes and their byte counters".

### Definitions from observable state

- The connection is open: some process of the push's tree holds a TCP socket whose `idiag_state` is `TCP_ESTABLISHED`.
- The pack is sent: the connection is open, no process of the tree runs `git pack-objects`, and either a `git pack-objects`
  was seen at an earlier look, or `git send-pack` is below the push. Over HTTP `git-remote-http` starts
  `git send-pack --stateless-rpc` once the remote's refs are known and keeps it until the response is read, so its presence
  separates "the server never answered the first request" (a stall, as the existing control with a listener that never
  answers shows) from "the server is checking a sent pack". A push over `git://` or `ssh` runs `send-pack` inside `git push`,
  so only the first sign exists there. The subcommand is read from `/proc/<pid>/cmdline`, because Git runs its subcommands as
  `git <name>` and the process name says only `git`. While `pack-objects` runs, the push is still sending, and a stall there is
  stopped as before.
- Progress: any growth of `tcpi_bytes_acked` plus `tcpi_bytes_received` of a tree socket since the previous sample is a
  sign of work, like bytes moved. A server that sends Git's keepalive (`receive.keepAlive`, default 5 seconds, "set to
  0 to disable keepalives entirely", installed `git help config`) therefore keeps the push alive by progress alone.
  Whether GitHub sends keepalives is unknown.
- Scope: only `git push` waits (`Patience::Push`); the auto-push's fetch and the LFS upload keep the stall rule, because a
  silent open connection is a normal state for them and nothing in the owner's answer is about them.
- The wait: when the stall window runs out and the pack is sent, the clock does not stop the push. It says the wait once, in a
  line that starts with `cli-git: git push is waiting for the server`, in the diagnostic log as it begins and in the push's
  output, and it keeps checking. The silence of a wait does not count against the window, so when the connection is no longer
  open (no tree process holds an established socket) the stall window runs again from that moment. The ceiling (30 minutes
  by default) still bounds the push.
- Superseded in part (2026-11-03, after this decision was built): pick 5 of `cli-git-shipped-limits-audit.md`,
  "Push stall window, ceiling and Git's low-speed abort", option A, "Wait while connected" whole,
  made by the coordinator under the owner's delegation and open to the owner's veto,
  removes three things this decision kept.
  The pack-sent test is gone: an open connection is waited for in every phase, including before the first answer.
  The ceiling and its variable are gone.
  The scope is the push, the reconcile fetch and the LFS upload, and none of them gets a low-speed abort.
  What remains is the stall window for a tree with no open connection.
  `SPEC.md`, "Wait while connected", is the current text.

### What Git 2.56.0 does after the pack is sent

Read in the source (tag `v2.56.0`), not inferred:

- `send-pack.c` starts a sideband demultiplexer, runs `pack_objects` (the `git pack-objects` child), then blocks in
  `receive_status` reading the server's report; nothing in the client times that read.
- `builtin/receive-pack.c` `copy_to_sideband` writes `0005\1` (a band-1 packet with no payload) when it has sent nothing for
  `receive.keepAlive` seconds (default 5) once the hook phase begins (`use_keepalive = KEEPALIVE_ALWAYS` before
  `execute_commands`); with the setting at 0 it sends none.
- `http.c` always sets `CURLOPT_TCP_KEEPALIVE`, with the kernel's idle time unless `GIT_TCP_KEEPIDLE` is set, which is when TCP
  gives up on a dead peer (the owner's accepted con).
- `http.c` sets `CURLOPT_LOW_SPEED_LIMIT` and `CURLOPT_LOW_SPEED_TIME` from `http.lowSpeedLimit` and `http.lowSpeedTime` or
  `GIT_HTTP_LOW_SPEED_LIMIT` and `GIT_HTTP_LOW_SPEED_TIME`, and libcurl `lib/speedcheck.c` (`Curl_speedcheck`) fails the
  transfer with "Operation too slow" when the speed stays under the limit for the time. Five bytes every five seconds is under
  1000 bytes per second, so with the wrapper's defaults (`bounded_push.rs`, `apply_low_speed_defaults`) an HTTP push whose check
  is silent for more than 60 seconds was ended by Git itself, whatever the meter did (measured: "curl 28 Operation too slow.
  Less than 1000 bytes/sec transferred the last 60 seconds"). On the coordinator's instruction the wrapper's push no longer
  sets either variable; Git sets neither itself, a value the person set in the environment or in `http.lowSpeedLimit` and
  `http.lowSpeedTime` still applies, and the auto-push's fetch keeps the defaults. The controls are
  `a_push_over_http_waits_past_the_old_low_speed_cutoff_and_lands` (a hook silent for the old cutoff plus five seconds, derived
  from `LOW_SPEED_TIME`) and `a_low_speed_setting_the_person_made_still_ends_a_silent_http_wait`.

### Limits stated up front

- A stall before any pack began, with `send-pack` not yet started and `pack-objects` never seen, is a stall as before. A tiny
  pack made and gone between two samples (500 ms) over `git://` or `ssh` is not seen, so such a push waits only if its
  `send-pack` is visible; it is stopped as it is today.
- A connection that dies without a reset is noticed when TCP gives up (the owner's accepted con).
- A remote reached through a local path or `file://` has no socket: its hook is a child of the push, and the stall rule
  stays as it was. This is a control of the tests.
- Non-Linux systems measure nothing today and stay so (`UNTESTED ON WINDOWS`).
- IPv6 sockets are asked for in the same dump; no control binds to `::1`, so that family is unexercised.

### Fixtures

- Counters: a loopback `TcpListener` bound to port 0 and a client in the same test, a known number of bytes each way;
  the report must grow by exactly those bytes, matched by inode (`tcp_sockets_tests.rs`). The kernel counts the opening
  side's `SYN` as an acknowledged byte, so growth is compared, not totals.
- Clock: a probe process (this test binary run as a test of its own, killed with its test, dying with its parent through
  `PR_SET_PDEATHSIG`) connects to a test-owned listener with `std::net`, which sends and receives with `send` and `recv`, and
  holds the connection or trickles bytes; children named `git send-pack` and `git pack-objects` stand in for the push's
  processes (`push_wait_tests.rs`).
- Landing: `git daemon` is not installed here, so a loopback HTTP server written for the test runs Git's own `git http-backend`
  as a CGI for every request (`smart_http_support.rs`). The bare repository sets `http.receivepack` to true and
  `receive.keepAlive` to 0 so the server is silent, and its `pre-receive` hook sleeps four stall windows (the stall window is a
  whole second). Red before the change: stopped at the window with exit 124 and `bytes_moved=75699`. Green: lands on the first
  try, the remote holds the commit, and the output carries the wait line (`push_slow_server_tests.rs`).
- Controls: the same hook behind a local path is stopped as before; a listener that takes the connection and never answers is
  stopped as before; a probe with the connection closed, with no `send-pack`, with `pack-objects` alive, or with no connection
  at all, is stopped on the window.
- `test_support::spawn_owned` is on the unmerged branch `fix/git-policy-cli-test-helpers-die-and-prune-audit-read` and not in
  this checkout, so the tests use a small guard of their own (kill and wait on drop) until that branch lands.
