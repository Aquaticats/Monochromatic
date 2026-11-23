# The forced arm of a removal: the reach out of a shared namespace, the preflight, and the signal

Non-final,
unapproved design proposal by an agent,
written in the round whose commits are listed under "Revisions".
It is not approved by the project owner and nothing in it is final.
It designs the two open items of the accepted ZFS worktree design that are one mechanism:
"Who ends a holder,
how a holder is identified,
and by what authority",
and "How the wrapper tells its caller's session apart from the other holders".
The record now answers seven of the review's questions ([record][record]):
the channel is the installed root handler;
a seen holder cancels a removal without `--force`;
the forced arm unmounts inside a holder's namespace and ends only the blockers;
the caller's session and every ancestor to the first process are never ended,
and `--force` is refused for a service-manager parent;
and,
in its "Four further answers",
the design must find a way to unmount the copy inside a namespace whose mounts are shared,
the signal is `SIGTERM`,
a wait,
then `SIGKILL`,
and a forced request's authority lasts only while the command that asked is running.
The shared-namespace answer is a direction,
not a final acceptance:
this document designs the ways and recommends one,
a guest measures it,
then the project confirms.
The signal's wait ships no number until the project decides one or a measurement sizes it.
What this document read,
ran and did not run is under those headings.
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
### The four further answers

The record's "A holder in a namespace whose mounts are shared" answers the review's R1:

> The project chose "B:
> design an unmount there":
> the design finds a way to unmount the copy inside such a namespace,
> for example by making its parent mount private first,
> so that the service keeps running and the removal finishes.

The same section states the authorisation's limit:

> this answer is that authorisation,
> for a forced removal's unmount of the clone's own copy and nothing wider.
> It is a direction like the third answer:
> one more design round,
> then a guest measurement,
> then the project confirms.

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

The fourth,
"A cancellation after part of the history is gone",
belongs to [the sibling][cancelled] and is not repeated here.
The record's "Not chosen among these four" lists the alternatives this design does not carry.

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

- [The sibling cancelled-removal design][cancelled] at its current tip `1fee10f2ce` (its document is the text of `70a863eeb5`),
  read last:
  its "The forced arm's failure path",
  "What the caller sees",
  "The stop points",
  "The receipt's new answer field",
  "One reversing request,
  written in one place",
  "Where this design and a sibling design disagree",
  and its questions S7 to S10.

- systemd `v262` at `src/core/namespace.c` and `man/systemd.exec.xml`,
  and Linux `v7.3` at `fs/namespace.c`,
  `fs/pnode.c`,
  `fs/proc/base.c`,
  `fs/proc/fd.c`,
  `fs/proc_namespace.c` and `fs/namei.c`,
  for the propagation and matcher facts below.

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
This round read the record,
the new review and its measurement,
the sibling at its current tip,
and the systemd and kernel sources named above.
It ran no guest,
no ZFS command,
no `sudo`,
no act through the system service manager,
and mounted nothing in the host's own namespace.
The review's own `PrivateMounts=yes` unit measurement is used where it bears;
this round did not repeat it.

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
  The guest's `6.8.0-147-generic` was not verified to carry it (L17).

- F46.
  `unshare` sets a new mount namespace's propagation to private by default since util-linux 2.27,
  and `--propagation unchanged` disables that.
  (Source,
  this host's `man unshare`;
  unrun in the guest.)

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
  Every sentence about the dataset releasing after an unmount or an ending,
  about a shared namespace's propagation change,
  and about what a holder sees,
  is an inference from source plus VM items 4 and 15,
  not a measurement.

- L2.
  The host probes measured this kernel and this systemd,
  not the route's binaries.

- L3.
  One host probe could not write `uid_map` and saw `EPERM` for a one-call user and mount namespace (F16);
  the review's capped run succeeded,
  so the earlier reading is corrected rather than kept.

- L4.
  A session id and a process id are numbers inside one pid namespace;
  neither identifies a process across namespaces,
  and a pidfd created in one namespace is not usable from a sibling namespace.

- L5.
  Two commands broke the project's rule and are not evidence.
  The first `authority-probe.py` ran `systemctl kill --signal=0 dbus.service` (system manager,
  a real system unit,
  no `--user`,
  no `--no-ask-password`),
  which printed `Failed to parse signal string "0"` and never reached the manager.
  At about 2026-11-22T17:24-05:00 the first round ran `systemctl kill --signal=KILL systemd-random-seed.service`,
  which printed `Failed to kill unit systemd-random-seed.service: Connection timed out` after putting a password dialog on the person's desktop;
  the unit stayed `active` with `MainPID` 0.

- L6.
  No task in an uninterruptible sleep was available to probe,
  so a blocker the kernel will not stop is unmeasured.

- L7.
  A mount namespace's inode number,
  a `listns` id and a mount id can be reused after the object dies,
  so the design reads each object again beside the number.

- L8.
  The forced arm's cost is not measured:
  the wait's length,
  the private changes,
  the move,
  the unmount and their waits are owed to the guest.

- L9.
  The wait's length ships as no number (the record's own words).
  Until the project decides one or a measurement sizes it,
  the design bounds the wait by the removal's own deadline,
  and the handler,
  which is not told that client number,
  by its sweep's work bound;
  neither is measured here.

- L10.
  A service unit's parent is the service manager,
  so `--force` is refused there (the fourth answer);
  a harness that must force runs the removal as its own child or under a `--scope`.

- L11.
  `/etc/polkit-1/rules.d` is unreadable to this account,
  so local polkit rules are unread and what `systemctl kill` answers here is unverified (F29).

- L12.
  `listns`,
  a namespace file handle,
  the descriptor unmount,
  and the propagation change by `mount(2)` on a descriptor path were read at source and not run this round (F14,
  F31,
  F32,
  F41);
  the probe arms M1 to M13 owe them a guest.

- L13.
  A confined unit the Pi driver starts is not an ancestor of the wrapper,
  so `--force` can end it if it holds the clone,
  and the handler's own namespace rules do not reach into its unit.

- L14.
  The task matcher over `exe`,
  `map_files`,
  the working directory,
  descriptors and `root` is new code over the handler's own walk (R3);
  only the descriptor record and the path-use ceiling it reuses are measured (F25).

- L15.
  Making a mount private cannot be undone exactly:
  re-sharing invents a new peer group (F39).
  A kill or a refused unmount between the change and the end leaves the holder's namespace with the copy and its parent private,
  and the design reports that state rather than restoring it (V5,
  V6).

- L16.
  The shared-namespace answer is a direction:
  this document designs the ways,
  a guest measures the recommended one,
  and the project confirms afterwards (the record's words).
  Until then nothing here is final.

- L17.
  `mount_setattr` carries a propagation field at Linux `v7.3` (F45),
  which names the mount only by descriptor;
  the guest the route tests on runs `6.8.0-147-generic`,
  whose `mount_setattr` this round did not verify to have it,
  so the design uses `mount(2)` on the descriptor's `/proc` path (F41) and treats the newer call as an alternative,
  not a dependency.

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

- `org.monochromatic:git-worktree-clone-removal-caller`,
  value `v1:<nonce>:<pid>:<starttime>:<parent-pid>:<parent-starttime>:<boot>`.
  The nonce is the intent's 32 lowercase hex digits,
  the numbers are decimal,
  and the boot id is the kernel's own.
  The byte length is bounded from those fields before parsing,
  and a malformed value sets nothing.

The review's earlier reparenting measurement showed that a live chain read after the request can lose the harness,
so the property's parent fields anchor it.
The handler proves the wrapper's pid and start time,
reads the wrapper's parent and requires its pid and start time to be the recorded pair,
and only then walks upward;
nothing above the parent is serialised,
because a serialised list can only age while a live read is the chain as it is.
The walk is the same rule as `chain_ancestors` (F26),
from the proved parent to pid 1,
with two checks:
after reading each parent's record the handler re-reads the child's `ppid` and requires it still to name that parent,
and it requires the parent's start time not to exceed the child's.

The handler also holds a pidfd on the wrapper,
opened once the wrapper's pid and start time are proved,
and checks it immediately before every signal and every unmount,
and includes it in the poll set the act waits on.
The record now decides this:
a forced request's authority lasts only while the very command that asked is running,
checked by its process identity before every signal and every unmount.
Liveness is therefore a property of the exact process,
not a number read per pass.

The protected set,
which the handler spares and never ends,
is two things:
every task whose `session` field equals the wrapper's session (F11),
and every ancestor of the wrapper from its parent to pid 1.
A holder in either set refuses the removal in both arms:
without `--force` the removal cancels and the clone is reattached (the second answer),
and with `--force` the refusal is `caller-protected-holds`.
[The sibling][cancelled] files the same word for the no-force arm in its "What the caller sees",
and its "The stop points" point 5 is where the cancellation happens.

The four shapes,
worked through:

- Pi,
  a command the driver spawns (F1).
  The wrapper is its own session leader and the driver,
  its parent,
  is in another session.
  The protected set is the wrapper and its undetached children,
  plus the driver and every ancestor to pid 1;
  a holder inside it refuses the removal.
- Claude Code's Bash tool (F4).
  The wrapper's parent is a session-leading `zsh` with no terminal,
  so the session is that `zsh`'s and the ancestors are the `zsh`,
  `claude` and everything above;
  both are spared.
- `systemd-run --user --scope` (F3).
  The child shares the calling shell's session and its parent is that shell,
  so the set is Claude Code's.
- `systemd-run --user --pipe --wait`,
  a service unit (F5).
  The wrapper is its own session leader and its parent is `systemd --user`;
  no ancestor is the process that started the unit.
  The fourth answer refuses `--force` here,
  and "A command whose parent is the service manager" says how.

### A command whose parent is the service manager

The fourth answer refuses `--force` outright when the command's parent is the service manager.
The parent is recognised from the kernel's own data,
not from a name:
the system manager is the parent whose pid is 1;
the user manager is the parent whose `/proc/<pid>/cgroup` row is exactly `/user.slice/user-<uid>.slice/user@<uid>.service/init.scope` for the wrapper's uid,
and whose `comm` is `systemd`.
A user process cannot create that cgroup path,
because it lies in a systemd-owned slice,
so the check is not a name test an account can satisfy with a look-alike cgroup.
The check runs in the wrapper,
on its own parent,
before it writes the force property:
the removal fails with exit `2` and prints no request.

The text:

```text
the removal of <slot> was asked to end the holders, and this command's parent is the service manager (pid <pid>, <systemd or systemd --user>), which cannot be told apart from the process that asked for it. If you did not start this removal as a service unit, the command was reparented to the manager because its own parent exited; nothing was changed and nothing was ended: the clone, its snapshots and the Git registration are as they were. Run the removal as a child of the process that wants it: from that process's own shell, or under `systemd-run --user --scope` started by that shell. Do not run it as a service unit.
```

The orphan sentence is the review's R5 point:
a wrapper whose shell exits while it runs is reparented to the user manager and is refused as if it were a unit,
which is safe and confusing,
so the text says which case the reader has.
A wrapper that is alive but whose recorded parent has changed or died has its force word declined as `caller-gone` by the handler;
the wrapper prints that the request was declined,
that nothing was ended,
and that the removal should be run again as a child of the process that wants it.

What a harness does instead:
it runs the removal in its own process,
as the Pi driver already creates a worktree before starting a confined unit,
or starts it under `systemd-run --user --scope` from that process,
so the process that wants the removal is an ancestor.
Whether the refusal also covers a unit whose main process is a script,
where the wrapper's parent is the script rather than the manager,
is S13.

## What a holder is, and what the proof establishes

A holder is one of two things,
and the design treats both with one proof:

- a task whose mount namespace holds a mount of the clone's dataset,
  which is what the root scan reports (F23);
- a task whose working directory,
  open file,
  running executable,
  mapped file or root lies inside a mount of the clone,
  which the scan does not report and which the handler's own walk finds.

The second kind is not decoration.
F17 says a working directory or an open file makes a plain unmount refuse,
and a running binary,
a mapped library or a chroot holds the mount the same way through `exe`,
`map_files` and `root` (F43);
VM item 4 measured a busy unmount caused by exactly the first.
The match is by the mount id the kernel records,
not by a resolved path:
a descriptor's `/proc/<pid>/fdinfo` carries its mount id (F36),
and the working directory,
the executable,
each mapped file and the process root are read with `statx` and `STATX_MNT_ID` on `/proc/<pid>/cwd`,
`/proc/<pid>/exe`,
`/proc/<pid>/map_files/*` and `/proc/<pid>/root`.
Finding every task with any file open on the clone's copy,
running a binary from it,
mapping a library from it,
rooted in it or with a working directory anywhere in it is new code over that record:
the wrapper's existing matcher names one file by mount id and inode,
not every file on a mount (R3,
L14).
The `exe`,
`root` and `map_files` links need ptrace access (F43),
which root has;
a read the handler cannot make is reported as a blind spot,
not as an absent holder.
A task under a child mount of the clone carries the child's mount id and is not matched by the copy's id;
the child mount is itself the blocker that no signal clears.

When the unmount is refused `EBUSY` and the walk found no blocker,
the report says exactly that:
the unmount named the copy,
the search over `cwd`,
descriptors,
`exe`,
`map_files` and `root` found no task,
a reference the design cannot see remains,
and the forced removal refuses `holder-unendable` naming the mount and the namespace.
It never retries blind and never falls back to ending tasks.

Kind (d) of the brief,
a namespace whose task is gone,
is reachable under the chosen reach:
F13 measured that such a namespace lives on a descriptor,
F14 says a privileged caller in the host pid namespace can list every active namespace with `listns` and reopen one by its file handle,
and F15 says entering it with `setns` must happen in a single-threaded process.

The scan's report is evidence that a namespace holds the clone,
never the operand of a signal.
The handler pins first and reads afterwards,
in this order:

1.  It takes a candidate task,
    from the scan's namespace and its own enumeration of that namespace's tasks.
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

What that proves is "this task is in the namespace the fresh scan reported and is the process this descriptor names",
not "this is the task the scan saw".
The design does not need the second:
a signal travels through the descriptor,
so a reused number cannot receive it.
The scan records no start time for the representative task (F23),
and the design does not assume one;
it asks step 9's build to record it so a report can name the exact process,
and says that the forced arm does not depend on that field.

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
    so it acts,
    namespace by namespace:
    `SIGTERM` to every blocker,
    a wait,
    `SIGKILL` to the survivors (the wrapper's pidfd checked before each signal),
    then the propagation changes,
    the move and the unmount ("The reach").
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
    Step 16's scans follow under fresh nonces,
    each carrying the force word again.
7.  The person sees the ordinary removal report plus the holders it ended,
    by namespace and task,
    and the propagation it changed in a holder's namespace when the reach made such a change.
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
with no fallback to ending a namespace.
Its "Four further answers" adds "A holder in a namespace whose mounts are shared":

> The project chose "B:
> design an unmount there":
> the design finds a way to unmount the copy inside such a namespace,
> for example by making its parent mount private first,
> so that the service keeps running and the removal finishes.

The same section states the authorisation's limit:

> this answer is that authorisation,
> for a forced removal's unmount of the clone's own copy and nothing wider.
> It is a direction like the third answer:
> one more design round,
> then a guest measurement,
> then the project confirms.

The review measured the case (R1):
a `PrivateMounts=yes` unit's namespace has all 47 rows `shared:N master:M`,
so the copy and its parent are shared,
the kernel refuses a move out of a shared parent (F40),
and the last revision refused the commonest holder.
This section designs the way out,
and the authorisation's limit is exactly the record's:
the forced removal's unmount of the clone's own copy and nothing wider.

### What the act is aimed at

The aim is to release the dataset's long hold,
not to free the namespace (F20,
F21).
A plain unmount of the clone's copy inside a holder's namespace drops that namespace's reference (F22);
a lazy detach leaves the copy alive while a task holds it,
so it does not release the dataset (F18,
F22).
A mount under the copy,
or a task that references the copy through its working directory,
an open file,
its running executable,
a mapped file or its root,
makes the plain unmount refuse `EBUSY` (F17,
F43);
the first is a mount and no signal clears it,
the second is a blocker and is ended.

### The ways out of a namespace whose mounts are shared

Six ways were weighed,
each with what it changes,
what the service sees,
what reaches other namespaces,
what a kill leaves,
and how the mount is named.
"Private" below means `mount(2)` with `MS_PRIVATE` on a descriptor's `/proc` path,
which changes that one mount's propagation and leaves its peers' groups alone (F39,
F41).

#### Way 1: make the copy and its parent private, then move and unmount

It changes two mounts in the holder's namespace,
the copy and the mount the copy is mounted on;
the destination is a directory on that parent after the change,
so no third mount changes.
The service sees the copy vanish and the parent stop receiving host mount changes under it;
its existing files are unchanged.
The host and other units keep their groups and copies,
the holder's own nested namespaces keep theirs,
and nothing propagates because nothing is attached to a shared mount and the unmount is of a private mount.
A kill leaves the two mounts private,
with the copy at its path or at the destination,
and the report says which (L15).
It names the copy and the parent root by descriptors whose mount ids are checked,
the destination by a descriptor opened before the move,
and the unmount by the destination's descriptor.

#### Way 2: make only the copy private and unmount it in place

It changes the copy only.
The service sees the copy vanish and nothing else.
Its peers keep theirs,
because the copy leaves the group,
and nothing propagates.
A kill leaves the copy private at its path.
It names the copy through the parent's descriptor and the copy's name,
or through the path with the topmost and mount-id checks:
a component the holder can rename before the call.

#### Way 3: make only the parent private, move, and unmount the shared copy at the destination

It changes the parent only.
The service sees the copy vanish and the parent stop receiving host changes.
The unmount propagates within the copy's peer group,
which is the unit's own group and the namespaces the unit made from it;
the host's copy is a master,
not a peer,
and stays.
A kill leaves the parent private and the copy at the destination.
It names everything by descriptor.

#### Way 4: plain unmount of the shared copy in place, changing nothing

It changes nothing.
The service sees the copy vanish,
and propagation is as it was.
The unmount propagates to the whole peer group,
the unit's nested copies included;
the host's copy stays.
A kill leaves nothing to restore:
the unmount either happened or not.
It names the copy by a name in the holder's tree.

#### Way 5: make the copy private, then lazy-detach it by descriptor and close the descriptor

It changes the copy only.
The service sees the copy detach,
and a reference the design cannot see keeps it alive and the dataset busy.
Nothing reaches others,
because the copy is private by then.
A kill leaves a detached copy held by a reference,
reported.
It names everything by descriptor.
It is a lazy detach,
which the accepted busy-teardown and staging sentences forbid.

#### Way 6: make the whole holder namespace private recursively, then move and unmount

It changes every mount in the holder's namespace.
The service receives no host mount change anywhere.
Nothing reaches others outside.
A kill leaves every mount private for the life of the namespace,
and nothing restores it.
It names everything by descriptor.

Not a way:
asking systemd to set `MountFlags=private` or `slave`,
or to stop using `PrivateMounts`,
needs a unit change and a restart,
which is not a removal act;
it is the right setting for a unit that wants a reachable namespace.
Not offered by the kernel:
moving the copy into an anonymous namespace,
because `move_mount` takes a source that is the root of an anonymous namespace or is in the caller's own (F40).
Not a dependency:
`mount_setattr` with its `propagation` field names the mount only by descriptor and would replace way 1's two `mount(2)` calls with two descriptor calls,
but the guest the route tests on runs `6.8.0-147-generic`,
whose `mount_setattr` this round did not verify to carry it (F45,
L17).

### Ranking and the recommended way

The ranking is 1 > 2 > 3 > 4 > 5 > 6.

- 1 over 2:
  every call after the checks is named by a descriptor,
  and no name inside the holder's tree is resolved;
  2 changes one mount fewer but resolves a component the holder can rename.
- 2 over 3:
  2 does not propagate the unmount to the holder's nested copies;
  3 does.
- 3 over 4:
  3 confines the change to one mount and the propagation reaches the copy's own group;
  4 changes nothing and lets the whole peer group receive the unmount.
- 4 over 5:
  4 is a plain unmount the accepted design allows,
  and 5 is a lazy detach it forbids and a hidden reference defeats.
- 5 over 6:
  5 changes one mount,
  6 every mount in the holder's namespace,
  for good.

Way 1 is the design's recommendation.
It uses the record's own example (the parent made private first) and adds the copy's own privacy,
without which a move leaves the mount in its peer group and its unmount propagates;
both changes are inside the holder's namespace and both are the act's means to unmount the clone's own copy,
which is what the record authorises.
V5 keeps the other ways as the veto entry's options,
and the project confirms way 1 after M1 to M3.

### The recommended way, step by step

After the preflight ("The preflight") and the signals ("The signal"),
for one holder namespace:

1.  The child already holds a descriptor on the copy's mount root,
    opened by the mount point once and checked by `statx` `STATX_MNT_ID` and `fstatfs` against the namespace's row (F25,
    F34,
    F36).
2.  It makes the copy private with `mount(NULL, "/proc/self/fd/<copy fd>", NULL, MS_PRIVATE, NULL)`,
    which acts on the mount the descriptor names because the target's magic link jumps to the descriptor's path (F41),
    and re-reads the row:
    it carries no `shared:` and no `master:`.
3.  It opens the parent mount's root from the parent row's mount point,
    checks `statx` gives the parent row's mount id,
    and makes that mount private the same way;
    it re-reads the parent row:
    no `shared:`,
    no `master:`.
4.  It creates the destination directory on the parent mount's root,
    mode `0700`,
    named with the request's nonce,
    and opens it `O_PATH` before the move;
    the attach in the next step is on a private mount and does not propagate (F37).
5.  It re-reads the copy's row and the copy's descriptor's mount id,
    requires them equal,
    and calls `move_mount(copy_fd, "", destination_fd, "", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH)`,
    which takes both ends from descriptors (F40).
6.  It closes the copy's descriptor,
    because a descriptor on the copy would keep it busy for its own unmount (F17),
    and re-reads the table:
    the copy's mount id is topmost at the destination dentry and gone from its old path.
7.  It calls `umount2("/proc/self/fd/<destination fd>", 0)`,
    plain,
    with no `MNT_FORCE` and no `MNT_DETACH`;
    the descriptor is on the parent,
    not on the copy,
    and the lookup steps into the topmost mount at its dentry (F31,
    F32).
8.  It re-reads the table:
    no row carries the copy's mount id,
    and no row of the dataset remains in that namespace;
    it removes the destination directory with `unlinkat` on the parent's descriptor and the recorded name (R6).
9.  It reports the two propagation changes,
    with the copy's and the parent's mount ids and the namespace,
    whether the removal finishes or fails.

### The preflight

No forced removal may end a process or change a mount before it knows it can finish with every holder namespace it will meet (R2).
The preflight is read-only and runs over every holder namespace before the first signal and the first mount change:

- It forks one single-threaded child per namespace,
  enters it (`setns` on the scan's task's namespace,
  a `listns` descriptor for a taskless one),
  and reads `/proc/self/mountinfo` there (F14,
  F15).
- It runs every structural check for each dataset row:
  exactly one own-root copy,
  not the namespace's root,
  no child mount below the copy,
  no second mount of the dataset,
  no `MNT_LOCKED` the move would refuse (F40),
  the copy's and parent's tags recorded (shared,
  slave,
  private),
  and the parent's root path and mount id.
- It confirms a destination can be made:
  the parent mount is not read-only,
  its root is a directory,
  and the nonce's name is free;
  it creates nothing.
- It reads the protected set and every blocker through the matcher,
  and records each read it could not make as a blind spot.
- A structural refusal,
  an unenterable namespace or a protected holder refuses the removal before anything is ended or changed;
  nothing was signalled and no mount changed.
- A blocker that cannot be ended,
  a private change the kernel refuses,
  a destination creation failure,
  an identity race at the act's own check,
  an `EBUSY` after an empty search,
  and the wrapper's death are the refusals that cannot be known by reading;
  they may follow an act,
  and the report names what was ended and what changed,
  with the ended set kept.

The order is fixed:
preflight over every holder namespace,
then for each:
`SIGTERM`,
wait,
`SIGKILL`,
then the private changes,
the destination,
the move,
the unmount.
The preflight and the act read the same rows again,
because a namespace can change between them;
the act's own mount-id checks are what catch that.

### The signal

The further answer decides it:
`SIGTERM`,
a wait for the process to exit,
then `SIGKILL` if it has not.

- Where the wait sits:
  in the blocker step,
  after the preflight and before any mount change,
  per namespace;
  every blocker gets `SIGTERM` through its pinned pidfd,
  the handler waits on their pidfds,
  and the survivors get `SIGKILL` through the same descriptors.
- During the wait the handler polls the blockers' pidfds and the wrapper's;
  if the wrapper's pidfd reports exit,
  it sends no `SIGKILL`,
  stops the act and files `caller-gone`.
- The wrapper's identity is checked immediately before the `SIGTERM` and again before the `SIGKILL`,
  as the further answer requires.
- A process that exits during the wait costs nothing:
  no `SIGKILL` is sent,
  its exit is recorded as "exited after `SIGTERM`",
  and the act goes on as soon as all blockers are gone.
- A blocker that ignores `SIGTERM` is killed after the wait;
  one that has already gone when its signal is due is recorded as gone.
- The wait's length ships as no number (the record's words):
  it is bounded by the removal's own deadline,
  and the handler,
  which is not told that client number,
  by its sweep's work bound;
  the wait ends earlier whenever every blocker exits.
  Neither bound is measured here (L8,
  L9).

### The blockers, and what the matcher reads

A blocker is a task that holds the copy's mount through any of:
its working directory,
an open descriptor,
its running executable (`/proc/<pid>/exe`),
a mapped file (`/proc/<pid>/map_files/*`),
or its root (`/proc/<pid>/root`),
matched by the mount id the kernel records for each (F36,
F43).
The working directory and descriptors were the earlier matcher;
`exe`,
`map_files` and `root` are added by this revision,
and the whole matcher is new code over the handler's walk (R3,
L14).
A task under a child mount is not a blocker of this unmount:
the child mount is,
and it is refused.
A blocker in the protected set refuses the whole removal at the preflight.
A blocker is ended by the signal order above;
a blocker that cannot be ended refuses the removal after the signal,
and the report names it and the ended set.

When the unmount is refused `EBUSY` and the matcher found no blocker,
the report says the search was empty over those five reads,
that a reference the design cannot see remains,
and refuses `holder-unendable` naming the copy,
the namespace and the errno.
It never retries blind.

### What is still refused

The shared copy and the shared parent are no longer refused;
they are the case the recommended way handles.
What the design still refuses:

- a namespace it cannot enter;
- the handler's own namespace,
  and any namespace containing pid 1,
  whose clone mount is the host's and whose unmount is the staging turn's;
  the act there ends path holders only;
- a namespace whose root mount is the copy,
  because the move needs a parent (F40);
- a mount under the copy,
  a blocker no signal clears;
- a second mount of the dataset in the same namespace,
  which the act does not unmount and ending tasks cannot release;
- a blocker it cannot end;
- a private change the kernel refuses,
  a destination it cannot make,
  an identity race at the act's own check;
- an unmount that refuses after an empty blocker search;
- a wrapper that died before an act.

### What the service and other namespaces see

The copy disappears from the service's namespace,
as it would after any unmount.
The parent mount is private from the act onward:
a mount established on the host under that parent later does not appear in the service's namespace,
and a mount the service makes there no longer propagates out (the accepted unit setup had already stopped that direction).
The service's files are unchanged;
its other mounts keep their groups;
the host's mount and every other namespace's copy keep theirs,
and the "nothing wider" control in M8 records that.
The holder's own nested namespaces,
made before the act,
keep their copies;
ones made after it copy a private parent.

### The undo, and what a kill leaves

Making a mount private cannot be undone exactly:
re-sharing invents a new peer group (F39),
and the original group number and master cannot be restored.
If the unmount is refused after the changes,
the child moves the copy back to its recorded path when that path is still on the same parent and carries no mount,
and reports the copy's state and both propagation changes.
A kill between the changes and the end leaves the copy private at its path (or at the destination) and the parent private;
the next run reads that state,
reports it,
and does not pretend to restore it (L15).

### The checks that exist only for a hostile account

The threat model is the project's:
the account is trusted,
and root and the account are not a security boundary here ([staging][staging],
lines 221 to 237).
These checks survive it as guards against a bug or a stale record:
the copy's and parent's mount-id checks on their descriptors,
the post-change tag reads,
the destination descriptor opened before the move,
the name's nonce and `0700` root-owned directory,
the wrapper's pidfd before each call,
and the dataset GUID re-read.
A check that the parent directory is owned by root and not writable by the account exists only against a hostile account and is not adopted:
every call after the checks is descriptor-named,
and the mount-id checks are what the protection rests on.

## The act's mechanism

The preflight,
the signal order and the ways are the reach's;
this section states what holds for every act.

- The handler never signals a task in its own namespace except a path holder whose working directory,
  open file,
  executable,
  mapped file or root is in the clone.
- The handler refuses when any protected task is a holder,
  and a namespace's tasks are all enumerated,
  so a protected task anywhere in it refuses the namespace.
- The number of tasks and namespaces one act reaches is bounded by the scan's own bounds (F23),
  and a scan that stopped at its bound is not a clear answer.
- Every task is pinned and proved before it is signalled,
  and a task whose proof fails is reported,
  not signalled.
- The wrapper's pidfd is checked immediately before every signal and every unmount,
  and again before each mount change the reach makes,
  and it is in the poll set the act waits on (N2;
  the further answer).
- The act's wait is the pidfds' exit (`POLLIN`),
  the wait between `SIGTERM` and `SIGKILL`,
  and,
  for an unmount,
  the kernel call's return,
  with the same pause the receipt wait uses.
  It is bounded by the sweep's own work bound,
  not by the wrapper's 30-second poll deadline (F24):
  the handler is not told that client-side number,
  and a dead wrapper ends the act.

The logging,
through the wrapper's existing notice stream on standard error and the handler's journal:

- one line per blocker and its end:
  the dataset GUID and request nonce,
  the namespace identity,
  the mount id,
  the task's number and birth identity,
  its session,
  the read that found it (cwd,
  descriptor,
  exe,
  map_files or root),
  the signals sent,
  and whether it exited after `SIGTERM` or was killed;
- one line per propagation change:
  the namespace,
  the mount id,
  the old tags and the new,
  and whether the removal finished or failed;
- one line per holder not ended:
  the same fields plus the failing proof;
- one line per refused namespace:
  the namespace,
  the mount row,
  and the refusal;
- one closing line:
  how many blockers were ended,
  how many were unproven,
  whether the following scan was clear,
  and how many namespaces' propagation changed.

The wrapper prints the same facts in its report,
because the decision's report has to name who was ended and what changed.
It does not print them for a holder of the protected set;
that case has its own refusal text.

## The refusal, and what it leaves

Two refusals are the forced arm's,
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
  a private change refused,
  no destination,
  an unmount refused after an empty search,
  a wrapper gone).

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
<the holder, its namespace and the reason: a blocker this removal could not end, a namespace it could not enter, a mount under the clone, a second mount of the dataset, a namespace whose root is the clone, a private change the kernel refused, a destination it could not make, or an unmount that refused after an empty search>.
<what was ended before the refusal, and which namespaces' propagation changed>.
Nothing was destroyed after that point.
```

A holder first seen after the first snapshot delete is met at the next scan,
and the refusal then reports what was already destroyed;
that state is [the sibling][cancelled]'s,
under its stop points.
What the forced arm leaves:
no snapshot delete and no head destroy after the refusal point,
and the restored state,
the records,
the reattach and the nonce of the request that reverses the intent belong to the sibling.
This design states only the forced arm's refusal and writes no intent.
The sibling owns what a removal leaves when it does not finish,
forced or not,
in its "The forced arm's failure path",
"One reversing request,
written in one place" and "The stop points".
It needs three things from this design,
which this design gives:
the writer's pid,
start time and recorded parent in the force property,
the two refusal words above,
and the fact that this design never writes the intent.
The sibling's current text uses `caller-protected-holds` and protects the session and the ancestors;
it does not name `holder-unendable`,
and its fix round owes that word.

## The time between the scan and the ending

A forced request must not act after the command that made it is gone,
and the further answer now decides how:
the handler ends processes only while the very command that asked is still running,
checked by its process identity before every signal and every unmount.
The checks are:

- the force word is honoured only with a standing `unmount` intent whose nonce,
  the force property's and the caller property's all agree;
- the caller property's pid and start time are proved,
  and its recorded parent's pid and start time are proved,
  before the chain is walked (N2);
- the handler holds a pidfd on the wrapper and checks it immediately before every signal and every unmount,
  and in the act's poll set;
- the handler does not read the account-owned removal claim;
  the staging design records that no channel from the claim to the handler exists and that the project chose not to add one ([stagingdesign][stagingdesign],
  "The unmount call from staging"),
  and this design says only that root could read it and the project chose otherwise.

A later trigger that finds a forced request whose wrapper is gone declines the force word,
files `caller-gone`,
and does only the non-force work the intent asks:
it may unmount as any removal would,
and it never ends a process.
A wrapper that is alive but whose recorded parent has changed or died has its force word declined the same way,
and prints that nothing was ended and that the removal should be run again as a child of the process that wants it.

The handler's own clock is the sweep's work bound,
not the wrapper's poll deadline:
the wrapper's deadline is a client wait (F24),
the handler is not told it,
and liveness ends the act sooner.
The wrapper's deadline still bounds its report,
and the report says the request stands,
as the accepted design already requires ([proposal][proposal],
lines 1010 to 1014).

A holder that attaches after the scan is met by the next scan,
which the accepted design already requires before the first snapshot delete and before the head destroy,
and every such scan under the force word acts on the holders it sees.
A holder that attaches after the last scan shows up as `dataset is busy` at the head destroy,
the accepted residual for a holder the scan cannot show;
the retry is bounded by the removal deadline.

A holder that exits and whose number is reused:
nothing is signalled by number,
the pidfd is opened while the number names the proven process,
and the pin's two start-time reads close the interval (F9,
F10).
A namespace identity or a mount id reused after its object died is not enough either:
the link,
the row and the object are read again beside the number (L7).

A holder the scan cannot see:
a path holder in another account,
or one whose resolved path differs,
is the named residual,
and the match by mount id narrows it;
a taskless pinned namespace is reached through `listns` (F14);
a task that survives the act's wait is reported,
not chased.

## Where the records differ, and how this design reads them

- The scan before the move.
  The accepted design's removal-ready scan comes after the move and the unmount,
  and the proposal asks for each scan through a new `unmount` intent.
  This design adds a scan under that same intent before the handler moves anything,
  so the preflight and the first act come before a snapshot is lost.
  Nothing in the accepted design forbids an earlier scan,
  V4 says why it runs there,
  and the sibling's point 5 is that state.

- The scan's strict clear.
  The step 9 scan calls a result clear only when no holder and no incomplete read stands,
  while the destroy-anyway decision covers holders the scan cannot see.
  This design keeps both:
  an incomplete read is reported and is never read as clear,
  and a holder the scan sees stops the destruction (the second answer).
  A scan file that never arrived is different,
  and V2 makes it a refusal at the step that needed it.

- The receipt's contract.
  The sibling adds a sixteenth key,
  `bind`,
  in its "The receipt's new answer field";
  this design adds no key and carries the holder list in the scan file and the refusal in a bounded failed-step word (F24).
  A key and a word can coexist,
  and the receipt's derived bound changes once,
  in the sibling's change.

- The failure path.
  The sibling owns what a removal leaves when it does not finish,
  forced or not,
  in its "The forced arm's failure path";
  this design states only the forced arm's refusal,
  with the two words,
  and writes no intent and no restored state.
  The sibling's current text has three failures and two refusals,
  uses `caller-protected-holds`,
  protects the session and the ancestors,
  and does not name `holder-unendable`;
  that word is all its fix round owes.

- The protected-holder refusal.
  Without `--force`,
  a holder in the protected set refuses and the sibling cancels,
  reattaching the clone;
  with `--force` the same refusal is the forced arm's.
  Both documents now use the same word.

- The claim.
  The sibling says the handler never reads the account's claim;
  this design's liveness proof is the wrapper process,
  not the claim.

## Choices this design marks for the project's veto

Each entry is a writer's choice the design adopts so it is whole.
A veto takes the other option and the sentences that say otherwise.

### V1. Where the scan's finding is carried

Adopted:
the scan's own text in a second root-owned file beside the receipt.
The other options (inside the receipt;
an on-demand privileged run;
outcome only) and the ranking are as the last revision stated:
beside the receipt > inside > on demand > outcome only,
because the first keeps both grammars and readers unchanged.
The project may veto it as before.

### V2. What a receipt without its scan file means

Adopted:
a refusal of the removal at the step that needed the scan.
A veto would make it an `unknown` that does not refuse,
or a retry;
the ranking stays refuse > retry > unknown,
because a missing answer is not the incomplete read the destroy-anyway decision covers.

### V3. How the caller's identity and the force word reach the handler

Adopted:
two clone-local properties bound to the intent's nonce,
the caller one now carrying the wrapper's pid,
start time,
recorded parent and boot.
A veto takes the longer intent grammar or a single request record;
the ranking stays two properties > longer intent > request record,
because the first changes no accepted grammar.

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
(1) make the copy and its parent private,
then move and unmount by descriptors (adopted);
(2) make only the copy private and unmount it in place through a name;
(3) make only the parent private,
move,
and unmount the shared copy at the destination;
(4) plain unmount of the shared copy in place;
(5) make the copy private and lazy-detach it by descriptor;
(6) make the whole holder namespace private recursively.

Concrete example:
a `PrivateMounts=yes` unit holds a copy at `/srv/x` in `mnt:[4026532397]`,
with all 47 rows `shared:N master:M`.
(1) makes the copy and its parent private,
moves the copy to a `0700` directory on the parent,
and unmounts it from that directory's descriptor;
the unit keeps running,
the copy vanishes,
a later host mount under the parent does not appear in the unit.
(2) does the same with one change fewer but names the copy's component in the unit's tree.
(3) moves it and the unmount propagates to the unit's nested namespaces.
(4) propagates to the copy's whole peer group.
(5) uses a lazy detach the accepted rules forbid,
and a hidden reference defeats it.
(6) makes every mount in the unit private for good.

Pros and cons:
(1) keeps every call descriptor-named and propagates nothing,
at the cost of two propagation changes that cannot be undone exactly;
(2) changes one mount and resolves a component;
(3) changes one mount and reaches the unit's nested copies;
(4) changes nothing and reaches the whole group;
(5) changes one mount and breaks an accepted rule;
(6) changes everything in the unit.
Ranking:
1 > 2 > 3 > 4 > 5 > 6,
for the reasons "Ranking and the recommended way" gives:
1 because no name is resolved after the checks and nothing propagates;
2 because one fewer change is not worth a name the holder can rename;
3 because it confines the change but not the propagation;
4 because it changes nothing but propagates widest;
5 because it is a detach;
6 because it changes every mount.
The project confirms option 1 after M1 to M3,
as the record says the confirmation follows the design round and the guest.

### V6. Whether a refused unmount after the move is moved back

Adopted:
move the copy back when its recorded path is on the same parent and carries no mount,
and report both paths and the propagation changes otherwise.
A veto leaves the copy at the destination and reports it;
the ranking stays move back > leave and report > detach,
because the first restores what the act found.

### V7. Whether the report and `doctor` name a changed propagation

Adopted:
the receipt and the report name every propagation change,
with the namespace and mount ids,
and step 18's `doctor` prints them.
A veto keeps the report only;
the own-record option has no reader or pruner.
Ranking:
report and `doctor` > report only > own record,
because the first reaches a person inspecting the clone later and adds no record of its own.

## Questions only the project can answer

The record decided S4 and S6:
the signal is `SIGTERM`,
a wait,
then `SIGKILL`,
and a forced request's authority lasts only while the command that asked runs.
It also answered S12 by choosing "design an unmount there",
and the project confirms the recommended way after the guest (V5).
S7 to S10 belong to [the sibling][cancelled];
S1,
S2,
S3 and S5 are the record's first four answers.
These two are still open,
numbered as the reviews number them.

### S13. Extend the service-manager refusal to a unit whose main process is a script

In words:
"Refuse `--force` also when the command's own session was started directly by the service manager (a unit whose main process is a script),
not only when the wrapper's parent is the manager?"

Options:
(a) the record's words,
the wrapper's parent only;
(b) also the command's session leader's parent.

Concrete case:
`systemd-run --user --unit=x bash -c 'git worktree remove --force one'`.
(a) allows it and leaves its starter unprotected;
(b) refuses it,
and still allows Pi (its commands' session leader has the driver as parent) and Claude Code.

Pros and cons:
(a) is what the record says and costs nothing;
(b) closes the script case and refuses a few legitimate unit scripts,
which a person answers by running the script as a child.
Ranking:
(b) > (a),
because the starter of a unit is the process the answer meant to protect and the cost is a refusal the person can answer.
The design implements (a);
a decision for (b) changes only the parent the check reads,
from the wrapper's to its session leader's.

### S11. Whether a forced ending leaves a durable record beyond the journal

In words:
"Should the plan record remember who a forced removal ended?"

Options:
the handler's journal and the wrapper's standard error;
an entry in the removal's plan record;
a record of its own per slot.

Concrete case:
a week after a forced removal a person asks who was ended;
the journal may have rotated;
a plan entry survives as long as the plan;
a record of its own survives until something prunes it,
and nothing does.

Pros and cons:
the journal adds no state and rotates;
a plan entry survives rotation and sits beside the destroyed set,
and needs a field the plan design lacks today ([planrecord][planrecord],
its fields section);
a record of its own is exact and has no reader or pruner.
Ranking:
plan entry > journal > own record,
because the plan entry survives rotation and the journal does not,
and the own record has no reader or pruner.
The plan entry is contingent on that design adding the field,
which this design asks of it (D4);
until then the journal ships.

## Which build step builds which part

- Step 9's holder-scan round has built the scan,
  the text,
  the parser and the mode,
  as work in flight;
  this design asks that build to record the representative task's start time (F23 has none),
  and it builds no scan there.
- Step 16 owns the scans' call sites and the deferred-destroy checks ([deferred][deferred]);
  the scan file's publisher and reader are built with the first consumer,
  step 16 or step 17,
  and this design needs only the read interface and V2's rule.
- Step 17 builds this design:
  the caller and force properties,
  the chain read with its parent anchor and hop checks,
  the pidfd on the wrapper,
  the service-manager parent refusal,
  the two refusal texts and their receipt words,
  the pre-move scan,
  the read-only preflight,
  the pin-first proof,
  the extended blocker matcher,
  the `SIGTERM`/wait/`SIGKILL` order,
  the private changes,
  the destination,
  the move,
  the unmount,
  the logging and the tests.
- Step 18 builds the `doctor` reading:
  the last scan file,
  the live scan mode,
  the holders named by namespace and task,
  the protected set,
  and the propagation changes a removal reported (V7).
- Step 19 builds the installed handler's unit,
  and that unit must carry no mount-namespace option (`PrivateMounts=`,
  `ProtectSystem=`,
  `ProtectHome=`,
  `PrivateTmp=` and the like) and no mount-namespace sandboxing,
  because the handler must run in the host mount namespace for its staging turn to see the host's clone mount and its path holders (A8,
  R8).
  The chosen channel adds no sudoers line and no helper.
- No step builds a second scan,
  a fallback kill-all,
  a lazy detach,
  a forced unmount of the host's mount,
  or a change to another unit's mount settings.

## Tests a build must have

Each test names the race or interleaving it forces,
how it forces it without timing luck,
and whether it needs a guest.

1.  The proof refuses a task that exits before its pin.
    Enumerate a task,
    let it exit,
    then run the proof and assert no signal.
    Device-free for the proof's own code;
    the guest owns the part that reads the host `/proc`.
2.  A signal reaches the pinned process only.
    Pin one child,
    end it,
    spawn a second with `set_tid` equal to its number,
    and assert a signal through the first child's descriptor fails while the second is untouched.
    Device-free,
    with the guest repeating it against the real `/proc`.
3.  The protected-set refusal for a working directory and for a namespace,
    with and without `--force`.
    A holder in the wrapper's session;
    assert the two texts,
    the holder alive,
    and without `--force` the cancel and reattach the sibling owns.
    Guest.
4.  Another session's holder ends while the caller lives,
    after the preflight and the signal.
    Start the holder in a new session,
    wait on a pipe,
    run the removal,
    and assert its wait status,
    the caller alive,
    a second job of the caller's session alive,
    and the head destroy done.
    Guest.
5.  The ancestor half is spared.
    Start the removal from a child of a process that holds the clone but is not in the wrapper's session,
    and assert the refusal names that ancestor and it stays alive.
    Device-free with fixture `/proc`;
    guest for the real removal.
6.  A holder that appears between the scan and the act is met by the next one.
    Force the interleaving with an injected hook between the scan and the act,
    of the shape `take_claim_seamed` gives the claim,
    not with a watch on the results folder.
    Guest.
7.  A receipt without its scan file refuses at a destroy step;
    a stale or absent caller property,
    a dead wrapper,
    and a wrapper whose recorded parent changed each decline the force.
    Assert no signal,
    `caller-gone` where the wrapper is gone,
    and only the non-force work.
    Device-free.
8.  The wrapper ends between two passes and during the wait.
    Kill it at the injected hook after the first pass and during the `SIGTERM` wait;
    assert the second pass sends nothing,
    no `SIGKILL` follows,
    the receipt says `caller-gone`,
    and the half-done state is reported.
    Guest.
9.  The preflight refuses before any act.
    Two holder namespaces,
    the second with a structural refusal (a child mount,
    or a protected holder);
    assert no signal and no mount change in the first,
    and the refusal names the second.
    Guest.
10. The service-manager parent refuses `--force`.
    Run the removal as a `systemd-run --user --pipe --wait` unit,
    as a system unit whose parent is pid 1,
    and as a unit whose main process is a script;
    assert exit `2`,
    the text (with the orphan sentence when the wrapper was reparented),
    no property written,
    and nothing changed;
    the script case is allowed under (a) and refused under S13's (b);
    a parent whose cgroup merely looks like the manager's,
    under the account's own subtree,
    is not refused.
    Device-free over a fixture `/proc` for the recognition;
    guest for the real run.
11. The default sandboxed unit is reached.
    A `PrivateMounts=yes` unit with no `MountFlags=` holds a copy;
    record its mount table before (all rows `shared:`),
    run with `--force`,
    and assert the copy unmounted after the two private changes,
    the head destroy done,
    the unit alive,
    the destination removed,
    and the mount table after showing the parent's new tags.
    Guest.
12. A `MountFlags=private` unit and an `unshare --mount` namespace are reached without a propagation change;
    record the tables before and after.
    Guest.
13. A taskless namespace is found through `listns` and unmounted;
    a namespace whose root is the copy is refused;
    a second mount of the dataset is refused.
    Guest.
14. The matcher finds what the earlier one missed.
    A test binary running from the clone (`exe`),
    a process mapping a library from it (`map_files`),
    and a chroot into it (`root`);
    each is found,
    ended after the signal order,
    and the unmount succeeds.
    Also a task whose read is refused is reported blind,
    and an `EBUSY` after an empty search refuses `holder-unendable`.
    Guest.
15. The signal order.
    A blocker that handles `SIGTERM` and exits is not sent `SIGKILL` and its exit is recorded;
    one that ignores it is killed;
    the wrapper's pidfd stops the second signal.
    Guest.
16. Nothing wider.
    Before and after the act,
    record the host's mount table,
    another unit's table and an `unshare` namespace's table;
    assert no other namespace's copy vanished and no other row's tags changed.
    Guest.
17. The move back and the undo report.
    Stack a mount on the destination between the move and the unmount,
    or make the unmount refuse,
    and assert the copy is back at its recorded path when it can be,
    both propagation changes are named,
    and nothing else moved.
    Guest.
18. The refusals.
    A namespace that cannot be entered,
    a child mount,
    a second mount,
    a root copy,
    a protected holder,
    a private change the kernel refuses,
    a destination it cannot make,
    an unmount refused after an empty search;
    each refuses before an act where reading can know it,
    and names what was ended and changed where it cannot.
    Guest.
19. `doctor` names holders,
    the protected set and the propagation changes without root.
    Device-free.

## Measurements owed

M1 to M12 are the arms of one guest run,
and each is a specification for the build step that builds the forced arm:
the state,
the one action,
the expected result,
and the bound on its wait.
The fixture is [measurements][measurements]:
an encrypted file-backed pool,
OpenZFS `2.4.4`,
sibling checkout and clone-parent datasets,
the built wrapper,
the built handler,
and plain `git` in the guest with a disposable `HOME` and `XDG_RUNTIME_DIR`.
Every arm records the unit's or namespace's mount table before and after,
with the `shared:`,
`master:` and `unbindable` tags;
the receipt,
the scan file,
the wrapper's standard error,
`doctor`'s rows,
the journal lines,
and the state of the clone,
its snapshots and the registration.
A run that cannot show the arm is recorded as unrun,
never as passed;
no value ships before the arm runs.

- M1.
  State:
  a `PrivateMounts=yes` unit started after the clone was attached,
  with no `MountFlags=` (the review measured all 47 rows `shared:N master:M`);
  a second reader namespace;
  no task using either copy.
  Action:
  one `git worktree remove --force <slot>`.
  Expected:
  the preflight is clean;
  the copy's and its parent's rows become private (no `shared:`,
  no `master:`),
  the copy is moved and unmounted,
  both copies are gone from their namespaces,
  both namespaces' processes alive,
  the head destroy exits 0,
  the destination directory is gone,
  the unit's later view:
  a host mount created under the parent after the removal does not appear in the unit.
  Bound:
  the child's calls and the `SIGTERM` wait return before the sweep's work bound;
  the record notes each call's and the wait's elapsed time.
  Control:
  the same arm without `--force`,
  recording the cancel,
  both copies staying,
  and the clone reattached.

- M2.
  State:
  a `PrivateMounts=yes` unit with `MountFlags=private`,
  holding a copy.
  Action:
  one `--force` removal.
  Expected:
  the table before is private already;
  the preflight records no propagation change needed;
  the copy is moved and unmounted;
  the head destroy succeeds;
  the unit alive.
  Bound:
  as M1.

- M3.
  State:
  an `unshare --mount` namespace (private by default) holding a copy;
  a taskless namespace pinned by a descriptor.
  Action:
  one `--force` removal.
  Expected:
  both are reached,
  the `unshare` copy unmounted with no propagation change,
  the taskless one found by `listns`,
  entered in a single-threaded child,
  unmounted;
  the head destroy succeeds.
  Bound:
  as M1.

- M4.
  State:
  M1's unit and a second holder namespace with a structural refusal (a child mount under the copy,
  or a protected holder).
  Action:
  one `--force` removal.
  Expected:
  the preflight refuses before any signal and any mount change;
  the first namespace's copy is untouched;
  the report names the second.
  Bound:
  no wait;
  the record notes that no signal and no mount call ran.

- M5.
  State:
  M1's unit,
  with a web server that never touched the clone and a helper whose working directory is in the copy.
  Action:
  one `--force` removal.
  Expected:
  the helper is found (cwd) and ended after `SIGTERM` or `SIGKILL`;
  the web server runs;
  the unmount and head destroy succeed.
  Bound:
  as M1.

- M6.
  State:
  a task running a test binary from the clone,
  one mapping a library from it,
  and one chrooted into it.
  Action:
  one `--force` removal.
  Expected:
  the matcher finds each through `exe`,
  `map_files` and `root`;
  each is ended;
  the unmount succeeds;
  a refused read is reported blind;
  an `EBUSY` after an empty search refuses `holder-unendable`.
  Bound:
  as M1.

- M7.
  State:
  a blocker that handles `SIGTERM` and exits;
  one that ignores it;
  the wrapper killed during the wait.
  Action:
  one `--force` removal for each.
  Expected:
  the first is not sent `SIGKILL` and its exit is recorded;
  the second is killed;
  the third stops the act,
  sends no `SIGKILL`,
  and files `caller-gone`.
  Bound:
  the wait ends at each process's exit or at the sweep's bound;
  the record notes both.

- M8.
  State:
  M1's unit and a control that records every namespace's table:
  the host's,
  another unit's and an `unshare` namespace's.
  Action:
  one `--force` removal.
  Expected:
  no other namespace's copy vanished and no other row's tags changed;
  the host's clone mount and its propagation are as they were;
  the holder's parent's tags changed only in the holder's namespace.
  Bound:
  no wait.

- M9.
  State:
  a namespace that cannot be entered;
  a copy with a child mount;
  a second mount of the dataset;
  a namespace whose root is the copy;
  a destination that cannot be made;
  a private change the kernel refuses.
  Action:
  one `--force` removal for each.
  Expected:
  each is refused with its named reason;
  where reading can know it,
  nothing was signalled and no mount changed;
  where it cannot,
  the report names what was ended and changed.
  Bound:
  the refusal needs no wait;
  the record notes which check fired.

- M10.
  State:
  the service-manager shapes:
  a `systemd-run --user --pipe --wait` unit whose main process is the removal;
  a unit whose main process is a script that runs it;
  a wrapper orphaned by its parent exiting.
  Action:
  one `--force` removal for each.
  Expected:
  the first is refused with the manager text;
  the second is allowed under the current rule (and refused under S13's option (b));
  the orphan is refused with the text naming the reparenting.
  Bound:
  the refusal is before any property write.

- M11.
  State:
  the authority answers,
  shared with the sibling:
  an unprivileged account with no authentication agent.
  Action:
  `kill` and `pidfd_send_signal` on a root process;
  `systemctl --user --no-ask-password kill --signal=SIGKILL` on its own transient unit;
  `systemctl --no-ask-password kill --signal=SIGKILL` on a system unit it does not own.
  Expected:
  `EPERM`;
  the own-unit kill works;
  the system-unit request is refused for want of authorization.
  Bound:
  the system-manager request returns without an agent;
  the record notes the elapsed time.

- M12.
  State:
  the scan file's latency and the act's cost.
  Action:
  time the publish to the wrapper's read,
  the `SIGTERM` wait for one and for many blockers,
  one private change,
  the move and the unmount.
  Expected:
  the numbers step 22's removal deadline and the wait's sizing need.
  Bound:
  no value ships before this runs.

## Limitations

- Every claim about the dataset releasing,
  the propagation changes,
  the move and the unmount is an inference from source and VM items 4 and 15 (L1,
  L12).
- The caller's protected set is a number inside one pid namespace (A1,
  L4).
- The caller property is an assertion the handler ties to a live wrapper and its recorded parent,
  not to a sender (A4).
- The default sandboxed unit is reached only by changing its propagation,
  which the record authorises as a direction (L16);
  a kill or a refusal leaves the copy and its parent private,
  and the design reports rather than restores (L15).
- A confined unit the Pi driver starts is not an ancestor and can be ended if it holds the clone (L13).
- `listns`,
  a namespace file handle,
  the descriptor unmount and the propagation change by `mount(2)` on a descriptor path are source-read and unrun (L12);
  M1 to M9 owe them.
- The blocker matcher over `exe`,
  `map_files` and `root` is new code (L14);
  only the descriptor record and the path-use ceiling it reuses are measured.
- The service-manager answers and local polkit rules are unverified on this host (L5,
  L11).
- The wait's length is unmeasured (L9);
  until the project decides one,
  the removal's deadline and the sweep's bound stand in,
  neither measured.
- The plan record has no field for "who was ended",
  so S11's first-ranked home is contingent on D4.

## What this design does not cover

- What a cancelled,
  interrupted,
  timed-out or killed removal leaves,
  and the kept `cancel-worktree-removal` command:
  the sibling owns it,
  including the forced stop point this design requires of it (D2).
- Where the removal's persisted destruction plan lives,
  which fields it has,
  and what a forced ending should record in it (D4,
  S11).
- The deferred-destroy checks themselves and step 16's call sites (D3).
- A second scan,
  any force or lazy unmount of a staged mount,
  any fallback that ends every task of a namespace,
  any unmount of the host's clone mount by the forced arm,
  and any change to a unit's own mount settings.
- A namespace the reach refuses for a reason this design names,
  beyond the refusal and the report:
  no recovery is designed for a copy left at a destination when moving it back also fails,
  and no restoration of the original propagation is designed.
- The handler's installation,
  its unit hardening and ZED's own timeouts;
  the unit's mount-namespace constraint is a requirement on step 19 (R8),
  and whether a kill-and-wait inside a zedlet child is bounded by ZED is not answered here.
- The add-time Git lock,
  the add binding record,
  the dirty-main restore,
  the origin-cleanup record,
  and the private propagation after a replacement mount (D6).
- Container removals,
  and a caller named across a service boundary (L10).
- Any number:
  no grace,
  retry count,
  timeout or ceiling ships from this design (L9).

## Dependencies on sibling designs

Each dependency says what this design assumes,
and where a sibling disagrees,
that it does.

- D1.
  Step 9's unmount and reattach build.
  This design assumes the pre-move scan can run under the same request before the move,
  and that a busy unmount still takes the reattach.
  The staged unmount round is in flight;
  nothing of the route is released.

- D2.
  [The sibling cancelled-removal design][cancelled] at `1fee10f2ce` (its document is the text of `70a863eeb5`) owns what a removal leaves and restores when it does not finish,
  forced or not:
  its "The stop points",
  "The forced arm's failure path",
  "What the caller sees" and "One reversing request,
  written in one place".
  This design states only the forced arm's refusal,
  gives it the two words `caller-protected-holds` and `holder-unendable`,
  and writes no intent.
  The sibling's current text uses `caller-protected-holds`,
  protects the session and the ancestors,
  has three failures and two refusals,
  and does not name `holder-unendable`;
  that word is what its fix round owes.
  It adds a sixteenth receipt key,
  `bind`,
  in its "The receipt's new answer field";
  this design adds no key.

- D3.
  [The deferred-destroy checks][deferred] own step 16's checks and call sites.
  This design claims no ownership of them;
  it needs the scan file's reader and V2's rule,
  and the build place is the first consumer's choice.

- D4.
  [The destruction plan record][planrecord] owns the plan's home,
  fields,
  readers and lifetime.
  This design needs the record only if S11 chooses the plan entry,
  and then it needs a field for who was ended and for the propagation changes,
  which that document does not have.

- D5.
  [The lost-last-event triggers][losttriggers] own nudge merging and the rate rule.
  This design adds no nudge;
  the forced request is the intent plus two properties,
  and the merge rule does not read them.

- D6.
  [The propagation after a replacement mount][propagation] states "no dependency" on this design.
  This design keeps the scan's visibility unchanged from VM item 15 and needs nothing from that design.

- D7.
  The chosen channel adds no sudoers line and no helper.
  The reach adds the preflight,
  the extended matcher,
  the signal order,
  the private changes and the move to the handler,
  which are step 17's build;
  the installed unit's mount-namespace constraint is step 19's (R8).

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
  the third revision: the shared-namespace case and the six ways,
  the recommended way and its steps,
  the read-only preflight,
  the `SIGTERM`/wait/`SIGKILL` order,
  the `exe`, `map_files` and `root` matcher,
  S13 and S11,
  V5 to V7,
  F38 to F46,
  the sibling read at `1fee10f2ce`,
  and the unit's mount-namespace constraint on step 19.

The four answers are built in:
"Who ends each kind of holder" is the chosen channel,
"The reach" is the unmount and its blockers (and, after this round, the way out of a shared namespace, the preflight and the signal order),
"The caller and its protected set" is the session plus every ancestor to pid 1 and the service-manager refusal,
and "The refusal" carries the protected-set word in both arms, leaving the unforced cancel for a holder outside the set to the sibling.
N2 is fixed in "The caller and its protected set" and "The time between the scan and the ending";
N3 in "A command whose parent is the service manager";
N4 by "The reach" and its M3 to M7 and M13;
N5 in "What a holder is".

## What this revision left as it was

The sections "What this design read" (apart from the new review,
the measurement,
the sibling tip and the systemd and kernel sources),
"What this design ran",
the kernel and OpenZFS facts F1 to F37 (apart from the new F38 to F46),
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
[record]: ../decision/cli-git-zfs-forced-removal-authority-and-reach.md
[stagingdesign]: cli-git-zfs-staging-under-cache-design.md
