# What cargo-shadow promises and what it assumes

Status:
amended for round 15 (the review of round 14,
`cargo-shadow-sol-14.txt`):
the time namespace number that round 14 put in the launcher
record is gone again,
because a namespace number is eligible for recycling after its namespace's last reference is released;
a start time that differs from the
record is now unknown,
never gone (guarantee A1);
amended for round 14 (the review of round 13,
`cargo-shadow-sol-13.txt`) with a time namespace number in the launcher record,
superseded by round 15;
amended for round 13 (the owner's pick of 2026-11-04,
"Out of scope,
keep cheap guards") with the extension of assumption a to
mounts and `/proc`,
the launcher question read through one pinned procfs descriptor,
and the number that shows an ended process;
amended for round 12 (the review of rounds 10 and 11,
`cargo-shadow-sol-11a.txt`,
`-11b.txt`) with the launcher question answered only through a
procfs that numbers processes as the caller's namespace does (guarantee A1),
a boot id read from the one object that was checked
(assumption c) and a failed wait that is not a reap (guarantee V6);
amended for round 11 (the owner's pick of 2026-11-04,
"Record the launcher") with the launcher record (guarantee A1:
a launcher that is
gone can never join its scope,
so a recovery that finds it gone reads the scope as final without an acknowledgement);
amended once more after the round 9 reviews (`cargo-shadow-sol-9a.txt`,
`-9b.txt`;
round 10,
written 2026-11-04T08:57:09-05:00) with
the unsettled scope (guarantee A1:
an empty or late-appearing scope is no more a completed lifecycle than an absent one),
the strict
and authoritative boot identity (assumption c and e) and the shared discovery deadline (guarantee V6);
amended again after the round 8 reviews (`cargo-shadow-sol-8a.txt`,
`-8b.txt`) with the boot identity (assumption c and the new
assumption e),
the settled record (guarantee A1) and the acknowledgement's whole record and durability (weaker path);
written 2026-11-04T04:45:55-05:00 after the round 7 re-review (`cargo-shadow-sol-7a.txt`,
`-7b.txt`,
`-7c.txt`,
all "not closed"),
amended after the cross-vendor review of this contract (`cargo-shadow-sol-contract-1.txt`,
"fit after the named edits"),
which also
supplied the acceptance list under the `Acceptance list` heading.
This file specifies transaction safety (acceptance,
rollback,
kept copies,
recovery).
It does not specify the policy semantics,
which are the catalog and pnpm specification:
`doc/decision/rust-dependency-catalog.md` (what is checked,
and when) and `doc/planning/cargo-shadow-update.md` (the mapping of
`cargo update` to `pnpm update` and the pnpm oracle).
The assumptions and the weaker-path rule are the coordinator's picks under the owner's delegation
("Pick options as if I had picked them and invite veto when I say I'm back",
handover "Picks made under the owner's delegation"),
veto open.

Each round the reviewer moved the adversary one level deeper
(a thread,
a mount view,
a same-user process that leaves its group)
because no written contract said what the wrapper promises and what it assumes.
A finding that attacks a stated assumption is answered by the assumption;
a finding that shows a stated guarantee failing is a bug.
Where this file and an earlier section of `doc/planning/cargo-shadow-update.md` disagree,
this file is current.

## Guarantees

Each guarantee is a property,
followed by the test that shows it.
Every test named here exists in the package;
the coverage map at the end says which part of a row has no test yet.
Every guarantee is stated for the normal path,
where a delegated control group is in force.
The weaker path (third part) keeps only the guarantees marked as holding there.

### Acceptance of a lockfile

- **A1.**
  A lockfile that a run left is accepted (left live,
  journal finished,
  command allowed to run) only after the wrapper has shown
  that every process the run started has stopped:
  the direct child was reaped,
  and either the kernel reported `populated 0` for
  the run's scope,
  or the scope is verifiably absent.
  Verified absence counts only for a run that recorded its end (a `settled` line in the journal,
  written after the launcher was
  reaped and the scope shown empty),
  or whose launcher is recorded and shown gone (the launcher record in the `Record the launcher` pick):
  until then an absent scope may be one the launcher has not made yet,
  and a scope that exists
  with `populated 0` may be one whose launcher the manager has not attached yet (the manager creates the control group and attaches
  the process in two steps),
  so a recovery that finds an unsettled scope absent,
  empty,
  or appearing while it checks,
  and has no launcher
  record that shows the launcher gone,
  refuses and asks for an acknowledgement (round 9,
  8a F1;
  round 10,
  9a D1).
  A later look at the scope never turns an unsettled run into a completed one;
  a scope with a member blocks and no acknowledgement
  overrides it.
  The launcher record (round 11,
  the owner's pick "Record the launcher") removes the refusal for a launcher that is gone.
  Right after it spawns the launcher,
  which becomes the program (same process),
  the wrapper appends
  `launcher <pid> <start time> <pid namespace> <scope path>` to the journal's `groups` file:
  the process id,
  field 22 of
  `/proc/<pid>/stat` (clock ticks since boot) and the inode of the process id namespace.
  Field 22 is relative to the boot-time offset of the time namespace of the process that reads it,
  so a recovery in another time
  namespace reads the same live process's start time differently (measured on this host:
  an offset of two hours moved it by 720000
  ticks).
  Round 14 recorded the writer's time namespace number and compared it;
  round 15 found that this proves nothing,
  because a
  namespace number is eligible for recycling after the namespace's last reference is released (a process in it is one reference,
  an
  open descriptor is another),
  and the writer's current time namespace can differ from the one its
  launcher lives in (`time_for_children`),
  so the number can name a later,
  different namespace (the review's schedule is a test:
  `a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone` built the recycled number on this host and the round 14
  rule read the live launcher as gone).
  So no namespace number decides anything about the start time.
  A recovery asks about the launcher before it reads the scope.
  The launcher is gone only when the recovery is in the recorded process id namespace and no process has that id.
  A process that has the id with the recorded start time is alive.
  A process that has the id with another start time is unknown,
  not gone:
  the contract has no way to show that the reader's time
  basis is the writer's,
  so it cannot tell a reused id from a live launcher read on another basis.
  The cost is that a reused id whose start time differs from the record (the pid counter wrapped since the launcher ended) waits for
  an acknowledgement;
  an id that no process has needs none,
  and that is the ordinary case.
  A reused id whose start time happens to equal the record,
  which can occur across time bases,
  is classified alive and blocks.
  A kernel's pidfs inode number could identify the process independently of every namespace,
  but it is unique for the boot only from
  Linux 6.14 on a 64-bit kernel (`fs/pidfs.c`:
  from 6.9 to 6.13 the inode number comes from an `ida` and is reused),
  and the
  conservative rule needs no kernel version,
  so it was not built.
  A process that is gone has no thread that could still be attached to a scope,
  so what the scope shows afterwards (absent,
  empty,
  or populated by a descendant) is final:
  absent or empty is read as ended,
  populated blocks.
  A launcher classified alive (the id with the recorded start time) blocks,
  whatever the scope looks like,
  and no acknowledgement
  overrides it;
  an unreaped zombie counts as alive,
  because only its reaping shows that it ended.
  A launcher that cannot be classified (unknown) waits for an acknowledgement,
  which a person may give;
  a test shows the
  classification,
  not the death,
  of such a launcher.
  The question is answered only when the mounted procfs numbers processes as the caller's own namespace does (round 12,
  11a;
  round 13:
  through one procfs descriptor,
  opened once,
  whose root and whose number for the caller are checked,
  with every later
  lookup made by `openat2` beneath it without crossing a mount or following a link;
  deliberate remounts while a recovery runs are
  outside the contract,
  assumption a):
  `kill` and `pidfd_open` read a process id in the caller's namespace,
  but `/proc/<pid>` reads it in the namespace of the procfs
  mount,
  and `/proc/self/ns/pid` names only the caller's,
  so a child namespace that still uses an ancestor's procfs would make the
  two name different processes.
  A pidfd is opened for the id,
  and the `Pid:` line of its `fdinfo`,
  read through the procfs,
  gives the number this procfs has for
  that very process:
  only when it equals the id is `/proc/<pid>/stat` read,
  and otherwise the answer is unknown,
  never gone.
  When the stat lookup then fails,
  only the number `-1` that the kernel prints for a process that ended and was reaped (measured:
  `the_fdinfo_of_a_pidfd_says_minus_one_only_for_a_reaped_process`;
  an unreaped zombie keeps its number) shows the process gone;
  the
  same number is a hidden process and any other number (another positive id,
  `0`) is unknown.
  The same check precedes the reading of the start time when the line is written,
  so a start time is not taken from another
  process through a procfs that is simply the wrong one.
  Durability of this line:
  it is appended with one `write` and no sync.
  It is read only after the wrapper was killed (the page cache keeps what a process wrote),
  or after a power loss or a reboot,
  where a
  lost or torn line is read as no proof (an unsettled scope without a launcher line waits for an acknowledgement) and the boot rule
  decides first.
  Every proper prefix of the bytes that are appended (the line and its line feed),
  as a torn write leaves it,
  keeps the journal
  blocked (the line fields are in the order pid,
  start time,
  namespace,
  path,
  so a cut leaves a line that is not understood,
  a
  different namespace,
  or a different path;
  a last launcher line without its line feed is not understood either,
  because a cut can
  fall exactly there).
  The later `settled` line is synced and carries the earlier lines with it.
  What the launcher record cannot prove,
  and that therefore still waits for `cargo-shadow acknowledge <journal directory>` (one
  command by a person,
  which tries the workspace lock and does not wait):
  a recovery in another process id namespace than the record,
  or a process that has the recorded id with another start time;
  a journal with no launcher line for an unsettled scope (a journal of an
  older version,
  a line that could not be appended,
  a wrapper killed between the spawn and the append);
  a launcher line for a scope
  the journal does not list;
  a launcher line that is torn or not understood;
  a launcher whose process cannot be inspected (a `/proc`
  that hides it,
  an unreadable stat file).
  A scope with a member blocks and no acknowledgement overrides it.
  The premise under "gone":
  the manager attaches only the launcher it was asked about,
  and a process that has ended cannot become a
  member of a scope;
  this is the process model,
  not a measurement against a manager that handles a request after its sender died.
  The earlier choice of round 10 (refuse every unsettled scope,
  acknowledge) stays as the fallback for the launcher-line cases of guarantee A1.
  Verified absence is proof under amended assumption c and one invariant of the manager:
  systemd removes a scope only when it is empty or after stopping its members
  (probed:
  a scope whose program exited stayed observable with `populated 1` while a descendant lived,
  and vanished 2991 ms later
  when that descendant ended).
  Absence is verified when the cgroup file system is the one the run recorded (assumption c),
  the directory that held the scope
  still exists with the recorded mount id and inode,
  and the scope's name is absent from it.
  A scope name carries a 128-bit random token and is never derived from the process id or the clock,
  so no journal that can
  reference a name can see another scope appear under it.
  Test:
  `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns` and
  `a_writer_in_another_session_is_stopped_before_a_normal_exit_returns` exist;
  `probe_2_a_thread_group_with_an_exited_leader_stays_populated_until_it_is_stopped` exists.
  Holds on the weaker path:
  no (third part).
- **A2.**
  Whenever a run could not show A1 (a member that cannot be killed,
  a failure to stop,
  any failure on the containment path after the
  child started),
  the run reports `NotQuiescent`,
  the scope reports that writers may remain,
  and install and update stop with the
  lockfiles neither rolled back nor accepted,
  the journal kept,
  and the user's command not run.
  Test:
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal` (call-site half) and
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent` (7a F3:
  a failed stop,
  a failed inspection and a
  member that cannot be emptied are each `NotQuiescent`).
  Holds on the weaker path:
  yes for what the process group shows.
- **A3.**
  A lockfile is accepted only if the policy accepted its final text,
  or the refusal stops the command.
  A mandatory failure of finishing (outputs not durable,
  completion marker not durable) stops the command and is never hidden behind
  an earlier error,
  including an operational one that would let the shim run the command again with `--locked`.
  Only the removal of a journal that is already marked done is a warning.
  Test:
  `an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`,
  `a_refusal_survives_a_journal_that_cannot_be_finished`,
  `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics`,
  `a_completion_marker_that_cannot_be_published_stops_the_command`,
  `the_cleanup_failure_happens_after_the_completion_marker_was_published`,
  `an_update_error_with_a_failed_mandatory_phase_reports_both`;
  the real rename and the real parent sync of the completion marker are failed by
  `a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active` and
  `a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal`.
- **A4.**
  The outputs a run leaves (lockfiles,
  journaled manifests) are on disk before the completion marker,
  the marker is durable before
  the journal is removed ("Force at leaving").
  No periodic or background syncing is required ("else background" has no carrier in a command that exits);
  every durability barrier the protocol mandates remains required:
  the outputs,
  the completion marker and its directory,
  the publication of a record before the child it describes,
  the entries of
  a kept copy (K2),
  and an acknowledgement (weaker path).
  Test:
  `finish_makes_the_outputs_durable_before_the_done_marker` exists.

### Rollback

- **R1.**
  A rollback replaces a lockfile only when it still holds what this run produced.
  A file another process wrote meanwhile is left alone with a warning.
  Test:
  `a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning` exists.
- **R2.**
  No inode that was live at a path loses its last name:
  the displaced file and the replacement each get a surviving name before
  anything is unlinked,
  and a displaced file that could not be linked is reported with where it is.
  Test:
  `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live`,
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name` (`EMLINK`,
  `ENOSPC`,
  `EXDEV`,
  `EACCES`),
  `when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where`.
- **R3.**
  A rollback that cannot complete stops the command (`Unsettled`) whether cargo failed or its output was refused,
  so the `--locked` fallback never builds an unrestored lockfile.
  Test:
  `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions`,
  `a_cargo_that_wrote_a_lockfile_and_failed_has_its_output_rolled_back` exist.
- **R4.**
  A rollback or an acceptance never happens while writers may remain (A2).
  Test:
  as A2.
- **R5.**
  A failed root does not hide the warnings of the others and does not stop their restore.
  Test:
  `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others` exists.

### Kept copies

- **K1.**
  Kept copies are never deleted by the tool.
  Their existence,
  count or size never causes a refusal;
  past a count every run prints the count,
  the total size and the folder.
  The inability to preserve a newly displaced file safely still stops the operation (R2,
  R3).
  Test:
  `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`,
  `the_kept_files_warning_point_is_exact_and_repeated_runs_agree`,
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning`.
- **K2.**
  A kept copy's directory entries are durable when preservation reports success (assumption b).
  Test:
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction`,
  `a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable`,
  `a_failing_directory_barrier_is_not_reported_as_preserved`;
  the bind-mount schedule of 7b P1-1 is not a counterexample under that rule and has no test of its own.

### Recovery

- **V1.**
  A recovery or a new transaction proceeds only when the whole set of journals of the workspace was listed:
  an entry that cannot be listed or examined stops it,
  and an entry that vanished mid-listing is unknown,
  not absent.
  Test:
  `a_journal_listing_that_fails_in_any_way_stops_the_recovery`,
  `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`
  (7b P1-2:
  open,
  iteration,
  inspection and an entry-level `NotFound`,
  injected inside the iteration on the second entry,
  with a
  journal already in the partial listing),
  and `an_absent_journal_directory_is_no_journals` for the separate positive case.
- **V2.**
  A recovery or a new transaction does not proceed while any journal of the workspace names a recorded group with a member,
  a record it cannot read or understand,
  a lost record beside other files,
  or a record made where the recorded group cannot be
  examined (assumption c).
  Test:
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`,
  `a_survivor_of_one_journal_stops_the_recovery_of_another`,
  `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`;
  the crash test asserts the refusal reason ("still running"),
  the retained journal and that the run used a delegated scope.
- **V3.**
  A record is durable before the child it describes starts,
  and a child whose record cannot be written is not started.
  Test:
  `a_child_whose_group_cannot_be_recorded_is_not_started` exists.
- **V4.**
  A journal that holds nothing at all is what a cut-off `begin` leaves,
  and a recovery removes it.
  Test:
  `a_recovery_removes_an_empty_journal_directory` exists.
- **V5.**
  A recovery restores per root,
  never splits a lockfile from its manifests,
  saves a refused leftover as `Cargo.lock.refused`,
  and finishes with the same three phases as A3.
  Test:
  the existing recovery tests of `journal_recover_tests.rs` and `journal_protocol_tests.rs`.
- **V6.**
  Every wait that cargo-shadow controls has a deadline:
  the start and the stop of the manager's scope,
  the lifetime of a command,
  the draining of its pipes,
  the reaping of its direct
  child,
  and every lock (a busy workspace lock or internal lock returns at once with the holder,
  it does not wait).
  The discovery of the manager reads its output,
  kills its program and reaps it under the same deadline (the reaping spends what is
  left of it,
  only a successful wait counts as a reap,
  and a program that is not reaped by then,
  or whose wait failed,
  is reported as
  not reaped),
  so a descendant that keeps the pipes open does
  not extend it;
  the caches and registries it keeps are read with `try_lock`,
  so a caller that finds one busy asks again or
  answers conservatively and does not wait for the holder.
  A shutdown that did not succeed within its deadline produces uncertainty (`NotQuiescent`),
  not a longer wait.
  Ordinary blocking kernel and file-system calls (a `read`,
  `write`,
  `fsync`,
  `rename`) have no deadline of their own;
  they are covered by the progress assumption d,
  which mirrors the owner's "Assume the final disk save would never be cut
  short."
  Test:
  `a_child_that_outlives_its_bound_is_stopped_and_named`,
  `recovery_defers_while_a_live_transaction_holds_the_workspace`
  exist.

### What no guarantee covers

- A process that deliberately acts against the wrapper (assumption a).
- Power-loss behavior of a file system that does not honor `fsync`;
  the package has no fault-injection model of power loss.
- The window in which a cargo that is not the shadow builds on an unapproved live lockfile (accepted earlier,
  "Use as few locks as
  possible").
- The user's own command after the checks:
  it runs as long as the user lets it.

## Assumptions

Each assumption is a pick under the delegation,
open to the owner's veto.
A finding that attacks one of them is not a finding while the assumption stands.
Assumptions the code already made are listed last,
so the list is finite.

### a. Run writers stay inside the containment and write the workspace only through it

The processes of a run (a build script,
a proc macro,
a selected executable) are assumed to behave in this way,
whether the
reason is malice or an accident:
they do not migrate outside the recorded containment (no write of their own id into another group's `cgroup.procs`,
no request to
a manager that moves them),
and they do not arrange writes to the workspace through a service outside the containment
(a daemon started by another user session,
a job queued with an outside scheduler,
a request to a build server that already runs).
Ordinary behavior stays covered:
`fork`,
threads,
double forks,
`setsid`,
closed standard streams,
and a descendant that outlives its parent are all inside the
scope and are stopped with it.
The manager is assumed not to migrate surviving members of a scope outside the recorded scope either.
Round 13 (the owner's pick of 2026-11-04,
"Out of scope,
keep cheap guards"):
no program running as the user deliberately changes
mounts or `/proc` while a run or a recovery is in progress,
which covers bind mounts,
overmounts of a file or a directory
(`/proc` itself,
a `/proc/<pid>` directory,
the boot id file,
a cgroup directory) and replacing a procfs view by another one.
This is the same kind of premise as the one above:
a same-user process that does so can equally write `Cargo.lock` directly.
The cheap guards that exist stay,
because they cost a few system calls and cover accidents and configuration:
the procfs is opened once and every numeric lookup of one launcher question goes through that descriptor with `openat2` beneath it,
without crossing a mount point and without following a link (Linux 5.6 or later;
below it the lookup fails and the answer is
"unknown",
never "gone";
the strong path needs 5.14 anyway),
the boot id file is opened without following a link and checked and read
on one descriptor,
and the formats of the identity are strict.
A schedule that needs a deliberate remount while a recovery runs is therefore excluded by this assumption,
not a defect:
round 12's 12a schedules of replacing `/proc` between the check of a pidfd and the read of a stat file,
and of overmounting
`/proc/<pid>` with a native procfs directory of another process.
What stays a defect is anything that needs no tampering:
a procfs of a different namespace that is simply mounted (inherited,
as in a
container),
and a changed number that is not the number of an ended process.

Reason:
a same-user process can write `Cargo.lock` directly after the run,
and no wrapper can prevent that,
so a containment that tried
to resist it would claim a guarantee the wrapper cannot give.
The assumption is about behavior,
not intent:
a tool that hands the write to an outside service breaks it as surely as one that means to.
Probe 3 of round 7 (`probe_3_a_member_of_the_same_user_can_move_itself_out_which_is_assumption_a`) shows the move is possible
and stays as the evidence of the limit.
This retires 7a F5 and 7c F2 as findings,
and the cgroup namespace closer as work.

### b. A pre-existing directory and its path are durably reachable

The directory-entry sync of a kept copy walks up from the kept directory to the first directory that existed before the
transaction began,
syncs it,
and stops.
A directory this transaction created is synced with the directory that lists it,
as `create_directories_synced` does.
A pre-existing directory,
and the reachability of its path from the root of its file system,
are assumed durable:
their entries were committed by whoever made them,
or by an earlier transaction of this tool,
which synced what it created.

The exception is knowledge.
A directory that the tool knows it left unsynchronized in an earlier failed attempt (a directory whose own sync,
or whose parent's
sync,
returned an error) is not treated as durable merely because it exists now;
the walk continues through it and syncs it.
The tool knows this for the lifetime of its process (a registry of the directories whose sync failed).
It does not know it across processes:
a directory left unsynced by a killed or failed earlier process is a pre-existing directory,
and the premise applies to it.

Reason:
the tool cannot know the durability history of a directory it did not make,
and walking to a device boundary or to `/` rests on
a different,
unprovable premise (a bind mount,
a subvolume,
an overlay root).
The rule replaces the round 7 stop at the first parent on another device (7b P1-1).
It also removes the failure of preservation on an unreadable ancestor above the workspace.

### c. The journal and the recorded containment resolve to the same mount and inode in recovery as in the run

When a journal is created,
the wrapper records the mount id and inode of the journal directory (`statx`,
`STATX_MNT_ID` and
`STATX_INO`).
When a child is recorded,
the record also holds the cgroup namespace and the mount id and inode of the directory that holds the
recorded scope (its parent in the cgroup file system).
A recovery compares both with what the same paths give in the recovery process,
and refuses,
naming the journal and the view that
differs,
when anything differs.
Only when they are equal is a recorded scope that is absent read as gone.

Reason:
a recovery in another mount namespace,
or on a path that now leads somewhere else,
cannot read an absent recorded scope as "gone"
(7a F6);
the 7a F6 schedule is a different view of `/sys/fs/cgroup`,
not of the journal,
which is why both checks exist
(the coordinator adopted the second check on 2026-11-04).
Tests:
`a_journal_seen_through_another_view_is_refused_and_kept`,
`a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone` and,
end to end in a new user and mount
namespace with the cgroup namespace unchanged,
`a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal`;
each asserts the reason and that the journal is retained.

The boot identity (the coordinator's pick of round 9,
veto open,
from the reviews of round 8):
the journal's view record also holds the machine id (`/etc/machine-id`) and the boot id (`/proc/sys/kernel/random/boot_id`) of the
system that made it;
it is written before any record of a scope or a process group,
by the one process that writes them,
so it
covers every record.
A recovery that reads the same machine id and a different boot id treats the run's process liveness as discharged,
because a reboot of this machine ended every process of the run (assumption a).
It then does not compare the mount and cgroup views (they are values of the boot that ended),
and it does not ask for an
acknowledgement for the containment.
What it keeps:
journal enumeration,
snapshot validation,
preservation,
the mandatory finish,
and one contradiction:
a recorded scope (its name carries a random token) that has a member after the reboot is not explained by the reboot and blocks.
A different machine id,
an unreadable boot or machine id on either side,
a journal without a recorded identity,
or the same boot id
keep the refusal and the acknowledgement.
Suspend and resume keep the boot id and so give no clearance.

Round 10 (9a D2 and section 4) makes the identity strict and authoritative:

- Both ids must have their format,
  on the recorded side and on the current side:
  the boot id is a lowercase `8-4-4-4-12`
  hexadecimal UUID,
  the machine id is 32 lowercase hexadecimal digits,
  neither is all zero.
  The view record is the line `view <mount>:<inode>` and then exactly one `boot` line and one `machine` line;
  a repeated field or an
  unknown line leaves the record without an identity.
  A malformed id is not evidence of a reboot:
  it keeps the refusal (the reviewer's schedule:
  `boot not-a-boot-id` beside a live
  process group).
- The boot id file must be the kernel's own file:
  it is opened once,
  without following a final symbolic link (round 12,
  11a:
  a link to
  a genuine procfs file passes every check of its target),
  and on that descriptor `fstatfs` reports procfs (`0x9fa0`) and `statx`
  reports that the file is not the root of a mount (`STATX_ATTR_MOUNT_ROOT` clear,
  and a kernel that does not report the attribute
  is an error);
  the text is then read from the same descriptor,
  so a mount that appears after the checks cannot change what is read.
  The record's first line must be exactly `view <mount>:<inode>` with two decimal numbers.
  Measured in a disposable user and mount namespace on 2026-11-04:
  a bind mount of a regular file over the boot id file reports
  another file system,
  a bind mount of another procfs file reports a mount root,
  and a read-only bind of the directory that holds
  the file (the way container runtimes mask paths) reports neither,
  so it is accepted.
  A file that fails the check is no identity:
  unreadable,
  the refusal stays.
  An identity is not recorded when the check fails at `begin`,
  so the journal has none.
- The premise that remains,
  stated here as part of this assumption:
  the kernel that serves the process is the kernel that ran the
  writers,
  and its procfs is not emulated.
  A sandbox that emulates `/proc` in user space or in another kernel (and reports procfs for its own boot id) is that kernel for
  this purpose;
  a boot id that a runtime replaces by a bind mount (`systemd-nspawn` does,
  with a random id per container start) fails
  the check that the boot id file is the kernel's own file,
  so such a container gets the acknowledgement,
  not the clearance.
Tests:
  `a_malformed_recorded_identity_is_not_a_reboot`,
  `a_malformed_current_identity_is_not_a_reboot`,
`the_current_identity_is_read_strictly_from_its_files`,
  and,
  end to end in a new user and mount namespace,
`a_boot_id_put_over_the_kernels_file_is_not_an_identity` (a regular file,
  another procfs file,
  a symbolic link to a genuine procfs
file inside a replacement directory,
  and a tmpfs over the directory are replaced and keep a live group blocking;
  a read-only bind of
the directory is not a replacement),
  `a_view_header_that_is_not_mount_and_inode_gives_no_identity`.
No test makes a mount appear between the checks and the read;
  that the read is from the checked descriptor is by construction.
Tests:
  `a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
`another_machine_an_unreadable_identity_no_record_or_the_same_boot_keeps_the_refusal`,
`the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member`.
The tests give the recovery a different identity;
  no machine was rebooted.

Two other facts,
stated so they are not found by surprise:

- The mount id is assigned for the life of one boot (the unique id of Linux 6.8 and later when the kernel gives it,
  the reusable id
  otherwise),
  so within one boot it identifies the mount;
  across a reboot the boot identity above decides,
  not the mount id.
- With the reusable id,
  a mount id that was reused for another mount together with a coinciding inode would be accepted within a boot;
  the unique id removes that case on kernels that have it.

### d. Ordinary operating-system calls make progress

A blocking kernel or file-system call that cargo-shadow makes (`read`,
`write`,
`open`,
`fsync`,
`rename`,
`unlink`) returns
in finite time.
Reason:
no deadline can be put on such a call from inside the process without isolating it behind a supervised helper,
and the owner's own
rule is the same premise for the disk:
"Assume the final disk save would never be cut short."
The assumption does not weaken V6:
the waits that cargo-shadow controls are bounded (V6),
and the calls this assumption covers are
not waits that cargo-shadow chooses.

### e. One machine at a time, and no checkpoint and restore

A workspace is used from one machine at a time:
the workspace lock reaches only one kernel,
so a second machine could not see the
first one's lock,
and a different boot id on another machine says nothing about the first machine's processes (8b:
host B recovering
over host A's live writer).
That is why the boot identity discharges liveness only for the same machine id.
Checkpoint and restore of processes (and the restoration of a whole virtual machine's state) is out of scope:
it can bring back
processes that the boot identity counts as ended.
A copied `/etc/machine-id` (a cloned host) is the same as one machine under this assumption.
The boot id is trusted as the kernel's only as assumption c says:
the file must be the kernel's own and not put over,
and the
kernel is not an emulation.

### Assumptions the code already makes

- Linux only (owner:
  "Ignore Win and mac until I say so.").
- Kernel 5.14 or later for `cgroup.kill` and cgroup v2 mounted at `/sys/fs/cgroup`;
  elsewhere the weaker path.
- Every writer of the workspace's lockfiles that matters goes through the shim and takes its workspace lock;
  a plain cargo is not stopped.
- The file system honors `fsync` and directory `fsync`.
- The clock does not matter for correctness:
  the ownership of a file is never decided by a time stamp alone.

## Where an assumption fails, or no delegated group can be had

### The weaker path

When no delegated control group can be had
(the user manager is not running or cannot be reached,
`systemd-run` is missing,
cgroup v2 is absent,
the manager's cgroup
directory has no `cgroup.kill`,
the kernel is older than 5.14),
the run goes on in a process group of its own.
It promises what pnpm promises:
the process group is killed at the bound and at the exit,
and no proof is claimed that a process that left the group has stopped.
A scope that the manager refuses to start after discovery succeeded is not this path:
the program does not run,
the run's end is
recorded,
and a recovery needs no acknowledgement (`a_directory_without_a_kill_file_takes_the_weaker_path_and_a_refused_scope_runs_nothing`).
"The own group is not delegated" was a class of the dropped second route (a child of the own group,
see "The spawn route");
delegation is requested for each scope with `Delegate=yes`,
so no separate class names it.
The notice says so in one line that names the missing capability,
for example
"cargo-shadow:
no delegated control group (the user manager cannot be reached);
a process that leaves the process group cannot be
stopped or found,
so quiescence is not shown".

The journal records the weaker path (`pgroup starting` before the child starts,
`pgroup <id>` after).
Recovery of such a journal refuses until a person acknowledges it.
The refusal message names the journal directory and the command that acknowledges it:

```sh
# acknowledge a journal of the weaker path after confirming that no cargo of that run is running
cargo-shadow acknowledge <journal directory>
```

The acknowledgement asserts that every process or worker of this run that is capable of modifying the workspace has stopped.
"No cargo is running" is not enough:
a descendant,
a build script or a worker that cargo started counts.
The acknowledgement:

- is a decision of a person;
  nothing acknowledges by itself,
  by age or by the absence of the recorded group;
- overrides the uncertainty of the containment context only:
  a weaker-path record,
  a record that is missing,
  unreadable or not understood,
  a recorded containment that cannot be examined,
  and a mount or namespace view that differs;
- does not override a failed enumeration of the journals,
  an invalid or missing snapshot,
  a failed mandatory durability barrier,
  or a recorded group that is positively observed to have a member;
- is written durably into the journal under the workspace lock (a file `acknowledged` published by an atomic rename,
  with the process
  id and the time of the acknowledging command),
  and a recovery then settles the journal as it settles any ready journal,
  including all of its own validation;
- counts only as the whole versioned record (the header,
  a process id,
  a time,
  each line ended by a line feed),
  and only after the
  reader has made the file and its directory entry durable itself;
  a marker that is incomplete,
  or that the reader cannot make
  durable,
  does not count;
- is per journal,
  so one acknowledgement never covers another journal,
  and a new transaction beside an unacknowledged journal
  of the weaker path is refused with the same message;
- if its publication is interrupted,
  leaves the journal unacknowledged (an unreadable or partial `acknowledged` file does not
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
  the wrapper may accept or restore a lockfile that a process then rewrites,
  or may read a scope as empty while a writer lives
  elsewhere;
  that is the same state as any same-user process writing `Cargo.lock`,
  and no message is promised.
- Assumption b fails:
  the runtime guarantee stays (no inode loses its last name while the process runs,
  R2),
  and every crash-durability guarantee
  about the directory entries of a kept copy is withdrawn:
  after a power failure the entry,
  and the copy it names,
  may be missing.
- Assumption d fails:
  a call blocks without end,
  and so does the command;
  nothing in this file promises otherwise.
- Assumption c fails:
  recovery refuses and names the journal;
  a person resolves it by acknowledging,
  as above.

## The spawn route

1.  **The only route to a control group:
    a transient user scope with `Delegate=yes`.**
    systemd's own rule (systemd.io/CGROUP_DELEGATION,
    read 2026-11-04):
    "Never create your own cgroups below arbitrary cgroups systemd manages,
    i.e cgroups you haven't set `Delegate=` in."
    cargo-shadow starts the program as `systemd-run --user --scope --unit=<name> --slice=<slice> --property=Delegate=yes --collect
    --quiet -- <program> <arguments>`,
    so that the manager,
    not a thread of cargo-shadow,
    puts the program in the new scope.
    (The equivalent `StartTransientUnit` call is not used:
    it needs a bus client,
    and `systemd-run` has the same effect.)
    The manager's real hierarchy is discovered,
    not assumed:
    the slice's group is
    `systemctl --user show <slice> --property=ControlGroup --value`,
    asked once per process,
    and the scope's group is that path plus
    `/<name>.scope`.
    The name is `cargo-shadow-<128-bit random token>` and is never derived from the process id or the clock.
    The direct child is killed and reaped before the scope is stopped,
    so that no process of cargo-shadow's own can still become the
    program after the stop;
    only then does the run append `settled <scope path>` to the journal.
2.  **A child of the own group is not built.**
    The earlier pick (a child of the own group where it carries `user.delegate`) needs a way to start a process inside a group
    without a thread of cargo-shadow writing `cgroup.procs`;
    the only such way is `clone3` with `CLONE_INTO_CGROUP`,
    which
    neither `std` nor `rustix` 1.1.6 offers.
    Where the scope route is unavailable,
    the run takes the weaker path.
    (Open for the coordinator to confirm:
    route 2 is dropped,
    not queued.)
3.  No route moves the wrapper,
    or any thread of it,
    between groups.
    This removes the spawn lock and the shared-process migration of 7a F1 and F2 by construction.
4.  **The scope keeps the caller's resource limits (added 2026-11-07).**
    The manager makes the scope under its own slice,
    so the limits of the control group that started the run do not reach it:
    an audit of every `systemd-run` builder in the repository found that a build inside `systemd-run --user --scope` with
    `CPUQuota`,
    `MemoryMax`,
    `TasksMax` and `IOWeight` caps ran uncapped.
    Repository rule RXI:
    every command an agent does not consider trivial runs in its own cgroup with CPU,
    memory,
    process-count and
    disk-weight limits,
    in every agent.
    Precedent:
    `package/wg/cli-quicker-exempt/src/inherited_limits.rs` walks the origin's ancestors and takes the tightest value of each
    limit.
    The guarantee:
    the scope of a run carries,
    for each of `MemoryMax` (`memory.max`),
    `CPUQuota` (`cpu.max`),
    `TasksMax` (`pids.max`)
    and `IOWeight` (the `default` line of `io.weight`),
    the tightest finite value on the caller's own group and on each ancestor up to,
    and not including,
    the deepest ancestor it shares with the slice (the shared ancestors' limits reach the scope through ancestry).
    Absent limits stay absent,
    and no size is invented.
    An I/O weight at or above the kernel default is not carried:
    the scope may be throttled more than its caller,
    never less.
    The quota is passed as a percent with two decimals,
    rounded down,
    never to nothing,
    never looser than the caller.
    The limits are read once per process,
    with the route.
    A limit file that cannot be read or understood is a missing capability,
    not "no limit":
    the run takes the weaker path,
    which stays in the caller's own group,
    so the caller's limits still apply.
    Not carried:
    `memory.high`,
    `memory.swap.max`,
    the per-device `io.max` limits,
    CPU weight and cpusets.
    Port,
    not reuse:
    `package/cargo-shadow/cli/src/inherited_limits.rs` copies the four limits of the precedent,
    because the precedent crate
    is a binary without a library and each crate stands on its own (its own `[workspace]`,
    published separately).
    Tests:
    `package/cargo-shadow/cli/src/inherited_limits_tests.rs` (a fixture cgroup tree,
    absent limits,
    the `default` line,
    the
    launcher's argument list,
    and a real scope that reads its own `memory.max`,
    `cpu.max`,
    `pids.max` and `io.weight`).

### Probe of the scope route (2026-11-04, Linux 7.2.15, systemd 262.1)

Run inside `systemd-run --user --scope --property=CPUQuota=100% --property=MemoryMax=512M --property=TasksMax=200
--property=IOWeight=10`;
scripts `probe-scope-spawn.ts` and the commands below in `${HOME}/temp/agent/cargo-shadow-r7/`.

Result:
the route works as the contract needs.

- **The program runs in the new scope and nothing of cargo-shadow's wrote a `cgroup.procs`.**
  The program's own `/proc/self/cgroup` was `.../app.slice/cargo-shadow-probe-<pid>.scope`.
  The parent's group was unchanged.
  `strace -f -e trace=openat,openat2,write` over `systemd-run --user --scope ... -- true` (73 lines) shows no open of a file named
  `cgroup` at all:
  `systemd-run` asks the manager over the bus,
  and the manager moves the process.
- **The 7a F1 schedule does not occur.**
  An ordinary child started from a second thread of the parent while the scope spawn ran stayed in the parent's group.
- **The scope path is predictable before the start.**
  With `--unit=<name> --slice=app.slice`,
  the scope is `<ControlGroup of app.slice>/<name>.scope`;
  `systemctl --user show app.slice
  --property=ControlGroup --value` gave `/user.slice/user-1000.slice/user@1000.service/app.slice`,
  equal to the prefix of the own
  group up to `user@<uid>.service` plus `/app.slice`,
  but the contract uses the answer of the manager,
  not the prefix.
  The predicted path existed and matched the program's group,
  so the record can be written and synced before the program starts
  (V3).
  The inode of the scope is known only after the start,
  so the record carries the path of a unique name (a second start of a name that was
  still loaded failed with "Unit ... was already loaded or has a fragment file",
  and the random token makes a repeated name
  impossible in practice),
  not an inode.
- **The program replaces `systemd-run`:
  same process,
  same pipes,
  same exit status.**
  `exit 7` in the program gave status 7;
  stdout was the program's.
- **Descendants stay in the scope.**
  A `setsid` descendant (own session) was listed in the scope's `cgroup.procs`.
- **The scope directory is the user's and the stop works through it.**
  Owner uid 1000;
  `cgroup.kill` written by the user stopped the members,
  and with `--collect` the manager removed the scope
  directory 1.8 ms after the kill.
  An absent scope path is therefore the normal end of a run,
  and "gone" for a recovery.
- **Start failure.**
  Without a reachable user manager,
  `systemd-run` printed "Failed to connect to user scope bus" and exited 1 without running the
  program.
  The program's own exit status 1 is indistinguishable by status alone.
  This costs a diagnostic,
  not safety:
  a program that did not run has no member,
  `populated` is never 1,
  and the run is reported as
  an ordinary failed command.
  A later round may separate the two by preflight (one `systemd-run ... true` per process) or by `StartTransientUnit` directly.
- **Cost.**
  The start of a scope took 7.0 ms median (6.2 minimum,
  14.3 maximum,
  15 runs) against 1.9 ms for a plain `true`,
  measured
  earlier the same day;
  the whole-run cost with this route is not measured yet.
- **The `user.delegate` extended attribute.**
  Present on `user@1000.service` (`user.delegate="1"`),
  absent on `app.slice`,
  absent on the harness scope `claude-code-bash`,
  and **absent on a transient scope started with `Delegate=yes`** (only `security.selinux` was listed).
  So the attribute identifies the user manager's own subtree,
  not the scopes under it;
  point 2 of the route applies in practice to a process whose own group is `user@<uid>.service` or a descendant that carries the
  attribute,
  and a scope is made through point 1.
  Whether any other delegated cgroup in the wild carries the attribute was not probed.

Probe 2 (2026-11-04T05:10-05:00):
a scope whose program exited while a `setsid` descendant lived stayed observable with
`populated 1` and the descendant's process id,
and vanished about 3 s later when the descendant ended.
That is the invariant A1 relies on for verified absence.

Not probed,
so the code does not rely on it:
the behavior on a host with a user manager but a read-only `/sys/fs/cgroup`.

## Finding mapping

- Retired by an assumption:
  7a F5,
  7c F2 (a),
  7b P1-1 (b);
  answered by assumption c with its two checks:
  7a F6.
- Closed by the weaker-path rule:
  7c F1,
  7a F4.
- Closed by the spawn route:
  7a F1,
  7a F2 (`two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary`).
- Bugs regardless of the contract:
  7a F3 (A2) and 7b P1-2 (V1).
- Test-evidence notes of the three reviews (handshakes instead of sleeps,
  entry-level injection,
  real rename and sync failures,
  crash-test assertions):
  to be applied with the tests named above.

## Acceptance list

This is the cross-vendor reviewer's finite checklist (`cargo-shadow-sol-contract-1.txt`,
section 4),
copied verbatim in substance.
The rule that comes with it:
**a new counterexample may reopen a row of this list,
but it does not add a row.**
Concurrency tests need handshakes at the relevant boundary,
not elapsed sleeps.
Strong-path tests must establish that they actually used delegated containment.
These are acceptance tests,
not a claim that finite testing mathematically proves every execution.

- **A1.**
  Zombie leader with a live writer thread;
  pipe-closing `setsid` or double-fork writer;
  normal exit and timeout.
  Acceptance occurs only after the specified emptiness proof and direct-child reap.
- **A1,
  V3 (spawn isolation).**
  Overlap two bounded starts and an unrelated ordinary spawn inside startup.
  The wrapper stays in its original group;
  runs have separate boundaries;
  unrelated work survives cleanup.
  Exercise every supported spawn route.
- **A1,
  A2 (scope lifecycle).**
  Direct command exits while a descendant remains.
  The scope either remains observable or the manager stops that descendant without moving it elsewhere.
  Also exercise rapid empty-scope collection and the chosen valid-absence rule.
- **A2,
  R4 (7a F3).**
  Start the child and a writer,
  then fail containment or startup completion,
  membership inspection,
  or stopping.
  Also simulate a member that cannot be stopped within the bound.
  Require `NotQuiescent`,
  writer uncertainty,
  retained journal,
  no acceptance,
  no rollback,
  and no user command.
  No post-start downgrade to ordinary operational failure.
- **A3,
  A4.**
  Approved,
  refused and operational-error outcomes;
  fail output synchronization,
  the actual completion rename,
  and the actual
  completion-parent sync.
  Mandatory failures stop shim fallback and preserve applicable diagnostics.
  Verify outputs,
  then marker,
  then marker-directory barrier,
  then removal ordering.
  Cleanup-only failure is a warning only after durable completion.
- **R1.**
  Another writer changes the lockfile before rollback and at the replacement or conflict boundary.
  Its result remains intact,
  with a warning;
  rollback does not silently install the snapshot over it.
- **R2.**
  Fault every preservation or link stage with the queued `EMLINK`,
  `ENOSPC` and `EXDEV` cases,
  plus relevant rename and sync
  failures.
  Inventory the displaced and replacement inodes after each failure:
  neither loses its last protected name;
  stranded locations are
  reported.
- **R3,
  R5.**
  Both refused output and cargo failure followed by rollback failure must stop the command,
  including `--locked` fallback.
  With multiple roots,
  fail one restore while another restores and emits warnings;
  preserve all diagnostics and continue
  independent restores.
- **K1.**
  Below,
  at and above the warning threshold,
  across repeated runs:
  correct count,
  size and directory;
  no count-based refusal or
  deletion.
  Inventory kept names,
  colliding names and stranded temporaries.
- **K2.**
  Create nested kept directories beneath a pre-existing anchor.
  Verify every required directory barrier through that anchor,
  and no traversal above it,
  including an unreadable higher ancestor.
  Fail each required barrier:
  do not report preservation success or discard the remaining protected name.
- **V1 (7b P1-2).**
  Fail opening,
  iteration and entry inspection independently.
  In particular,
  accumulate a live journal,
  then inject an entry-level `NotFound` for another entry.
  Both recovery and `begin` must stop without discarding the partial listing.
  Initial absence of the journal directory remains a separate positive case.
- **V2,
  amended c.**
  A survivor in one journal blocks recovery and admission involving another.
  Exercise unreadable,
  malformed and missing records beside other files.
  Independently change the journal view and the recorded cgroup-parent view,
  including the same-cgroup-namespace bind-mount
  schedule:
  refuse with the intended reason and retain the journal.
  An unsettled scope that is absent,
  empty or appears during the check refuses unless its launcher is recorded and gone;
  a live
  launcher blocks without an acknowledgement;
  a malformed or replaced boot identity never clears a live group (rounds 10 and 11;
  counterexamples to this row,
  not a new row).
- **V3,
  V4.**
  Fail each record-publication or durability step:
  no payload starts.
  Kill the wrapper in preparation and start windows and verify conservative recovery.
  A genuinely empty journal is removable;
  a nonempty journal lacking required records is not treated as empty.
- **V5.**
  Recover multiple roots with their manifests and lockfiles kept together.
  Preserve refused leftovers without overwriting an existing `Cargo.lock.refused`.
  Exercise prepared,
  ready and done states,
  repeated recovery,
  and the same mandatory-finish versus cleanup distinctions as A3 and A4.
- **V6.**
  Bound manager startup,
  command lifetime,
  pipe draining,
  shutdown and reaping,
  including descendants holding pipes open.
  Busy workspace and internal locks return rather than wait.
  Unsuccessful shutdown produces uncertainty,
  not an unbounded wait.
- **Weaker path (A2,
  V2,
  V3,
  V5,
  V6).**
  Each unavailable-capability class produces the promised notice and the pre-start weaker-path record.
  Both `pgroup starting` and completed-but-gone records block recovery and admission without acknowledgement.
  The advertised CLI command works,
  uses the workspace lock,
  durably acknowledges exactly one journal,
  survives interrupted
  publication conservatively,
  and cannot bypass unrelated validation failures.
  Acknowledging one of two journals leaves the other blocking.

## Coverage map

Written 2026-11-04T10:21:27-05:00 against the tip that carries it (round 15);
the round 14 text it replaces was written at 2026-11-04T10:02:46-05:00.
A row is "shown" when each of its schedules has a test that fails when the guarantee is removed or that asserts the observable
result;
"partly" when a named part has no test yet.
No row is certified by this map:
the cross-vendor review of round 15 has not run,
and the reviews of rounds 10 to 14 said that the
count of shown rows was not established by the attachments they saw,
whose unchanged tests they could not see.
Counts:
16 rows,
16 shown,
0 partly,
0 not yet.
The certification pass (`cargo-shadow-cert-sol-1.txt`,
`-2.txt`,
`-3.txt`) found every row partly shown or shown with named false
greens,
and the map marked the rows partly until the fixes landed;
the fixes of round 16 landed (a list per row follows,
each with
the tests that discriminate;
the planted regressions that turned the old assertions red are in the red-first list),
and the pass is
to judge them again:
no row is certified by this map.
The shared helper `journal_dirs` now stops on every listing error except the initial absence of the journal directory
(`the_journal_listing_helper_stops_on_an_unreadable_directory_and_reads_only_absence_as_empty`,
red against the old helper).
Suite counts at the round 19 code tip (written 2026-11-04T16:13:00-05:00):
the release gate (`mise run //package/cargo-shadow/cli:test`,
run
in a scope with the installed shadow first on PATH,
as on main) ran 648 tests:
648 passed,
0 failed,
16 skipped (the `#[ignore]`
helper entries,
three more than before for the syscall and notice helpers);
`lint:clippy` with warnings denied is clean;
logs
`/home/user/temp/agent/r19-gate-final.log`,
`r19-gate-busy.log` (the same gate under an I/O pressure of 31 to 38 percent:
647 passed in
17 s),
`r19-clippy.log`.
Suite counts at the round 18 code tip `c82ee474fc` (written 2026-11-04T15:22:00-05:00):
the debug run and the release gate each ran
634 tests:
634 passed,
0 failed,
14 skipped (the thirteen `#[ignore]` helper entries of before and the helper of
`each_missing_capability_class_prints_the_notice_on_the_real_standard_error`);
`lint:clippy` with warnings denied is clean;
logs
`/home/user/temp/agent/r18-debug.log`,
`r18-gate.log`,
`r18-clippy.log`.
Suite counts at the round 17 code tip `8e009bc256` (written 2026-11-04T14:50:00-05:00):
the debug run and the release gate each ran
618 tests:
618 passed,
0 failed,
13 skipped (the same thirteen `#[ignore]` helper entries);
`lint:clippy` with warnings denied is
clean;
logs `/home/user/temp/agent/r17-debug.log`,
`r17-gate.log`,
`r17-clippy.log`.
Suite counts at the round 16 code tip `9cbee37b52`:
the debug run (`mise run //package/cargo-shadow/cli:test:debug`) and the release gate
(`mise run //package/cargo-shadow/cli:test`) each ran 609 tests:
609 passed,
0 failed,
13 skipped;
`lint:clippy` with warnings
denied is clean.
The 13 skipped are the `#[ignore]` helper entries that thirteen other tests start as child processes (their names are in the `ignore`
attributes of the test files);
they are not run on their own.
Logs:
`/var/home/user/temp/agent/cargo-shadow-r10/logs/r16-debug1.log`,
`r16-gate2.log`,
`r16-clippy2.log`.
The first full gate of round 16 failed once,
in `a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone`,
because another
session on the host took the freed namespace number (`r16-gate1.log`);
the schedule is now tried again up to eight times.
Red first was done where the old code could show red:
the round 7 red commit `939dcb31d1` holds counterexamples of rows A1,
A3,
A4,
V1 and V2 against the round 6 code.
In rounds 8 to 10 the new guards were removed from the committed tests and the tests failed,
then the guard was put back:
round 8:
`a_journal_listing_that_fails_in_any_way_stops_the_recovery`,
`a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`,
`arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged`;
round 9:
`a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended`,
`a_discovery_whose_descendant_holds_the_pipes_ends_at_its_deadline`,
`a_busy_unsynced_registry_returns_at_once_and_answers_conservatively`,
`an_acknowledgement_with_three_lines_that_are_not_a_process_id_and_a_time_does_not_count`,
`an_acknowledgement_the_reader_cannot_make_durable_does_not_count_until_it_can`,
`an_unreadable_crate_root_gives_an_unknown_inventory_warning_and_never_refuses`,
`a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
`the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member`;
round 16 (the planted regressions of the certification fixes;
each log is in `logs/`):
`the_direct_child_is_reaped_after_a_normal_return_and_after_a_timeout` and `a_zombie_leader_with_a_live_writer_thread_is_stopped_at_the_bound`
(the reap replaced by a kill that returns success without a wait:
`r16-f-red.log`),
`a_failed_startup_completion_with_a_writer_running_is_not_quiescent_when_the_scope_cannot_be_emptied` (a post-start failure that returns
the ordinary error without the stop check:
`r16-b-red.log`),
`the_record_is_written_made_durable_and_only_then_the_program_is_launched` and the trace of `a_record_that_cannot_be_written_or_made_durable_starts_no_program`
(a sync before the append:
`r16-d-red.log`),
the control `a_wrapper_moved_before_the_spawn_puts_the_launcher_in_the_wrong_group_which_the_boundary_test_sees`
(the move planted by the test itself),
the three V2 tests of `round_sixteen_tests.rs` (malformed lines ignored,
the live-member check removed:
`r16-g-red.log`;
the old tests
stayed green under the same plant),
the changed-view cases and the bind substitution (the view comparisons removed:
`r16-h-red.log`),
the lock contenders (a wait planted into the lock:
`r16-i-red.log`),
`an_unsuccessful_shutdown_returns_not_quiescent_at_the_quiescence_bound`
(the stop deadline stretched:
`r16-k-red.log`),
the A3/A4 tests (the join of the operational error with the failed finish removed:
`r16-l-red.log`),
`a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name` (the replacement removed on the
fallback:
`r16-m-red.log`),
`kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning` (the beside-target
bytes dropped:
`r16-o-red.log`),
`the_walk_stops_at_the_first_directory_that_existed_before_the_transaction` (the barrier of the kept
directory after the link skipped:
`r16-p-red.log`),
`an_active_recovery_finishes_with_its_own_mandatory_phases_and_tells_cleanup_apart`
(the finish replaced by a direct removal:
`r16-q-red.log`),
and the journal listing helper test (red against the old helper,
in the first run of that test,
whose log was overwritten by the green run).
Without a planted regression,
because none could be built cheaply,
and so evidence of the assertions only:
the in-place kept file of
R1,
the rollback test of R3/R5,
the weaker-path tests of round 16,
the stalled startup,
the process oracle,
the killed-wrapper test of
V2 and the timeout test of V6.
round 15 (red at the commit "red control for a recycled time namespace number",
green at the commit "a start time that differs from the
launcher record is unknown,
never gone";
`logs/r15-red.log`):
`a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone`;
round 14 (red at the commit "red control for the launcher start time read from another time namespace",
green at the commit
"the launcher record carries the time namespace of its writer"):
`a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone`
(red `logs/r14-red.log`);
round 13 (red at `69be31fb74`,
green at `77e48d7c5e`):
`only_the_ended_process_number_shows_a_gone_process` (another positive
number was read as gone);
round 12 (red at `301faaf776`,
green at `87da96a05f`):
`a_wait_that_fails_is_not_a_reap`,
`a_view_header_that_is_not_mount_and_inode_gives_no_identity`,
`a_procfs_of_another_pid_namespace_never_reads_as_a_gone_launcher`
(a new user and pid namespace that keeps the ancestor's procfs,
with a procfs of its own as the control),
the `symlink` mode of
`a_boot_id_put_over_the_kernels_file_is_not_an_identity`;
and the line feed rule of a launcher line,
red with the fix switched off
(`logs/r12-red2.log`,
commit `8ea7a0b63c`):
`a_torn_launcher_line_never_reads_as_a_gone_launcher`;
round 11 (red at `37c5524be1`,
green at `e6fab10dd4`):
`a_gone_launcher_lets_the_recovery_read_the_scope_as_final`,
`a_live_launcher_blocks_whether_its_scope_is_absent_or_empty`,
`a_torn_launcher_line_never_reads_as_a_gone_launcher`,
`a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone` (four red on the scaffold that parsed the record and ignored it);
round 10 (each red on the committed old code or with the guard removed,
then green;
commits `6a6513b482`,
`1318ffd101`,
`147f558d13`):
`an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended`,
`a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended`,
`a_malformed_recorded_identity_is_not_a_reboot` (the lenient reader put back for the red run),
`a_malformed_current_identity_is_not_a_reboot`,
`a_boot_id_put_over_the_kernels_file_is_not_an_identity` (the authority check made to pass for any existing file,
for the red run),
`a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so`,
`a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` (the parent-view comparison removed
for the red run).
New behavior with no old code to fail (the scope route,
the acknowledgement,
the two views,
the boot identity as a whole) has tests
written with the code.

- **A1:
  shown.**
  Round 16 (certification group 1):
  `the_direct_child_is_reaped_after_a_normal_return_and_after_a_timeout` (the program publishes its
  own process id and a checked `waitpid` gives `ECHILD` afterwards;
  red with the reap planted away),
  `a_zombie_leader_with_a_live_writer_thread_is_stopped_at_the_bound` (the zombie-leader fixture through the timeout,
  with the same
  handshake,
  worker oracle and recorded-scope assertion,
  and the direct child reaped).
  `a_zombie_leader_with_a_live_writer_thread_is_stopped_before_a_normal_exit_returns` (the fixture says when its leader is a zombie
  and records its worker's thread id;
  the program's group is recorded and must be a delegated scope),
  `the_thread_oracle_sees_a_live_worker_behind_a_zombie_leader_and_its_end` (the control of the oracle),
  `a_writer_in_another_session_is_stopped_before_a_normal_exit_returns`,
  `a_writer_in_another_session_is_stopped_at_the_bound`,
  `probe_3_a_setsid_and_double_fork_descendant_stays_a_member`,
  `orphans_of_parents_that_exit_while_the_run_ends_are_all_stopped` (forty distinct orphans,
  each recording its own process id;
  the
  program's group must be the scope the journal recorded for the run and differ from the wrapper's own group,
  round 10,
  9b 3.5),
  `a_direct_child_that_cannot_be_reaped_makes_the_run_not_quiescent`.
  Round 10 (9b 3.7):
  the process oracle counts only `ESRCH` as an ended process,
  the thread fixture publishes a complete thread id by
  rename,
  and the writer tests of `src/bounded_contain_tests.rs` assert the scope record of their run.
- **A1,
  V3 (spawn isolation):
  shown.**
  Round 16 (certification group 2):
  `two_launchers_held_at_the_launch_boundary_and_an_unrelated_spawn_share_the_wrappers_group` (two
  bounded runs with their launchers held at the boundary where the manager would associate them with a scope,
  an unrelated spawn
  between them;
  the wrapper,
  the unrelated child and both held launchers are in the wrapper's original group),
  and its control
  `a_wrapper_moved_before_the_spawn_puts_the_launcher_in_the_wrong_group_which_the_boundary_test_sees` (the regression the pass
  describes,
  planted,
  is seen).
  `two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary`,
  `probe_5_the_spawner_never_changes_group_and_an_unrelated_child_stays_out`,
  `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended`.
- **A1,
  A2 (scope lifecycle):
  shown.**
  `probe_2_a_scope_stays_observable_while_a_descendant_outlives_the_program`,
  `probe_2_a_scope_that_is_collected_at_once_is_accepted_as_empty_by_verified_absence`,
  `a_settled_run_whose_scope_was_collected_is_read_as_ended` (round 10,
  9b 3.4:
  it waits until the recorded scope is absent),
  `an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended` (creation before placement:
  the control group exists with
  `populated 0`),
  `a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended` (the race between two observations),
  and the launcher record of round 11:
  `a_launcher_record_reads_back_and_a_damaged_one_is_not_understood`,
  `the_launcher_question_tells_gone_from_alive_reused_other_namespace_and_zombie`,
  `a_live_launcher_blocks_whether_its_scope_is_absent_or_empty` (no acknowledgement clears it,
  with the scope absent or empty),
  `a_gone_launcher_lets_the_recovery_read_the_scope_as_final` (absent and empty;
  another start time under the id is unknown and needs
  the acknowledgement),
  `a_gone_launcher_does_not_clear_a_scope_with_a_member` (round 12:
  no acknowledgement overrides it),
  `a_launcher_record_reads_back_and_a_line_with_a_time_namespace_is_not_understood`,
  `a_start_time_that_differs_from_the_record_is_unknown_and_an_equal_one_is_alive`,
  `a_recovery_in_another_time_namespace_never_reads_a_live_launcher_as_gone` (round 14,
  tightened in round 15:
  a helper in a new user
  and time namespace with a two-hour boot-time offset,
  the same process id,
  mount and cgroup namespaces,
  the launcher held alive;
  the
  shifted reader's own start time must be read successfully as a number and differ from the recorded one;
  the question answers
  unknown and the recovery of a planted journal refuses with a diagnostic that names the start time;
  the same question in the
  writer's own time namespace is alive,
  the control) and
  `a_recycled_time_namespace_number_never_reads_a_live_launcher_as_gone` (round 15,
  the review's schedule:
  a writer in time namespace
  T0 with its launcher in T1,
  the writer's processes killed so that T0 has none,
  a helper that joins the launcher's user namespace
  and creates T2,
  which receives the freed number;
  asserted:
  the launcher is alive,
  the number is reused,
  the recovery's own start
  time differs;
  the question is unknown and the recovery refuses for the start times),
  `what_the_launcher_record_cannot_prove_waits_for_an_acknowledgement` (another namespace,
  no line,
  a line for another scope,
  a line
  not understood),
  `a_torn_launcher_line_never_reads_as_a_gone_launcher` (every proper prefix of the bytes production appends,
  including the line
  without its line feed;
  the whole bytes are the control),
  `a_killed_wrapper_recovers_by_itself_once_its_launcher_is_gone` (a real wrapper killed,
  once its launcher record is whole,
  while its
  program waits at a release gate;
  the record's process id,
  start time,
  namespace and scope are compared with the program;
  blocked
  while it lives;
  settled,
  and the journal removed,
  without an acknowledgement after the launcher is shown gone and the scope was
  collected),
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`.
- **A2,
  R4 (7a F3):
  shown.**
  Round 16 (certification group 2):
  `a_failed_startup_completion_with_a_writer_running_is_not_quiescent_when_the_scope_cannot_be_emptied`
  (the child and a writer ready,
  then the startup completion fails;
  red with the post-start downgrade planted);
  in
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent` each fault has a control without it,
  is consumed,
  and the writer's ready marker is asserted;
  the shim fixture reports arming apart from consumption.
  The joined runner-to-install test,
  the fault classes and the unstoppable member,
  and,
  round 13,
  `a_run_that_cannot_show_quiescence_stops_the_shim_before_the_users_command` (the shim with a user-command sentinel behind the
  refusal;
  round 14:
  the helper's journal must record a scope,
  the weaker-path notice must be absent,
  the armed scope fault must have
  been reached,
  and the refusal must be the not-quiescent one;
  with the fault never armed the test fails).
  `a_failure_after_the_start_to_stop_inspect_or_empty_the_scope_is_not_quiescent`,
  `a_real_run_that_cannot_show_quiescence_stops_the_install_with_the_lockfile_kept_and_nothing_after_it` (a fake cargo whose update goes
  through the bounded runner with an injected fault:
  refusal,
  lockfile kept,
  journal kept,
  no call after the update),
  `a_run_whose_descendants_could_not_be_stopped_stops_the_command_and_keeps_the_journal`.
- **A3,
  A4:
  shown.**
  Round 16 (certification group 3):
  `an_operational_error_with_each_failed_completion_stage_stops_the_command_and_keeps_both_diagnostics`
  (the injected completion failure,
  the real rename,
  the real parent sync),
  `an_operational_error_joined_with_a_failed_mandatory_phase_becomes_the_mandatory_failure_kind`
  (red with the join planted away).
  `an_accepted_run_whose_outputs_cannot_be_forced_to_disk_stops_the_command`,
  `a_refusal_survives_a_journal_that_cannot_be_finished`,
  `a_refusal_survives_a_failed_completion_marker_and_names_it`,
  `an_operational_error_with_a_failed_durability_barrier_stops_the_command_and_keeps_both_diagnostics`,
  `an_accepted_run_whose_completion_rename_fails_stops_the_command`,
  `an_accepted_run_whose_completion_directory_sync_fails_stops_the_command`,
  `a_failing_completion_rename_is_mandatory_and_leaves_the_journal_active`,
  `a_failing_completion_directory_sync_is_mandatory_and_a_later_recovery_only_removes_the_journal` (exactly one `.done` journal
  before the recovery),
  `finish_makes_the_outputs_durable_before_the_done_marker`,
  `the_cleanup_failure_happens_after_the_completion_marker_was_published`,
  `a_removal_failure_after_the_marker_is_cleanup_only`.
- **R1:
  shown by tests of earlier rounds.**
  Round 16 (certification group 3):
  `a_replacement_edited_in_place_after_the_exchange_is_kept_not_deleted` reads the kept file and the
  captured replacement inode.
  `a_lock_another_process_wrote_meanwhile_is_left_alone_with_a_warning`,
  `recovery_leaves_a_lock_that_another_writer_published_while_the_policy_was_judged`,
  `a_replacement_that_another_writer_holds_open_is_kept_when_the_exchange_is_undone`,
  `a_replacement_edited_in_place_after_the_exchange_is_kept_not_deleted`.
- **R2:
  shown.**
  Round 16 (certification group 3):
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name` asserts that each
  queued error fired and inventories both inodes (red with the replacement removed on the fallback).
  `a_link_refused_in_the_kept_directory_falls_back_beside_the_target_and_keeps_a_name`,
  `when_every_link_or_a_barrier_fails_the_displaced_inode_keeps_its_temporary_name_and_the_message_says_where` (the message must
  name a path under which the inode survives;
  round 10,
  9b 3.1:
  the replacement inode is captured at the exchange,
  differs from the
  displaced one,
  and keeps its name at the live path after every fault),
  `a_failing_exchange_or_a_read_only_link_keeps_both_inodes_named`,
  `a_displaced_file_that_could_not_be_linked_is_reported_with_the_snapshot_live`,
  `a_filesystem_without_exchange_fails_the_restore_and_overwrites_nothing`.
- **R3,
  R5:
  shown.**
  Round 16 (certification group 3):
  `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions` asserts the cargo
  write,
  the consumed restore fault and the rollback-specific refusal.
  `a_refusal_whose_rollback_fails_stops_the_command_without_depending_on_permissions`,
  `a_cargo_failure_followed_by_a_failed_rollback_stops_the_command_instead_of_falling_back` (asserts that cargo wrote the file,
  that
  the restore fault was reached and that the refusal is the rollback failure),
  `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others`.
- **K1:
  shown.**
  Round 16 (certification group 3):
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning` checks the
  combined size (4008 bytes) and the folder,
  repeats the transaction and inventories every entry (red with the beside-target bytes
  dropped).
  `the_kept_files_warning_point_is_exact_and_repeated_runs_agree`,
  `kept_names_beside_the_lockfile_and_stranded_temporaries_count_toward_the_warning`,
  `an_unreadable_crate_root_gives_an_unknown_inventory_warning_and_never_refuses`,
  `a_kept_directory_past_the_count_warns_with_count_size_and_folder_and_never_refuses`,
  `a_second_refused_lock_is_saved_under_its_own_name_and_the_first_copy_survives`.
- **K2:
  shown.**
  Round 16 (certification group 3):
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction` logs links and barriers
  together and requires a successful barrier of the kept directory after the preservation link (red with it skipped).
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction`,
  `a_directory_left_unsynced_by_a_failed_attempt_is_not_taken_as_durable`,
  `a_failing_directory_barrier_is_not_reported_as_preserved`,
  `failing_each_directory_barrier_of_a_preservation_in_turn_never_loses_the_name_or_reports_the_unsynced_chain` (round 10,
  9b 3.2:
  a
  reported preservation needs a `.displaced` name beside the target,
  published by a logged link,
  and a successful barrier of that
  directory after that link in one ordered log;
  both inodes are inventoried after every failure,
  and at least one failed barrier
  must end in a reported preservation so the branch runs),
  `a_higher_ancestor_the_user_cannot_open_is_never_touched` (skips with a printed reason for a user who opens a directory without
  the read permission),
  `a_kept_directory_behind_a_symbolic_link_is_synced_by_its_resolved_path`.
- **V1 (7b P1-2):
  shown.**
  `a_journal_listing_that_fails_in_any_way_stops_the_recovery`,
  `a_journal_listing_that_fails_in_any_way_stops_a_new_transaction`,
  `an_absent_journal_directory_is_no_journals`.
- **V2,
  amended c:
  shown.**
  Round 16 (certification group 1):
  `a_journal_whose_only_record_is_malformed_is_kept_with_its_diagnostic_until_it_is_acknowledged`,
  `a_member_in_the_scope_of_one_journal_stops_the_recovery_of_another_until_it_ends`,
  `a_new_transaction_does_not_start_beside_a_scope_with_a_member_until_it_ends` (each red with its check planted away,
  each with a
  control);
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery` now establishes the launcher gone and names the scope's
  member;
  `a_journal_seen_through_another_view_is_refused_and_kept` and the bind-substitution helper of
  `a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` invoke the real recovery.
  `a_survivor_of_one_journal_stops_the_recovery_of_another`,
  `a_new_transaction_does_not_start_beside_a_survivor`,
  `a_killed_wrapper_with_an_escaped_writer_stops_the_recovery`,
  `a_group_record_that_cannot_be_read_defers_the_recovery`,
  `a_malformed_group_record_defers_the_recovery`,
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor`,
  `a_journal_seen_through_another_view_is_refused_and_kept`,
  `a_recorded_scope_whose_parent_is_seen_differently_is_refused_instead_of_read_as_gone`,
  `probe_4_a_scope_is_found_by_its_record_and_a_different_view_is_unknown`,
  `a_recovery_in_another_mount_namespace_refuses_and_keeps_the_journal` (the helper gives the journal the view its own namespace has,
  so only the cgroup parent view differs;
  the refusal must be about that view and not about the journal),
  and the boot identity:
  `a_reboot_of_the_same_machine_discharges_the_liveness_of_a_weaker_path_journal`,
  `another_machine_an_unreadable_identity_no_record_or_the_same_boot_keeps_the_refusal`,
  `the_boot_discharge_keeps_every_other_check_and_a_scope_with_a_member` (a missing snapshot,
  an incomplete listing and a scope with
  a member) and `a_reboot_still_publishes_the_completion_marker_of_the_recovery_or_keeps_the_journal` (round 10,
  9b section 2:
  the
  failed completion rename keeps the journal after a reboot discharge,
  and the control completes it:
  the journal is removed),
  and round 10:
  `a_malformed_recorded_identity_is_not_a_reboot`,
  `a_malformed_current_identity_is_not_a_reboot`,
  `the_current_identity_is_read_strictly_from_its_files`,
  `a_boot_id_put_over_the_kernels_file_is_not_an_identity`,
  `a_bind_mount_over_the_directory_of_a_recorded_scope_is_refused_in_the_same_cgroup_namespace` (9b 2.4:
  with the cgroup namespace
  and every recorded view equal,
  a bind mount of another cgroup directory over the directory that held the scope is refused for the
  parent view;
  the control before the bind clears).
- **V3,
  V4:
  shown.**
  Round 16 (certification group 2):
  `the_record_is_written_made_durable_and_only_then_the_program_is_launched` (an ordered trace of
  write,
  a sync that sees the whole record,
  the sync,
  the launch;
  red with a sync before the append planted) and
  `a_record_that_cannot_be_written_or_made_durable_starts_no_program` (the trace names the containment kind attempted at the fault,
  also when the write fails).
  The record is visible,
  written and synced before the program starts,
  and the faults of the write and of the sync are each reached
  and surface as the refusal (round 10,
  9b 3.6);
  and,
  round 13,
  `a_wrapper_killed_after_each_step_of_the_preparation_is_recovered_conservatively`
  kills a real wrapper after each step of the preparation of the journal (the hand-built states stay as
  `a_journal_cut_off_during_its_preparation_is_removed_without_touching_the_lockfile`).
  `the_scope_record_is_visible_before_the_program_starts` (visibility at the boundary),
  `a_record_that_cannot_be_written_or_made_durable_starts_no_program` (a failed write and a failed sync,
  both containments:
  the
  sync precedes the start),
  `a_record_that_cannot_be_published_starts_no_program_and_does_not_wait`,
  `a_child_whose_group_cannot_be_recorded_is_not_started`,
  `a_wrapper_killed_inside_its_start_is_recovered_conservatively` (the unsettled record with an absent scope refuses and says that the
  run did not record its end),
  `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended` (the window after the launcher spawn),
  `a_journal_cut_off_during_its_preparation_is_removed_without_touching_the_lockfile`,
  `a_recovery_removes_an_empty_journal_directory`,
  `a_nonempty_journal_without_its_record_is_not_treated_as_empty`.
- **V5:
  shown.**
  Round 16 (certification group 3):
  `an_active_recovery_finishes_with_its_own_mandatory_phases_and_tells_cleanup_apart` (output barrier,
  completion rename,
  marker sync,
  cleanup;
  red with the finish replaced by a direct removal);
  the unready and ready tests of
  `journal_recover_tests.rs` are `an_unready_journal_whose_manifest_differs_from_its_snapshot_is_kept`,
  `an_unready_journal_that_lost_its_index_but_holds_snapshots_is_kept`,
  `an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable`,
  `a_ready_record_that_omits_a_manifest_snapshot_is_rejected`,
  `a_ready_record_with_a_duplicate_entry_is_rejected`,
  `a_ready_record_with_an_unexpected_entry_is_rejected`.
  `two_roots_with_manifests_are_restored_or_kept_together_per_root` (round 10,
  9b 3.3:
  the new refused copy is read and holds the
  refused output),
  `a_second_recovery_pass_changes_nothing_and_keeps_the_refused_copy`,
  `a_killed_breaking_run_whose_lock_is_refused_gets_its_manifest_back_too`,
  `a_journal_marked_done_is_removed_by_the_next_recovery`,
  the unready and ready tests of `journal_recover_tests.rs`.
- **V6:
  shown.**
  Round 16 (certification group 1):
  `a_scope_startup_that_stalls_before_the_payload_ends_at_the_bound`,
  `an_unsuccessful_shutdown_returns_not_quiescent_at_the_quiescence_bound` (a surviving member runs the real stop loop to its
  deadline;
  red with the deadline stretched),
  the lock contenders answer within a window while the holder is held
  (`recovery_defers_while_a_live_transaction_holds_the_workspace`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused`;
  red with a wait planted into the lock),
  the process oracle of `bounded_run_tests.rs` fails closed through the pinned procfs
  (`process_is_running`),
  the descendants tests record a delegated scope,
  and `a_child_that_outlives_its_bound_is_stopped_and_named`
  asserts `RunError::TimedOut` and the payload's start.
  `a_manager_that_does_not_answer_ends_the_discovery_at_its_deadline`,
  `a_discovery_whose_descendant_holds_the_pipes_ends_at_its_deadline`
  (round 12,
  11b 2.7:
  an observer thread establishes,
  before the deadline,
  that the descendant is ready and alive when the direct
  program has exited),
  `a_discovery_that_cannot_reap_its_child_returns_at_the_deadline_and_says_so` (9a D3;
  round 12,
  11b 2.1:
  a bound of 1.5 s,
  so one
  deadline returns within 0.7 s of it and a second budget does not),
  `a_wait_that_fails_is_not_a_reap` (11a),
  `a_busy_unsynced_registry_returns_at_once_and_answers_conservatively`,
  `the_route_cache_does_not_make_a_caller_wait_for_a_discovery_in_flight`,
  `a_direct_child_that_cannot_be_reaped_makes_the_run_not_quiescent`,
  `a_child_that_outlives_its_bound_is_stopped_and_named`,
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete`,
  `a_timeout_returns_promptly_even_when_the_child_ignores_its_input`,
  `recovery_defers_while_a_live_transaction_holds_the_workspace`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused`.
- **Weaker path:
  shown.**
  Round 16 (certification group 3):
  `a_directory_without_a_kill_file_takes_the_weaker_path_and_a_refused_scope_runs_nothing`,
  `recovery_with_one_of_two_weaker_path_journals_acknowledged_restores_nothing_until_both_are`,
  `acknowledging_while_a_transaction_holds_the_workspace_is_refused` (the contended lock,
  with a control),
  `the_weaker_path_still_stops_the_process_group_at_the_bound` (a running-writer handshake,
  a `TimedOut` result,
  the weaker path's
  record).
  `each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start`,
  `a_weaker_path_journal_is_refused_by_recovery_and_admission_before_it_is_acknowledged`,
  `an_acknowledged_weaker_path_journal_is_settled_like_any_other` (acknowledge,
  recover,
  settle),
  `acknowledging_one_of_two_journals_leaves_the_other_blocking_everything`,
  `a_partial_acknowledgement_does_not_count`,
  `an_acknowledgement_with_three_lines_that_are_not_a_process_id_and_a_time_does_not_count`,
  `an_acknowledgement_the_reader_cannot_make_durable_does_not_count_until_it_can` (round 10,
  9b 3.6:
  the file sync and the directory
  sync of the reader are failed on their own,
  each fault reached),
  `an_interrupted_publication_leaves_the_journal_unacknowledged_until_it_is_made_again` (a complete temporary beside no published
  name;
  built by hand from a real acknowledgement),
  `an_acknowledgement_whose_barrier_or_publication_fails_is_not_reported_as_made`,
  `an_acknowledgement_does_not_override_a_live_group_or_unrelated_validation`,
  `acknowledge_refuses_what_is_not_an_open_journal_of_a_workspace` (also a finished journal),
  `the_acknowledge_subcommand_acknowledges_the_journal_the_refusal_named`,
  `the_weaker_path_still_stops_the_process_group_at_the_bound`.

### Round 17 additions (second certification pass, 2026-11-04T14:36:06-05:00)

The second certification pass (`cargo-shadow-cert2-sol-1.txt` to `-4.txt`) judged every group not certified,
with no production
defect,
only test-discrimination and bundle defects.
Each item of `Round 17 additions` names the test that now carries it
and says whether a planted regression turned an earlier assertion red;
where no
cheap plant exists the entry says so.
No row is certified by this map.

- **A1,
  V3 (spawn isolation).**
  `two_launchers_held_at_the_launch_boundary_and_an_unrelated_spawn_share_the_wrappers_group` and
  `two_starts_and_an_unrelated_spawn_overlap_inside_startup_without_sharing_a_boundary` compare each payload's control group path
  with the scope its own run launched (the `launch scope <name>` event of that run's trace),
  with the `0::` prefix removed from every
  path,
  and against the wrapper's own group;
  `group_of` stops the test on a read error.
  No cheap plant:
  the mixed-format comparison of the earlier tests could not go red,
  and a production plant that runs one payload in
  the wrapper's group is the moved-wrapper control of round 16 (`a_wrapper_moved_before_the_spawn_puts_the_launcher_in_the_wrong_group_which_the_boundary_test_sees`).
- **R2.**
  `a_failing_exchange_or_a_read_only_link_keeps_both_inodes_named` takes the replacement inode at the exchange (no cheap plant:
  a
  cleanup that swaps in a fresh inode of the same text needs a new production path),
  and
  `a_conflict_whose_undo_exchange_fails_keeps_the_other_writers_inode_named_and_reports_where` fails the second (undo) exchange after
  another writer changed the file;
  red when the stranded inode is unlinked (planted).
- **V3,
  V4.**
  `a_nonempty_journal_that_lost_its_records_is_kept_by_recovery_and_refuses_admission` (`round_seventeen_tests.rs`) runs the real
  recovery and the real admission on a nonempty journal without its `groups` record;
  red with a planted shortcut in recovery.
- **V5.**
  `recover_report` returns whether a mandatory phase of finishing failed (the output barrier,
  the completion rename,
  the sync of the
  marker's directory);
  `an_active_recovery_finishes_with_its_own_mandatory_phases_and_tells_cleanup_apart` asserts the flag for each
  fault and `a_recovery_that_cannot_finish_its_journal_stops_the_command_but_a_failed_cleanup_does_not` runs the install step with the
  cargo runner as the continuation sentinel (red with the flag never set,
  planted).
  The three malformed ready-record tests in `journal_recover_tests.rs` now run the real recovery on a journal whose live lockfile
  changed,
  with `a_journal_with_a_valid_ready_record_is_restored_by_the_same_recovery` as the control (red with the rejection
  ignored,
  planted),
  and `an_unready_journal_is_kept_when_the_live_files_cannot_be_made_durable` requires the fault to be consumed
  and the same state to complete once the fault is gone (no plant:
  removing the fault branch fails the consumption assertion).
- **V6.**
  `a_descendant_in_another_session_that_holds_the_pipes_is_stopped_and_the_output_is_complete` requires the answer within the bound
  while the descendant lives for a multiple of it,
  with a readiness handshake;
  red with the output drained before the scope is
  stopped (planted,
  40 s).
  The helpers the V6 tests call (`answers_within`,
  `assert_reaped`,
  `is_running`) are in the certification packets whole.
  Qualifications the reviewers asked for:
  `probe_2_a_scope_that_is_collected_at_once_is_accepted_as_empty_by_verified_absence` shows
  the acceptance of an already collected scope by verified absence,
  not collection speed;
  the time-namespace tests call
  `survivor_note`,
  so they show the refusal and its diagnostic of the recovery's liveness gate,
  not a complete recovery;
  a delegated
  test returns early when `delegated_route` finds no delegation and `CARGO_SHADOW_ALLOW_NO_CGROUP` is set,
  so a count of passed tests
  shows execution only with the log of a host that has a delegated route (the logs of this host are named in the report).
- **Weaker path.**
  `each_missing_capability_class_gives_the_notice_and_the_weaker_path_records_before_the_start` reads the notice from the trace of
  the run itself (`notice <text>`),
  one per run and per class (red when the run gives none,
  planted);
  `an_acknowledgement_whose_file_cannot_be_synced_is_not_published` is the writer-side file-sync fault of this row;
  The process and thread oracles of the tests fail closed:
  `is_running` and `thread_is_alive` stop on any error,
  and
  `thread_is_alive` first requires a procfs that matches the namespace.
- **V2,
  amended c.**
  The tests of the unsettled scope and the launcher lifecycle belong to this row:
  `an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended`,
  `a_scope_that_appears_between_the_query_and_the_judgement_is_not_read_as_a_run_that_ended`,
  `a_live_launcher_blocks_whether_its_scope_is_absent_or_empty`,
  `a_gone_launcher_lets_the_recovery_read_the_scope_as_final` and
  `a_launcher_that_has_not_made_its_scope_yet_is_not_read_as_a_run_that_ended`.
- **A3,
  A4.**
  `a_refusal_survives_a_journal_that_cannot_be_finished` requires the fault to be consumed and the refusal to carry both the policy
  diagnostic and the failed output barrier (red when the finish error is dropped from the refusal,
  planted);
  `finishing_is_observed_as_outputs_then_marker_then_marker_barrier_then_removal` reads one ordered trace of the output barriers,
  the
  marker rename,
  the successful barrier of the marker's directory and the removal attempt for the success and for each fault (red when
  the barrier call is removed,
  planted).
- **R5.**
  `a_failed_root_does_not_hide_the_warnings_or_stop_the_restore_of_the_others` has a root after the failed one that needs its own
  warning (red when warnings after a failure are dropped,
  planted).
- **K1.**
  `the_command_warns_on_every_completed_run_past_the_count_and_deletes_no_kept_file` runs the install step three times past the count
  with the central folder and the lockfile's directory populated,
  reads the emitted warning (count,
  size,
  folder) at its emission,
  and
  inventories the kept files after each run (red when the command does not print the notice,
  and red when finishing deletes the
  central folder,
  both planted).
- **K2.**
  `the_walk_stops_at_the_first_directory_that_existed_before_the_transaction` requires,
  for each new child directory,
  a barrier of its
  parent that already lists the child,
  before the link of the kept file,
  and the climb barriers after it (red when the creation-time
  parent barrier is removed,
  and red when the climb is removed,
  both planted);
  `failing_each_directory_barrier_of_a_preservation_in_turn_never_loses_the_name_or_reports_the_unsynced_chain` has a lower bound that
  does not come from the implementation.
- **A2,
  R4,
  V1,
  R1.**
  No change;
  the pass found them shown.
- **0.1.0 behind the toolchain launcher.**
  The PATH search skips a candidate that is a launcher whose `.launch` record names the shadow,
  directly or through further
  launchers,
  so the shadow starts once
  (`started_through_a_launcher_the_shadow_runs_once_and_never_takes_the_launcher_for_the_real_cargo`,
  red with the skip removed,
  planted;
  `a_launcher_whose_record_names_the_shadow_is_skipped_directly_and_through_another_launcher`,
  `a_launcher_of_another_program_a_broken_record_and_a_record_loop`).

### Round 18 additions (third certification pass, 2026-11-04T15:17:00-05:00)

The third certification pass (`cargo-shadow-cert3-sol-1.txt` to `-7.txt`) found three production defects and evidence gaps.
Each item names the test that carries it and says whether a planted regression turned it red;
"no plant" says that no cheap
regression of the production code could be built.
No row is certified by this map.

- **Launcher.**
  Production defect:
  the PATH search compared canonical pathnames,
  so a hard link to the shadow was taken for the real cargo.
  The search now compares file identity (device and inode) for direct candidates and for the targets of launcher records
  (`a_hard_link_to_the_shadow_is_skipped_by_file_identity`,
  `a_launcher_record_that_names_a_hard_linked_alias_of_the_shadow_is_skipped`;
  both red with the identity set emptied,
  planted).
  The bounded fallback and the loop guard of an unrecorded launcher that leads back are gone (owner decision of 2026-11-04,
  option A):
  the second cargo-shadow stops at once at the arrival stop with exit status 2,
  and no test named
  `a_launcher_without_a_record_is_ended_by_the_loop_guard_within_a_bounded_number_of_runs` exists any more.
  The current evidence is
  `started_through_a_launcher_the_shadow_runs_once_and_never_takes_the_launcher_for_the_real_cargo`,
  `an_unrecorded_launcher_that_leads_back_stops_the_second_start_at_once_and_its_message_fixes_the_problem_when_copied`
  (whose append-only launcher entry trace is depth-aware:
  every launcher invocation is at the first shadow's depth,
  so no second shadow re-enters one) and
  `an_unrecorded_launcher_that_leads_back_stops_the_second_start_of_add_with_status_2`.
- **V5,
  A3.**
  Production defect:
  the `.done` journals of the recovery were finished through `mark_done_and_remove`,
  which erased the phase of a
  failed completion barrier,
  so no mandatory failure was reported.
  The loop now calls `finish_journal` and sets the flag for a
  non-cleanup error (`a_done_journal_whose_completion_barrier_fails_stops_the_command_but_a_failed_removal_does_not`,
  red with the
  flag not set,
  planted;
  the removal control stays a warning).
  `a_root_whose_lockfile_restore_fails_keeps_its_manifest_while_the_other_root_is_restored_with_its_own` shows the pair of a failed
  root is not split while another root of the journal is restored with its manifest (red with both guards of `recover_manifests`
  removed,
  planted).
- **K1.**
  Production defect:
  the total size of the kept files was an unchecked `u64` sum;
  it is a `u128` sum now
  (`kept_entries_with_huge_sparse_lengths_are_summed_exactly_and_never_refuse`,
  three entries of `i64::MAX` bytes,
  red with the
  old sum:
  it panics with an overflow,
  planted;
  the test stops with a message on a file system that refuses such a length,
  and
  this host's btrfs allows it).
  `the_shadow_prints_the_kept_files_warning_on_stderr_every_completed_run_and_keeps_every_kept_file_itself` reads the real standard
  error of the command on three completed runs and compares every kept file by content and inode (red with the printing removed,
  and red with the central files deleted and recreated empty under their names,
  both planted);
  directory errors fail it.
- **R1.**
  `a_rollback_whose_exchange_finds_another_writers_change_keeps_both_files_and_warns` drives the conflict at the exchange boundary
  through the rollback and asserts the warning,
  both writers' bytes,
  the other writer's inode and the kept replacement (red with the
  warning removed,
  planted).
  The replacement-hook implementation is in the bundle of this row.
- **A1.**
  The thread oracle's prerequisite is now a procfs that lists the process in exactly one namespace (`NSpid`),
  not equal numbers:
  `a_procfs_whose_nspid_lists_two_namespaces_is_refused_even_when_the_numbers_coincide` (red with the count check weakened,
  planted);
  the same check is in `open_proc_root`,
  so the launcher question and the process oracle use it too.
- **V6.**
  `the_route_cache_answers_at_once_while_its_own_mutex_is_held` holds the cache's actual mutex (red with `lock` in place of
  `try_lock`,
  planted);
  `a_timeout_returns_promptly_even_when_the_child_ignores_its_input` shows the program started (a pid file)
  and the feeder was blocked by a larger-than-pipe input;
  the fault consumers (`ControlGroup::stop`,
  `reap`,
  `reap_within`) are in
  the bundle of this row.
- **A1,
  A2 (scope lifecycle).**
  `a_settled_run_whose_scope_was_collected_is_read_as_ended_without_any_launcher_record` removes the launcher proof and recovers
  without an acknowledgement (red with `settled` ignored,
  planted).
  The procfs and ended-number launcher tests belong to this row:
  `a_procfs_of_another_pid_namespace_never_reads_as_a_gone_launcher`
  and `only_the_ended_process_number_shows_a_gone_process`.
- **V1 (7b P1-2).**
  The listing faults leave an ordered trace event ("listing fault ... at entry 1 after 1 accumulated") that
  `a_journal_listing_that_fails_in_any_way_stops_the_recovery` asserts,
  so the fault is at the second entry after the first journal
  was accumulated (no plant:
  the trace is the evidence of placement);
  the listing implementation is in the bundle.
- **V2,
  amended c.**
  `a_live_launcher_blocks_whether_its_scope_is_absent_or_empty` now drives the real recovery and the real admission after each of the
  four states,
  acknowledged or not (red with acknowledged journals excluded from the liveness check,
  planted);
  `a_group_record_that_cannot_be_read_defers_the_recovery` and
  `a_member_in_the_scope_of_one_journal_stops_the_recovery_of_another_until_it_ends` assert the retained journals at once (red with a
  recovery that deletes the journals while refusing,
  planted);
  `a_scope_that_appears_inside_the_real_recovery_is_not_read_as_a_run_that_ended` drives the appearing-scope boundary through
  `recover` (red with the late-appearing scope ignored,
  planted);
  `a_journal_that_lost_its_group_and_ready_records_is_a_possible_survivor` now also runs the recovery and the admission (no plant for
  this state;
  `a_nonempty_journal_that_lost_its_records_is_kept_by_recovery_and_refuses_admission`,
  whose body is in this row's bundle,
  was red against a planted shortcut).
  Qualifications:
  `boot_authority_fixture` asks the liveness check (`survivor_note`),
  not the whole recovery;
  `a_gone_launcher_lets_the_recovery_read_the_scope_as_final` shows the absent case through the whole recovery and the empty case
  through `survivor_note` only.
- **A3,
  A4.**
  `the_ordinary_finish_makes_the_lockfile_and_the_manifest_durable_before_the_marker` observes `finish()` through the production
  output barrier `make_durable` (red with `finish` wired to a no-op callback,
  planted).
- **Weaker path.**
  `each_missing_capability_class_prints_the_notice_on_the_real_standard_error` runs a fresh process per capability class,
  including
  the manager group without `cgroup.kill`,
  and reads the notice from its standard error (red with the printing removed,
  planted);
  `an_unreadable_acknowledgement_does_not_count_for_the_recovery_or_the_admission` (a marker that is a directory;
  red with a read
  error counted as an acknowledgement,
  planted);
  `the_weaker_path_stops_a_writer_of_the_process_group_at_a_normal_exit` (red with the exit cleanup skipped,
  planted);
  `an_acknowledged_journal_is_still_held_by_a_failed_listing_and_a_failed_mandatory_finish` (no plant:
  both paths are shared code).
- **V3,
  V4.**
  The preparation hook (`begin_step_for_test` and its call sites in `begin`) and the producers of the ordering trace
  (`append_record_with`,
  `start_contained`) are in the bundle of this row.
- **Host qualifications.**
  A delegated test returns early when no delegated route exists and `CARGO_SHADOW_ALLOW_NO_CGROUP` is set;
  `a_higher_ancestor_the_user_cannot_open_is_never_touched`
  and the permission-based failed-restore test skip when the user can bypass permissions.
  The logs of this host show them running.

### Round 19 additions (fourth certification pass and the gate, 2026-11-04T16:12:00-05:00)

The fourth certification pass (`cargo-shadow-cert4-sol-1.txt` to `-11.txt`) certified the group of A2 and R4 after one named fix and
found two production defects and evidence gaps elsewhere.
The release gate on main also failed once for a reason in the tests.
Each item names its test and says whether a planted regression turned it red.
No row is certified by this map.

- **A2,
  R4.**
  `an_update_whose_run_cannot_show_quiescence_stops_without_rollback_or_acceptance_and_keeps_the_journal` joins the separate update
  path:
  cargo wrote a lockfile,
  its run reports that a descendant may remain,
  and the update fails terminally with the produced bytes
  live,
  the journal unfinished and no later cargo call (red with the guard of `update.rs` bypassed,
  planted).
- **Launcher.**
  Production defect:
  the identity of the running executable came from its installed pathname,
  which an installer can replace while
  the shadow runs.
  The identity is now taken from the open image (`/proc/self/exe` opened as a file) and joins the skip set
  (`a_hard_link_of_the_running_image_is_skipped_after_the_installed_path_was_replaced`,
  with the control that the schedule is real;
  red
  with the image identity dropped,
  planted);
  `a_launcher_chain_that_ends_at_a_hard_linked_alias_of_the_shadow_is_skipped` applies the
  identity at every hop.
  Found by the use of the shadow as `cargo` in the gate of another package:
  `a_staged_copy_in_an_ignored_directory_is_left_to_cargo_and_the_tracked_crate_is_not` (a crate in a directory that git ignores has
  no committed lockfile,
  so every locked version counted as newly locked and was refused;
  such a crate is now left to cargo;
  red with
  the ignore question answering no,
  planted).
- **K1.**
  Production defect:
  an inventory that could not be read (an entry that cannot be examined) looked below the threshold and printed
  nothing.
  It now prints that the number kept is unknown
  (`an_inventory_that_could_not_be_read_is_reported_as_unknown_and_never_taken_as_small`,
  red with the old condition,
  planted;
  the test
  stops with a message for a user whom permissions do not stop).
- **A1.**
  The process oracle reads `ESRCH` of the stat file as gone as it reads `ENOENT`
  (`a_stat_read_that_fails_with_enoent_or_esrch_names_an_absent_process`).
  A successful stop is observed going through the
  authoritative emptiness proof (`scope proved empty` or `scope proved absent` in the trace,
  asserted by both zombie-leader tests;
  red
  with a stop that returns after the kill without the population check,
  planted).
- **A3,
  A4,
  V5.**
  The output barrier `make_durable` has failpoints at its file sync and its directory sync
  (`a_failed_output_file_or_directory_sync_of_the_ordinary_finish_is_mandatory_and_publishes_no_marker`),
  and it is observed at the
  system call under `strace` (`the_output_barrier_syncs_the_file_and_then_its_directory_as_the_kernel_sees_it`;
  red with the
  `sync_all` call removed,
  planted,
  where the in-process trace test stayed green:
  the in-process trace cannot show a dropped
  syscall).
  `the_recovery_makes_a_kept_roots_lockfile_and_manifest_durable_through_the_real_barrier` fails the real barriers of the
  recovery on a kept root and observes both outputs made durable before completion (red with the loops removed,
  planted),
  and
  `an_unready_journal_reports_its_mandatory_and_cleanup_failures_through_the_report` carries the unready branch's flags (red with the
  flags dropped,
  planted).
  The failed-root test decides its permission prerequisite while the directory is read-only.
- **V3,
  V4.**
  `a_record_that_is_written_only_in_part_starts_no_program` (a real short write;
  red with the length check dropped,
  planted) and
  `the_record_sync_is_given_to_the_kernel_after_the_write` (under `strace`;
  red with the `fsync` removed,
  planted).
  The kept partial journals of the preparation cuts are checked as the same journals with their snapshot and index.
- **V2,
  amended c,
  and the weaker path.**
  A populated scope whose launcher is gone,
  and an acknowledged live process group,
  now block the real recovery and admission
  (`a_gone_launcher_does_not_clear_a_scope_with_a_member` with a control after the member stops,
  `an_acknowledgement_does_not_override_a_live_group_or_unrelated_validation`;
  each red with acknowledged journals excluded from the
  liveness check,
  planted);
  `an_unsettled_scope_that_exists_empty_is_not_read_as_a_run_that_ended` keeps the journal and refuses
  admission (red with the scope-appears shortcut,
  planted);
  a repeated boot field is refused in both orders (red with last-wins,
  planted);
  the weaker-path cleanup tests judge the writer at return;
  the refused-scope test requires its stand-in to have run.
  The reaping fault tests assert consumption and have a fault-free control.
- **The gate on main (certification of the suite,
  2026-11-04).**
  Cause:
  the suite's journal protocol makes durable writes (an `fsync` of a file or a directory per step;
  a full run does hundreds),
  and
  the disk under the build directory is a SATA SSD under the BFQ scheduler that other sessions' work,
  an image build among them,
  keeps busy;
  the I/O pressure of the host (`/proc/pressure/io`) was 26 to 40 percent over the last ten seconds,
  and a test that takes
  0.24 s on a quiet disk took 11 s.
  Two kinds of test turned that into failures:
  waits that followed the clock (a helper that is ready
  in a tenth of a second was waited for thirty seconds) and bounds shorter than the start of a scope (a run bounded at 400 ms ended
  while its scope was still being made,
  before the payload existed).
  Waits now follow the helper (a child of the test that is alive;
  a missing file with no helper left fails at once),
  and a bound that ended in startup is retried with a doubled bound until the
  payload really started (`a_scope_that_starts_slower_than_the_first_bound_is_retried_with_a_longer_bound_and_then_judged`,
  red with a
  single attempt).
  The gate passed under that pressure.
- **Not done in this round** (named by the fourth pass;
  the map claims them neither shown nor planted):
  R2 fallback-link fault and
  reported stranded path;
  K2 retry after a failed parent-entry barrier;
  the listing faults routed through the real operation results
  (V1);
  the mandatory-finish refusal through a real shim run (A3);
  the acknowledgement writer's successful ordering;
  the reboot-related
  refusals through the recovery and admission entry points;
  the blocked-feeder handshake of the input-ignoring timeout test.

### Round 20 additions (fifth certification pass, 2026-11-04T16:43:15-05:00)

The fifth pass (`cargo-shadow-cert5-sol-1.txt` to `-15.txt`) judged all groups not certified.
Some groups ran without the manifest because
the packet and the manifest together passed the input cap,
and some lacked bodies the manifest assigned;
the packets are rebuilt below
the cap with the manifest attached and with production items listed apart from tests.
Substantive findings and their state:

- **Launcher.**
  `a_running_image_whose_installed_path_was_replaced_still_skips_its_hard_links_in_the_production_lookup` runs a real process from an
  installed path,
  replaces that path with a rename while a hard link stays on PATH,
  and asks the production lookup (no identity is
  injected);
  it covers a direct alias and a recorded chain that ends at the alias (red with the image identity dropped,
  planted).
  Production race:
  the chosen path was a mutable pathname between the choice and the start.
  `refuse_if_ourselves` repeats the check
  right before each of the three starts (`a_chosen_path_that_names_the_running_image_at_the_start_is_refused`).
  That shrinks the
  window to the instants between the check and the start;
  it does not close it.
  Closing it means executing an opened descriptor,
  which breaks scripts and proxies that open their own path,
  so it is not done and needs an owner decision.
  Superseded in round 21:
  the owner's rule ("We should never invoke ourselves recursively.")
  settles it;
  see the round 21 block.
  Owner decision of 2026-11-04 (option A):
  a launcher with no record that leads back no longer gets a bounded fallback;
  the second
  cargo-shadow stops at once with exit status 2 and a message a reader with no context can follow
  (`an_unrecorded_launcher_that_leads_back_stops_the_second_start_at_once_and_its_message_fixes_the_problem_when_copied`,
  which runs
  both printed fixes and shows the command then works;
  red with the stop removed,
  planted).
- **Weaker path,
  V6.**
  Production defect:
  the once-per-process notice used `Once::call_once`,
  so a second run waited with no deadline for the first run's
  write to standard error.
  It is a flag won with a swap now (`a_run_does_not_wait_for_another_runs_slow_notice`,
  a fresh process that
  holds the winner inside its emission;
  red with the old `Once`,
  planted).
- **A2,
  R4.**
  The install and shim uncertainty tests require the journal to stay unfinished,
  not only present (no plant).
- **K1.**
  `a_completed_run_warns_exactly_past_the_count_and_not_at_or_below_it` (below,
  at,
  past).
- **Scope lifecycle.**
  The parent view's mount id and inode are changed on their own,
  with an unchanged control,
  through recovery and admission (red with
  only the mount compared,
  planted).
- **V2.**
  A member after a reboot is refused by the real recovery and admission under the same identity (no plant).
- **V6 retries.**
  The retried tests measure the judged attempt and require the timeout itself.
- **Not done in this round.**
  V1 injections still return before the real handling of the operation results;
  R2 sibling-fallback fault and stranded path;
  K2 retry
  after a failed parent-entry barrier;
  admission checks for the other boot-related and unreadable-record schedules;
  the acknowledgement
  writer's successful ordering;
  the mandatory-finish refusal through a real shim run;
  the blocked-feeder handshake;
  the assertion that
  the start-time error line names the namespace;
  the actual spawned child's id for the reap check.

### Round 21 additions (sixth certification pass, 2026-11-04T17:40:00-05:00)

- **Check-to-start race,
  closed for compiled programs.**
  `real_binary::vet` opens the chosen real cargo once (close-on-exec),
  runs the running-image and recorded-launcher checks on the
  open file,
  and recognises a compiled (ELF) program by its first bytes.
  `Vetted::command` then starts a compiled program through
  `/proc/self/fd/<n>` with the chosen path as `argv[0]`,
  so a path made to name the shadow after the check cannot reach the start
  and rustup's proxy still picks the tool by name.
  The two starts that this process makes itself use it (`shim::run_program` and the
  shim exec site).
  The cargo of an install step (`RealCargo::run`) runs inside a scope whose launcher (`systemd-run`) takes the program
  as an argument and starts it from its own process,
  so no descriptor of this process can name it there,
  and the launcher would also
  replace `argv[0]`;
  that start keeps the path with the check repeated right before (`Vetted::command_by_path`),
  and its window ends
  at the arrival stop like the script residual below (`a_compiled_real_cargo_runs_in_the_contained_runner_of_the_install_step`,
  red
  when that start uses the descriptor).
  `a_compiled_real_cargo_swapped_to_the_shadow_after_the_check_still_runs_the_real_one` swaps the checked path to a hard link of the
  running image between the check and the start (the `after_check_for_test` seam) and requires the real program's output;
  red with
  the start by pathname and no second check (the shadow listed its tests instead),
  planted.
- **Script and contained-start residual.**
  A script cannot be started through a close-on-exec descriptor (the interpreter opens the path after the descriptor is gone,
  and
  the start fails with ENOENT),
  so a script keeps its pathname and is checked again immediately before the start
  (`a_script_real_cargo_swapped_to_the_shadow_after_the_check_is_refused_at_the_start`).
  The instants between that second check and
  the start remain;
  a hit there reaches the arrival stop of the second cargo-shadow (exit status 2),
  so nothing runs twice.
  The
  owner has not been asked to accept this residual beyond the
  `Check-to-start race, closed for compiled programs` rule;
  it is stated,
  not hidden.
- **Group 2 and group 4 named fixes.**
  `a_wait_that_fails_is_not_a_reap` bounds the wait and requires the elapsed time to match the bound (red with a one-shot extra
  budget,
  planted).
  The quiescence tests wait for the program's ready marker,
  require the fault to be consumed,
  and require the
  program to have run in the recorded scope (`assert_ran_in_recorded_scope`),
  in both the inspection schedule and the joined
  install test.

- **Review of 2026-11-07 (merge of round 21,
  cross-vendor reviewer `cargo-shadow-r21-sol-1.txt`),
  what changed and what stays limited.**
  The exec site dropped the open file before `exec` (a binding named `_vetted` is a binding,
  not a discard),
  so a compiled real cargo
  failed to start there;
  the vetted program now lives to the `exec`,
  and `a_compiled_real_cargo_is_started_by_the_exec_branch_with_its_arguments`
  and `a_compiled_real_cargo_is_started_by_the_child_branch_with_its_arguments` start a compiled program through both of the shim's own
  starts (red at the exec site before the fix).
  An execute-only program (the search for the real cargo accepts it) is opened by `O_PATH` for the identity checks and started by path with
  the check repeated,
  because a file that cannot be read cannot be told from a script and a script must be given its own path
  (`an_execute_only_compiled_real_cargo_is_started_like_any_other`,
  `an_execute_only_script_real_cargo_is_given_its_own_path_by_the_kernel`,
  red before the fix).
  The first process id Linux never allocates (the value of `pid_max`,
  "one greater than the maximum PID") no longer counts as an
  acknowledging process.
  Limits that stay,
  stated so the `Check-to-start race, closed for compiled programs` claim
  is read as it is:
  the descriptor start pins the file object that was checked,
  not its bytes (an in-place rewrite,
  or an `ETXTBSY` refusal while a
  writer holds the file,
  is not covered);
  it needs a working `/proc/self/fd`:
  a missing directory makes the start by path with the repeated check,
  and a directory that exists and does not work is not detected,
  so the start fails instead;
  the identity checks run on the open file,
  while the launcher-record check reads `.launch` files and their targets by pathname;
  a program that is not a readable ELF file (a script,
  any other format,
  or a file this process cannot read) is started by path;
  the printed Fix B record uses the lossy text of a path,
  so a path with bytes that are not UTF-8 is not reproduced faithfully;
  the script swap test proves the refusal at the repeated check,
  not a swap after it;
  the journal-barrier tests prove failure propagation and trace order,
  not the syscalls themselves (open list,
  group 5).

### What the map does not claim

- The strong-path tests that do not run a transaction rely on the host having a delegated scope (`delegated_route`) and on the
  runner choosing it when it has one;
  `assert_scope_record` proves that the journal holds the record of the scope the runner opened,
  and the program's own group
  proves where the program ran;
  neither alone proves the payload was placed:
  the record is the intent,
  and the
  program's group is the placement (group 1 wording,
  sixth pass).
- The identity tests give the recovery a different identity;
  no machine was rebooted.
- Nothing else in the launcher identity or the boot rule reads a time-namespace-relative value:
  the boot rule uses the boot id and
  the machine id only,
  and the code reads no uptime,
  `btime` or `CLOCK_BOOTTIME`;
  only field 22 of the stat file is relative.
  The time-namespace helpers need `unshare` and `nsenter` with permission for a user and time namespace,
  and a kernel with time
  namespaces (Linux 5.6);
  a host that refuses fails the tests loudly (this host allows them).
  The recycled-number test relies on the namespace number allocator giving the lowest free number to the next namespace,
  and asserts
  that the number was reused;
  on a host where it is not reused the test fails instead of passing without the schedule.
  The scenario with the writer's current time namespace different from its time namespace for children is built by a writer helper
  in T0 that starts its launcher in T1 (not by a writer that calls `unshare` on itself);
  the effect on the record is the same,
  T0 is
  held by nothing once the writer's processes are gone while the launcher lives.
- The 12a schedules that need a deliberate remount while a recovery runs (replacing `/proc` between the pidfd check and the stat
  read,
  overmounting `/proc/<pid>`) are excluded by assumption a as amended in round 13 and have no test;
  the guards that cover them
  (one pinned procfs descriptor,
  `openat2` beneath it without crossing a mount) are exercised by every launcher question,
  not by a
  schedule that tampers.
- The launcher record's premise that a launcher which is gone cannot join a scope is the process model;
  no test makes a manager
  handle a request after its sender died.
  The launcher is a stand-in process (a shell at a gate) in the planted-journal tests and a real `systemd-run` launcher only in the
  end-to-end test.
- The authority check of the boot id was shown in a disposable user and mount namespace on this host;
  no container runtime
  (`systemd-nspawn`,
  Docker,
  Podman,
  gVisor,
  a virtual machine) was run,
  and the nspawn behavior is read from its source
  (`src/nspawn/nspawn.c`,
  `setup_boot_id`,
  on the default branch,
  2026-11-04).
- The startup overlap of the spawn-isolation row,
  the shim-level test of A2/R4 and the cuts of V3/V4 at the real boundaries of `begin`
  are not new evidence of this round.
- A flake of the orphan test during this round (a process oracle that read a process that was being reaped) was explained by a
  state read that fails with `ESRCH` as well as `ENOENT`;
  both oracles now treat both as gone and stop on any other error.
  The explanation was not reproduced afterwards,
  so it is a conclusion from the code,
  not from a repeated failure.
- Power loss,
  a real reboot,
  and checkpoint and restore are outside the tests.

### Findings the work of rounds 8 and 9 made, outside the list

- `systemd-run` reads the command it is given like a unit's command line:
  `$$` became `$` and `${HOME}` was replaced by the value of
  the variable (probed 2026-11-04).
  Every argument and the program are encoded at the point of hand-over,
  and
  `arguments_that_systemd_run_reads_as_a_grammar_reach_the_program_unchanged` runs fifteen awkward arguments,
  also through a cleared
  environment.
- A cleared environment (`env_clear`,
  used for Git) cannot be read back from a `Command`,
  so there is a second entry point,
  `output_within_clean_environment`,
  and the program is started through `env --ignore-environment`.
- A program that does not exist is found before the launcher starts,
  so it stays an I/O error.
- An inactive slice has no control group,
  so the discovery starts the slice first
  (`probe_1_an_inactive_slice_is_started_before_its_control_group_is_asked_for`).
- The journal view uses the unique mount id of Linux 6.8 and later when the kernel gives it.
- Host requirements of the tests:
  a reachable user manager with `systemd-run`,
  a cgroup v2 mount,
  `unshare` and `mount` with
  permission to make a user and mount namespace (three tests:
  the mount view,
  the boot id authority and the bind substitution),
  a
  host whose `/proc/sys/kernel/random/boot_id` is the kernel's own (the identity tests read it and fail loudly where a runtime
  replaced it),
  and a C compiler named `cc` (the tests with the thread-leader fixture);
  `CARGO_SHADOW_ALLOW_NO_CGROUP=1` turns the missing-manager case into a printed skip,
  the others fail loudly.
- Route 2 of the earlier pick (a child of the own group where it carries `user.delegate`) is dropped:
  it needs `clone3` with `CLONE_INTO_CGROUP`,
  which neither `std` nor `rustix` 1.1.6 offers.
