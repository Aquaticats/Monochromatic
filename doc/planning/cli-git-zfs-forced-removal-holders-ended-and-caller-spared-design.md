# The forced arm of a removal: the chosen reach, the caller's protected set, and the probe the build owes

Non-final,
unapproved design proposal by an agent,
written 2026-11-22T19:05:00-05:00.
It is not approved by the project owner and nothing in it is final.
It designs the two open items of the accepted ZFS worktree design that are one mechanism:
"Who ends a holder,
how a holder is identified,
and by what authority",
and "How the wrapper tells its caller's session apart from the other holders".
The project answered four questions about them on 2026-11-22,
recorded in [the authority and reach record][record]:
the channel is the installed root handler,
a seen holder cancels a removal without `--force`,
the forced arm unmounts inside a holder's namespace and ends only the blockers,
and the caller's session and every ancestor to the first process are never ended,
with `--force` refused when the command's parent is the service manager.
The record's third answer is a direction,
not a final acceptance:
the reach it chooses is not final until this design states its mechanism and a guest measures it.
This document states the mechanism in "The reach:
unmounting inside the holder's namespace",
and the probe the build owes is under "Measurements owed".

What it read:
the record whole,
the two open items and the decisions around them,
the second-pass gate review,
the sibling cancelled-removal design at its tip,
the accepted staging design's unmount,
and the code,
kernel,
OpenZFS,
systemd and sudo sources named below.
What it ran:
no probe this round.
The five host probes of the earlier round stand as they were and are listed under "What this design ran",
and this round read source and records only.
What it did not run:
no guest,
no virtual machine,
no container,
no ZFS command that changes anything,
no `sudo`,
and no act through the system service manager.
The two host commands that broke the never-prompt rule are recorded under L5 and are not evidence.

The design is [the decision record][decision],
the whole design is [the proposal][proposal],
the build order is [the plan][plan],
and the measured facts the route already has are [the measurement record][measurements].

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

- [The sibling cancelled-removal design][cancelled] at commit `2fb7caea17`,
  read whole:
  its "The stop points",
  "The forced arm's failure path",
  "What the caller sees",
  "The receipt's new answer field",
  "One reversing request,
  written in one place",
  "Where this design and a sibling design disagree",
  "Dependencies on the sibling designs",
  and its questions Q7 to Q10.
  It names itself the owner of what a removal leaves when it does not finish;
  this design takes its refusal kinds and restoration from it and gives it back the two refusal words.

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

## What this design ran

No probe this round.
The five host probes of the earlier round stand,
kept with their outputs in
`/home/user/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/keep/`:

1.  `process-relations.ts` printed the session,
    process group,
    cgroup and ancestors of a command the Pi harness ran,
    and of a job that command left behind.
2.  `pidfd-probe.py` opened a pidfd on an own child,
    signalled it,
    re-opened after the reap,
    opened a pidfd on pid 1,
    asked the kernel to send pid 1 signal 0,
    and printed session memberships of a shared job and a `setsid` child.
3.  `namespace-pin-probe.py` made a private user and mount namespace,
    had a holder make a mount namespace,
    mount a tmpfs and pass the namespace descriptor out,
    then killed the holder and read the namespace from the descriptor.
4.  `authority-probe.py` started a transient unit of this account,
    killed it through the user manager,
    and read the kernel's permission over pid 1.
    No line can ask the system service manager to act,
    and every `systemctl` it runs carries `--user` and `--no-ask-password` and names only a unit it started.
5.  Two inline Python arms wrote `cgroup.kill` in a cgroup this account owns,
    and created a child cgroup.

The keep log records two attempts that asked the system service manager to act,
made before the rule reached this run:
`systemctl kill --signal=0 dbus.service` (stopped by `systemctl`'s own argument check before any request left the process) and the later `systemctl kill --signal=KILL systemd-random-seed.service` (L5).
Neither is evidence.

No program ran inside a guest.
No ZFS command ran on the host.
Nothing mounted in the host's own namespace.
This round read the record,
the review,
the sibling document and the sources named above,
and changed this document only.

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
  No guest ran in either round.
  Every sentence about ZFS releasing the dataset after a plain unmount or after ending a blocker,
  and every sentence about what a holder sees,
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
  Earlier in the first round the first `authority-probe.py` ran,
  as its arm 2,
  `systemctl kill --signal=0 dbus.service` (system manager,
  a real system unit,
  no `--user`,
  no `--no-ask-password`);
  it printed `Failed to parse signal string "0"` and never reached the manager.
  At about 2026-11-22T17:24-05:00 the first round ran `systemctl kill --signal=KILL systemd-random-seed.service`;
  it printed `Failed to kill unit systemd-random-seed.service: Connection timed out` after putting a password dialog on the person's desktop,
  and the unit stayed `active` with `MainPID` 0.
  The design takes the polkit facts from source (F28,
  F29),
  and the run that would show what such a request answers belongs in a guest (M8).

- L6.
  No task in an uninterruptible sleep was available to probe,
  so the case of a holder the kernel cannot stop is unmeasured.

- L7.
  A mount namespace's inode number,
  a `listns` id and a mount id can be reused after the object dies,
  so the design never rests on a number alone:
  each is read again beside the object it names.

- L8.
  The forced arm's cost is not measured:
  how long an unmount inside a namespace,
  an ending and their waits take,
  and how often a holder reappears,
  are owed to the guest.

- L9.
  No number is shipped by this design.
  The protection's stop is pid 1 (F26),
  the handler's work is bounded by the sweep's own bound,
  and no grace or retry count is proposed.

- L10.
  A service unit's parent is the service manager,
  so `--force` is refused there (the fourth answer);
  the outer process that started the unit is not in the protected set,
  and a harness that must force runs the removal as its own child or under a `--scope`.

- L11.
  `/etc/polkit-1/rules.d` is unreadable to this account,
  so local polkit rules are unread and what `systemctl kill` answers here is unverified (F29).

- L12.
  `listns`,
  a namespace file handle,
  and the descriptor unmount inside a holder's namespace were read at source and not run this round (F14,
  F31,
  F32,
  F33);
  M3 to M7 owe them a guest.

- L13.
  A confined unit the Pi driver starts is not an ancestor of the wrapper,
  so `--force` can end it if it holds the clone,
  and the handler's own namespace rules do not reach into its unit.
  "What this design does not cover" states it.

- L14.
  The task matcher that finds every task with a file open on the clone's copy or a working directory in it is new code over the handler's own walk (N5);
  only the descriptor record and the path-use ceiling it reuses are measured (F25).

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

The review's N2 measured that a live chain read after the request can lose the harness:
a child whose parent shell exits is reparented to the user manager,
so the chain no longer contains the process that started it.
The property's parent fields answer that:
the handler proves the wrapper's pid and start time,
then reads the wrapper's parent and requires its pid and start time to be the recorded pair,
and only then walks upward.
A wrapper whose parent changed or died before the handler read it is treated as gone,
and the force word is declined (see "The time between the scan and the ending").
The recorded pair is the anchor;
nothing above it is serialised,
because a serialised list can only age while a live read is the chain as it is.

The walk itself is the same rule as `chain_ancestors` (F26),
from the proved parent to pid 1,
with two checks the review's N2 adds:
after reading each parent's record the handler re-reads the child's `ppid` and requires it still to name that parent,
and it requires the parent's start time not to exceed the child's,
which no real ancestry can violate and a redirected number can.
The walk stops at pid 1,
which is in the set.

The handler also holds a pidfd on the wrapper,
opened once the wrapper's pid and start time are proved,
and checks it immediately before each signal and each unmount,
and includes it in the poll set the act waits on.
Liveness is therefore a property of the exact process,
not a number read per pass:
a pass cannot follow a Ctrl-C that arrived between the pass's start and its first call.

The protected set,
which the handler spares and never ends,
is two things:

- every task whose `session` field equals the wrapper's session (F11),
  which is the decision's "the caller and the processes of its session";
- every ancestor of the wrapper from its parent to pid 1,
  which the handler reads itself from the proved parent.

A holder in either set refuses the removal,
and with `--force` that refusal is the forced arm's (`caller-protected-holds`);
without `--force` the sibling design cancels the removal before anything is deleted (its "The stop points",
point 5).

The four shapes the brief and the review name,
worked through:

- Pi,
  a command the driver spawns (F1).
  The wrapper is its own session leader and the driver,
  its parent,
  is in another session.
  The protected set is the wrapper and its undetached children,
  plus the driver and every ancestor to pid 1.
  A holder inside it refuses the removal.
- Claude Code's Bash tool (F4).
  The wrapper's parent is a session-leading `zsh` with no terminal,
  so the session is that `zsh`'s and the ancestors are the `zsh`,
  `claude` and everything above.
  Both are spared.
- `systemd-run --user --scope` (F3).
  The child shares the calling shell's session and its parent is that shell,
  so the set is Claude Code's.
- `systemd-run --user --pipe --wait`,
  a service unit (F5).
  The wrapper is its own session leader and its parent is `systemd --user`;
  no ancestor is the process that started the unit.
  This shape is now refused rather than left unprotected (N3),
  and "A command whose parent is the service manager" says how.

### A command whose parent is the service manager

The fourth answer ([record][record],
"Which processes a forced removal never ends") refuses `--force` outright when the command's parent is the service manager.
The parent is recognised from the kernel's own data,
not from a name:

- the system manager:
  the parent's pid is 1;
- the user manager:
  the parent's `/proc/<pid>/cgroup` row is exactly `/user.slice/user-<uid>.slice/user@<uid>.service/init.scope`,
  where `<uid>` is the wrapper's uid,
  and the parent's `comm` is `systemd`.

A user process cannot create that cgroup path,
because it lies in a systemd-owned slice,
so the check is not a name test an account can satisfy with a look-alike cgroup;
the `comm` read is a second check against a manager whose cgroup row was not read.
The check runs in the wrapper,
on its own parent,
before it writes the force property:
the removal fails with exit `2` and prints no request.

The text:

```text
the removal of <slot> was asked to end the holders, and this command's parent is the service manager
(pid <pid>, <systemd --user or systemd>), which cannot be told apart from the process that asked for it.
Nothing was changed: the clone, its snapshots and the Git registration are as they were.
Run the removal as a child of the process that wants it: from that process's own shell, or under
`systemd-run --user --scope` started by that shell. Do not run it as a service unit.
```

What a harness does instead:
it runs the removal in its own process,
as the Pi driver already creates a worktree before starting a confined unit,
or starts it under `systemd-run --user --scope` from that process,
so the process that wants the removal is an ancestor.
A harness that can only run a service unit cannot force a removal,
and the message says so.

## What a holder is, and what the proof establishes

A holder is one of two things,
and the design treats both with one proof:

- a task whose mount namespace holds a mount of the clone's dataset,
  which is what the root scan reports (F23);
- a task whose working directory or open file lies inside a mount of the clone,
  which the scan does not report and which the handler's own walk finds (F25).

The second kind is not decoration.
F17 says a working directory or an open file makes a plain unmount refuse,
and F18 and F22 say a detached mount lives on while a reference remains;
VM item 4 measured a busy unmount caused by exactly that.
The match is by the mount id the kernel records,
not by a resolved path:
a descriptor's `/proc/<pid>/fdinfo` carries its mount id (F36),
and a working directory is read with `statx` and `STATX_MNT_ID` (F25).
Finding every task with any file open on the clone's copy,
or a working directory anywhere in it,
is new code over that record:
the wrapper's existing matcher names one file by mount id and inode,
not every file on a mount (N5).
A task under a child mount of the clone carries the child's mount id and is not matched by the copy's id;
the child mount is itself the blocker that no signal clears (see "The reach").
Where a read is refused,
the report says the walk was blind for that task.

Kind (d) of the brief,
a namespace whose task is gone,
is reachable under the chosen reach.
F13 measured that such a namespace lives on a descriptor;
F14 says a privileged caller in the host pid namespace can list every active namespace with `listns` and reopen one by its file handle;
F15 says entering it with `setns` must happen in a single-threaded process.
The reach section uses that route.

The scan's report is evidence that a namespace holds the clone.
It is never the operand of a signal.
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
5.  Only then does it end the task,
    under the chosen reach's rule that it is a blocker.

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
  a task of any account with a working directory or an open file inside the clone,
  which keeps the mount alive without mounting it (F17,
  F18).

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
3.  It scans before it moves anything and publishes the scan file.
    It finds kind (a) in one namespace,
    kinds (b) and (c) in another (the same root unit seen by task and by unit),
    and kind (e) in its own namespace,
    the staging folder.
4.  It compares every holder with the protected set.
    None is in it,
    so it acts:
    it ends the path blockers of its own namespace,
    then for every other holder namespace it unmounts the clone's copy and ends the tasks that block that unmount ("The reach").
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
    by namespace and task.
    The head destroy succeeds.
    A holder the act could not end,
    or a namespace it had to refuse,
    is the forced refusal (`holder-unendable`) and nothing is destroyed before the first delete.

### Options the answers did not choose

One place,
briefly,
with the record ([record][record],
"What was not chosen"):

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
  proceeding to delete while the holder is still seen,
  because it spends the clone's history to learn what the scan already said;
  and refusing while keeping the removal pending,
  because the accepted answer cancels,
  and the checked reattach returns a usable worktree.
- The reach:
  ending every task of each holder namespace,
  because it ends processes that held nothing;
  and that as a fallback after the unmount,
  because the answer says it never falls back.
- The protection:
  the caller's session alone,
  because it ends the harness;
  the session plus ancestors to the first session leader,
  because it misses the harness in every measured shape;
  and the same without the service-unit refusal,
  because a service unit's parent cannot be told apart from the requester.

The record's "What was not chosen" lists the same four sets.

## The reach: unmounting inside the holder's namespace

The third answer ([record][record],
"How much a forced removal ends in another mount namespace") chooses the unmount and ends only its blockers,
with no fallback to ending a namespace.
The record leaves the mechanism to this design and the measurement to a guest.
This section states the mechanism.

### What the act is aimed at

The aim is to release the dataset's long hold,
not to free the namespace (F20,
F21).
A plain unmount of the clone's copy inside a holder's namespace drops that namespace's reference (F22);
a lazy detach leaves the copy alive while a task holds it,
so it does not release the dataset (F18,
F22),
and the design never detaches.
A mount under the clone,
or a task whose working directory or open file references the copy,
makes the plain unmount refuse `EBUSY` (F17);
the first is a mount and no signal clears it,
the second is a blocker and is ended.
A second mount of the same dataset in the same namespace,
such as a bind of a folder inside the clone,
keeps the dataset's long hold on its own and is refused as a mount blocker.

### How the mount is named

A name inside the holder's namespace is a name an account process can change,
and the accepted staging design's answer is to unmount from a descriptor (F32;
[stagingdesign][stagingdesign],
"The unmount call from staging").
Its covered descriptor is opened before the clone is stacked on the folder,
which a holder's namespace does not offer.
This design makes the covered location itself,
and names nothing the holder controls:

1.  A single-threaded child enters the namespace:
    `setns` on a descriptor from `listns` or `/proc/<pid>/ns/mnt`,
    because `setns(CLONE_NEWNS)` fails `EINVAL` from a thread of a multithreaded process (F15).
    The child is a `fork` followed by `setns` in the child,
    so the handler's other threads never enter.
2.  The child reads its own `/proc/self/mountinfo` in that namespace and finds the copy:
    a row whose file-system type is `zfs`,
    whose source is exactly the clone's dataset name,
    and whose root is `/` (F34).
    The scan's match rule is the same one,
    so the two cannot disagree about what a mount of the dataset is.
3.  It checks the copy before naming it:
    the row is not the namespace's root (its parent id differs from its own,
    F34),
    it carries no `shared:` tag (F35),
    its parent row carries no `shared:` tag (F33 refuses a move from a shared parent),
    it carries no `MNT_LOCKED` (the call's `EINVAL` is the check,
    F31),
    and no other row of the namespace is a mount of the dataset.
4.  It opens the copy's mount point,
    crossing into the copy,
    with `O_PATH | O_DIRECTORY | O_CLOEXEC`,
    and requires `statx` with `STATX_MNT_ID` to give the row's mount id and `fstatfs` to give the ZFS type (F25,
    F36).
    The descriptor names the mount root;
    it is the only name the holder could have changed,
    and the mount id check is what binds it to the scan's row.
5.  It opens a destination it made itself:
    a directory created mode `0700` under a root-owned,
    writable directory of that namespace whose mount row carries no `shared:` tag,
    so the attach in the next step does not propagate to peers (F37).
    It opens that directory `O_PATH` before the move;
    that descriptor is the covered dentry the staging design's rule needs.
6.  It calls `move_mount(copy_fd, "", destination_fd, "", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH)`,
    which takes both ends from descriptors (F33).
    The copy now sits at the destination,
    and the copy's original path shows the parent mount again.
7.  It closes the copy's descriptor,
    because a descriptor on the copy would keep it busy for its own unmount (F17),
    and re-reads the mount table:
    the row with the copy's mount id is now at the destination's path,
    and it is the topmost mount at the destination dentry.
8.  It calls `umount2("/proc/self/fd/<destination fd>", 0)`,
    plain,
    with no `MNT_FORCE` and no `MNT_DETACH`.
    The descriptor is on the destination's mount,
    not on the copy,
    and the lookup steps into the topmost mount at its dentry (F31,
    F32).
9.  It re-reads the mount table:
    no row carries the copy's mount id,
    and no row of the dataset remains in that namespace.
    It then removes the destination directory.

The checks after each call are what make a substituted or stacked mount a refusal rather than an unmount of the wrong thing:
after the move the copy's mount id must be topmost at the destination,
and after the unmount it must be absent,
while the destination dentry must show the destination mount again.
A mount stacked on the destination between the move and the unmount is the one the call would remove;
the post-call check finds the copy still present and refuses,
and the copy is moved back (below).
The window is the same one the staging design records and does not close.

### What is refused, and what is reported

- A copy carrying `shared:`,
  or under a shared parent mount:
  refused.
  A plain unmount of a shared mount propagates to its peers and would take the copy out of namespaces the request never named (F35);
  the accepted design withholds exactly that path unmount ([proposal][proposal],
  lines 730 to 732).
- A mount under the copy:
  refused and reported as a mount blocker that no signal clears.
  The design never shrinks submounts and never detaches.
- A second mount of the dataset in the same namespace:
  refused and named,
  because the act unmounts the namespace's own-root copy and ending tasks cannot release another mount.
- A namespace whose root mount is the copy:
  refused,
  because the move needs a parent and there is none (F33).
- The handler's own namespace,
  and any namespace containing pid 1:
  refused.
  Its clone mount is the host's,
  and the removal's staging turn unmounts it without force;
  the act's only work there is ending path holders.
  A8 says the handler runs in the host mount namespace,
  so the two are the same namespace.
- A namespace that cannot be entered (`setns` refuses),
  or a destination that cannot be made (no root-owned writable directory on a private or slave mount):
  refused,
  naming the namespace and the step.
- A system service's namespace:
  entered and treated like any other holder namespace,
  because its task is another account's process and the forced answer ends those holders.
  The report names the unit from the blocking task's cgroup row.
  If the project wants services carved out,
  that is a narrower rule than the record states,
  and this design would take it as a veto.
- Every path holder of the handler's own namespace:
  ended,
  as the blockers of the host's unmount (VM item 4's case).

### When the unmount is refused after the move

The copy must not be left at the destination with nobody told.
If the move succeeded and the unmount is refused,
or a blocker appears after the blocker scan,
the child re-reads both locations and moves the copy back to its recorded mount point only when that path still resolves to the parent mount and carries no mount;
otherwise it leaves the copy at the destination,
names both paths and the mount id in the receipt,
and the forced removal refuses (`holder-unendable`).
The move back is the one recovery this design writes,
and it is unrun (M7).

### What the tasks that block an unmount are, and how they are found

A blocker is a task with a working directory or an open file whose mount id is the copy's (F36),
found by the handler's walk over `/proc` (F25;
new code,
N5).
A task under a child mount is not a blocker of this unmount:
the child mount is,
and it is refused.
A blocker in the protected set refuses the whole removal before anything is moved.
A blocker is ended through its pinned pidfd (the five steps above),
and the unmount is tried again.
A blocker that appears between the scan and the unmount is met by the same sequence once more within the sweep's bound;
a task that keeps forking cannot extend the act past that bound.

### The checks that exist only for a hostile account

The threat model is the project's:
the account is trusted,
and root and the account are not a security boundary here ([staging][staging],
lines 221 to 237).
These checks survive that sentence as guards against a bug or a stale record,
and are listed so no one reads them as a security boundary:

- the mount id check after the path open and after the move:
  it also catches a stale scan;
- the topmost and transition checks:
  they also catch the handler's own mistake;
- the destination descriptor and its `0700` root-owned directory:
  they also keep the handler's own unmount off a path it did not make;
- the wrapper's pidfd before each call:
  it guards a dead caller,
  not a hostile one;
- the dataset GUID re-read:
  it is the removal's existing delegated-grant residual,
  not a new boundary;
- the refusal of a shared mount:
  it also protects a cooperative account's other namespaces.

This check exists only against a hostile account and is not adopted:
requiring the copy's parent directory to be owned by root and not writable by the account.
The move takes the mount by descriptor after the mount id check,
so no component name is resolved after it,
and the check would add nothing the mount id does not.

## The act's mechanism

Whatever the reach does for mounts,
these hold for every act:

- The handler never signals a task in its own namespace except a path holder whose working directory or open file is in the clone.
- The handler refuses when any protected task is a holder,
  and a namespace's tasks are all enumerated,
  so a protected task anywhere in it refuses the namespace.
- The number of tasks and namespaces one act reaches is bounded by the scan's own bounds (F23),
  and a scan that stopped at its bound is not a clear answer.
- Every task is pinned and proved before it is signalled (the five steps in "What a holder is"),
  and a task whose proof fails is reported,
  not signalled.
- The wrapper's pidfd is checked immediately before each signal and each unmount,
  and it is in the poll set the act waits on (N2).
- The act's wait is the pidfd's exit (`POLLIN`) and,
  for an unmount,
  the kernel call's return,
  with the same pause the receipt wait uses.
  It is bounded by the sweep's own work bound,
  not by the wrapper's 30-second poll deadline (F24):
  the handler is not told that client-side number,
  and a dead wrapper ends the act.

The logging,
through the wrapper's existing notice stream on standard error and the handler's journal:

- one line per ended holder:
  the dataset GUID and request nonce,
  the namespace identity,
  the mount id,
  the task's number and birth identity,
  its session,
  whether it was in the protected set (it must not be),
  the act (signal or unmount),
  and the wait's outcome;
- one line per holder not ended:
  the same fields plus the failing proof;
- one line per refused namespace:
  the namespace,
  the mount row,
  and the refusal (shared,
  child mount,
  second mount,
  root mount,
  cannot enter,
  no destination,
  unmount refused);
- one closing line:
  how many holders were ended,
  how many were unproven,
  and whether the following scan was clear.

The wrapper prints the same facts in its report,
because the decision's report has to name who was ended.
It does not print them for a holder of the protected set;
that case has its own refusal text.

## The refusal, and what it leaves

Two refusals are the forced arm's,
and the receipt's failed step names which:

- `caller-protected-holds`:
  a holder task is in the protected set.
  The handler decides it on the pre-move scan,
  before anything moves.
- `holder-unendable`:
  a holder the act could not end,
  or a namespace the reach refused for a reason this design names (shared,
  child mount,
  second mount,
  root mount,
  cannot enter,
  no destination,
  unmount refused).
  It is decided at the same scan or at the unmount itself.

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
a forced holder that cannot be ended:

```text
the removal of <slot> was asked to end the holders and could not.
<the holder, its namespace and the reason: a root task this removal could not prove, a namespace with a shared copy, a mount under the clone, a second mount of the dataset, a namespace this removal could not enter, or an unmount that refused>.
Nothing was destroyed.
```

A holder first seen after the first snapshot delete is met at the next scan,
and the refusal then reports what was already destroyed;
that state is the sibling's,
under its stop points.
What the forced arm leaves:
no snapshot delete and no head destroy after the refusal point,
and the restored state,
the records,
the reattach and the nonce of the request that reverses the intent belong to [the sibling][cancelled].
This design states only the forced arm's refusal before the first delete;
the sibling owns what a removal leaves when it does not finish,
forced or not,
in its "The forced arm's failure path",
"One reversing request,
written in one place" and "The stop points".
It needs three things from this design,
which this design gives:
the writer's pid and start time in the force property (and now its parent's,
N2),
the two refusal words above,
and
the fact that this design never writes the intent.
The sibling's current text calls the first refusal `caller-session-holds` and spares the caller's session only;
its fix round owes the wider protected set and these two words.

## The time between the scan and the ending

A forced request must not act after the wrapper that made it is gone (H2).
The checks that settle it are:

- the force word is honoured only with a standing `unmount` intent whose nonce,
  the force property's and the caller property's all agree;
- the caller property's pid and start time are proved,
  and its recorded parent's pid and start time are proved,
  before the chain is walked (N2);
- the handler holds a pidfd on the wrapper and checks it immediately before each signal and each unmount,
  and in the act's poll set;
- the handler does not read the account-owned removal claim.
  The staging design records that no channel from the claim to the handler exists and that the project chose not to add one ([stagingdesign][stagingdesign],
  "The unmount call from staging");
  the review's earlier suggestion that this check "is not implementable without moving that record under root" is too strong,
  and the design says instead that root could read it and the project chose not to.

A later trigger that finds a forced request whose wrapper is gone declines the force word,
files `caller-gone`,
and does only the non-force work the intent asks:
it may unmount as any removal would,
and it never ends a process.

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
the accepted residual for a holder the scan cannot show.
The retry is bounded by the removal deadline;
nothing loops until the pool is quiet.

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
a taskless pinned namespace is reached under the chosen reach through `listns` (F14);
a task that survives the act's wait is reported,
not chased.

## Where the records differ, and how this design reads them

- The scan before the move.
  The accepted design's removal-ready scan comes after the move and the unmount,
  and the proposal asks for each scan through a new `unmount` intent.
  This design adds a scan under that same intent before the handler moves anything,
  so the protected-set refusal and the first act come before a snapshot is lost.
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
  forced or not (its "The forced arm's failure path");
  this design states only the forced arm's refusal before the first delete,
  with the two words,
  and writes no intent and no restored state.
  The sibling's current text still reads the forced failure path as two cases and names the first with its old word;
  its fix round reconciles the words and the wider protected set.

## Choices this design marks for the project's veto

Each entry is a writer's choice the design adopts so it is whole.
A veto takes the other option and the sentences that say otherwise.
Each gives its options,
a concrete example,
pros and cons,
and a ranking with the reason for each adjacent pair.

### V1. Where the scan's finding is carried

Options:
the scan's own text in a second root-owned file beside the receipt (adopted);
the text inside the receipt;
the wrapper runs the scan mode on demand through a privileged route;
the wrapper never learns and root acts alone.

Concrete example:
a clone at `.claude/worktrees/one` held in `mnt:[4026532396]` by `/proc/127996`.
Under the adopted option the person's report names the namespace,
the task and the mount id from the receipt's nonce,
and `doctor` prints the same row.
Under the in-receipt option every existing receipt reader gains an optional multi-line blob.
Under the on-demand option there is no scan file for `doctor` to read.
Under the outcome-only option the refusal cannot be printed from the holder's own fields.

Pros of the adopted option:
reuses the scan's parser and bound,
the receipt folder and the atomic publish;
leaves the mount protocol's grammar alone;
gives step 16 and step 18 the same artifact.
Cons:
two files must agree on the nonce,
and a missing scan file is a failure the wrapper must report rather than read as clear.
Ranking:
beside the receipt > inside the receipt > on demand > outcome only,
because the first keeps both grammars and both readers unchanged,
the second changes every receipt reader,
the third needs a privileged route,
and the last loses the names the decision's report owes.

### V2. What a receipt without its scan file means

Options:
a refusal of the removal at the step that needed the scan (adopted);
an `unknown` that does not refuse;
a retry of the scan before going on.

Concrete example:
the wrapper asks for the scan before the first snapshot delete and the receipt arrives with no scan file.
Under the adopted option the removal refuses,
prints that the finding never arrived,
and destroys nothing.
Under the second it treats the missing file as an incomplete read and goes on to delete.
Under the third a handler that cannot publish will fail again.

Pros:
nothing is destroyed on an answer the wrapper never received.
Cons:
a handler fault stops a removal that might have been safe.
Ranking:
refuse > retry > unknown-and-proceed,
because a missing answer is not the incomplete read the destroy-anyway decision covers,
and retry buys nothing a second failure would not.

### V3. How the caller's identity and the force word reach the handler

Options:
two more clone-local properties,
each bound to the intent's nonce (adopted),
one carrying the wrapper's pid,
start time,
recorded parent and boot and the other the force word;
a longer intent grammar;
a single property carrying a request record.

Concrete example:
`--force` on the clone above.
The adopted option writes `v1:<nonce>:<pid>:<starttime>:<parent-pid>:<parent-starttime>:<boot>` and `v1:<nonce>:force` beside the existing `v1:<nonce>:unmount`.
The longer grammar writes everything into the one property,
and every reader of the intent must grow with it.
The single-record option writes a third property holding a structured request.

Pros of the adopted option:
the accepted intent grammar and its bound stay as they are,
each value is bounded alone,
and a mismatched nonce is one comparison.
Cons:
three properties must be read consistently,
which the nonce makes checkable.
Ranking:
two properties > longer intent > request record,
because the first changes no accepted grammar,
the second does,
and the third is a new protocol shape with no measured need.

### V4. When the forced arm acts

Options:
at every scan under the force word,
beginning with a pre-move scan (adopted);
only at the scans step 16 requires,
which run after the move;
only once,
at the first scan.

Concrete example:
a holder appears after the unmount but before the snapshot batch.
The adopted option ends it at the pre-delete scan,
before any snapshot is lost.
The second option does the same at that scan.
The third lets it reach the head destroy,
and the history is gone before the failure is known.
Ranking:
every scan > step 16's scans only > once,
because the first also refuses before the move when the protected set holds,
and the last trades history for nothing.

### V5. How the mount is named and unmounted inside a holder's namespace

Options:
a descriptor on the copy,
moved to a root-owned destination the handler made and unmounted from the destination's descriptor (adopted);
a name under a descriptor on the mount point's parent directory (the staging design's "name under the descriptor" arm,
rejected);
a covered descriptor from before the mount (not available after the fact);
a path unmount (withheld by proposal lines 730 to 732);
a lazy detach (refused,
F18,
F22).

Concrete example:
a `PrivateMounts=yes` unit holds a copy at `/srv/x` in `mnt:[4026532397]`.
The adopted option opens `/srv/x` once,
proves its mount id,
moves the mount by descriptor to a `0700` directory the handler made under a private mount,
and unmounts from that directory's descriptor;
the holder sees its copy vanish and nothing it controls was named twice.
The parent-name option unmounts `/proc/self/fd/<parent>/x`,
and a rename of `x` steers the call.
The path unmount is what the shared control did and what the accepted design withholds.
The lazy detach reports success and leaves the dataset busy.

Pros of the adopted option:
both ends of the move are descriptors,
the only account-changeable name is checked against the mount id before it is used,
and the unmount argument is a dentry the handler made.
Cons:
it needs `move_mount` and a private destination mount,
it changes the holder's tree for a moment,
and it is unrun.
Ranking:
move by descriptor > parent-name > path > lazy detach,
because the first keeps every name after the check out of the call,
the second resolves a component the holder can change,
the third is the withheld path unmount,
and the last does not release the dataset.
The project confirms this after M3 to M7.

### V6. Whether a refused unmount after the move is moved back

Options:
move the copy back when its recorded path is still the same parent with no mount (adopted);
leave it at the destination and report both paths;
detach it.

Concrete example:
a blocker appears after the move and the unmount refuses.
The adopted option puts the copy back where the holder had it and reports the refusal.
The second leaves the holder's path empty and names the destination.
The third is the lazy detach the design refuses.
Ranking:
move back > leave and report > detach,
because the first restores what the act found,
the second leaves the holder's tree changed,
and the third does not release the dataset.

## Questions only the project can answer

The record closed S1,
S2,
S3 and S5.
S7 to S10 belong to the sibling design and are not repeated here.
These three are still open,
numbered as the review numbers them.

### S4. The signal, and any grace before it

In words:
"When the forced arm ends a process,
may it give the process a chance to save and exit first?"

Options:
`SIGKILL` alone;
`SIGTERM`,
wait on the pidfd,
then `SIGKILL`;
`SIGTERM` alone.

Concrete case:
an editor with unsaved buffers whose working directory is in the clone.
`SIGKILL` loses them.
`SIGTERM` lets it save and exit,
and the removal then waits for that.
`SIGTERM` alone cannot promise the holder ends.

Pros and cons:
`SIGKILL` ships no number and cannot be refused,
and loses state;
`SIGTERM` then `SIGKILL` gives a holder its chance and needs a grace the project must measure or decide (L9);
`SIGTERM` alone can fail to end the holder.
Ranking:
`SIGTERM` then `SIGKILL` > `SIGKILL` > `SIGTERM` alone,
because the cost of an unmeasured grace falls on the removal's time while the cost of none falls on a person's data,
and the third can leave the holder alive.
A grace period ships no number:
it is owed to a measurement or to the project.

### S6. How long a forced request's authority lasts

In words:
"If the person gives up on a forced removal (Ctrl-C),
may a later trigger still end processes for it?"

Options:
(a) only while the wrapper that wrote it is alive,
by pid,
start time and its recorded parent (the design's implementation);
(b) as long as the nonce matches and the caller's session exists;
(c) for a fixed time.

Concrete case:
Ctrl-C at the twentieth second;
a ZED event at ten minutes.
(a) ends nothing and files `caller-gone`;
(b) ends whatever attached since;
(c) depends on the number.

Pros and cons:
(a) matches what the person asked and still lets the standing request do harmless mount work,
and needs the liveness check and a receipt word;
(b) is cheaper and lets a later trigger finish a removal the person abandoned with a live session;
(c) ships an unmeasured number.
Ranking:
a > b > c,
because the person who interrupted a forced removal is the person whose later processes would be ended,
and b ships no number where c does.
The design implements (a),
and the project should confirm it.

### S11. Whether a forced ending leaves a durable record beyond the journal

In words:
"Should the plan record remember who a forced removal ended?"

Options:
the handler's journal and the wrapper's standard error;
an entry in the removal's plan record;
a record of its own per slot.

Concrete case:
a week after a forced removal a person asks who was ended.
The journal may have rotated;
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
  as work in flight.
  This design asks that build to record the representative task's start time (F23 has none),
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
  the pin-first proof,
  the reach (the entering child,
  the move and the unmount,
  the blocker matcher),
  the logging and the tests.
- Step 18 builds the `doctor` reading:
  the last scan file,
  the live scan mode,
  the holders named by namespace and task,
  and the protected set the removal spares.
- Step 19 installs nothing new:
  the chosen channel adds no sudoers line and no helper.
- No step builds a second scan,
  a fallback kill-all,
  a lazy detach,
  or a forced unmount of the host's mount.

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
3.  The protected-set refusal for a working directory.
    Start a holder in the wrapper's session,
    wait for its readiness on a pipe,
    run the removal with `--force`,
    and assert the first refusal text,
    the clone and its snapshots,
    the registration,
    and the holder alive.
    Guest.
4.  The protected-set refusal for a namespace.
    Start a holder in a mount namespace of the wrapper's session (a `PrivateMounts=yes` unit started by that session),
    run with `--force`,
    and assert the second refusal text names the namespace and says leaving the folder does not release it.
    Guest.
5.  Another session's holder ends while the caller lives.
    Start the holder in a new session,
    wait on a pipe,
    run the removal,
    and assert the holder's wait status,
    the caller alive,
    a second job of the caller's session alive,
    and the head destroy done.
    Guest.
6.  The ancestor half is spared.
    Start the removal from a child of a process that holds the clone but is not in the wrapper's session (the harness shape,
    F4),
    and assert the refusal names that ancestor and it stays alive.
    Device-free with fixture `/proc` for the comparison;
    guest for the real removal.
7.  A holder that appears between the scan and the act is met by the next one.
    Force the interleaving with an injected hook between the scan and the act,
    of the shape `take_claim_seamed` gives the claim (`lock/single_owner_claim.rs:362`),
    not with a watch on the results folder.
    Guest.
8.  A receipt without its scan file refuses at a destroy step.
    Publish the receipt and no scan file,
    and assert the removal refuses before the snapshot delete.
    Device-free.
9.  A stale or absent caller property declines the force.
    Send the force word with an absent caller property,
    a mismatched nonce,
    a dead pid and start time,
    and a wrapper whose recorded parent is not its live parent.
    Assert no signal,
    the receipt says `caller-gone`,
    and only the non-force work happens.
    Device-free.
10. The wrapper ends between two passes.
    Kill the wrapper at the injected hook after the first pass and before the second,
    and assert the second pass sends no signal and no unmount,
    the receipt says `caller-gone`,
    and the half-done state is reported.
    This is the review's N2 test.
    Guest.
11. The service-manager parent refuses `--force`.
    Run the removal as a `systemd-run --user --pipe --wait` unit,
    assert exit `2`,
    the refusal text,
    no property written,
    and nothing changed.
    Guest.
12. The reach unmounts a copy and ends only its blockers.
    A `PrivateMounts=yes` unit holds a copy;
    a web server that never touched the clone and a helper whose working directory is in it run in that unit.
    Run with `--force`,
    and assert the copy is gone,
    the helper is dead,
    the web server runs,
    and the head destroy succeeds.
    Guest.
13. The reach refuses what it cannot take.
    Five arms,
    each asserting the named refusal and nothing destroyed:
    a `shared:` copy;
    a mount under the copy;
    a second mount of the dataset in the namespace;
    a namespace whose root is the copy;
    a wrapper killed after the move.
    Guest.
14. A taskless namespace.
    Pin a namespace that mounts the clone,
    end its task,
    run with `--force`,
    and assert the handler lists it (F14),
    enters a single-threaded child,
    unmounts inside it,
    and the head destroy succeeds.
    Guest.
15. The moved copy goes back when the unmount refuses.
    Stack a mount on the destination between the move and the unmount (the injected hook),
    and assert the copy is back at its recorded path,
    the receipt names both,
    and nothing else moved.
    Guest.
16. `doctor` names holders and the protected set without root.
    Give `doctor` a scan file and a live scan mode answer for a clone held in a namespace,
    and assert the namespace,
    task,
    mount id and path appear,
    that the protected set is named,
    and that `doctor` never signals or unmounts.
    Device-free.

## Measurements owed

M1 to M13 are the arms of one guest run,
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
Each arm records the receipt,
the scan file,
the wrapper's standard error,
`doctor`'s rows,
the journal lines,
and the state of the clone,
its snapshots and the registration after it.
A run that cannot show the arm is recorded as unrun,
never as passed.
No value from any arm ships until the arm runs.

- M1.
  State:
  a `PrivateMounts=yes` unit,
  started outside the protected set,
  holds a copy of the clone;
  a second reader namespace holds another copy;
  neither has a task using the copy.
  Action:
  `git worktree remove --force <slot>` once.
  Expected:
  the handler enters each namespace,
  moves the copy and unmounts it;
  both copies are gone from their namespaces;
  both readers are still alive and see the mount vanish;
  the head destroy exits 0 and the dataset is gone from `zfs list`.
  Bound:
  the child that enters and unmounts returns before the sweep's own work bound;
  the record notes the elapsed time and whether the child was killed at the bound.
  Control:
  the same arm without `--force`,
  recording that the removal cancels,
  both copies stay,
  nothing is destroyed,
  and the clone is back at its slot.

- M2.
  State:
  the same two readers,
  but both started by the removal's own session;
  one holds a working directory in the clone,
  one holds a mount namespace.
  Action:
  one `git worktree remove --force <slot>`.
  Expected:
  the two refusal texts;
  both readers alive;
  the mount at its slot;
  the snapshots and registration whole;
  the receipt's `caller-protected-holds` step;
  no destroy request.
  Bound:
  the refusal is at the pre-move scan;
  the record notes that no signal and no unmount call ran.

- M3.
  State:
  a holder namespace with a copy at `/srv/x`,
  no task using it,
  and a private destination available in that namespace.
  Action:
  the handler's reach enters the namespace,
  moves the copy to the destination and unmounts it.
  Expected:
  the copy's mount id is absent from that namespace;
  the destination dentry shows the destination mount;
  the destination directory is removed;
  the head destroy succeeds;
  the holder's original path shows the parent mount.
  Bound:
  the child's calls return;
  the record notes each call's elapsed time and the sweep bound.
  Control:
  the same arm with the move omitted and a path `umount` used only in a disposable decoy,
  recording what the holder sees and why the design does not use it.

- M4.
  State:
  M3's namespace,
  plus a web server that never opened the clone and a helper whose working directory is in the copy.
  Action:
  one `--force` removal.
  Expected:
  the helper is found by the copy's mount id,
  ended through its pidfd,
  and its wait status is recorded;
  the web server is alive;
  the unmount and the head destroy succeed.
  Bound:
  same as M3.

- M5.
  State:
  M3's namespace plus a task with an open file on the copy (not a working directory),
  and a task whose working directory is under a child mount of the copy.
  Action:
  one `--force` removal.
  Expected:
  the open-file task is found and ended;
  the child mount is refused,
  reported,
  and no signal is sent for the task under it;
  the removal refuses `holder-unendable` and nothing is destroyed.
  Bound:
  the refusal is before the first delete;
  the record notes the blocker scan's cost.

- M6.
  State:
  a `shared:` copy,
  and separately a copy under a shared parent mount;
  a copy with a bind of a folder inside the clone at another path;
  a namespace whose root mount is the copy.
  Action:
  one `--force` removal for each state.
  Expected:
  each is refused with its own named reason;
  no move and no unmount ran;
  the copies stay;
  nothing is destroyed.
  Bound:
  the refusal needs no wait;
  the record notes which check fired.

- M7.
  State:
  M3's namespace,
  with a taskless namespace pinned by a descriptor,
  and separately a system service's namespace holding a copy.
  Action:
  one `--force` removal for each.
  Expected:
  `listns` finds the taskless namespace,
  the child enters it,
  unmounts,
  and the head destroy succeeds;
  the system service's namespace is entered,
  unmounted,
  its blocker (if any) ended,
  and the report names the unit.
  Bound:
  as M3.

- M8.
  State:
  the authority answers this host could not be asked,
  and the arm the sibling design shares:
  an unprivileged account with no authentication agent.
  Action:
  `kill` and `pidfd_send_signal` on a root process;
  `systemctl --user --no-ask-password kill --signal=SIGKILL` on its own transient unit;
  `systemctl --no-ask-password kill --signal=SIGKILL` on a system unit it does not own.
  Expected:
  `EPERM`;
  the own-unit kill works;
  the system-unit request is refused for want of authorization (F29).
  Bound:
  the system-manager request returns without an agent;
  the record notes the elapsed time.

- M9.
  State:
  a wrapper that is killed after the move and before the unmount;
  and one killed between two passes.
  Action:
  the next sweep finds the standing request.
  Expected:
  the act stops,
  files `caller-gone`,
  moves the copy back when it can,
  and ends nothing.
  Bound:
  the sweep's bound;
  the record notes the state left.

- M10.
  State:
  a rename of the copy's mount point between the mount-table read and the open;
  a mount stacked on the destination between the move and the unmount.
  Action:
  one `--force` removal for each.
  Expected:
  the mount id check refuses the first;
  the post-move check refuses the second and the copy goes back;
  nothing is unmounted but the handler's own destination.
  Bound:
  no wait beyond the calls.

- M11.
  State:
  the service-unit shape of the fourth answer.
  Action:
  start the removal as a `systemd-run --user --pipe --wait` unit from a process that holds the clone.
  Expected:
  `--force` is refused with the text,
  exit 2,
  and nothing changed;
  the same removal run as a child of that process is allowed and spares it.
  Bound:
  the refusal is before any property write.

- M12.
  State:
  the scan file's latency and the act's cost.
  Action:
  time the publish to the wrapper's read,
  and time an unmount inside one namespace and the ending of one blocker.
  Expected:
  the numbers step 22's removal deadline needs.
  Bound:
  no value ships before this runs.

- M13.
  State:
  a namespace whose destination cannot be made:
  no root-owned writable directory on a private or slave mount.
  Action:
  one `--force` removal.
  Expected:
  refused,
  naming the namespace and the step;
  nothing moved.
  Bound:
  no wait beyond the calls.

## Limitations

- Every claim about the dataset becoming destroyable after a plain unmount or after ending a blocker is an inference from source and from VM items 4 and 15 (L1).
- The caller's protected set is a number inside one pid namespace (A1,
  L4);
  a removal inside a container is not covered and probably cannot reach `/dev/zfs` at all.
- The caller property is an assertion the handler ties to a live wrapper and its recorded parent,
  not to a sender (A4);
  the liveness check keeps it from being stale,
  not from being wrong about which process is the caller.
- A confined unit the Pi driver starts is not an ancestor and can be ended if it holds the clone (L13).
- Kind (d) is reached through `listns` or a namespace file handle,
  which is source-read and unrun (F14,
  L12).
- The reach,
  its move and its unmount are unrun;
  M3 to M7 and M13 are the arms that would measure them.
- The blocker matcher is new code (N5,
  L14).
- The protected-set refusal is unmeasured (M2),
  and the system-manager answers are unverified on this host (L5,
  L11).
- The handler's own work bound is the sweep's,
  not a number this design ships (L9);
  its value is not measured here.
- The plan record has no field for "who was ended" today,
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
  and any unmount of the host's clone mount by the forced arm.
- A namespace the reach refuses for a reason this design names,
  beyond the refusal and the report:
  no recovery is designed for a copy left at a destination when moving it back also fails.
- The handler's installation,
  its unit hardening and ZED's own timeouts;
  whether a kill-and-wait inside a zedlet child is bounded by ZED is a question for step 19,
  not answered here.
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
  [The sibling cancelled-removal design][cancelled] at `2fb7caea17` owns what a removal leaves and restores when it does not finish,
  forced or not:
  its "The stop points",
  "The forced arm's failure path",
  "What the caller sees" and "One reversing request,
  written in one place".
  This design states only the forced arm's refusal before the first delete,
  gives it the two words `caller-protected-holds` and `holder-unendable`,
  and writes no intent.
  The sibling's current text names the first refusal with its old word and the caller's session only;
  its fix round owes the wider protected set and the two words.
  It also adds a sixteenth receipt key,
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
  and then it needs a field for who was ended,
  which that document does not have.

- D5.
  [The lost-last-event triggers][losttriggers] own nudge merging and the rate rule.
  This design adds no nudge;
  the forced request is the intent plus two properties,
  and the merge rule does not read them.

- D6.
  [The propagation after a replacement mount][propagation] states "no dependency" on this design and leaves the holder scan and the forced arm to this document.
  This design keeps the scan's visibility unchanged from VM item 15 and needs nothing from that design.

- D7.
  The chosen channel adds no sudoers line and no helper.
  The reach adds a single-threaded entering child to the handler,
  which is step 17's build,
  not a new installation.

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

The four answers are built in:
"Who ends each kind of holder" is the chosen channel,
"The reach" is the unmount and its blockers,
"The caller and its protected set" is the session plus every ancestor to pid 1 and the service-manager refusal,
and "The refusal" carries the force-only refusal and leaves the unforced cancel to the sibling.
N2 is fixed in "The caller and its protected set" and "The time between the scan and the ending";
N3 in "A command whose parent is the service manager";
N4 by "The reach" and its M3 to M7 and M13;
N5 in "What a holder is".

## What this revision left as it was

The sections "The open items and the words that decide them" apart from the record's four quotations,
"What this design read" apart from the record,
the sibling and the staging design,
the kernel and OpenZFS facts F7 to F30 apart from the rewrite of F8 and F10,
the reference definitions,
and the route-level exclusions of "What this design does not cover" were left as they were,
with the corrections the review names.

[decision]: ../decision/cli-git-zfs-worktree-design.md
[proposal]: cli-git-zfs-worktree-design-proposal.md
[plan]: cli-git-zfs-worktree-implementation-plan.md
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
