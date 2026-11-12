# The commit service without a service manager

Status, 2026-11-12: part 1 done (the inventory).
Parts 2, 3, 4, 5, 6, 7 and 8 are added below as each is finished;
this line is rewritten each time.

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
