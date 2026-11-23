# The forced arm of a removal: the repaired checkout, the holder's copy, and the record the journal keeps

Non-final,
unapproved design proposal by an agent,
written in the round whose commits are listed under "Revisions".
It is not approved by the project owner and nothing in it is final.
It designs the two open items of the accepted ZFS worktree design that are one mechanism:
"Who ends a holder,
how a holder is identified,
and by what authority",
and "How the wrapper tells its caller's session apart from the other holders".
The record answers the channel,
the seen holder,
the reach,
the protected set,
the shared-namespace direction,
the signal,
the authority's duration and the script unit ([record][record]);
the records record decides that a forced removal's record of who it ended is the handler's journal alone ([records][records]).
The shared-namespace answer is a direction:
this document designs the ways and recommends one,
a guest measures it,
then the project confirms.
Its correction withdraws the sentence that the answer was "the authorisation ...
for the unmount of the clone's own copy and nothing wider";
how far the change reaches is this design's to propose,
with its cost stated.
The signal's wait ships no number until the project decides one or a measurement sizes it.
The two host commands that broke the never-prompt rule in the first round are recorded under L5 and are not evidence.

## The open items and the words that decide them

The first open item,
quoted whole ([proposal][proposal],
"Open items the decisions leave"):

> Who ends a holder,
> how a holder is identified,
> and by what authority.
> The answer's words:
> "Kill or otherwise remove or destroy the holders whenever --force is specified,
> auto-cancel if not."
> A holder can be another account's process,
> a unit's task the caller cannot signal,
> or a namespace whose task is gone,
> and a refusal rule would contradict the answer.
> The design settles none of these parts.
> Identification:
> a namespace ID and a representative PID from a root scan say nothing about PID reuse,
> so nothing yet proves the process ended is the process seen.
> The receiving path:
> how an unprivileged wrapper is handed root's finding,
> when a receipt is read-only and names no holder.
> What such a holder itself sees is the thing the owed VM measurement records.

The second,
quoted whole ([proposal][proposal],
the next paragraph):

> How the wrapper tells its caller's session apart from the other holders.
> The answer's words:
> "Spare the caller (Recommended)":
> the removal never ends the caller and the processes of its session,
> the shell that ran the command and that shell's other jobs,
> and when the caller's session holds the clone the removal refuses and says to leave the folder and run the command again;
> other holders are still ended.
> Which process the wrapper knows as its caller,
> how it identifies that caller's session and the jobs in it,
> how it tells those from another job that happens to hold the clone,
> and what it prints on the refusal are undesigned,
> and nothing measures such a refusal.

The decision's never-end set,
quoted whole ([decision][decision],
lines 49 to 55):

> - The never-end set,
> "Spare the caller (Recommended)".
>   The removal never ends the caller and the processes of its session,
>   the shell that ran the command and that shell's other jobs.
>   When the caller's session holds the clone,
> the removal refuses and says to leave the folder and run the
>   command again.
>   Other holders are still ended,
> as "Kill or otherwise remove or destroy the holders whenever --force is
>   specified" says.

The four answers this design now follows are [the authority and reach record][record],
accepted 2026-11-22.
Its first answer,
"By what channel a holder of another account is ended":

> The project chose "B:
> root handler (Recommended)":
> the wrapper writes a request that the installed root handler reads,
> and root ends the holders,
> through the property and receipt protocol the route already installs.
> No second privileged channel is added.

Its second,
"What a removal without `--force` does when its scan sees a holder":

> The project chose "A:
> cancel,
> delete nothing (Recommended)":
> the removal stops before any snapshot is deleted,
> attaches the clone at its slot again,
> and reports the holder.
> A removal that would have gone through had the holder left a moment later is cancelled and is run again by the person.
> This fixes the reading of the earlier sentence:
> a holder the scan sees stops the destruction.

Its third,
"How much a forced removal ends in another mount namespace":

> The project chose "B:
> unmount,
> end blockers only":
> the handler unmounts the clone inside the holder's namespace and ends only the tasks that block that unmount,
> and when the namespace cannot be entered or the unmount is refused,
> the forced removal refuses and names the holder.
> It never falls back to ending every task of the namespace.
> The recommended option was the same unmount with that fallback;
> the project did not take the fallback.
> A forced removal can therefore still refuse.
> Still owed before this is final:
> the design states how the mount to unmount is named inside the holder's namespace,
> what is checked before and after the call,
> what happens to a peer group and to a child mount,
> and a guest measures it;
> then the project confirms.

Its fourth,
"Which processes a forced removal never ends":

> The project chose "D:
> ancestors,
> refuse as unit (Recommended)":
> a forced removal never ends the caller's session or any ancestor of the command up to the first process,
> and refuses,
> naming it,
> when one of them holds the clone;
> and it refuses `--force` outright when the command's parent is the service manager,
> telling the person to run the removal as a child of the process that wants it.
> The cost the question stated with this option:
> a forced removal launched as a service unit is refused.

The proposal on an unmount inside a holder's namespace,
which the third answer authorises ([proposal][proposal],
lines 730 to 732):

> With a shared checkout,
> descriptor move refused with `EINVAL`;
> the positive control used a path unmount solely to exercise propagation.
> It removed the peer and slave copies while both readers were alive,
> and head destroy succeeded.
> This does not authorize path unmount in the design.

The measurement the adoption still rests on,
quoted whole ([proposal][proposal],
"Forced removal and deferred origins"):

> Extend [VM item 15]'s post-attach readers and [VM item 4]'s busy holder:
> with `--force`,
> force-unmount or end the holders in each namespace kind,
> each of them outside the caller's session,
> and destroy the clone,
> recording what each holder sees (an error,
> a vanished mount,
> stale content,
> or a signal) and whether the destroy then succeeds,
> with the wrapper's caller and its session's other jobs alive through the arm.
### The further answers, and the correction

The record's "A holder in a namespace whose mounts are shared" chose "B:
design an unmount there":

> the design finds a way to unmount the copy inside such a namespace,
> for example by making its parent mount private first,
> so that the service keeps running and the removal finishes.

Its cost,
in the record's words:

> it changes what a running service's namespace receives from the host,
> which the accepted design does not authorise today;
> one more design round and a guest measurement before you can confirm.

Its correction of 2026-11-22T20:34:39-05:00:

> this section first added that the answer "is that authorisation,
> for a forced removal's unmount of the clone's own copy and nothing wider",
> and cited a line of the proposal for it.
> The option said neither,
> its own example changes a mount other than the clone's copy,
> and the cited line is about a path unmount.
> How far the change may reach is for the design round to propose and for the project to confirm after the measurement.

So the reach is this design's proposal,
its cost is stated with it,
and no way is excluded for being wider than an authorisation the record never gave.

Its "The signal":

> The project chose "A:
> ask,
> wait,
> then kill (Recommended)":
> `SIGTERM`,
> a wait for the process to exit,
> then `SIGKILL` if it has not.
> The length of the wait ships as no number until the project decides one or a measurement sizes it.

Its "How long a forced request's authority lasts":

> The project chose "A:
> only while the command lives (Recommended)":
> the handler ends processes only while the very command that asked is still running,
> checked by its process identity before every signal and every unmount.
> Afterwards the standing request may still cause mount work that ends nothing.

Its "One more answer:
a unit whose main process is a script":

> The project chose "B:
> refuse that too (Recommended)":
> a forced removal is refused also when the command's session was started directly by the service manager,
> not only when the command itself was.
> Commands that agents run stay allowed,
> since their sessions are started by the agent.

The fourth further answer,
"A cancellation after part of the history is gone",
belongs to [the sibling][cancelled] and is not repeated here.
The record's "Not chosen among these four" lists the alternatives this design does not carry.
The records record,
"Whether a forced removal records who it ended,
beyond the journal",
chose "the journal alone":
"The handler's journal lines are the record."
Its cost:
the journal may rotate away,
and the removal's own record does not show it.
The options not chosen are listed once under "The record of who a forced removal ended,
and the questions left".

## What this design read

- The decisions and records:
  [the decision][decision] whole,
  [the authority and reach record][record] whole,
  the proposal's "Blocked-removal cancellation",
  "Mounts held by other namespaces",
  "Removal when the namespace exclusion cannot be shown",
  "Requests,
  results and ordering",
  "Removal and recovery",
  "Cancelling a blocked removal",
  "What doctor checks and prints",
  "Decided defaults",
  "Busy unmount after a staging move",
  "Clone removal identity protection",
  and "Forced removal and deferred origins",
  and the plan's step 17 and its open-item list.

- [The accepted staging design][stagingdesign],
  "The unmount call from staging" and "The private bind",
  whose covered-descriptor rule this design reuses where it fits.

- The second-pass gate review,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-second-pass.md`,
  whole.

- `package/git-policy/cli/SPEC.md` at "The wrapper's side and its deadline",
  the never-prompt rule,
  and the receipts the unmount files.

- The code the route already has:
  `clone_holder_scan` and `mount_namespace_scan` on the step 9 branch,
  `worktree_clone_mount_protocol` receipts and intent,
  `worktree_clone_unmount`'s busy answer and descriptor unmount,
  `process/process_use.rs`,
  `process/process_identity.rs`,
  `process/process_tree.rs`,
  `process/member_signalled_through_its_pidfd.rs`,
  `lock/single_owner_claim.rs`,
  `lock/index_lock_holders.rs`,
  and `invocation/started_as_git.rs`'s ancestor walk.

- The staging decision's threat model ([staging][staging],
  lines 221 to 237),
  the no-configuration answer ([sweep][sweep],
  lines 17 to 55),
  and the slot identity store's shape ([store][store]).

- Upstream Linux `v7.3` in `/home/user/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/scratch/linux-v7.3`,
  OpenZFS `zfs-2.4.4`,
  systemd `v262`,
  and sudo `v1.9.17p2`,
  each at a named file and line below.

- The step 9 holder-scan branch and the step 9 unmount branch as work in flight.
  This design designs no scan and no unmount turn.
- The review of this revision,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-after-the-owners-answers.md`,
  whole,
  and its measurement `private-mounts-unit-mountinfo.txt` beside it.

- systemd `v262` at `src/core/namespace.c` and `man/systemd.exec.xml`,
  and Linux `v7.3` at `fs/namespace.c`,
  `fs/pnode.c`,
  `fs/proc/base.c`,
  `fs/proc/fd.c`,
  `fs/proc_namespace.c` and `fs/namei.c`,
  for the propagation and matcher facts below.
- The review after the further answers,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-after-the-further-answers.md`,
  whole,
  and its kernel readings.

- Three sibling designs accepted on `main`:
  [the deferred-destroy checks][deferred],
  its "The look before the unmount";
  [the propagation design][propagationdesign],
  its "The step that re-applies the setting" and "What the clones under the old instance become";
  and [the dirty-main restore design][dirtymain],
  its "The further answer:
  an add whose own unmount is refused as busy".

- `doc/decision/cli-git-zfs-staging-under-cache.md`,
  "The answer a stopped unmount files,
  answered 2026-11-23":
  the fourth outcome word for a put-back,
  the clone's place and the bind's state,
  and the word and fields left to plan step 9's build.

- The corrected record's "Correction" paragraph and its "One more answer:
  a unit whose main process is a script",
  quoted under "The open items and the words that decide them".

- [The sibling cancelled-removal design][cancelled] at its current tip `3f2475e430`,
  read last:
  its "The forced arm's failure path",
  "What the caller sees",
  "The stop points",
  its refusal words,
  and its S8.

- Linux `v7.3` at `fs/pnode.c`,
  `fs/namespace.c`,
  `net/unix/af_unix.c`,
  `fs/proc/base.c` and `fs/namei.c`,
  and `Documentation/filesystems/sharedsubtree.rst`;
  the accepted staging design's "The unmount call from staging" and "The staged-folder record" for the recorded-handle route.
- The fourth review,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review-fourth.md`,
  whole,
  and its "Corrections to my own last review".

- The records record `doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
  its "Whether a forced removal records who it ended,
  beyond the journal" and "How long the plan a cancellation keeps stays".

- Linux `v7.3` at `fs/pnode.c` (`propagation_next`,
  `change_mnt_propagation`),
  `net/unix/diag.c`,
  `fs/fhandle.c` and `fs/namespace.c`;
  the checkout-propagation record and the proposal's "Checkout propagation policy" and VM item 3;
  and this host's `man setsid`.

## What this design ran

No probe this round.
The five host probes of the first round stand,
kept with their outputs in `/home/user/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/keep/`:
`process-relations.ts`,
`pidfd-probe.py`,
`namespace-pin-probe.py`,
`authority-probe.py`,
and the two inline `cgroup.kill` arms.
The keep log records the two attempts that asked the system service manager to act before the rule reached the first round;
neither is evidence (L5).
This round read the corrected record,
the records record,
the fourth review with its corrections to its own last review,
the sibling at its current tip,
and the kernel,
systemd,
staging-design and propagation-record sources named above.
It ran no guest,
no ZFS command,
no `sudo`,
no act through the system service manager,
and mounted nothing in the host's own namespace.
The review's own measurements and readings are used where they bear;
this round did not repeat them.

## Facts

Each fact says how it is known.
A fact measured here is this machine's,
not a guest's.

- F1.
  A command the Pi harness runs is a session leader of its own:
  its `session` field equals its own pid,
  and its parent,
  the Pi process,
  is in another session.
  (Measured,
  probe 1.)

- F2.
  The same command's cgroup is the long-lived Pi unit's cgroup,
  not a per-command unit,
  so the cgroup does not separate one command from another here.
  (Measured,
  probe 1.)

- F3.
  `systemd-run --user --scope` does not give its child a new session;
  the child shares the calling shell's session.
  (Measured,
  probe 2,
  and the review's own run.)

- F4.
  A command Claude Code's Bash tool runs has a session-leading `zsh` as its parent and `claude`,
  in another session with a controlling terminal,
  as the `zsh`'s parent.
  (Measured by the gate review,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/chain-claude-code-bash-shape.txt`.)

- F5.
  A command run as a `systemd-run --user --pipe --wait` service unit is its own session leader and its parent is
  `systemd --user`;
  no process that started the unit is an ancestor.
  (Measured by the gate review,
  `.../keep/chain-systemd-run-service-unit-shape.txt`.)

- F6.
  A child that calls `setsid` leaves its parent's session,
  and its process group equals its own pid.
  (Measured,
  probe 2,
  arm 3.)

- F7.
  `pidfd_open` succeeds on any process visible in the caller's pid namespace,
  including pid 1 of another account:
  the kernel resolves the number and creates the descriptor with no permission test.
  (Measured,
  probe 2;
  Linux `v7.3`,
  `kernel/pid.c:697` to `:716`.)

- F8.
  Signalling through that descriptor to another account's process fails `EPERM`,
  because the signal checks the same credentials `kill(2)` does.
  (Measured by the first authority probe,
  whose arm is no longer in the script;
  the log keeps it;
  Linux `v7.3`,
  `kernel/signal.c:789` to `:799`,
  `:805` to `:830`,
  and `:4140` to `:4191`.)

- F9.
  An own child signalled through its pidfd was killed by signal 9 (the wait status was -9;
  a shell reports 137),
  and after it was reaped `pidfd_open` on its number failed `ESRCH`.
  The reuse case itself (a second process taking the number and the old descriptor reaching it) is not in any probe;
  it is test 2's,
  and the kernel's rule is that `pidfd_send_signal` reaches the process the descriptor names.
  (Measured part,
  probe 2;
  kernel rule read at Linux `v7.3`,
  `kernel/signal.c:4140` to `:4191`.)

- F10.
  A pidfd is also the race-free way to read the pinned task's namespace:
  `PIDFD_GET_MNT_NAMESPACE` returns the mount namespace of the task the descriptor names,
  and `PIDFD_GET_INFO` returns its pid,
  ppid and credentials,
  with a fresh task lookup that fails `ESRCH` once the task is gone.
  (Source,
  Linux `v7.3`,
  `include/uapi/linux/pidfd.h:112` to `:122`,
  `fs/pidfs.c:530` to `:615`,
  the namespace ioctls take the fresh task lookup at `:552` to `:554`,
  and `PIDFD_GET_INFO` takes the shortcut at `:548` to `:550` and its own lookup returns `-ESRCH` at `:417` to `:431` once the task is gone.)

- F11.
  `/proc/<pid>/stat` is readable across accounts and carries the parent,
  process group,
  session and start time;
  pid 1's record read `session=1` and `starttime=14`.
  The field order is `fs/proc/array.c:595` to `:611`.
  (Measured,
  probe 2.)

- F12.
  `/proc/<pid>/ns/mnt` is refused with `EACCES` for another account's process and for a same-account process that
  is not dumpable,
  so a wrapper cannot read another account's namespace link.
  (Measured,
  probe 1;
  Linux `v7.3`,
  `fs/proc/namespaces.c:63` tests `ptrace_may_access`.)

- F13.
  A mount namespace with no task stays alive while a descriptor names it,
  and its mount stays with it:
  a `setns` into it from a process with `CAP_SYS_ADMIN` in its owning user namespace found the tmpfs row.
  No `/proc/<pid>/ns/mnt` path existed,
  and a `/proc` grouping by namespace found no task.
  (Measured,
  probe 3.)

- F14.
  A taskless but active namespace is still visible to a privileged caller:
  `listns` lists every active namespace of a user namespace when the caller is in the init pid namespace with
  `CAP_SYS_ADMIN` in the init user namespace,
  and an active namespace can be reopened through a namespace file handle.
  (Source,
  Linux `v7.3`,
  `kernel/nstree.c:515` to `:523`,
  `:539`,
  `:576` and `:584`;
  `kernel/nscommon.c:313` to `:317`;
  `include/linux/ns/ns_common_types.h:45` to `:70`;
  `fs/nsfs.c:518` to `:640`;
  the syscall number at `arch/x86/entry/syscalls/syscall_64.tbl:397`.
  Unrun.)

- F15.
  `setns(CLONE_NEWNS)` from a thread of a multithreaded process fails `EINVAL`;
  the same call from a single-threaded process succeeded.
  (Measured,
  probe 3;
  Linux `v7.3`,
  `fs/namespace.c:6720` to `:6721` tests that the file-system structure has one user.)

- F16.
  `unshare(CLONE_NEWUSER | CLONE_NEWNS)` in one call works for an unprivileged process when the harness allows it:
  the gate review ran `unshare --user --mount` successfully,
  and this round's earlier `EPERM` came with a refused `uid_map` write and is this harness's limit,
  not the kernel's.
  (Measured by the review,
  `.../keep/unshare-one-call-user-and-mount.txt`;
  the prepared credential is passed into the namespace build at Linux `v7.3`,
  `kernel/nsproxy.c:221` to `:223`.)

- F17.
  A plain `umount` refuses `EBUSY` while the mount has child mounts or more references than the caller and its parent,
  and a task's working directory or open file is such a reference.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:2004` to `:2014`,
  `fs/pnode.c:495` to `:530` and `:371` to `:374`;
  measured by VM item 4,
  whose busy holder kept its working directory in the staging folder and whose unmount refused.)

- F18.
  `MNT_DETACH` skips that check and detaches the mount anyway,
  while the mount object lives on while any reference to it remains.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:2004` to `:2007`.)

- F19.
  Freeing a mount namespace detaches its whole mount tree,
  and that happens when the namespace's last reference goes.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:6503` to `:6511`.)

- F20.
  OpenZFS takes a long hold on the dataset when the superblock is created for a mount,
  and releases it only when that superblock is destroyed.
  (Source,
  OpenZFS `zfs-2.4.4`,
  `module/os/linux/zfs/zpl_super.c:868`;
  `zpl_super.c:150` to `:159` calls `zfs_umount`,
  and `module/os/linux/zfs/zfs_vfsops.c:1526` to `:1554` disowns the objset.)

- F21.
  The clone's head destroy refuses `EBUSY` unless the dataset's long holds equal the expected count.
  (Source,
  OpenZFS `zfs-2.4.4`,
  `module/zfs/dsl_destroy.c:767` to `:790`.)

- F22.
  A plain unmount of the clone's mount inside a holder namespace drops that namespace's copy,
  and with it one reference to the superblock;
  the long hold goes only when the last reference does.
  A lazy detach leaves the copy alive while tasks hold the mount,
  so it does not release the dataset.
  (Inference from F17 to F21,
  unmeasured for a forced removal;
  VM item 15 measured the same chain from the other side,
  where the head destroy succeeded once the readers stopped.)

- F23.
  The holder scan,
  as built in flight,
  reports each namespace's kernel identity,
  the task whose table was read as a `/proc/<pid>` path,
  and each mount's id,
  root and mount point;
  its text has a parser and a GUID plus nonce acceptance;
  its `holder-scan` mode prints the result and writes nothing.
  It records no start time for the task,
  and its own bounds are `MAXIMUM_MOUNT_NAMESPACES` and `MAXIMUM_DETAIL_LINES` (both 1024,
  adopted with the project's veto open).
  (Work in flight,
  step 9's branch;
  `clone_holder_scan/namespace_holder.rs:42` holds `task: PathBuf`;
  the bounds at `clone_holder_scan/holder_scan.rs:60` and `:71`.)

- F24.
  A receipt is root-owned `0644` in a root-owned `0755` folder,
  named by the dataset GUID,
  carrying the operation,
  nonce,
  outcome,
  mount identity,
  failed step,
  errno,
  boot and time,
  and no holder.
  The failed step and errno words are bounded at 64 bytes.
  The wait matches nonce,
  operation and boot,
  polls every 50 milliseconds,
  and ends at a provisional 30-second deadline.
  (Source:
  `package/git-policy/cli/SPEC.md`,
  "The wrapper's side and its deadline",
  lines 6390 to 6420;
  `clone_mount_receipt.rs:94`;
  `mount_request_wait.rs:59`;
  [proposal][proposal],
  "Requests,
  results and ordering" and "Decided defaults".)

- F25.
  The wrapper already has a bounded path-use scan that matches a working directory or an open descriptor,
  with a measured two-second ceiling over 1,000 processes and 54,500 descriptors,
  and it can match an open file by the mount number and inode the kernel records for the descriptor
  (`UseTarget::Mounted`) rather than by a resolved path.
  It also has a pidfd pin with a birth identity read before and after the open.
  (Source:
  `package/git-policy/cli/src/native/process/process_use.rs`,
  `PROCESS_SCAN_CEILING` and `UseTarget::Mounted` at lines 70 to 75;
  `process/member_signalled_through_its_pidfd.rs`;
  `process/process_identity.rs`.)

- F26.
  The ancestor walk the wrapper already has is `chain_ancestors` (`invocation/started_as_git.rs:309` to `:322`);
  its loop bound is `MOST_ANCESTORS_ASKED_FOR_THE_RECORD` (4096) and its only stop is pid 1.
  It has no session-leader stop,
  and nothing in it is 12.
  (Source,
  same file,
  lines 109,
  293 and 309 to 322.)

- F27.
  The wrapper is the `git` binary here (`0.6.1-alpha.44`,
  shadowing `/usr/bin/git`).
  That a `git worktree remove` runs in that invoking process rather than in the resident commit service is the
  route's own statement and was not re-verified this round.
  (Measured:
  `git --version`;
  the rest unverified.)

- F28.
  The account's own transient unit was killed through `systemctl --user kill --signal=SIGKILL`,
  and the unit went `failed` (measured,
  probe 4).
  The design adds `--no-ask-password` to every such call;
  the measured run did not carry it.
  The system manager's `KillUnit` is gated by the polkit action `org.freedesktop.systemd1.manage-units`;
  the installed defaults are `auth_admin`,
  `auth_admin` and `auth_admin_keep`;
  and `systemctl kill` with `SIGKILL` and the default `--kill-whom=all` kills the unit's whole cgroup,
  while `--kill-whom=` can select `main`,
  `control`,
  `cgroup` or `all`.
  (Source,
  systemd `v262`,
  `src/core/dbus-unit.c:591` to `:600`,
  `src/core/dbus-util.c:161` to `:199`,
  `src/core/unit.c:4179` to `:4181`,
  `man/systemctl.xml:2508` to `:2566`;
  the installed policy file `/usr/share/polkit-1/actions/org.freedesktop.systemd1.policy`,
  lines 32 to 40.)

- F29.
  `systemctl` allows interactive authorization by default,
  and a caller that sets `--no-ask-password` gets `INTERACTIVE_AUTHORIZATION_REQUIRED` instead of a prompt.
  This account is in `wheel`,
  which `/usr/share/polkit-1/rules.d/50-default.rules` names as the administrator identity,
  so `auth_admin` here means this account's own password;
  `/etc/polkit-1/rules.d` is not readable by this account,
  so what a request answers on this host is unverified beyond the shipped defaults.
  (Source,
  systemd `v262`,
  `src/systemctl/systemctl.c:340`,
  `src/systemctl/systemctl-util.c:68`,
  `src/shared/bus-polkit.c:364` to `:367`;
  measured `id -nG` and the rules file and directory modes.)

- F30.
  With no pty (`src/exec_nopty.c`) sudo does not call `setsid`,
  so the wrapper,
  `sudo` and the command share the wrapper's session;
  on the pty path the monitor calls `setsid` (`src/exec_monitor.c:584`),
  so the command's session is the monitor's.
  The command is a child of the sudo process in both cases (`src/exec_nopty.c:614`).
  (Source,
  sudo `v1.9.17p2`.)

- F31.
  A plain `umount2` resolves its argument with `LOOKUP_MOUNTPOINT | LOOKUP_FOLLOW`,
  and its checks require the resolved path to be the root of a mount in the caller's mount namespace,
  not `MNT_LOCKED`,
  with `CAP_SYS_ADMIN` there.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:2077` to `:2129`,
  and `path_mounted` at `fs/internal.h:344` to `:353`.)

- F32.
  A `/proc/self/fd/<n>` argument reaches that same lookup with the descriptor's own path,
  and `LOOKUP_MOUNTPOINT` steps into the topmost mount at the final dentry,
  so the call acts on the mount at the descriptor's location and resolves no name inside it.
  The accepted staging design reads the same rule and unmounts from a covered descriptor.
  (Source,
  Linux `v7.3`,
  the magic-link jump in `fs/proc/fd.c` and `fs/namei.c`'s mount step;
  the rule is read at `v6.8` in `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  "The unmount call from staging",
  and used by its "M2".
  Unrun on this route.)

- F33.
  `move_mount` takes its source from a descriptor with `MOVE_MOUNT_F_EMPTY_PATH` and its destination from a descriptor with `MOVE_MOUNT_T_EMPTY_PATH`.
  It requires the source to be a mount root,
  to have a parent,
  not to carry `MNT_LOCKED`,
  and its parent mount not to be shared,
  and the destination to be in the caller's namespace.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:4743` to `:4813` and `:3765` to `:3793`.)

- F34.
  `/proc/self/mountinfo` prints each row's mount id and parent mount id,
  each mount's source and file-system type,
  and the `shared:`,
  `master:` and `unbindable` tags.
  (Source,
  Linux `v7.3`,
  `fs/proc_namespace.c:137` to `:171`.)

- F35.
  A plain unmount of a mount in a shared peer group propagates to its peers,
  which is why the design refuses a `shared:` row.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:1823` to `:1856` and `fs/pnode.c:718` to `:730`.)

- F36.
  Mount ids come from one global allocator,
  so a mount id names one mount across namespaces,
  and `/proc/<pid>/fdinfo` prints the mount id of a descriptor's open file.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:227` to `:237` and `fs/proc/fd.c:57` to `:59`.)

- F37.
  A mount attach propagates when the mount it is attached to is in a shared peer group,
  and a mount created by `mount(2)` or chosen on a private mount does not.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c`'s `attach_recursive_mnt` and `fs/pnode.c`'s `propagate_mnt`.)

- F38.
  systemd `v262` makes a unit's mount namespace by remounting `/` `MS_SLAVE|MS_REC` and then `MS_SHARED|MS_REC`,
  the default final flag being `MS_SHARED`;
  `MountFlags=` selects `shared`,
  `slave` or `private` instead.
  The review measured a `PrivateMounts=yes` user unit's table:
  all 47 rows,
  including all ten ZFS rows,
  carry `shared:N master:M`.
  (Source,
  systemd `v262`,
  `src/core/namespace.c:2621`,
  `:3119` to `:3131`,
  `:3221` to `:3226`,
  and `man/systemd.exec.xml:2858` to `:2900`;
  measured by the review,
  `~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/private-mounts-unit-mountinfo.txt`.)

- F39.
  `mount(2)` with `MS_PRIVATE` on a mount root makes that one mount private:
  it leaves its peer group and clears its master,
  and its peers keep theirs.
  A later `MS_SHARED` invents a new group,
  so the original group and master cannot be restored.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:2976` to `:3010`,
  and `fs/pnode.c:93` to `:124`.)

- F40.
  `do_move_mount` refuses a source whose parent mount is shared;
  a shared source may move onto a non-shared destination,
  and a private source onto a private destination stays private.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:3765` to `:3793`,
  and the move table at `:2605` to `:2623`.)

- F41.
  A `mount(2)` target of `/proc/self/fd/<n>` resolves through the magic link to the descriptor's path,
  and `MS_PRIVATE` then acts on the mount the descriptor names,
  because the propagation change requires the path to be a mount root.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:4329` to `:4338` and `:2976` to `:3010`,
  and `fs/proc/fd.c:174` to `:190`;
  the rule is the accepted staging design's "The unmount call from staging" and the owner's settled reading of a descriptor path.)

- F42.
  `vfs_rename` refuses to rename a mountpoint,
  so a destination dentry with the moved mount on it cannot be renamed by the holder.
  (Source,
  Linux `v7.3`,
  `fs/namei.c:6175`.)

- F43.
  `/proc/<pid>/exe`,
  `/proc/<pid>/root` and `/proc/<pid>/map_files/<n>` are links to the task's executable,
  its root and its mapped files,
  each needs ptrace access,
  and a `statx` mount id on each names the mount a task runs from,
  is rooted in,
  or maps.
  (Source,
  Linux `v7.3`,
  `fs/proc/base.c:222` to `:243`,
  `:1761` to `:1776`,
  and `:2299` to `:2320`;
  the mount id at `include/uapi/linux/stat.h:216`.)

- F44.
  `MountFlags=` exists in systemd `v262` and takes `shared`,
  `slave` or `private`;
  with `PrivateMounts=yes` and no `MountFlags=` the final flag is shared.
  (Source,
  systemd `v262`,
  `man/systemd.exec.xml:2858` to `:2900`,
  and `src/core/namespace.c:2621`.)

- F45.
  `mount_setattr` carries a `propagation` field at Linux `v7.3`,
  taking `MS_SHARED|MS_SLAVE|MS_PRIVATE|MS_UNBINDABLE`,
  with `AT_EMPTY_PATH` and `AT_RECURSIVE`.
  (Source,
  Linux `v7.3`,
  `include/uapi/linux/mount.h:136`,
  and `fs/namespace.c:5095` to `:5150`,
  `:5285` to `:5357`.)
  It is present in the guest's `v6.8` headers (the review read them) and unrun (L17).

- F46.
  `unshare` sets a new mount namespace's propagation to private by default since util-linux 2.27,
  and `--propagation unchanged` disables that.
  (Source,
  this host's `man unshare`;
  unrun in the guest.)

- F47.
  An unmount reaches the mounts that receive propagation from the unmounted mount's parent,
  never a set chosen by the unmounted mount's own type,
  and propagation runs from a mount to its peers and its slaves,
  never up to its master.
  (Source,
  Linux `v7.3`,
  `fs/pnode.c:572` to `:607` and `:501` to `:531`,
  and `Documentation/filesystems/sharedsubtree.rst:278` to `:283` and `:566` to `:568`.)

- F48.
  `move_mount` refuses a source whose parent mount is shared,
  and a non-shared destination receives no propagation of the move;
  a shared source may move onto a non-shared destination.
  (Source,
  Linux `v7.3`,
  `fs/namespace.c:3783` to `:3790`,
  and the move table at `:2605` to `:2623`.)

- F49.
  Making a mount private or slave removes it from its peer group and takes it off its master's slave list,
  and hands its own slaves to `m`:
  `m` is its master when it has no peers,
  or its next peer when it has peers.
  With no peers,
  private clears the master and slave keeps it;
  with peers,
  slave makes it a slave of the next peer and its `master:` becomes the former group.
  A later shared invents a new group and leaves the master alone;
  a new mount namespace copies the tree,
  with slaves in place of peers only across a user namespace.
  (Source,
  Linux `v7.3`,
  `fs/pnode.c:74` to `:124`,
  `fs/namespace.c:2997` to `:3004` and `:4427` to `:4434`.)
- F50.
  A unix socket bound to a pathname holds the mount of that path ([u->path.mnt]),
  and `sock_diag`'s `UDIAG_SHOW_VFS` returns its bound file's device and inode,
  so a socket bound inside the clone holds the clone's mount while the socket's own descriptor carries no file mount id.
  (Source,
  Linux `v7.3`,
  `net/unix/af_unix.c:1403`,
  and F56's `sock_diag` read.)

- F51.
  A unit with `ProtectHome=read-only` (or another read-only path setting) has the parent mount read-only in its namespace,
  so a directory cannot be made there;
  the mount row carries `ro`.
  (Source,
  `man/systemd.exec.xml`,
  `ProtectHome=` and `ProtectSystem=`;
  unrun in the guest.)

- F52.
  `statfs` reports a file system's free blocks and inodes,
  and the clone's dataset quota and refquota are read through the binding;
  together they say whether one directory fits.
  (Source,
  `statfs(2)` in Linux `v7.3`,
  `fs/statfs.c`;
  the dataset's properties through the binding,
  unrun.)

- F53.
  `open_by_handle_at` resolves a file handle on the mount of the given descriptor and returns a descriptor to the inode without crossing a mount,
  and it needs `CAP_DAC_READ_SEARCH` in the initial user namespace;
  the accepted staging design records the folder's handle and reopens it this way.
  (Source,
  Linux `v7.3`,
  `fs/fhandle.c` (`may_decode_fh` at `:317` to `:323`,
  `capable(CAP_DAC_READ_SEARCH)`);
  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  "The staged-folder record" and "The unmount call from staging".)

- F54.
  A thread that does not share its file-system structure has its own working directory under `/proc/<pid>/task/<tid>/cwd`,
  which the task's `/proc/<pid>/cwd` does not show.
  (Source,
  Linux `v7.3`,
  `fs/proc/base.c`'s `tid` links at `:3671`;
  whether the scan enumerates threads is unverified.)

- F55.
  The walk over the receivers of an unmount is a closure,
  not one level:
  `propagation_next` descends into each receiver's slaves,
  so slaves of slaves are receivers.
  (Source,
  Linux `v7.3`,
  `fs/pnode.c:204` to `:212`,
  with the walk at `:572` to `:607`.)

- F56.
  `sock_diag` with `UDIAG_SHOW_VFS` returns a bound unix socket's bound file by device and inode,
  from `u->path`,
  in the socket's own network namespace,
  where `/proc/net/unix` lists only the reader's network namespace and prints the bind string rather than a resolved path.
  (Source,
  Linux `v7.3`,
  `net/unix/diag.c:29` to `:39` and `:142` to `:143`,
  and `net/unix/af_unix.c:3558` to `:3598`.)

- F57.
  `setsid(1)` forks when its caller is a process group leader,
  which a foreground command of an interactive shell is,
  so the session leader becomes `setsid` itself and the command's parent becomes the manager;
  `setsid --wait` keeps the parent and waits for the command.
  (Source,
  this host's `man setsid`;
  unrun.)

- F58.
  The route repairs the checkout:
  every root run first checks the checkout and sets it private when it finds it shared,
  and the accepted design measured that a private checkout stops late delivery to an already-running unit,
  while the shared arm preserved delivery.
  (Source,
  `doc/decision/cli-git-zfs-checkout-propagation-repair.md`,
  "Who may set the propagation when no person is present",
  and `doc/planning/cli-git-zfs-worktree-design-proposal.md`,
  "Checkout propagation policy" and VM item 3.)

## Assumptions

- A1.
  A removal runs in the host pid namespace.
  The route needs `/dev/zfs`,
  which the confined unit in "Confined callers" cannot see,
  so a container's removal is out of scope;
  a process id,
  a session id and a pidfd mean the same thing to the wrapper and to root only inside one pid namespace.

- A2.
  One wrapper at a time acts on one slot,
  under the per-slot claim the design already requires.

- A3.
  The caller property,
  the force property and the intent are written by that one wrapper in that order,
  so a matching nonce proves they belong to one request.

- A4.
  The account is trusted for this machine's threat model ("We have to assume my machine isn't compromised.
  sudo is passwordless here anyway.",
  [staging][staging],
  lines 223 to 236).
  The nonce and liveness checks are guards against a stale record and a bug,
  not a boundary against the account.

- A5.
  The scan file and the receipt are published atomically under the root-owned results folder,
  by the same staging-name-then-rename rule the receipts use.

- A6.
  The handler can read the wrapper's chain:
  both run in the host pid namespace (A1),
  and the handler runs as root,
  so every `/proc/<pid>/stat` on the chain is readable (F11).

- A7.
  The handler does not read the account-owned removal claim:
  the sibling places it in the repository and says "the handler never reads it at all" ([cancelled][cancelled],
  "The removal claim").
  Root's liveness proof is the wrapper process,
  not the claim.

- A8.
  The handler runs in the host mount namespace,
  with no `PrivateMounts`,
  so its own namespace is pid 1's and the host's clone mount is the one its staging turn unmounts.
  A handler with a private mount namespace would see neither that mount nor the host's path holders,
  and the route is not designed for one.

## Limits

- L1.
  No guest ran in any round.
  Every sentence about the dataset releasing,
  the propagation rules,
  the private and slave changes,
  the move,
  the unmount and the handle route is an inference from source plus VM items 3,
  4 and 15,
  not a measurement.

- L2.
  The host probes measured this kernel and this systemd,
  not the route's binaries.

- L3.
  One host probe could not write `uid_map` and saw `EPERM` for a one-call user and mount namespace (F16);
  the review's capped run succeeded.

- L4.
  A session id and a process id are numbers inside one pid namespace;
  neither identifies a process across namespaces,
  and a pidfd created in one namespace is not usable from a sibling namespace.

- L5.
  Two commands broke the project's rule and are not evidence:
  `systemctl kill --signal=0 dbus.service` (stopped by `systemctl`'s argument check) and `systemctl kill --signal=KILL systemd-random-seed.service` (which put a password dialog on the person's desktop;
  the unit stayed active).

- L6.
  No task in an uninterruptible sleep was available to probe;
  the preflight's endability read is unmeasured.

- L7.
  A mount namespace's inode number,
  a `listns` id and a mount id can be reused after the object dies,
  so the design reads each object again beside the number.

- L8.
  The forced arm's cost is not measured:
  the wait,
  the private or slave change,
  the move,
  the unmount and the directory write are owed to the guest.

- L9.
  The wait between `SIGTERM` and `SIGKILL` ships no number (the record's words).
  It is a constant with no value,
  set by step 17's build only after the owner decides one or M12 sizes it;
  the forced arm does not ship before then.
  The removal's own deadline is a client report deadline,
  not this wait.

- L10.
  A service unit's parent is the service manager,
  so `--force` is refused there,
  and also when the command's session was started by the manager;
  a harness that must force runs the removal as its own child or under a `--scope`.

- L11.
  `/etc/polkit-1/rules.d` is unreadable to this account,
  so local polkit rules are unread and what `systemctl kill` answers here is unverified (F29).

- L12.
  `listns`,
  a namespace file handle,
  the descriptor unmount,
  the propagation change by `mount(2)` on a descriptor path,
  and the recorded-handle open were read at source and not run this round (F14,
  F31,
  F32,
  F41,
  F53);
  the probe arms owe them.

- L13.
  A confined unit the Pi driver starts is not an ancestor of the wrapper,
  so `--force` can end it if it holds the clone,
  and the handler's own namespace rules do not reach into its unit.

- L14.
  The task matcher over `cwd`,
  descriptors,
  `exe`,
  `map_files`,
  `root`,
  the threads' own working directories and bound unix sockets is new code over the handler's own walk;
  only the descriptor record and the path-use ceiling it reuses are measured.

- L15.
  A mount made private is not restored exactly:
  re-sharing invents a new peer group.
  A mount made slave keeps its master when it has no peers,
  and becomes a slave of its next peer when it has peers,
  showing the former group as master (F49);
  a later shared gives a new group with that master,
  so the slave change is nearly reversible.
  The in-place way 8 changes no propagation and leaves nothing to restore.

- L16.
  The shared-namespace answer is a direction:
  this document designs the ways,
  a guest measures the recommended one,
  and the project confirms afterwards.
  Until then nothing here is final.

- L17.
  `mount_setattr` carries a propagation field in the guest's `v6.8` headers (the review read `include/uapi/linux/mount.h` 134 and the `statmount` fields at 169 to 171),
  so the header is present and unrun;
  the design uses `mount(2)` on a descriptor's `/proc` path and does not depend on it.

- L18.
  The guest image runs `6.8.0-147-generic`,
  whose syscall table ends at 461 and has no `listns`.
  No guest with the production kernel exists (the fixture's item 16:
  the image registry offers no CachyOS guest),
  so Tier 0 runs on `6.8` as informative,
  the taskless-namespace arm and the production evidence need a guest of `7.3.5-1-cachyos`'s generation,
  and a CachyOS qcow2 with `linux-cachyos-zfs` and a guest agent is what the project must build for it.
  Every arm records `uname -r` and `zfs version`.

- L19.
  A directory created in the checkout's filesystem by way 7 is visible to the host and to snapshots;
  a kill can leave it;
  the next removal's handler sweep and the cancel command remove one whose reserved prefix `.git-worktree-clone-removal-` and checks pass,
  and `doctor` prints it without removing it,
  which is unmeasured.

- L20.
  A hardened unit (`ProtectHome=read-only`,
  a read-only parent) refuses way 7's directory write;
  way 8,
  which writes nothing,
  is the only way there if its handle exists;
  both are unmeasured.

- L21.
  The act's liveness uses a progress file,
  a heartbeat period `T_h` and a give-up window `T_g` (four `T_h`),
  all unmeasured and all constants with no value until M12 sizes them;
  the wrapper's and the handler's waits end on each other's death only through them.
  `T_g` is a stated multiple of `T_h` so a reader compares whole periods,
  and the counter is advanced by a timer thread so a long stage is not read as death.

- L22.
  The repaired state is an inference from the propagation design (every root run sets the checkout private first) and from `fs/pnode.c`;
  the guest arm that starts a unit while the checkout is shared and then repairs it is what shows the chain.

## The caller and its protected set

The caller is the wrapper's own process.
The wrapper is the `git` binary here (F27),
so the command a person or a harness ran is that process,
and the wrapper needs no environment variable,
no argument and no configuration to know it.
Its birth identity is the start time the kernel recorded for it,
the value `process_identity` already computes,
and with the boot id it names one process across a number's reuse.

The request carries that identity and the one hop the handler must check first,
in a clone-local property written before the intent:
`org.monochromatic:git-worktree-clone-removal-caller`,
value `v1:<nonce>:<pid>:<starttime>:<parent-pid>:<parent-starttime>:<boot>`.
The nonce is the intent's 32 lowercase hex digits,
the numbers are decimal,
and the boot id is the kernel's own;
the byte length is bounded before parsing,
and a malformed value sets nothing.

The handler proves the wrapper's pid and start time,
reads the wrapper's parent and requires its pid and start time to be the recorded pair,
and only then walks upward.
The walk is the same rule as `chain_ancestors` (F26),
from the proved parent to pid 1,
with two checks:
after reading each parent's record the handler re-reads the child's `ppid` and requires it still to name that parent,
and it requires the parent's start time not to exceed the child's.
The handler also holds a pidfd on the wrapper,
opened once the wrapper's pid and start time are proved,
and checks it immediately before every signal and every unmount,
and includes it in the poll set the act waits on.
The record decides this:
a forced request's authority lasts only while the very command that asked is running.

The protected set,
which the handler spares and never ends,
is two things:
every task whose `session` field equals the wrapper's session (F11),
and every ancestor of the wrapper from its parent to pid 1.
A holder in either set refuses the removal in both arms:
without `--force` the removal cancels and the clone is reattached (the second answer),
and with `--force` the refusal is `caller-protected-holds`.
[The sibling][cancelled] files the same word for the no-force arm in its "What the caller sees".

The shapes:

- Pi,
  a command the driver spawns (F1):
  the wrapper is its own session leader and the driver,
  its parent,
  is in another session;
  the protected set is the wrapper and its undetached children,
  the driver and every ancestor to pid 1.
- Claude Code's Bash tool (F4):
  the wrapper's parent is a session-leading `zsh` with no terminal;
  the session is that `zsh`'s and the ancestors are the `zsh`,
  `claude` and above.
- `systemd-run --user --scope` (F3):
  the session and parent are the invoking shell's.
- `systemd-run --user --pipe --wait`,
  a service unit (F5):
  the wrapper is its own session leader and its parent is `systemd --user`;
  this is refused,
  and now also when the session itself was started by the manager ("A command whose parent is the service manager").

### A command whose parent is the service manager, or whose session was started by it

The record's fourth answer refuses `--force` when the command's parent is the service manager;
its "One more answer" refuses it also when the command's session was started directly by the service manager,
so a unit whose main process is a script that runs the removal is caught.
The check reads two things,
both from the kernel's own data:

- the wrapper's parent,
  as before:
  the system manager is the parent whose pid is 1;
  the user manager is the parent whose `/proc/<pid>/cgroup` row is exactly `/user.slice/user-<uid>.slice/user@<uid>.service/init.scope` for the wrapper's uid,
  and whose `comm` is `systemd`;
- the session leader's parent,
  by the same recognition:
  the session leader is `getsid(0)`'s process,
  read from `/proc/<sid>/stat`,
  and its parent is read the same way.

Either match refuses.
A session leader whose record cannot be read,
or whose parent has gone,
refuses too,
with the orphan sentence:
the record's words do not cover it and refusing is the safe reading (the review's R5).
A user process cannot create the manager's cgroup path,
because it lies in a systemd-owned slice,
so the check is not a name test an account can satisfy with a look-alike cgroup.

The check runs in the wrapper before it writes the force property:
the removal exits `2` and prints no request.
The text:

```text
the removal of <slot> was asked to end the holders, and this command was started by the service manager: its <parent | session leader's parent> is the manager (pid <pid>, <systemd or systemd --user>), which cannot be told apart from the process that asked for it. If you did not start this removal as a service unit, the command was reparented to the manager because its own parent exited, or it runs in a console login's session; nothing was changed and nothing was ended: the clone, its snapshots and the Git registration are as they were. Run the removal as a child of the process that wants it: from that process's own shell, or under `systemd-run --user --scope` started by that shell; in a console login, start it under `setsid --wait` so its own session begins at the command. Do not run it as a service unit.
```

The consequences,
stated:
a unit whose main process is a script is refused (the new answer);
a console login's shell has `login` as its session leader,
started by the manager,
so a `--force` removal typed there is refused and the text says to use `setsid --wait`;
an orphaned wrapper is refused with the orphan sentence;
that a console login's session leader is `login`,
started by the manager,
is the review's reading and was not run,
and `setsid --wait` is the remedy because `setsid` alone forks when its caller is a process group leader (F57,
A13);
and the measured agent shapes stay allowed,
because Pi's and Claude Code's commands have an agent process as the session leader's parent,
and a `--scope` child stays in the invoking shell's session.
A wrapper that is alive but whose recorded parent has changed or died has its force word declined as `caller-gone` by the handler;
the wrapper prints that nothing was ended and that the removal should be run again as a child of the process that wants it.

## What a holder is, and what the proof establishes

A holder is one of two things,
and the design treats both with one proof:

- a task whose mount namespace holds a mount of the clone's dataset,
  which is what the root scan reports (F23);
- a task whose working directory,
  open file,
  running executable,
  mapped file,
  root,
  thread working directory or bound unix socket lies inside a mount of the clone,
  which the scan does not report and which the handler's own walk finds.

The second kind is not decoration.
A working directory or an open file makes a plain unmount refuse (F17);
a running binary,
a mapped library or a chroot holds the mount through `exe`,
`map_files` and `root` (F43);
a unix socket bound to a path inside the copy holds its mount through `u->path.mnt` (F50);
and a thread that does not share its file-system structure has its own working directory under `/proc/<pid>/task/<tid>/cwd`,
which the task's own `/proc/<pid>/cwd` does not show.
The match is by the mount id the kernel records:
a descriptor's `/proc/<pid>/fdinfo` (F36),
`statx` `STATX_MNT_ID` on `cwd`,
`exe`,
each `map_files` entry and `root` (F25,
F43),
each thread's `task/<tid>/cwd` (F54),
and for a socket its bound file's device from `sock_diag` and `UDIAG_SHOW_VFS` (F56) matched against the clone's filesystem.
The whole matcher is new code over the handler's walk (L14).
A read the handler cannot make is a blind spot (a refusal with `EACCES` or `EPERM`;
a task that has gone,
`ENOENT` or `ESRCH`,
is gone,
not blind),
and at the preflight a blind spot refuses the removal before any act:
the design cannot prove the unmount will not meet a holder it could not see.

When the unmount is refused `EBUSY` and the walk found no holder,
the report says the reads were empty over `cwd`,
descriptors,
`exe`,
`map_files`,
`root`,
the threads and the unix sockets,
that a reference the design cannot see remains,
and refuses `holder-unseen` naming the copy,
the namespace and the errno;
it never retries blind.

Kind (d),
a namespace whose task is gone,
is reachable under the chosen reach:
F13 measured that such a namespace lives on a descriptor,
F14 says a privileged caller in the host pid namespace can list every active namespace with `listns` and reopen one by its file handle,
and F15 says entering it with `setns` must happen in a single-threaded process.
The guest's `6.8` kernel has no `listns` (L18),
so that arm runs on the production generation.

The scan's report is evidence that a namespace holds the clone,
never the operand of a signal.
The handler pins first and reads afterwards,
in this order:

1.  It takes a candidate task,
    from the scan's namespace and its own enumeration of that namespace's tasks and threads.
2.  It reads the task's start time,
    opens a pidfd while the number names that process,
    and reads the start time again,
    requiring the two to agree,
    which is the shape `PinnedMember::open` already uses (F25).
3.  Through the pidfd it reads the mount namespace (`PIDFD_GET_MNT_NAMESPACE`,
    F10) and requires it to equal the fresh scan's namespace identity,
    and it reads the task's session from `/proc/<pid>/stat` (F11).
4.  It proves the pinned task is still the same live process:
    a zero-timeout `poll` on the pidfd,
    or a successful `PIDFD_GET_INFO`,
    whose own fresh task lookup returns `ESRCH` once the task is gone (F10).
5.  Only then does it signal the task,
    under the order "preflight,
    `SIGTERM`,
    wait,
    `SIGKILL`" the act states.

A signal travels through the descriptor,
so a reused number cannot receive it;
the scan's representative task is evidence,
and its lack of a start time (F23) is asked of step 9's build so a report can name it,
not assumed.

## How root's finding reaches the wrapper

The unprivileged wrapper needs the finding for three things:
to name the holders in its report and in `doctor`,
to show the caller's protected set among the processes the removal spares,
and to know whether a scan was clear before the destroy that follows it.
The receipt cannot carry it:
F24's grammar has fixed keys and no holder field,
and it belongs to the mount protocol.

The transport,
adopted with the project's veto open (V1),
is a second root-owned file published beside the receipt:

- the path `/run/git-worktree-clone-mount-results/<guid>.holder-scan`,
  with the dataset GUID as the whole name before the suffix,
  root-owned `0644` in the root-owned `0755` folder;

- the content is the scan's own text,
  which F23 says already has a writer,
  a parser and a bound,
  so the wrapper reads it with `parse_holder_scan` and accepts it with `accept` against the GUID and the nonce of
  the request it wrote;

- root publishes it before the receipt,
  so a receipt that says a scan happened is never read beside an older scan;
  a missing file is a missing answer,
  never a clear one;

- a receipt whose nonce matches the request and whose scan file is missing is a refusal of the removal at that step,
  not an `unknown` that goes on (V2).
  The proposal's "not refusing by itself" (line 739) is about a report the wrapper could read;
  a report that never arrived is not that;

- the file's lifetime is the receipt's:
  one per dataset GUID,
  replaced by the clone's next result,
  never removed when the clone is destroyed,
  and reported by `doctor`,
  which is the cost the receipts default already accepted.

The wrapper keeps three answers apart:
`clear` (no holder,
no incomplete read),
`holders` (the namespaces and their mounts),
and `unknown` (an incomplete read).
Only `clear` lets the removal go on without naming anything;
`holders` is what the refusal and the forced report name;
`unknown` is reported,
and the proposal reads such a report as not refusing by itself (line 739),
which V2 marks as a reading rather than a decision.

`doctor` reads the same file for the last request it finds and the live scan mode for its own reading.
It never kills,
never unmounts,
and never needs root:
the mode is the handler's,
and `doctor` keeps the proposal's rule that it names holders by namespace identity and representative task and
reports incomplete coverage as unknown.

## Who ends each kind of holder

The first answer ([record][record],
"By what channel a holder of another account is ended") is the channel:
the installed root handler,
on the account's request,
through the property and receipt protocol.
No second privileged channel is added,
and the account alone does not act.

The kinds this machine produces are these,
and the chosen channel meets all of them in one removal:

- kind (a),
  the same account in another session:
  a leftover agent shell or a build job that `cd`'d into the clone or mounted it;
- kind (b),
  another account's process:
  a root-run reader,
  for example a root `systemd-run` job that mounted the clone;
- kind (c),
  a unit's task the caller cannot signal:
  the same root-run job seen as its unit's task,
  or a task in a system unit;
- kind (d),
  a namespace whose task is gone:
  pinned by a descriptor (F13),
  reachable through `listns` or a namespace file handle (F14);
- kind (e),
  a path holder:
  a task of any account with a working directory,
  open file,
  running executable,
  mapped file or root inside the clone,
  which keeps the mount alive without mounting it (F17,
  F18,
  F43).

One forced removal from start to end,
on a clone at `.claude/worktrees/one`:

1.  `git worktree remove --force <slot>` writes the caller property,
    the force property and then the intent,
    all with one nonce,
    and waits for the receipt (F24).
2.  The handler rechecks the intent under the per-slot claim,
    reads the three matching values,
    proves the wrapper's pid,
    start time and recorded parent (A6,
    N2),
    and walks the chain to pid 1 (F26).
3.  It scans before it moves anything and publishes the scan file,
    then runs the read-only preflight over every holder namespace ("The preflight").
   A structural refusal there refuses the removal before any signal and any mount change.
4.  It compares every holder with the protected set.
    None is in it,
    so the structural stage runs over every namespace first (way 8's open and check,
    or way 7's slave change and directory),
    then `SIGTERM` to every blocker,
    a wait,
    `SIGKILL` to the survivors (the wrapper's pidfd checked before each signal),
    then the unmount stage (way 8's unmount,
    or way 7's move and unmount) ("The reach").
   Kind (d) is found by `listns` and treated like any other namespace.
5.  It re-reads the wrapper's pidfd after each pass.
    While it stands,
    the act goes on;
    when it stops,
    the act stops and the receipt says `caller-gone`.
6.  It scans again,
    moves the clone to staging,
    unmounts its own mount without force and without lazy detach,
    and files the scan file and then `done`.
    This is the success path only:
    when the host's staging unmount stops short,
    the handler files the fourth outcome word of the staging record's "The answer a stopped unmount files,
    answered 2026-11-23" (`doc/decision/cli-git-zfs-staging-under-cache.md`) at once,
    carrying the clone's place and the bind's state,
    and the removal ends as a cancelled removal that the sibling owns.
    The structural stage and the unmount inside a holder's namespace are not that case:
    they stop with the host's clone unstaged and file this design's refusal words.
    Step 16's scans follow under fresh nonces,
    each carrying the force word again.
7.  The person sees the ordinary removal report plus the holders it ended,
    by namespace and task,
    the propagation it changed in a holder's namespace when the reach made such a change,
    and any destination directory it left.
   The head destroy succeeds.
   A holder the act could not end,
   or a namespace it had to refuse,
   is `holder-unendable`,
   with what was ended and what changed named.

### Options the answers did not choose

One place,
briefly,
with the record:

- The channel:
  A,
  the account alone with a refusal for what it cannot end,
  because it contradicts "Kill or otherwise remove or destroy the holders whenever --force is specified" for other accounts' holders;
  C,
  a root helper through `sudo -n`,
  because it adds a second privileged channel and a sudoers line;
  D,
  the systemd managers,
  because a system-manager request prompts a person or fails and a unit kill can end the caller's own harness.
- A seen holder:
  proceeding to delete while the holder is still seen;
  and refusing while keeping the removal pending,
  because the accepted answer cancels and the checked reattach returns a usable worktree.
- The reach:
  ending every task of each holder namespace,
  alone or as a fallback,
  because it ends processes that held nothing;
  and refusing a namespace whose mounts are shared,
  because the further answer chose to design an unmount there.
- The protection:
  the caller's session alone,
  because it ends the harness;
  the session plus ancestors to the first session leader,
  because it misses the harness in every measured shape;
  and the same without the service-unit refusal,
  because a service unit's parent cannot be told apart from the requester.
- The signal:
  `SIGKILL` alone,
  and `SIGTERM` alone,
  because the first loses unsaved work and the second can leave a blocker.
- The authority's duration:
  while the nonce matches and the session exists,
  and for a fixed time,
  because the first ends later processes and the second ships a number.

The record's "What was not chosen" lists the same sets.

## The reach: unmounting inside a holder's namespace

The third answer ([record][record],
"How much a forced removal ends in another mount namespace") chooses the unmount and ends only its blockers,
with no fallback.
Its "Four further answers",
"A holder in a namespace whose mounts are shared",
chose "B:
design an unmount there":
the design finds a way to unmount the copy inside such a namespace,
for example by making its parent mount private first,
so that the service keeps running and the removal finishes.
The record states the cost,
that it changes what a running service's namespace receives from the host,
and that it is a direction:
a design round,
a guest measurement,
then the project confirms.
Its correction withdraws the sentence that the answer was the authorisation for "the clone's own copy and nothing wider";
how far the change reaches is this design's to propose,
and no way is excluded for being wider than an authorisation the record never gave.
This section proposes,
states each way's cost,
and recommends one for the project to confirm after the guest.

### What the act is aimed at

The aim is to release the dataset's long hold,
not to free the namespace (F20,
F21).
A plain unmount of the copy inside a holder's namespace drops that namespace's reference (F22);
a lazy detach leaves the copy alive while a task holds it,
so it does not release the dataset (F18,
F22).
A mount under the copy,
or a task that references it through its working directory,
an open file,
its running executable,
a mapped file,
its root or a bound unix socket,
makes the plain unmount refuse `EBUSY` (F17,
F43,
F50);
the first is a mount and no signal clears it,
the second is a blocker and is ended.

### What the kernel does with propagation

Every way is judged by these rules,
read at Linux `v7.3`:

- An unmount reaches the mounts that receive propagation from the **unmounted mount's parent**,
  never a set chosen by the unmounted mount's own type (F47),
  and the walk over receivers is a closure:
  `propagation_next` descends into each receiver's slaves,
  so slaves of slaves are receivers too (F55).
- Propagation runs to peers and slaves,
  never up to a master (F47).
- `move_mount` refuses only a source whose parent mount is shared,
  and a non-shared destination takes the move with no propagation (F48).
- Making a mount private or slave takes it out of its peer group and off its master's slave list,
  and hands its own slaves to `m`:
  `m` is its master when it has no peers,
  or its next peer when it has peers.
  With no peers,
  private clears the master and slave keeps it;
  with peers,
  slave makes it a slave of the next peer and its `master:` becomes the former group (F49).
  A later shared invents a new group (F49).
- In the route's own state,
  the checkout is private:
  every root run sets it private first when it finds it shared (F58),
  and the accepted design measured that a private checkout stops late delivery to an already-running unit (F58,
  VM item 3).
  A unit started after that has no master on the mount that carries the slot;
  a unit started while the checkout was shared keeps its master only if the repair left it one,
  which depends on the host mount's own master and peers and is the guest's to show (L22).

### The ways out of a namespace whose mounts are shared

Eight ways were weighed;
"private" and "slave" mean `mount(2)` with `MS_PRIVATE` or `MS_SLAVE` on a descriptor's `/proc` path (F41,
F47).

#### Way 1: make the copy and its parent private, then move and unmount

It changes two mounts:
the parent,
whose privacy stops the propagation,
and the copy,
which by F47 decides nothing about where the unmount goes;
the copy's change cannot be undone exactly.
In the repaired state the parent's change is way 3's;
with a shared parent it also cuts the service off from the host's later mounts and unmounts under it.
A kill leaves both private,
and the copy at its path or the destination.
It names everything by descriptor after the checks;
it is way 3 plus a needless change.

#### Way 2: make only the copy private, then unmount it in place

It changes the copy only,
and that change does not stop the unmount:
by F47 the parent is still shared,
so it reaches the parent's receivers exactly as way 4 does.
The service keeps everything;
the receivers lose their copies.
A kill leaves the copy private,
which no later share restores exactly.
It names the copy by a path or a parent descriptor and a name,
both of which the accepted staging design forbids acting on;
it is way 4 plus a needless change.

#### Way 3: make only the parent private, then move and unmount at the destination

It changes the parent only,
from the act until the namespace dies;
nothing restores it (L15).
The unmount has no receivers (F47).
In the repaired state,
where the parent has no master and no peers,
this is the same change as way 7 and the service loses nothing it was receiving;
with a shared parent,
the service loses the host's later mounts **and unmounts** under it,
so other slots' clones copied into it stay mounted there after the host removes them,
and the parent's slaves are handed to its master,
or to its next peer when it has peers (F49).
A kill leaves the parent private,
the copy at the destination and the directory there.
It names everything by descriptor.

#### Way 4: plain unmount of the copy in place, changing nothing

It changes nothing,
and the unmount reaches the parent's peers and slaves (F47,
F55);
a receiver holding the same copy loses it or the unmount refuses `EBUSY` until its blockers are ended.
The host's mount stays only where it is a master or no receiver of the parent,
which the repaired state's private checkout guarantees and a shared checkout does not.
A kill leaves nothing to restore.
It names the copy by a path or a name in the holder's tree,
which the accepted staging design forbids.

#### Way 5: make the copy private, then lazy-detach it by descriptor

It changes the copy only;
nothing reaches others;
a reference the design cannot see keeps the detached copy and the dataset alive.
It is a lazy detach,
which the accepted busy-teardown and staging sentences forbid.
A kill leaves a detached copy held by a reference.

#### Way 6: make the whole holder namespace private recursively, then move and unmount

It changes every mount in the namespace for its life;
the unit receives no host mount change anywhere and every nested relation is re-pointed.
A kill leaves all of it private.
It names everything by descriptor.

#### Way 7: make the parent a slave, then move and unmount at the destination

It changes the parent only.
With no peers,
a slave keeps its master (F49);
in the repaired state it has none,
so it reads in `mountinfo` as private and the change is way 3's.
With peers,
it becomes a slave of its next peer,
its `master:` becomes the former group,
and its own slaves are handed to that peer (F49);
the service keeps receiving through the peer,
and the peer namespace stops receiving the unit's later mounts under the parent.
The unmount has no receivers;
the move passes (F48).
A later shared gives a new group with the same master,
so the change is nearly reversible (L15).
A kill leaves the parent a slave,
the copy at the destination and the directory there.
It names everything by descriptor.

#### Way 8: unmount the copy in place by a recorded handle, changing nothing

It changes no propagation,
makes no move and writes no directory:
a descriptor opened from the add's handle names the directory the copy covers without crossing the mount (F53),
and `umount2("/proc/self/fd/<n>", 0)` unmounts the topmost mount at that dentry,
the copy (F31,
F32).
The service loses the copy only;
the receivers are the parent's peers and slaves,
so it is usable only when the preflight's receiver closure has no member outside the namespaces the act handles that has a mount at the slot (F55);
where a member is one of those,
its copy is unmounted too or refuses until its blockers are ended,
and a copy already gone when its turn comes is success.
A kill leaves nothing to restore.
It needs a handle recorded before the clone was mounted (D7),
so clones that predate it fall to way 7.

Not a way:
asking systemd to set `MountFlags=private` or `slave`,
or to stop using `PrivateMounts`,
needs a unit change and a restart,
which is not a removal act.
Not offered by the kernel:
moving the copy into an anonymous namespace (F40).
Not a dependency:
`mount_setattr`'s propagation field is present in the guest's headers and unrun (L17).

### Ranking and the recommended way

The ranking is 8 > 7 > 3 > 1 > 4 > 2 > 5 > 6,
the review's and mine,
and it does not flip on the repaired state:
way 7 is never worse than way 3,
equal where the parent has no master and no peers,
better where it has a master or peers.

- 8 over 7:
  no change,
  no move,
  no write;
  but it needs the handle and a receiver closure with no outside member at the slot,
  and is unmeasured.
- 7 over 3:
  equal in the repaired state;
  where the checkout is shared,
  7 keeps the service's reception where 3 cuts it off.
- 3 over 1:
  the same effect with one irreversible change fewer;
  1's copy-private step buys nothing.
- 1 over 4:
  1 names everything by descriptor and propagates nothing;
  4 names the copy by a path the accepted staging design forbids.
- 4 over 2:
  2 is 4 with a change that buys nothing.
- 2 over 5,
  5 over 6:
  5 is a detach the accepted rules forbid;
  6 changes every mount for good.

The recommended policy is way 8 where the preflight proves the handle and no outside receiver with a mount at the slot;
otherwise way 7,
which in the repaired state is also way 3's change;
one directory in the checkout,
named and removed;
a namespace that is not a holder and receives from the parent is not refused:
the act proceeds and reports the namespace and what it stops receiving (V8);
and refuse,
naming the holder,
where neither fits.
The project confirms this after the Tier 0 and Tier 1 arms and the Tier 2 arms the guest runs (V5).

### The recommended ways, step by step

After the preflight and the signal stage ("The preflight",
"The signal"),
for one holder namespace:

**Way 8,
in place by the handle.**

1.  The preflight proved the handle;
    the handler opens it on the checkout's mount in the holder's namespace (F53).
2.  It reads `/proc/self/mountinfo` there and requires the topmost mount at the descriptor's dentry to be the copy,
    by mount id,
    source and type (F34,
    F36).
3.  It calls `umount2("/proc/self/fd/<handle fd>", 0)`,
    plain;
    the descriptor is on the checkout's mount,
    not on the copy,
    and the lookup steps into the topmost mount at its dentry (F31,
    F32).
4.  It re-reads:
    the copy's mount id is gone,
    and no row of the dataset remains.

**Way 7,
the parent a slave.**

1.  In the structural stage,
    before any signal,
    it opens the copy's and the parent mount's roots,
    checked by `statx` mount id and `fstatfs`,
    makes the parent a slave with `mount(NULL, "/proc/self/fd/<parent fd>", NULL, MS_SLAVE, NULL)`,
    and re-reads the parent row:
    no `shared:`,
    and a `master:` that is the old master when it had no peers,
    or the former group when it had peers (F49).
2.  In the same stage it creates one directory on the parent mount's root,
    mode `0700`,
    named with the reserved prefix and the clone's GUID,
    and opens it;
    the attach is on a non-shared mount and does not propagate (F37).
3.  In the unmount stage,
    after the signals,
    it moves the copy with `move_mount(copy_fd, "", destination_fd, "", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH)` (F48),
    closes the copy's descriptor,
    and re-reads that the copy's id is topmost at the destination.
4.  It calls `umount2("/proc/self/fd/<destination fd>", 0)` and re-reads that the id is gone and no row of the dataset remains.
5.  It removes the directory with `unlinkat` on the parent's descriptor and the recorded name,
    and reports the propagation change and the directory's life.

The move sits after the signals because a blocker that opens the slot by an absolute path during the `SIGTERM` wait would otherwise create files in the checkout's own filesystem through the emptied slot;
with the move after the wait,
such a write lands in the mounted copy instead,
and the slot's folder is empty when the removal later needs it so (A5).
The destination is at the parent mount's root,
which is the checkout's own mount:
the host,
every namespace and a ZFS snapshot taken in the window see the directory,
`git status` sees it,
and a kill leaves it.
The handler removes it after the unmount;
the receipt and the journal record the name;
the next removal's handler sweep and the cancel command remove one whose reserved prefix `.git-worktree-clone-removal-` and clone GUID match,
which is empty,
root-owned,
mode `0700`,
and directly under the parent mount's root (L19).
A hardened unit has a read-only parent,
so way 7 is refused there and way 8 is the way in when its handle exists (L20).

### The preflight

No forced removal may end a process or change a mount before it knows it can finish with every holder namespace it will meet.
The preflight is read-only and runs over every holder namespace before the first structural act,
the first signal and the first unmount:

- It forks one single-threaded child per namespace,
  enters it (`setns` on the scan's task's namespace,
  a `listns` descriptor for a taskless one),
  and reads `/proc/self/mountinfo` there (F14,
  F15).
- It checks the parent:
  the mount the copy sits on is the checkout's own mount,
  by source,
  root and mount id;
  anything else is refused.
- It runs every structural check for each dataset row:
  exactly one own-root copy,
  not the namespace's root,
  no child mount below the copy,
  no second mount of the dataset,
  no `MNT_LOCKED` the move would refuse (F48),
  the tags recorded,
  the parent's root path and mount id.
- It computes the receiver set as the closure over groups,
  and this bullet is its one statement:
  the seed is the parent's `shared:` group when the parent is shared,
  or,
  when it is not,
  the group its slaves print as their `master:` (or `master:0`,
  which names no group);
  where neither names a group,
  way 8 refuses for any receiver it cannot name,
  and way 7 proceeds and reports.
  A row is a member when its `shared:` group or its `master:` is already in the set,
  and a member adds its own `shared:` group to the set;
  repeat until nothing is added (F55).
  It builds the set from every reachable mount namespace's table:
  the scan's list,
  and `listns` for taskless namespaces;
  the scan's bound applies,
  and a scan that stopped at its bound is not clear.
  A namespace whose table cannot be read is reported,
  and for way 8 it refuses.
  It keeps only members that have a mount at the slot's dentry,
  and records each kept member's topmost mount there,
  which must be the copy,
  not a stacked mount.
- For way 8,
  a kept receiver outside the namespaces the act handles means way 8 is not used and the act falls to way 7;
  it is not a refusal.
  For way 7 a kept receiver outside the act is not refused:
  the act proceeds and the report names the namespace and what it stops receiving under the parent (V8).
  Every other place that speaks of a receiver outside the act cites this bullet and says the same.
- It checks the handle for way 8 and,
  for way 7,
  that a directory can be made:
  the parent is not read-only (F51),
  its root is a directory,
  the name is free,
  and the parent dataset has free space and inodes and,
  where a quota is set,
  room for one directory (F52).
- It reads the protected set and every holder through the matcher,
  and each holder's endability:
  the state from `/proc/<pid>/stat` and `/proc/<pid>/status`,
  whether it is a kernel thread,
  and whether a pidfd can be opened.
  A holder it cannot end refuses here,
  before any signal.
  A blind spot is a read refused with `EACCES` or `EPERM`;
  a task that has gone (`ENOENT`,
  `ESRCH`) is gone,
  not blind;
  a blind spot refuses.
- It reads bound unix sockets by `sock_diag` with `UDIAG_SHOW_VFS` in each network namespace,
  whose answer is the bound file's device and inode (F56),
  and matches the device against the clone's filesystem;
  the owning task is found by the socket's inode in `/proc/<pid>/fd` and ended like any other blocker.
- A structural refusal,
  an unenterable namespace,
  a protected holder,
  an unendable holder,
  a blind spot,
  a parent that is not the checkout's,
  a parent that is not the checkout's,
  or no destination for way 7,
  or no destination for way 7 refuses the removal before anything is ended or changed.

The order is fixed:
the preflight over every holder namespace;
the holder namespaces are handled in namespace-inode order,
so two runs meet them in the same order;
then the structural stage over every namespace (way 8's open and check,
or way 7's slave change and directory);
then the signal stage over every namespace;
then the unmount stage over every namespace (way 8's unmount,
or way 7's move and unmount).
The structural stage comes before any signal because a busy mount can be moved (F17's check does not stop a move) and the kernel can refuse that stage for a cause reading cannot know;
after it,
only a blocker that will not die and an `EBUSY` race follow an ending.
A kernel refusal in the middle of the structural stage is the third refusal state:
nothing was ended,
the namespaces before it were changed (a parent a slave,
a directory made),
no copy was moved,
the directory is removed when safe and the parent stays a slave and is reported (A9).
Way 8's unmount has receivers,
so all blockers in all receiving namespaces are ended before any unmount,
and a receiver's copy already gone when its turn comes is success.

### The signal

The further answer decides it:
`SIGTERM`,
a wait for the process to exit,
then `SIGKILL` if it has not.

- The wait sits in the signal stage,
  after every namespace's structural stage and before every unmount;
  every blocker gets `SIGTERM` through its pinned pidfd,
  the handler waits on their pidfds,
  and the survivors get `SIGKILL` through the same descriptors.
- The wait's length is a constant with no value;
  step 17's build sets it only after the owner decides one or M12 sizes it,
  and the forced arm does not ship before then (L9).
- During the wait the handler polls the blockers' pidfds,
  the wrapper's pidfd and its own progress file;
  if the wrapper's pidfd reports exit,
  it sends no `SIGKILL`,
  stops the act and files `caller-gone`.
- The wrapper's identity is checked immediately before the `SIGTERM` and again before the `SIGKILL`.
- A process that exits during the wait costs nothing:
  no `SIGKILL`,
  its exit is recorded as "exited after `SIGTERM`",
  and the act goes on when all blockers are gone.
- A blocker that ignores `SIGTERM` is sent `SIGKILL` when the wait's constant expires and the wrapper is still alive;
  the handler then waits for the kill to be observed,
  bounded by the same constant.

### The act's liveness

The act and the wrapper each need to know the other is alive,
and each has a wait that ends when the other dies (A4):

- The handler writes a progress file beside the scan file,
  root-owned `0644`,
  named `<guid>.removal-progress`,
  carrying the request nonce,
  the stage word and a counter.
  A timer thread advances the counter once per heartbeat period `T_h`,
  and the act also advances it at its start and at each stage,
  so a long preflight,
  wait or unmount does not read as a dead handler.
- The wrapper,
  while it waits,
  reads that file.
  Its own report deadline prints that the request still stands and does not end the act;
   until it has seen a counter it keeps its accepted receipt wait (the 30-second poll deadline,
  F24),
  so a handler that has not started is not read as dead;
  after the first counter it gives up only when the counter has not moved for the give-up window `T_g`,
  which is a stated multiple of `T_h` (four in this design,
  since a whole number of periods is what a reader can compare),
  counted from its first read after the request,
  and then prints that the handler is gone,
  that nothing further will be ended,
  and that the request stands as non-force work for a later trigger.
- The handler gives up on the wrapper by the wrapper's pidfd,
  checked before every signal and every unmount;
  a dead wrapper ends the act with no `SIGKILL` and files `caller-gone`.
- `T_h` and `T_g` are constants with no value until M12 sizes them,
  `T_g` being four `T_h` (L21);
  the stage word marks the act's progress and the counter marks its liveness,
  so a stalled handler and a slow one are told apart in the wrapper's report.
- [The sibling][cancelled] cites this rule under this heading for its timeout states,
  which otherwise assume a wrapper that times out and a request that stands.

### The blockers, and what the matcher reads

A blocker is a task that holds the copy's mount through any of:
its working directory,
an open descriptor,
its running executable (`/proc/<pid>/exe`),
a mapped file (`/proc/<pid>/map_files/*`),
its root (`/proc/<pid>/root`),
a thread's own working directory (`/proc/<pid>/task/<tid>/cwd`),
or a unix socket bound to a path inside the copy (the `sock_diag` answer's device and inode,
F56),
matched by the mount id or the bound file the kernel records (F36,
F43,
F50).
The matcher is new code over the handler's walk (L14).
A task under a child mount is not a blocker of this unmount:
the child mount is,
and it is refused.
A blocker in the protected set refuses the whole removal at the preflight.
A blind spot refuses at the preflight;
a task that has gone is not blind.

When the unmount inside a holder's namespace is refused `EBUSY` and the matcher found no blocker,
the report says the reads were empty,
that a reference the design cannot see remains,
and refuses `holder-unseen` naming the copy,
the namespace and the errno;
it never retries blind.
The host's own staging unmount is not this word:
when it stops short,
the handler files the staging record's fourth outcome word instead (`doc/decision/cli-git-zfs-staging-under-cache.md`,
"The answer a stopped unmount files,
answered 2026-11-23").

### What is still refused

The shared copy and the shared parent are not refused;
they are the case the recommended ways handle.
What the design still refuses:

- a namespace it cannot enter;
- the handler's own namespace,
  and any namespace containing pid 1,
  whose clone mount is the host's and whose unmount is the staging turn's;
  the act there ends path holders only;
- a namespace whose root mount is the copy (F48);
- a mount under the copy;
- a second mount of the dataset in the same namespace;
- a holder the preflight proves cannot be ended,
  and a blind spot;
- a parent mount that is not the checkout's own;
- for way 8,
  no recorded handle,
  or a kept receiver with a mount at the slot outside the act's namespaces (either way way 8 is not used and the act falls to way 7,
  which is not a refusal);
- for way 7,
  a read-only parent,
  no room for the directory,
  or a change the kernel refuses in the structural stage;
- an `EBUSY` at the unmount inside a holder's namespace after the empty search;
  the host's staging unmount that stops short is not a refusal here,
  and its answer is the staging record's fourth outcome word.

### What the service and other namespaces see

In the repaired state the parent has no master and no peer that is the host,
so nothing flowed from the host to the service under it,
and way 7 and way 3 make the same change:
the parent stops propagating,
and the service loses nothing it was receiving (F58,
L22).
With a shared checkout the two differ:
way 3 cuts the host's later mounts **and unmounts** under the parent off,
so other slots' clones copied into the service stay mounted after the host removes them;
way 7 keeps reception through the parent's master,
or through its next peer when it has peers,
while the service stops forwarding its own later mounts to namespaces made from it (F49).
Way 8 changes nothing,
and the receivers lose their copies or killed their blockers first.
The host's mount is untouched and no table shows the host's slave list losing an entry.
The directory of way 7 is visible to the host,
`git status` and snapshots while it lives;
a kill leaves it.
The peer namespace of a peer parent stops receiving the unit's later mounts under the parent;
a nested receiver of a receiver is in the closure as "The preflight" computes it and is treated like any other (F55).
T0-6 and M11 record both host states,
and the chain from a unit started while the checkout was shared to the repair to the act is the guest's to show (L22).

### The undo, and what a kill leaves

Way 8 changes nothing and leaves nothing to undo.
Way 7's slave change keeps the master when there is none to lose,
keeps it when the mount has no peers,
and makes the mount a slave of its next peer when it has peers;
a later `MS_SHARED` gives a new group with that master,
so it is nearly reversible;
the original group number is not restored (L15).
Way 3's and way 1's private change cannot be undone.
A kill after the structural stage leaves the parent changed and the directory in the checkout,
and no copy moved;
a kill after the unmount stage's move leaves the copy at the destination in that namespace;
the next run reads the state,
reports it,
moves the copy back when its recorded path still carries the parent and no mount,
and removes a matching leftover directory (L19).

### The checks that exist only for a hostile account

The threat model is the project's:
the account is trusted,
and root and the account are not a security boundary here ([staging][staging],
lines 221 to 237).
These checks survive it as guards against a bug or a stale record:
the copy's and parent's mount-id checks,
the post-change tag reads,
the destination and handle descriptors,
the reserved-prefix `0700` directory,
the wrapper's pidfd and progress file,
the dataset GUID re-read,
and the socket and thread reads.
A check that the parent directory is owned by root and not writable by the account exists only against a hostile account and is not adopted;
every call after the checks is descriptor-named,
and the mount-id checks are what the protection rests on.

### The confirmation

What the project is shown after the measurement,
in the review's model.

Asked:
a running service,
`watcher.service` with `PrivateMounts=yes`,
was started after the worktree `.claude/worktrees/one` was added and holds its own copy of that clone in its mount namespace;
a forced removal of `one` must end what blocks that copy and unmount it inside the service's namespace,
so the service keeps running and the removal finishes.

Concrete case:
the checkout `/home/user/Monochromatic` is its own mount and is private on the host,
the repaired state;
`watcher.service` has a build sandbox inside it (a slave namespace),
and a job in the service has its working directory in `one/target`.

- Option A (recommended):
  the policy of "Ranking and the recommended way":
  way 8 where the preflight proves the handle and no outside receiver with a mount at the slot;
  otherwise way 7,
  which in the repaired state is also way 3's change;
  one short-lived `0700` directory in the checkout,
  removed after the unmount;
  a namespace that is not a holder and receives from the parent is not refused,
  and the report names it and what it stops receiving (V8);
  refuse and name the holder where neither fits.
  A sandbox with no mount at the slot is not a kept receiver,
  so way 8's condition is met and nothing in the service is refused;
  a sandbox that holds a copy is one of the namespaces the act handles,
  and both copies go;
  the job is ended after `SIGTERM`,
  then `SIGKILL`.
  What it changes:
  nothing where way 8 fits;
  otherwise the checkout's mount in the service stops propagating,
  and one directory appears in the checkout.
  What the service loses:
  in the repaired state nothing it was receiving;
  in the shared state nothing with way 7,
  because it keeps its master or takes its next peer as one,
  and the host's unmounts under the checkout still reach it.
  What the person and the sandbox see:
  the removal's report names the copy and the namespace,
  and for a sandbox outside the act it names the sandbox and the later mounts it will not receive;
  `doctor` prints the same and removes nothing;
  the sandbox's own `findmnt` after the act shows the new mount missing,
  and the job's `wait` status records `SIGTERM` or `SIGKILL`.
- Option B:
  the record's own example for every case,
  the parent made private,
  the copy moved,
  the unmount.
  It changes one mount and writes the directory every time;
  in the repaired state it is Option A's fallback;
  with a shared checkout it also cuts the service off from the host's later unmounts under it.
- Option C:
  refuse and name the unit.
  Nothing in the service changes;
  the removal does not finish for the commonest holder,
  which the project declined once.

Ranking:
A > B > C,
because A does nothing where the handle fits and is otherwise B's change,
B finishes the removal the owner asked to finish while keeping the service running,
and C leaves the service untouched but returns the refusal the project chose against.

What confirming A accepts:
in the repaired state,
a service loses nothing it was receiving under the checkout,
since nothing flowed (the accepted design measured that loss,
F58);
for a service started while the checkout was shared and then repaired,
the same holds only where the repair left it no master,
which is inference from `fs/pnode.c` and rests on the unbuilt repair (L22);
a short-lived directory may appear in the checkout and a kill can leave it;
a kill can leave the checkout's mount in the service stopped from propagating and the copy at the directory.
None of this is shown yet:
it is a direction,
and the measurement below is what must show it.
The arm for each claim:
A's no change is T0-5a's stand-in-handle run and Z1-1;
the repaired state's loss is T0-11 and M1;
the shared state's difference is T0-6 and M11a with M11b;
the peer case is T0-4 and M2;
the sandbox's lost later mount is T0-12;
the job's end is M1.

What the guest must show first,
in this order:
the repaired state first and the shared checkout as the positive control;
the chain from a unit started shared to the repair to the act;
T0-1 to T0-4,
with the slave-first and private-first halves;
T0-11's chain from a unit started shared to the repair;
T0-4's peer rows,
T0-5a's nested-in-nested receiver,
T0-5b's receiver without a copy and T0-5c's stacked mount,
and T0-12's sandbox without a copy;
T0-6's shared-state difference;
the structural-stage refusal in a middle namespace and the move after the signals,
with T0-7's old-order control;
Z1-1's handle on the real filesystem after an ancestor's rename;
T0-8's socket in another network namespace;
a hardened unit,
the kill arms and a leftover directory's cleanup;
the wrapper's liveness arm;
Tier 0 on the `6.8` guest as informative and on a `7.3` guest when one exists,
Tier 1 and Tier 2 when the route is built;
every record carries `uname -r`.

## The act's mechanism

The preflight,
the signal order and the ways are the reach's;
this section states what holds for every act.

- The handler never signals a task in its own namespace except a path holder whose working directory,
  open file,
  executable,
  mapped file,
  root or bound socket is in the clone.
- The handler refuses when any protected task is a holder,
  and a namespace's tasks and threads are all enumerated.
- The number of tasks and namespaces one act reaches is bounded by the scan's own bounds (F23),
  and a scan that stopped at its bound is not a clear answer.
- Every task is pinned and proved before it is signalled,
  and a task whose proof fails is a blind spot;
  at the preflight a blind spot refuses the removal.
- The wrapper's pidfd is checked immediately before every signal and every unmount,
  and again before the structural stage's mount changes,
  and it is in the poll set the act waits on.
- The act's waits are the pidfds' exit (`POLLIN`),
  the wait between `SIGTERM` and `SIGKILL` (the constant L9 names without a value),
  and,
  for an unmount,
  the kernel call's return.
- The wrapper and the handler each know the other lives through the act's liveness rule:
  the handler's progress file,
  advanced by a timer thread once per heartbeat period `T_h` and at each stage,
  and the wrapper's pidfd.
  A dead wrapper means no `SIGKILL` and `caller-gone`;
  a heartbeat that stops means the wrapper gives up with the handler-gone report and the request stands as non-force work.
  `T_h` and `T_g` are constants with no value until M12 sizes them (L21).

The logging,
through the wrapper's existing notice stream on standard error and the handler's journal;
the journal lines are the record of who was ended,
by the records record:

- one line per blocker and its end:
  the dataset GUID and request nonce,
  the namespace identity,
  the mount id,
  the task's number and birth identity,
  its session,
  the read that found it,
  the signals sent,
  and whether it exited after `SIGTERM` or was killed;
- one line per propagation change:
  the namespace,
  the mount id,
  the old tags and the new,
  whether the change was private or slave,
  and whether the removal finished or failed;
- one line per destination directory:
  its name,
  the namespace,
  and whether it was removed or left;
- one line per holder not ended:
  the same fields plus the failing proof;
- one line per refused namespace,
  and one for a refusal in the middle of the structural stage,
  with what was changed and moved back;
- one closing line:
  how many blockers were ended,
  how many were unproven,
  whether the following scan was clear,
  and how many namespaces' propagation changed.

The wrapper prints the same facts in its report.
It does not print them for a holder of the protected set;
that case has its own refusal text.

## The refusal, and what it leaves

The forced arm's refusals are these,
and the receipt's failed step names which:

- `caller-protected-holds`:
  a holder task is in the protected set,
  at the preflight or a later scan;
  the same word is used without `--force`,
  where the sibling cancels the removal and reattaches the clone.
- `holder-unendable`:
  a holder the act could not end,
  or a namespace the reach refused for a reason this design names (cannot enter,
  root mount,
  child mount,
  second mount,
  a parent that is not the checkout's,
  no destination,
  no room).
- `holder-unseen`:
  an `EBUSY` at the unmount inside a holder's namespace after the matcher found no holder,
  meaning a reference this design cannot see;
  the host's own staging unmount that stops short files the staging record's fourth outcome word instead.
- `holder-blind`:
  a read the preflight could not make (`EACCES` or `EPERM`);
  it refuses before any act.
- `holder-structural`:
  a change the kernel refused in the middle of the structural stage;
  nothing was ended,
  no copy was moved,
  the earlier namespaces' changes are reported and the directory is removed when safe.

A dead wrapper is `caller-gone` only.
A refusal in the middle of the structural stage is its own state:
nothing was ended;
the namespaces before it were changed,
a parent made a slave and a directory made,
and no copy was moved;
  the directory is removed when it is safe,
  the parent stays a slave and is reported,
  and nothing re-shares it;
the report names the change and the directory,
and no copy was moved (A9).
The first text,
a working directory inside the clone,
with the variables filled and every path quoted as a shell word:

```text
the clone at <slot> is held by the removal's own protected set, and a removal never ends it.
process <pid> (<command>) holds it in its working directory, in session <session>, and that process is the caller, or is in the caller's session or its ancestor chain.
Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were, and the folder is still mounted.
Leave the folder (end that job, or run the command from another shell) and run the removal again.
```

The second,
a mount namespace holding a copy:

```text
the clone at <slot> is held by the removal's own protected set, and a removal never ends it.
mount namespace <namespace> keeps a copy at <mount point> (mount <id>), and process <pid> (<command>) in session <session> is the caller, or is in the caller's session or its ancestor chain.
Leaving the folder does not release a namespace's copy: end that session or its unit, or run the removal from a session that never entered the clone.
Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were.
```

The third,
a forced holder that cannot be ended or a namespace the reach refused:

```text
the removal of <slot> was asked to end the holders and could not.
<the holder, its namespace and the reason: a holder this removal could not end, a namespace it could not enter, a mount under the clone, a second mount of the dataset, a namespace whose root is the clone, a parent that is not the checkout's, a change the kernel refused in the structural stage, no destination or no room, or an unmount that refused after an empty search>.
<what was ended before the refusal, which namespaces' propagation changed, and which directory was left>.
Nothing was destroyed after that point.
```

The structural stage,
the signal stage and the unmount inside a holder's namespace stop with the host's clone unstaged and file this design's words;
only the host's own staging turn after the act is the staging record's decided case,
and its fourth outcome word is plan step 9's to propose (`doc/decision/cli-git-zfs-staging-under-cache.md`,
"The answer a stopped unmount files,
answered 2026-11-23").
The journal lines listed under "The act's mechanism" are the record of who was ended,
by the records record;
the removal's own record carries nothing about it.
A holder first seen after the first snapshot delete is met at the next scan,
and the refusal then reports what was already destroyed;
that state is [the sibling][cancelled]'s.
This design states only the forced arm's refusals and writes no intent.
The sibling at `3f2475e430` owns what a removal leaves when it does not finish,
forced or not,
names this design's words,
and has no question left;
its "The act's liveness" citation is this design's "The act's liveness",
and its structural-stage refusal takes this design's third state.
It needs three things from this design,
which this design gives:
the writer's pid,
start time and recorded parent in the force property,
its refusal words,
and the fact that this design never writes the intent.

## The time between the scan and the ending

A forced request must not act after the command that made it is gone,
and the further answer decides how:
the handler ends processes only while the very command that asked is still running,
checked by its process identity before every signal and every unmount.
The checks are:

- the force word is honoured only with a standing `unmount` intent whose nonce,
  the force property's and the caller property's all agree;
- the caller property's pid and start time are proved,
  and its recorded parent's pid and start time are proved,
  before the chain is walked;
- the handler holds a pidfd on the wrapper and checks it immediately before every signal and every unmount,
  and in the act's poll set;
- the handler does not read the account-owned removal claim;
  the staging design records that no channel from the claim to the handler exists and that the project chose not to add one ([stagingdesign][stagingdesign],
  "The unmount call from staging").

The wrapper and the handler each have a wait that ends when the other dies,
under "The act's liveness":
the handler's progress file,
advanced by a timer thread once per heartbeat period `T_h` and at each stage,
tells the wrapper the act is running;
the wrapper gives up with the handler-gone report when the counter has not moved for the give-up window `T_g` (four `T_h`),
and the request then stands as non-force work for a later trigger;
the handler gives up on the wrapper by its pidfd,
sends no `SIGKILL` and files `caller-gone`.
`T_h` and `T_g` are constants with no value until M12 sizes them (L21).

A later trigger that finds a forced request whose wrapper is gone declines the force word,
files `caller-gone`,
and does only the non-force work the intent asks.
A wrapper that is alive but whose recorded parent has changed or died has its force word declined the same way,
and prints that nothing was ended and that the removal should be run again as a child of the process that wants it.

A holder that attaches after the scan is met by the next scan,
and every such scan under the force word acts on the holders it sees;
a holder that attaches after the last scan shows up as `dataset is busy` at the head destroy,
the accepted residual.
A supervised blocker that is ended may be restarted by its supervisor into a fresh copy while the host still has it mounted;
the next scan meets it,
bounded by the removal deadline,
and the report says the holder returned.

A holder that exits and whose number is reused:
nothing is signalled by number,
the pidfd is opened while the number names the proven process,
and the pin's two start-time reads close the interval (F9,
F10).
A mount namespace or mount id reused after its object died is not enough either:
the link,
the row and the object are read again beside the number (L7).

A holder the scan cannot see:
a path holder in another account,
a socket or a thread whose read the handler could not make,
is a named blind spot and refuses the removal at the preflight;
a taskless pinned namespace is reached through `listns` (F14);
a task that survives the act's wait is reported,
not chased.

## Where the records differ, and how this design reads them

- The scan before the move.
  The accepted design's removal-ready scan comes after the move and the unmount;
  this design adds a scan under the same `unmount` intent before anything moves,
  so the preflight and the first act come before a snapshot is lost.
  V4 says why it runs there,
  and the sibling's point 5 is that state.

- The scan's strict clear.
  An incomplete read is reported and never read as clear;
  a holder the scan sees stops the destruction;
  a scan file that never arrived refuses at the step that needed it (V2).

- The receipt's contract.
  The sibling adds a sixteenth key,
  `bind`;
  this design adds no key and carries its refusals in bounded failed-step words (F24).

- The failure path.
  The sibling at `3f2475e430` owns what a removal leaves when it does not finish,
  names this design's words,
  has no question left,
  and cites this design's "The act's liveness" for its timeout states.
  It takes this design's structural-stage third state (A9) and the destination directory's leftover (V7),
  which its `3f2475e430` text names.
  This design states only the forced arm's refusals and writes no intent.

- The records.
  The records record decides S8,
  "until that clone's next removal",
  and S11,
  "the journal alone";
  this design asks the plan design for no field and names the journal lines as the record.

- The protected-holder refusal.
  Without `--force` a holder in the protected set refuses and the sibling cancels and reattaches;
  with `--force` the same refusal is the forced arm's.

- The claim.
  The sibling says the handler never reads the account's claim;
  this design's liveness proof is the wrapper process,
  not the claim.

- The wait,
  the liveness and the destination.
  The wait and liveness constants,
  the progress file and the directory's reserved prefix are this design's;
  the record's correction leaves their scope to it.

## Choices this design marks for the project's veto

Each entry is a writer's choice the design adopts so it is whole.
A veto takes the other option and the sentences that say otherwise.

### V1. Where the scan's finding is carried

Adopted:
the scan's own text in a second root-owned file beside the receipt.
The alternatives are the text inside the receipt,
an on-demand privileged run,
and outcome only;
the ranking stays beside the receipt > inside > on demand > outcome only.

### V2. What a receipt without its scan file means

Adopted:
a refusal of the removal at the step that needed the scan.
A veto makes it an `unknown` that does not refuse,
or a retry;
the ranking stays refuse > retry > unknown.

### V3. How the caller's identity and the force word reach the handler

Adopted:
two clone-local properties bound to the intent's nonce,
the caller one carrying the wrapper's pid,
start time,
recorded parent and boot.
A veto takes the longer intent grammar or a single request record;
the ranking stays two properties > longer intent > request record.

### V4. When the forced arm acts

Adopted:
at every scan under the force word,
beginning with the pre-move scan and its preflight.
A veto takes step 16's scans only,
or a single scan;
the ranking stays every scan > step 16's only > once.

### V5. How the copy is got out of a namespace whose mounts are shared

Options,
each in "The ways out of a namespace whose mounts are shared":
(1) copy and parent private;
(2) copy private,
unmount in place;
(3) parent private,
move,
unmount;
(4) plain unmount in place;
(5) copy private,
lazy detach;
(6) whole namespace private;
(7) parent slave,
move,
unmount;
(8) in-place unmount by a recorded handle.

The ranking is 8 > 7 > 3 > 1 > 4 > 2 > 5 > 6;
the reasons are in "Ranking and the recommended way".
The recommended policy is way 8 where the preflight proves the handle and no outside receiver with a mount at the slot,
otherwise way 7 (the parent stops propagating);
a namespace that is not a holder and receives from the parent is not refused,
and the report names it and what it stops receiving (V8).
In the repaired state the parent has no master and no peer that is the host,
so ways 3 and 7 make the same change and the service loses nothing it was receiving;
the difference exists only where the checkout is shared,
which the route repairs before every act (A1,
L22).
The record's own example,
the parent made private (way 3),
stays an option with the cost of the shared state.
The project confirms the policy after the Tier 0 and Tier 1 arms and the Tier 2 arms the guest runs;
this entry is a recommendation,
not an adoption.

### V6. Whether a copy whose unmount is refused after the structural stage is moved back

Adopted:
move the copy back when its recorded path is on the same parent and carries no mount,
report both paths,
the change and the directory,
and remove the directory when it is safe.
A veto leaves the copy at the destination and reports it;
the ranking stays move back > leave and report > detach.

### V7. Whether the report, `doctor` and the next run name a changed propagation and a leftover directory

Adopted:
the receipt and the report name every propagation change and every destination directory,
`doctor` prints them and removes nothing,
and the next removal's handler sweep and the cancel command remove a directory that carries the reserved prefix `.git-worktree-clone-removal-` and the clone's GUID,
is empty,
root-owned,
mode `0700`,
and sits directly under the parent mount's root.
A veto keeps the report only and leaves removal to the person;
the own-record option has no reader or pruner.
Ranking:
report,
`doctor` and the next run > report only > own record.

### V8. What the act does about a namespace that is not a holder and receives from the parent

Options:
proceed and report the namespace and what it stops receiving (adopted);
refuse the removal and name it.

Concrete example:
`watcher.service` holds a copy and has a sandbox inside it with no copy of its own;
the sandbox receives from the checkout's mount in the service.
Under the adopted option the act makes the parent stop propagating,
the copy is unmounted,
and the report names the sandbox and the later mounts under the checkout it will not receive.
Under the refusal the removal stops and names the sandbox,
and the copy stays.

Pros of the adopted option:
the removal finishes for the commonest holder,
and the loss is a later mount in a sandbox that never held the clone.
Cons:
a namespace the act does not handle changes what it receives,
and the person must read the report to know.
Pros of the refusal:
nothing changes in that namespace.
Cons:
a unit with any nested sandbox cannot be forced,
which returns the refusal the project declined.
Ranking:
proceed and report > refuse,
because the removal is what the owner asked for and the loss is named;
the refusal is the veto's alternative,
with the sandbox case as its example.

## The record of who a forced removal ended, and the questions left

The records record,
"Whether a forced removal records who it ended,
beyond the journal",
decided "A:
the journal alone (Recommended)":
"The handler's journal lines are the record."
The cost it states:
the journal may rotate away,
and the removal's own record does not show it ([records][records]).
The options not chosen,
once:
the handler's receipt naming the processes,
a count in the plan record,
and a record of its own.
This design therefore asks the plan design for no field,
and the removal's own record carries nothing about who was ended;
the journal lines the act's mechanism lists are the record.

No question is left:
S1 to S13 are decided by the two records,
and S8 (how long a cancellation's kept plan lives) is the records record's "How long the plan a cancellation keeps stays",
decided as "until that clone's next removal".
The forced arm's remaining choices are the veto entries below,
and the shared-namespace policy the project confirms after the guest.

## Which build step builds which part

- Step 9's holder-scan round has built the scan,
  the text,
  the parser and the mode;
  this design asks it to record the representative task's start time,
  and builds no scan.
- Step 10 or 14 also records the slot underlay's file handle before the clone is mounted,
  which way 8 needs (F53,
  D7);
  clones that predate it fall to way 7.
- Step 16 owns the scans' call sites and the deferred-destroy checks ([deferred][deferred]);
  the scan file's publisher and reader are built with the first consumer.
- Step 17 builds this design:
  the properties,
  the chain and parent checks,
  the pidfd,
  the service-manager and session checks,
  the refusals and words,
  the pre-move scan,
  the preflight (parent,
  structure,
  receiver closure,
  handle,
  room,
  endability,
  sockets,
  blind spots),
  the structural stage (way 8's open,
  or way 7's slave change and directory),
  the signal stage,
  the unmount stage (way 8's unmount,
  or way 7's move and unmount),
  the wait and liveness constants,
  the progress file,
  the directory removal,
  the logging and the tests.
  The constants are set only after the owner decides them or M12 sizes them;
  until then step 17 cannot ship the forced arm (L9,
  L21).
- Step 18 builds `doctor`:
  the last scan file,
  the live scan mode,
  the holders,
  the protected set,
  the propagation changes and a leftover directory matching V7's prefix and checks.
- Step 19 builds the handler's unit with no mount-namespace option (A8,
  R8).
- No step builds a second scan,
  a fallback kill-all,
  a lazy detach,
  a forced unmount of the host's mount,
  or a change to another unit's mount settings.

## Tests a build must have

Each test names the race or interleaving it forces,
how it forces it without timing luck,
and whether it needs a guest.

1.  The proof refuses a task that exits before its pin;
    device-free for the proof,
    guest for the host `/proc`.
2.  A signal reaches the pinned process only,
    with `set_tid` reuse;
    device-free and guest.
3.  The protected-set refusal for a working directory and a namespace,
    with and without `--force`;
    guest.
4.  Another session's holder ends while the caller lives;
    guest.
5.  The ancestor half is spared;
    device-free with fixture `/proc`,
    guest for the real removal.
6.  A holder that appears between the scan and the act is met by the next one,
    through the injected hook;
    guest.
7.  A receipt without its scan file refuses;
    a stale or absent caller property,
    a dead wrapper,
    and a changed recorded parent decline the force and print the declined text;
    device-free.
8.  The wrapper ends between two passes and during the wait;
    the handler's progress stops:
    no `SIGKILL`,
    `caller-gone`,
    and the wrapper's handler-gone report;
    guest.
9.  The preflight refuses before any act for each cause reading can know,
    including the receiver closure's outside member with a mount at the slot,
    a nested-in-nested receiver,
    a receiver with no mount at the slot (not refused),
    a blind read against a gone read,
    and no room;
    guest.
10. The service-manager checks:
    the wrapper's parent,
    the session leader's parent,
    a look-alike cgroup,
    an orphaned session,
    a script-main unit (refused),
    and the allowed shapes;
    the console-login remedy runs under `setsid --wait` and is allowed;
    device-free and guest.
11. The repaired state is reached by way 7:
    the unit's table before and after,
    the parent's `master:` absent (no peers) and the former group (peers),
    the copy moved and unmounted,
    the directory removed,
    the head destroy done,
    the unit alive;
    guest,
    production kernel.
12. Way 8 in place by the handle,
    and after a rename of an ancestor;
    guest with the route.
13. A unit started while the checkout was shared,
    then repaired,
    then the act;
    guest.
14. The peer parent:
    the post-check's two cases,
    the peer namespace stops receiving the unit's later mounts;
    guest.
15. The move is after the signals:
    a blocker writes by absolute path during the `SIGTERM` wait and its write lands in the mounted copy,
    not the checkout;
    guest.
16. A socket bound by absolute path in a unit with `PrivateNetwork=yes` is found through `sock_diag`;
    guest.
17. The structural-stage refusal in a middle namespace:
    nothing ended,
    the earlier namespaces' changes reported and moved back;
    guest.
18. The signal order:
    a `SIGTERM` handler exits and is not killed,
    an ignorer is killed at the wait constant,
    the wrapper's death sends no `SIGKILL`;
    guest.
19. The host states:
    repaired first,
    shared as the control,
    with a host mount and a host unmount of another slot after the act;
    guest.
20. A hardened unit:
    way 7 refused,
    way 8 reached;
    the kill arms and a leftover directory's cleanup;
    guest.
21. `doctor` names holders,
    the protected set,
    the propagation changes and a matching leftover directory;
    device-free.

## Measurements owed

The fixture is [the measurement record][measurements],
whose item 16 says no CachyOS guest exists.
The arms are grouped by what each needs to run,
because that decides what can run now.
A builder scripts one arm per row;
each row gives the state,
the one action,
the expected result,
a bound with its unit (or the measurement that gives it),
the constants it needs,
how the removal or its stand-in is invoked,
and the statement it decides.
Every arm records `uname -r`,
`zfs version`,
the tables of the unit,
its peer and slave namespaces,
another unit and the host,
and the `shared:`,
`master:`,
`propagate_from:` and `unbindable` tags of each row.
Tables are compared by the relation (who is a peer or slave of whom),
never by a group's number,
which changes between runs (Z9).
Every arm's invocation:
Tier 0 and Tier 1 arms are a small Python program extending the fixture's `guest-probe.py`,
using `ctypes` for `mount`,
`umount2`,
`move_mount`,
`name_to_handle_at`,
`open_by_handle_at`,
`statx` and the netlink `sock_diag` request,
and `subprocess` for `unshare`,
`mount`,
`umount` and `findmnt`;
each arm names the calls and arguments it makes;
every Tier 2 arm runs the wrapper as a child of the harness's shell,
with `setsid --wait` where the guest agent would otherwise put it in a service unit (Z1).
A Tier 0 arm's own `Bound:` overrides the 900 s harness timeout.
An arm the guest cannot show is recorded as unrun,
never passed (Z5).

### The tiers, the guest and the constants

- **Tier 0** needs only mounts and namespaces in a guest:
  `unshare`,
  `mount`,
  bind copies,
  `findmnt`,
  `/proc/self/mountinfo`,
  `setsid --wait`.
  It runs on the `6.8.0-147-generic` guest the fixture already has,
  where its results are informative:
  what it leaves unshown is the taskless namespace (no `listns`) and the production kernel's own mount code,
  and a `7.3` guest is what would show them (Z8).
- **Tier 1** needs a ZFS pool and datasets but none of the route's code:
  the handle route and the no-room states.
- **Tier 2** needs step 17's code,
  which does not exist yet:
  the wrapper,
  the handler,
  the add's handle record and the repair.
  No guest with the production kernel exists (the fixture's item 16 says the image registry offers no CachyOS guest);
  the arms that only a `7.3` guest decides are named,
  and a CachyOS qcow2 with `linux-cachyos-zfs` and a guest agent is what the project must build for them.
- **The constants**:
  the wait between `SIGTERM` and `SIGKILL` is `T_wait = 10 s` provisional;
  the heartbeat period `T_h = 1 s` and the give-up window `T_g = 4 s` (= 4 `T_h`) provisional;
  the harness timeout per arm is 900 s.
  M12 sizes `T_wait`,
  `T_h`,
  `T_g` and step 22's removal deadline;
  until then the arms run with these provisional values,
  recorded with each run.
  The provisional values are the harness's,
  not shipped.
- **How the removal is invoked** (Z1):
  the wrapper refuses `--force` when its parent or its session leader's parent is the service manager,
  so every Tier 2 arm invokes the wrapper as a child of the harness's shell,
  and where the guest agent puts a command in a service unit the harness starts it with `setsid --wait` from that shell so the wrapper's session leader is the shim and its parent the shell.
  M10 is the arm for the refused shapes.
- **Names the arms assert** (Z8/X8):
  the progress file `<guid>.removal-progress`;
  the reserved directory prefix `.git-worktree-clone-removal-`;
  the words `holder-unendable`,
  `holder-unseen`,
  `holder-blind`,
  `holder-structural`;
  the tags `shared:`,
  `master:`,
  `propagate_from:`,
  `unbindable`.

### Tier 0: mounts and namespaces only

These are a small Python program extending the fixture's `guest-probe.py`:
`ctypes` for `mount`,
`umount2`,
`move_mount`,
`name_to_handle_at`,
`open_by_handle_at`,
`statx` and the netlink `sock_diag` request,
and `subprocess` for `unshare`,
`mount`,
`umount` and `findmnt`.
Each arm's `Bound:` is its own (60 s or 120 s) and overrides the 900 s harness timeout,
because it runs no handler.
- **T0-1.
  A plain unmount reaches the parent's receivers.**
  State:
  a bind copy of a directory at a slot on a shared stand-in parent;
  a slave namespace made with `unshare --mount --propagation unchanged` from that parent after the copy is mounted,
  so it inherits its own copy;
  a host-side bind at another path under the same parent,
  which the host sees and the copy's unmount does not touch.
  One action:
  `umount2` the copy in the unit's namespace,
  via `ctypes` `umount2(path, 0)`.
  Expected:
  the slave's copy vanishes;
  the host-side bind and the host's stand-in mount stay.
  Bound:
  60 s.
  Constants:
  none.
  Decides:
  the receiver rule;
  ways 2 and 4 propagate.
  Tier 0;
  run on `6.8`,
  informative.

- **T0-2.
  A private parent before the unmount stops it.**
  State:
  T0-1's state,
  with `mount --make-private` on the parent (the `guest-probe.py` call `mount(NULL, parent, NULL, MS_PRIVATE, NULL)`).
  One action:
  `umount2` the copy.
  Expected:
  the slave's copy stays mounted.
  Bound:
  60 s.
  Decides:
  way 3's no-receiver effect.
  Tier 0.

- **T0-3.
  A slave parent before the unmount stops it.**
  State:
  T0-1's state with the parent made slave,
  in two variants:
  with a master (T0-1's host-side relation) and without one (a parent with no master).
  One action:
  `umount2` the copy.
  Expected:
  the slave's copy stays in both;
  the parent's row reads `master:<the old master>` with the master and no `master:` without it (a slave with no master reads like private).
  Bound:
  60 s.
  Decides:
  way 7's no-receiver effect,
  and which row each state produces.
  Tier 0.

- **T0-4.
  The peer case's rows.**
  State:
  a parent P,
  a peer P' of P,
  and a slave S whose master is P;
  P' is made before the copy,
  S after it,
  and the copy under P.
  One action:
  `mount --make-slave P`.
  Expected (derived from `fs/pnode.c` 93 to 124 and `fs/proc_namespace.c` 159 to 169,
  not yet run):
  P's row becomes `master:<the former group>` without `shared:`;
  P' keeps its `shared:` and its own `master:`;
  S's row still prints `master:<the former group>`,
  because P' shares the group;
  a mount made under P' reaches P and S,
  and one made under P reaches neither.
  Bound:
  60 s.
  Decides:
  A2's peer case,
  the post-check's two cases.
  Tier 0.

- **T0-5a.
  The nested-in-nested receiver.**
  State:
  a covered directory whose cover is the topmost mount at the slot;
  a receiver of a receiver (a namespace made from a namespace made from the holder) with its own copy;
  a stand-in handle taken with `name_to_handle_at` on the covered directory before the cover,
  from the guest's root filesystem (if it refuses a handle,
  the arm records unrun).
  One action:
  `open_by_handle_at` the handle,
  read the topmost mount at its dentry,
  and `umount2` by the descriptor.
  Expected:
  a closure script over the tables (T0-5's oracle) finds the nested receiver in the set,
  and the set of namespaces whose copy vanished equals it.
  Bound:
  60 s.
  Decides:
  X2's closure and the "mount at the slot" filter.
  Tier 0.

- **T0-5b.
  A receiver with no mount at the slot.**
  State:
  T0-5a's state,
  plus a receiver with no mount at the slot.
  One action:
  the same handle unmount.
  Expected:
  that receiver is not a kept receiver and does not refuse;
  it is not in the vanished set.
  Bound:
  60 s.
  Decides:
  the "only members with a mount at the slot" rule.
  Tier 0.

- **T0-5c.
  A stacked mount at the slot.**
  State:
  T0-5a's state,
  a stacked mount on the slot's dentry in a receiver.
  One action:
  the same handle unmount.
  Expected:
  the topmost mount is not the copy,
  so way 8 is not chosen and the act falls to way 7;
  the stacked mount is untouched.
  Bound:
  60 s.
  Decides:
  the topmost check and N2's one behaviour.
  Tier 0.

- **T0-5's oracle**:
  a Python closure over the tables:
  map every group number to the order it first appears,
  then compute the set from the seed and the members rule;
  compare it with the namespaces whose copy vanished.
  The normalisation is the comparison rule for every arm (Z9),
  and the members rule is the one "The preflight" states.

- **T0-6.
  The shared-state difference.**
  State:
  a shared stand-in parent,
  a slave namespace with its own copy,
  a second copy mounted on the host at a host path and propagated into the unit,
  and a host peer.
  One action:
  `mount --make-slave` the parent (way 7);
  then,
  separately,
  `mount --make-private` (way 3);
  make a host mount and a host unmount of the second copy in each.
  Expected:
  the way 7 unit keeps receiving the host's mount and unmount of the second copy (its master or next peer is a receiver);
  the way 3 unit loses both.
  Bound:
  120 s.
  Decides:
  A1's shared-state difference.
  Tier 0.

- **T0-7.
  The move sits after the signals.**
  State:
  a bind copy,
  a blocker in its namespace that traps `SIGTERM`,
  writes a file by absolute path into the slot and exits;
  `guest-probe.py` forks it.
  One action:
  `SIGTERM`,
  wait,
  then `mount --make-slave` and `move_mount`.
  Expected:
  the write lands in the mounted copy and the slot's folder stays empty when the move is due.
  Control:
  the same state with the move first;
  the write then lands in the slot's underlying folder on the checkout's filesystem,
  which the host sees,
  which is the hazard the order answers.
  Bound:
  `T_wait` 10 s.
  Decides:
  A5's exposure and its answer.
  Tier 0.

- **T0-8.
  A socket in another network namespace.**
  State:
  a namespace made with `unshare --net`,
  a unix socket bound by absolute path inside the copy.
  One action:
  a netlink `sock_diag` request with `UDIAG_SHOW_VFS` from `ctypes`,
  entered by `setns` on a task's `/proc/<pid>/ns/net`.
  Expected:
  the bound file's device and inode name the copy's filesystem;
  the owning task is found by the socket's inode in `/proc/<pid>/fd`.
  Bound:
  60 s.
  Decides:
  A7's socket read.
  Tier 0.

- **T0-9.
  Blind and gone reads.**
  State:
  a root-owned task,
  and an unprivileged reader (`setuid` to `nobody` in a forked child) reading its `/proc/<pid>/exe`;
  a task that exits during the read.
  One action:
  the reader's `statx` on `/proc/<pid>/exe`.
  Expected:
  the first is `EACCES` and is classified blind;
  the second is `ENOENT`/`ESRCH` and is gone;
  this tests the classifier,
  not what the root handler meets.
  Bound:
  60 s.
  Decides:
  A10's definition.
  Tier 0.

- **T0-10.
  The taskless namespace.**
  State:
  a namespace pinned by a descriptor with no task.
  One action:
  the `listns` syscall,
  number 470 on x86-64,
  from `ctypes`,
  then `setns` and a table read.
  Expected:
  it is found and entered;
  a guest whose `listns` refuses or is absent records the arm unrun,
  and the rest of the group shows everything else.
  Bound:
  60 s.
  Decides:
  the taskless route.
  Tier 0,
  but a guest that answers `listns` only.

- **T0-11.
  The chain:
  a unit started shared,
  then repaired.**
  State:
  a host stand-in parent shared,
  a slave namespace made from it (the unit's stand-in) with its own copy;
  then `mount --make-private` on the host stand-in,
  the repair.
  One action:
  read the unit's table after the repair,
  then make the parent slave (way 7) and read it again.
  Expected:
  after the repair the unit's copy has no master (the host stand-in had no peers) or keeps the next peer's master (it had peers);
  the way 7 change is the same as way 3's in the no-master case.
  Bound:
  120 s.
  Decides:
  A1's chain,
  L22.
  Tier 0.

- **T0-12.
  A sandbox that never held the clone,
  after way 7.**
  State:
  a unit-like namespace with a copy,
  a sandbox inside it with no copy,
  receiving from the parent;
  a host mount made under the parent after the act.
  One action:
  `mount --make-slave` the parent,
  unmount the copy,
  then make a host mount under the parent.
  Expected:
  the sandbox does not receive the later host mount;
  the report names it (V8).
  Bound:
  120 s.
  Decides:
  N6's claim (the sandbox's lost later mount).
  Tier 0.
### Tier 1: a ZFS pool, no route

- **Z1-1.
  The recorded handle on a ZFS dataset.**
  State:
  a ZFS dataset mount,
  a directory under it,
  a handle taken with `name_to_handle_at` before a clone or bind covers the directory;
  then the cover,
  then a rename of an ancestor.
  One action:
  `open_by_handle_at` on the dataset mount's descriptor (the handler runs as root;
  the capability and the model are F53).
  Expected:
  the descriptor names the covered directory in both cases,
  and `umount2("/proc/self/fd/<n>")` unmounts the topmost mount there.
  Bound:
  120 s.
  Decides:
  way 8's real-filesystem question.
  Tier 1.

- **Z1-2.
  No room and the clone-holding states.**
  State:
  a dataset at its quota,
  and a dataset holding a clone.
  One action:
  try the directory creation;
  read the free space and quota.
  Expected:
  the creation fails with the quota's error;
  the preflight's read names it.
  Bound:
  120 s.
  Decides:
  M8's no-room and clone-holding states at Tier 1.
  Tier 1.

### Tier 2: step 17's code

- **M1.
  The repaired state end to end.**
  State:
  the repaired state,
  a `PrivateMounts=yes` unit holding an own-root copy with no recorded handle (so the policy takes way 7),
  a job under `WorkingDirectory=` in the clone,
  a second slot's clone in the unit.
  One action:
  `git worktree remove --force <slot>`.
  Expected:
  the parent stops propagating,
  the copy is moved and unmounted,
  the directory is removed,
  the job's `wait` status is `SIGTERM` or `SIGKILL`,
  the head destroy 0,
  a host mount and a host unmount of the second slot after the act not reaching the unit.
  Bound:
  900 s harness timeout;
  the wait `T_wait`;
  the liveness `T_h`/`T_g`.
  Invocation:
  the wrapper as a child of the harness's shell (Z1).
  Decides:
  the repaired state's cost (A1),
  the directory,
  the order.
  Tier 2.

- **M2.
  The peer parent in the full act.**
  State:
  M1's unit with a peer namespace and a slave namespace inside it;
  all holders,
  handled in namespace-inode order,
  with the table recorded after each namespace's stage and at the end.
  One action:
  one way 7 removal.
  Expected:
  after the parent's stage,
  P's row reads `master:<the former group>` with no `shared:`,
  P' keeps its `shared:` and its own `master:`,
  and S's row still reads `master:<the former group>`;
  a mount under P' reaches P and S,
  one under P reaches neither;
  at the end,
  in the repaired state,
  no `master:` anywhere;
  each holder's copy gone by its own act.
  Bound:
  900 s.
  Decides:
  A2 in the act,
  the order's effect.
  Tier 2;
  its kernel facts are T0-4.

- **M3a.
  Way 8 in the full act.**
  State:
  M1's unit with a recorded handle.
  One action:
  one `--force` removal.
  Expected:
  no change,
  no move,
  no directory,
  the copy unmounted in place,
  the head destroy 0.
  Bound:
  900 s.
  Decides:
  way 8's selection by the preflight and the act.
  Tier 2;
  the handle itself is Z1-1.
- **M3b.
  Way 8 after an ancestor's rename.**
  State:
  M3a's unit,
  the slot's ancestor renamed before the act.
  One action:
  one `--force` removal.
  Expected:
  the handle still names the covered directory,
  the unmount succeeds,
  the head destroy 0.
  Bound:
  900 s.
  Decides:
  the handle's rename resistance in the act.
  Tier 2;
  the fact is Z1-1.

- **M5.
  The chain from a shared start.**
  State:
  a unit started while the checkout was shared,
  then the checkout repaired private (the handler's or `mount --make-private` as the stand-in),
  then the act.
  One action:
  one way 7 removal.
  Expected:
  the parent has no master after the repair and the service loses nothing it was receiving;
  the shared control's way 7 keeps a master.
  Bound:
  900 s.
  Decides:
  A1's chain,
  L22.
   Tier 2;
  the propagation facts are T0-11 and T0-3.

- **M7.
  The writing blocker in the full act.**
  State:
  M1's unit with a blocker that writes by absolute path during the `SIGTERM` wait.
  One action:
  one way 7 removal.
  Expected:
  the write lands in the copy;
  the slot is empty when the move is due.
  Bound:
  900 s;
  `T_wait`.
  Decides:
  A5 in the act.
  Tier 2;
  the fact is T0-7.

- **M9a.
  The wrapper killed during the handler's wait.**
  State:
  M1's unit,
  the wrapper killed during the wait.
  One action:
  one `--force` removal.
  Expected:
  the handler sends no `SIGKILL`,
  files `caller-gone`;
  the copy and the parent are as the stages left them.
  Bound:
  900 s;
  `T_h`/`T_g`.
  Decides:
  A4's handler side.
  Tier 2.
- **M9b.
  The handler killed during the wrapper's wait.**
  State:
  M1's unit,
  the handler killed while the wrapper waits.
  One action:
  one `--force` removal.
  Expected:
  the counter stops,
  the wrapper prints the handler-gone report within `T_g`,
  and the request stands as non-force work for the next trigger.
  Bound:
  900 s;
  `T_h`/`T_g`.
  Decides:
  A4's wrapper side.
  Tier 2.
- **M9c.
  The structural-stage refusal.**
  State:
  M1's unit,
  a kernel refusal in the middle of the structural stage,
  injected by a seam as the claim controls do.
  One action:
  one `--force` removal.
  Expected:
  nothing ended;
  the parent is a slave and reported,
  the directory is removed when safe,
  no copy was moved;
  the receipt's `holder-structural`.
  Bound:
  900 s.
  Decides:
  A9,
  X3.
  Tier 2.

- **M10a.
  A user service unit.**
  State:
  a `systemd-run --user --pipe --wait` unit holding a copy.
  One action:
  one `--force` removal.
  Expected:
  exit 2,
  the text names the manager,
  no property written.
  Bound:
  300 s.
  Decides:
  the parent check.
  Tier 2.
- **M10b.
  A system service unit.**
  State:
  a system unit whose parent is pid 1.
  One action:
  one `--force` removal.
  Expected:
  exit 2,
  the text names the manager.
  Bound:
  300 s.
  Decides:
  the system-manager recognition.
  Tier 2.
- **M10c.
  A script-main unit.**
  State:
  a unit whose main process is a script that runs the removal.
  One action:
  one `--force` removal.
  Expected:
  exit 2,
  the text names the session leader's parent.
  Bound:
  300 s.
  Decides:
  the script answer.
  Tier 2.
- **M10d.
  A console login and its remedy.**
  State:
  a console login with a getty.
  One action:
  one `--force` removal typed there,
  then the same under `setsid --wait`.
  Expected:
  the first exits 2 with the console sentence;
  the remedy is allowed and its session leader is the shim.
  Bound:
  300 s.
  Decides:
  A13.
  Tier 2.
- **M10e.
  An orphaned session.**
  State:
  a session whose leader's parent cannot be read.
  One action:
  one `--force` removal.
  Expected:
  exit 2 with the orphan sentence.
  Bound:
  300 s.
  Decides:
  the unreadable-leader rule.
  Tier 2.
- **M10f.
  Pi's and Claude Code's shapes.**
  State:
  the `chain.ts` shapes (F1,
  F4).
  One action:
  one `--force` removal from each.
  Expected:
  allowed;
  the protected set names the harness and the shell.
  Bound:
  300 s.
  Decides:
  the allowed shapes.
  Tier 2.

- **M11a.
  The shared control,
  way 7.**
  State:
  the shared checkout with a unit holding a copy.
  One action:
  one way 7 removal.
  Expected:
  the host's mounts and unmounts under the checkout still reach the unit;
  no other namespace changes.
  Bound:
  900 s.
  Decides:
  A1's shared state with way 7.
  Tier 2;
  the fact is T0-6.
- **M11b.
  The shared control,
  way 3 by a test-only switch.**
  State:
  M11a's state,
  a test-only switch that selects way 3.
  One action:
  one removal.
  Expected:
  the unit loses the host's mounts and unmounts under the checkout.
  Bound:
  900 s.
  Decides:
  A1's shared difference;
  way 3 is not the policy.
  Tier 2.

- **M12.
  The timings and the constants.**
  State:
  the scan file's publish to the wrapper's read;
  the wait for one and for many blockers;
  the liveness detection;
  the slave change,
  the directory,
  the move and the unmount.
  One action:
  time each,
  interleaved with a control arm in the same round (rule `TVC`).
  Expected:
  the numbers that size `T_wait`,
  `T_h`,
  `T_g` and step 22's removal deadline.
  Bound:
  900 s;
  each timing's own control.
  Decides:
  the constants.
  Tier 2.

- **M13.
  A hardened unit.**
  State:
  a `ProtectHome=read-only` unit holding a copy.
  One action:
  one `--force` removal with way 7 selected,
  and one with way 8.
  Expected:
  way 7 refuses at the preflight (no room,
  read-only parent);
  way 8 is reached when the handle exists.
  Bound:
  900 s.
  Decides:
  L20.
  Tier 2.

- **M14a.
  A kill between the slave change and the directory.**
  State:
  M1's unit,
  the handler killed there.
  One action:
  one `--force` removal,
  then the next removal.
  Expected:
  the parent stays a slave and is reported;
  no directory was made;
  the next removal proceeds and reports the parent.
  Bound:
  900 s.
  Decides:
  V6's state,
  X3.
  Tier 2.
- **M14b.
  A kill between the directory and the move.**
  State:
  M1's unit,
  the handler killed there.
  One action:
  one `--force` removal,
  then the next removal and `doctor`.
  Expected:
  the parent stays a slave,
  the directory is left;
  the next removal's sweep removes it;
  `doctor` prints it.
  Bound:
  900 s.
  Decides:
  V7's cleanup,
  X7.
  Tier 2.
- **M14c.
  A kill after the move before the unmount.**
  State:
  M1's unit,
  the handler killed there.
  One action:
  one `--force` removal,
  then the next removal.
  Expected:
  the copy is at the destination,
  the parent a slave,
  the directory left;
  the next removal reports both and removes the directory when safe.
  Bound:
  900 s.
  Decides:
  V6,
  V7.
  Tier 2.

## Limitations

- Every claim about the dataset releasing,
  the propagation rules,
  the private and slave changes,
  the move,
  the unmount and the handle route is an inference from source and VM items 3,
  4 and 15 (L1,
  L12).
- The caller's protected set is a number inside one pid namespace (L4).
- The caller property is an assertion the handler ties to a live wrapper and its recorded parent,
  not to a sender (A4).
- The wait `T_wait` and the liveness constants `T_h` and `T_g` have no values yet (L9,
  L21);
  the forced arm cannot ship until the owner decides them or M12 sizes them.
- The repaired state's chain,
  the peer parent's master and the receivers' closure are source readings and inferences;
  T0-4,
  T0-5 and M5 are what show them (L22).
- `listns`,
  the handle route and the propagation changes are unrun (L12);
  M1 to M9 owe them,
  on the production kernel (L18).
- The matcher over `exe`,
  `map_files`,
  `root`,
  threads and unix sockets is new code (L14),
  and the socket read is a `sock_diag` read (F56) no run shows.
- The service-manager checks and local polkit rules are unverified on this host (L5,
  L11).
- The journal alone is the record of who was ended,
  by the records record;
  it rotates.

## What this design does not cover

- What a cancelled,
  interrupted,
  timed-out or killed removal leaves,
  and the kept `cancel-worktree-removal` command:
  the sibling owns it (D2),
  including the liveness rule it cites and the structural-stage state it takes.
- Where the removal's persisted destruction plan lives and which fields it has;
  the records record declined a field for who was ended,
  so this design asks for none.
- The deferred-destroy checks themselves and step 16's call sites (D3).
- A second scan,
  any force or lazy unmount of a staged mount,
  any fallback that ends every task of a namespace,
  any unmount of the host's clone mount by the forced arm,
  and any change to a unit's own mount settings.
- A namespace or a state the reach refuses beyond the refusal and the report:
  no recovery is designed for a copy left at a destination when moving it back also fails,
  and no restoration of a private mount's original group is designed.
- The handler's installation,
  its unit hardening and ZED's timeouts;
  the unit's mount-namespace constraint is step 19's,
  and whether a kill-and-wait inside a zedlet child is bounded by ZED is not answered here.
- The add-time Git lock,
  the add binding record,
  the dirty-main restore,
  the origin-cleanup record,
  and the private propagation after a replacement mount (D6);
  the slot's handle record is D7.
- Container removals,
  and a caller named across a service boundary (L10).
- Any number:
  no grace,
  retry count,
  timeout or ceiling ships from this design (L9,
  L21).

## Dependencies on sibling designs

Each dependency says what this design assumes,
and where a sibling disagrees,
that it does.

- D1.
  Step 9's unmount and reattach build;
  the staged unmount round is in flight,
  and nothing of the route is released.

- D2.
  [The sibling cancelled-removal design][cancelled] at `3f2475e430` owns what a removal leaves and restores when it does not finish,
  forced or not.
  It names this design's words,
  has no question left,
  cites this design's "The act's liveness" for its timeout states,
  and takes the structural-stage third state and the destination directory's leftover as items its text names.
  The records record decides S8 and S11 for both documents;
  this design asks the plan design for no field.

- D3.
  [The deferred-destroy checks][deferred] own step 16's checks and call sites;
  this design needs the scan file's reader and V2's rule.

- D4.
  [The destruction plan record][planrecord] owns the plan's home,
  fields,
  readers and lifetime;
  this design needs no field of it,
  since the records record declined the record of who was ended.

- D5.
  [The lost-last-event triggers][losttriggers] own nudge merging and the rate rule;
  this design adds no nudge.

- D6.
  [The propagation after a replacement mount][propagation] owns the repair;
  this design cites its record (F58) and needs nothing of its build beyond the checkout's privacy before each act.

- D7.
  The slot underlay's file handle,
  which way 8 opens,
  is recorded by the add's step that records the slot identity (step 10 or 14),
  before the clone is mounted;
  this design needs the handle,
  the boot and the GUID from that record.
  The wait and liveness constants are this design's own,
  set by step 17's build only after the owner decides them or M12 sizes them.
  The installed unit's mount-namespace constraint is step 19's.

## Revisions

Each entry names the commit that made the change and its committer time (`git log --format=%cI`),
and what it changed.
The commit that carries this list is the one `git log --format=%cI -1 -- <path>` prints for this file.

- `9078c15831`,
  2026-11-22T17:39:09-05:00:
  the first text,
  non-final and unapproved.
- `bd86c35e50`,
  2026-11-22T17:39:40-05:00:
  the refusal's failed step named,
  F17's wording,
  and the reused-number test's shape.
- `2dda3d993e`,
  2026-11-22T17:40:40-05:00:
  the opening states the one system-manager command that broke the rule.
- `077bf869b6`,
  2026-11-22T18:30:14-05:00:
  the gate review's findings H1 to H4,
  M1 to M9 and L1 to L4,
  the citation corrections,
  the sibling reconciliation,
  and the questions moved and added.
- `ba03f31b47`,
  2026-11-22T18:30:33-05:00:
  this revision list,
  and the final newline.
- `9949210964`,
  2026-11-22T18:32:03-05:00:
  the caller property carries the wrapper's identity rather than each chain entry,
  and why.
- `095b807355`,
  2026-11-22T18:32:10-05:00:
  the chain-carrier commit named in the revision list.

- `f6743550d2`,
  2026-11-22T19:16:06-05:00:
  the record's four answers built in,
  the reach designed (the entering child,
  the move by descriptor,
  the unmount from the destination's descriptor,
  the blockers,
  and the refusals),
  the options not chosen listed once,
  N2 to N5 fixed,
  the sibling cited by section at `2fb7caea17` with its current words,
  the questions reduced to S4,
  S6 and S11,
  the veto list extended to V6,
  and F8 and F10 corrected.
- `ade3af0d49`,
  2026-11-22T19:16:21-05:00:
  this revision list's entry for the round.
- `8f31beacc0`,
  2026-11-22T19:17:00-05:00:
  the refusal's sentence about a holder first seen after the first delete,
  and the revision note that cites the sibling by section.
- `60f69b93a1`,
  2026-11-22T19:17:09-05:00:
  the line break that sentence needed.
- `61644c7a7e`,
  2026-11-22T19:17:27-05:00:
  the three follow-up commits listed.
- `c50e82f397`,
  2026-11-22T19:17:44-05:00:
  the closing note's list of what was kept and added,
  and D2's current refusal word.
- `6871053b79`,
  2026-11-22T19:53:58-05:00:
  the merge of `main` with the record's four further answers.
- `fbbe265fff`,
  2026-11-22T20:03:41-05:00:
  the third revision:
  the shared-namespace case and the six ways,
  the recommended way and its steps,
  the read-only preflight,
  the `SIGTERM`/wait/`SIGKILL` order,
  the `exe`,
  `map_files` and `root` matcher,
  S13 and S11,
  V5 to V7,
  F38 to F46,
  the sibling read at `1fee10f2ce`,
  and the unit's mount-namespace constraint on step 19.
- `bb875b0282`,
  2026-11-22T20:03:54-05:00:
  this round's entries in the revision list.
- `ceb7bc06c2`,
  2026-11-22T20:03:59-05:00:
  the line breaks those entries needed.
- `10a308a973`,
  2026-11-22T21:12:45-05:00:
  the merge of `main` with the corrected reach record and the script-unit answer.
- `7e0f31ed9e`,
  2026-11-22T21:18:03-05:00:
  this revision:
  the ways and ranking corrected to the kernel's propagation rule,
  ways 7 and 8,
  the preflight's endability,
  socket,
  room,
  parent and receiver reads,
  the all-namespace order,
  the wait constant,
  the destination's directory,
  the extended matcher,
  the script refusal,
  and F47 to F54.
- `1ddc60f0d3`,
  2026-11-22T21:18:23-05:00:
  this round's revision-list entries.
- `09b6365c6f`,
  2026-11-22T21:18:32-05:00:
  the line breaks those entries needed.
- `0f9b500c58`,
  2026-11-22T21:47:59-05:00:
  the merge of `main` with the records record.
- `1b7e769230`,
  2026-11-22T21:52:25-05:00:
  this revision:
  the fourth review's A1 to A13,
  the ways' peer case and the repaired state,
  the act's liveness,
  the move after the signals,
  the receiver closure,
  the socket read,
  the leftover directory's prefix,
  the structural-stage state,
  the blind definition,
  S11 and S8 as the records record's,
  the rewritten confirmation and probe,
  and the corrected F45,
  F49 and F53 with F55 to F58.
- `cb3b291f53`,
  2026-11-22T21:52:36-05:00:
  the correction of the previous commit's message spelling.
- `5c2c841f53`,
  2026-11-22T21:52:48-05:00:
  this round's entries in the revision list,
  and the paragraph saying which finding each section holds.
- `a2bc9d9436`,
  2026-11-22T21:53:01-05:00:
  the duplicate sibling bullet removed.

The fourth review's A1 to A13 are built in:
the ways' propagation rule and the peer case (A1,
A2),
the repaired state as the primary arm and the shared checkout as the control (A1),
S11 and S8 as the records record's (A3),
the act's liveness rule (A4),
the move after the signals (A5),
the receiver closure (A6),
the `sock_diag` socket read (A7),
the leftover directory's reserved prefix (A8),
the structural-stage refusal state (A9),
the blind definition (A10),
F53's `may_decode_fh` cite (A11),
L17 closed (A12),
and the `setsid --wait` remedy (A13).
The confirmation and the probe are rewritten;
the questions section is the decided record.
- `48a480cfc6`,
  2026-11-23T03:51:31-05:00:
  the merge of `main` with the accepted sibling designs.
- `f640fe6194`,
  2026-11-23T03:56:27-05:00:
  the fifth review's X1 to X11:
  V8 and the proceed-and-report rule,
  the closure over shared and master groups,
  the structural stage's leaves,
  the heartbeat period and give-up window,
  M2's derived peer rows,
  the arms regrouped into Tiers 0,
  1 and 2 with bounds,
  constants,
  invocations and names,
  and the kernel paragraph.
- `71adcd0e9d`,
  2026-11-23T03:56:35-05:00:
  what the 6.8 guest leaves unshown.
- `813b0dd924`,
  2026-11-23T03:56:42-05:00:
  the line break that sentence needed.
- `2edc939552`,
  2026-11-23T04:29:56-05:00:
  the merge of `main` with the accepted lost-event design.
- `1050d0c48a`,
  2026-11-23T04:32:44-05:00:
  the sixth read's N1 to N9:
  the closure's seed and one members rule,
  the kept receiver as way 8 not used,
  Tier 0's programs,
  creation orders,
  host sides,
  controls and new arms T0-11 and T0-12,
  the claim map,
  and the stopped-unmount answer cited.

## What this revision left as it was

The sections "What this design read" (apart from the fourth review,
the records record and the sibling tip),
"What this design ran",
the kernel and OpenZFS facts F1 to F54 (apart from the corrections to F45,
F49 and F53 and the new F55 to F58),
"How root's finding reaches the wrapper",
the reference definitions,
and the route-level exclusions of "What this design does not cover" were left as they were,
with the corrections the review names.

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[measurements]: cli-git-zfs-measurements-owed-before-acceptance.md
[staging]: ../decision/cli-git-zfs-staging-under-cache.md
[sweep]: ../decision/cli-git-zfs-step-8-sweep-choices.md
[store]: ../decision/cli-git-zfs-slot-identity-store-format.md
[cancelled]: cli-git-zfs-cancelled-or-interrupted-removal-and-cancel-command-design.md
[deferred]: cli-git-zfs-deferred-destroy-checks-design.md
[planrecord]: cli-git-zfs-removal-destruction-plan-record-design.md
[losttriggers]: cli-git-zfs-lost-last-event-triggers-design.md
[propagation]: cli-git-zfs-private-propagation-after-replacement-mount-design.md
[propagationdesign]: cli-git-zfs-private-propagation-after-replacement-mount-design.md
[dirtymain]: cli-git-zfs-dirty-main-restore-on-rung-1-design.md
[record]: ../decision/cli-git-zfs-forced-removal-authority-and-reach.md
[stagingdesign]: cli-git-zfs-staging-under-cache-design.md
[records]: ../decision/cli-git-zfs-clone-worktree-records-seven-answers.md
