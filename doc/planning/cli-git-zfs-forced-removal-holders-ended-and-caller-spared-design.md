# The forced arm of a removal: the reach in a shared namespace, the wait, and the script unit

Non-final,
unapproved design proposal by an agent,
written in the round whose commits are listed under "Revisions".
It is not approved by the project owner and nothing in it is final.
It designs the two open items of the accepted ZFS worktree design that are one mechanism:
"Who ends a holder,
how a holder is identified,
and by what authority",
and "How the wrapper tells its caller's session apart from the other holders".
The record now answers eight of the review's questions ([record][record]):
the channel is the installed root handler;
a seen holder cancels a removal without `--force`;
the forced arm unmounts inside a holder's namespace and ends only the blockers;
the caller's session and every ancestor to the first process are never ended,
and `--force` is refused for a service-manager parent;
the design must find a way to unmount the copy inside a namespace whose mounts are shared;
the signal is `SIGTERM`,
a wait,
then `SIGKILL`;
a forced request's authority lasts only while the command that asked is running;
and a forced removal is refused also when the command's session was started directly by the service manager.
The shared-namespace answer is a direction:
this document designs the ways and recommends one,
a guest measures it,
then the project confirms.
The record's correction withdraws the sentence that the answer was "the authorisation ...
for the unmount of the clone's own copy and nothing wider";
how far the change reaches is this design's to propose,
with its cost stated,
and no way is ruled out as wider than authorised.
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

- The corrected record's "Correction" paragraph and its "One more answer:
  a unit whose main process is a script",
  quoted under "The open items and the words that decide them".

- [The sibling cancelled-removal design][cancelled] at its current tip `2c70b9203c`,
  read last:
  its "The forced arm's failure path",
  "What the caller sees",
  "The stop points",
  its two refusal words,
  and its S8.

- Linux `v7.3` at `fs/pnode.c`,
  `fs/namespace.c`,
  `net/unix/af_unix.c`,
  `fs/proc/base.c` and `fs/namei.c`,
  and `Documentation/filesystems/sharedsubtree.rst`;
  the accepted staging design's "The unmount call from staging" and "The staged-folder record" for the recorded-handle route.

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
the review after the further answers and its kernel readings,
the sibling at its current tip,
and the kernel,
systemd and staging sources named above.
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
  Making a mount private or slave removes it from its peer group and takes it off its master's slave list;
  private clears its master,
  slave keeps it,
  and its own slaves are handed to its master or detached.
  A later shared invents a new group and leaves the master alone;
  a new mount namespace copies the tree,
  with slaves in place of peers only across a user namespace.
  (Source,
  Linux `v7.3`,
  `fs/pnode.c:74` to `:124`,
  `fs/namespace.c:2997` to `:3004` and `:4427` to `:4434`.)

- F50.
  A unix socket bound to a pathname holds the mount of that path ([u->path.mnt]),
  and `/proc/net/unix` prints each socket's inode and its bound path,
  so a socket bound inside the clone holds the clone's mount while the socket's own descriptor carries no file mount id.
  (Source,
  Linux `v7.3`,
  `net/unix/af_unix.c:1403` and `:3558` to `:3597`.)

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
  `fs/fhandle.c` (the capability at `:182` to `:186`);
  `doc/planning/cli-git-zfs-staging-under-cache-design.md`,
  "The staged-folder record" and "The unmount call from staging".)

- F54.
  A thread that does not share its file-system structure has its own working directory under `/proc/<pid>/task/<tid>/cwd`,
  which the task's `/proc/<pid>/cwd` does not show.
  (Source,
  Linux `v7.3`,
  `fs/proc/base.c`'s `tid` links at `:3671`;
  whether the scan enumerates threads is unverified.)

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
  the move and the unmount is an inference from source plus VM items 4 and 15,
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
  It is named here as a constant with no value,
  set by step 17's build only after the owner decides one or M12 sizes it;
  the forced arm does not ship before then.
  The removal's own deadline is not this wait:
  it is a client report deadline.

- L10.
  A service unit's parent is the service manager,
  so `--force` is refused there,
  and also when the command's session was started by the manager (the record's answer on a unit whose main process is a script);
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
  the threads' own working directories and bound unix sockets is new code over the handler's own walk (R3);
  only the descriptor record and the path-use ceiling it reuses are measured.

- L15.
  A mount made private is not restored exactly:
  re-sharing invents a new peer group and the master is cleared.
  A mount made slave keeps its master and a later shared gives a new group with that same master,
  so the slave change is nearly reversible;
  the private one is not.
  The in-place way 8 changes no propagation and leaves nothing to restore.

- L16.
  The shared-namespace answer is a direction:
  this document designs the ways,
  a guest measures the recommended one,
  and the project confirms afterwards.
  Until then nothing here is final.

- L17.
  `mount_setattr` carries a propagation field in the guest's `v6.8` headers (the review read `include/uapi/linux/mount.h` 134 and the `statmount` fields at 169 to 171),
  but the design still uses `mount(2)` on a descriptor's `/proc` path and does not depend on it;
  the guest's behaviour of either call is unrun.

- L18.
  The guest image runs `6.8.0-147-generic`,
  whose syscall table ends at 461 and has no `listns`;
  the taskless-namespace arm and the production kernel's generation (`7.3.5-1-cachyos`) must be measured on a guest of that generation,
  and the arms record `uname -r` and `zfs version`.

- L19.
  A directory created in the checkout's filesystem by way 7 is visible to the host and to snapshots;
  a kill can leave it,
  and this design's answer is a recorded name the next run or `doctor` removes,
  which is unmeasured.

- L20.
  A hardened unit (`ProtectHome=read-only`,
  a read-only parent) refuses way 7's directory write;
  way 8,
  which writes nothing,
  is the only way there if its handle exists;
  both are unmeasured.

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
the removal of <slot> was asked to end the holders, and this command was started by the service manager: its <parent | session leader's parent> is the manager (pid <pid>, <systemd or systemd --user>), which cannot be told apart from the process that asked for it. If you did not start this removal as a service unit, the command was reparented to the manager because its own parent exited, or it runs in a console login's session; nothing was changed and nothing was ended: the clone, its snapshots and the Git registration are as they were. Run the removal as a child of the process that wants it: from that process's own shell, or under `systemd-run --user --scope` started by that shell; in a console login, start it under `setsid` so its own session begins at the command. Do not run it as a service unit.
```

The consequences,
stated:
a unit whose main process is a script is refused (the new answer);
a console login's shell has `login` as its session leader,
started by the manager,
so a `--force` removal typed there is refused and the text says to use `setsid`;
an orphaned wrapper is refused with the orphan sentence;
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
and for a socket its bound path from `/proc/net/unix` compared with the copy's mount point (F50).
The whole matcher is new code over the handler's walk (L14).
A read the handler cannot make is a blind spot,
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
    or way 7's slave change,
    directory and move),
    then `SIGTERM` to every blocker,
    a wait,
    `SIGKILL` to the survivors (the wrapper's pidfd checked before each signal),
    then the unmounts ("The reach").
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
with no fallback to ending a namespace.
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
So this section proposes,
states each way's cost,
and recommends one for the project to confirm after the guest.

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
a mapped file,
its root or a bound unix socket,
makes the plain unmount refuse `EBUSY` (F17,
F43,
F50);
the first is a mount and no signal clears it,
the second is a blocker and is ended.

### What the kernel does with propagation

Every way below is judged by these rules,
read at Linux `v7.3` (F47 to F49):

- An unmount reaches the mounts that receive propagation from the **unmounted mount's parent**,
  never a set chosen by the unmounted mount's own type (`fs/pnode.c:572` to `:607`,
  the busy check at `:501` to `:531`,
  and `Documentation/filesystems/sharedsubtree.rst:566` to `:568`).
  The copy's own `shared:` or `private` state therefore decides nothing about where its unmount goes;
  the parent's state does.
- Propagation runs from a mount to its peers and its slaves,
  never up to its master (`sharedsubtree.rst:278` to `:283`),
  so an unmount inside a service's namespace cannot reach the host's mount.
- `move_mount` refuses only a source whose parent mount is shared (`fs/namespace.c:3783` to `:3790`);
  a non-shared destination receives no propagation of the move (the table at `:2605` to `:2623`).
- Making a mount private or slave removes it from its peer group and takes it off its master's slave list;
  private clears its master,
  slave keeps it;
  its own slaves are handed to its master,
  or detached when it has none (`fs/pnode.c:74` to `:124`).
  A later shared invents a new group and leaves the master alone (`fs/namespace.c:2997` to `:3004`).
- A new mount namespace copies the tree;
  mounts become slaves rather than peers only across a user namespace (`fs/namespace.c:4427` to `:4434`).
  With systemd's recursive slave-then-shared setup (F38),
  a unit's rows are `shared:N` with no `master:` for mounts that were private on the host,
  and `shared:N master:M` for mounts that were shared and slave;
  the review measured the latter state,
  which is the host before the route's own private-checkout repair.

The mount that carries a slot's folder is the checkout's own mount,
because the accepted design makes the checkout private before each mount operation;
the design reads which mount that is and refuses when it is not (the preflight).

### The ways out of a namespace whose mounts are shared

Eight ways were weighed;
"private" and "slave" mean `mount(2)` with `MS_PRIVATE` or `MS_SLAVE` on a descriptor's `/proc` path (F41,
F47).

#### Way 1: make the copy and its parent private, then move and unmount

It changes two mounts:
the parent,
whose privacy stops the propagation,
and the copy,
whose privacy by F47 buys nothing and cannot be undone exactly.
The service loses the host's later mounts and unmounts under the parent,
so other slots' clones copied into it stay mounted there;
its own nested slaves are re-pointed.
A kill leaves both private,
and the copy at its path or the destination.
It names everything by descriptor after the checks.
It is way 3 plus a needless change.

#### Way 2: make only the copy private, then unmount it in place

It changes the copy only,
and that change does not stop the unmount:
by F47 the parent is still shared,
so the unmount reaches the parent's receivers exactly as way 4 does.
The service keeps everything;
the receivers lose their copies.
A kill leaves the copy private,
which no later share restores exactly.
It names the copy by a path or a parent descriptor and a name,
both of which the accepted staging design forbids acting on.
It is way 4 plus a needless change.

#### Way 3: make only the parent private, then move and unmount at the destination

It changes the parent only,
from the act until the namespace dies;
nothing restores it (L15).
The unmount has no receivers,
because the parent is private before it (F47).
The service loses the host's later mounts **and unmounts** under the parent,
so a shared host checkout's other slots stay mounted in it after the host removes them,
and its nested slaves are re-pointed to the parent's master.
A kill leaves the parent private,
the copy at the destination and the directory there.
It names everything by descriptor.

#### Way 4: plain unmount of the copy in place, changing nothing

It changes nothing,
and the unmount reaches the parent's peers and slaves (F47);
a receiver holding the same copy loses it or the unmount refuses `EBUSY` until its blockers are ended,
so all blockers in all receiving namespaces must be ended first.
The service keeps everything;
the receivers lose their copies;
the host's mount stays,
because it is a master,
not a receiver.
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

It changes every mount in the namespace,
for its life;
the unit receives no host mount change anywhere and every nested relation is re-pointed.
A kill leaves all of it private.
It names everything by descriptor.

#### Way 7: make the parent a slave, then move and unmount at the destination

It changes the parent only,
from shared to slave:
by F49 a slave keeps its master,
so the service keeps receiving the host's mounts and unmounts under it,
and loses only what it forwards to namespaces made from it;
its own slaves are re-pointed to the parent's master (F49).
The unmount has no receivers,
because the parent is not shared (F47);
the move passes,
because the parent is not shared (F48).
A later shared gives a new group with the same master,
so the change is nearly reversible (L15).
A kill leaves the parent slave,
the copy at the destination and the directory there.
It names everything by descriptor.
It is the record's own example with the smallest change that satisfies the kernel.

#### Way 8: unmount the copy in place by a recorded handle, changing nothing

It changes no propagation,
makes no move and writes no directory:
a descriptor opened from a handle the add recorded for the slot's underlay names the directory the copy covers without crossing the mount (F53),
and `umount2("/proc/self/fd/<n>", 0)` unmounts the topmost mount at that dentry,
which is the copy (F31,
F32).
The service loses the copy and nothing else;
the receivers are the parent's peers and slaves,
so it is usable only when the preflight shows no receiver outside the namespaces the act already handles (F47);
where a receiver is one of those,
its copy is unmounted too or refuses until its blockers are ended,
so all blockers in all such namespaces are ended before any unmount.
A kill leaves nothing to restore.
It names everything by descriptor,
and it needs a handle recorded before the clone was mounted,
which is a build change (D7);
clones whose handle predates it fall to way 7.

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
`mount_setattr`'s propagation field names the mount only by descriptor and could replace the `mount(2)` calls,
but the design does not depend on it (L17).

### Ranking and the recommended way

The ranking is 8 > 7 > 3 > 1 > 4 > 2 > 5 > 6.
It agrees with the review's ranking;
I weigh the same reasons and add one note:
way 8 is conditional on a recorded handle and an empty receiver set,
so the design recommends it where the preflight proves both,
and way 7 otherwise,
as one policy ("8 where it fits,
else 7,
and never private").

- 8 over 7:
  no change,
  no move,
  no write;
  but it needs the handle and no receiver outside the act,
  and it is unmeasured.
- 7 over 3:
  the service keeps receiving the host's mounts and unmounts;
  3 cuts both off.
- 3 over 1:
  the same effect with one irreversible change fewer;
  1's copy-private step buys nothing (F47).
- 1 over 4:
  1 names everything by descriptor and propagates nothing;
  4 names the copy by a path the accepted staging design forbids.
- 4 over 2:
  2 is 4 with a change that buys nothing.
- 2 over 5 and 5 over 6:
  5 is a detach the accepted rules forbid;
  6 changes every mount for good.

The recommended policy is:
way 8 where the preflight proves a recorded handle and no receiver outside the act's namespaces;
otherwise way 7,
with the destination directory named and removed as below;
never make a mount private;
and refuse,
naming the holder,
where neither fits.
The project confirms this policy after M1 to M3 (V5).

### The recommended ways, step by step

After the preflight ("The preflight") and the all-namespace structural stage,
for one holder namespace:

**Way 8,
in place by the handle.**

1.  The preflight proved the handle:
    the add recorded the slot's underlay handle before the clone was mounted,
    and the handler opens it on the checkout's mount in the holder's namespace (F53).
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
    the copy's mount id is gone from the namespace,
    and no row of the dataset remains.

**Way 7,
the parent a slave.**

1.  It opens the copy's and the parent mount's roots,
    checked by `statx` mount id and `fstatfs` against the namespace's rows,
    and makes the parent a slave with `mount(NULL, "/proc/self/fd/<parent fd>", NULL, MS_SLAVE, NULL)`;
    it re-reads the parent row:
    no `shared:`,
    its `master:` as before (F49).
2.  It creates one directory on the parent mount's root,
    mode `0700`,
    named with the request's nonce,
    and opens it;
    the attach is on a non-shared mount and does not propagate (F37).
3.  It moves the copy with `move_mount(copy_fd, "", destination_fd, "", MOVE_MOUNT_F_EMPTY_PATH | MOVE_MOUNT_T_EMPTY_PATH)` (F40),
    closes the copy's descriptor,
    and re-reads that the copy's id is topmost at the destination.
4.  It calls `umount2("/proc/self/fd/<destination fd>", 0)` and re-reads that the id is gone and no row of the dataset remains.
5.  It removes the directory with `unlinkat` on the parent's descriptor and the recorded name,
    and reports the one propagation change and the directory's life.

The destination is where the parent's root is,
which is the checkout's own mount:
the host,
every namespace and a ZFS snapshot taken in the window see a `0700` directory named with the nonce,
`git status` in the checkout sees it,
and a kill leaves it.
The handler removes it after the unmount;
the receipt records the name;
a later run and step 18's `doctor` remove a leftover directory whose name carries a recorded nonce and which is empty (L19).
A hardened unit (`ProtectHome=read-only`) has a read-only parent,
so way 7's directory cannot be made and is refused;
way 8,
which writes nothing,
is the way there when its handle exists (L20).

### The preflight

No forced removal may end a process or change a mount before it knows it can finish with every holder namespace it will meet (R2).
The preflight is read-only and runs over every holder namespace,
all of them,
before the first structural act,
the first signal and the first unmount:

- It forks one single-threaded child per namespace,
  enters it (`setns` on the scan's task's namespace,
  a `listns` descriptor for a taskless one),
  and reads `/proc/self/mountinfo` there (F14,
  F15).
- It checks the parent:
  the mount the copy sits on is the checkout's own mount,
  by its source,
  root and mount id;
  anything else is refused,
  because the ways change a mount of the checkout and not `/`,
  `/home` or another tree.
- It runs every structural check for each dataset row:
  exactly one own-root copy,
  not the namespace's root,
  no child mount below the copy,
  no second mount of the dataset,
  no `MNT_LOCKED` the move would refuse (F40),
  the tags recorded,
  the parent's root path and mount id.
- It reads every receiver of the parent,
  in every namespace it can reach:
  a row whose `shared:` group or `master:` equals the parent's group is a receiver;
  way 8 requires that no receiver is outside the namespaces the act handles,
  and way 7 does not care.
- It checks the handle for way 8 (does the add's record carry one,
  and does it open) and,
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
  and whether a pidfd can be opened for it.
  A holder it cannot end refuses here,
  before any signal,
  as does a blind spot.
- A bound unix socket inside the copy is a holder:
  its bound path is read from `/proc/net/unix`,
  its owning task from the socket's inode in `/proc/<pid>/fd`,
  and that task is ended like any other blocker (F50).
- A structural refusal,
  an unenterable namespace,
  a protected holder,
  an unendable holder,
  a blind spot,
  a parent that is not the checkout's,
  a receiver outside the act for way 8,
  or no destination for way 7 refuses the removal before anything is ended or changed.

The order is fixed and all-namespace first:
the preflight over every holder namespace;
then the structural stage over every namespace (way 8's open and check,
or way 7's slave change,
directory and move);
then the signal stage over every namespace;
then the unmounts over every namespace.
The structural stage comes before any signal because a busy mount can be moved (F17's check does not stop a move) and the kernel can refuse that stage for a cause reading can know;
after it,
only a blocker that will not die and an `EBUSY` race follow an ending,
and the report names what was ended and what changed.
Way 8's unmount has receivers,
so all blockers in all receiving namespaces are ended in the signal stage before any unmount.

### The signal

The further answer decides it:
`SIGTERM`,
a wait for the process to exit,
then `SIGKILL` if it has not.

- Where the wait sits:
  in the signal stage,
  after the preflight and the structural stage,
  over every namespace's blockers;
  every blocker gets `SIGTERM` through its pinned pidfd,
  the handler waits on their pidfds,
  and the survivors get `SIGKILL` through the same descriptors.
- The wait's length is a constant this document names without a value,
  because the record says it ships no number until the project decides one or a measurement sizes it.
  Step 17's build sets it only after the owner decides one or M12 sizes it,
  and the forced arm does not ship before then (L9).
- During the wait the handler polls the blockers' pidfds and the wrapper's.
  If the wrapper's pidfd reports exit,
  it sends no `SIGKILL`,
  stops the act and files `caller-gone`;
  the wrapper stays alive for the act,
  and its own report deadline prints that the request still stands rather than killing the act.
- The wrapper's identity is checked immediately before the `SIGTERM` and again before the `SIGKILL`.
- A process that exits during the wait costs nothing:
  no `SIGKILL` is sent,
  its exit is recorded as "exited after `SIGTERM`",
  and the act goes on as soon as all blockers are gone.
- A blocker that ignores `SIGTERM` is sent `SIGKILL` when the wait's constant expires and the wrapper is still alive;
  the handler then waits for the kill to be observed,
  bounded by the same constant;
  a blocker that has already gone when its signal is due is recorded as gone.

### The blockers, and what the matcher reads

A blocker is a task that holds the copy's mount through any of:
its working directory,
an open descriptor,
its running executable (`/proc/<pid>/exe`),
a mapped file (`/proc/<pid>/map_files/*`),
its root (`/proc/<pid>/root`),
a thread's own working directory (`/proc/<pid>/task/<tid>/cwd`),
or a unix socket bound to a path inside the copy (`/proc/net/unix` plus the socket's inode in `/proc/<pid>/fd`),
matched by the mount id the kernel records for each (F36,
F43,
F50).
The earlier matcher read the working directory and descriptors;
`exe`,
`map_files`,
`root`,
the threads and the sockets are this revision's additions,
and the whole matcher is new code (L14).
A task under a child mount is not a blocker of this unmount:
the child mount is,
and it is refused.
A blocker in the protected set refuses the whole removal at the preflight.
A blocker is ended by the signal order above;
a blocker the preflight proves cannot be ended refuses there;
a blind spot refuses there.

When the unmount is refused `EBUSY` and the matcher found no blocker,
the report says the reads were empty over those seven kinds,
that a reference the design cannot see remains,
and refuses `holder-unseen` naming the copy,
the namespace and the errno.
It never retries blind.

### What still refuses

The shared copy and the shared parent are no longer refused;
they are the case the recommended ways handle.
What the design still refuses:

- a namespace it cannot enter;
- the handler's own namespace,
  and any namespace containing pid 1,
  whose clone mount is the host's and whose unmount is the staging turn's;
  the act there ends path holders only;
- a namespace whose root mount is the copy,
  because the move needs a parent (F40);
- a mount under the copy,
  a holder no signal clears;
- a second mount of the dataset in the same namespace;
- a holder the preflight proves cannot be ended,
  and a blind spot;
- a parent mount that is not the checkout's own;
- for way 8,
  a receiver outside the namespaces the act handles,
  or no recorded handle (it falls to way 7);
- for way 7,
  a read-only parent,
  no room for the directory,
  or a private change the kernel refuses;
- an `EBUSY` at the unmount after the empty search.

### What the service and other namespaces see

With way 8 nothing changes but the copy's disappearance,
and the parent's receivers lose their copies or refused until their blockers were ended.
With way 7 the parent is a slave from the act until the namespace dies:
the service keeps receiving the host's mounts and unmounts under the parent,
stops forwarding its own later mounts under it to namespaces made from it,
and its nested slaves are re-pointed to the parent's master;
the host's mount is unchanged and its slave list loses one entry,
which no table shows.
With way 3 (not recommended) the service loses the host's later mounts and unmounts under the parent,
so other slots' clones copied into it stay mounted after the host removes them,
and its nested slaves are re-pointed.
The destination directory of way 7 is visible to the host,
`git status` and snapshots while it lives;
a kill leaves it.
The host's own mount and every namespace that is not a receiver are unchanged,
and M8 records that.

### The undo, and what a kill leaves

Way 8 changes nothing and leaves nothing to undo.
Way 7's slave change keeps the master:
a later `MS_SHARED` gives a new group with the same master,
so it is nearly reversible;
the original group number is not restored.
Way 3's and way 1's private change cannot be undone (L15).
A kill after the structural stage leaves the parent slave or private,
the copy at the destination in that namespace,
and the directory in the checkout's filesystem;
the next run reads the state,
reports it,
moves the copy back when its recorded path still carries the parent and no mount,
and removes a leftover directory whose name carries the recorded nonce (L19).

### The checks that exist only for a hostile account

The threat model is the project's:
the account is trusted,
and root and the account are not a security boundary here ([staging][staging],
lines 221 to 237).
These checks survive it as guards against a bug or a stale record:
the copy's and parent's mount-id checks,
the post-change tag reads,
the destination and handle descriptors,
the nonce-named `0700` directory,
the wrapper's pidfd before each call,
the dataset GUID re-read,
and the socket and thread reads.
A check that the parent directory is owned by root and not writable by the account exists only against a hostile account and is not adopted;
every call after the checks is descriptor-named,
and the mount-id checks are what the protection rests on.

### The confirmation

The project confirms the policy after M1 to M3,
in the review's words:
a forced removal may,
inside the mount namespace of a service that still holds a copy,
unmount in place by a recorded handle when nothing else shares the mount the clone sits on,
otherwise change that one mount (the checkout's own) from shared to slave,
create one empty directory,
move the copy onto it,
unmount it there and remove the directory;
never make a mount private,
never touch the host's mounts or a namespace that is not a holder,
and refuse and name the holder when it can do neither.
What the owner accepts with it:
the service keeps running and keeps receiving the host's mounts and unmounts under the checkout,
and stops forwarding its own later mounts to namespaces made from it;
a short-lived directory appears in the checkout's filesystem,
visible to the host,
and a kill can leave it;
a kill can leave the parent slave and the copy at the directory;
and the record's own private example remains an option with the cost that a shared host checkout's service stops receiving the host's unmounts there (V5).

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
  and a namespace's tasks and threads are all enumerated,
  so a protected task anywhere in it refuses the namespace.
- The number of tasks and namespaces one act reaches is bounded by the scan's own bounds (F23),
  and a scan that stopped at its bound is not a clear answer.
- Every task is pinned and proved before it is signalled,
  and a task whose proof fails is reported as a blind spot;
  at the preflight a blind spot refuses the removal.
- The wrapper's pidfd is checked immediately before every signal and every unmount,
  and again before the structural stage's mount changes,
  and it is in the poll set the act waits on.
- The act's waits are the pidfds' exit (`POLLIN`),
  the wait between `SIGTERM` and `SIGKILL` (the constant L9 names without a value),
  and,
  for an unmount,
  the kernel call's return.
- The wrapper stays alive for the act.
  Its own report deadline prints that the request still stands rather than ending the act;
  if the wrapper does die,
  the handler sends no `SIGKILL`,
  stops and files `caller-gone`.

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
  map_files,
  root,
  a thread or a socket),
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
  the namespace and whether it was removed or left;
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

The wrapper prints the same facts in its report.
It does not print them for a holder of the protected set;
that case has its own refusal text.

## The refusal, and what it leaves

Three refusals are the forced arm's,
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
  a private or slave change refused,
  no destination,
  no room,
  a receiver outside the act for way 8).
- `holder-unseen`:
  an `EBUSY` at the unmount after the matcher found no holder,
  meaning a reference this design cannot see.

A dead wrapper is `caller-gone` only;
it is not a `holder-unendable` (the review's L1).
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
<the holder, its namespace and the reason: a holder this removal could not end, a namespace it could not enter, a mount under the clone, a second mount of the dataset, a namespace whose root is the clone, a parent that is not the checkout's, a private or slave change the kernel refused, no destination or no room, a receiver outside the act, or an unmount that refused after an empty search>.
<what was ended before the refusal, which namespaces' propagation changed, and which directory was left>.
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
its refusal words,
and the fact that this design never writes the intent.
The sibling at `2c70b9203c` names both `caller-protected-holds` and `holder-unendable`,
splits its refusals by what a preflight can see,
and leaves only S8 open;
its fix round takes `holder-unseen` and the destination's leftover directory as new items.

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

The wrapper stays alive for the act:
its own report deadline prints that the request still stands and does not end the act,
and the act's only time limit is the wait constant L9 names without a value.
If the wrapper does die,
the handler sends no `SIGKILL`,
stops and files `caller-gone`;
a later trigger that finds a forced request whose wrapper is gone declines the force word,
files `caller-gone`,
and does only the non-force work the intent asks.
A wrapper that is alive but whose recorded parent has changed or died has its force word declined the same way,
and prints that nothing was ended and that the removal should be run again as a child of the process that wants it.

A holder that attaches after the scan is met by the next scan,
which the accepted design already requires before the first snapshot delete and before the head destroy,
and every such scan under the force word acts on the holders it sees.
A holder that attaches after the last scan shows up as `dataset is busy` at the head destroy,
the accepted residual for a holder the scan cannot show;
the retry is bounded by the removal deadline.
A supervised blocker that is ended may be restarted by its supervisor into a fresh copy of the clone while the host still has it mounted;
the next scan meets it,
bounded by the removal deadline,
and the report says the holder returned (the review's L7).

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
a socket or a thread whose read the handler could not make,
is a named blind spot and refuses the removal at the preflight;
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
  `bind`;
  this design adds no key and carries the holder list in the scan file and the refusals in bounded failed-step words (F24).
  A key and the words can coexist.

- The failure path.
  The sibling owns what a removal leaves when it does not finish,
  forced or not,
  in its "The forced arm's failure path";
  this design states only the forced arm's refusals,
  with its three words,
  and writes no intent.
  The sibling at `2c70b9203c` names `caller-protected-holds` and `holder-unendable`,
  splits its refusals by what a preflight can see,
  and leaves only S8 open;
  this design adds `holder-unseen` and the destination directory's removal as items its fix round takes.

- The protected-holder refusal.
  Without `--force` a holder in the protected set refuses and the sibling cancels and reattaches;
  with `--force` the same refusal is the forced arm's.
  Both documents use the same word.

- The claim.
  The sibling says the handler never reads the account's claim;
  this design's liveness proof is the wrapper process,
  not the claim.

- The wait and the destination.
  The wait's constant and the destination's directory are this design's,
  not the record's;
  V5,
  V7 and L9 state them,
  and the record's correction leaves their scope to this design.

## Choices this design marks for the project's veto

Each entry is a writer's choice the design adopts so it is whole.
A veto takes the other option and the sentences that say otherwise.

### V1. Where the scan's finding is carried

Adopted:
the scan's own text in a second root-owned file beside the receipt.
The alternatives are the text inside the receipt,
an on-demand privileged run,
and outcome only;
the ranking stays beside the receipt > inside > on demand > outcome only,
because the first keeps both grammars and readers unchanged.

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

The ranking is 8 > 7 > 3 > 1 > 4 > 2 > 5 > 6,
agreeing with the review's;
the reasons are in "Ranking and the recommended way".
The recommended policy is way 8 where the preflight proves a recorded handle and no receiver outside the act's namespaces,
otherwise way 7,
and never a private mount.
The record's own example (the parent made private) is way 3 and stays an option,
with the cost that a shared host checkout's service loses the host's later mounts and unmounts under the parent.
The project confirms the policy after M1 to M3;
this entry is a recommendation,
not an adoption (the review's L4).

### V6. Whether a refused unmount after the structural stage is moved back

Adopted:
move the copy back when its recorded path is on the same parent and carries no mount,
report both paths,
the propagation change and the directory,
and remove the directory when it is safe.
A veto leaves the copy at the destination and reports it;
the ranking stays move back > leave and report > detach.

### V7. Whether the report, `doctor` and the next run name a changed propagation and a left directory

Adopted:
the receipt and the report name every propagation change and every destination directory,
with the namespace and mount ids,
step 18's `doctor` prints them,
and a later run removes a leftover directory whose name carries a recorded nonce and which is empty.
A veto keeps the report only,
and leaves a left directory to the person;
the own-record option has no reader or pruner.
Ranking:
report,
`doctor` and the next run > report only > own record.

## Questions only the project can answer

The record decided S4,
S6 and S13:
the signal is `SIGTERM`,
a wait,
then `SIGKILL`;
a forced request's authority lasts only while the command that asked runs;
and a forced removal is refused also when the command's session was started directly by the service manager.
It answered S12 by choosing "design an unmount there",
and the project confirms the recommended policy after the guest (V5).
S7 is decided by the record's "A cancellation after part of the history is gone";
S9 and S10 are the sibling's veto entries;
S8 is the sibling's question.
S1,
S2,
S3 and S5 are the record's first four answers.
Only this one is still open here,
numbered as the reviews number it.

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
- Step 10 or 14,
  whichever records the slot's durable identity,
  also records the slot underlay's file handle before the clone is mounted,
  which way 8 needs (F53,
  D7).
  Clones that predate it fall to way 7.
- Step 16 owns the scans' call sites and the deferred-destroy checks ([deferred][deferred]);
  the scan file's publisher and reader are built with the first consumer,
  step 16 or step 17.
- Step 17 builds this design:
  the caller and force properties,
  the chain read with its parent anchor and hop checks,
  the pidfd on the wrapper,
  the service-manager and session checks,
  the refusals and their receipt words,
  the pre-move scan,
  the read-only preflight (structure,
  parent-is-the-checkout,
  receivers,
  handle,
  room,
  endability,
  sockets,
  blind spots),
  the structural stage (way 8's open or way 7's slave change,
  directory and move),
  the wait constant,
  the signal order,
  the unmounts,
  the directory removal,
  the logging and the tests.
  The wait constant is set only after the owner decides one or M12 sizes it;
  until then step 17 cannot ship the forced arm (L9).
- Step 18 builds the `doctor` reading:
  the last scan file,
  the live scan mode,
  the holders named by namespace and task,
  the protected set,
  the propagation changes a removal reported,
  and a leftover destination directory whose name carries a recorded nonce and which is empty (V7).
- Step 19 builds the installed handler's unit,
  and that unit must carry no mount-namespace option (`PrivateMounts=`,
  `ProtectSystem=`,
  `ProtectHome=`,
  `PrivateTmp=` and the like),
  because the handler must run in the host mount namespace (A8,
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
    and a wrapper whose recorded parent changed decline the force and print the declined text;
    device-free.
8.  The wrapper ends between two passes and during the wait:
    no second pass,
    no `SIGKILL`,
    `caller-gone`;
    guest.
9.  The preflight refuses before any act for each cause reading can know:
    a structural refusal in a second namespace,
    a holder the preflight proves cannot be ended,
    a blind read,
    a bound unix socket inside the copy,
    no room for the directory,
    a parent that is not the checkout's,
    and (for way 8) a receiver outside the act;
    no signal and no mount change anywhere;
    guest.
10. The service-manager checks:
    the wrapper's parent,
    the session leader's parent,
    a look-alike cgroup under the account's own subtree,
    an orphaned session whose leader's record cannot be read,
    a script-main unit (refused),
    and the allowed shapes (Pi,
    Claude Code,
    a `--scope` child);
    device-free over fixture `/proc`,
    guest for the real runs.
11. The default sandboxed unit is reached by way 7:
    its table before (all rows `shared:`),
    the parent changed to slave,
    the copy moved and unmounted,
    the directory removed,
    the head destroy done,
    the unit alive;
    guest,
    on the production kernel.
12. Way 8 in place by the recorded handle:
    no change,
    no move,
    no directory;
    the head destroy done;
    guest.
13. A `MountFlags=private` unit and an `unshare --mount` namespace,
    and a taskless namespace on a kernel with `listns`;
    guest.
14. The matcher finds `exe`,
    a mapped file,
    a chroot,
    a bound socket and a thread's own working directory;
    with an `EBUSY` after an empty search refusing `holder-unseen`;
    guest.
15. The signal order:
    a `SIGTERM` handler exits and is not killed,
    an ignorer is killed when the wait constant expires,
    and the wrapper's death sends no `SIGKILL`;
    guest.
16. Receivers:
    a peer namespace inside the unit and a slave namespace,
    before and after;
    the parent's slave's `master:` is re-pointed and no other namespace's copy is touched;
    guest.
17. The host states:
    a private checkout and a shared one;
    a host mount and a host unmount of another slot's clone after the act,
    with the service's view recorded;
    guest.
18. A hardened unit (`ProtectHome=read-only`):
    way 7 refused,
    way 8 reached when the handle exists;
    guest.
19. The structural refusal after the structural stage:
    a kill between the slave change and the move,
    and between the move and the unmount;
    the copy moved back,
    the directory and the change named,
    the ended set kept;
    guest.
20. `doctor` names holders,
    the protected set,
    the propagation changes and a leftover directory,
    without root;
    device-free.

## Measurements owed

M1 to M12 are the arms of one guest run,
each a specification for the build step that builds the forced arm:
the state,
the one action,
the expected result and the bound on its wait.
The fixture is [measurements][measurements],
with one change:
the run must be on the **production kernel's generation**,
Linux `7.3` (the host runs `7.3.5-1-cachyos`),
because the guest's `6.8.0-147-generic` has no `listns` and this design's citations are at `v7.3` (L18).
Each record carries `uname -r`,
`zfs version`,
and whether `listns` and `mount_setattr`'s propagation field answered.
Every arm records the unit's or namespace's mount table before and after with the `shared:`,
`master:` and `unbindable` tags,
the host's table,
another unit's table,
a peer namespace's and a slave namespace's,
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
  with no `MountFlags=`;
  inside it,
  a peer namespace made with `unshare --propagation unchanged` and a slave namespace made so that it receives from the unit;
  a second slot's clone copied into the unit when it started;
  the host checkout both private and shared,
  in two arms.
  Action:
  one `git worktree remove --force <slot>` on the first clone,
  way 7.
  Expected:
  the parent becomes slave (no `shared:`,
  its `master:` as before),
  the copy is moved to one directory and unmounted there,
  the directory is removed,
  the unit is alive,
  the head destroy exits 0;
  the peer and slave receivers' tables and copies are recorded;
  the host's mount and every non-receiver namespace are unchanged;
  a host mount created under the parent after the act appears in the unit (slave keeps the master),
  and a host unmount of the second slot's clone after the act disappears from the unit;
  in the shared-checkout arm the same holds,
  which is the cost way 3 would lose.
  Bound:
  the calls and the wait return before the sweep's bound;
  the record notes each elapsed time.
  Control:
  the same arm without `--force`,
  recording the cancel and both copies.

- M2.
  State:
  M1's unit with `MountFlags=private`;
  an `unshare --mount` namespace;
  a taskless namespace pinned by a descriptor.
  Action:
  one `--force` removal for each.
  Expected:
  the private units and namespaces need no propagation change;
  the copy is unmounted;
  the taskless one is found by `listns` (production kernel),
  entered in a single-threaded child and unmounted;
  the head destroy succeeds.
  Bound:
  as M1.

- M3.
  State:
  M1's unit,
  with a slot handle recorded by the add before its clone was mounted.
  Action:
  one `--force` removal,
  way 8.
  Expected:
  no propagation change,
  no move,
  no directory;
  the copy is unmounted in place;
  the head destroy succeeds;
  the unit and the host unchanged.
  Bound:
  as M1;
  the record notes the handle's open and the unmount.

- M4.
  The positive control for the propagation rule.
  State:
  the parent shared,
  a slave namespace holding its own copy of the clone,
  the copy made private in the unit.
  Action:
  a plain unmount of the copy in place.
  Expected:
  the slave's copy vanishes,
  which shows that the unmount reaches the parent's receivers and that the copy's own privacy does not stop it (F47);
  the same arm with the parent made private first leaves the slave's copy mounted.
  Bound:
  no wait.

- M5.
  The preflight.
  State:
  each of:
  a holder in uninterruptible sleep (a FUSE or NFS arm),
  a unix socket bound inside the copy,
  no room for the directory,
  a parent that is not the checkout's mount,
  a receiver outside the act for way 8,
  a blind read.
  Action:
  one `--force` removal for each.
  Expected:
  each refuses at the preflight,
  before any signal and any mount change;
  the socket's owning task is named;
  nothing was ended.
  Bound:
  no wait.

- M6.
  The signal.
  State:
  a blocker that handles `SIGTERM` and exits;
  one that ignores it;
  the wrapper killed during the wait.
  Action:
  one `--force` removal for each.
  Expected:
  the first is not sent `SIGKILL` and its exit is recorded;
  the second is sent `SIGKILL` when the wait constant expires and the wrapper lives,
  and its status is recorded;
  the third stops the act,
  sends no `SIGKILL` and files `caller-gone`.
  Bound:
  the wait ends at each exit,
  at the constant,
  or at the wrapper's death;
  the record notes the constant used.

- M7.
  The matcher.
  State:
  a test binary running from the clone,
  one mapping a library from it,
  one chrooted into it,
  a task with a unix socket bound inside it,
  and a thread with its own working directory in it.
  Action:
  one `--force` removal.
  Expected:
  each is found through `exe`,
  `map_files`,
  `root`,
  `/proc/net/unix` and `task/<tid>/cwd`,
  ended,
  and the unmount succeeds;
  an `EBUSY` after an empty search refuses `holder-unseen`.
  Bound:
  as M1.

- M8.
  Nothing wider.
  State:
  M1's unit with its peer and slave receivers;
  the host's table;
  another unit's table;
  an unrelated namespace's.
  Action:
  one `--force` removal,
  way 7 and separately way 3.
  Expected:
  way 7 changes only the holder's parent,
  re-points its slaves' `master:` to the parent's master,
  and leaves every other row and copy as it was;
  way 3 also cuts the unit off from the host's later mounts and unmounts in the shared-checkout arm;
  the host's mount and its slave list are unobservable but unchanged in every table.
  Bound:
  no wait.

- M9.
  The refusals.
  State:
  a namespace that cannot be entered;
  a copy with a child mount;
  a second mount of the dataset;
  a namespace whose root is the copy;
  a hardened unit (`ProtectHome=read-only`) for way 7 and for way 8;
  a private change the kernel refuses.
  Action:
  one `--force` removal for each.
  Expected:
  each is refused with its named reason,
  before any signal where reading can know it;
  the hardened unit is refused under way 7 and reached under way 8 when the handle exists.
  Bound:
  the refusal needs no wait.

- M10.
  The service-manager checks.
  State:
  a `systemd-run --user --pipe --wait` unit;
  a system unit;
  a unit whose main process is a script;
  a console-login shell;
  an orphaned session;
  Pi's and Claude Code's shapes.
  Action:
  one `--force` removal for each.
  Expected:
  the first three and the console login and the orphan are refused with the text,
  exit 2,
  no property written,
  nothing changed;
  Pi's and Claude Code's shapes are allowed and their holders are protected.
  Bound:
  the refusal is before any property write.

- M11.
  The kill arms and the leftovers.
  State:
  a kill between the slave change and the move,
  between the move and the unmount,
  and after the directory is made.
  Action:
  the next run and `doctor`.
  Expected:
  the parent stays slave,
  the copy is at the destination,
  the directory is left;
  the next run reports all three,
  moves the copy back when it can,
  and removes the directory when its name carries the recorded nonce and it is empty;
  `doctor` prints the same.
  Bound:
  no wait.

- M12.
  Timings and the wait's sizing.
  State:
  the scan file's publish to the wrapper's read;
  the wait for one and for many blockers;
  the slave change,
  the move and the unmount.
  Action:
  time each,
  interleaved with a control arm in the same round.
  Expected:
  the numbers that size the wait constant (L9) and step 22's removal deadline;
  no value ships before this runs.
  Bound:
  the record states its spread and compares arms in the same round (rule `TVC`).

## Limitations

- Every claim about the dataset releasing,
  the propagation rules,
  the private and slave changes,
  the move,
  the unmount and the handle route is an inference from source and VM items 4 and 15 (L1,
  L12).
- The caller's protected set is a number inside one pid namespace (A1,
  L4).
- The caller property is an assertion the handler ties to a live wrapper and its recorded parent,
  not to a sender (A4).
- The wait between `SIGTERM` and `SIGKILL` has no value yet (L9);
  the forced arm cannot ship until the owner decides one or M12 sizes it.
- The recommended policy changes the holder's parent mount (slave) or uses a recorded handle (way 8);
  both are unrun,
  and the private example stays an option with its cost (L15,
  L16,
  L19,
  L20).
- A confined unit the Pi driver starts is not an ancestor and can be ended if it holds the clone (L13).
- `listns`,
  the handle route and the propagation changes are source-read and unrun (L12);
  M1 to M9 owe them,
  on the production kernel (L18).
- The matcher over `exe`,
  `map_files`,
  `root`,
  threads and unix sockets is new code (L14).
- The service-manager checks and local polkit rules are unverified on this host (L5,
  L11).
- The plan record has no field for "who was ended",
  so S11's first-ranked home is contingent on D4.

## What this design does not cover

- What a cancelled,
  interrupted,
  timed-out or killed removal leaves,
  and the kept `cancel-worktree-removal` command:
  the sibling owns it (D2).
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
- A namespace or a state the reach refuses beyond the refusal and the report:
  no recovery is designed for a copy left at a destination when moving it back also fails,
  and no restoration of a private mount's original group is designed.
- The handler's installation,
  its unit hardening and ZED's own timeouts;
  the unit's mount-namespace constraint is a requirement on step 19,
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
  [The sibling cancelled-removal design][cancelled] at `2c70b9203c` owns what a removal leaves and restores when it does not finish,
  forced or not:
  its "The stop points",
  "The forced arm's failure path",
  "What the caller sees" and "One reversing request,
  written in one place".
  It names both `caller-protected-holds` and `holder-unendable`,
  splits its refusals by what a preflight can see,
  and leaves only S8 open;
  this design adds `holder-unseen`,
  the destination directory's removal and the wait constant as items its fix round takes.
  It adds a sixteenth receipt key,
  `bind`;
  this design adds no key.

- D3.
  [The deferred-destroy checks][deferred] own step 16's checks and call sites.
  This design claims no ownership of them;
  it needs the scan file's reader and V2's rule.

- D4.
  [The destruction plan record][planrecord] owns the plan's home,
  fields,
  readers and lifetime.
  This design needs the record only if S11 chooses the plan entry,
  and then it needs fields for who was ended,
  which namespaces changed propagation and which directory was left.

- D5.
  [The lost-last-event triggers][losttriggers] own nudge merging and the rate rule.
  This design adds no nudge.

- D6.
  [The propagation after a replacement mount][propagation] states "no dependency" on this design.
  This design keeps the scan's visibility unchanged from VM item 15 and needs nothing from that design.

- D7.
  The slot underlay's file handle,
  which way 8 opens,
  is recorded by the add's step that records the slot identity (step 10 or 14),
  before the clone is mounted;
  the clone route's own binding record owns it,
  and this design states what it needs of that record:
  the handle and the record's boot and GUID.
  The wait constant is this design's own,
  set by step 17's build only after the owner decides one or M12 sizes it.
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

The record's answers are built in:
"Who ends each kind of holder" is the chosen channel,
"The reach" is the eight ways with the recommended policy (way 8 where the preflight proves a handle and no receiver outside the act,
otherwise way 7),
the preflight over every namespace,
and the `SIGTERM`/wait/`SIGKILL` order,
"The caller and its protected set" is the session plus every ancestor to pid 1 and the check on the wrapper's parent and the session leader's parent,
and "The refusal" carries the three words,
leaving the unforced cancel for a holder outside the set to the sibling.
The review's H1 is fixed by the corrected propagation rules and the two new ways;
M1 by the wait constant and the all-namespace order;
M2 by the destination's conditions and the handle route;
M3 by the arms on the production kernel;
M4 by the preflight's reads;
the changed facts are F47 to F54.

## What this revision left as it was

The sections "What this design read" (apart from the new review,
its kernel readings,
the corrected record and the sibling tip),
"What this design ran",
the kernel and OpenZFS facts F1 to F46 (apart from the new F47 to F54),
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
