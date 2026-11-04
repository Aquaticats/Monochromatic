# What cargo-shadow promises and what it assumes

Status:
draft for the cross-vendor reviewer's finite acceptance list, written 2026-11-04T04:45:55-05:00 after the round 7 re-review
(`cargo-shadow-sol-7a.txt`, `-7b.txt`, `-7c.txt`, all "not closed").
No product code changed in this step.
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
  that every process the run started has stopped: the kernel reported `populated 0` for the run's group, and the direct child
  was reaped.
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
  Nothing else is synced ("else background" has no carrier in a command that exits).
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
  Kept copies are never deleted and never cause a refusal;
  past a count every run prints the count, the total size and the folder.
  Test: `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses` exists;
  the inventory of kept names beside their target and of stranded temporaries is queued for round 8 (7b P2).
- **K2.**
  A kept copy's directory entries are durable when preservation reports success (assumption b below).
  Test: `every_ancestor_of_the_kept_directory_is_synced_up_to_the_mount_root` exists and is to be replaced by
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction` (to be written) when the stop rule changes;
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
  Every wait is bounded and every lock is tried, never waited for.
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

### a. No same-user process deliberately leaves its group or acts adversarially

A process of the same user that runs inside a run (a build script, a proc macro, a selected executable) is assumed to act as an
ordinary program does:
it may fork, daemonize, call `setsid`, close its streams and outlive its parent, and the group still holds it.
It is assumed not to write its id into another group's `cgroup.procs`, not to ask the user manager to move it,
and not to do anything else whose purpose is to defeat the wrapper.

Reason:
a same-user adversary can write `Cargo.lock` directly after the run, and no wrapper can prevent that, so a containment that tried
to resist it would claim a guarantee the wrapper cannot give.
Probe 3 of round 7 (`probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_the_stated_limit`) shows the move is possible
and stays as the evidence of the limit.
This retires 7a F5 and 7c F2 as findings, and the cgroup namespace closer as work.

### b. Directories that existed before the transaction began are durable

The directory-entry sync of a kept copy walks up from the kept directory to the first directory that existed before the
transaction began, syncs it, and stops.
A directory this transaction created is synced with the directory that lists it, as `create_directories_synced` does.
Directories that existed before the transaction began are assumed durable:
their own entries were committed by whoever made them, or by an earlier transaction of this tool, which synced what it created.

Reason:
the tool cannot know the durability history of a directory it did not make, and walking to a device boundary or to `/` rests on
a different, unprovable premise (a bind mount, a subvolume, an overlay root).
The rule replaces the round 7 stop at the first parent on another device (7b P1-1).
It also removes the failure of preservation on an unreadable ancestor above the workspace.

### c. The journal resolves to the same mount and inode in recovery as in the run

When a journal is created, the wrapper records the mount id and inode of the journal directory
(`statx`, `STATX_MNT_ID` and `STATX_INO`).
A recovery compares them with what the same path gives in the recovery process, and refuses, naming the journal, when either differs.

Reason:
a recovery in another mount namespace, or on a path that now leads somewhere else, cannot read an absent recorded group as "gone"
(7a F6).
Proposed addition, for the coordinator's amendment (not part of the pick as sent):
the same record also holds the mount id and inode of the directory that holds the recorded control group
(its parent in the cgroup file system), because the 7a F6 schedule is a different view of `/sys/fs/cgroup`, not of the journal;
the journal check alone would not see it.
Test: `a_recovery_in_another_mount_view_refuses_instead_of_reading_a_missing_group_as_gone` to be written
(a bind-mounted other view of the group's parent and of the journal directory).

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

The acknowledgement:

- is a decision of a person that no cargo of that run is still running;
  nothing acknowledges by itself, by age or by the absence of the recorded group;
- is written durably into the journal under the workspace lock
  (a file `acknowledged` with the time and the process id of the acknowledging command),
  and a recovery then settles the journal as it settles any ready journal;
- is per journal, so one acknowledgement never covers another journal, and a new transaction beside an unacknowledged journal
  of the weaker path is refused with the same message.

This reverses the part of the earlier pick that let weaker-path recovery go ahead when the recorded group had no member.
It closes 7c F1 and 7a F4.
The test `an_announced_child_whose_group_is_recorded_and_gone_is_no_survivor` encodes the reversed pick and is replaced by
`a_weaker_path_journal_whose_group_is_gone_is_refused_until_it_is_acknowledged` and
`an_acknowledged_weaker_path_journal_is_settled_like_any_other` (both to be written).
The acknowledging command also needs the "advertised repair" of 6b2 7 to work:
the message names a command that the reader accepts.

### A failed assumption

- Assumption a fails:
  the wrapper may accept or restore a lockfile that a process then rewrites;
  that is the same state as any same-user process writing `Cargo.lock`, and no message is promised.
- Assumption b fails:
  a kept copy's directory entry may not survive a power failure; the kept inode is never unlinked before its other names are durable
  as far as the walk reached, so a live copy remains.
- Assumption c fails:
  recovery refuses and names the journal; a person resolves it by acknowledging, as above.

## The spawn route

Picked before this file, recorded here so the code builds to it.

1. **Default: a transient user scope with `Delegate=yes`.**
   systemd's own rule (systemd.io/CGROUP_DELEGATION, read 2026-11-04):
   "Never create your own cgroups below arbitrary cgroups systemd manages, i.e cgroups you haven't set `Delegate=` in."
   cargo-shadow starts the program as `systemd-run --user --scope --unit=<unique name> --property=Delegate=yes --collect --quiet
   -- <program> <arguments>` (or the equivalent `StartTransientUnit` call), so that the manager, not a thread of cargo-shadow, puts
   the program in the new scope.
2. **A child of the own group only where that group carries the `user.delegate` extended attribute**
   (systemd sets it on delegated cgroups; checked below: `user@1000.service` carries it).
   Otherwise the weaker path.
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
  With `--unit=<name>`, the scope is `<user manager group>/app.slice/<name>.scope`, where the manager group is the prefix of the
  own group up to `user@<uid>.service`.
  The predicted path existed and matched the program's group, so the record can be written and synced before the program starts
  (V3).
  The inode of the scope is known only after the start, so the record carries the path of a unique name (never reused while the
  unit is loaded; a second start of the same name failed with "Unit ... was already loaded or has a fragment file"), not an inode.
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

Not probed, so the code does not rely on it:
what systemd does with a delegated scope started with `--collect` when the program ends while a member remains
(the probe killed the group while the program lived),
and the behavior on a host with a user manager but a read-only `/sys/fs/cgroup`.

## Finding mapping

- Retired by an assumption: 7a F5, 7c F2 (a), 7b P1-1 (b), 7a F6 (c, with the proposed addition).
- Closed by the weaker-path rule: 7c F1, 7a F4.
- Closed by the spawn route, once built: 7a F1, 7a F2.
- Bugs regardless of the contract, to fix after the reviewer's list: 7a F3 (A2) and 7b P1-2 (V1).
- Test-evidence notes of the three reviews (handshakes instead of sleeps, entry-level injection, real rename and sync failures,
  crash-test assertions): to be applied with the tests named above.
