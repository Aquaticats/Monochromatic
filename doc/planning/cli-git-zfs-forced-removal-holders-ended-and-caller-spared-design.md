# The forced arm of a removal: which holders are ended, by whom, and how the caller's session is spared

Non-final,
unapproved design proposal by an agent,
written 2026-11-22T17:29:53-05:00 and revised 2026-11-22 after the gate review
(`~/temp/agent/gate-review-zfs-forced-removal-and-cancellation-designs-2026-11-22/keep/review.md`,
read whole).
It is not approved by the project owner and nothing in it is final.
It designs two open items of the accepted ZFS worktree design that are one mechanism:
"Who ends a holder,
how a holder is identified,
and by what authority",
and "How the wrapper tells its caller's session apart from the other holders".
It makes no decision that the accepted design,
its answers of 2026-11-19,
or the earlier answers made,
beyond the writer's choices that "Choices this design marks for the project's veto" lists.
The authority over another account's process,
how much a forced removal ends,
the signal,
how long a forced request's authority lasts,
and what the caller's protection covers stay the project's questions,
as the build plan puts it at lines 2787 to 2798 of the plan.
Every sentence that rests on the forced arm working is conditional on the guest measurement this round did not run.

What this revision rewrote,
after the review:
the caller and its protected set (H1,
Q6),
the request's liveness (H2),
the identification proof (H3),
the act and its owner question (H4,
Q2),
the four authority options (M1,
M2),
the refusal (M3,
M4),
the scan file's publication and lifetime (M5),
the fact and probe account (M6),
the ancestor citation (M7),
the questions moved from the veto list (M8),
the tests and measurement labels (M9),
and the low findings L1 to L4.
"The open items and the words that decide them",
"What this design read" (except the additions),
the facts about kernels and ZFS not listed above,
and the reference material at the end were left as they were apart from the corrections named.

What it read:
the decision record whole,
the proposal's sections named in "What this design read",
the plan's step 17 and its open-item list,
`package/git-policy/cli/SPEC.md` around the mount protocol,
the receipts,
the never-prompt rule and the mount deadline,
the code paths named in the facts,
the step 9 holder-scan and unmount branches as work in flight,
the nine sibling design documents on their branches,
upstream Linux at `v7.3`,
OpenZFS at `zfs-2.4.4`,
systemd at `v262`,
and sudo at `v1.9.17p2`.
What it ran:
five host probes,
recorded in
`/home/user/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/keep/host-probes-2026-11-22.md`,
and the two system-manager attempts that incident log records (L5).
What it did not run:
no guest,
no virtual machine,
no container,
no ZFS command that changes anything,
no `sudo`,
no mount on the host,
and no signal to a process it did not start.
One probe asked the system service manager to act,
twice,
against the rule that nothing does;
both attempts and what they printed are in L5 and in the keep log,
and neither is evidence.

The design is [the decision record][decision],
the whole design is [the proposal][proposal],
the build order is [the plan][plan],
and the measured facts the route already has are [the measurement record][measurements].
The sibling that owns what a removal leaves when it does not finish is [the cancelled or killed removal][cancelled].

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
> and when the caller's session holds the clone the removal refuses and says to leave the folder and run the
> command again;
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
> The removal never ends the caller and the processes of its session,
> the shell that ran the command and that shell's other jobs.
> When the caller's session holds the clone,
> the removal refuses and says to leave the folder and run the command again.
> Other holders are still ended,
> as "Kill or otherwise remove or destroy the holders whenever --force is specified" says.

The forced arm the decision adopted ([proposal][proposal],
lines 1357 to 1374),
in the entry's own words:

> So `--force` gains a second meaning beside waiving cleanliness:
> the holders are ended.
> Without it,
> a blocked removal reverts by itself.

The same entry names the measurement the adoption rests on:
"The second part of the answer is a measurement,
"Measure in a VM (Recommended)",
and the forced arm is adopted only if it works."

The measurement the build owes,
quoted whole ([proposal][proposal],
lines 1950 to 1960):

> - Forcing the removal of a clone a holder keeps.
> The cancellation decision's second part is "Measure in a VM (Recommended)",
> with the rule adopted only if it works.
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
> If the holder survives or the head destroy still refuses `dataset is busy`,
> the answer has nothing behind it,
> and this document reads that as the rule going back to the owner rather than shipping;
> the record itself says only "adopted only if it works".
> A force-unmount result would change two statements in this file:
> the busy-teardown rule that reports which clone holds the mount and never forces or detaches it,
> and the staging step's unmount without force or lazy detach.

What the plan says the authority is not a writer's to decide ([plan][plan],
lines 2787 to 2798):

> - "Who ends a holder,
> how a holder is identified,
> and by what authority":
> step 17,
> blocking,
> mixed.
> Identification and the receiving path are writer work;
> by what authority the wrapper ends another account's process or a unit's task is the owner's question.
> - "How the wrapper tells its caller's session apart from the other holders":
> step 17,
> blocking,
> writer design work.

What the proposal says of an unmount inside a holder's namespace,
which Q2 needs ([proposal][proposal],
lines 730 to 732):

> With a shared checkout,
> descriptor move refused with `EINVAL`;
> the positive control used a path unmount solely to exercise propagation.
> It removed the peer and slave copies while both readers were alive,
> and head destroy succeeded.
> This does not authorize path unmount in the design.

## What this design read

- The decision whole,
  [decision][decision],
  and the proposal's "Blocked-removal cancellation" (lines 1357 to 1374),
  "Mounts held by other namespaces" (lines 724 to 781),
  "Removal when the namespace exclusion cannot be shown" (lines 1306 to 1338),
  "Requests,
  results and ordering" (lines 552 to 596),
  "Removal and recovery" (lines 796 to 976),
  "Cancelling a blocked removal" (lines 977 to 1023),
  "What doctor checks and prints" (lines 1188 to 1258),
  "Decided defaults" (lines 1468 to 1512),
  "Clone removal identity protection" (lines 1513 to 1603),
  "Busy unmount after a staging move" (lines 1436 to 1443),
  and "Forced removal and deferred origins" (lines 1946 to 1971).

- The plan's step 17 (lines 1949 to 2015),
  its open-item list (lines 2770 to 2856),
  and its gaps list (lines 2858 to 2895).

- `package/git-policy/cli/SPEC.md` at "The wrapper's side and its deadline" (lines 6390 to 6420),
  the never-prompt rule (lines 7345 to 7355),
  and the receipts the unmount files (the step 9 branch's section).

- The code the route already has:
  `clone_holder_scan` and `mount_namespace_scan` on the step 9 branch,
  `worktree_clone_mount_protocol` receipts and intent,
  `worktree_clone_unmount`'s busy answer,
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

- The sibling documents:
  [the cancelled or killed removal][cancelled] whole for the state it owns,
  [the deferred-destroy checks][deferred]'s ownership of step 16 and its dependency entry,
  [the destruction plan record][planrecord]'s fields and readers,
  [the lost-last-event triggers][losttriggers]'s nudge merge rule,
  and [the propagation after a replacement mount][propagation]'s no-dependency statement.

- Upstream Linux `v7.3` in
  `/home/user/temp/agent/zfs-forced-removal-holders-ended-and-caller-spared-design-2026-11-22/scratch/linux-v7.3`,
  OpenZFS `zfs-2.4.4`,
  systemd `v262`,
  and sudo `v1.9.17p2` beside it,
  each at a named file and line below.

- The step 9 holder-scan branch (`feat/git-policy-cli-zfs-step-9-holder-scan-across-mount-namespaces`) and the
  step 9 unmount branch as work in flight.
  This design designs no scan and no unmount turn.

## What this design ran

Five probes,
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
    It was rewritten this round:
    no line can ask the system service manager to act,
    and every `systemctl` it runs carries `--user` and `--no-ask-password` and names only a unit it started.

5.  Two inline Python arms wrote `cgroup.kill` in a cgroup this account owns,
    and created a child cgroup.

The keep log also records two attempts that asked the system service manager to act,
made before the rule reached this run:
`systemctl kill --signal=0 dbus.service` (arm 2 of the first `authority-probe.py`,
stopped by `systemctl`'s own argument check before any request left the process) and the later
`systemctl kill --signal=KILL systemd-random-seed.service` (L5).
Neither is evidence;
both are named where a reader meets them.

No program ran inside a guest.
No ZFS command ran on the host.
Nothing mounted in the host's own namespace.

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
  (Measured,
  probe 2;
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
  and the lookup at `:552` to `:554`.)

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
  lines 1352 to 1358).
  Root's liveness proof is the wrapper process,
  not the claim.

## Limits

- L1.
  No guest ran this round.
  Every sentence about ZFS releasing the dataset after a plain unmount or after a kill,
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
  Earlier in this round the first `authority-probe.py` ran,
  as its arm 2,
  `systemctl kill --signal=0 dbus.service` (system manager,
  a real system unit,
  no `--user`,
  no `--no-ask-password`);
  it printed `Failed to parse signal string "0"` and never reached the manager.
  At about 2026-11-22T17:24-05:00 this round ran `systemctl kill --signal=KILL systemd-random-seed.service`;
  it printed `Failed to kill unit systemd-random-seed.service: Connection timed out` after putting a password
  dialog on the person's desktop,
  and the unit stayed `active` with `MainPID` 0.
  The design takes the polkit facts from source (F28,
  F29),
  and the run that would show what such a request answers belongs in a guest (M8).

- L6.
  No task in an uninterruptible sleep was available to probe,
  so the case of a holder the kernel cannot stop is unmeasured.

- L7.
  A mount namespace's inode number and a `listns` id can be reused after the namespace dies,
  so the design never rests on the number alone.

- L8.
  The forced arm's cost is not measured:
  how long a kill,
  a plain unmount inside a namespace,
  and their waits take,
  and how often a holder reappears,
  are owed to the guest.

- L9.
  No number is shipped by this design.
  The protection's stop is pid 1 (F26),
  the handler's act is bounded by the sweep's own work bound,
  and no grace or retry count is proposed.

- L10.
  A caller on the far side of a service unit is not an ancestor (F5),
  so the outer process that started the unit is not in the protected set.
  The design says so and recommends that a harness run the removal as a child of the harness process,
  as Pi's driver does.

- L11.
  `/etc/polkit-1/rules.d` is unreadable to this account,
  so local polkit rules are unread and what `systemctl kill` answers here is unverified (F29).

- L12.
  `listns`,
  `open_by_handle_at` on a namespace file handle,
  and `PIDFD_GET_MNT_NAMESPACE` were read at source and not run this round (F10,
  F14);
  M11 owes the first two a guest arm.

- L13.
  A confined unit the Pi driver starts is not an ancestor of the wrapper,
  so `--force` can end it if it holds the clone.
  "What this design does not cover" states it.

## The caller and its protected set

The caller is the wrapper's own process.
The wrapper is the `git` binary here (F27),
so the command a person or a harness ran is that process,
and the wrapper needs no environment variable,
no argument and no configuration to know it.
Its birth identity is the start time the kernel recorded for it,
the value `process_identity` already computes,
and with the boot id it names one process across a number's reuse.

The request carries that identity,
and nothing else about the caller,
in a clone-local property written before the intent:

- `org.monochromatic:git-worktree-clone-removal-caller`,
  value `v1:<nonce>:<pid>:<starttime>:<boot>`.
  The nonce is the intent's 32 lowercase hex digits,
  the pid and start time are decimal,
  and the boot id is the kernel's own.
  The byte length is bounded from those fields before parsing,
  and a malformed value sets nothing.

The handler reads the property only for a request whose intent and force property carry the same nonce (V3),
reads `/proc/<pid>/stat`,
and requires its start time to equal the value.
When it does not,
the wrapper that asked is gone,
and the handler declines the force word (see "The time between the scan and the ending").
Root therefore needs no ancestor list in the property:
it walks the chain itself from the wrapper's pid,
so nothing the wrapper would have had to serialise can be stale or truncated.
The review's narrowest fix for H1 proposed carrying each chain entry in the property;
this design carries the wrapper's identity and lets the handler read the live chain,
because a serialised list can only age and the handler reads the same `/proc` the wrapper would have.

The protected set,
which the handler spares and never ends,
is two things:

- every task whose `session` field equals the wrapper's session (F11),
  which is the decision's "the caller and the processes of its session";

- every ancestor of the wrapper from its parent to pid 1,
  which the handler walks with the same rule as `chain_ancestors` (F26:
  its round bound is that walk's existing `MOST_ANCESTORS_ASKED_FOR_THE_RECORD`,
  its stop is pid 1,
  and no number is new;
  the walk from another process's pid is new code over that rule),
  so a harness,
  a terminal multiplexer or a terminal owner above the wrapper is spared even when it is in another session.

The ancestor half is new in this revision and is what H1 found missing:
the earlier text spared ancestors on the wrapper's side and never told the handler,
so the handler ended them like any other holder.
The reader now reads the ancestors itself,
from the only number it is given.

The four shapes the brief and the review name,
worked through:

- Pi,
  a command the driver spawns (F1).
  The wrapper is its own session leader and the driver,
  its parent,
  is in another session.
  The protected set is the wrapper and its undetached children,
  plus the driver,
  the driver's parent and every ancestor to pid 1.
  The handler is told the wrapper's pid,
  start time and boot;
  it walks that chain and spares all of it.
  A holder inside it refuses the removal.

- Claude Code's Bash tool (F4).
  The wrapper's parent is a session-leading `zsh` with no terminal,
  so the wrapper's session is that `zsh`'s,
  with that shell's other jobs;
  the ancestors are the `zsh`,
  `claude`,
  and everything above to pid 1.
  Both the `zsh` and `claude` are spared.
  The earlier text would have spared the `zsh` through the session and ended `claude`;
  with the chain read by the handler,
  `claude` is spared too.

- `systemd-run --user --scope` (F3).
  The child shares the calling shell's session and its parent is that shell,
  so the protected set is the same as Claude Code's.

- `systemd-run --user --pipe --wait`,
  a service unit (F5).
  The wrapper is its own session leader and its parent is `systemd --user`;
  no ancestor is the process that started the unit,
  so the protected set is the wrapper and the manager's chain to pid 1,
  and the session that started the unit is not protected.
  The design says so (L10) rather than pretending otherwise,
  and recommends that a harness run the removal as a child of its own process,
  as the Pi driver does;
  naming a requester across a service boundary has no mechanism here and is not designed.

To tell the caller's protected set from another job that merely holds the clone,
the handler compares each holder task with that set and nothing else:
not a process group,
not a cgroup,
not a command name.
F2 is why a cgroup cannot serve:
the harness's commands share one cgroup,
so a cgroup rule would spare every holder the harness ever started,
or end the harness itself.

The width of the protection is the project's question Q6,
because it decides who may die.
This design implements the widest option,
the session plus every ancestor to pid 1,
so a narrower answer is a subset that changes the stop rule and no other part of the design.
The refusal and the receipt's word do not change with the width.

## What a holder is, and what the proof establishes

A holder is one of two things,
and the design treats both with one proof:

- a task whose mount namespace holds a mount of the clone's dataset,
  which is what the root scan reports (F23);

- a task whose working directory or open file lies inside a mount of the clone,
  which the scan does not report and which the wrapper's existing path-use scan finds for its own account (F25).

The second kind is not decoration.
F17 says a working directory or an open file makes a plain unmount refuse,
and F18 and F22 say a detached mount lives on while a reference remains;
VM item 4 measured a busy unmount caused by exactly that,
and F22 says the dataset stays busy while any reference to the mount remains.

Kind (d) of the brief,
a namespace whose task is gone,
is reachable after all,
and the earlier text was wrong to call it unreachable.
F13 measured that such a namespace lives on a descriptor;
F14 says a privileged caller in the host pid namespace can list every active namespace with `listns` and reopen
one by its file handle;
F15 says entering it with `setns` must happen in a single-threaded process.
Q2's options (b) and (c) use that route.
The act chosen decides whether kind (d) is ended by an unmount inside it or left,
which is why Q2 puts the act to the owner.

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

5.  Only then does it signal,
    or,
    under Q2's options (b) and (c),
    decide whether the task blocks the unmount.

What that proves is "this task is in the namespace the fresh scan reported and is the process this descriptor names",
not "this is the task the scan saw".
The earlier text claimed the second,
and its first read could be of a reused number.
The design does not need the second:
it ends every task in the holder namespace (or ends only the blockers),
so the scan's representative task is evidence and never an operand.
A reused number after the pin cannot receive the signal,
because the signal travels through the descriptor.

The scan records no start time for the representative task (F23),
and the design does not assume one.
It asks step 9's build to record it,
so a report can name the exact process and a later design could compare it,
and it says plainly that the forced arm does not depend on that field.

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

## Who ends each kind of holder: the authority options

The kinds this machine produces are these,
and each option below meets all of them in one removal:

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
  reachable by root under Q2's options (b) and (c) through `listns` and a namespace file handle (F14);

- kind (e),
  a path holder:
  a task of any account with a working directory or an open file inside the clone,
  which keeps the mount alive without mounting it (F17,
  F18).

The project's answer says the forced arm ends the holders whenever `--force` is given,
so an option that refuses kind (b) or (c) contradicts the answer;
the proposal's open item says so in as many words.
The option is still shown,
because it is the baseline the other three depart from and the ranking needs it.

The question,
with the review's added options and order,
is Q1.
This section shows what each option does.

### Option A: the account alone, and a refusal for the rest

What it is:
the wrapper ends kinds (a) and (e) of its own account with the pidfd proof and refuses every holder it cannot end.
Root still writes the scan the wrapper reads (V1);
no root process ends anything.

One removal from start to end,
on a clone at `.claude/worktrees/one`:

1.  The removal writes the intent and the caller property,
    the handler scans and publishes the scan file and its receipt.
    The scan reports kind (a) in `mnt:[4026532396]` and kind (b) in `mnt:[4026532397]`.

2.  The wrapper reads the file.
    Before its first signal it refuses,
    because kind (b) is outside its authority:
    it prints the root reader's namespace and task,
    says nothing was changed,
    and says what a person can do for that holder,
    which is to stop the holding unit themselves or end that process.
    It does not print "leave the folder" for a root-run reader,
    because leaving a folder releases nothing there.

3.  No process is ended for a removal that will not happen,
    no snapshot is deleted,
    and the clone is where it was.

Pros:
no privileged path ends anything,
so the blast radius is the account's own processes;
the refusal comes before the first signal,
which the earlier draft got wrong.
Cons:
it contradicts the forced answer for kinds (b) and (c) and leaves kind (d) untouched.
Ranking:
third.
The project's forced answer is not satisfied;
the option exists because it is the safe floor.

### Option B: the installed root handler, under the property and receipt protocol

What it is:
the account asks the handler it already installed.
Two clone-local properties carry the request (the caller's identity and the force word,
both bound to the intent's nonce),
the handler reads the caller's chain and protects it,
scans,
refuses when a protected task holds,
ends what the act option says,
scans again,
and files its receipt and the scan file.

One removal from start to end:

1.  `git worktree remove --force <slot>` writes the caller property,
    the force property and then the intent,
    all with one nonce,
    and waits for the receipt (F24).

2.  The handler rechecks the intent under the per-slot claim,
    reads the three matching values,
    proves the wrapper's pid and start time from `/proc` (A6),
    and walks its chain to pid 1 (F26).
    It scans before it moves anything.
    It finds kind (a) in one namespace,
    kinds (b) and (c) in another (the same root unit seen by task and by unit),
    and kind (e) in its own namespace,
    the staging folder.

3.  It compares every holder with the protected set.
    None is in it,
    so it ends each proved task through its pidfd,
    or,
    under Q2's option (b),
    unmounts the clone's mount inside each holder namespace and ends only the tasks that block that unmount.

4.  It re-reads the wrapper's identity after each pass.
    While it stands,
    the act goes on;
    if it stops standing,
    the act stops and the receipt says `caller-gone`.

5.  It scans again,
    moves the clone to staging,
    unmounts its own mount without force and without lazy detach,
    and files the scan file,
    then `done`.
    Step 16's scans follow under fresh nonces,
    each carrying the force word again,
    so a holder that appeared is met then too.

6.  The person sees the ordinary removal report,
    plus the holders it ended by namespace and task.
    The head destroy succeeds.
    A holder the act could not end leaves the snapshots gone and the head refused,
    which the report names as the resume state and claims nothing about.

Pros:
one privileged program the route already installs,
the accepted receipt protocol,
no configuration file,
no prompt,
one protected set read from the wrapper's own process,
every act bounded by the nonce and the wrapper's liveness.
Cons:
the handler must read the account's process table,
which A4 permits;
a handler round trip is needed for every scan,
whose latency is unmeasured (L8).
Ranking:
first.

### Option C: a root helper the wrapper starts through passwordless `sudo`

What it is:
the wrapper runs the installed handler binary in a `kill-holders` mode under `sudo -n`,
and reads its answer from standard output.
The helper takes the protected set from its own parent chain rather than from a property.

One removal from start to end:

1.  The wrapper runs `sudo -n <handler> kill-holders --dataset <name> --checkout <slot>`.
    `-n` makes a password request a failure rather than a prompt,
    which the never-prompt rule requires.

2.  The helper walks up its own chain past the processes that are `sudo` itself,
    and takes the first process that is not sudo as the one that ran it,
    which is the wrapper.
    F30 is why the skip is needed:
    with no pty,
    wrapper,
    `sudo` and helper share the wrapper's session;
    on the pty path the monitor calls `setsid` and the helper's session is the monitor's.
    Skipping the sudo processes lands on the wrapper's process in both cases.
    The protected set is that process's session and its chain to pid 1.

3.  The helper scans,
    refuses when a protected task holds,
    and acts as Q2 decides,
    with the same pin-first proof.
    It reports what it did on the same pipe,
    and the wrapper prints it.

Pros:
the caller's set is inherited by parentage rather than asserted in a property,
and the answer is synchronous;
the sudo-process skip is read from sudo's source rather than guessed.
Cons:
it needs a sudoers rule naming the binary (configuration,
against the project's no-configuration answer),
it fails under a unit with `NoNewPrivileges=yes`,
it adds a second privileged channel beside the accepted one,
and this round could run no `sudo` at all,
so its parentage is owed a guest run (M9).
Ranking:
second.

### Option D: the systemd managers

What it is:
the account asks its own user manager to kill a unit it started,
and the system manager for a system unit.

One removal from start to end:

1.  For kind (a) inside a transient unit of this account,
    the wrapper runs `systemctl --user --no-ask-password kill --signal=SIGKILL <unit>`.
    This works (F28 records the measured own-unit kill),
    and it kills the whole unit's cgroup,
    including tasks that never held the clone.

2.  For kind (b) or (c),
    the wrapper's request goes to the system manager,
    which asks polkit for `org.freedesktop.systemd1.manage-units`.
    The installed defaults are `auth_admin` and this account is in `wheel` (F29),
    so the request waits on a person or,
    with `--no-ask-password`,
    is answered `INTERACTIVE_AUTHORIZATION_REQUIRED`;
    the never-prompt rule forbids the wait,
    and the account cannot make the request quietly.

3.  Kind (d) is not reached,
    and kind (e) outside a unit is untouched by a unit kill.

4.  In the Pi shape the unit that holds the clone may be the harness's own unit (F2),
    so step 1 can end the caller's harness,
    which the never-end set forbids.

Pros:
no new privileged program for the account's own units,
and a unit kill is atomic.
Cons:
the system-manager half prompts or fails;
the account's own units are killed whole,
which in the Pi shape is the caller's harness;
the mechanism names no holder,
so the report cannot say who was ended.
Ranking:
fourth.
It breaks an explicit owner rule (nothing asks a person for a secret) and can end the caller,
where option A only falls short of the forced answer.

### Ranking of the authority options

B > C > A > D,
and this ranking decides nothing;
the authority stays Q1.

- B over C:
  B rides the protocol,
  the receipts and the per-slot claim the route already has,
  needs no configuration file,
  and this round could measure no `sudo` path.
  C's inherited caller identity is the better proof,
  and a later owner answer may prefer it for that reason,
  at the cost of a sudoers line and a second channel.

- C over A:
  C honours the forced answer for kinds (b) and (c),
  and A refuses them outright,
  which the answer's words exclude.

- A over D:
  A prompts nobody and cannot end the caller's harness;
  D breaks the never-prompt rule and can end it.

- The earlier text ranked D above A.
  That was wrong for the reason the review gives:
  D breaks an explicit owner rule,
  while A only falls short of an answer the owner can still choose to change.

## What each authority option does to the rest of this design

The review's M2:
after Q1 is called the owner's,
the later sections must say which answer they depend on.
The design is written for option B.
The carry-over is:

- Under A:
  the caller property and the force property are not needed,
  because root ends nothing and the wrapper acts itself.
  The identification proof,
  the protected set,
  the refusal and its two texts,
  the scan file and its V2 rule,
  and tests 3,
  4,
  6,
  7 and 9 hold unchanged.
  The act section reduces to same-account signals,
  and Q2's unmount options are unreachable because an unprivileged wrapper cannot enter another namespace
  (F15 needs `CAP_SYS_ADMIN` in the owning user namespace).
  Kinds (b) and (c) become refusals.
  The design's "every authority" test arms for kinds (b) and (c) expect the refusal.

- Under C:
  the properties are replaced by the helper's arguments and its answer on the pipe,
  and the protected set is read from the helper's own ancestors (the sudo skip,
  F30) rather than from `/proc/<wrapper pid>`.
  The scan file and the refusal still come from the handler;
  the act runs in the helper;
  liveness is implicit,
  because the helper is a child whose parent is the waiting wrapper.
  The logging moves to the helper's output and the wrapper's report.

- Under D:
  the handler does not act,
  so the caller property and the force property are replaced by unit names,
  and the identification proof is systemd's (a unit's cgroup),
  not this design's.
  The refusal rule still needs the scan,
  because the wrapper must not ask for a unit that holds a protected task.
  Q2's act options are unreachable for kinds (b) and (c) without the system manager's authorization,
  and kind (d) is out of reach.
  Tests about the caller's protection become tests about a unit's cgroup,
  and the Pi-shape test asserts the harness survives,
  which D cannot promise.

- Under B:
  as designed throughout.

## How much a forced removal ends, and with which signal

These are the project's questions Q2 and Q3.
The earlier text chose both silently;
the review found the act's premise contradicted by the kernel source it targets,
so the act is now Q2,
and the signal is Q3.

Q2,
how much a forced removal ends,
offers three acts:

- (a) end every task in each holder namespace;

- (b) unmount the clone's mount inside each holder namespace,
  without force and without lazy detach,
  and end only the tasks that block that unmount;

- (c) (b) first,
  then (a) for any namespace (b) cannot reach.

The premise the earlier text got wrong:
the aim is to release the dataset's long hold,
not to free the namespace (F20,
F21).
A plain unmount inside a holder namespace drops that namespace's copy (F22),
and F17 says it refuses `EBUSY` while a task's working directory or open file is there,
so the blockers are the tasks that must end.
A lazy detach is not the tool:
F18 and F22 say the copy stays alive while a task holds it,
so the dataset stays busy.
Proposal lines 730 to 732 are what the project has authorised:
the path unmount in VM item 15's shared control "does not authorize path unmount in the design",
so (b) and (c) go to the owner,
not to this writer.
The route the owner's answer would use is read and unrun:
`listns` lists every active namespace to a privileged caller (F14),
a namespace file handle reopens one,
and `setns` must be called from a single-threaded process (F15),
which is a `fork` inside the handler.

Concrete case,
one removal on this machine:
a `PrivateMounts=yes` unit with a web server that never opened the clone and a helper whose working directory is in it.
Under (a) the handler ends both,
two tasks;
under (b) it unmounts the unit's copy,
the helper blocks it,
so it ends the helper and the web server runs;
under (c) the same as (b),
and if the entering or the unmount is refused it falls back to (a) for that namespace.

Pros and cons:
(a) is simple and needs no `listns`;
it ends processes that never touched the clone and cannot reach a namespace with no task.
(b) ends the fewest and is the only option that reaches kind (d) without ending anything;
it needs root to enter namespaces,
is unmeasured on this route,
and needs the owner's authorization that the proposal withholds.
(c) finishes a `--force` removal most often while ending the fewest,
and is the most code.
Ranking:
c > b > a.
c over b because b alone leaves a forced removal failing where the answer says holders are ended;
b over a because a ends processes that held nothing.
The guest arm M3 is aimed at a plain unmount for (b) and (c).

Q3,
the signal,
applies to the tasks any act option decides to end:
SIGKILL alone;
SIGTERM,
wait on the pidfd,
then SIGKILL;
SIGTERM alone.
The concrete case is an editor with unsaved buffers whose working directory is in the clone.
SIGKILL loses them;
SIGTERM lets it save and exit and the removal waits;
SIGTERM alone cannot promise it ends.
Pros and cons:
SIGKILL ships no number and cannot be refused,
and loses a person's unsaved work;
SIGTERM then SIGKILL gives the holder its chance and needs a grace the project must decide or measure (L9);
SIGTERM alone can fail to end the holder.
Ranking:
SIGTERM then SIGKILL > SIGKILL > SIGTERM alone.
The first over the second because the cost of an unmeasured grace falls on the removal's time while the cost of
none falls on a person's data;
the second over the third because the third can fail.
This reverses the earlier V3,
which ranked SIGKILL first for the writer's convenience.

## The act's mechanism, which holds under every act option

Whatever Q2 answers,
these hold:

- The handler never signals a task in its own namespace except a path holder whose working directory or open file
  is in the clone.
  A task in the handler's own namespace is the handler's own world,
  and ending it is not what any holder claim can justify.

- The handler refuses a holder namespace that contains PID 1,
  and it refuses when any protected task is a holder.
  A namespace's tasks are all enumerated,
  so a protected task anywhere in it refuses the whole namespace.

- The number of tasks and namespaces one act reaches is bounded by the scan's own bounds (F23):
  the act never sees more than the scan listed,
  and a scan that stopped at its bound is not a clear answer.
  The values are the scan's,
  not this design's,
  and they ship with the scan's own veto.

- Every task is pinned and proved before it is signalled (the five steps above),
  and a task whose proof fails is reported,
  not signalled.

- Path holders are matched by the mount id and inode the kernel records,
  not by a resolved path:
  `UseTarget::Mounted` (F25) for an open descriptor,
  and `statx` with `STATX_MNT_ID` on `/proc/<pid>/cwd` for a working directory (Linux `v7.3`,
  `include/uapi/linux/stat.h:216`).
  A resolved path would miss a task in another namespace or under another root,
  which is the review's L3;
  where a read is refused,
  the report says the walk was blind for that task.

- Before each pass and after it the handler re-reads the wrapper's pid and start time.
  When they no longer stand,
  the act stops and the receipt names `caller-gone`.

- The act's wait is the pidfd's exit (`POLLIN`) and,
  for an unmount,
  the kernel call's return,
  with the same pause the receipt wait uses.
  It is bounded by the sweep's own work bound,
  not by the wrapper's 30-second poll deadline (F24):
  the handler is not told that client-side number,
  and it does not need it,
  because a dead wrapper ends the act.

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

- one closing line:
  how many holders were ended,
  how many were unproven,
  and whether the following scan was clear.
  The earlier draft put "the removal deadline's remaining time" here;
  the handler has no source for that number (review L1),
  so it is gone.

The wrapper prints the same facts in its report,
because the decision's report has to name who was ended.
It does not print them for a holder of the protected set;
that case has its own refusal text.

## The refusal, and what it leaves

The handler refuses when any holder task is in the protected set.
It decides that on the pre-move scan,
before anything moves,
and files the receipt's failed step as `caller-protected-holds`.
The decision's own words cover one case,
a working directory inside the clone ("leave the folder and run the command again",
[decision][decision],
lines 52 to 53);
a mount namespace holding a copy needs a second text,
because leaving a folder releases nothing there (review M3).

The working-directory text,
with the variables filled and every path quoted as a shell word:

```text
the clone at <slot> is held by the removal's own protected set, and a removal never ends it.
process <pid> (<command>) holds it in its working directory, in session <session>, and that process is the caller,
or is in the caller's session or its ancestor chain. Nothing was moved, unmounted or destroyed: the clone, its
snapshots and the Git registration are as they were, and the folder is still mounted.
Leave the folder (end that job, or run the command from another shell) and run the removal again.
```

The namespace text:

```text
the clone at <slot> is held by the removal's own protected set, and a removal never ends it.
mount namespace <namespace> keeps a copy at <mount point> (mount <id>), and process <pid> (<command>) in session
<session> is the caller, or is in the caller's session or its ancestor chain. Leaving the folder does not release
a namespace's copy: end that session or its unit, or run the removal from a session that never entered the clone.
Nothing was moved, unmounted or destroyed: the clone, its snapshots and the Git registration are as they were.
```

What the wrapper leaves untouched:
no signal to any task,
no move,
no unmount inside a holder namespace,
no lazy detach,
no snapshot delete and no head destroy,
so the clone's history is whole.
The restored state,
the records,
the reattach and the nonce of the request that reverses the intent belong to
[the cancelled or killed removal][cancelled] (M4);
this design removes its own account of them.
What this design needs from that document:
a stop point for a forced removal refused for `caller-protected-holds` and for a forced holder that cannot be ended,
the restored state named there,
and a fresh nonce for the reversing request,
because a receipt is matched by nonce,
operation and boot (F24) and the earlier `unmount` answer must not be read as the new `mount` answer.
The sibling's current text says "the automatic arm of this design does not run for a forced removal"
([cancelled][cancelled],
lines 2042 to 2045),
so this stop point is a requirement of this design on the sibling's fix round,
and the two documents currently disagree about who owns it.

## The time between the scan and the ending

A forced request must not act after the wrapper that made it is gone (H2).
Three things settle it:

- The force is honoured only with a standing `unmount` intent whose nonce,
  the force property's and the caller property's all agree,
  and only while the wrapper's pid and start time,
  read from `/proc` by the handler,
  still stand (A6,
  F11).
  The handler checks them before the first act and after every pass.

- The handler does not read the account-owned removal claim,
  because the sibling places it in the repository and says "the handler never reads it at all" ([cancelled][cancelled],
  lines 1352 to 1358).
  The liveness proof is the wrapper process;
  the claim keeps two wrappers from overlapping on the wrapper side (A7).
  The review's suggested claim check is therefore not implementable without moving that record under root,
  which this design does not propose.

- A later trigger that finds a forced request whose wrapper is gone declines the force word,
  files `caller-gone`,
  and does only the non-force work the intent asks:
  it may unmount as any removal would,
  and it never ends a process.
  That is the answer to "what a later trigger does with a forced request nobody is waiting on".

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
and every such scan under the force word ends the holders it sees.
A holder that attaches after the last scan shows up as `dataset is busy` at the head destroy,
the accepted destroy-anyway residual.
The retry is bounded by the removal deadline;
nothing loops until the pool is quiet.

A holder that exits and whose number is reused.
Nothing is signalled by number;
the pidfd is opened while the number names the proven process,
and the pin's two start-time reads close the interval (F9,
F10).
A namespace identity reused after its namespace died is not enough either:
the namespace link and the mount row are read from the pinned task.

A holder the scan cannot see.
A path holder in another account,
or a path holder whose resolved path differs,
is the named residual;
the match by mount id and inode above narrows it,
and where a read is refused the report says so.
A taskless pinned namespace is reachable under Q2's options (b) and (c) through F14;
under option (a) it is not reached and the head destroy refuses,
which the report names.
A task that survives the act's wait is reported,
not chased;
a task that keeps forking cannot extend the act past the scan's bounds and the liveness check.

## Where the records differ, and how this design reads them

- The refusal and the no-force arm.
  The decision's never-end set says the removal refuses when the caller's session holds the clone.
  The blocked-removal answer says a removal without `--force` cancels itself.
  This design reads the refusal as the forced arm's rule:
  without `--force` nobody is ended,
  the auto-cancel applies,
  and the never-end set is still met.
  V5 puts the other reading to the project.

- The scan before the move.
  The accepted design's removal-ready scan comes after the move and the unmount,
  and the proposal asks for each scan through a new `unmount` intent.
  This design adds a scan under that same intent before the handler moves anything,
  so the protected-set refusal and the first act come before a snapshot is lost.
  Nothing in the accepted design forbids an earlier scan,
  and V4 says why it runs there.

- The scan's strict clear.
  The step 9 scan calls a result clear only when no holder and no incomplete read stands,
  while the proposal reads an unreadable or racing entry as not refusing a removal by itself (line 739).
  This design keeps both:
  an incomplete read is reported and is never read as clear,
  and the removal goes on under the destroy-anyway rule.
  A scan file that never arrived is different,
  and V2 makes it a refusal at the step that needed it.

- The protected set's width.
  The decision names the caller,
  its session,
  the shell that ran the command and that shell's other jobs.
  This design protects the session and every ancestor to pid 1,
  which is wider than the words.
  Q6 puts the width to the project,
  and the design implements the widest so a narrower answer changes only the stop rule.

- The failure path.
  The sibling owns what a removal leaves when it does not finish,
  forced or not (M4);
  its current text excludes forced removals,
  so the stop point is a requirement of this design on its fix round.
  This design no longer reverses the intent,
  reattaches or reports lost snapshots itself,
  and it writes no `mount` request;
  it asks the sibling to write one with a fresh nonce.

- The receipt's contract.
  The sibling adds a sixteenth key,
  `bind` ([cancelled][cancelled],
  lines 1395 to 1416);
  this design adds no key,
  and carries the holder list in the scan file and the refusal in a bounded failed-step word (F24).
  A key and a word can coexist,
  and the receipt's derived bound changes once,
  in the sibling's change.

- The claim.
  The sibling says the handler never reads the account's claim (A7);
  this design's liveness proof is the wrapper process,
  not the claim.

- Step 16's ownership.
  The deferred-destroy checks own step 16's call sites ([deferred][deferred],
  lines 2168 to 2180,
  and its "What each build step builds");
  this design claims no ownership of those checks and only needs the scan file's reader,
  whose build place is the first consumer's.

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
Under the adopted option the person's report names `mnt:[4026532396]`,
`/proc/127996` and the mount id from the receipt's nonce,
and `doctor` prints the same row.
Under the in-receipt option the report says the same,
and every existing receipt reader gains an optional multi-line blob.
Under the on-demand option the report says the same and there is no scan file for `doctor` to read.
Under the outcome-only option the report says "a holder was ended" and cannot name it,
and the refusal cannot be printed from the holder's own fields.

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
an `unknown` that does not refuse and lets the removal go on,
as the proposal's reading of an incomplete read would;
a retry of the scan before going on.

Concrete example:
the wrapper asks for the scan before the first snapshot delete and the receipt arrives with no scan file.
Under the adopted option the removal refuses,
prints that the finding never arrived,
and destroys nothing.
Under the second it treats the missing file as an incomplete read and goes on to delete the snapshots.
Under the third it asks again,
and a handler that cannot publish a scan file will fail again.

Pros of the adopted option:
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
start time and boot and the other the force word;
a longer intent grammar carrying both;
a single property carrying a request record.

Concrete example:
`--force` on the clone above.
The adopted option writes `v1:<nonce>:<pid>:<starttime>:<boot>` and `v1:<nonce>:force` beside the existing
`v1:<nonce>:unmount`.
The longer grammar writes `v1:<nonce>:unmount-force:<pid>:<starttime>:<boot>` into the one property,
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

### V5. Whether the protected-set refusal needs `--force`

Options:
the refusal is the forced arm's rule,
printed when `--force` was asked and a protected task holds,
while a removal without `--force` cancels itself (adopted);
the refusal is unconditional.

Concrete example:
a person runs `git worktree remove <slot>` with a shell sitting in the clone.
Under the adopted option the removal cancels,
the checked reattach puts the clone back,
and the person reads the cancelled-removal report.
Under the unconditional option the person reads the refusal,
the clone stays as it is,
and nothing is reversed.

Pros of the adopted option:
it follows the blocked-removal answer for the no-force arm,
and the caller is safe because nobody is ended there.
Cons:
two different words reach the person for one holder,
depending on a flag.
Ranking:
forced-arm-only > unconditional,
because the first keeps the auto-cancel the project chose and the second turns an ordinary cancel into a refusal
the project did not ask for.

## Questions only the project can answer

### Q1. By what authority the wrapper ends another account's process or a unit's task

Options:

- A.
  The account alone.
  The wrapper signals only what its account may and refuses the rest,
  naming the holder and what a person can do for it.

- B.
  The installed root handler ends holders on the account's request,
  under the property and receipt protocol.

- C.
  A root helper the wrapper starts through `sudo -n`.

- D.
  The systemd managers.

Concrete case:
`git worktree remove --force one` on `.claude/worktrees/one` while a root `systemd-run` job holds a copy of the
clone in `mnt:[4026532397]` and a shell of this account has its working directory inside.
A ends the shell and refuses,
printing the job's unit and "stop that unit yourself,
then run the removal again";
nothing is destroyed.
B ends both and the head destroy succeeds.
C does what B does through a helper,
after a sudoers entry the setup writes.
D asks polkit,
which on the installed defaults wants administrator authentication (this account is in `wheel`,
so that is its own password in a desktop dialog,
the thing that happened),
or,
with `--no-ask-password`,
is answered "requires interactive authentication" (F29);
local rules are unread (L11).

Pros and cons:
A adds no privileged path and cannot over-reach;
it contradicts "Kill ...
the holders whenever --force is specified" for other accounts' holders,
as the proposal's open item says.
B keeps the accepted protocol and needs no configuration;
it lets an account-writable property make root end processes,
which A4 permits only with the liveness binding and the act's bounds.
C gives the better caller proof by parentage once the sudo skip (F30) is in the algorithm;
it adds a second privileged channel and a sudoers line,
and fails under `NoNewPrivileges`.
D uses no new program;
it prompts a person,
which the never-prompt rule forbids,
and `systemctl kill` ends a whole unit including,
in the Pi shape,
the caller's own (F2).

Ranking:
B > C > A > D.
B over C because B rides the receipts and the per-slot claim already built and adds no channel.
C over A because C honours the forced answer for other accounts' holders,
and its configuration cost is a line the step 19 setup could write,
as it writes the handler's configuration.
A over D because A prompts nobody and cannot end the caller's unit,
while D breaks the never-prompt rule and can.

### Q2. How much a forced removal ends

Options:
(a) every task in each holder namespace;
(b) unmount the clone's mount inside each holder namespace,
without force and without lazy detach,
and end only the tasks that block that unmount;
(c) (b) first,
then (a) for any namespace (b) cannot reach.

Concrete case:
a `PrivateMounts=yes` unit with a web server that never opened the clone and a helper whose working directory is in it.
(a) ends both.
(b) unmounts the copy,
the helper blocks it,
so only the helper is ended and the web server runs.
(c) ends the same as (b),
and falls back to ending the namespace's tasks only if entering the namespace or the unmount is refused.

Pros and cons:
(a) is simple and the closest to what the guest arm already plans;
it ends processes that never touched the clone and cannot reach a namespace with no task.
(b) ends the fewest;
it needs root to enter namespaces from a single-threaded child (F14,
F15),
is unmeasured on this route,
and goes beyond what proposal line 732 authorises ("does not authorize path unmount in the design").
(c) is the most likely to finish a `--force` removal while ending the fewest;
it is the most code and carries (b)'s authorisation question.
Ranking:
c > b > a.
c over b because b alone leaves a `--force` removal failing where the answer says the holders are ended;
b over a because a ends processes that held nothing.
Nothing is chosen until the guest arm M3 runs and the owner decides whether an unmount inside a namespace is
within "otherwise remove".

### Q3. The signal

Options:
SIGKILL alone;
SIGTERM,
wait on the pidfd,
then SIGKILL;
SIGTERM alone.

Concrete case:
an editor with unsaved buffers whose working directory is in the clone.
SIGKILL loses them.
SIGTERM lets it save and exit,
and the removal then waits for that.

Pros and cons:
SIGKILL ships no number and cannot be refused,
and loses state.
SIGTERM then SIGKILL gives a holder its chance,
and needs a grace the project must measure or decide (L9).
SIGTERM alone cannot promise the holder ends.
Ranking:
SIGTERM then SIGKILL > SIGKILL > SIGTERM alone.
First over second because the cost of an unmeasured grace falls on the removal's time while the cost of none falls
on a person's data;
second over third because the third can fail to end the holder.

### Q4. How long a forced request's authority lasts

Options:
(a) the handler honours the force word only while the wrapper that wrote it is alive,
by pid and start time (adopted by the design's implementation);
(b) as long as the nonce matches and the caller's session exists;
(c) for a fixed time (a number).

Concrete case:
Ctrl-C at the twentieth second;
a ZED event at ten minutes.
(a) ends nothing at ten minutes and files `caller-gone`.
(b) ends whatever attached since.
(c) depends on the number.

Pros and cons:
(a) matches what the person asked for and lets the standing request still do its harmless mount work;
it needs the liveness check and a receipt word.
(b) is cheaper and lets a later trigger finish a removal the person abandoned with a live session.
(c) ships an unmeasured number.
Ranking:
a > b > c.
a over b because the person who interrupted a forced removal is the person whose later processes would be ended;
b over c because b ships no number.

### Q5. What a forced removal does when a holder cannot be ended

Options:
refuse before the first snapshot delete;
proceed while the holder is still seen;
retry after the deadline.

Concrete case:
a root reader in uninterruptible sleep at the pre-move scan.
The first prints the refusal with the clone,
its snapshots and registration whole;
the second destroys the snapshots and then meets `dataset is busy`.

Pros and cons:
the first keeps history and makes `--force` failing visible,
and a `--force` removal can end with nothing destroyed;
the second spends the clone's own snapshots to learn the act failed;
the third repeats the same failure at the removal's cost.
The second is not the destroy-anyway decision (that covers holders the scan cannot see),
so choosing it overrides "the scan that refuses the holders it sees stays".
Ranking:
refuse first > retry > proceed.
Refuse over retry because retry repeats the same failure at the removal's cost;
retry over proceed because proceed spends the clone's history to learn the act failed and contradicts the kept scan.
The failure's restored state is the sibling's to design (M4,
D2).

### Q6. What the caller's protection covers

Options:
(a) the caller's session only (the decision read literally);
(b) session plus ancestors to the first session leader;
(c) session plus every ancestor to pid 1,
and refuse when any of them holds the clone.

Concrete case:
Claude Code started in `.claude/worktrees/one` removing `one`.
(a) and (b) end `claude` (it was started there,
so it holds the directory) and the removal's caller with it.
(c) refuses,
naming `claude` and saying to run the removal from outside the worktree.

Pros and cons:
(a) is the record's words and ends the harness.
(b) is one chain wider and misses the harness in every shape measured here (F4,
F5).
(c) never ends a harness or a terminal owner;
it refuses where (a) and (b) would have removed,
and the person runs the command from the main checkout.
Ranking:
c > b > a.
c over b because b misses the harness in the Claude Code,
scope and unit shapes;
b over a because a ends the harness even in the shape where b would have spared it.

### Q7. Whether a forced ending leaves a durable record beyond the journal

Options:
the handler's journal and the wrapper's standard error;
an entry in the removal's persisted plan record;
a record of its own per slot.

Concrete case:
a week after a forced removal a person asks who was ended.
The journal may have rotated;
a plan entry survives as long as the plan;
a record of its own survives until something prunes it,
and nothing does.

Pros and cons:
the journal adds no state and is where root's acts already go;
it rotates.
A plan entry survives rotation and sits beside the loss report;
it couples a record kept while unfinished to an audit,
and needs a field the plan record's design does not have today ([planrecord][planrecord],
its fields section,
lines 681 to 892,
and its no-dependency entry,
lines 2333 to 2337).
A record of its own is exact and has no reader or pruner.
Ranking:
plan entry > journal > own record.
The plan entry over the journal because it survives rotation and sits beside the destroyed set;
the journal over the own record because the own record has no reader or pruner.
The plan entry is contingent on that design adding the field,
which this design asks of it (D4);
until then the journal is what ships.

## Which build step builds which part

- Step 9's holder-scan round has built the scan,
  the text,
  the parser and the mode,
  as work in flight.
  This design asks that build to record the representative task's start time (F23 has none),
  and it builds no scan and no act there.

- Step 16 owns the scans' call sites and the deferred-destroy checks ([deferred][deferred]);
  the scan file's publisher and reader are built with the first consumer,
  which is step 16 or step 17,
  and this design only needs the read interface and V2's rule.
  The design claims no ownership of step 16's checks.

- Step 17 builds this design:
  the caller and force properties,
  the chain read and the protected set,
  the liveness check,
  the two refusal texts and their receipt word,
  the pre-move scan,
  the pin-first proof,
  the act (as Q2 answers),
  the logging and the tests.

- Step 18 builds the `doctor` reading:
  the last scan file,
  the live scan mode,
  the holders named by namespace and task,
  and the protected set the removal spares.

- Step 19 adds a sudoers line only if Q1 chooses C,
  and the single-threaded entering helper only if Q2 chooses (b) or (c).

- No step builds a second scan.
  `listns` and entering a namespace are built only under Q2's (b) or (c).

## Tests a build must have

Each test names the race or interleaving it forces,
how it forces it without timing luck,
and whether it needs a guest.

1.  The proof refuses a task that exits before its pin.
    Enumerate a task,
    let it exit,
    then run the proof and assert no signal;
    the pin's two start-time reads are the same shape `PinnedMember` uses.
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
    Start the removal from a child of a process that holds the clone but is not in the wrapper's session
    (the harness shape,
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
    with a mismatched nonce,
    and with a dead pid and start time,
    and assert no signal,
    the receipt says `caller-gone`,
    and only the non-force work happens.
    Device-free.

10. `doctor` names holders and the protected set without root.
    Give `doctor` a scan file and a live scan mode answer for a clone held in a namespace,
    and assert the namespace,
    task,
    mount id and path appear,
    that the protected set is named,
    and that `doctor` never signals or unmounts.
    Device-free.

11. A taskless namespace.
    Pin a namespace that mounts the clone,
    end its task,
    and assert under Q2's (b) and (c) that the handler lists it (F14),
    enters a single-threaded child,
    unmounts inside it,
    and the head destroy succeeds;
    under (a) assert the head destroy refuses and the report names the resume state.
    Guest.

## Measurements owed

M1 to M11 are the arms of one guest run,
on the fixture of [measurements][measurements]:
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

- M1.
  Mount-namespace holders outside the protected set.
  Extend VM item 15's arms:
  an `unshare --mount --propagation unchanged` reader and a `PrivateMounts=yes` unit,
  both started by a session that is not the caller's,
  after the attach.
  Run `git worktree remove --force <slot>` under Q2's options (a) and (b).
  Record each reader's wait status,
  the last output it wrote and the last error it printed before the act,
  whether its namespace row vanished,
  the scan before and after,
  and that the head destroy exited 0.
  Control:
  the same arm without `--force`,
  recording that both readers live and the clone is reattached.

- M2.
  Protected-set holders.
  The same two readers started by the removal's own session:
  one holding a working directory,
  one holding a mount namespace.
  Assert the two refusal texts,
  the readers alive,
  the mount at its slot,
  the snapshots and the registration whole,
  the receipt's `caller-protected-holds` step,
  and the absence of any destroy request.

- M3.
  A plain unmount inside a holder namespace.
  Repeat VM item 15's shared-checkout control as a plain,
  non-lazy path unmount inside a disposable holder namespace only,
  and record what each holder sees (a vanished mount,
  stale content,
  an error,
  or its own last output) and whether the head destroy then succeeds.
  This is Q2's (b);
  it is not a force-unmount and it is the arm whose success would let the owner choose (b) or (c).

- M4.
  A path holder in the staging folder.
  Reproduce VM item 4's busy holder with the built handler,
  run with `--force`,
  and record the unmount,
  the destroy,
  and the holder's wait status.
  Control:
  without `--force`.

- M5.
  A path holder outside every holder namespace.
  A process with an open descriptor on a file inside the clone,
  in a namespace that does not mount it,
  and a second with its working directory under another root.
  Record whether the mount-id-and-inode match finds them,
  whether they are ended,
  and whether the destroy succeeds.

- M6.
  A namespace whose task is gone.
  Pin a namespace that mounts the clone with a descriptor held by a process outside it,
  end its task,
  run with `--force`,
  and record under Q2's (a) that the scan reports no holder,
  the batch succeeds and the head destroy refuses `dataset is busy`;
  under (b) and (c),
  that the handler lists the namespace with `listns` or a file handle,
  enters it in a single-threaded child,
  unmounts,
  and the head destroy succeeds.
  Then close the descriptor and retry,
  recording the head destroy's success.

- M7.
  A holder the kernel will not stop.
  A task in an uninterruptible sleep on a FUSE or NFS arm if the guest recipe can build one;
  otherwise recorded as unrun with the reason.
  Record the act's end,
  the report,
  and what the removal does per Q5.

- M8.
  The authority answers this host could not be asked,
  and the arm the sibling design owes as its M9.
  In the guest,
  as an unprivileged account with no authentication agent:
  `kill` and `pidfd_send_signal` on a root process (expected `EPERM`,
  F8);
  `systemctl --user --no-ask-password kill --signal=SIGKILL` on a transient unit of that account (expected to work,
  F28);
  `systemctl --no-ask-password kill --signal=SIGKILL` on a system unit the account does not own
  (expected to be refused for want of authorization,
  F29);
  and the same with an agent that answers,
  recording that the request waits on the agent.
  The guest has no desktop,
  so the last arm records the failure path,
  not the dialog the host showed (L5).
  One arm,
  shared with the sibling;
  neither design owes it twice.

- M9.
  The `sudo` route's parentage.
  If Q1 ever prefers option C:
  in a guest,
  with a sudoers rule naming only the helper,
  run `sudo -n <handler> kill-holders` and record whether the helper's chain past the sudo processes leads to the
  wrapper and whether the no-pty and pty paths differ as F30 says.

- M10.
  The scan file's latency and the act's cost.
  Time the publish to the wrapper's read,
  and time a kill of one and of many holders,
  and a plain unmount inside one namespace,
  to their outcomes.
  No value ships before this runs.

- M11.
  The service-unit shape.
  Start a removal as a `systemd-run --user --pipe --wait` service unit from a process that holds the clone,
  and record that the outer process is not in the protected set and is ended (L10).
  Then start the same removal as a child of that process,
  and record that it is spared.
  The first half shows the limit;
  the second shows the recommendation.

## Limitations

- Every claim about the dataset becoming destroyable after a plain unmount or after a kill is an inference from
  source and from VM items 4 and 15 (L1).

- The caller's protected set is a number inside one pid namespace (A1,
  L4);
  a removal inside a container is not covered and probably cannot reach `/dev/zfs` at all.

- The caller property is an assertion the handler ties to a live process,
  not to a sender (A4);
  the liveness check keeps it from being stale,
  not from being wrong about which process is the caller.

- A caller on the far side of a service unit is not protected (F5,
  L10).

- A confined unit the Pi driver starts is not an ancestor and can be ended if it holds the clone (L13).

- Kind (d) is reached only under Q2's (b) and (c);
  under (a) it stays and the head destroy refuses.

- The `listns` and namespace-file-handle route is source-read and unrun (F14,
  L12).

- The act chosen by Q2 is unmeasured;
  M3 is the arm that would measure it.

- The protected-set refusal is unmeasured (M2),
  and the system-manager answers are unverified on this host (L5,
  L11).

- The handler's own work bound is the sweep's,
  not a number this design ships (L9);
  its value is not measured here.

- The plan record has no field for "who was ended" today,
  so Q7's first-ranked home is contingent on D4.

## What this design does not cover

- What a cancelled,
  interrupted,
  timed-out or killed removal leaves,
  and the kept `cancel-worktree-removal` command:
  the sibling owns it (D2),
  including the forced stop point this design requires of it.

- Where the removal's persisted destruction plan lives,
  which fields it has,
  and what a forced ending should record in it (D4,
  Q7).

- The deferred-destroy checks themselves and step 16's call sites (D3).

- A second scan,
  any force or lazy unmount of a staged mount,
  and any automatic design of the `listns` route beyond what Q2's (b) needs.

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
  [The cancelled or killed removal][cancelled] owns what a removal leaves and restores when it does not finish,
  forced or not:
  the states,
  the records,
  the reattach,
  and the nonce of a new `mount` request.
  This design takes its own failure-path mechanics out and asks that document's fix round for a forced stop point
  (`caller-protected-holds` and a holder that cannot be ended),
  the restored state,
  and a fresh nonce.
  It currently says "the automatic arm of this design does not run for a forced removal" (lines 2042 to 2045),
  which is the disagreement this design's requirement resolves.
  It also adds a sixteenth receipt key,
  `bind` (lines 1395 to 1416);
  this design adds no key and a bounded failed-step word,
  so the two can coexist.

- D3.
  [The deferred-destroy checks][deferred] own step 16's checks and call sites.
  This design claims no ownership of them;
  it needs the scan file's reader and V2's rule,
  and the build place is the first consumer's choice.
  That document records an earlier version of this one as assuming "a refusal rewrites the intent to `mount`"
  (lines 2168 to 2180);
  this revision removes that assumption and defers the state to D2.

- D4.
  [The destruction plan record][planrecord] owns the plan's home,
  fields,
  readers and lifetime.
  This design needs the record only if Q7 chooses the plan entry,
  and then it needs a field for who was ended,
  which that document does not have (its "no dependency" entry,
  lines 2333 to 2337).

- D5.
  [The lost-last-event triggers][losttriggers] own nudge merging and the rate rule.
  This design adds no nudge;
  the forced request is the intent plus two properties,
  and the merge rule does not read them.

- D6.
  [The propagation after a replacement mount][propagation] states "no dependency" on this design and leaves the
  holder scan and the forced arm to this document (lines 886 to 895,
  2835 to 2848).
  This design keeps the scan's visibility unchanged from VM item 15 and needs nothing from that design.

- D7.
  If Q1 chooses C,
  the step 19 setup writes a sudoers line naming only the helper.
  If Q2 chooses (b) or (c),
  the handler gains a single-threaded entering child;
  both are dependencies of those answers,
  not of this text.

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
  the paragraph that says the caller property carries the wrapper's identity rather than each chain entry,
  and why.

## What this revision left as it was

The sections "The open items and the words that decide them",
"What this design read" apart from the additions named,
the kernel and OpenZFS facts F7 to F22 apart from the rewrites of F9,
F14,
F16 and F22,
"What this design does not cover"'s route-level exclusions,
and the reference definitions were left as they were,
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
