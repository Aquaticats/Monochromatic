# The commit service without a service manager

Status, 2026-11-13 (fourth run, answering the review of `53726ade2d`): `main` is merged in,
the review's one finding is fixed with three controls and its reproductions no longer
reproduce, and the advice a person reads no longer names a unit that this release does not
have. The deadline inventory the review asked for is below.

The task: the wrapper's commit service stops being a unit of the user's
service manager and becomes a server the wrapper starts, finds, replaces and
stops itself, the way Bazel's client does with its own server.
The decision behind it (2026-11-12, `doc/handover/cli-git-rust-implementation.md`,
"Current state", the section about removing systemd as a hard dependency):
"Remove systemd as a hard dependency. Bazel doesn't need it."
Its precedent is named there: Bazel's client needs no service manager for
anything a build depends on.

## Part 1: what the wrapper takes from systemd today

Every place below was found by searching the package
`package/git-policy/cli` and its `mise.toml`
for `systemd`, `systemctl`, `systemd-run`, `sd_notify`, `loginctl`, `cgroup`,
`D-Bus`, `dbus`, `journal`, `unit`, `socket unit`, `LISTEN_FDS`, `LISTEN_PID`
and `XDG_RUNTIME_DIR`, then read.
The places number **forty-eight**.
Each entry says the file and lines, what the use is, what it gives the wrapper,
and whether a commit depends on it.

What a commit depends on, in one sentence, is this:
a process that outlives the command and the terminal that started it,
one per worktree, that is reachable at a socket path, that is started by the
first command that needs it and started again after it died, whose program is
the wrapper's own release, and whose removal a removed worktree gets.
Everything else in the list is a consequence of the manager being the thing
that provides those, a repair for a state the manager can be in, or a
convenience built on the manager's bookkeeping.

### Provisioning: writing units and asking the manager to start them

**1. `src/native/service/service_provision.rs`, lines 1 to 467.**
The provisioning itself: `unit_texts` (line 112) fills the two templates for
this host, `in_place` and `in_place_with` (139, 144) decide whether anything
must be written by comparing the installed files byte for byte, reading the
record of what the manager was told, checking for an earlier release's units
and checking the socket file, `provision_with` (198) writes the templates
through a temporary file and a rename, asks the manager to reload when a held
unit changed, notes the worktree, asks whether a held socket listens elsewhere,
resets a failed state, starts the worktree's socket instance, records the
texts, and stops the earlier release's socket of the same worktree,
`ask_manager` (295) is the sequence of calls, `stop_earlier_instance_socket`
(376) the stop, `provision` (403) the entry that takes the fast path when
`in_place` says so and otherwise connects to the session bus, and
`provision_when_needed` (453) the call on the commit path, which times itself
into `provisionMs`.
What it gives: starting on demand (the socket unit starts the service on the
first connection), one instance per worktree (a unit instance is named by a
hash of the worktree's Git directory), restart after a crash (`Restart=on-failure`
in the service unit), a life that outlasts the command and its terminal (the
manager owns the process), a cgroup of its own (the manager puts every unit in
one), logs (the manager captures standard error into the journal), and order at
login (units are enabled or started by the manager, not by the wrapper).
A commit depends on: starting on demand, one instance per worktree, restart
after a crash, a life that outlasts the command and the terminal, and a cgroup
of its own. It does not depend on the reload, the failed-state reset, or the
unit file being the record of anything.

**2. `src/native/service/service_units/monochromatic-git-policy-cli@.socket` and
`monochromatic-git-policy-cli@.service`, both files.**
The two unit templates the wrapper ships and writes.
The socket template listens on `%t/monochromatic-git-policy-cli/%i.sock` with
mode 0600 and directory mode 0700.
The service template names the wrapper's own executable in its service mode
with `--journal %S/monochromatic-git-policy-cli/service/%i --git <path> --instance %i`,
sets `SyslogIdentifier`, `Restart=on-failure`, `StateDirectory`, `KillMode=mixed`
and `TimeoutStopSec`.
What it gives: everything in entry 1 except the reload, expressed as the
manager's own declarations. The `StateDirectory=` is also what tells the
wrapper where the instance's journal lives
(`service_journal_folder_of_an_instance.rs`, entry 12).
A commit depends on: the socket path, the one-instance-per-worktree naming,
the restart, the `--journal` and `--instance` words, and the stop semantics.
It does not depend on the manager reading the file.

**3. `src/native/service/service_provision_host_and_manager.rs`, lines 1 to 297.**
What provisioning needs: `ProvisionHost` (the runtime directory, the user unit
directory `~/.config/systemd/user`, the session bus socket, the wrapper's own
executable, the real Git, the instance, the worktree's Git directory, the
retirement wait), the `Manager` trait (reload, reset failed, start, stop, stop
alone, active state, listen addresses), the `Bus` implementation of it,
`runtime_unit_dir` (`$XDG_RUNTIME_DIR/systemd/user`), `RETIREMENT_WAIT`,
`STOP_TIMEOUT_SECONDS`, `platform_without_manager`, and `refusal`, which turns a
D-Bus error into a person's message.
What it gives: the shape of every manager call and the words for a machine that
has none.
A commit depends on: the host's paths and programs, and the words for a machine
that has no manager (which must survive as words for a machine that has none
*needed*).

**4. `src/native/service/service_provision_unit_file_ownership_and_writes.rs`,
lines 1 to 197.**
`OWN_MARKERS` (25) and the earlier releases' markers, `is_own_unit_file` (43),
`carries_own_marker` (51), `any_named_anywhere` (69),
`own_unit_file_in_runtime_or_user_directory`, `write_atomically`, and
`remove_settled`.
What it gives: a unit file is written whole or not at all, and a unit file that
is not this wrapper's is never touched.
A commit depends on: the atomic write (which the new design needs for its own
files) and the ownership check (which the retirement of earlier releases'
files needs). It does not depend on the manager reading the files.

**5. `src/native/service/service_provision_loaded_unit_texts.rs`, lines 1 to 197.**
The record of the two unit texts the manager was last told about, kept beside
the worktree notes, plus `record`, `remembers`, `forget` and the
digest of the texts.
What it gives: one small read decides whether the installed templates are also
the ones the manager loaded, so the fast path needs no bus call; it is what
repairs an upgrade that a kill interrupted between the write and the reload.
A commit depends on: no. This is a repair of the manager's caching, and with no
manager there is no cache.

**6. `src/native/service/service_provision_earlier_service_retirement.rs`,
lines 1 to 197.**
`retire_earlier_service` asks the single service of the first releases (one
socket for every worktree) to retire, waits for the manager to report it
stopped, stops its units and removes their unit files, and returns whether the
manager must reload. `clear_earlier_units` removes unit files the earliest
releases wrote into `~/.config/systemd/user`.
What it gives: an earlier release's single service stops serving once this
release runs, and its files stay gone.
A commit depends on: no. What it protects is a service of another release that
would answer this release's protocol; the protocol's own version check and the
retire message cover that without the manager (entry 21).

**7. `src/native/service/service_provision_refusals_before_the_wait.rs`,
lines 1 to 165.**
`refuse_a_path_no_socket_address_can_carry` and
`refuse_a_service_that_listens_elsewhere`, the second built on the manager's
`Listen` property.
What it gives: a commit says why it cannot reach a service instead of waiting
out a ceiling, when the runtime directory is too long for a socket address or
when the manager made the unit listen in a runtime directory the command does
not use.
A commit depends on: the first (the kernel's limit stays), not the second,
which exists only because a unit's `%t` is the manager's runtime directory.

**8. `src/native/service/systemd_dbus.rs`, `systemd_dbus_wire_format.rs`,
`systemd_dbus_tests.rs` and `systemd_dbus_wire_format`'s tests, lines 1 to 511,
1 to 352 and 1 to 826.**
The D-Bus client: a bounded connect to the session bus socket at
`session_bus_path` (171), `Hello`, and the manager calls `Reload` (77),
`StartUnit` (80), `RestartUnit` (83), `StopUnit` (86), `ResetFailedUnit` (89),
`GetUnit` (94) and a property read, each under `EXCHANGE_WAIT` (20 s, line 61),
with `NO_SUCH_UNIT` and `manager_is_absent` (130) as the two answers that mean
"nothing to do" and "no manager here". The wire format module writes and reads
the D-Bus message itself. `zbus` is a dependency of the crate for this file
(`Cargo.toml`, the `zbus` line).
What it gives: every call in the inventory goes through these two files.
A commit depends on: no. Nothing here starts or finds the service without the
manager.

**9. `src/native/service/service_instance.rs`, lines 1 to 277.**
The instance name (32 hex digits of the FNV-1a hash of the worktree's real Git
directory), `socket_path` (line 200), the socket folder
`monochromatic-git-policy-cli` under the runtime directory, the unit instance
names `socket_unit_instance` and `service_unit_instance` (line 220, 225), the
earlier release's names (232, 238) and its socket path (245), and the socket
address length checks (155 to 197).
What it gives: one service per worktree, named by the worktree, reachable at
one path.
A commit depends on: the name and the socket path, yes. The unit instance names
and the earlier release's paths only for retirement.

**10. `src/native/service/service_journal_folder_of_an_instance.rs`,
lines 1 to 60.**
`journal_of(<state dir>/monochromatic-git-policy-cli, instance)`, the
`--journal` value the unit passes.
What it gives: the instance's journal location.
A commit depends on: yes, and it is derived from the unit today: the new design
must pass the same path to the service it starts.

**11. `src/native/service/earlier_unix_sockets_that_listen.rs`, lines 1 to 180.**
Reads `/proc/net/unix` to find whether an earlier release's unit for this
worktree is listening, with the observation that a socket unit's file stays
after its unit stops.
What it gives: provisioning knows whether the earlier release's socket of this
worktree still accepts connections, which decides whether the stop is to be
tried again.
A commit depends on: no.

### The service's own dependence on how it was started

**12. `src/native/service/service_server_socket_binding.rs`, lines 1 to 200.**
`Bind` (`Abstract`, `Path`, `Inherited`), `INHERITED_DESCRIPTOR` (3),
`ensure_socket_passed_in` (`LISTEN_PID` and `LISTEN_FDS`), `socket_passed_in`,
and `bind_path`, which takes over a socket file that nothing listens on.
What it gives: the service takes the listening socket the manager passed it,
and a path-bound service replaces a stale socket file.
A commit depends on: yes for `bind_path` and for the path form, no for
`Inherited`.

**13. `src/native/service/service_main.rs`, lines 1 to 361.**
The service mode. Its documentation says "The operating system starts it on
demand and hands it the listening socket (socket activation); the wrapper never
starts it" (lines 28 to 34); `Bind::Inherited` is the default when neither
`--abstract` nor `--path` is given (line 300), and a start with no socket from
the manager is refused with words that say the manager passes it (line 306);
`--journal` (319), `--git` (332), `--instance` (329), `--idle-seconds` and
`--release` (333, 336) are the options; `stop_own_socket_when_the_worktree_is_gone`
(341) stops the unit through the manager when the worktree is gone;
`SERVICE_WORKER_CAP` (line 160) sizes the runtime; `EXIT_NOTHING_TO_SERVE` is
zero because `Restart=on-failure` would restart a refused unit (line 150).
What it gives: the process, its options, and its idle exit.
A commit depends on: yes, the mode and its options. The `Inherited` default and
the words about the manager do not survive.

**14. `src/native/service/service_server.rs`, lines 1 to 644, and
`service_server_socket_binding.rs` as above.**
The idle exit `IDLE_EXIT` (line 129, three hours, citing Bazel's
`--max_idle_secs` default), and the serving loop's own statement that the
service never starts itself (lines 20 to 24).
What it gives: the service ends by itself when idle, so a manager that keeps it
alive is not what bounds its life.
A commit depends on: yes for the idle exit, which the new design keeps for the
same reason and with the same number.

**15. `src/native/service/service_signal.rs` (1 to 80) and
`src/native/service/service_server_shutdown_after_a_stop.rs` (1 to 120).**
The signal that ends the service, documented as the manager's `SIGTERM` stop
(`service_signal.rs` line 3), and the bounded shutdown that follows it, which is
what `KillMode=mixed` and `TimeoutStopSec=` in the unit are sized for.
What it gives: a stop that lets a branch write finish cannot happen any more;
what stays is a signal handler and a bounded shutdown, which are useful whoever
sends the signal.
A commit depends on: no, but the handler stays for a person's `kill` and for
the removal sweep.

**16. `src/native/service/service_client_reach_service.rs`, lines 1 to 381.**
`reach` (line 51): connect, read the greeting, and, while nothing greets, ask
the manager for the unit's `ActiveState` every second (`start_watch.every`),
start an `inactive` unit (`watch.start`), reset and start a `failed` one once
(`watch.reset`, `watch.start`), restart an `active` unit that greets nothing
for `SERVICE_GREETING_SILENCE` (`watch.restart`), reload all units once when
the start ceiling is reached (`watch.reload`), and give up at `START_CEILING`
(60 s) or two `failed` answers. `greeting_patience_without_a_manager` (line 236)
is the branch where no manager can be asked, and `nothing_answered` (line 259)
its message.
What it gives: a start is diagnosed instead of hung, and the three states a
manager can put a unit in are repaired.
A commit depends on: the wait for a start and its bound, yes; the manager's
answers, no.

**17. `src/native/service/service_client_settings_and_watches.rs`,
lines 1 to 234.**
`Activation::Managed` (line 41), `ClientConfig.start_watch` (line 55) and
`units_record_dir` (line 105), `ManagerWatch` (line 130), `MANAGER_ASK_EVERY`
(174), `unit_state`, `start_unit`, `reload_units`, `reset_failed_unit`,
`restart_unit` (178 to 220), and `manager_watch_for` (224).
What it gives: the closures the client's wait calls, and the fact that
`Activation` has only two values, one of which means "this platform cannot".
A commit depends on: the settings, yes; the watch, no.

**18. `src/native/service/service_provision.rs`'s `in_place` and
`service_provision_loaded_unit_texts.rs`'s record, already counted, are also
what makes a commit's provisioning cost nothing when nothing changed.**
Listed here so the count does not hide it: the fast path is two small reads and
two `stat` calls per commit that reaches the service.

### The route: how a commit finds out whether it may use the service

**19. `src/native/landing/service_route.rs`, lines 1 to 495.**
`RouteInputs` reads `XDG_RUNTIME_DIR` (74, 380), `DBUS_SESSION_BUS_ADDRESS`
(76, 383), `XDG_CONFIG_HOME` and `HOME`; `user_unit_dir` (107) builds
`~/.config/systemd/user`; `choose_route` (139) returns no route at all unless a
session bus socket and the wrapper's own executable are there, and says so in
its documentation ("Provisioning needs the session bus socket and the wrapper's
own executable"); `remember_own_executable` (59) exists to name the executable
in the unit file.
What it gives: a commit decides whether it can reach a service.
A commit depends on: yes, and today it depends through this decision on a
session bus existing, which is the single hardest dependency in the package:
on a machine without a bus, `choose_route` returns nothing and the commit stops
with the "no user service manager" message.

**20. `src/native/landing/service_commit_input_notes_and_embedded_commit.rs`,
lines 240 to 310.**
`managed_client_config` (283) sets `Activation::Managed` and
`start_watch: provisioning.map(manager_watch_for)`, and `refusal_without_route`
(299) renders the platform's message for a worktree with no route.
What it gives: the client's settings for a commit, and the refusal.
A commit depends on: yes.

**21. `src/native/landing/service_messages.rs`, lines 1 to 520.**
`LOG_COMMAND` (`journalctl --user --unit 'monochromatic-git-policy-cli@*.service'`,
line 39), `LINGER_COMMAND` (`loginctl enable-linger`, 42), `RESTART_COMMAND`
(`systemctl --user restart ...`, 46), `STOP_COMMAND` (`systemctl --user stop ...`,
55), `Platform::LinuxWithoutUserServices` (79), the start-again and
runtime-directory subjects (215, 226), and the rendered remedies (284, 475).
What it gives: the words a person reads when no service answers.
A commit depends on: the words, yes; the commands, no, because there is
nothing for a person to enable or restart on the new path.

**22. `src/native/landing/service_messages_of_a_start_that_cannot_come.rs`,
lines 1 to 60.**
The message for a service the manager made listen in the manager's runtime
directory while the command was given another, built on
`refuse_a_service_that_listens_elsewhere`.
What it gives: a plain explanation of a state only a manager can create.
A commit depends on: no.

**23. `src/native/landing/service_messages_of_a_wait_for_the_service.rs`,
lines 1 to 120.**
The notice a waiting commit prints after a second, naming the address and "the
service manager says <unit> is <state>".
What it gives: a wait a person can see, and the state that explains it.
A commit depends on: the notice, yes; the manager's state in it, no.

**24. `src/native/landing/earlier_service_journal_adopted.rs` (1 to 200) and
`earlier_service_process_look.rs` (1 to 200).**
The adoption of the journals an earlier release's helper left open, its
expectations about what the manager enforces (line 51: "systemd would enforce"),
why no unit file or start option turns the adoption on for the service itself
(61), the stop of the earlier release's socket unit of the same worktree (67),
and the reading of every process's command line to recognise an earlier helper
by the words its unit passed (`earlier_service_process_look.rs` line 103).
What it gives: a person's earlier batches are settled instead of being served
by a helper of an earlier release.
A commit depends on: no.

**25. `src/native/landing/held_push_release_wakes_the_service.rs`, lines 40 to 100.**
Wake the service to try a held push: the route is read and the service is
woken; the documentation says the wake needs "a service manager and the session
bus" (line 46).
What it gives: a push a lock held is retried without a person running a command.
A commit depends on: no, but a push depends on it: without a route the record
stays on disk for the next command (see entry 34).

**26. `src/native/push/auto_push_trigger.rs`, lines 395 to 430.**
The writes the service makes after a commit: the record is written before the
route is read, so "a context with no route (a non-systemd session, a container,
another `XDG_RUNTIME_DIR`)" leaves the push on disk instead of losing it.
What it gives: nothing is lost when no service can be reached.
A commit depends on: no.

**27. `src/native/push/push_deadline_advice.rs`, lines 1 to 50, and
`src/native/push/push_failure_notice_words.rs`, line 90 to 100.**
The advice a push prints from inside the service: "run
`systemctl --user set-environment NAME=value` and then
`systemctl --user restart ...`" (line 43 and 98), with reason.
What it gives: a person can change an environment variable the service reads.
A commit depends on: no.

**28. `src/native/invocation/child_environment.rs`, line 460 to 480.**
The service does not inherit the wrapper's environment because "the systemd
user manager that starts the commit service" does not, so the wrapper passes
what the service needs at its handoff.
What it gives: the service's children see a deliberate environment.
A commit depends on: the overlay, yes; the sentence, no. The new design chooses
the child's environment explicitly, which makes this rule the same and its
reason different.

**29. `src/native/landing/service_handoff.rs`, lines 85 to 100.**
The same fact at the handoff: "A systemd user service does not inherit the
wrapper's environment", so the handoff passes a unit file's worth of settings
in the message.
A commit depends on: the message, yes; the sentence, no.

### The unit file as a record, and what the manager's bookkeeping is read for

**30. `src/native/invocation/real_git_record.rs` (1 to 200),
`real_git_resolution.rs` (1 to 260), `real_git.rs` (line 551 to 620) and
`invocation/mod.rs` (line 119).**
The wrapper reads the real Git a worktree's service runs from the unit file
under the runtime directory (`real_git_record.rs` line 71 names
`systemd/user/cli-git@.service`), so that a wrapper used as Git can refuse to
start itself and the record of the real Git survives an upgrade.
What it gives: a record of the real Git per worktree, and the refusal that keeps
a wrapper from being started as Git.
A commit depends on: the record's content, yes, because the service must run the
same real Git the command would; the file being a unit file, no.

**31. `src/native/worktree/worktree_removal_stops_units.rs`, lines 1 to 250.**
After a worktree removal, stop the removed worktree's units through the manager
(`stop_units_after_removal`, 126, `stop_units_of_removed_worktree`, 184), read
`XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` for it (185, 194), and place
the state directory as `systemd.exec(5)` places it (227, 242).
What it gives: removal of what a removed worktree leaves: its socket unit and
its service, and with them its socket file.
A commit depends on: no. A removal depends on it, and the brief's requirement is
that nothing is left that stops the next command.

**32. `src/native/service/service_units_of_removed_worktrees.rs`, lines 1 to 560.**
The sweep: the worktree note beside the units
(`<runtime>/systemd/user/monochromatic-git-policy-cli-worktree-notes/<instance>.git-dir`),
`worktree_of` (248), `instances_with_units` (292), `survey` (313),
`own_worktree_state` (378), `stop_units` (420), `stop_own_socket` (427),
`stop_own_socket_of_a_gone_worktree` (463),
`stop_own_socket_when_the_worktree_is_gone` (540), and the words about the state
directory's journal (508).
What it gives: the removal a removed worktree gets, and the recognition of a
worktree that is gone.
A commit depends on: the recognition (a service whose worktree is gone must
not serve), yes; the stop through the manager, no.

**33. `src/native/service/service_files_of_a_stopped_instance.rs`, lines 1 to 500.**
`NOTE_ENDING` (70), `NOTES_FOLDER` (77, under the runtime **systemd** unit
directory), `note_path` (84), `note_worktree` (125), the journal of an instance
(318), `work_a_person_may_want` (338),
`remove_journal_of_a_stopped_instance` (444) and `forget_stopped_instance` (485).
What it gives: the note that says which worktree a unit belongs to, and the
removal of a stopped instance's files.
A commit depends on: the note (the new design writes it before the service
reads it), yes; its location inside `systemd/user`, no.

**34. `src/native/service/service_is_gone_after_a_stop.rs`, lines 1 to 120.**
Ask the manager for the unit's `ActiveState` every
`SERVICE_GONE_AFTER_A_STOP` until nothing of the unit is left, because a stop
call returns when the job is queued, not when the process is gone.
What it gives: the fact that nothing writes an instance's journal any more,
which is what lets the journal be removed.
A commit depends on: no. A kill of a service that a command did not start needs
a different way to prove it is gone (a pid and its start time).

**35. `src/native/invocation/management_service_stop_units_of_removed_worktrees.rs`,
lines 1 to 240.**
The management command `git monochromatic-git-policy-cli service
stop-units-of-removed-worktrees`: read `XDG_RUNTIME_DIR` (49), the bus (210),
and stop every removed worktree's units through the manager (127 to 200).
What it gives: a person can run the sweep by hand.
A commit depends on: no.

**36. `src/native/invocation/management_service_settle_earlier_batches.rs`,
lines 1 to 300.**
The management command that settles an earlier release's open batches, whose
advice names `systemctl --user stop <own units>` and
`systemctl --user stop <shared units>` (140, 141), and which reads the earlier
unit's `StateDirectory=` to find the journal (74).
What it gives: a person's earlier batches are finished.
A commit depends on: no.

### The journal, which is not the manager but is read from it

**37. `src/native/service/journald_native.rs` (1 to 760),
`journald_native_socket_target.rs` (1 to 120), `journald_native_tests.rs`,
`client_journal.rs` (1 to 200).**
The service's log path: the native journal protocol over the datagram socket at
`/run/systemd/journal/socket` when `$JOURNAL_STREAM` says standard error is the
journal, and otherwise one plain line on standard error
(`journald_native.rs` lines 1 to 40, `Sink::choose` line 425,
`deliver` 501, `fall_back` 549).
The identifiers `monochromatic-git-policy-cli-service` and
`monochromatic-git-policy-cli` (ID 47, 51).
`client_journal.rs` sends the wrapper's own entries without waiting.
What it gives: logs with a priority, fields and the kernel's own time, and the
journal's rotation and retention; the fallback is a line on standard error,
which needs a place to go and a bound.
A commit depends on: no. Nothing waits for the journal; a failed send goes to
standard error. The journal is not the service manager: the socket is written
by journald, which exists on machines with no user manager, and the fallback
already covers its absence. What the new design must add is where the fallback
goes when the wrapper starts the service itself, and a bound on its size.

**38. `src/native/service/service_timing.rs`, lines 1 to 130, and
`service_timing_tests.rs`.**
The timing lines the service writes, the `SyslogIdentifier` they use, and the
context (release, instance) every later entry carries (line 37, 57).
What it gives: one place that composes a line and sends it.
A commit depends on: no, beyond the lines themselves.

**39. `src/native/service/service_round.rs`, line 40 to 60, and
`src/native/commit/commit_timing.rs`, line 213.**
The memory peak of a round, read from the 28 commit-service units of this host
with `systemctl --user show 'cli-git@*.service'`, and the instance name that a
person's `journalctl --user --unit=cli-git@<instance>.service` takes.
What it gives: a measurement that sized a bound, and a name a person types.
A commit depends on: no.

**40. `src/native/invocation/real_git.rs` line 603 to 615,
`real_git_record.rs` line 60 to 80, and `src/native/service/mod.rs` line 1 to 20.**
The module documents that state the service is "started by the operating system
on demand" (`mod.rs` lines 1 to 10, `service_server.rs` 20 to 24,
`service_main.rs` 28 to 34), and the unit-file path constants.
What it gives: the design's own statement of the dependency.
A commit depends on: no.

### Tests, fixtures and guards that reach for a manager

**41. `src/native/service/service_provision_tests.rs` (1 to 700),
`service_provision_never_starts_at_login_tests.rs` (1 to 300),
`service_provision_refusals_before_the_wait_tests.rs`,
`service_provision_earlier_service_retirement_tests.rs`,
`service_provision_loaded_unit_texts_tests.rs`,
`service_provision_unit_file_ownership_and_writes_tests.rs`,
`service_units_of_removed_worktrees_tests.rs`,
`service_files_of_a_stopped_instance_tests.rs`,
`service_units_fixture_test_support.rs` and
`service_unit_manager_recorder_test_support.rs`.**
The provisioning controls: a stand-in `Manager` records the calls and answers
scripted states, a disposable runtime directory stands in for the machine, and
the tests assert the unit texts, the reload, the start, the retirement and the
refusals. The login guard asserts that nothing of the wrapper writes a unit
outside the runtime directory, links a unit into a `*.wants` directory, or
calls the manager at login.
What it gives: proof that the manager path does what the documents say.
A commit depends on: no. Every one of these tests goes with its subject.

**42. `src/native/service/systemd_dbus_tests.rs` (1 to 826) and the wire format
tests.**
The D-Bus client's controls against a stand-in bus.
A commit depends on: no.

**43. `src/native/binary/binary_service_provision_tests.rs` (1 to 400),
`binary_service_upgrade_one_executable_tests.rs`,
`binary_service_mode_tests.rs`, `binary_service_commit_tests.rs`,
`binary_support_fixture_service.rs` (lines 150 to 300) and
`binary_support_resource_limits.rs` (line 105).**
The binary-level controls: a fixture runtime directory and a fixture `HOME`
(so a service never reaches this machine's manager), the service started by
hand with `--abstract` or `--path`, and the one-executable checks.
What it gives: proof at the user boundary.
A commit depends on: no, but these are the tests that must grow the new
behaviour.

**44. `src/native/source_guard/no_start_at_login_tests.rs` (1 to 200) and
`src/native/source_guard/service_lock_inventory_tests.rs`.**
Source guards: no unit file outside the runtime directory, no `*.wants` link,
no manager call at login, and the inventory of the locks the wrapper takes.
What it gives: the owner's rule that nothing of the wrapper starts at login is
held by a test rather than by a promise.
A commit depends on: no, but the guards must be rewritten for a design that
writes no unit at all.

**45. `src/native/worktree/worktree_removal_stops_units_tests.rs`,
`src/native/worktree/worktree_remove_hand_over_tests.rs` (line 233),
`src/native/landing/service_handoff_tests.rs`,
`src/native/invocation/real_git_record_tests.rs`,
`management_service_stop_units_of_removed_worktrees`'s tests and
`management_service_settle_earlier_batches`'s tests.**
The controls of the consumers of the manager calls listed above.
A commit depends on: no.

### The package's own tooling and tasks, which a user never gets

**46. `bin/`, at these places: `native-gate-launch.ts` (3 to 20),
`native-gate-start.ts` (2 to 40), `native-gate-unit.ts` (2 to 60),
`native-gate-unit-name.ts`, `native-gate-verdict-reader.ts` (4 to 230),
`native-gate-caps.ts`, `native-capped-scope-command.ts` (2 to 380),
`native-ts-checks.ts` (3 to 200), `systemd-user-unit-running.ts` (1 to 40),
`native-planted-controls-unit.ts`, `native-planted-parallel.ts`,
`native-mutation-parallel.ts`, `native-perf-service.ts`, and the stand-ins
`stand-in-scope-programs-for-tests.ts` and `stand-in-gate-programs-for-tests.ts`,
with their `.unit.test.ts` files.
These run the package's gates, mutation campaigns and checks each in a capped
**systemd user scope or transient unit of this host**, and read a run's verdict
from a log rather than from `systemctl show`.
What it gives: cgroups around builds and test runs, one log per run, and a
verdict that survives the unit being collected.
A commit depends on: no, and a user never gets these files: `Cargo.toml`'s
`include` ships `src/native/**`, `README.md` and `LICENSE`, so `bin/` is the
repository's tooling, not the release.

**47. `mise.toml`, lines 1 to 130 and 240 to 330: the tasks
`native:test:container`, `native:mutation`, `native:mutation:scoped`,
`native:mutation:parallel`, `native:mutation:planted`, `native:gate`,
`native:gate:main-tip`, `native:gate:verdict`, `native:ts-checks` and
`native:release:host-controls`.**
The same fact as entry 46 at the task level: every task that runs a bounded
build, test, mutation or release check does it in a scope of this host's
manager, and `native:install` installs a release that provisions units.
What it gives: the developer's bounds.
A commit depends on: no. The install task is the only one that touches what a
user gets, and it does so by installing a release whose units the wrapper
writes; that is entry 1, not this one.

### Documents

**48. `SPEC.md`, `README.md`, `DESIGN.md` and the package's `doc/`, at these
places.**
`SPEC.md` lines 1843, 6330 to 6340, 6925 to 7003 and 7404 to 7423 state the
provisioning, the manager watch, the repairs and the reload as the design.
`README.md` lines 26, 137, 1431 to 1432, 1594 to 1614, 3698 to 3699, 4199,
6599 to 6869 state socket activation, the unit files, the runtime unit
directory, `loginctl enable-linger`, and "Systemd units are fine."
(lines 6825).
`README.md` lines 5550 to 5697 describe the package's own gate tooling and its
systemd scopes (entries 46 and 47).
`DESIGN.md` line 1 mentions the service once, without a manager.
The package's `doc/` mentions it only in `planted-control-and-durable-survivors-resume.md`,
about a mutation campaign's cgroup, not about the wrapper.
Repository-wide, `doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md`
(53 mentions), `doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`
(47), `doc/planning/git-wrapper-reload-remedy.md` (4) and
`doc/handover/cli-git-rust-implementation.md` record the incident and the
decision; they are history and stay as they are (rule `WR6`).
What it gives: the documents a person reads.
A commit depends on: no, but every one of these places becomes false when the
manager leaves the path, so part 8 of this work rewrites them.

### What the count comes to

Of the forty-eight places, a commit depends on these:
the socket path and the one-instance-per-worktree naming (1, 2, 9),
a process that outlives the command and the terminal (1, 2, 13),
a cgroup of its own (1, 2),
restart after a crash (1, 2),
starting on demand (1, 2, 16),
the service's mode and its options (13, 14),
the idle exit (14),
the journal folder the service is given (10),
the address the client connects to and the wait for a start (16, 17),
the words a person reads when nothing answers (21, 23),
and the record of the real Git (30).

The other twenty-nine places are the manager's bookkeeping and the repairs it
makes possible: the reload (1, 5), the failed-state reset (16), the restart of a
unit that stopped inside its own process (16), the two-runtime-directory
refusal (7, 22), the retirement of the earlier single service (6, 11),
the unit files as an install record (4), the sweep and the management commands
(31 to 36), the journal (37, which is not the manager), and the tooling and the
tests (41 to 48).

## Part 2: how Bazel does without a service manager

Sources: the Bazel repository at commit
`7574c94c3b87f4edba2167e61912c86a6a6cf77c` (fetched 2026-11-12, a sparse
checkout of `src/main/cpp`, `src/main/java/com/google/devtools/build/lib/server`,
`src/main/java/com/google/devtools/build/lib/runtime`, `src/main/tools` and
`docs`), and its documentation page `docs/run/client-server.mdx`.
Every claim below is read from those files at those lines; where the
documentation is silent the source is quoted.

**Finding a running server.**
The documentation says the client finds the server "based on the output base"
(`docs/run/client-server.mdx` lines 15 to 22), and that one output base is one
server, so two workspaces have two servers and two users have two.
The source is more specific. `<output base>/server/server_info.rawproto` holds
a protobuf `ServerInfo` with the address, a request cookie, a response cookie
and the server's pid. `BlazeServer::Connect` (`src/main/cpp/blaze.cc` lines 1102
to 1135) reads that file, refuses an address that is not `127.0.0.1:`,
`[0:0:0:0:0:0:0:1]:` or `[::1]:` (1120 to 1125), refuses a negative pid, calls
`VerifyServerProcess(server_pid)` and then pings the server, sending the request
cookie (1087) and accepting the answer only if the response cookie equals the
one the file held (1093).
So the file is not trusted by itself: the process it names must still be the
process that wrote it, and the server must prove it holds the matching cookie.
`VerifyServerProcess` (`src/main/cpp/blaze_util_linux.cc` lines 290 to 302) is
the pid plus the process's start time read from `/proc/<pid>/stat`, and it
treats a zombie or dead state as gone; the start time is written beside the pid
by `WriteSystemSpecificProcessIdentifier` (268 to 285) into
`server/server.starttime`.
A second file, `server/server.pid.txt`, is written by the daemonizer before the
socket is bound, and `GetServerPid` (`src/main/cpp/server_launcher.cc` lines 468
to 478) reads it; its comment says "there is no race here on startup since the
server creates the pid file strictly before it binds the socket" (462 to 464).
Bazel's transport is TCP on the loopback address with gRPC; the wrapper's is a
unix socket path, so the port file is the one part of this that has no analogue
to adopt, and the wrapper already has the better answer: a path in the runtime
directory.

**Starting one.**
`StartServerAndConnect` (`src/main/cpp/blaze.cc` lines 504 to 560) deletes the
stale `server/command_port` and OOM files, writes the `server/cmdline` file the
version check reads later, changes into the workspace, and calls
`ExecuteDaemon`.
`ExecuteDaemon` (`src/main/cpp/blaze_util_posix.cc` lines 411 to 512) creates a
`socketpair`, `posix_spawn`s a small helper program, `daemonize`, closing the
parent's end in the child, waits for that helper to exit (which guarantees the
pid file exists), reads the pid from the pid file, and keeps the other end of
the socketpair in a `SocketBlazeServerStartup`. `IsStillAlive` (383 to 407)
polls that descriptor for `POLLIN` or `POLLHUP`; when the helper's end closes,
the server died during startup, and `ConnectOrDie` then prints the server's log
and exits instead of waiting out its timeout (478 to 500).
The helper does the detachment (`src/main/tools/daemonize.cc`):
it waits for the parent's pipe write so the pid file exists first (266 to 272),
ignores `SIGHUP`, calls `setsid()` (270), sets standard input to `/dev/null` and
standard output and standard error to the log file, opened `O_TRUNC` or
`O_APPEND` (`SetupStdio`, 62 to 90), and then `execv`s the server (303).
The server therefore leaves the client's session, its terminal and its process
group, which is also why the client can later kill all of it with
`killpg` (see "replacing").
What it inherits is the environment the client prepares
(`PrepareEnvironmentForJvm`, `server_launcher.cc` 573 to 660, and
`ExecuteDaemon`'s `env` argument), not the client's own: it is passed as a
complete environment, and the variables a JVM cannot survive are removed.

**Two clients starting at once.**
Bazel does not let two clients race to start one server: the client takes an
exclusive file lock and holds it for its whole invocation.
`AcquireClientLocks` (`src/main/cpp/server_launcher.cc` lines 58 to 113) takes
a shared lock on the install base (`<install base>.lock`, to allow concurrent
readers and to serialize garbage collection) and an exclusive lock on
`<output base>/lock`. `AcquireLock` (`src/main/cpp/blaze_util_posix.cc` lines
695 to 760) opens the lock file, takes `flock`-style lock through `TryLock`,
writes the holder's identity into the file for the person (`WriteOwnerInformation`,
724 to 727), checks the file was not concurrently deleted (`StillExists`, 720),
and on failure reads the owner's text, prints "Another command holds the output
base lock" with it (744), sleeps 500 ms and retries until
`--block_for_lock`'s timeout, which defaults to waiting forever and can be a
duration or `false` (`startup_options.cc` 228 to 262).
The documentation states the consequence: "Each server can handle at most one
invocation at a time; further concurrent invocations will either block or
fail-fast (see `--block_for_lock`)" (`docs/run/client-server.mdx` lines 20 to
22).
So Bazel's answer to "two clients starting at once" is a lock that covers the
whole command. That is the one Bazel answer the wrapper cannot adopt: a wrapper
commit queues at the service instead, so the lock must cover only the start of
one worktree's server, and the brief's rule that a lock covers the smallest
unit and has its reason beside it says the same.

**Replacing a server of another version or other startup options.**
The client writes the canonical server command line into `server/cmdline` before
a start (blaze.cc 535 to 538), because `/proc/<pid>/cmdline` cannot be read for
long command lines (673 to 677). `KillRunningServerIfDifferentStartupOptions`
(blaze.cc 658 to 709) reads that file, compares it through
`AreStartupOptionsDifferent` (589 to 650) ignoring a fixed list of volatile
options (`IsVolatileArg`, 565 to 587: `--option_sources`, `--max_idle_secs`,
`--connect_timeout_secs`, `--local_startup_timeout_secs`, `--client_debug`,
`--preemptible`, `-XX:HeapDumpPath`), prints which options differ, and kills the
server when they do. A server of another version is found by another record:
the `install` symlink in the output base, which
`EnsureCorrectRunningVersion` (712 to 731) compares with this installation and
replaces when it differs, killing a connected server with the message "Killing
running server because it is using another version". The documentation says the
same in one sentence: "the client first checks that the server is the
appropriate version; if not, the server is stopped and a new one started"
(`docs/run/client-server.mdx` lines 73 to 76).
The stop is `EnsurePreviousServerProcessTerminated` (blaze.cc 484 to 502), which
first verifies the pid, then `KillServerProcess`
(`src/main/cpp/blaze_util_posix.cc` lines 783 to 800), which sends `SIGKILL` to
the *process group* (`killpg`), not to the process, so everything the server
started dies with it, and then waits for the group to be gone.
This is Bazel's answer for "the server's program changed": a recorded command
line, a comparison, a kill of the group, and a fresh start.

**When a server exits by itself.**
`ServerWatcherRunnable` (`src/main/java/com/google/devtools/build/lib/server/ServerWatcherRunnable.java`
lines 30 to 221) watches the command manager: when the server has been idle for
`max_idle_secs`, it calls `server.shutdown()` (215 to 219). The default is three
hours (`src/main/cpp/startup_options.cc` line 124,
`max_idle_secs = blaze::IsRunningWithinTest() ? 15 : 3 * 3600`, which is 15 s
inside Bazel's own tests), documented as "The server process will stop after a
period of inactivity (3 hours, by default, which can be modified using the
startup option `--max_idle_secs`)" (`docs/run/client-server.mdx` lines 32 to
34), and the option says zero means never
(`BlazeServerStartupOptions.java` lines 216 to 225).
Memory pressure is an answer too, but an opt-in one:
`--shutdown_on_low_sys_mem` makes a server that has been idle for at least five
minutes exit when free RAM is below both 5 percent and 1 GiB
(`ServerWatcherRunnable.java` lines 36 to 40, 100 to 115; the option's text is
`BlazeServerStartupOptions.java` 227 to 236), and the documentation recommends
it together with a shorter idle timeout for machines with many workspaces
(`docs/run/client-server.mdx` lines 43 to 57).

**What a client does when the server died or does not answer.**
`ConnectOrDie` (blaze.cc 425 to 478) tries to connect every 100 ms, prints
"... still trying to connect to local <product> server (<pid>) after N
seconds ..." every second or so (435 to 448), and ends at
`local_startup_timeout_secs` (default 120, `startup_options.cc` line 94) with
"couldn't connect to server (<pid>) after N seconds." (477 to 478).
If the startup socketpair closes first, the message is "Server crashed during
startup" and the server's log is printed or named (457 to 470).
For a server that is not starting but already recorded, the client verifies the
pid and its start time: a pid file whose process does not verify is
`PID_FILE_BUT_NO_SERVER`, and one that verifies but does not answer is killed
and reported as `SERVER_UNRESPONSIVE` (blaze.cc 484 to 502).
The single RPC connect timeout is `--connect_timeout_secs`, default 30
(`startup_options.cc` line 93).

**Where the server's output goes.**
`ServerProcessInfo` (`src/main/cpp/server_process_info.cc` lines 20 to 33)
names `<output base>/server/jvm.out`, and the log is truncated at each start
unless `--server_jvm_out` names a file, in which case that file is appended to
(`jvm_log_file_append_ = !server_jvm_out.IsEmpty()`).
There is no size bound of Bazel's own: the truncation at each start is the
bound, and a long-lived server can append to it for hours.

**What Bazel says about cgroups and about being started under a service
manager.**
A cgroup of its own is an option, not a requirement, and the option is
Linux-only and off by default:
`--experimental_run_in_user_cgroup` ("If true, the Bazel server will be run with
systemd-run, and the user will own the cgroup. This flag only takes effect on
Linux.", `BlazeServerStartupOptions.java` lines 553 to 560), and
`--experimental_cgroup_parent` for a cgroup path someone else prepared (527 to
545).
The implementation is in the daemonizer (`src/main/tools/daemonize.cc` lines 198
to 303): it writes a shell script that `exec`s the server with its arguments
(199 to 240), then, only if `/usr/bin/systemd-run` exists and is executable
(281), runs `systemd-run --user --scope -- /bin/true` as a probe (283 to 291)
and only if that exits zero replaces itself with
`systemd-run --user --scope -- /bin/bash <script>` (294 to 298); "In all other
cases, including the cases when systemd-run is not available, we replace it with
the original exe" (278 to 280, 303).
For a prepared cgroup it reads `/proc/self/mounts` and writes the pid into
`cgroup.procs` of each cgroup mount, ignoring a controller it may not write
(`MoveToCgroup`, 330 to 370).
That is the whole of Bazel's use of a service manager, and it is a fallback:
the manager is found by a probe whose failure costs nothing, the probe and the
call are bounded by the client's own startup timeout, and nothing a build does
depends on it.

What the wrapper adopts from this, and where it does not fit, is part 3.

## Part 3: the design

The wrapper starts its own commit service and needs no service manager for
anything a commit depends on. Where a manager exists it is used as an
enhancement (part 4), on a path no commit waits on.
Each answer below says whether it is Bazel's, adopted, and where Bazel's does
not fit, why.

### Finding the server (adopted from Bazel, with the socket path as the file)

Bazel's client reads `<output base>/server/server_info.rawproto` and trusts it
only after the pid, its start time and the cookies agree.
The wrapper's server is reachable at a path that says whose it is, so the path
replaces the port file, and the greeting replaces the cookies:
a connection that does not answer with `Welcome { version, release }` in this
protocol is not this service, whatever file is at that path.
Adopted from Bazel: the pid note carries the process's start time as well as its
pid, and is trusted only when both still match the running process, so a reused
pid is never mistaken for the server.

The files of one instance, all in `$XDG_RUNTIME_DIR/monochromatic-git-policy-cli/`:

- `<instance>.sock`, the listening socket, mode 0600 in a directory mode 0700.
  The socket is the ownership: whoever binds it owns that worktree's service,
  and a second server that binds fails with `Address already in use` and exits.
- `<instance>.pid`, the text `<pid> <starttime>`, written by the service after
  it binds and removed by it as it ends.
- `<instance>.git-dir`, the note that says which worktree the instance is for,
  moved here from `$XDG_RUNTIME_DIR/systemd/user/monochromatic-git-policy-cli-worktree-notes/`.
- `<instance>.git`, the real Git the service runs, which replaces the record the
  unit file held (`real_git_record.rs`, entry 30 of part 1).
- `<instance>.start.lock`, the start lock (below), empty.
- `<instance>.log`, the service's output (below).

The greeting is the cookie; a socket file that a killed service left behind is
either replaced by the next bind (it refuses a connection) or, if something
answers on it with another protocol, that answer is not this service and the
client reports it instead of trusting it.

### Starting one (adopted from Bazel)

The client, when nothing answers:

1. takes the start lock (below);
2. connects once more, because the holder may have started the service;
3. starts the server and waits, bounded, for the greeting.

The start is Bazel's `daemonize`, in this program instead of in a helper:

- the wrapper forks an intermediate process, which forks the server and exits;
  the wrapper waits for the intermediate, so no zombie is left in the wrapper
  and no `waitpid(-1)` can steal another child's status;
- the server's `setsid()` makes it the leader of a new session and process
  group, so it outlives the command, the terminal and the shell that started it,
  and it can be killed as a group;
- `SIGHUP` is ignored, standard input is `/dev/null`, and standard output and
  standard error are the instance's log file, opened for append;
- it is started as the wrapper's own executable in its service mode:
  `monochromatic-git-policy-cli service run --path <socket> --journal <dir>
  --git <real git> --instance <instance>`, plus `--release` never (the server
  announces its own build's release) and `--idle-seconds` never (the default is
  the design's).
  The journal directory is `journal_of(<state dir>/monochromatic-git-policy-cli,
  <instance>)`, the same path the unit's `StateDirectory=` gave, so an upgrade
  keeps every record it had.
- the child's environment is prepared by the wrapper, not inherited by accident:
  `service_child_overlay` as today, because the service must not inherit a
  terminal's `GIT_*` or a debugger's variables.

Bazel's socketpair for "the server died during startup" is not adopted: the
wrapper reads the log file's size and the pid note on failure and names the log,
and a server that dies before binding leaves no socket to greet, which the same
bounded wait sees. A pipe would be a second mechanism for the same fact.

Bazel's `server/cmdline` record for "the options changed" is not adopted either:
the wrapper's startup options cannot differ between two runs of one release, and
a changed release is caught by the greeting's release.

### Two clients starting at once (Bazel's lock, narrowed)

Bazel holds an exclusive lock on the output base for its whole command.
That does not fit: a wrapper commit is queued in the service and never holds a
repository lock, so a lock over the command would make one worktree's commits
wait on each other for no reason.

Adopted from Bazel: `flock` on a lock file, retried, with the holder's identity
written into the file for the person.
Narrowed: the lock is `<instance>.start.lock` and it covers exactly one thing,
starting this worktree's server, from "nothing answered" to "the greeting
came", and nothing else.
Its reason, beside it in the code: two clients that both find no service would
both start one, and the loser would exit with `Address already in use`, which is
harmless but writes two log files, two starts' worth of memory and one confusing
failure; the lock makes the second client wait for the first's service instead.
The kernel releases the lock when its holder dies at any instruction, so a kill
between the lock and the greeting costs the waiter one retry, never a stuck
lock.
The socket bind stays the real ownership: a lock that leaked would cost a
duplicated start, never two servers.

A client that cannot take the lock waits for the greeting at 25 ms intervals,
which is the pause the client already uses (`CONNECT_PAUSE`).

### Replacing a server of another release (Bazel's answer, through the protocol)

Bazel records the server's command line and the installation symlink, and kills
the server when either differs.
Adopted: the greeting carries the release and the protocol version, and the
client acts on it exactly as it does today: a server of another release is asked
to retire in the version it announced, it finishes its batch, removes its socket
and exits, and the next connection of the client starts this release's server
(the client's loop already retries within `start_wait`). The record of the
release is the server's own greeting, so nothing has to be written or compared.

For the state a manager used to repair (`RestartUnit` of a unit it calls
`active` whose program stopped inside its own process), the replacement is the
pid note: the client verifies the pid and start time, sends `SIGTERM` to the
process group, waits a bounded time for it to be gone, and starts a server
itself. That happens at most once per command, and only when a service accepted
the connection and then greeted nothing for `SERVICE_GREETING_SILENCE` (3 s,
unchanged).

### When a server exits by itself (adopted from Bazel)

The idle exit stays three hours, which is Bazel's `--max_idle_secs` default and
the number this repository already measured against its own timing log
(`IDLE_EXIT` in `service_server.rs`).
Bazel's low-memory exit, which is opt-in there (`--shutdown_on_low_sys_mem`),
becomes the default here, because the service no longer has a cgroup of its own:
after five minutes with no work, a service whose host has less than 5 percent
and less than 1 GiB of free memory exits, so that the kernel's memory-pressure
kill (which would take the whole cgroup, starter and all) is not what ends it.
The check is Bazel's, including the numbers (see part 2), read through `sysinfo`
as the budget code already does, and it is a decision the tests can drive with an
injected reading.

### Where the server's output goes (Bazel's truncation, plus a cap)

Bazel writes `<output base>/server/jvm.out`, truncated at each start unless a
file is named. Adopted: the instance's log file is truncated at each start when
it is larger than 4 MiB, so a start never inherits an unrelated history and the
file cannot grow without bound across starts.
Added, because Bazel has no bound within one server's life: the service drops
its plain-standard-error lines once the file reaches the cap, counting what it
dropped and writing one line about it, with one `fstat` per line and no lock.
Where journald's native socket exists, the service sends its entries there as
well, which is the log's rotation and retention and is read with
`journalctl --identifier=monochromatic-git-policy-cli`; that send never waits
(`journald_native.rs`, entry 37), and its failure changes nothing.

### What is lost without a service manager, and the answer to each loss

**The service lives in the cgroup of whatever started it.**
On this machine every agent's shell commands share one cgroup with a memory cap,
and a manager that kills on memory pressure ends that cgroup whole, the agent
included (`doc/troubleshooting/systemd-oomd-reaps-terminal-scope-with-the-agent-in-it.md`).
So the service can be killed at any moment with its starter, and its memory and
its children's are charged to its starter.
Answer: first, a kill at any instruction loses nothing and leaves nothing that
stops the next command, which the service's durable records already give (its
journal, its claims and its intents, restored at the next start), and the
client that was waiting starts the service again itself; second, the service is
small and bounded (4 workers, the measured reserve), and it exits when the host
is short of memory rather than being killed with its starter; third, where a
manager exists the enhancement gives it a cgroup of its own (part 4).

**Nobody restarts it but the next command.**
A service that dies while a commit waits is not restarted by anybody else. The
waiting client notices (its greeting silence bound), restarts the server itself
and re-sends its request, which the journal's record of the request makes safe
to repeat; a commit whose request was in a batch that already landed finds its
commit in the journal instead of landing twice (the behaviour that exists
today). A command that needs nothing (a hand-over) waits `HAND_OVER_GREETING_WAIT`
(2 s) and leaves its record for the next command.
There is no restart counter and no rate limit any more, because those existed to
keep the manager from restarting a broken program forever.

**Logs need a home with a bound on their size.**
The instance's log file, capped as above, truncated at each start, and beside it
the journal where journald exists. A person reads one file per worktree, named
in every failure message.

**Order at login, and anything that starts at boot, is lost.**
Nothing is lost that the wrapper wants: the owner's rule is that nothing of the
wrapper starts at login, and today's units are written into the runtime
directory, which is emptied at logout, so nothing of the wrapper ever started at
login either.

**Removal of what a removed worktree leaves.**
Kept: the service that finds its own worktree gone stops itself at the first
connection (`stop_own_socket_when_the_worktree_is_gone`, entry 32).
Replaced: the sweep stops asking the manager. It removes the instance's socket
file when nothing listens on it, removes the pid note when its process is gone,
and removes the journal only when the pid note proves the service is gone (the
pid and start time no longer match a live process), which replaces
`service_is_gone_after_a_stop`'s question to the manager.

### How the requirements are held

**A commit never fails where real Git would succeed.**
There is no state in which "no service can be started" any more: the wrapper
starts the service itself, so the failure that used to say "this account has no
user service manager" is gone. The one failure left is a service that does not
greet within `START_WAIT`, and its message names the log file, the socket path
and the command that removes what is left.
Nothing that used to work through a manager is left unbuilt: a machine with a
manager runs the same base.

**Nothing waits without a bound.**
The start lock is waited for inside the start's bound; the greeting is waited
for `START_WAIT` (10 s, unchanged, 9.6 times the slowest measured cold start of
1,041 ms) with a notice at 1 s; the process group of a wedged service is given a
bounded time to end before the start; the client's own answer, withdrawal and
waiting-notice bounds are unchanged. The log cap bounds no wait.

**No command waits on another agent's command.**
The lock covers one worktree's start and is released at the greeting, so a
commit in one worktree never waits on a commit in another, and a commit never
waits behind another commit of its own worktree except through the service's
queue, which is the design.

**A process can be killed at any instruction.**
The socket file of a killed service is stale and replaced by the next bind; the
pid note of a killed service fails its start-time check and is removed; the
start lock dies with its holder; the journal's records are restored at the next
start; a request whose answer was lost is found in the journal.

**A commit takes no more than five seconds longer than real Git's.**
The fast path adds one `connect`, one line read and one `close`, measured in
part 7. The first commit of a worktree pays one start, which the manager path
also paid.

**Coordination is free of locks first.**
The socket bind is the only ownership. The start lock is the one lock, it
covers the smallest unit that can be covered (one start of one worktree's
server), and its reason is written beside it.

**The code cleans up after itself.**
The service removes its socket and pid note as it ends; the sweep removes what a
killed one left and the journal of a worktree that is gone; the log is capped;
the start lock is empty and released by the kernel.

## Part 4: whether a service manager may still be used when one is there

Decided on 2026-11-12 and recorded in
`doc/handover/cli-git-rust-implementation.md` under "Current state":
"Progressively enhance. You can also test on Alpine."

What that settles: the base is the wrapper and its service working whole with no
service manager, and everything a commit depends on is in that base. Where a
manager is present the wrapper may use it to do better, as an enhancement on top
of the base and never as a condition of it. An enhancement must leave the base's
behaviour intact whenever the manager is absent, slow, refuses or answers with an
error, with no failure and no message a person has to act on; finding out whether
a manager is there costs nothing a commit waits for and is bounded; no enhancement
writes a unit file or asks for a reload; and taking the enhancement away changes
nothing a person or a test can observe except the thing the enhancement gives.

### The enhancement: a transient scope for the server

Bazel's answer, adopted whole: the server is started as a transient scope of the
user manager, so it gets a cgroup of its own and is neither killed with nor
charged to the shell that started it.

What it costs, from systemd's own source and documentation
(`systemd/systemd` at commit `92e51dd8a52c19ab56388d02fceeb8c7ae851beb`,
fetched 2026-11-12):

- **One D-Bus method call per start, and no unit file and no reload.**
  `systemd-run --scope` creates the scope with `StartTransientUnit`
  (`src/run/run.c` lines 1997, 2620 and 2813) and then runs the command as its
  own child, synchronously, in the caller's environment
  (`man/systemd-run.xml` lines 76 to 81). A transient unit is not a file under
  the unit search path, so nothing has to be reloaded, which is the one property
  that matters here: the 2026-11-12 incident was a reload.
- **A call to a manager that is re-reading its units waits as long as the reload,
  or fails at the client's timeout.** `Reload()`'s reply is deliberately not sent
  until the reload is finished ("we just remember that we need to and then send it
  after the reload is finished. That way the caller knows when the reload
  finished", `src/core/dbus-manager.c` lines 1726 to 1734), and the reload runs
  in the manager's own main loop (`m->objective = MANAGER_RELOAD`, line 1734;
  `manager_reload` in `src/core/manager.c` line 3961). A call that arrives while
  that loop is busy is dispatched afterwards, and the client's default method
  timeout is 25 seconds
  (`BUS_DEFAULT_TIMEOUT`, `src/libsystemd/sd-bus/bus-internal.h` line 327), so
  the caller either waits up to 25 s or waits as long as the reload, whichever is
  shorter. Measured here on 2026-11-12: one reload took 139,770 ms.
  So the enhancement may never be on a path a commit waits on, and its own bound
  must be far below 25 s.
- **The scope's lifetime is the command's.** `systemd-run --scope` returns only
  when the command finishes (same man page lines 78 to 81), so the starter
  process is the scope's client for the server's whole life. It must survive the
  wrapper's exit, which `setsid` already gives it.
- **The probe is Bazel's answer to "is there a manager".** Bazel runs
  `systemd-run --user --scope -- /bin/true` and only uses `systemd-run` when that
  exits zero (`src/main/tools/daemonize.cc` lines 281 to 303).

The design's shape, then:

1. When the base would start a server, and only then, the starter probes with
   `systemd-run --user --scope -- /bin/true` under a bound of 2 seconds. A
   missing program, a non-zero exit, a timeout or a signal all mean "no
   enhancement": the server is started plainly, exactly as the base does.
2. On a successful probe the server is started through
   `systemd-run --user --scope -- <own executable> service run ...`, under the
   same bound. If the greeting does not come within the base's start wait plus
   one second, the starter kills the whole group of what it started and starts
   the server plainly, once, so a manager that became busy between the probe and
   the call costs one second and never a commit.
3. Nothing else changes on any path. No unit file, no reload, no `systemctl`, no
   `ResetFailedUnit`, no `RestartUnit`, no `ActiveState` question. The scope is
   named by `systemd-run`'s own default unless a name is needed for a person to
   find it; if a name is given it is derived from the instance, and it is not a
   name anything waits on.

### What else a manager gives, and whether it is worth taking

- **A cgroup of its own.** Taken, as above. It is the one thing the base cannot
  give at all (part 3, "What is lost").
- **Restart of a server that died.** Not taken. The next command restarts it,
  and a waiting client restarts it itself within its bound, so a restart by the
  manager would only race the client that is already doing it. Bazel's client
  does not ask the manager to restart its server either.
- **Logs in the journal.** Taken without the manager's units: the service sends
  its entries to journald's native socket whenever that socket exists, and to
  its own bounded file always. The journal's own rotation and retention then
  apply beside the cap.
- **A stop that kills what is left after a timeout.** Not taken. The service's
  own `SIGTERM` handling and its stop bound (`stop_of_the_service_noticed_by_its_children.rs`)
  already end it within a known time, and with a scope the whole group can be
  signalled by the pid note.
- **Start at boot or at login.** Not taken: the owner's rule is that nothing of
  the wrapper starts at login.

### Both sides, and the recommendation

For taking the enhancement:

- it gives the service a cgroup of its own, which is exactly the loss that
  matters most on this machine (a memory cap shared with an agent's shell, and a
  kill that ends the whole cgroup);
- it costs one probe and one D-Bus call per server start, both off the fast path,
  both bounded by the starter, and both with a plain fallback;
- it writes no unit file and asks for no reload, so the incident's mechanism is
  not reintroduced;
- Bazel does the same, with the same fallback.

Against taking it:

- it is a second way to start the server, and a second way is a second thing to
  test, to document and to break;
- a scope started by a busy manager can delay a first commit by the bound before
  the fallback, and a commit's first start is the one a person waits for;
- the base's answer (the low-memory exit, the small bounded service, and a
  durable journal) already keeps the loss small.

Recommendation: take it, with the probe and the fallback as described, because
the cgroup is the one thing the base cannot give and the cost is bounded and off
the path a commit waits on. It should be switchable off without changing
anything a person can observe besides the cgroup, which is what its two tests
assert.

## Part 5, what was built, in steps

Each step is a commit on branch
`refactor/git-policy-cli-commit-service-is-started-by-the-wrapper-itself-and-needs-no-service-manager`.

1. `179c723a98` the base: the route, the commit's start, the client's wait, the real
   Git recorded beside the sockets, the sweep and the removal hook, and the removal of
   the unit templates, the provisioning, the unit-text record, the earlier single
   service's retirement, the two-runtime-directory refusal, the D-Bus client and the
   `zbus` dependency.
2. `403b3a9624` the new modules and their controls: the start
   (`service_server_start_by_the_wrapper.rs`), the detached process
   (`service_server_start_spawns_a_detached_process.rs`), the note that names the
   process (`service_server_pid_note_of_an_instance.rs`), the capped log
   (`service_server_log_file_of_an_instance.rs`), the private folder and socket
   (`service_socket_directory_only_this_account_may_open.rs`), the files a running
   service owns (`service_server_files_of_a_running_instance.rs`), the host
   (`service_host_of_a_worktree.rs`), the commit's start
   (`service_commit_starts_the_server.rs`), and the user-boundary controls
   (`binary_service_starts_tests.rs`).
3. `7db1d00785` the journal keeps every service line beside the file;
   `SPEC.md`, `README.md` and two source guards rewritten.
4. `8660c2e575` a bind takes an address over from a socket that accepts and greets
   nothing, which is a service that is exiting.
5. `4f63e44f7f` the controls of the new modules: a short folder for a socket path, the
   session field read at the right offset, a greeter thread that ends.
6. `89cf84f069` the controls that named the manager name what replaced it;
   `SERVICE_GREETING_SILENCE` is gone with the restart it bounded.
7. `3164c7225e` the fallback control names the record beside the sockets.
8. `e151a68539` two defects the user-boundary controls found: the real Git was never
   recorded, and a start that could not create its marker was called another
   command's start.
9. `be51f5894b` a commit lands on a machine that names no runtime directory.

### The interface three of these changed

- **A start takes the address over from a socket that greets nothing.** The old bind
  refused any socket a process held. A service of this release greets before it
  restores anything, so a socket that accepts and says nothing is a service that is
  leaving or stopped inside itself; waiting for it cost a commit 37 s on 2026-11-12.
- **`SERVICE_GREETING_SILENCE` is gone.** It bounded the repair of a unit a manager
  called `active`; with no manager there is no such state. A service that greeted and
  then went silent is read against `HEARTBEAT_SILENCE` and the command starts a fresh
  server itself.
- **`XDG_RUNTIME_DIR` is optional.** A container, a service's own child and a cron job
  set none, and real Git commits there. A command that names one uses it; one that
  names none keeps its sockets in `~/.run`, made private at every start; an account
  with neither names nowhere and says so rather than guessing a folder that is not
  its.

## Part 6, the proofs

### The base needs no service manager: an Alpine container

Image: `alpine:latest`, built by `Containerfile.alpine-wrapper-build` from the Alpine
packages `cargo`, `rust`, `git`, `musl-dev` and `build-base`, so the wrapper in it is
musl-linked and needs no glibc: `ELF 64-bit LSB pie executable, x86-64, dynamically
linked, interpreter /lib/ld-musl-x86_64.so.1, stripped, 25,698,392 bytes`. The build
inside the image took 15 m 27 s.

The run (`podman run --rm --cpus 4 --memory 4g --pids-limit 1024`), with the wrapper
installed as `/usr/local/bin/git` ahead of the real Git at `/usr/bin/git`, and with
`XDG_RUNTIME_DIR` and `DBUS_SESSION_BUS_ADDRESS` removed from the environment:

- `pid 1: sh`; `systemctl`, `systemd-run`, `dbus-daemon` and `systemd` all `none`;
  `/run/systemd` and `/run/dbus` absent; `XDG_RUNTIME_DIR: <unset>`;
  `DBUS_SESSION_BUS_ADDRESS: <unset>`.
- commit one, through the wrapper, with no service running: **80 ms**, subject
  `first`.
- commit two, which must find the service commit one started: **30 ms**, subject
  `second`.
- commit three, in a **second worktree of the same repository**: subject `third`, with
  its own service, so one server per worktree and none shared.
- the files it left: `<instance>.log`, `<instance>.pid`, `<instance>.sock` and
  `real-git` under `~/.run/monochromatic-git-policy-cli/`; **no unit file anywhere**
  under `HOME`.
- the two servers, from `ps`, each with exactly the arguments the design names:
  `--path …/f6b2….sock --journal ~/.local/state/monochromatic-git-policy-cli/service/f6b2…
  --git /usr/bin/git --instance f6b2…`.

The output is kept whole as `proof-alpine-no-manager-output.txt` beside this run's
report, with the script (`alpine-proof-no-manager.sh`) and the Containerfile.

**A finding of its own**: Alpine's Git is 2.54.1, below the 2.56.0 this wrapper was
tested with, and the wrapper says so on standard error and commits anyway. Nothing in
the wrapper or its service failed to build or behave on musl.

### A manager that received no call

Run, in a container, in the run after this one: see "The manager half of the proof, in a
container". It shows three cases side by side with their counts: the manager of this
session's own bus is asked twice for one service start and never again; a bus of another
directory than the runtime directory is asked nothing; and with no bus at all nothing is
asked. No wrapper command makes a single D-Bus method call.

## Part 7, the times

`monochromatic-git-policy-cli 0.6.1-alpha.16` installed, against a release build of
this branch, alternating on the same fixture shape, a fresh worktree each round so the
first commit starts a service and the second finds it. Milliseconds:

| round | installed first | mine first | installed later | mine later |
|-------|-----------------|------------|-----------------|------------|
| 1     | 60              | 140        | 50              | 30         |
| 2     | 160             | 90         | 30              | 30         |
| 3     | 80              | 80         | 40              | 30         |
| median| 80              | 90         | 40              | 30         |
| spread| 60 to 160       | 80 to 140  | 30 to 50        | 30 to 30   |

A first commit is 10 ms slower at the median and 80 ms slower in the worst round,
inside the two arms' own spreads of 60 to 100 ms; a later commit is 10 ms faster at the
median. The requirement ("a first commit must not be slower than today's by more than
the spread") holds on this sample, which is small: three rounds an arm, on a machine
with four agents and no build running.

The installed release ran in this machine's own environment, because it cannot commit
at all without its service manager: that is the defect this branch removes, and it is
why the two arms cannot share an environment.

## The enhancement, built

`service_server_start_through_the_manager.rs`, wired into
`service_server_start_by_the_wrapper::start`:

- **The gate.** A manager is asked only when the environment named the runtime directory
  and the inherited bus resolves to a socket inside it (`bus_is_ours`). A folder this
  program fell back to (`~/.run`) is no session, and a bus of another directory is
  another session's manager, so neither is asked: that is issue 6293's rule, held by the
  same function that keeps a private runtime directory out of the account's manager.
- **The probe.** Bazel's: `systemd-run --user --scope --quiet -- /bin/true`, a plain child
  bounded by `PROBE_WAIT` (2 s). A refusal, a missing program, a bus that is not there and
  a manager still thinking all answer `false`, and the base's start runs.
- **The start.** `systemd-run --user --scope --quiet -- <the base's own words>`, given
  `START_WAIT` plus one second; a scope that made nothing answer is ended by its group and
  the base's start runs once, so a manager that made a scope and then failed costs a second
  and never a commit.
- **Controls.** Four of the gate, the probe's bound and the start's words; and the two the
  decision asks for at the user boundary: with a manager of this session the service is
  started as its scope (the record holds the probe and the start, with the base's words
  after `--`), and with a manager that does not answer in time the result is exactly the
  base's (one server, the same files, no `service run` in the manager's record).

## The retire storm, fixed

The audit of alpha.14 measured a client sending `retire` a hundred times in ten seconds to
a healthy service of a newer release. Two halves:

- **The client** (`service_release_order.rs`): a service of this protocol is used whatever
  release it announces, and only a *lesser* one, or one whose release cannot be read, is
  asked to leave. The protocol is the contract; the release check existed because the
  manager started the unit's program, which an upgrade could remove. A command served by
  another release says so in one line.
- **The service** (`service_server_connection_handling.rs`): one ask per service life.
  A second ask from any client is refused with a line and queues nothing, where before every
  ask queued a job of its own. The retire message now reads the version its client
  announces, which is what the refusal names.

The control is one client, one newer service, three commands: no ask honoured, the same
service serving all three, each command naming the release that served it.

## The manager half of the proof, in a container

`manager-proof-in-a-container.sh` and `Containerfile.manager-proof`: a Debian image with
Git, a real `dbus-daemon` on the container's own bus, and a stand-in named `systemd-run`
that records every word it is given and then does what `--scope` does. Three cases, one
`podman run --cpus 4 --memory 4g --pids-limit 1024`:

- **A, the bus is this session's own** (`XDG_RUNTIME_DIR` = the bus's directory): **2**
  manager calls, the probe and one start
  (`--user --scope --quiet -- /usr/local/bin/git monochromatic-git-policy-cli service run …`),
  and the later command added none. Both commits landed.
- **B, a runtime directory of the command's own, the bus inherited from another
  directory**: **0** manager calls, both commits landed, the service's files in the private
  directory, and its state under the container's `HOME` (which is what issue 6293 will move).
- **C, no bus and no runtime directory**: **0** manager calls, both commits landed, sockets
  in `~/.run`.
- The bus's own log: 2 method calls for a control call of my own (so the log can show one),
  and **0** from any wrapper command: the wrapper never speaks D-Bus, its only manager
  interaction is the manager's own program, as Bazel's is.

## Times, and the CARGO_HOME of the run

The commit times of the previous run stand (a first commit 10 ms slower at the median,
inside the arms' spread; a later one 10 ms faster). Nothing in these three changes touches
the fast path: the gate reads two environment values and one `PATH` scan, only on a start.

A `CARGO_HOME` of the run's own, with `registry` and `git` linked to the shared ones, did
keep every cargo command off the package-cache lock: a clean `cargo check` took 67 s with no
"Blocking waiting for file lock" line. It needs one thing `~/.cargo/config.toml` also holds:
the `build-dir` setting. Without it, builds land in the worktree's own `target/`, and a
fixture made under `target/` is then *inside* this repository, which made
`push::auto_push_trigger::tests::every_decline_names_its_reason` fail for a reason that has
nothing to do with the code. With the line copied, the whole set is green.

## What is left, in the order I would take it

1. **`git monochromatic-git-policy-cli service stop`**: a person cannot ask a worktree's
   service to leave by hand, and the push deadline advice still names the manager's
   `set-environment`.
2. **The unit files of earlier releases** stay where they lie. Nothing reads them, but
   nothing removes them either.
3. **Bazel's low-memory idle exit** is designed and not written.
4. **A retire from a client of another *protocol*** is still honoured, which is deliberate:
   such a service cannot serve that client at all, so replacing it is the only useful answer.
   Its remaining cost is that the client must start its own service, which it now does
   rather than failing.
5. **The package's `bin/` tooling and `native:gate*` tasks** still use this host's manager
   for cgroups. They never reach a release.
6. **The six library controls that fail in files this branch leaves byte-identical to its
   base** (two `worktree_landed_removal`, two `worktree_copying`, `policy_checks`'
   overwriting forms, the snapshot-rung assertion in `wait_ceilings_tests.rs`, and the
   blocking-code guard's `lazy_runtime.rs` lines). They are somebody else's to fix; each is
   named with its file, and `git diff 9d5a7b0ed7..HEAD` prints nothing for those files.

## The third run: the two-release control, its defect, `service stop`, and the earlier units

### The two-release control, and the defect it found

Two release builds of this branch across the merge: `0.6.1-alpha.13` (commit `2fb2e61d90`)
and `0.6.1-alpha.16` (the tip). The script alternates them over three commands each way, in
one worktree whose service is the *other* release's, which is the state in the hours after
an upgrade.

What it showed first was a **defect, not a limit**: an older client meeting a newer service
made **0 of 3** commits, each ending "The commit service refused this commit because it
names a path or a value that cannot be committed". Protocol 12 (`eb27b494c7`, main) added a
message (`Shadow`/`Drawn`) and an optional field, so the newer service serves the older wire
unchanged; what blocked the old client was the *numbers*: the greeting named 12, and an
installed client refuses a version above its own, and the decoder demanded exactly 12 of a
request.

Both are fixed in the release that is installed last:

- `MIN_SERVED_PROTOCOL` (11) is what the service announces, and what the decoder accepts in a
  request, with the rule for raising it beside it. A message this build does not know is
  still refused, which is how the older client's `Shadow` question already arrives as no
  answer.
- The client accepts any version in `MIN_SERVED_PROTOCOL..=PROTOCOL_VERSION`.

After the fix, the same script:

- older client, newer service: **3 of 3** commits, **0** asks to leave, and every command
  saying which release served it;
- newer client, older service: **3 of 3** commits, **1** ask, and the older service
  replaced.

So a machine in the hours after an upgrade commits through whichever release's service is
running, and the newer release's service is asked to leave only when it is the older one.

### `service stop`

`git monochromatic-git-policy-cli service stop` asks this worktree's service to finish what
it holds and leave, over its own address, and waits `START_WAIT` for the address to be free.
Four controls: a service that answers is asked and leaves; an address nothing answers on is
nothing to stop (exit 0); a Git directory that cannot be resolved stops nothing and names
the cause; an account with neither a runtime directory nor a home is told so.

### The earlier releases' units

An earlier release started the service from a unit, and the manager's socket unit keeps
`<instance>.sock` bound for as long as that unit is loaded. Two things answer it:

- **The bind takes the address over** from a socket that accepts and greets nothing, which
  the manager's socket is until it has a service to hand the connection to. That is why the
  upgrade works on a machine with no manager at all.
- **`service_server_earlier_release_units.rs`** removes the earlier release's unit files and
  notes folder (only files carrying this program's marker), and stops this worktree's socket
  unit, the earlier per-worktree one and the earlier single service's, with the manager's own
  client, bounded by `PROBE_WAIT`, and only when the runtime directory is the one the
  environment named and the bus is that session's own. It runs once, at the first service of
  another release a command meets.

Shown in a container with no manager of any kind (`earlier-release-units-proof.sh`): with a
manager's client answering, the files go, `--user stop monochromatic-git-policy-cli@<instance>.socket
cli-git@<instance>.socket cli-git.socket` is asked once, and this release's service owns the
address; with no client at all, nothing is asked, the files still go, and the commit is still
served by this release.

### The cost on a commit's path

Five rounds an arm, alternating, a fresh worktree each round:

- first commit of a worktree (a service must start): 120, 80, 80, 90, 90 ms before;
  120, 90, 90, 90, 80 ms after. Median 90 both.
- later commits: 40, 40, 40, 30, 30 and a second arm 50, 40, 40, 50, 40 before;
  40, 30, 40, 40, 30 and 40, 30, 50, 60, 30 after. Median 30 to 40 before, 40 after.

Nothing of this run's changes is measurable above the 20 to 30 ms spread: the gate reads two
environment values and one `PATH` scan, the earlier-units sweep does one read of a directory
that is usually empty, and the release order is one comparison of two short texts.

### What is left, in the order I would take it

1. **`git monochromatic-git-policy-cli service stop` for another worktree.** It stops the
   one it runs in.
2. **Bazel's low-memory idle exit** is designed and not written.
3. **The package's `bin/` tooling and `native:gate*` tasks** still use this host's manager for
   cgroups; they never reach a release.
4. **Controls of main's that fail in this tree and are not this branch's**: the heartbeat one
   in `service_server_greets_before_the_journal_is_restored_tests.rs` (it fails identically
   with this branch's greeting change put back) and `service_round`'s panic lane (its files
   are byte-identical to main). Both want their owners.

## The review of the takeover, and what answers it

The review of `53726ade2d` reproduced one defect with the real binary: a service that was
alive and silent for a second lost its address to a second server, and the first one's exit
then unlinked the second's socket and note by their paths. Two live services for one
worktree; the note, which proves a service gone, missing while a service ran.

Both halves are fixed, and neither depends on the other:

- **Silence is never proof that a holder is gone.** `bind_path` takes over only a socket the
  kernel refuses a connection to. What a silent socket means is "a live service that is
  slow", and the three ways that happens are recovery before the acceptor starts, a starved
  acceptor, and a debugger. The follow-on is what makes it safe: `start` starts no server
  while something accepts at the address and returns the wait the caller keeps, bounded by
  its own start ceiling. So a slow recovery costs a wait, never a commit.
- **A service removes only what is still its own.** `OwnFiles` removes the socket only while
  the path still names the inode it bound, and the note only while it still holds the text it
  wrote. Two removals in the sweep were by path as well: `remove_what_a_stopped_instance_left`
  now needs the note's death *and* the kernel's refusal, and `stop_own_socket` needs the note
  to name this process.

Three controls, each shown failing without its guard:

- `a_socket_that_accepts_is_left_to_its_service_and_a_refused_one_is_taken_over` — without
  the fix, `bind_path` returns a listener where it must return `AddrInUse`.
- `a_service_leaves_files_that_are_not_its_own` — without the fix,
  "the later server's address is left where it is" fails.
- `a_command_waits_at_an_address_that_accepts_and_greets_nothing` — without both guards,
  the start returns `DidNotGreet` after a start wait where it must return at once.
- `a_live_address_whose_note_is_stale_is_left_alone` and
  `a_service_removes_only_the_note_that_names_it` — the sweep's two guards.

The review's reproduction, run again against the fixed build: B refuses to take the silent
service's address (`cannot listen: Address already in use`) where it had taken it before, so
the wrong result it printed (`REPRODUCED: B is running but A's exit removed B's socket`) is
gone. What a person sees instead is a commit that waits for the service that is there and
lands through it (`service-silent-commit-waits.sh`: a commit waits 3.1 s through a stopped
service, lands, the same pid serves it, one service was started in that worktree).

### Every place this change decides from a deadline alone that another process is gone

- `bind_path`: **no deadline**. A takeover requires the kernel's refusal; a silent socket is
  left and the command waits. (Was a one-second greeting read; that was the defect.)
- `start`'s wait for a greeting: a deadline, but it decides only that *this* command stops
  waiting for a server it spawned; it removes nothing and kills nothing, and a failure names
  the log. When something still accepts at the address it does not fail at all: the caller
  keeps waiting to its own ceiling.
- `service_server_start_through_the_manager::end_the_group_of`, and the probe's kill: a
  deadline ends a process group **this command started**, never another's.
- `service_is_gone_after_a_stop`: **no deadline**. The note's process birth identity is the
  proof, and a note that names a live process means the service is there.
- `remove_what_a_stopped_instance_left` and `stop_own_socket`: no deadline; the note's death
  and the kernel's refusal, or the note naming this process.
- `management_service_stop::stop` and `service_server_earlier_release_units`: a deadline ends
  the *asking* (`START_WAIT`, `PROBE_WAIT`); neither unlinks another process's file, and both
  say what they found.
- `service_client_reach_service`'s `Heard::Silent if service_seen` (`HEARTBEAT_SILENCE`, 5 s):
  a deadline that ends the *wait*, and the failure it reports says the outcome is not known
  (`ServiceSilent`) rather than claiming the commit was not made; the request may still land,
  so the words point at `git log` and `git status` first.
