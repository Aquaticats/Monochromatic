# What cargo-shadow promises and what it assumes

Status:
amended once more after the round 9 reviews (`cargo-shadow-sol-9a.txt`, `-9b.txt`; round 10, written 2026-11-04T08:57:09-05:00) with
the unsettled scope (guarantee A1: an empty or late-appearing scope is no more a completed lifecycle than an absent one), the strict
and authoritative boot identity (assumption c and e) and the shared discovery deadline (guarantee V6);
amended again after the round 8 reviews (`cargo-shadow-sol-8a.txt`, `-8b.txt`) with the boot identity (assumption c and the new
assumption e), the settled record (guarantee A1) and the acknowledgement's whole record and durability (weaker path);
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
Every test named here exists in the package; the coverage map at the end says which part of a row has no test yet.
Every guarantee is stated for the normal path, where a delegated control group is in force.
The weaker path (third part) keeps only the guarantees marked as holding there.

### Acceptance of a lockfile

- **A1.**
  A lockfile that a run left is accepted (left live, journal finished, command allowed to run) only after the wrapper has shown
  that every process the run started has stopped: the direct child was reaped, and either the kernel reported `populated 0` for
  the run's scope, or the scope is verifiably absent.
  Verified absence counts only for a run that recorded its end (a `settled` line in the journal, written after the launcher was
  reaped and the scope shown empty): until then an absent scope may be one the launcher has not made yet, and a scope that exists
  with `populated 0` may be one whose launcher the manager has not attached yet (the manager creates the control group and attaches
  the process in two steps), so a recovery that finds an unsettled scope absent, empty, or appearing while it checks refuses and asks
  for an acknowledgement (round 9, 8a F1; round 10, 9a D1).
  A later look at the scope never turns an unsettled run into a completed one; a scope with a member blocks and no acknowledgement
  overrides it.
  The way out of such a refusal is the acknowledgement of the weaker path (`cargo-shadow acknowledge <journal directory>`): one
  command by a person, which tries the workspace lock and does not wait, so nothing in the program waits for it.
  A protocol that would remove the refusal without a person (the launcher's process id and start time recorded after its spawn, and
  a check that it is gone before the scope is read) is not built; it is an option in "Round 10" of
  `doc/planning/cargo-shadow-update.md`.
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
  Test: `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal` (call-site half) and
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent` (7a F3: a failed stop, a failed inspection and a
  member that cannot be emptied are each `NotQuiescent`).
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
  `an_update_error_with_a_failed_mandatory_phase_reports_both`;
  the real rename and the real parent sync of the completion marker are failed by
  `a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active` and
  `a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal`.
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
  Test: `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live`,
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name` (`EMLINK`, `ENOSPC`, `EXDEV`, `EACCES`),
  `when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where`.
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
  Test: `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`,
  `the_kept_files_warning_point_is_exact_and_repeated_runs_agree`,
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning`.
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
  Test: `a_journal_listing_that_fails_in_any_way_stops_the_recovery`, `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`
  (7b P1-2: open, iteration, inspection and an entry-level `NotFound`, injected inside the iteration on the second entry, with a
  journal already in the partial listing), and `an_absent_journal_directory_is_no_journals` for the separate positive case.
- **V2.**
  A recovery or a new transaction does not proceed while any journal of the workspace names a recorded group with a member,
  a record it cannot read or understand, a lost record beside other files, or a record made where the recorded group cannot be
  examined (assumption c).
  Test: `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`,
  `a_survivor_of_one_journal_stops_the_recovery_of_another`, `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`;
  the crash test asserts the refusal reason ("still running"), the retained journal and that the run used a delegated scope.
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
  The discovery of the manager reads its output, kills its program and reaps it under the same deadline (the reaping spends what is
  left of it, and a program that is not reaped by then is reported as not reaped), so a descendant that keeps the pipes open does
  not extend it; the caches and registries it keeps are read with `try_lock`, so a caller that finds one busy asks again or
  answers conservatively and does not wait for the holder.
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
Probe 3 of round 7 (`probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_assumption_a`) shows the move is possible
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
Tests: `a_journal_seen_through_another_view_is_refused_and_kept`,
`a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone` and, end to end in a new user and mount
namespace with the cgroup namespace unchanged, `a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal`;
each asserts the reason and that the journal is retained.

The boot identity (the coordinator's pick of round 9, veto open, from the reviews of round 8):
the journal's view record also holds the machine id (`/etc/machine-id`) and the boot id (`/proc/sys/kernel/random/boot_id`) of the
system that made it; it is written before any record of a scope or a process group, by the one process that writes them, so it
covers every record.
A recovery that reads the same machine id and a different boot id treats the run's process liveness as discharged,
because a reboot of this machine ended every process of the run (assumption a).
It then does not compare the mount and cgroup views (they are values of the boot that ended), and it does not ask for an
acknowledgement for the containment.
What it keeps: journal enumeration, snapshot validation, preservation, the mandatory finish, and one contradiction:
a recorded scope (its name carries a random token) that has a member after the reboot is not explained by the reboot and blocks.
A different machine id, an unreadable boot or machine id on either side, a journal without a recorded identity, or the same boot id
keep the refusal and the acknowledgement.
Suspend and resume keep the boot id and so give no clearance.

Round 10 (9a D2 and section 4) makes the identity strict and authoritative:

- Both ids must have their format, on the recorded side and on the current side: the boot id is a lowercase `8-4-4-4-12`
  hexadecimal UUID, the machine id is 32 lowercase hexadecimal digits, neither is all zero.
  The view record is the line `view <mount>:<inode>` and then exactly one `boot` line and one `machine` line; a repeated field or an
  unknown line leaves the record without an identity.
  A malformed id is not evidence of a reboot: it keeps the refusal (the reviewer's schedule: `boot not-a-boot-id` beside a live
  process group).
- The boot id file must be the kernel's own file: `statfs` reports procfs (`0x9fa0`), and `statx` reports that the file is not the
  root of a mount (`STATX_ATTR_MOUNT_ROOT` clear, and a kernel that does not report the attribute is an error).
  Measured in a disposable user and mount namespace on 2026-11-04: a bind mount of a regular file over the boot id file reports
  another file system, a bind mount of another procfs file reports a mount root, and a read-only bind of the directory that holds
  the file (the way container runtimes mask paths) reports neither, so it is accepted.
  A file that fails the check is no identity: unreadable, the refusal stays.
  An identity is not recorded when the check fails at `begin`, so the journal has none.
- The premise that remains, stated here as part of this assumption: the kernel that serves the process is the kernel that ran the
  writers, and its procfs is not emulated.
  A sandbox that emulates `/proc` in user space or in another kernel (and reports procfs for its own boot id) is that kernel for
  this purpose; a boot id that a runtime replaces by a bind mount (`systemd-nspawn` does, with a random id per container start) fails
  the check above, so such a container gets the acknowledgement, not the clearance.
Tests: `a_malformed_recorded_identity_is_not_a_reboot`, `a_malformed_current_identity_is_not_a_reboot`,
`the_current_identity_is_read_strictly_from_its_files`, and, end to end in a new user and mount namespace,
`a_boot_id_put_over_the_kernels_file_is_not_an_identity` (a regular file, another procfs file and a tmpfs over the directory are
replaced and keep a live group blocking; a read-only bind of the directory is not a replacement).
Tests: `a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
`another_machine_an_unreadable_identity_no_record_or_the_same_boot_keeps_the_refusal`,
`the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member`.
The tests give the recovery a different identity; no machine was rebooted.

Two other facts, stated so they are not found by surprise:

- The mount id is assigned for the life of one boot (the unique id of Linux 6.8 and later when the kernel gives it, the reusable id
  otherwise), so within one boot it identifies the mount; across a reboot the boot identity above decides, not the mount id.
- With the reusable id, a mount id that was reused for another mount together with a coinciding inode would be accepted within a boot;
  the unique id removes that case on kernels that have it.

### d. Ordinary operating-system calls make progress

A blocking kernel or file-system call that cargo-shadow makes (`read`, `write`, `open`, `fsync`, `rename`, `unlink`) returns
in finite time.
Reason:
no deadline can be put on such a call from inside the process without isolating it behind a supervised helper, and the owner's own
rule is the same premise for the disk: "Assume the final disk save would never be cut short."
The assumption does not weaken V6: the waits that cargo-shadow controls are bounded (V6), and the calls this assumption covers are
not waits that cargo-shadow chooses.

### e. One machine at a time, and no checkpoint and restore

A workspace is used from one machine at a time: the workspace lock reaches only one kernel, so a second machine could not see the
first one's lock, and a different boot id on another machine says nothing about the first machine's processes (8b: host B recovering
over host A's live writer).
That is why the boot identity discharges liveness only for the same machine id.
Checkpoint and restore of processes (and the restoration of a whole virtual machine's state) is out of scope: it can bring back
processes that the boot identity counts as ended.
A copied `/etc/machine-id` (a cloned host) is the same as one machine under this assumption.
The boot id is trusted as the kernel's only as assumption c says: the file must be the kernel's own and not put over, and the
kernel is not an emulation.

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
- is written durably into the journal under the workspace lock (a file `acknowledged` published by an atomic rename, with the process
  id and the time of the acknowledging command), and a recovery then settles the journal as it settles any ready journal,
  including all of its own validation;
- counts only as the whole versioned record (the header, a process id, a time, each line ended by a line feed), and only after the
  reader has made the file and its directory entry durable itself; a marker that is incomplete, or that the reader cannot make
  durable, does not count;
- is per journal, so one acknowledgement never covers another journal, and a new transaction beside an unacknowledged journal
  of the weaker path is refused with the same message;
- if its publication is interrupted, leaves the journal unacknowledged (an unreadable or partial `acknowledged` file does not
  count).

This reverses the part of the earlier pick that let weaker-path recovery go ahead when the recorded group had no member.
It closes 7c F1 and 7a F4.
The round 7 test that encoded the reversed pick (a recorded and gone group is no survivor) is replaced by
`a_weaker_path_journal_is_refused_by_recovery_and_admission_before_it_is_acknowledged` and
`acknowledging_one_of_two_journals_leaves_the_other_blocking_everything`.
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
   program after the stop; only then does the run append `settled <scope path>` to the journal.
2. **A child of the own group is not built.**
   The earlier pick (a child of the own group where it carries `user.delegate`) needs a way to start a process inside a group
   without a thread of cargo-shadow writing `cgroup.procs`; the only such way is `clone3` with `CLONE_INTO_CGROUP`, which
   neither `std` nor `rustix` 1.1.6 offers.
   Where the scope route is unavailable, the run takes the weaker path.
   (Open for the coordinator to confirm: route 2 is dropped, not queued.)
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
- Closed by the spawn route: 7a F1, 7a F2 (`two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary`).
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
  An unsettled scope that is absent, empty or appears during the check refuses; a malformed or replaced boot identity never clears
  a live group (round 10; counterexamples to this row, not a new row).
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

Written 2026-11-04T08:57:09-05:00 against the tip that carries it (round 10); the round 9 text it replaces was written at
2026-11-04T07:00:00-05:00.
A row is "shown" when each of its schedules has a test that fails when the guarantee is removed or that asserts the observable
result; "partly" when a named part has no test yet.
No row is certified by this map: the cross-vendor review of round 10 has not run, and the round 9 review of the tests
(`cargo-shadow-sol-9b.txt`) said that "16 rows shown" was not established by the attachments it saw.
Counts: 16 rows, 14 shown, 2 partly (A2/R4: the call-site test has no shim and no user-command sentinel behind it; V3/V4: the
preparation windows are built by hand, not cut at the real boundaries), 0 not yet.
Suite counts at the round 10 tip: the debug run (`mise run //package/cargo-shadow/cli:test:debug`) and the release gate
(`mise run //package/cargo-shadow/cli:test`) each ran 575 tests: 570 passed, 0 failed, 5 skipped; `lint:clippy` with warnings denied
is clean.
The 5 skipped are the `#[ignore]` helper entries that five other tests start as child processes (their names are in the `ignore`
attributes of `src/contain_counterexample_tests.rs`, `src/containment_record_tests.rs` and `src/round_ten_tests.rs`); they are not run
on their own.
Logs: `/var/home/user/temp/agent/cargo-shadow-r10/logs/r10-debug2.log`, `r10-gate2.log`, `r10-clippy3.log`.
Red first was done where the old code could show red:
the round 7 red commit `939dcb31d1` holds counterexamples of rows A1, A3, A4, V1 and V2 against the round 6 code.
In rounds 8 to 10 the new guards were removed from the committed tests and the tests failed, then the guard was put back:
round 8: `a_journal_listing_that_fails_in_any_way_stops_the_recovery`, `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`,
`arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged`;
round 9: `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended`,
`a_discovery_whose_descendant_holds_the_pipes_ends_at_its_deadline`,
`a_busy_unsynced_registry_returns_at_once_and_answers_conservatively`,
`an_acknowledgement_with_three_lines_that_are_not_a_process_id_and_a_time_does_not_count`,
`an_acknowledgement_the_reader_cannot_make_durable_does_not_count_until_it_can`,
`an_unreadable_crate_root_gives_an_unknown_inventory_warning_and_never_refuses`,
`a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
`the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member`;
round 10 (each red on the committed old code or with the guard removed, then green; commits `6a6513b482`, `1318ffd101`, `147f558d13`):
`an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended`,
`a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended`,
`a_malformed_recorded_identity_is_not_a_reboot` (the lenient reader put back for the red run),
`a_malformed_current_identity_is_not_a_reboot`,
`a_boot_id_put_over_the_kernels_file_is_not_an_identity` (the authority check made to pass for any existing file, for the red run),
`a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so`,
`a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` (the parent-view comparison removed
for the red run).
New behavior with no old code to fail (the scope route, the acknowledgement, the two views, the boot identity as a whole) has tests
written with the code.

- **A1: shown.**
  `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns` (the fixture says when its leader is a zombie
  and records its worker's thread id; the program's group is recorded and must be a delegated scope),
  `the_thread_oracle_sees_a_live_worker_behind_a_zombie_leader_and_its_end` (the control of the oracle),
  `a_writer_in_another_session_is_stopped_before_a_normal_exit_returns`, `a_writer_in_another_session_is_stopped_at_the_bound`,
  `probe_3_a_setsid_and_double_fork_descendant_stays_a_member`,
  `orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped` (forty distinct orphans, each recording its own process id; the
  program's group must be the scope the journal recorded for the run and differ from the wrapper's own group, round 10, 9b 3.5),
  `a_direct_child_that_cannot_be_reaped_makes_the_run_not_quiescent`.
  Round 10 (9b 3.7): the process oracle counts only `ESRCH` as an ended process, the thread fixture publishes a complete thread id by
  rename, and the writer tests of `src/bounded_contain_tests.rs` assert the scope record of their run.
- **A1, V3 (spawn isolation): shown.**
  `two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary`,
  `probe_5_the_spawner_never_changes_group_and_an_unrelated_child_stays_out`,
  `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended`.
- **A1, A2 (scope lifecycle): shown.**
  `probe_2_a_scope_stays_observable_while_a_descendant_outlives_the_program`,
  `probe_2_a_scope_that_is_collected_at_once_is_accepted_as_empty_by_verified_absence`,
  `a_settled_run_whose_scope_was_collected_is_read_as_ended` (round 10, 9b 3.4: it waits until the recorded scope is absent),
  `an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended` (creation before placement: the control group exists with
  `populated 0`),
  `a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended` (the race between two observations),
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`.
- **A2, R4 (7a F3): partly.**
  Shown: the joined runner-to-install test, the fault classes and the unstoppable member.
  Not shown (9b section 2): the call-site test does not invoke the shim and does not place a user-command sentinel behind the
  refusal.
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent`,
  `a_real_run_that_cannot_show_quiescence_stops_the_install_with_the_lockfile_kept_and_nothing_after_it` (a fake cargo whose update goes
  through the bounded runner with an injected fault: refusal, lockfile kept, journal kept, no call after the update),
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal`.
- **A3, A4: shown.**
  `an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`, `a_refusal_survives_a_journal_that_cannot_be_finished`,
  `a_refusal_survives_a_failed_completion_marker_and_names_it`,
  `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics`,
  `an_accepted_run_whose_completion_rename_fails_stops_the_command`,
  `an_accepted_run_whose_completion_directory_sync_fails_stops_the_command`,
  `a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active`,
  `a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal` (exactly one `.done` journal
  before the recovery),
  `finish_makes_the_outputs_durable_before_the_done_marker`,
  `the_cleanup_failure_happens_after_the_completion_marker_was_published`, `a_removal_failure_after_the_marker_is_cleanup_only`.
- **R1: shown by tests of earlier rounds.**
  `a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning`,
  `recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged`,
  `a_replacement_that_another_writer_holds_open_is_kept_when_the_exchange_is_undone`,
  `a_replacement_edited_in_place_after_the_exchange_is_kept_not_deleted`.
- **R2: shown.**
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name`,
  `when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where` (the message must
  name a path under which the inode survives; round 10, 9b 3.1: the replacement inode is captured at the exchange, differs from the
  displaced one, and keeps its name at the live path after every fault),
  `a_failing_exchange_or_a_read_only_link_keeps_both_inodes_named`,
  `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live`,
  `a_filesystem_without_exchange_fails_the_restore_and_overwrites_nothing`.
- **R3, R5: shown.**
  `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions`,
  `a_cargo_failure_followed_by_a_failed_rollback_stops_the_command_instead_of_falling_back` (asserts that cargo wrote the file, that
  the restore fault was reached and that the refusal is the rollback failure),
  `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others`.
- **K1: shown.**
  `the_kept_files_warning_point_is_exact_and_repeated_runs_agree`,
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning`,
  `an_unreadable_crate_root_gives_an_unknown_inventory_warning_and_never_refuses`,
  `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`,
  `a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives`.
- **K2: shown.**
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction`,
  `a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable`, `a_failing_directory_barrier_is_not_reported_as_preserved`,
  `failing_each_directory_barrier_of_a_preservation_in_turn_never_loses_the_name_or_reports_the_unsynced_chain` (round 10, 9b 3.2: a
  reported preservation needs a `.displaced` name beside the target and a successful barrier of that directory, and at least one
  failed barrier must end in a reported preservation so the branch runs),
  `a_higher_ancestor_the_user_cannot_open_is_never_touched` (skips with a printed reason for a user who opens a directory without
  the read permission), `a_kept_directory_behind_a_symbolic_link_is_synced_by_its_resolved_path`.
- **V1 (7b P1-2): shown.**
  `a_journal_listing_that_fails_in_any_way_stops_the_recovery`, `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`,
  `an_absent_journal_directory_is_no_journals`.
- **V2, amended c: shown.**
  `a_survivor_of_one_journal_stops_the_recovery_of_another`, `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`, `a_group_record_that_cannot_be_read_defers_the_recovery`,
  `a_malformed_group_record_defers_the_recovery`, `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`,
  `a_journal_seen_through_another_view_is_refused_and_kept`,
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`,
  `probe_4_a_scope_is_found_by_its_record_and_a_different_view_is_unknown`,
  `a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal` (the helper gives the journal the view its own namespace has,
  so only the cgroup parent view differs; the refusal must be about that view and not about the journal), and the boot identity:
  `a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
  `another_machine_an_unreadable_identity_no_record_or_the_same_boot_keeps_the_refusal`,
  `the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member` (it exercises a missing snapshot, an incomplete listing
  and a scope with a member, not every other check),
  and round 10: `a_malformed_recorded_identity_is_not_a_reboot`, `a_malformed_current_identity_is_not_a_reboot`,
  `the_current_identity_is_read_strictly_from_its_files`, `a_boot_id_put_over_the_kernels_file_is_not_an_identity`,
  `a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` (9b 2.4: with the cgroup namespace
  and every recorded view equal, a bind mount of another cgroup directory over the directory that held the scope is refused for the
  parent view; the control before the bind clears).
- **V3, V4: partly.**
  Shown: the record is visible, written and synced before the program starts, and the faults of the write and of the sync are each
  reached and surface as the refusal (round 10, 9b 3.6).
  Not shown: the cuts in preparation are hand-built files, not a wrapper stopped at each real boundary of `begin`.
  `the_scope_record_is_visible_before_the_program_starts` (visibility at the boundary),
  `a_record_that_cannot_be_written_or_made_durable_starts_no_program` (a failed write and a failed sync, both containments: the
  sync precedes the start),
  `a_record_that_cannot_be_published_starts_no_program_and_does_not_wait`, `a_child_whose_group_cannot_be_recorded_is_not_started`,
  `a_wrapper_killed_inside_its_start_is_recovered_conservatively` (the unsettled record with an absent scope refuses and says that the
  run did not record its end),
  `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended` (the window after the launcher spawn),
  `a_journal_cut_off_during_its_preparation_is_removed_without_touching_the_lockfile`, `a_recovery_removes_an_empty_journal_directory`,
  `a_nonempty_journal_without_its_record_is_not_treated_as_empty`.
- **V5: shown.**
  `two_roots_with_manifests_are_restored_or_kept_together_per_root` (round 10, 9b 3.3: the new refused copy is read and holds the
  refused output), `a_second_recovery_pass_changes_nothing_and_keeps_the_refused_copy`,
  `a_killed_breaking_run_whose_lock_is_refused_gets_its_manifest_back_too`, `a_journal_marked_done_is_removed_by_the_next_recovery`,
  the unready and ready tests of `journal_recover_tests.rs`.
- **V6: shown.**
  `a_manager_that_does_not_answer_ends_the_discovery_at_its_deadline`, `a_discovery_whose_descendant_holds_the_pipes_ends_at_its_deadline`
  (round 10, 9b 3.7: the descendant records its id, is alive at the return, and the direct program is gone),
  `a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so` (9a D3),
  `a_busy_unsynced_registry_returns_at_once_and_answers_conservatively`,
  `the_route_cache_does_not_make_a_caller_wait_for_a_discovery_in_flight`,
  `a_direct_child_that_cannot_be_reaped_makes_the_run_not_quiescent`, `a_child_that_outlives_its_bound_is_stopped_and_named`,
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete`,
  `a_timeout_returns_promptly_even_when_the_child_ignores_its_input`, `recovery_defers_while_a_live_transaction_holds_the_workspace`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused`.
- **Weaker path: shown.**
  `each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start`,
  `a_weaker_path_journal_is_refused_by_recovery_and_admission_before_it_is_acknowledged`,
  `an_acknowledged_weaker_path_journal_is_settled_like_any_other` (acknowledge, recover, settle),
  `acknowledging_one_of_two_journals_leaves_the_other_blocking_everything`, `a_partial_acknowledgement_does_not_count`,
  `an_acknowledgement_with_three_lines_that_are_not_a_process_id_and_a_time_does_not_count`,
  `an_acknowledgement_the_reader_cannot_make_durable_does_not_count_until_it_can` (round 10, 9b 3.6: the file sync and the directory
  sync of the reader are failed on their own, each fault reached),
  `an_interrupted_publication_leaves_the_journal_unacknowledged_until_it_is_made_again` (a complete temporary beside no published
  name; built by hand from a real acknowledgement),
  `an_acknowledgement_whose_barrier_or_publication_fails_is_not_reported_as_made`,
  `an_acknowledgement_does_not_override_a_live_group_or_unrelated_validation`,
  `acknowledge_refuses_what_is_not_an_open_journal_of_a_workspace` (also a finished journal),
  `the_acknowledge_subcommand_acknowledges_the_journal_the_refusal_named`, `the_weaker_path_still_stops_the_process_group_at_the_bound`.

### What the map does not claim

- The strong-path tests that do not run a transaction rely on the host having a delegated scope (`delegated_route`) and on the
  runner choosing it when it has one; the tests that read a journal record or the program's own group show the choice
  (`assert_scope_record`, the program's group).
- The identity tests give the recovery a different identity; no machine was rebooted.
- The authority check of the boot id was shown in a disposable user and mount namespace on this host; no container runtime
  (`systemd-nspawn`, Docker, Podman, gVisor, a virtual machine) was run, and the nspawn behavior is read from its source
  (`src/nspawn/nspawn.c`, `setup_boot_id`, on the default branch, 2026-11-04).
- The startup overlap of the spawn-isolation row, the shim-level test of A2/R4 and the cuts of V3/V4 at the real boundaries of `begin`
  are not new evidence of this round.
- A flake of the orphan test during this round (a process oracle that read a process that was being reaped) was explained by a
  state read that fails with `ESRCH` as well as `ENOENT`; both oracles now treat both as gone and stop on any other error.
  The explanation was not reproduced afterwards, so it is a conclusion from the code, not from a repeated failure.
- Power loss, a real reboot, and checkpoint and restore are outside the tests.

### Findings the work of rounds 8 and 9 made, outside the list

- `systemd-run` reads the command it is given like a unit's command line: `$$` became `$` and `${HOME}` was replaced by the value of
  the variable (probed 2026-11-04). Every argument and the program are encoded at the point of hand-over, and
  `arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged` runs fifteen awkward arguments, also through a cleared
  environment.
- A cleared environment (`env_clear`, used for Git) cannot be read back from a `Command`, so there is a second entry point,
  `output_within_clean_environment`, and the program is started through `env --ignore-environment`.
- A program that does not exist is found before the launcher starts, so it stays an I/O error.
- An inactive slice has no control group, so the discovery starts the slice first
  (`probe_1_an_inactive_slice_is_started_before_its_control_group_is_asked_for`).
- The journal view uses the unique mount id of Linux 6.8 and later when the kernel gives it.
- Host requirements of the tests: a reachable user manager with `systemd-run`, a cgroup v2 mount, `unshare` with permission to make a
  user and mount namespace (one test), and a C compiler named `cc` (the tests with the thread-leader fixture);
  `CARGO_SHADOW_ALLOW_NO_CGROUP=1` turns the missing-manager case into a printed skip, the others fail loudly.
- Route 2 of the earlier pick (a child of the own group where it carries `user.delegate`) is dropped:
  it needs `clone3` with `CLONE_INTO_CGROUP`, which neither `std` nor `rustix` 1.1.6 offers.
