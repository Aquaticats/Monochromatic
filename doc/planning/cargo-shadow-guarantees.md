# What cargo-shadow promises and what it assumes

Status:
written 2026-11-04T04:45:55-05:00 after the round 7 re-review (`cargo-shadow-sol-7a.txt`, `-7b.txt`, `-7c.txt`, all "not closed"),
amended after the cross-vendor review of this contract (`cargo-shadow-sol-contract-1.txt`, "fit after the named edits"), which also
supplied the acceptance list below.
This file specifies transaction safety (acceptance, rollback, kept copies, recovery).
It does not specify the policy semantics, which are the catalog and pnpm specification:
`doc/decision/rust-dependency-catalog.md` (what is checked, and when) and `doc/planning/cargo-shadow-update.md` (the mapping of
`cargo update` to `pnpm update` and the pnpm oracle).
The assumptions and the weaker-path rule are the coordinator's picks under the owner's delegation
("Pick options as if I had picked them and invite veto when I say I'm back", handover "Picks made under the owner's delegation"),
veto open.

Each round the reviewer moved the adversary one level deeper
(a thread, a mount view, a same-user process that leaves its group)
because no written contract said what the wrapper promises and what it assumes.
A finding that attacks a stated assumption is answered by the assumption;
a finding that shows a stated guarantee failing is a bug.
Where this file and an earlier section of `doc/planning/cargo-shadow-update.md` disagree, this file is current.

## Guarantees

Each guarantee is a property, followed by the test that shows it.
"Exists" names a test in the package now;
"to be written" is a test round 8 writes before the code that satisfies it.
Every guarantee is stated for the normal path, where a delegated control group is in force.
The weaker path (third part) keeps only the guarantees marked as holding there.

### Acceptance of a lockfile

- **A1.**
  A lockfile that a run left is accepted (left live, journal finished, command allowed to run) only after the wrapper has shown
  that every process the run started has stopped: the direct child was reaped, and either the kernel reported `populated 0` for
  the run's scope, or the scope is verifiably absent.
  Verified absence is proof under amended assumption c and one invariant of the manager:
  systemd removes a scope only when it is empty or after stopping its members
  (probed: a scope whose program exited stayed observable with `populated 1` while a descendant lived, and vanished 2991 ms later
  when that descendant ended).
  Absence is verified when the cgroup file system is the one the run recorded (assumption c), the directory that held the scope
  still exists with the recorded mount id and inode, and the scope's name is absent from it.
  A scope name carries a 128-bit random token and is never derived from the process id or the clock, so no journal that can
  reference a name can see another scope appear under it.
  Test: `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns` and
  `a_writer_in_another_session_is_stopped_before_a_normal_exit_returns` exist;
  `probe_2_a_thread_group_with_an_exited_leader_stays_populated_until_it_is_stopped` exists.
  Holds on the weaker path: no (third part).
- **A2.**
  Whenever a run could not show A1 (a member that cannot be killed, a failure to stop, any failure on the containment path after the
  child started), the run reports `NotQuiescent`, the scope reports that writers may remain, and install and update stop with the
  lockfiles neither rolled back nor accepted, the journal kept, and the user's command not run.
  Test: `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal` exists (call-site half);
  `a_failure_on_the_containment_path_after_the_start_is_not_quiescent` to be written (7a F3: today a failure to leave the group
  returns an ordinary I/O error).
  Holds on the weaker path: yes for what the process group shows.
- **A3.**
  A lockfile is accepted only if the policy accepted its final text, or the refusal stops the command.
  A mandatory failure of finishing (outputs not durable, completion marker not durable) stops the command and is never hidden behind
  an earlier error, including an operational one that would let the shim run the command again with `--locked`.
  Only the removal of a journal that is already marked done is a warning.
  Test: `an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`,
  `a_refusal_survives_a_journal_that_cannot_be_finished`,
  `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics`,
  `a_completion_marker_that_cannot_be_published_stops_the_command`,
  `the_cleanup_failure_happens_after_the_completion_marker_was_published`,
  `an_update_error_with_a_failed_mandatory_phase_reports_both` exist;
  a test that fails the real rename and the real parent sync of the completion marker (not a failpoint before them) is to be written
  (7b P3).
- **A4.**
  The outputs a run leaves (lockfiles, journaled manifests) are on disk before the completion marker, the marker is durable before
  the journal is removed ("Force at leaving").
  No periodic or background syncing is required ("else background" has no carrier in a command that exits);
  every durability barrier the protocol mandates remains required:
  the outputs, the completion marker and its directory, the publication of a record before the child it describes, the entries of
  a kept copy (K2), and an acknowledgement (weaker path).
  Test: `finish_makes_the_outputs_durable_before_the_done_marker` exists.

### Rollback

- **R1.**
  A rollback replaces a lockfile only when it still holds what this run produced.
  A file another process wrote meanwhile is left alone with a warning.
  Test: `a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning` exists.
- **R2.**
  No inode that was live at a path loses its last name: the displaced file and the replacement each get a surviving name before
  anything is unlinked, and a displaced file that could not be linked is reported with where it is.
  Test: `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live` exists;
  the hard-link fault matrix (`EMLINK`, `ENOSPC`, `EXDEV`) is queued for round 8.
- **R3.**
  A rollback that cannot complete stops the command (`Unsettled`) whether cargo failed or its output was refused,
  so the `--locked` fallback never builds an unrestored lockfile.
  Test: `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions`,
  `a_cargo_that_wrote_a_lockfile_and_failed_has_its_output_rolled_back` exist.
- **R4.**
  A rollback or an acceptance never happens while writers may remain (A2).
  Test: as A2.
- **R5.**
  A failed root does not hide the warnings of the others and does not stop their restore.
  Test: `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others` exists.

### Kept copies

- **K1.**
  Kept copies are never deleted by the tool.
  Their existence, count or size never causes a refusal;
  past a count every run prints the count, the total size and the folder.
  The inability to preserve a newly displaced file safely still stops the operation (R2, R3).
  Test: `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses` exists;
  the inventory of kept names beside their target and of stranded temporaries is queued for round 8 (7b P2).
- **K2.**
  A kept copy's directory entries are durable when preservation reports success (assumption b below).
  Test: `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction`,
  `a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable`,
  `a_failing_directory_barrier_is_not_reported_as_preserved`;
  the bind-mount schedule of 7b P1-1 is not a counterexample under that rule and has no test of its own.

### Recovery

- **V1.**
  A recovery or a new transaction proceeds only when the whole set of journals of the workspace was listed:
  an entry that cannot be listed or examined stops it, and an entry that vanished mid-listing is unknown, not absent.
  Test: `a_journal_listing_that_fails_stops_the_recovery`, `a_journal_listing_that_fails_stops_a_new_transaction` exist;
  `an_entry_that_vanishes_while_the_journals_are_listed_stops_a_new_transaction` to be written (7b P1-2: `begin` today turns an
  entry-level `NotFound` into "no earlier journals"), with the failpoint placed inside the iteration, not before it (7b P3).
- **V2.**
  A recovery or a new transaction does not proceed while any journal of the workspace names a recorded group with a member,
  a record it cannot read or understand, a lost record beside other files, or a record made where the recorded group cannot be
  examined (assumption c).
  Test: `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`,
  `a_survivor_of_one_journal_stops_the_recovery_of_another`, `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor` exist;
  the crash test is to be strengthened to assert the refusal reason and journal retention (7a test notes).
- **V3.**
  A record is durable before the child it describes starts, and a child whose record cannot be written is not started.
  Test: `a_child_whose_group_cannot_be_recorded_is_not_started` exists.
- **V4.**
  A journal that holds nothing at all is what a cut-off `begin` leaves, and a recovery removes it.
  Test: `a_recovery_removes_an_empty_journal_directory` exists.
- **V5.**
  A recovery restores per root, never splits a lockfile from its manifests, saves a refused leftover as `Cargo.lock.refused`,
  and finishes with the same three phases as A3.
  Test: the existing recovery tests of `journal_recover_tests.rs` and `journal_protocol_tests.rs`.
- **V6.**
  Every wait that cargo-shadow controls has a deadline:
  the start and the stop of the manager's scope, the lifetime of a command, the draining of its pipes, the reaping of its direct
  child, and every lock (a busy workspace lock or internal lock returns at once with the holder, it does not wait).
  A shutdown that did not succeed within its deadline produces uncertainty (`NotQuiescent`), not a longer wait.
  Ordinary blocking kernel and file-system calls (a `read`, `write`, `fsync`, `rename`) have no deadline of their own;
  they are covered by the progress assumption d below, which mirrors the owner's "Assume the final disk save would never be cut
  short."
  Test: `a_child_that_outlives_its_bound_is_stopped_and_named`, `recovery_defers_while_a_live_transaction_holds_the_workspace`
  exist.

### What no guarantee covers

- A process that deliberately acts against the wrapper (assumption a).
- Power-loss behavior of a file system that does not honor `fsync`; the package has no fault-injection model of power loss.
- The window in which a cargo that is not the shadow builds on an unapproved live lockfile (accepted earlier, "Use as few locks as
  possible").
- The user's own command after the checks: it runs as long as the user lets it.

## Assumptions

Each assumption is a pick under the delegation, open to the owner's veto.
A finding that attacks one of them is not a finding while the assumption stands.
Assumptions the code already made are listed last, so the list is finite.

### a. Run writers stay inside the containment and write the workspace only through it

The processes of a run (a build script, a proc macro, a selected executable) are assumed to behave in this way, whether the
reason is malice or an accident:
they do not migrate outside the recorded containment (no write of their own id into another group's `cgroup.procs`, no request to
a manager that moves them), and they do not arrange writes to the workspace through a service outside the containment
(a daemon started by another user session, a job queued with an outside scheduler, a request to a build server that already runs).
Ordinary behavior stays covered:
`fork`, threads, double forks, `setsid`, closed standard streams, and a descendant that outlives its parent are all inside the
scope and are stopped with it.
The manager is assumed not to migrate surviving members of a scope outside the recorded scope either.

Reason:
a same-user process can write `Cargo.lock` directly after the run, and no wrapper can prevent that, so a containment that tried
to resist it would claim a guarantee the wrapper cannot give.
The assumption is about behavior, not intent:
a tool that hands the write to an outside service breaks it as surely as one that means to.
Probe 3 of round 7 (`probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_the_stated_limit`) shows the move is possible
and stays as the evidence of the limit.
This retires 7a F5 and 7c F2 as findings, and the cgroup namespace closer as work.

### b. A pre-existing directory and its path are durably reachable

The directory-entry sync of a kept copy walks up from the kept directory to the first directory that existed before the
transaction began, syncs it, and stops.
A directory this transaction created is synced with the directory that lists it, as `create_directories_synced` does.
A pre-existing directory, and the reachability of its path from the root of its file system, are assumed durable:
their entries were committed by whoever made them, or by an earlier transaction of this tool, which synced what it created.

The exception is knowledge.
A directory that the tool knows it left unsynchronized in an earlier failed attempt (a directory whose own sync, or whose parent's
sync, returned an error) is not treated as durable merely because it exists now;
the walk continues through it and syncs it.
The tool knows this for the lifetime of its process (a registry of the directories whose sync failed).
It does not know it across processes: a directory left unsynced by a killed or failed earlier process is a pre-existing directory,
and the premise applies to it.

Reason:
the tool cannot know the durability history of a directory it did not make, and walking to a device boundary or to `/` rests on
a different, unprovable premise (a bind mount, a subvolume, an overlay root).
The rule replaces the round 7 stop at the first parent on another device (7b P1-1).
It also removes the failure of preservation on an unreadable ancestor above the workspace.

### c. The journal and the recorded containment resolve to the same mount and inode in recovery as in the run

When a journal is created, the wrapper records the mount id and inode of the journal directory (`statx`, `STATX_MNT_ID` and
`STATX_INO`).
When a child is recorded, the record also holds the cgroup namespace and the mount id and inode of the directory that holds the
recorded scope (its parent in the cgroup file system).
A recovery compares both with what the same paths give in the recovery process, and refuses, naming the journal and the view that
differs, when anything differs.
Only when they are equal is a recorded scope that is absent read as gone.

Reason:
a recovery in another mount namespace, or on a path that now leads somewhere else, cannot read an absent recorded scope as "gone"
(7a F6); the 7a F6 schedule is a different view of `/sys/fs/cgroup`, not of the journal, which is why both checks exist
(the coordinator adopted the second check on 2026-11-04).
Tests: `a_recovery_in_another_mount_view_of_the_journal_refuses` and
`a_recovery_in_another_mount_view_of_the_cgroup_parent_refuses_instead_of_reading_a_missing_scope_as_gone`
(bind-mounted other views, same cgroup namespace), both asserting the reason and that the journal is retained.

### d. Ordinary operating-system calls make progress

A blocking kernel or file-system call that cargo-shadow makes (`read`, `write`, `open`, `fsync`, `rename`, `unlink`) returns
in finite time.
Reason:
no deadline can be put on such a call from inside the process without isolating it behind a supervised helper, and the owner's own
rule is the same premise for the disk: "Assume the final disk save would never be cut short."
The assumption does not weaken V6: the waits that cargo-shadow controls are bounded (V6), and the calls this assumption covers are
not waits that cargo-shadow chooses.

### Assumptions the code already makes

- Linux only (owner: "Ignore Win and mac until I say so.").
- Kernel 5.14 or later for `cgroup.kill` and cgroup v2 mounted at `/sys/fs/cgroup`; elsewhere the weaker path.
- Every writer of the workspace's lockfiles that matters goes through the shim and takes its workspace lock;
  a plain cargo is not stopped.
- The file system honors `fsync` and directory `fsync`.
- The clock does not matter for correctness: the ownership of a file is never decided by a time stamp alone.

## Where an assumption fails, or no delegated group can be had

### The weaker path

When no delegated control group can be had
(the user manager is not running, `systemd-run` is missing, the own group is not delegated, cgroup v2 is absent,
the kernel is older than 5.14),
the run goes on in a process group of its own.
It promises what pnpm promises:
the process group is killed at the bound and at the exit, and no proof is claimed that a process that left the group has stopped.
The notice says so in one line that names the missing capability, for example
"cargo-shadow: no delegated control group (the user manager cannot be reached); a process that leaves the process group cannot be
stopped or found, so quiescence is not shown".

The journal records the weaker path (`pgroup starting` before the child starts, `pgroup <id>` after).
Recovery of such a journal refuses until a person acknowledges it.
The refusal message names the journal directory and the command that acknowledges it:

```sh
# acknowledge a journal of the weaker path after confirming that no cargo of that run is running
cargo-shadow acknowledge <journal directory>
```

The acknowledgement asserts that every process or worker of this run that is capable of modifying the workspace has stopped.
"No cargo is running" is not enough:
a descendant, a build script or a worker that cargo started counts.
The acknowledgement:

- is a decision of a person; nothing acknowledges by itself, by age or by the absence of the recorded group;
- overrides the uncertainty of the containment context only:
  a weaker-path record, a record that is missing, unreadable or not understood, a recorded containment that cannot be examined,
  and a mount or namespace view that differs;
- does not override a failed enumeration of the journals, an invalid or missing snapshot, a failed mandatory durability barrier,
  or a recorded group that is positively observed to have a member;
- is written durably into the journal under the workspace lock (a file `acknowledged` published by an atomic rename, with the time
  and the process id of the acknowledging command), and a recovery then settles the journal as it settles any ready journal,
  including all of its own validation;
- is per journal, so one acknowledgement never covers another journal, and a new transaction beside an unacknowledged journal
  of the weaker path is refused with the same message;
- if its publication is interrupted, leaves the journal unacknowledged (an unreadable or partial `acknowledged` file does not
  count).

This reverses the part of the earlier pick that let weaker-path recovery go ahead when the recorded group had no member.
It closes 7c F1 and 7a F4.
The test `an_announced_child_whose_group_is_recorded_and_gone_is_no_survivor` encodes the reversed pick and is replaced by
`a_weaker_path_journal_whose_group_is_gone_is_refused_until_it_is_acknowledged` and
`an_acknowledged_weaker_path_journal_is_settled_like_any_other` (both to be written).
The acknowledging command also needs the "advertised repair" of 6b2 7 to work:
the message names a command that the reader accepts.

### A failed assumption

- Assumption a fails:
  the wrapper may accept or restore a lockfile that a process then rewrites, or may read a scope as empty while a writer lives
  elsewhere; that is the same state as any same-user process writing `Cargo.lock`, and no message is promised.
- Assumption b fails:
  the runtime guarantee stays (no inode loses its last name while the process runs, R2), and every crash-durability guarantee
  about the directory entries of a kept copy is withdrawn: after a power failure the entry, and the copy it names, may be missing.
- Assumption d fails:
  a call blocks without end, and so does the command; nothing in this file promises otherwise.
- Assumption c fails:
  recovery refuses and names the journal; a person resolves it by acknowledging, as above.

## The spawn route

1. **The only route to a control group: a transient user scope with `Delegate=yes`.**
   systemd's own rule (systemd.io/CGROUP_DELEGATION, read 2026-11-04):
   "Never create your own cgroups below arbitrary cgroups systemd manages, i.e cgroups you haven't set `Delegate=` in."
   cargo-shadow starts the program as `systemd-run --user --scope --unit=<name> --slice=<slice> --property=Delegate=yes --collect
   --quiet -- <program> <arguments>`, so that the manager, not a thread of cargo-shadow, puts the program in the new scope.
   (The equivalent `StartTransientUnit` call is not used: it needs a bus client, and `systemd-run` has the same effect.)
   The manager's real hierarchy is discovered, not assumed: the slice's group is
   `systemctl --user show <slice> --property=ControlGroup --value`, asked once per process, and the scope's group is that path plus
   `/<name>.scope`.
   The name is `cargo-shadow-<128-bit random token>` and is never derived from the process id or the clock.
   The direct child is killed and reaped before the scope is stopped, so that no process of cargo-shadow's own can still become the
   program after the stop.
2. **A child of the own group is not built.**
   The earlier pick (a child of the own group where it carries `user.delegate`) needs a way to start a process inside a group
   without a thread of cargo-shadow writing `cgroup.procs`; the only such way is `clone3` with `CLONE_INTO_CGROUP`, which
   neither `std` nor `rustix` 1.1.6 offers.
   Where the scope route is unavailable, the run takes the weaker path.
   (Coordinator's amendment to be confirmed: route 2 dropped, not queued.)
3. No route moves the wrapper, or any thread of it, between groups.
   This removes the spawn lock and the shared-process migration of 7a F1 and F2 by construction.

### Probe of the scope route (2026-11-04, Linux 7.2.15, systemd 262.1)

Run inside `systemd-run --user --scope --property=CPUQuota=100% --property=MemoryMax=512M --property=TasksMax=200
--property=IOWeight=10`; scripts `probe-scope-spawn.ts` and the commands below in `${HOME}/temp/agent/cargo-shadow-r7/`.

Result: the route works as the contract needs.

- **The program runs in the new scope and nothing of cargo-shadow's wrote a `cgroup.procs`.**
  The program's own `/proc/self/cgroup` was `.../app.slice/cargo-shadow-probe-<pid>.scope`.
  The parent's group was unchanged.
  `strace -f -e trace=openat,openat2,write` over `systemd-run --user --scope ... -- true` (73 lines) shows no open of a file named
  `cgroup` at all: `systemd-run` asks the manager over the bus, and the manager moves the process.
- **The 7a F1 schedule does not occur.**
  An ordinary child started from a second thread of the parent while the scope spawn ran stayed in the parent's group.
- **The scope path is predictable before the start.**
  With `--unit=<name> --slice=app.slice`, the scope is `<ControlGroup of app.slice>/<name>.scope`; `systemctl --user show app.slice
  --property=ControlGroup --value` gave `/user.slice/user-1000.slice/user@1000.service/app.slice`, equal to the prefix of the own
  group up to `user@<uid>.service` plus `/app.slice`, but the contract uses the answer of the manager, not the prefix.
  The predicted path existed and matched the program's group, so the record can be written and synced before the program starts
  (V3).
  The inode of the scope is known only after the start, so the record carries the path of a unique name (a second start of a name that was
  still loaded failed with "Unit ... was already loaded or has a fragment file", and the random token makes a repeated name
  impossible in practice), not an inode.
- **The program replaces `systemd-run`: same process, same pipes, same exit status.**
  `exit 7` in the program gave status 7; stdout was the program's.
- **Descendants stay in the scope.**
  A `setsid` descendant (own session) was listed in the scope's `cgroup.procs`.
- **The scope directory is the user's and the stop works through it.**
  Owner uid 1000; `cgroup.kill` written by the user stopped the members, and with `--collect` the manager removed the scope
  directory 1.8 ms after the kill.
  An absent scope path is therefore the normal end of a run, and "gone" for a recovery.
- **Start failure.**
  Without a reachable user manager, `systemd-run` printed "Failed to connect to user scope bus" and exited 1 without running the
  program.
  The program's own exit status 1 is indistinguishable by status alone.
  This costs a diagnostic, not safety: a program that did not run has no member, `populated` is never 1, and the run is reported as
  an ordinary failed command.
  A later round may separate the two by preflight (one `systemd-run ... true` per process) or by `StartTransientUnit` directly.
- **Cost.**
  The start of a scope took 7.0 ms median (6.2 minimum, 14.3 maximum, 15 runs) against 1.9 ms for a plain `true`, measured
  earlier the same day; the whole-run cost with this route is not measured yet.
- **The `user.delegate` extended attribute.**
  Present on `user@1000.service` (`user.delegate="1"`), absent on `app.slice`, absent on the harness scope `claude-code-bash`,
  and **absent on a transient scope started with `Delegate=yes`** (only `security.selinux` was listed).
  So the attribute identifies the user manager's own subtree, not the scopes under it;
  point 2 of the route applies in practice to a process whose own group is `user@<uid>.service` or a descendant that carries the
  attribute, and a scope is made through point 1.
  Whether any other delegated cgroup in the wild carries the attribute was not probed.

Probe 2 (2026-11-04T05:10-05:00): a scope whose program exited while a `setsid` descendant lived stayed observable with
`populated 1` and the descendant's process id, and vanished about 3 s later when the descendant ended.
That is the invariant A1 relies on for verified absence.

Not probed, so the code does not rely on it:
the behavior on a host with a user manager but a read-only `/sys/fs/cgroup`.

## Finding mapping

- Retired by an assumption: 7a F5, 7c F2 (a), 7b P1-1 (b); answered by assumption c with its two checks: 7a F6.
- Closed by the weaker-path rule: 7c F1, 7a F4.
- Closed by the spawn route, once built: 7a F1, 7a F2.
- Bugs regardless of the contract: 7a F3 (A2) and 7b P1-2 (V1).
- Test-evidence notes of the three reviews (handshakes instead of sleeps, entry-level injection, real rename and sync failures,
  crash-test assertions): to be applied with the tests named above.

## Acceptance list

This is the cross-vendor reviewer's finite checklist (`cargo-shadow-sol-contract-1.txt`, section 4), copied verbatim in substance.
The rule that comes with it:
**a new counterexample may reopen a row of this list, but it does not add a row.**
Concurrency tests need handshakes at the relevant boundary, not elapsed sleeps.
Strong-path tests must establish that they actually used delegated containment.
These are acceptance tests, not a claim that finite testing mathematically proves every execution.

- **A1.**
  Zombie leader with a live writer thread; pipe-closing `setsid` or double-fork writer; normal exit and timeout.
  Acceptance occurs only after the specified emptiness proof and direct-child reap.
- **A1, V3 (spawn isolation).**
  Overlap two bounded starts and an unrelated ordinary spawn inside startup.
  The wrapper stays in its original group; runs have separate boundaries; unrelated work survives cleanup.
  Exercise every supported spawn route.
- **A1, A2 (scope lifecycle).**
  Direct command exits while a descendant remains.
  The scope either remains observable or the manager stops that descendant without moving it elsewhere.
  Also exercise rapid empty-scope collection and the chosen valid-absence rule.
- **A2, R4 (7a F3).**
  Start the child and a writer, then fail containment or startup completion, membership inspection, or stopping.
  Also simulate a member that cannot be stopped within the bound.
  Require `NotQuiescent`, writer uncertainty, retained journal, no acceptance, no rollback, and no user command.
  No post-start downgrade to ordinary operational failure.
- **A3, A4.**
  Approved, refused and operational-error outcomes; fail output synchronization, the actual completion rename, and the actual
  completion-parent sync.
  Mandatory failures stop shim fallback and preserve applicable diagnostics.
  Verify outputs, then marker, then marker-directory barrier, then removal ordering.
  Cleanup-only failure is a warning only after durable completion.
- **R1.**
  Another writer changes the lockfile before rollback and at the replacement or conflict boundary.
  Its result remains intact, with a warning; rollback does not silently install the snapshot over it.
- **R2.**
  Fault every preservation or link stage with the queued `EMLINK`, `ENOSPC` and `EXDEV` cases, plus relevant rename and sync
  failures.
  Inventory the displaced and replacement inodes after each failure: neither loses its last protected name; stranded locations are
  reported.
- **R3, R5.**
  Both refused output and cargo failure followed by rollback failure must stop the command, including `--locked` fallback.
  With multiple roots, fail one restore while another restores and emits warnings; preserve all diagnostics and continue
  independent restores.
- **K1.**
  Below, at and above the warning threshold, across repeated runs: correct count, size and directory; no count-based refusal or
  deletion.
  Inventory kept names, colliding names and stranded temporaries.
- **K2.**
  Create nested kept directories beneath a pre-existing anchor.
  Verify every required directory barrier through that anchor, and no traversal above it, including an unreadable higher ancestor.
  Fail each required barrier: do not report preservation success or discard the remaining protected name.
- **V1 (7b P1-2).**
  Fail opening, iteration and entry inspection independently.
  In particular, accumulate a live journal, then inject an entry-level `NotFound` for another entry.
  Both recovery and `begin` must stop without discarding the partial listing.
  Initial absence of the journal directory remains a separate positive case.
- **V2, amended c.**
  A survivor in one journal blocks recovery and admission involving another.
  Exercise unreadable, malformed and missing records beside other files.
  Independently change the journal view and the recorded cgroup-parent view, including the same-cgroup-namespace bind-mount
  schedule: refuse with the intended reason and retain the journal.
- **V3, V4.**
  Fail each record-publication or durability step: no payload starts.
  Kill the wrapper in preparation and start windows and verify conservative recovery.
  A genuinely empty journal is removable; a nonempty journal lacking required records is not treated as empty.
- **V5.**
  Recover multiple roots with their manifests and lockfiles kept together.
  Preserve refused leftovers without overwriting an existing `Cargo.lock.refused`.
  Exercise prepared, ready and done states, repeated recovery, and the same mandatory-finish versus cleanup distinctions as A3 and A4.
- **V6.**
  Bound manager startup, command lifetime, pipe draining, shutdown and reaping, including descendants holding pipes open.
  Busy workspace and internal locks return rather than wait.
  Unsuccessful shutdown produces uncertainty, not an unbounded wait.
- **Weaker path (A2, V2, V3, V5, V6).**
  Each unavailable-capability class produces the promised notice and the pre-start weaker-path record.
  Both `pgroup starting` and completed-but-gone records block recovery and admission without acknowledgement.
  The advertised CLI command works, uses the workspace lock, durably acknowledges exactly one journal, survives interrupted
  publication conservatively, and cannot bypass unrelated validation failures.
  Acknowledging one of two journals leaves the other blocking.

## Coverage map

Written 2026-11-04T05:40:00-05:00 against the tip that carries it.
A row is "shown" when its schedules each have a test that fails when the guarantee is removed or that asserts the observable result;
"partly" when a named part has no test yet, with what that part needs.
No row is without any test.
Counts: 16 rows, 9 shown, 7 partly, 0 not yet.
The suite at this tip: 539 tests passed, 3 skipped (the three ignored helper processes of the crash tests:
`wrapper_with_an_escaped_writer_fixture`, `recovery_in_another_mount_namespace_fixture`, `wrapper_held_inside_its_start_fixture`).
Red first was done where the old code could show red:
the round 7 red commit `939dcb31d1` holds the counterexamples of rows A1, A3, A4, V1 and V2 against the round 6 code;
for the two bugs of round 8 and the new argument-grammar test the guard was removed on the committed tests and the tests failed
(`a_journal_listing_that_fails_in_any_way_stops_the_recovery`, `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`,
`arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged`), then the guard was put back.
The rows about the scope route, the acknowledgement and the two mount views are new behavior with no old code to fail;
their tests were written with the code.

- **A1: partly.**
  Zombie leader with a live thread: `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns` (the fixture
  says by itself when its leader is a zombie; the run's result is checked by the process having no live thread).
  Pipe-closing `setsid` writer, normal exit and timeout: `a_writer_in_another_session_is_stopped_before_a_normal_exit_returns`,
  `a_writer_in_another_session_is_stopped_at_the_bound`; double fork: `probe_3_a_setsid_and_double_fork_descendant_stays_a_member`,
  `orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped`.
  Not yet: acceptance after a direct child that cannot be reaped within its bound
  (needs a failpoint in `reap` that makes the child look unreapable).
- **A1, V3 (spawn isolation): shown.**
  `two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary` (a hook holds the first run inside its
  start; an unrelated spawn and a second run happen; the wrapper's group never changes; separate scopes; the unrelated child survives),
  `probe_5_the_spawner_never_changes_group_and_an_unrelated_child_stays_out`.
  The scope route is the only supported route; the weaker path moves no group.
- **A1, A2 (scope lifecycle): shown.**
  `probe_2_a_scope_stays_observable_while_a_descendant_outlives_the_program`,
  `probe_2_a_scope_that_is_collected_at_once_is_accepted_as_empty_by_verified_absence`,
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`.
- **A2, R4 (7a F3): partly.**
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent` (three faults; `NotQuiescent`, writers may remain,
  journal retained, a writer left in the scope that the test then stops),
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal` (the install half: no rollback, no acceptance,
  the command not run).
  Not yet: one test that joins the two (a fake cargo that goes through `output_within` with an injected fault, so the whole path from
  the runner to the shim's refusal is exercised); the install half uses `mark_scope_lost_for_test` for the runner's verdict.
- **A3, A4: shown.**
  Approved: `an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`;
  refused: `a_refusal_survives_a_journal_that_cannot_be_finished`, `a_refusal_survives_a_failed_completion_marker_and_names_it`;
  operational: `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics`;
  the actual rename and parent sync: `an_accepted_run_whose_completion_rename_fails_stops_the_command`,
  `an_accepted_run_whose_completion_directory_sync_fails_stops_the_command`,
  `a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active`,
  `a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal`;
  ordering: `finish_makes_the_outputs_durable_before_the_done_marker`, and removal does not run after a failed marker sync (the
  last test above finds the `.done` journal still there);
  cleanup after durable completion: `the_cleanup_failure_happens_after_the_completion_marker_was_published`,
  `a_removal_failure_after_the_marker_is_cleanup_only`.
- **R1: shown by tests of earlier rounds, not rewritten.**
  `a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning` (before the rollback),
  `recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged`,
  `a_replacement_that_another_writer_holds_open_is_kept_when_the_exchange_is_undone`,
  `a_replacement_edited_in_place_after_the_exchange_is_kept_not_deleted` (the replacement and conflict boundary).
- **R2: partly.**
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name` (`EMLINK`, `ENOSPC`, `EXDEV`, `EACCES`),
  `when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where` (every link, the data
  barrier, the directory barrier), `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live`,
  `a_filesystem_without_exchange_fails_the_restore_and_overwrites_nothing`.
  Not yet: a failing rename of the temporary and a read-only transition between stages (need a seam on the rename and on the
  stage order), and the replacement inode's name inventory (the tests inventory the displaced inode only).
- **R3, R5: shown.**
  `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions`,
  `a_cargo_failure_followed_by_a_failed_rollback_stops_the_command_instead_of_falling_back`,
  `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others`.
- **K1: shown.**
  `the_kept_files_warning_point_is_exact_and_repeated_runs_agree` (below, at, above; two runs each; nothing deleted or refused),
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning`,
  `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`;
  colliding names: `a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives`.
- **K2: partly.**
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction`,
  `a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable`,
  `a_failing_directory_barrier_is_not_reported_as_preserved`, `a_higher_ancestor_the_user_cannot_open_is_never_touched`,
  `a_kept_directory_behind_a_symbolic_link_is_synced_by_its_resolved_path`.
  Not yet: failing each barrier of the creation chain one at a time (the tests fail the kept directory's barrier and the registry
  case); needs a seam on `create_directories_with`'s syncs in a test of the whole preservation.
- **V1 (7b P1-2): shown.**
  `a_journal_listing_that_fails_in_any_way_stops_the_recovery`, `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`
  (open, iteration, inspection and entry-level `NotFound`, injected inside the listing on the second entry with a journal already in
  the partial listing), `an_absent_journal_directory_is_no_journals` (the separate positive case).
- **V2, amended c: shown.**
  Survivors: `a_survivor_of_one_journal_stops_the_recovery_of_another`, `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`; records: `a_group_record_that_cannot_be_read_defers_the_recovery`,
  `a_malformed_group_record_defers_the_recovery`, `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`;
  views: `a_journal_seen_through_another_view_is_refused_and_kept`,
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`,
  `probe_4_a_scope_is_found_by_its_record_and_a_different_view_is_unknown`, and the same-cgroup-namespace schedule end to end,
  `a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal` (a new user and mount namespace through `unshare`).
- **V3, V4: shown.**
  `the_scope_record_is_durable_before_the_program_starts` (a hook at the boundary reads the record and the program's marker),
  `a_record_that_cannot_be_published_starts_no_program_and_does_not_wait` (a pipe where the record was, both containments),
  `a_child_whose_group_cannot_be_recorded_is_not_started`,
  `a_wrapper_killed_inside_its_start_is_recovered_conservatively` (killed at the boundary after the record, both containments),
  `a_recovery_removes_an_empty_journal_directory`, `a_nonempty_journal_without_its_record_is_not_treated_as_empty`.
- **V5: partly.**
  `a_second_recovery_pass_changes_nothing_and_keeps_the_refused_copy`, `a_killed_breaking_run_whose_lock_is_refused_gets_its_manifest_back_too`,
  `a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives`, `a_journal_marked_done_is_removed_by_the_next_recovery`,
  the unready and ready tests of `journal_recover_tests.rs`.
  Not yet: a recovery of two roots with manifests in one journal where one root is refused and the other accepted (needs a judge that
  decides per root, which the tests' judge does not).
- **V6: partly.**
  `a_manager_that_does_not_answer_ends_the_discovery_at_its_deadline`, `a_child_that_outlives_its_bound_is_stopped_and_named`,
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete`,
  `a_timeout_returns_promptly_even_when_the_child_ignores_its_input`, `recovery_defers_while_a_live_transaction_holds_the_workspace`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused`, and the stuck-member case of A2.
  Not yet: the reaping bound of a direct child that cannot be reaped (the same failpoint as A1).
- **Weaker path: shown.**
  `each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start` (no `systemd-run`, an unreachable
  manager, no cgroup v2 mount, an unknown slice; the kernel before 5.14 by `probe_1_a_group_directory_without_cgroup_kill_is_an_older_kernel`),
  `a_weaker_path_journal_is_refused_by_recovery_and_admission_until_it_is_acknowledged` (`pgroup starting` alone and a completed record
  whose group is gone), `acknowledging_one_of_two_journals_settles_it_and_leaves_the_other_blocking`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused`, `a_partial_acknowledgement_does_not_count`,
  `an_acknowledgement_does_not_override_a_live_group_or_unrelated_validation`, `the_acknowledge_subcommand_acknowledges_the_journal_the_refusal_named`,
  `the_weaker_path_still_stops_the_process_group_at_the_bound`.

### Findings the round 8 work made, outside the list

- `systemd-run` reads the command it is given like a unit's command line: `$$` became `$` and `${HOME}` was replaced by the value of
  the variable (probed 2026-11-04).
  Every argument and the program are now encoded at the point of hand-over, and
  `arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged` runs fifteen awkward arguments, also through a cleared
  environment.
- A cleared environment (`env_clear`, used for Git) cannot be read back from a `Command`, so there is a second entry point,
  `output_within_clean_environment`, and the program is started through `env --ignore-environment`.
- A program that does not exist is found before the launcher starts, so it stays an I/O error (`systemd-run` reports it as an exit
  status).
- The cost of a trivial run in the test build, which asks the manager every time (the production build asks once per process):
  23.6 ms median (18.4 minimum, 33.5 maximum, 30 runs) against 6.3 ms in the weaker path; the production figure is lower by the cost
  of the question, which was not measured separately.
- Route 2 of the earlier pick (a child of the own group where it carries `user.delegate`) is dropped:
  it needs `clone3` with `CLONE_INTO_CGROUP`, which neither `std` nor `rustix` 1.1.6 offers.
