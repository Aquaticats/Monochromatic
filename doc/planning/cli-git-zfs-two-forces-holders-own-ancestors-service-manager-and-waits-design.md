# Two forces at a ZFS removal: whose processes it ends, how a command that ends its own ancestor finishes, under the service manager, and the waits

Status:
a design written by an agent,
**unapproved and non-final**.
It has had one same-vendor gate read,
not an independent review:
a Claude Sonnet pass of 2026-11-28,
whose findings were six High,
seven Medium and five Low.
Every finding is answered in this revision,
either by a change or by a line in "What the gate read got wrong".
No release code,
no test and no configuration was changed by the round that wrote it.
The owner declares it complete,
and every question at its end is the owner's to answer.

## What is asked, what is recommended, and what rests on a measurement

The owner decided on 2026-11-28,
in these words,
"--force overrides anything and delete and destroy no matter what.",
and,
asked how far that reaches,
"Two force does everything.
One force - you pick."
The record's own reading is that
`git worktree remove --force --force <slot>` passes every refusal the removal has
(`doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
"What force does on a ZFS removal,
two refusals without it,
and the foundation pick").
That record names two things its texts did not say how to do:
how a command that ends its own ancestors still finishes its removal,
and by what act two forces get past other processes that hold the clone,
while the forced arm's waits have no value.

Ten decisions follow from those two things.
Each is put to the owner on its own,
with its options,
its visible result and what a yes leaves undecided,
in "The questions for the owner".
This is the set,
with what this design recommends:

1. **Another account's task that holds the clone.**
   End it through the installed root handler (`SIGTERM`,
   the wait,
   `SIGKILL` through a pin),
   because a person's own process cannot signal it.
2. **A task of a unit the caller cannot signal.**
   End it the same way,
   except a task of a system unit,
   which refuses by name:
   a person's typed flag does not stop a machine-wide service.
3. **A locked mount namespace that is not the person's own unit.**
   End its tasks,
   which frees the copy with the namespace.
   No unmount reaches a locked copy,
   and this is the record of 2026-11-22's
   "never fall back to ending every task" that the change amends.
4. **A locked holder that is a service unit of the person's own user manager.**
   Stop it through that manager,
   and end its tasks only where the stop fails
   or the unit comes back.
5. **A command that must end its own session or one of its own ancestors.**
   The removal detaches itself first:
   a child with a session of its own,
   a working directory outside the clone,
   and `SIGHUP` ignored,
   which then writes the request and makes the act.
   No manager,
   no lingering and no second holder of the authority.
6. **A removal the service manager started.**
   Lift today's refusal and name the requester by its own unit,
   read from its unified control-group line;
   a command whose control-group line names no unit still refuses.
7. **Which refusals survive two forces.**
   Keep every one this design lists,
   with the two exceptions
   decisions 1 to 3 already make;
   each is named with its reason so the owner can lift any one by name.
8. **`T_wait`,
   the wait between `SIGTERM` and `SIGKILL`.**
   2000 ms,
   step 22's proposal carried over,
   with what sizes it stated as a ratio.
9. **`T_h`,
   the handler's heartbeat period,
   and its four-period window.**
   1000 ms and 4000 ms,
   step 22's proposal carried over,
   with what contradicts it past about 205 clones stated.
10. **`T_stop`,
    the wait for a locked unit's stop.**
    2000 ms,
    step 22's proposal carried over,
    with the fact that a service's own `ExecStop` may take far longer stated.

What each recommendation rests on,
in one line:

- decisions 1 and 2 rest on a reading of this round:
  an unprivileged caller's
  pin to another account's task answers `EPERM`,
  so the mechanism is the handler's and not the person's,
  and on the accepted channel record of 2026-11-22,
  which gives those tasks to the root handler;
- decision 3 rests on a reading of this round and of the 2026-11-27 round:
  a locked copy answers `EINVAL` to every unmount,
  and its own task's end takes the copy with the namespace;
- decision 4 rests on the manager's own answer,
  measured this round:
  a stop ends the unit and its main task,
  a unit whose task was killed outside its manager comes back within two seconds and holds the
  clone again,
  and the locked shape the route exists for could **not** be made on this round's stack at all,
  which is stated with the reading;
- decision 5 rests on a reading of this round with a pty:
  a child in a dying session that has not detached receives `SIGHUP`,
  and one that called `setsid` in a session of its own does not;
- decision 6 rests on this machine's own control-group lines,
  read this round:
  a command this project's agents run carries a control-group line whose last component is
  `claude-code-bash`,
  which names no unit,
  and one run under `systemd-run --user --scope`
  carries a line that ends `.scope`;
- decisions 8 to 10 rest on step 22's proposals,
  with this round's own readings of a guest that
  is not the owner's stack beside them:
  2000 ms is 9.9 times this round's slowest cooperating exit,
  1000 ms is 24.9 times its slowest act stage,
  and 2000 ms is 147 times its slowest stop of a unit that holds nothing.

What is measured,
what is read from source,
and what is inferred.

- **Measured this round** (one OVHcloud guest,
  kernel `7.0.0-28-generic`,
  OpenZFS 2.4.1 from the distribution,
  systemd 259,
  as `root` and as the unprivileged account `ubuntu`):
  the four holder shapes and the destroy at the instant a holder leaves;
  the waits against interleaved controls;
  a locked unit's stop and a restarted unit;
  another account's task and an unprivileged pin's `EPERM`;
  a session leader's death with a pty,
  for a child that detaches and one that does not;
  a mount namespace no task is in.
  Each reading names its four facts in "The readings of this round".
- **Read from source** on `main`:
  the force count and the four acts it decides,
  the kind table,
  the preflight's variants,
  the constants,
  the service-manager check,
  and the taskless walk.
  Each claim names the file and what it does.
- **Inferred,
  and said so where it stands**:
  what a force count would do once the
  arm is switched on;
  what a systemd 262 user manager does (this round ran 259);
  what OpenZFS 2.4.4 does (this round ran 2.4.1 where the recipe's build is not the one measured).
- **Not measured,
  and why**:
  see "What was not measured,
  and why".
  Nothing here is called a reading of the owner's stack unless kernel,
  OpenZFS,
  systemd and account all match it,
  and this round's guest matches none of the first three.

What a person loses,
under the recommendations,
one line each:

- **another account's task**:
  that person's unsaved work in the task that held the clone,
  ended on the strength of a flag somebody else typed;
- **a task of a system unit**:
  nothing,
  because it refuses;
- **a locked namespace's tasks**:
  everything those tasks had not saved,
  and with them whatever the sandbox or container was doing;
- **a unit of the person's own manager**:
  the service's own work as its own stop ends it,
  or,
  where the stop fails,
  everything its tasks had not saved;
- **an ancestor of the command,
  or its session leader**:
  everything below that process,
  which for a terminal emulator is every tab's shell and every job in them,
  and for a tmux server every session it holds;
- **the clone itself**:
  its uncommitted and untracked files,
  always,
  whichever holder was ended and even where no holder was involved;
- **outside the clone**:
  whatever a holder was writing when it was killed can be left
  half written in the main repository's `.git`,
  in a package store or in a database,
  because the act does not look outside the clone.

What is built today,
in one paragraph:
the removal reads a force count of zero,
one or two,
and the four acts that count decides are a person's lock,
a destruction plan that cannot be read,
a clone the pool does not prove,
and Git's own cleanliness refusal.
Nothing ends a process under any count:
the forced arm in
`src/native/worktree_clone_forced_removal/` is built and no command reaches it,
its kind table gives two kinds an ending no round has shown,
and its waits have no value.
"What is built today,
and what a release would change" gives the list.

What is not true of the numbers:
step 22's own second round measured 256-clone and 512-clone sweeps that
contradict the four-period window above about 205 clones,
and contradict the 5000 ms mount and unmount deadlines it proposed.
Decision 9 keeps 4000 ms anyway,
and says so.

## The readings of this round, and the stack each is from

One guest,
created and destroyed by this round through `mvm --backend ovh`,
from the catalogue's `Ubuntu 26.04`,
one machine at a time,
with a listing before the create
and after it.
Its facts,
as it answered them (`keep/arm-stack.log`):

```text
# keep/arm-stack.log
STACK osrelease=7.0.0-28-generic
STACK os=Ubuntu 26.04 LTS
STACK systemd=systemd 259 (259.9-0ubuntu0.1)
STACK zfs-module=2.4.4-1
STACK zfs-userspace=zfs-2.4.4-1
STACK out-ns=mnt:[4026531832]
STACK out-user-ns=user:[4026531837]
STACK listns=-1 errno=EINVAL
STACK ids-a-task-holds=9
```

The four facts every reading below carries are kernel,
OpenZFS,
systemd and account.
The owner's machine is kernel 7.3,
OpenZFS 2.4.4,
systemd 262 and uid 1000.
This guest matches one of the four,
the OpenZFS release,
and differs in the other three,
so **no reading below is a reading of the owner's stack**,
and each passage that needs one says what stays open.
The OpenZFS it runs is the recipe's own build at 2.4.4,
which the recipe finished in a step of 953 seconds before any arm ran.

Readings,
each with what answered it:

- **The stack itself**:
  kernel `7.0.0-28-generic`,
  OpenZFS 2.4.4 (module and userspace,
  the recipe's build at `/opt/openzfs-2.4.4`),
  systemd 259,
  accounts `root`,
  `ubuntu` and `holder2`.
  `keep/arm-stack.log`.
- **This machine**,
  for the control-group readings only:
  kernel `7.3.5-2-cachyos`,
  systemd 262,
  uid 1000.
  `keep/arm-cgroups.log`.
- **`listns` on this kernel**:
  the call the built taskless walk makes answers `EINVAL`
  for the request this build constructs,
  not `ENOSYS`.
  What that leaves open:
  whether the kernel lacks the call or refuses this request,
  which the built walk's own guest arm on `7.3.5-1-cachyos` settled for that kernel.
  This round did not settle it for 7.0 and does not claim to.
- The rest are quoted where they are used,
  in "Holder classes against no force,
  one force and two",
  "The values",
  and the two questions about the act.

## Holder classes against no force, one force and two forces

One row per class.
Each row's three cells say what the removal does,
what is ended,
and what a person loses.
A cell that rests on the built code names the file;
a cell that rests on a reading of this round names its log;
a cell that is a design proposal says so.

### The caller's own session or one of its ancestors

- **No force**:
  refused,
  `caller-protected-holds`,
  before any signal
  (`holder_preflight.rs`,
  `ForcedRefusal::ProtectedHolds`).
  Nothing is ended.
  The person loses nothing.
- **One force**:
  the same refusal.
  The session decision of 2026-11-28 put the limits of
  2026-11-23 behind one force,
  and that record's never-end set is the caller's session and
  every ancestor (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
  "Which processes a forced removal never ends").
- **Two forces**:
  the same refusal from the preflight,
  and the route of decision 5 instead:
  the act detaches,
  and then the recorded session leader and ancestors are ended by a process
  that is in neither.
  What is ended:
  the session leader and everything below it.
  What a person loses:
  every job of that terminal,
  and,
  where an ancestor holds the clone,
  everything beneath that ancestor.

### A task of the same account in a session of its own

- **No force**:
  the clone stays busy,
  so the removal refuses.
  Nothing is ended.
- **One force**:
  ended,
  `SIGTERM`,
  the wait,
  `SIGKILL` through a pin
  (`holder_endings.rs`,
  `HolderKind::SessionTask`,
  `Ending::EndBlockersThenUnmount`,
  shown by this round's and the earlier round's arms).
- **Two forces**:
  the same.
  What a person loses:
  that task's unsaved work.

### A task of the same account in a mount namespace of its own

- **No force**:
  the copy stays,
  the destroy answers `dataset is busy`,
  the removal refuses.
- **One force**:
  the built order,
  unmount inside the namespace where it can be,
  end the blockers where it cannot (`arm-unmount-in-namespace.log` of the earlier round).
- **Two forces**:
  the same.
  What a person loses:
  the view of the clone the job had,
  plus,
  where way 7 is used,
  what the mount under the copy stops receiving.

### A task in a locked namespace, not the person's own unit

- **No force**:
  `holder-locked` (`holder_preflight.rs`,
  `ForcedRefusal::Locked`).
- **One force**:
  the same refusal.
- **Two forces**:
  decision 3:
  its tasks are ended,
  and the copy goes with the namespace.
  Measured this round:
  the unmount inside answers `EINVAL` to a plain unmount,
  `MNT_FORCE` and `MNT_DETACH` (`keep/arm-userns.log`),
  the destroy refuses `dataset is busy` while the copy stands,
  and it answers `gone` once the task is ended (`keep/arm-holders.log`).
  What a person loses:
  everything that sandbox or container had not saved.

### A task of another account

- **No force**:
  `dataset is busy`,
  or the guard's refusal,
  and nothing is ended.
- **One force**:
  the accepted channel record of 2026-11-22 gives another account's task,
  and a unit's task,
  to a single `--force` through the root handler
  ("By what channel a holder of another account is ended").
  Nothing in the built code does it yet:
  `holder_endings.rs` carries the ending with
  "no round shows this kind's end" beside it.
- **Two forces**:
  decision 1,
  the same mechanism.
  Measured this round:
  an unprivileged caller's pin to that task answers `EPERM`,
  and the handler's privilege ends it (`keep/arm-other-account.log`).
  What a person loses:
  the other account's unsaved work.

### A unit of the person's own user manager

- **No force**:
  refused,
  naming the unit.
- **One force**:
  the stop through the manager is the built route
  (`a_service_unit_stopped_through_the_accounts_manager.rs`),
  and `T_stop` has no value,
  so the act does not run at all today (`forced_arm_numbers.rs`).
- **Two forces**:
  decision 4,
  the stop,
  then the tasks' end where the stop failed or the unit
  returned.
  Measured this round:
  the stop ends the unit and its main task,
  and a unit whose task is killed outside its manager comes back within two seconds and holds the
  clone again (`keep/arm-unit-stops.log`).
  What this round could not measure:
  a locked unit at all.
  A `PrivateMounts=yes` unit of the unprivileged account had no mount namespace of its own on
  systemd 259 (`keep/arm-locked.log`),
  and the image forbids an unprivileged user namespace,
  so the shape this route exists for was not
  available to be stopped.
  The 2026-11-27 reading that shows the stop ending a unit's control group and namespace is root's
  manager on systemd 255 (`SPEC.md`,
  "The forced arm,
  built and reachable from no command").
  What a person loses:
  the service's own work,
  as its own stop ends it.

### A scope, and the shell that launched it

- **No force**:
  refused where the scope's tasks hold the clone.
- **One force**:
  ended as a session task,
  since the scope's tasks are tasks of their own session.
- **Two forces**:
  the same,
  and one correction to the earlier revision:
  a scope's tasks are the scope's control group,
  and **the shell that launched the scope is not
  in it**.
  The earlier revision said the scope route spares that shell "because the scope is the
  session and its jobs are in the set",
  which its own log refutes:
  the scope's jobs share the caller's session but not its control group
  (`arm-scopes.log`,
  `C2 scope-jobs-before=2`,
  which counts the scope's shell and its child only).
  So under decision 6 the shell is outside the set and is ended like any other session task if
  it holds the clone.
  What a person loses:
  that shell and its jobs.

### A namespace no task is in

- **No force**:
  refused where the kernel has no `listns`.
- **One force**:
  the same.
  The built walk takes the copy out where `listns` answers
  (`taskless_namespace_walk.rs`,
  evidence on `7.3.5-1-cachyos`).
- **Two forces**:
  the same built route;
  a kernel without `listns` refuses `holder-unendable`,
  because nothing names the namespace.
  This round could not read `listns` on its own guest's kernel,
  so what is left open is that
  kernel and not the route.
  What a person loses:
  nothing,
  since no task is there to lose anything.

### A plain command of an agent session, whose control group is what it is on this machine

- **No force,
  one force,
  two forces**:
  as whatever class above its task falls in.
  What is particular to it is decision 6:
  its control-group line names no unit,
  so under decision 6 a forced removal started from it refuses to read an identity,
  and the person runs it under `systemd-run --user --scope` or from a unit.
  Nothing is ended by the refusal.
  What a person loses:
  nothing,
  one command.

### The two rules the gate read found contradicting each other

The earlier revision said at once that "the manager is never ended,
and no other unit is ever
ended" and that a unit's task and a unit are ended.
Both were wrong,
and this revision replaces them with one rule:

- the removal ends the tasks of the unit it was started in only as far as decisions 1 to 4 say,
  which is never the manager itself,
  never another unit's tasks without that decision's
  mechanism,
  and never a system unit's task at all;
- the manager,
  pid 1 and anything above the requester's own unit are never ended by any count.

## What is built today, and what a release would change

### The force count the removal reads

Merge `7c4962988d` built it,
and it is read in four places
(`src/native/worktree_clone_removal_proof_and_plan/`):

- `removal_force.rs` is one value,
  Git's own count,
  with `is_given()` and `is_twice()`.
  `--no-force` sets the count back,
  so the value is not a count of words but of forces.
- `the_lock_a_removal_refuses.rs` deletes the person's lock when the count is two,
  and refuses `TheLocksWordsAreNotThisAdds` below that.
- `removal_entry_facts.rs` destroys a clone the pool does not prove when the count is two,
  and otherwise answers `TheCloneCannotBeProved`.
- `removal_resume.rs` goes past a destruction plan it cannot read when the count is at least one,
  through `removal_forced_start.rs`.
- `removal_entry.rs` waives the bookmark rule when the count is at least one,
  and `the_slots_cleanliness.rs` passes Git's own cleanliness refusal with one.

Two states refuse whatever the count:
a plan that cannot be read whose clone the pool no longer answers
(`RemovalStop::ThePlanIsUnreadableAndTheCloneIsGone`),
and a lock that could not be deleted (`RemovalStop::TheLockCouldNotGo`).
Both are in `removal_force_refusal_sentences.rs`,
and the first is the handover's own veto entry.

**No force count ends a process.**
The removal's own path has no caller into `src/native/worktree_clone_forced_removal/`:
`ending_of` has a definition,
a re-export,
its tests and its name in the source-guard list,
and no production caller (`git grep`).

### What the forced arm's own folders say, kind by kind

- `PathHolder` and `SessionTask`:
  `EndBlockersThenUnmount`,
  each with the round that showed it.
- `OtherAccountTask` and `UnitTask`:
  `EndBlockersThenUnmount` in the table's data,
  with `shown_by` reading "no round shows this kind's end;
  the schedule that ran is one
  account's own child" and "no arm sent a pinned signal to a unit's task".
  The module's own doc and `SPEC.md` say a kind the evidence does not show is refused,
  so **the prose and the data disagree**,
  and nothing in the code classifies a holder into these
  kinds.
  The earlier revision repeated the prose and called these "three rows the table refuses
  today",
  which is wrong:
  no row carries a refusal for them.
- `TasklessNamespace`:
  `EnterAndTakeTheCopyOut`,
  built by commit `539313dc0c` and measured on
  `7.3.5-1-cachyos`;
  it refuses `holder-unendable` only where `listns` answers `ENOSYS`.
  The earlier revision said it "stays refused",
  which is wrong.
- `LockedHolder`:
  `StopTheAccountsOwnUnitThenGoOn`,
  and the refusal for every other locked
  holder is in `holder_preflight.rs` (`ForcedRefusal::Locked`),
  not in the table.
  So an owner who chose "end a locked namespace's tasks" needs a new ending,
  because `ends_blockers` is false for a locked holder today and the signal stage ends only the
  blockers the matcher found.

### The delta a release would have to make

None of this is in the design's own earlier list,
and each item says where it goes:

1. A force count that reaches the forced arm:
   today the count stops at the removal's own acts.
2. The four constants set,
   and `TimeoutWindow::of_the_constants` unblocked
   (`forced_arm_numbers.rs`;
   all three periods are `None`).
3. A production caller of `ending_of`,
   and one decision for the two rows whose data carries an
   ending no round shows:
   either an arm that shows them or a `Refuse` in the table.
4. `holder_preflight.rs`:
   `ProtectedHolds` and `Locked` become routes under two forces
   (decisions 5 and 3),
   and `Unendable` stays a refusal.
5. The service-manager check replaced by decision 6's control-group read,
   in `started_by_the_service_manager.rs` and its sentences.
6. Decision 5's detach:
   a child with its own session and working directory,
   `SIGHUP` ignored,
   the request written by that child,
   and the request recording the caller's session leader and
   ancestors by pid and start time so the detached child can end them without being beneath them.
7. Decision 4's fallback,
   which needs `T_stop` and a second read of the unit's state.
8. `T_release`,
   the bounded re-ask of the destroy after the last holder leaves
   (`T_release` does not exist today;
   the design proposes it in "The values").
9. The taskless route is built;
   what a release adds is nothing.
10. `SPEC.md`'s forced-arm section,
    the source-guard name list,
    and the four records listed in
    "What this amends".

What the release does **not** need,
though the earlier revision implied it:
the two kinds whose rows already carry an ending,
and the taskless route.
What it does need and the earlier revision called "three rows":
new code for decision 3's ending,
and a caller for the table.
## The act, decision by decision

### A command that must end its own session or one of its own ancestors

This is the record's first open question and decision 5.

The refusal it lifts is the never-end set's:
a holder that is the caller's own session leader,
or one of the command's ancestors,
refuses the removal under both counts today
(`holder_preflight.rs`,
`ForcedRefusal::ProtectedHolds`).
Two forces lift it,
and then the act must end a process above or beside itself and still
reach its last step.

The candidates,
each with how it works,
what it stores for good,
and what it gives up.

#### The removal detaches itself before it ends anything

How it works.
The wrapper writes the plan and the request,
then forks.
The child calls `setsid`,
changes to a working directory outside the clone,
sets `SIGHUP` to be ignored,
and leaves the person's session for good.
The **child** then writes the caller property with its own process identity,
rereads the plan,
and makes the act:
the preflight,
the structural stage,
the signal stage,
the unmount and the destroy.
The parent exits at once,
or waits for as long as its own session gives it,
whichever the terminal allows.
The child's own ancestry is no longer the caller's,
so ending the caller's session leader and
the recorded ancestors reaches nothing that is doing the removal.

What it stores for good:
nothing but the plan,
the receipt and the request it already wrote.
It adds no mount,
no unit and no file outside the removal's own records.

What makes it lawful under the rule the project accepted on 2026-11-22:
the rule is that "the handler ends processes only while the very command that asked is still
running,
checked by its process identity before every signal and every unmount"
(`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
"How long a forced request's authority lasts").
The child is the command that asked,
because it is the one that writes the request,
and it is running until the act's last step.
The parent's death is not the requester's death,
and no request is left standing after the act.

What it gives up:
the person's terminal already dies with the session it is ending,
so this candidate loses no
output the others keep;
it loses the exit status,
which has no reader once the shell is gone;
and it leaves a process whose parent has exited,
so a failure after that point is visible only in
the receipt and the plan,
and a person who wants to stop it uses the cancel command.

What it does not need:
a user manager,
lingering,
a unit,
or root.

#### The removal re-executes itself as a unit of the account's own user manager

How it works.
Before it ends anything,
the removal starts a transient service unit of the account's own user
manager (`systemd-run --user --unit=<name> --collect`) whose task runs the act.
That task has the manager as its parent,
a session and a control group of its own,
and no process of the caller's session above it.
**The unit's task writes the request**,
under the nonce the wrapper already chose,
and the caller property names that task's identity.
The wrapper waits for the receipt.

What it stores for good:
a transient unit,
gone when it ends,
and the plan and receipt.
What it gives up:
the account must have a running user manager **and lingering**;
the removal's output moves to the manager's journal;
a unit the person did not start holds the act's authority,
and the receipt is the only thing that
names the person as its cause.

Whose identity the root handler checks,
and why that keeps the rule:
the unit's own task,
because it is the writer of record.
The wrapper's identity is checked by nothing,
and that is the deliberate change:
the wrapper is the thing whose session dies.
This is **not** the root-mode candidate below,
where the request outlives its writer:
here the writer lives exactly as long as the act,
so "only while the very command that asked is
still running" holds,
with "the command" meaning the unit's task.
What would have broken the rule is a request written by the wrapper and executed by the unit:
then the writer is dead while the act runs,
`blocker_end.rs`'s `caller-gone` stops the first
signal,
and the act ends half done.
So the request must be written by the unit,
and the wrapper's
own copy,
if it wrote one,
is discarded when the unit's request does not carry its identity.

What lingering has to do with the route's trigger:
with lingering off,
the account's manager stops after `UserStopDelaySec`
(ten seconds by default,
`man logind.conf`) once the last login session ends,
which is exactly what this route does by ending that session.
A manager that is there when the route starts and gone ten seconds later stops the transient unit
mid-act,
so the trigger is "a running manager **and** lingering on",
and with either missing the route falls to the detach or the plan.

#### The removal writes its plan, ends what it must, and leaves the rest to the next run

How it works:
the plan is written first,
the endings are made,
the command stops,
and the next command adopts the plan under a fresh nonce and finishes.
This is exactly the owner's option text:
"The command can then end the shell it runs in before it
has finished,
leaving a destruction plan for the next run."
What it stores for good:
the plan,
with every name it settled.
What it gives up:
one command;
the first run's own account of what it ended;
and a slot that stays busy until someone runs the second command.

#### The removal never ends them and unmounts under them

How it works:
the copy is taken out of the caller's own namespace,
as question 1's second candidate does for
any holder's namespace.
Measured:
while the caller stands in the clone the unmount answers `EBUSY` and the destroy stays
refused,
and once the caller's working directory is elsewhere the same unmount is accepted
(`keep/arm-under-the-caller.log`).
What it gives up:
the case the refusal was about,
which is the caller standing in the clone.

#### The removal hands the act to the root mode and lets the request outlive the caller

How it works:
the installed root handler does the act after the wrapper is gone.
What it stores for good:
a standing request that can act again after a crash.
What it gives up:
the duration rule itself,
which is the one thing that makes a stale request
harmless today.
This is the candidate the design does not choose.

#### What the four fire together on

The detach and the unit both end the same processes and both finish in one command;
they differ in what they need and in what a person can read afterwards.
The plan needs a second command.
The unmount passes only a caller whose reference is not a path
into the clone.
The root mode reverses the rule.

### Under the service manager

This is decision 6.

What the refusal is today:
`--force` is refused when the command's parent is the service manager,
or when the
session leader's parent is,
with the reason that the manager "cannot be told apart from the
process that asked for it"
(`started_by_the_service_manager.rs`,
`the_refusal_text`).
The two reads are the parent's `/proc/<pid>/cgroup` and `comm`,
and the session leader's parent's.
What that protects is the never-end set and nothing else:
with the manager among the command's ancestors,
the set's upper half is the manager and pid 1,
its lower half is the command alone,
and every other unit stands between them,
neither spared nor the caller.

What two forces do there under decision 6:
the requester is named by its own unit,
read from the last component of its unified control-group line that ends `.service` or `.scope`.
That name is its identity,
the never-end set is the tasks in that unit's control group,
and a line whose last component names no unit refuses.

Read on this machine (`keep/arm-cgroups.log`):
a command this project's agents run carries
`0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash`,
which names no unit,
and one run under `systemd-run --user --scope` carries a line that ends
`.scope` and holds only that command and its children.
So a forced removal from this harness's own Bash tool refuses under decision 6,
and the person runs it under a scope.

What the refusal's reason still protects after the change:
the removal never ends the manager,
pid 1,
or anything above its own unit,
because none of them is in the set;
it never ends another unit's tasks except as decisions 1 to 4 name a mechanism for it;
and a removal started in a scope spares **the scope's own tasks**,
not the shell that launched it,
which is the correction to the earlier revision given in "A scope,
and the shell that launched it".

What it gives up:
the sentence that told a person to run the removal as a child of the process that wants it.
A request from a unit can act,
and its witnesses are the manager's journal and the receipt.

## Every refusal that survives two forces

Each is a state the act cannot finish,
or a state in which finishing would destroy something the
answer did not name.
Each is named with its reason,
and decision 7 keeps them all except the two
that decisions 1 to 3 replace.

1. **A blind read** (`ForcedRefusal::Blind`):
   a read the preflight could not make,
   or a scan that
   did not read every namespace.
   Force has nothing to pass:
   the act does not know what stands.
2. **A parent mount that is not the checkout's** (`ParentIsNotTheCheckout`):
   the act cannot tell what an unmount there would reach,
   which is another dataset's data.
3. **A mount under the copy** (`Structure`):
   destroying the clone would destroy whatever is
   mounted inside it.
4. **No destination for way 7** (`NoDestination`):
   the parent is read-only,
   or the dataset has no
   room for one directory,
   so no route names the copy.
5. **An unendable holder** (`Unendable`):
   the preflight proved the task cannot be ended.
6. **A task that outlives `SIGKILL`** (the signal stage's `Survived`):
   the act says so and stops.
7. **A taskless namespace on a kernel without `listns`** (`holder-unendable`):
   nothing names the namespace.
8. **A control-group line that names no unit** (decision 6),
   which is this project's own harness on this machine.
9. **A destruction plan that cannot be read whose clone the pool no longer answers**
   (`ThePlanIsUnreadableAndTheCloneIsGone`):
   deleting the plan would delete the only record of what
   an earlier run destroyed.
   This is the handover's own veto entry,
   and the settling command
   refuses that plan too,
   so a person has no way out of the state in this build.
10. **A lock that could not be deleted** (`TheLockCouldNotGo`):
    the lock still stands,
    so the removal stops with nothing unmounted and nothing destroyed.
11. **A task of a system unit** (decision 2's exception):
    a person's flag does not stop a
    machine-wide service,
    and the act cannot see what depends on it.
12. **Four refusals the force round's own record keeps** (`doc/handover/questions-for-the-owner.md`,
    "Veto open:
    what the round that made the removal read force chose"):
    the transferred-history
    refusal,
    the origin-side hit,
    the deferred-origin checks,
    and the holder scans,
    which still run
    and still refuse what they cannot prove.

Two of the design's earlier refusals are withdrawn,
because the answer to the read's question
("why is a locked namespace passed by ending its tasks while one that cannot be entered is
refused?")
is that the same act works on both:

- an **unenterable namespace** whose tasks the act can signal is now passed the way a locked
  namespace is,
  by ending those tasks:
  the copy goes with the namespace,
  and entering it was never
  needed for that;
- a **locked namespace that is not the person's own unit** is passed by decision 3,
  which amends the
  2026-11-22 record named in "What this amends".

## What this amends

Four accepted records,
each quoted from its own file.

1. **The lock design's sentence** that a removal must not release a lock it cannot attribute to
   this add.
   Merge `7c4962988d` already carries the change two forces make,
   in the removal's own
   report ("two forces were given,
   so this removal deleted the lock at ... that this add did not
   write"),
   and the record of 2026-11-28 amends the sentence in its own list.
2. **The answer of 2026-11-26** that named three refusals "and no more",
   which two forces go past
   in the same merge.
3. **The forced design's set of processes a forced removal never ends**,
   which decision 5 replaces
   with the detach's recorded set:
   `doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
   "Which processes a forced removal never ends",
   chose "a forced removal never ends the caller's session or any ancestor of the command up to the
   first process,
   and refuses,
   naming it,
   when one of them holds the clone".
4. **The answer of 2026-11-22 on how much a forced removal ends in another mount namespace**:
   > It never falls back to ending every task of the namespace.

   (`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
   "How much a forced removal ends in another mount namespace".)
   Decision 3 does exactly what that sentence refuses,
   for a locked namespace and for an
   unenterable one whose tasks can be signalled,
   and the reason is measured:
   no unmount reaches a
   locked copy,
   so the alternative is a refusal the owner's words did not ask for.

One accepted rule is **not** amended,
and the design says why:

> The project chose "A:
> only while the command lives (Recommended)":
> the handler ends processes
> only while the very command that asked is still running,
> checked by its process identity before
> every signal and every unmount.

(`doc/decision/cli-git-zfs-forced-removal-authority-and-reach.md`,
"How long a forced request's authority lasts".)
The detach and the unit route both keep it,
by making the requester of record the process that runs
the act;
the root-mode candidate would break it,
which is why it ranks last.

## What was not measured, and why

- **The owner's stack**:
  kernel 7.3,
  OpenZFS 2.4.4,
  systemd 262,
  uid 1000.
  The guest this round could make ran kernel `7.0.0-28-generic`,
  OpenZFS 2.4.4 (the recipe's build) and systemd 259,
  as root,
  `ubuntu` and `holder2`.
  So one of the four matches,
  and each reading carries all four.
- **systemd 262**:
  no image on that backend carries it,
  so every manager reading here is 259's.
  That is what makes the locked-unit finding sharp:
  on 259 an unprivileged account's `PrivateMounts=yes` unit had no mount namespace of its own,
  so this round cannot say whether the owner's manager gives one.
  What that leaves open is stated in question 4.
- **An unprivileged user namespace**:
  the image restricts them (`kernel.apparmor_restrict_unprivileged_userns=1`),
  so `ubuntu` could not make a locked namespace at all,
  and the locked shape this round measured was made as root.
- **A holder that flushes a large file to a real disk**:
  this round's 16 MiB writer wrote to the guest's own disk and its ratio to the control spread
  threefold.
- **The wrapper's own binary and the wrapper's own removal**:
  every reading here is a kernel's or a manager's answer to the same calls the built arm makes,
  not the built binary's answer,
  and no arm needed a removal to finish.
- **`listns`** on kernel 7.0:
  the call answered `EINVAL` to the request this build constructs,
  so this round did not settle whether 7.0 has the call,
  and it did not run the taskless arm.
  The built route's own evidence is on `7.3.5-1-cachyos`,
  the owner's kernel family.
- **A removal of a real clone that finishes**:
  on `main` and in releases `0.6.1-alpha.53` and
  `0.6.1-alpha.54` it stops before its first destroy for two causes (issue 6669),
  so no arm here
  makes the wrapper's own removal finish,
  and none needed to.
- **The wrapper's own binary**:
  every reading here is a kernel's or a manager's answer to the same
  calls the built arm makes,
  not the built binary's answer.

## The values

Four numbers ship as `Option`s that answer nothing today
(`forced_arm_numbers.rs`):
`TERMINATION_WAIT` (`T_wait`),
`HEARTBEAT_PERIOD` (`T_h`),
`GIVE_UP_HEARTBEATS` (four,
a relation and not a period),
and `STOP_WAIT` (`T_stop`).
`TimeoutWindow::of_the_constants` refuses while any period is unset,
so the act does not run.
A fifth,
`T_release`,
does not exist and this design proposes it.

What each is a bound on:

- `T_wait` bounds how long a polite exit is waited for.
  It is a ceiling:
  a holder that never
  leaves costs the whole wait and is killed at its end.
- `T_h` bounds how long the wrapper may read a counter that has not moved and still call the
  handler alive.
  The built mechanism advances the counter with a timer thread
  (`the_progress_file_writes.rs`),
  so a stage's length does not bound the period by itself;
  what does is starving that thread (load and swap) and the wrapper's read cadence,
  which nothing has measured.
- `T_stop` bounds a manager's answer,
  which is another program's call and not the act's,
  so it must exceed the slowest stop measured and not the act's own stages.
- `T_release` bounds how long the act keeps asking the destroy after the last holder has left,
  because a mount's final release can be deferred to a delayed work item
  (`mntput_no_expire`,
  `fs/namespace.c` at `v7.3`).
  Without it,
  the act refuses a removal the kernel has already freed a moment later.

### What step 22 measured, and what contradicts the proposals

Step 22's figures,
from `doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`,
"The values this measurement would support":

- a cooperating holder left in "0 to 2 milliseconds",
  and 2000 ms is offered as "about a thousand
  times the measured exit";
- a 64-clone sweep is 853 ms,
  and 1000 ms with a four-period window of 4000 is offered as
  "4.7 times the largest sweep the first round measured";
- the same record's second round measured 256-clone sweeps at 3,877 to 5,939 ms and 512-clone
  sweeps at 8,167 to 12,374 ms warm,
  and says the window "is shorter than one sweep at 256 clones
  in two rounds of three and two to three times shorter than one at 512",
  so that a window of four
  periods of 1000 ms "contradicts the proposal at any parent above about 205 clones";
- the same second round contradicts the 5000 ms mount and unmount deadlines it proposed,
  for the same reason.

So of the three periods,
one (`T_wait`) rests on an exit measured in single milliseconds,
one (`T_h`) rests on a sweep this project's own parent now exceeds the window for at about 205 clones,
and one (`T_stop`) has no measurement at all in step 22.
The earlier revision of this document wrote that the values rest on "the largest figure each
bounds" and called the window "260 times the slowest stage",
which is not a reason to ship it:
it is a ratio written after the value.

### This round's readings, arm by arm

Every arm ran on the guest described in "The readings of this round":
kernel `7.0.0-28-generic`,
OpenZFS 2.4.4,
systemd 259,
as `root` and as `ubuntu`.
The arm,
how it can fail,
and what it answered:

**`holders`**,
the four holder shapes.
Each shape has its own before-state,
which is what makes the arm able to fail:
a plain unmount refused (a path holder) or the pool's own `dataset is busy`
(a namespace holder) while the holder stands.
The destroy is then asked at the instant the exit is reported,
with no settle sleep.

```text
# keep/arm-holders.log
HOLDERS kind=namespace-private destroy-before=cannot destroy 'ztf/work/slot': dataset is busy
HOLDERS kind=namespace-private end={"term_ms": 16.574, "left_after_term": true, "kill_ms": null, "left_after_kill": null}
HOLDERS kind=namespace-private destroy-at-the-exit={"immediately": "gone", "ms_to_gone": 28.442, "tries": 1}
HOLDERS kind=cwd our-unmount-before=errno=EBUSY
HOLDERS kind=cwd end={"term_ms": 0.608, "left_after_term": true, "kill_ms": null, "left_after_kill": null}
HOLDERS kind=cwd destroy-at-the-exit={"immediately": "gone", "ms_to_gone": 33.931, "tries": 1}
```

All four shapes answered the same way:
the destroy was `gone` on its first call,
so no deferred
release of the mount appeared in these readings and the retry never fired.
The 24 to 34 milliseconds is the destroy call itself,
not a wait.

**`waits`**,
three arms and two controls,
every arm in every round so each round's own ratio can
be recomputed.
The holder that ignores the ask ran against a one-second bound.

```text
# keep/arm-waits.log
WAITS t-wait={"control-bare-sleep": [{"ms": 1.117, "term_ms": 0.941, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 0.966, "term_ms": 0.811, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 0.636, "term_ms": 0.556, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 0.799, "term_ms": 0.713, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 0.746, "term_ms": 0.639, "left_after_term": true, "kill_ms": null, "left_after_kill": null}], "holder-that-syncs-then-leaves": [{"ms": 35.209, "term_ms": 35.093, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 9.098, "term_ms": 9.002, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 30.394, "term_ms": 30.301, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 6.384, "term_ms": 6.302, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 11.821, "term_ms": 11.738, "left_after_term": true, "kill_ms": null, "left_after_kill": null}], "holder-that-writes-16m-then-leaves": [{"ms": 202.905, "term_ms": 202.769, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 52.989, "term_ms": 52.865, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 52.893, "term_ms": 52.759, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 80.235, "term_ms": 80.091, "left_after_term": true, "kill_ms": null, "left_after_kill": null}, {"ms": 67.832, "term_ms": 67.694, "left_after_term": true, "kill_ms": null, "left_after_kill": null}]}
WAITS t-wait-ignorer={"holder-that-ignores-the-ask": [{"ms": 1001.404, "term_ms": 1001.01, "left_after_term": false, "kill_ms": 0.271, "left_after_kill": true}, {"ms": 1001.408, "term_ms": 1001.073, "left_after_term": false, "kill_ms": 0.252, "left_after_kill": true}]}
WAITS act-stage={"enter-a-holders-namespace-unmount-reap-the-holder-and-unmount-at-home": [{"ms": 40.232, "inside": {"child-status": 0}, "term_ms": 0.541, "our-unmount": "accepted", "destroy": "gone"}, {"ms": 37.251, "inside": {"child-status": 0}, "term_ms": 0.579, "our-unmount": "accepted", "destroy": "gone"}, {"ms": 36.674, "inside": {"child-status": 0}, "term_ms": 0.564, "our-unmount": "accepted", "destroy": "gone"}], "enter-a-holders-namespace-and-unmount-only": [{"ms": 3.653, "child-status": 0}, {"ms": 2.948, "child-status": 0}, {"ms": 3.353, "child-status": 0}], "control-the-same-unmount-at-home": [{"ms": 79.1, "unmount": "accepted"}, {"ms": 82.769, "unmount": "accepted"}, {"ms": 79.57, "unmount": "accepted"}]}
WAITS progress-file={"publish-with-both-syncs": [{"ms": 0.224, "synced": true}, {"ms": 0.101, "synced": true}, {"ms": 0.057, "synced": true}, {"ms": 0.056, "synced": true}, {"ms": 0.082, "synced": true}], "control-publish-without-syncs": [{"ms": 0.112, "synced": false}, {"ms": 0.058, "synced": false}, {"ms": 0.047, "synced": false}, {"ms": 0.047, "synced": false}, {"ms": 0.047, "synced": false}]}
```

What these say,
with each round's own ratio:

- the bare-sleep control's median is 0.799 ms;
  a holder that runs `sync` and leaves has a median of
  11.821 ms,
  and its per-round ratios to the control are 8.0,
  9.4,
  15.9,
  31.5 and 47.8;
  a holder that writes 16 MiB and syncs it before leaving has a median of 67.832 ms,
  with per-round ratios of 54.9,
  83.2,
  90.9,
  100.4 and 181.7.
  So the ratio's spread is about sixfold for the syncing holder and about threefold for the writing
  one,
  and the slowest cooperating exit measured is 202.905 ms;
- a holder that ignores the ask did not leave inside the one-second bound in either round,
  and the `SIGKILL` through the same pin was observed 0.271 and 0.252 ms later;
- the act's stage,
  named for every call it makes,
  has a median of 37.251 ms and a slowest of
  40.232 ms;
  the enter-and-unmount alone has a median of 3.353 ms,
  so the kill,
  the reap and the
  act's own unmount are about nine tenths of the stage and the namespace call is about a tenth;
- the control written for the stage (`the same unmount at home`) **also makes the clone**,
  so its 79 to 83 ms is the fixture's creation and not an unmount's time,
  and it is not used as a ratio.
  That is the kind of arm the gate read's L2 finding named,
  and this round's own control has it;
- the progress file's publish has a median of 0.082 ms with both syncs against 0.047 ms without
  them,
  per-round ratios 1.19 to 2.0,
  so the file's own write is three orders of magnitude under
  any period considered here.

**`unit-stops`**,
run as the unprivileged account for the timings and as root for the namespace
readings.

```text
# keep/arm-unit-stops.log
INNER UNITSTOPS t-stop={"ztf-locked": [{"ms": 10.22, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152407, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152407/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152407/ns/mnt'", "our-user-ns": "user:[4026531837]"}, {"ms": 10.644, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152426, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152426/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152426/ns/mnt'", "our-user-ns": "user:[4026531837]"}, {"ms": 11.197, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152445, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152445/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152445/ns/mnt'", "our-user-ns": "user:[4026531837]"}], "ztf-plain": [{"ms": 14.634, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152417, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152417/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152417/ns/mnt'", "our-user-ns": "user:[4026531837]"}, {"ms": 13.561, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152436, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152436/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152436/ns/mnt'", "our-user-ns": "user:[4026531837]"}, {"ms": 12.943, "gone": true, "said": "", "state": "inactive", "main-after": "0", "main": 152455, "mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152455/ns/mnt'", "user-ns-of-its-mnt": "unreadable:[Errno 2] No such file or directory: '/proc/152455/ns/mnt'", "our-user-ns": "user:[4026531837]"}]}
INNER RESTART after-the-task-was-killed={"state": "active", "new-main": "152479", "came-back": true, "the-new-task-holds-the-clone": "1", "the-new-task-ns": "mnt:[4026531832]"}
```

- the unit named `ztf-locked` reaches `inactive` with its main task gone,
  in 10.22 to 11.197 ms,
  and the plain unit in 12.943 to 14.634 ms:
  per-round ratios 0.70,
  0.78 and 0.87,
  so **the manager's stop shows no cost of the lock at all**,
  and the whole time is the `systemctl` call
  this arm makes;
- a unit with `Restart=always` whose main task is killed outside its manager came back inside two
  seconds with a new main task,
  and that task held the clone again (`the-new-task-holds-the-clone`
  is 1).

**`locked`**,
what the three shapes of a locked holder are on this stack.
This arm is the one that changes a claim of the earlier revision.

```text
# keep/arm-locked.log
LOCKED apparmor-restricts-unprivileged-userns=1 unprivileged-userns-clone=1
LOCKED private-mounts-unit-of-the-account mnt=mnt:[4026531832] mnt-owner=user:[4026531837] our-user-ns=user:[4026531837] ours-is-the-same-ns=True
LOCKED private-mounts-unit-of-the-account destroy-with-its-copy-standing=
LOCKED private-mounts-and-private-users-unit-of-the-account mnt=mnt:[4026531832] mnt-owner=user:[4026531837] our-user-ns=user:[4026531837] ours-is-the-same-ns=True
LOCKED private-mounts-and-private-users-unit-of-the-account destroy-with-its-copy-standing=
```

- the image restricts unprivileged user namespaces
  (`kernel.apparmor_restrict_unprivileged_userns=1`),
  so the unprivileged account cannot make one:
  `unshare --user --map-root-user` answered `write failed /proc/self/uid_map: Operation not permitted`.
  A bubblewrap sandbox or a rootless container of that account is therefore not available on this
  image at all;
- a `PrivateMounts=yes` transient unit of the account's own manager,
  with and without
  `PrivateUsers=yes`,
  **had no mount namespace of its own**:
  its `/proc/<pid>/ns/mnt` is the
  initial namespace's,
  its owner is the initial user namespace,
  and the destroy answered `gone`
  while the unit stood,
  so it held no copy.
  On systemd 259 with an unprivileged account this shape is not a locked holder.

**`userns`**,
the locked shape made as root,
which is the one the owner's stack would have to
produce:

```text
# keep/arm-userns.log
USERNS holder-mnt=mnt:[4026532417] owner=user:[4026532414] our-user-ns=user:[4026531837] same-ns=False
USERNS unmount-plain=refused errno=EINVAL
USERNS unmount-MNT_FORCE=refused errno=EINVAL
USERNS unmount-MNT_DETACH=refused errno=EINVAL
USERNS destroy-with-the-copy-standing=cannot destroy 'ztf/work/slot': dataset is busy
USERNS destroy-after-the-end=
```

So a copy in a namespace owned by another user namespace answers `EINVAL` to all three unmount
calls,
refuses the destroy while it stands,
and the dataset is `gone` once its task is ended.

**`other-account`**,
a task of the second account:

```text
# keep/arm-other-account.log
OTHER holder=pid=148853 uid=1001 ns=mnt:[4026531832] alive=True runuser-pid=148852
OTHER destroy-before=cannot unmount '/srv/ztf/work/slot': pool or dataset is busy
OTHER unprivileged=UNPRIVILEGED signal=refused errno=[Errno 1] Operation not permitted
OTHER holder-alive-after-the-unprivileged-pin=True
OTHER end-by-the-handlers-privilege={"term_ms": 0.587, "left_after_term": true, "kill_ms": null, "left_after_kill": null}
OTHER destroy-at-the-exit={"immediately": "gone", "ms_to_gone": 34.709, "tries": 1}
```

So the unprivileged caller's pin is refused and the task stays;
the same call from root ends it in
0.587 ms and the destroy follows.

**`session-death`**,
a pty,
a session leader holding the clone,
and two children:

```text
# keep/arm-session-death.log
SESSION child=in-the-session report=signals=['SIGHUP'] ticks=199 writes=199 cwd=/srv/ztf/work/slot session=148967
SESSION child=detached report=signals=[] ticks=199 writes=199 cwd=/ session=148969
SESSION destroy-after={"immediately": "gone", "ms_to_gone": 32.243, "tries": 1}
```

The child that stayed in the dying session received `SIGHUP`;
the child that called `setsid`,
changed to `/` and ignored `SIGHUP` received nothing and kept working in a session of its own.
Both wrote every tick,
because this arm keeps the pty's master open in the parent,
so the arm says
nothing about a write to a terminal that has died.

**`cgroups`**,
this machine's own control-group lines (`keep/arm-cgroups.log`):
`0::/user.slice/user-1000.slice/user@1000.service/app.slice/<this run's unit>.service` for a
command this Pi run starts,
`0::/user.slice/user-1000.slice/user@1000.service/app.slice/claude-code-bash` for a Claude Code
Bash command,
and `.../agent.slice/agent-work.slice/run-<token>.scope` under a scope.

### The four proposed values

- `T_wait` 2000 ms:
  step 22's proposal,
  carried over,
  and **9.9 times** the slowest cooperating exit this round
  measured (202.905 ms),
  where step 22's own basis was 0 to 2 ms and its 2000 was "about a thousand
  times" it.
  A holder that ignores the ask costs the whole of it,
  and the `SIGKILL` that follows is observed
  within 0.3 ms of the bound.
  What this does not cover:
  a holder that flushes a large file to a real disk,
  which this round
  measured only on the guest's own disk.
- `T_h` 1000 ms with `GIVE_UP_HEARTBEATS` 4,
  so a window of 4000 ms:
  step 22's proposal,
  carried over,
  and 24.9 to 27.3 times this round's act stage (36.674 to
  40.232 ms),
  with the window 99 to 109 times it.
  What contradicts the window:
  step 22's own 256-clone and 512-clone sweeps,
  above about 205
  clones.
  What that costs is a wrapper that reads a live handler as dead during a large sweep,
  which cancels the act and leaves the plan.
- `T_stop` 2000 ms:
  step 22's proposal,
  carried over,
  and **147 times** this round's slowest stop of a unit that holds
  nothing (13.561 ms),
  where step 22 had no measurement at all.
  What it does not cover:
  a service whose own `ExecStop` takes longer than two seconds,
  which the manager's default stop timeout allows up to ninety,
  so the fallback can cut a graceful stop short.
- `T_release` 1000 ms,
  new:
  the kernel may release a mount's last reference from a delayed work item,
  so the act asks the
  destroy again rather than refusing at the first answer.
  This round's four readings all answered `gone` on the first call,
  so the retry never fired and
  **no reading here sets this bound**;
  1000 ms is a bound chosen from the kernel's own rule,
  and the act says `holder-still-holds`,
  naming what the pool answered,
  when it passes.

Each of the last three is put to the owner as its own question,
with a tighter and a looser arm,
because they bound different things and one may move without the others.

## The questions for the owner

Ten questions.
Each stands alone,
each option says how it works and what a yes leaves undecided,
and each ranking gives the reason for every adjacent pair in what the option delivers.
Nothing here needs an earlier section to be read.

### Question 1: another account's process holds the clone

Somebody else's command is working in the clone,
with its working directory or an open file inside
it.
Your account cannot signal that process:
this round measured that a pin from `ubuntu` to a
`holder2` task answers `EPERM` (`keep/arm-other-account.log`).
The clone stays busy and the removal stops.

**Option A,
recommended:
two forces end it through the root handler.**
How it works:
your command writes the request the installed root handler already reads;
root pins that task,
reads its start time again,
sends `SIGTERM`,
waits `T_wait`,
sends `SIGKILL` to a survivor,
takes the copy out of that task's namespace where it can,
and destroys the clone.
Root checks your own process identity before every signal and every unmount,
so the request dies
with your command.
What you see:
`git worktree remove --force --force <slot>` prints the clone destroyed and one line naming the
task's account and pid;
nothing else on the machine changes.
What you lose:
that person's unsaved work,
ended on the strength of a flag you typed,
and nothing tells them
afterwards except the half-written files they find.
What a yes leaves undecided:
whether your command waits for the handler's receipt or returns once the request is filed,
and whether the line names the task's command line.

**Option B:
two forces refuse and name it.**
How it works:
the removal stops before any signal,
prints the task's account,
pid and command,
and says `git worktree unlock` will not help because the holder is a process.
What you see:
exit 1 with that line,
and the clone still mounted with its data intact.
What you lose:
the removal does not finish,
and you have to find a process you cannot even signal.
What a yes leaves undecided:
whether the refusal reuses the pool's `dataset is busy` or gets a word of its own.

Ranking A > B,
because A finishes the removal you asked for and B leaves you holding a process you
have no right to touch;
the cost of A,
the other person's unsaved work,
is real and is the reason
B is on the list at all.

### Question 2: a process of a unit you cannot signal holds the clone

A unit of another account,
or of the system manager,
has a task working in the clone.
The same `EPERM` applies,
and the handler is the only channel.

**Option A,
recommended:
end it through the handler,
and refuse for a system unit.**
How it works:
as question 1's option A,
with one read added:
a task whose control-group line names a unit under
`system.slice` is a system unit's task,
and the removal refuses there by name.
What you see:
the clone destroyed with one line naming the unit,
or exit 1 naming the system unit.
What you lose:
the other account's service work where the unit is not a system unit;
nothing but the wait where
it is.
What a yes leaves undecided:
whether a task a person started under a *system* unit (a `systemd-run` with root,
say) counts as a
system unit,
which the control-group line alone cannot tell.

**Option B:
end a system unit's task too.**
How it works:
the same,
with the `system.slice` exception removed.
What you see:
the clone destroyed,
and a machine-wide service stopped.
What you lose:
whatever that service was doing for the machine,
which is not the clone's business.
What a yes leaves undecided:
whether the unit is restarted and re-holds the clone,
which question 4's fallback answers for the person's own units and nothing answers for a system one.

**Option C:
refuse every unit's task.**
How it works:
exit 1 naming the unit and `systemctl stop`.
What you see:
the clone intact and a command to run.
What you lose:
the removal does not finish until you run that command.

Ranking A > C > B:
A finishes and keeps the machine's services out of a flag's reach;
C does not finish;
B finishes at the cost of a machine-wide service,
which no record of this
project puts behind a typed flag.

### Question 3: a locked mount namespace holds the copy

The holder's mount namespace is owned by a user namespace of its own,
which is what a bubblewrap sandbox,
a rootless container,
or a sandboxed unit gets.
No unmount reaches that copy:
this round measured `EINVAL` inside it as root (`keep/arm-userns.log`),
and the 2026-11-27 round measured the same.

**Option A,
recommended:
two forces end that namespace's tasks.**
How it works:
the signal stage ends the namespace's tasks,
`SIGTERM`,
the wait,
`SIGKILL` through a pin,
and the copy goes with the namespace;
then the act's own unmount and the destroy follow.
What you see:
the clone destroyed,
and one line naming the sandbox and the tasks it ended.
What you lose:
everything the sandbox or container had not saved.
What a yes leaves undecided:
whether a locked namespace belonging to *another account* takes question 1's route or refuses.

**Option B:
keep `holder-locked`.**
How it works:
the removal refuses before any signal,
naming the namespace and its holder.
What you see:
exit 1 with that line,
and the clone intact.
What you lose:
the removal does not finish,
and you have to find and stop the sandbox yourself.

Ranking A > B,
because the only other way to free a locked copy is to stop its tasks by hand,
so B moves the same work onto you and calls it a refusal.
A amends the answer of 2026-11-22 quoted in
"What this amends",
and that amendment is the price.

### Question 4: a locked holder that is a service unit of your own user manager

This is the one locked holder the built arm already has a route for:
stop it through the manager
that owns it (`a_locked_holders_own_unit.rs`,
`a_service_unit_stopped_through_the_accounts_manager.rs`).

What this round found first,
and what it means for the question:
on systemd 259 with an unprivileged account,
a `PrivateMounts=yes` transient unit had **no mount
namespace of its own** and held no copy of the clone (`keep/arm-locked.log`),
and the image
forbids unprivileged user namespaces outright
(`kernel.apparmor_restrict_unprivileged_userns=1`),
so the account cannot make a locked namespace
by hand either.
The shape this question is about exists on this project's own record for root's manager on systemd
255,
and a bubblewrap sandbox on a machine whose kernel allows unprivileged user namespaces;
neither was available on this round's guest.
So this question is put on the 2026-11-27 reading and the code,
and this round's own contribution is
the `Restart=` reading and the fact that the shape did not appear.

**Option A,
recommended:
the manager's stop,
with the tasks' end as its fallback.**
How it works:
four reads tell the unit apart (the namespace's owner,
the unit's name in the control-group line,
the manager's answer for that name,
and your protected set);
then `StopUnit`,
and a wait of `T_stop` for the control group and the namespace to go;
where the stop was refused,
a task is still there at the bound,
or the unit is active again,
the act ends the unit's tasks and says so.
What you see:
the clone destroyed,
one line saying whether the manager's stop or the fallback did it.
What you lose:
the service's work,
ended the way its own manager ends it,
or,
in the fallback,
everything its tasks had not saved.
What a yes leaves undecided:
whether the fallback waits one `T_stop` for a `Restart=` unit to come back before ending tasks,
which this round measured mattering:
a unit whose main task was killed outside its manager was `active` again inside two seconds with a
new main task that held the clone (`keep/arm-unit-stops.log`).

**Option B:
end the unit's tasks directly and never ask the manager.**
How it works:
the signal stage over the unit's control group.
What you see:
the clone destroyed,
with no line from the manager.
What you lose:
the service's own cleanup,
which a stop runs and a `SIGTERM` to its tasks may not,
and the manager's accounting of what happened.
What a yes leaves undecided:
the same `Restart=` question,
with no manager call to answer it.

**Option C:
keep today's refusal.**
How it works:
`holder-locked`,
naming the unit and the stop command.
What you see:
exit 1 and a command to run.
What you lose:
the removal does not finish.

Ranking A > B > C:
A finishes and keeps the service's own way of leaving,
which is the only
difference between A and B;
B finishes and takes the service's cleanup away;
C does not finish.

### Question 5: the command must end its own session or one of its own ancestors

The holder is your terminal's shell,
your terminal,
or a process above the command.
Ending it ends the session the removal is printing into.

**Option A,
recommended:
the removal detaches itself first.**
How it works:
after writing the plan,
the wrapper forks;
the child calls `setsid`,
changes to a directory outside the clone,
ignores `SIGHUP`,
writes the caller property with its own identity,
and makes the act;
the parent exits.
The act then ends the recorded session leader and ancestors,
which are neither its session nor
its ancestry any more.
What you see:
the shell dies;
the clone is gone;
the removal's own lines are in the receipt and
the plan,
not on the dead terminal.
Measured this round,
with a pty and a session leader holding the clone:
a child that stayed in the dying session received `SIGHUP`,
and a child that called `setsid`,
changed to `/` and ignored `SIGHUP` received nothing and kept working in a session of its own
(`keep/arm-session-death.log`).
What you lose:
the terminal's output,
which the other options lose too,
and the exit status,
which has no reader.
What a yes leaves undecided:
whether a failure in the detached child is reported only through the receipt or also through a
line the next command prints,
and how long the plan is kept when the act never finishes.

**Option B:
the removal re-executes itself as a unit of your own user manager.**
How it works:
a transient user unit runs the act and writes the request;
the wrapper waits for the receipt.
What you see:
the shell dies,
the clone is gone,
and the manager's journal names the unit.
What you lose:
the manager must be running **and lingering must be on**,
because ending the last
login session stops the account's manager after `UserStopDelaySec`,
ten seconds by default,
which stops the unit mid-act;
and the receipt is the only thing naming you as the cause.
What a yes leaves undecided:
which of the two fallbacks is used where lingering is off (the detach or the plan).

**Option C:
the plan and the next command.**
How it works:
write the plan,
end what must be ended,
print one line saying the next command finishes it,
stop.
What you see:
the shell dies,
then you run the removal again and it finishes from the plan.
What you lose:
one command,
as the owner's own option text says.
What a yes leaves undecided:
how long the plan stands before the slot refuses a new add.

**Option D:
unmount under them and end nothing.**
How it works:
the caller's namespace loses the copy;
the shell stays.
What you see:
the removal's ordinary report,
because the terminal survives,
or `EBUSY` while the
caller stands in the clone.
What you lose:
nothing is ended,
and the commonest case is not passed.

**Option E:
the root mode,
and a request that outlives your command.**
How it works:
the installed handler does the act after your command is gone.
What you see:
the clone destroyed with nothing on your terminal.
What you lose:
the rule that a forced request's authority lasts only while its writer lives,
quoted in "What this amends".

Ranking A > B > C > D > E:
A over B because A finishes on any account that can fork,
and B needs a manager and lingering,
which a server account and a container do not have and a desktop does not have by default,
while what B adds is a journal line and not a finished removal.
B over C because both finish and B finishes in one command where C needs two.
C over D because C passes every case by ending what holds the clone and D does not pass the caller
standing in it.
D over E because D ends nothing and needs no new authority,
where E makes a destructive request
act after its writer is gone,
which is the one thing the duration rule exists to prevent.

**Pairs that cannot both be built**:
B with question 6's option B.
B starts a unit,
and question 6's option B refuses every removal the manager started,
so the pair refuses the route it recommends.

### Question 6: a removal the service manager started

Today the removal refuses `--force` outright when the command's parent is the manager,
or its session leader's parent is,
because the manager "cannot be told apart from the process that
asked for it" (`started_by_the_service_manager.rs`).

**Option A,
recommended:
lift it,
and name the requester by its own unit.**
How it works:
the removal reads its unified control-group line,
takes the last component,
and requires it to end
`.service` or `.scope`;
that unit's tasks are the never-end set;
a line whose last component names no unit refuses.
Read on this machine (`keep/arm-cgroups.log`):
a Pi run's own command carries
`0::/user.slice/user-1000.slice/user@1000.service/app.slice/<the run's unit>.service`;
a Claude Code Bash command carries `.../app.slice/claude-code-bash`,
which names no unit;
and a command under `systemd-run --user --scope` carries `.../run-<token>.scope`.
What you see:
the removal runs where it refused before;
the report names the unit it was started from.
What you lose:
the sentence that told you to run it as a child of the process that wants it,
and the guarantee that a request from a unit cannot act:
its witnesses are the journal and the
receipt.
What a yes leaves undecided:
whether a unit name that no longer exists refuses or falls back to the session,
and whether the harness's own unit name (a run's service,
which is every command of that run)
should be the set rather than the command.

**Option B:
keep today's refusal.**
How it works:
exit 2 with the sentence naming the manager and the remedy.
What you see:
the same refusal as today,
under any force count.
What you lose:
a removal run from a unit,
a timer or a hook never finishes with any force count.
What a yes leaves undecided:
nothing;
it is today's behaviour.

Ranking A > B:
A is the only one of the two that finishes a removal the manager started,
and the property the refusal protected (the removal never ends the manager or an unrelated unit) is kept
by the new set;
B keeps a refusal whose stated reason was that the requester could not be told
apart,
which the control-group line answers.

### Question 7: which refusals survive two forces

Every state in "Every refusal that survives two forces" stays refused under the recommendation.
Each is listed with its reason there,
and each can be lifted only by a mechanism the act does not
have today:

- a blind read,
  an unendable holder and a task that outlives `SIGKILL` are states where the act
  does not know what stands or could not finish what it started;
- a parent mount that is not the checkout's,
  and a mount under the copy,
  would destroy another
  dataset's data;
- no destination for way 7 leaves no route that names the copy;
- a taskless namespace on a kernel without `listns` cannot be named;
- a control-group line that names no unit gives no identity;
- a plan that cannot be read whose clone is gone,
  and a lock that could not be deleted,
  are states where finishing deletes the only record,
  which the force round's own record keeps;
- a system unit's task is question 2's exception;
- and four refusals the force round's record keeps:
  transferred history,
  the origin-side hit,
  the deferred-origin checks,
  and the holder scans,
  which still run and still refuse what they
  cannot prove.

**Option A,
recommended:
keep all of them.**
What you see:
a removal that stops with a sentence naming the state and,
where a command helps,
the command.
What you lose:
a removal that meets one of these does not finish,
whatever you type.
What a yes leaves undecided:
whether any one of them should be lifted later,
which this answer
does not decide for good.

**Option B:
lift one or more by name.**
How it works:
the named state stops refusing,
and the act goes on with what it has.
What you see:
the removal finishes,
and the report says what it did without a proof.
What you lose:
for each lifted state,
the thing its reason protects,
which is another dataset's
data,
the only record of an earlier run,
or the truth of what the act could not read.
What a yes leaves undecided:
what the act does when the lifted state is the only thing that stood
between it and a destroy it cannot attribute.

Ranking A > B:
B lifts the only guards the act has for states it cannot read,
and each of them
protects data rather than a process;
A keeps them and says so.

### Question 8: `T_wait`, the wait between `SIGTERM` and `SIGKILL`

**Option A,
recommended:
2000 ms**,
9.9 times the slowest cooperating exit this round measured (202.905 ms).
How it works:
a holder is asked out;
if it is still there after 2000 ms it is killed through the
same pin.
What you see:
a cooperating holder leaves in milliseconds and you never wait;
a holder that ignores the ask costs two seconds.
What you lose:
two seconds,
once,
for a holder that will not leave.
What a yes leaves undecided:
whether the wait is one budget for the whole act or restarted for each
holder,
which changes what a clone with three ignoring holders costs.

**Option B:
1000 ms**,
half,
4.9 times the slowest cooperating exit.
What you see:
an ignoring holder costs one second.
What you lose:
the margin over a holder that flushes a large file,
which this round did not measure
on a real disk.
What a yes leaves undecided:
the same budget question.

**Option C:
5000 ms**,
two and a half times A.
What you see:
an ignoring holder costs five seconds.
What you lose:
five seconds for every holder that will not leave.
What a yes leaves undecided:
the same budget question.

Ranking A > B > C:
A is the only one with margin over a measured cooperating exit and without a
five-second ceiling on a holder that ignores the ask;
B halves the margin for one second;
C pays three extra seconds for margin nothing measured a need for.

### Question 9: `T_h`, the heartbeat period, and its window

**Option A,
recommended:
1000 ms,
with four periods,
so a window of 4000 ms.**
How it works:
the handler advances a counter each period;
the wrapper reads four periods of no
movement as a dead handler.
What you see:
a dead handler costs four seconds before the removal cancels itself and leaves the
plan.
The lowest measured basis for it is this round's act stage at 40.232 ms,
so the period is 24.9
times the slowest stage and the window 99 times it.
What you lose:
nothing at rest;
on a parent past about 205 clones,
a live handler running one sweep
can be read as dead,
and the removal cancels itself and leaves the plan for the next command.
What a yes leaves undecided:
whether the window is four periods or a fixed duration,
which matters as soon as the period moves.

**Option B:
500 ms,
window 2000 ms.**
What you see:
a dead handler costs two seconds;
a sweep past about 100 clones can read as death.
What you lose:
half the window,
for twice the false cancellations on large parents.

**Option C:
2000 ms,
window 8000 ms.**
What you see:
a dead handler costs eight seconds,
and the window covers a 256-clone sweep.
What you lose:
eight seconds of a person's time every time a handler dies.

Ranking A > C > B:
A keeps step 22's window,
which covers this project's own parent size today
(between 31 and 33 linked worktrees when the gate read counted) and not a 256-clone sweep;
C covers the larger sweep and costs four more seconds of a dead handler;
B narrows the window to the point where a third of step 22's own 256-clone readings would read as
death.

### Question 10: `T_stop`, the wait for a locked unit's stop

**Option A,
recommended:
2000 ms.**
How it works:
the manager is asked to stop the unit;
the act waits up to 2000 ms for the control
group and the namespace to go,
then ends the unit's tasks and says so.
What you see:
a service whose own stop is quick is stopped cleanly;
a slow one is cut short at two
seconds and its tasks ended.
What you lose:
a service whose `ExecStop` needs longer than two seconds,
which the manager's own
default allows up to ninety,
is ended the hard way.
What a yes leaves undecided:
whether the wait is one per unit or one for the whole act,
and whether a unit still active at the bound is asked again before its tasks are ended.

**Option B:
1000 ms.** What you see:
the same,
one second earlier.
What you lose:
twice as many
services cut short.

**Option C:
5000 ms.** What you see:
the same,
three seconds later.
What you lose:
three seconds for a unit that is not going to stop.

Ranking A > C > B:
step 22 has no stop measurement at all and this round's slowest stop of a unit that holds nothing
is 13.561 ms,
so the wait is a bound on a service's own cleanup and not on the
manager's answer;
A is the only arm that neither doubles the cut-short rate (B) nor pays three
extra seconds for a service that is not stopping (C).

## What the gate read got wrong

Each finding of the read of 2026-11-28,
and what this revision did with it.
A finding is "wrong" only where the line or the run that settles it says so.

### H1: the rules contradicted each other

**Right,
and fixed.**
The two sentences it quotes (the manager and "no other unit" never ended,
beside a unit's task
ended) are gone,
replaced by the one rule in "The two rules the gate read found contradicting each
other".
Its scope claim is right too and is corrected in "A scope,
and the shell that launched it":
the scope's jobs share the caller's session but not its control group,
so the shell that launched
the scope is outside the set,
and this revision says what happens to it.
Its absence finding (no table of holder classes) is answered by "Holder classes against no force,
one force and two".

### H2: the unit route's identity is unstated

**Right,
and fixed.**
"How long a forced request's authority lasts" is quoted in "What this amends" and kept:
the unit's own task writes the request,
so the requester of record is the process that runs the
act,
and the handler's identity check is meaningful.
"This is not the root mode" says why.
Lingering is named as part of the route's trigger,
with `UserStopDelaySec` and the ten-second
default.
What the read says the arms did not exercise (root's manager,
a script doing the kill itself)
is what question 5's option B still rests on:
**designed,
and not run on an unprivileged manager
with lingering off.**

### H3: what is built is misdescribed

**Right,
and fixed.**
The two rows carry an ending in the data,
the taskless route is built,
and the locked refusal is
in the preflight;
"What is built today" says all three,
and "The delta a release would have to
make" lists what a release changes.
One thing the read could not see,
because it read commit `0d03853329`:
merge `7c4962988d` landed
after that commit and the removal **does** read a force count now,
in the four places
"What is built today" names.
The read's statement that "the removal reads no force at all and no part of the code reads two" is
therefore wrong at `main` and right at the commit it read.
Its finding that no force count ends a process stands.

### H4: the values are round numbers carried over

**Right,
and fixed.**
"The values" states each as step 22's proposal with what sizes it,
and states what contradicts
them,
including its own 256-clone and 512-clone sweeps and the "above about 205 clones" reading.
The "sixty-four" figure is corrected there.
The read's point that a timer thread makes a stage's length no bound on the period is accepted:
`T_h` is now said to be bounded by starving the timer thread,
which nothing has measured.

### H5: the refusals and the fourth record

**Right,
and fixed.**
"Every refusal that survives two forces" lists twelve,
each with its reason,
and question 7 puts
them to the owner.
The fourth record is named and quoted in "What this amends".
The scope-of-yes point is accepted:
a yes to question 1 covers ending another account's task and
nothing else,
and "what a yes leaves undecided" says so in each question.

### H6: the first screen

**Right,
and fixed**:
"What is asked,
what is recommended,
and what rests on a measurement".

### M1: the arms did not test what they were cited for

**Right,
and fixed by narrowing.**
The two service-manager arms' claims are no longer made:
the earlier revision's sentence that a
unit "held its copy" and the `EBUSY` it cited are gone,
and the `plan-then-next-run` arm is cited
only for what it shows,
that a plan survives and a second run finishes from it.
The candidate the read says was never examined (the detach) is now question 5's option A,
designed in full,
and measured this round with a pty:
a child that detaches receives no `SIGHUP` where one
that stays in the session does (`keep/arm-session-death.log`).

### M2: "holder gone, dataset free" through settle sleeps

**Right,
and fixed.**
`T_release` is proposed with this round's reading:
the destroy is asked at the instant the exit is
reported,
then every millisecond until it answers or one second passes.
The reading is in "This round's readings".

### M3: bundled decisions and unreachable combinations

**Right,
and fixed.**
Ten questions,
one decision each,
with the pair that cannot be built named in question 5.
The body no longer ranks complements as alternatives:
"end then unmount" is the act's order and
not an option beside its parts.

### M4: no reasons between adjacent pairs

**Right,
and fixed**:
every question's ranking gives one.

### M5: labels without a result, and refusals re-asked

**Right,
and fixed.**
Each option has "what you see",
"what you lose" and "what a yes leaves undecided",
and the slot
holds a choice rather than a cost.
Question 4's option B (keep today's refusal) is kept only because the read's point is that the
owner's own option text accepted the service manager:
so it is now question 6's option B and is
labelled as today's behaviour,
while question 4's is about the stop's route,
which the owner's
text did not touch.
Internal labels ("the retort",
"the protected set",
"way 7",
"the kind's own word") are gone from
the questions.

### M6: readings on the older stack read as settled

**Right,
and fixed by stating four facts with every reading.**
The earlier revision's sentences that read as settled are either corrected ("the shape 255 gives"
is now an open question for 262) or moved to "What was not measured,
and why".
This round added readings on a newer stack where it could,
and says which of the four facts each
of them has.

### M7: what a person loses

**Right,
and fixed**:
five losses the read names (ancestors,
other accounts,
dependent units,
outside the clone,
the clone's own uncommitted files) are in "What a person loses" and in each
option.

### L1: slips in what the design quotes

**Right,
and answered.**
None of the earlier revision's quoted blocks survives:
this revision quotes this round's own logs,
and every quoted line is checked against its log by a script before the commit
(`keep/check-quotes.py`,
whose finding count is in the report).
The earlier logs' slips (a start time,
six lines under the wrong log's name,
a paraphrased refusal,
two rounded multiples) are recorded here so a reader of the old revision knows what to distrust.

### L2: two timing arms were not the work they were named for

**Right,
and fixed.**
This round's stage arm is named `enter a holder's namespace, unmount, reap the holder, unmount at
home`,
and the enter-and-unmount is timed beside it as its own arm,
so the parts can be told apart.
Every ratio in "The values" is given with the range it was taken over where the arm has more than
one round,
and this round's own stage control turned out to time the fixture's creation rather than
an unmount,
which the values section says instead of using it.

### L3: arms whose controls could not have failed

**Right,
and fixed.**
The holder arm of this round has a before-state for every shape:
a plain unmount refused (path
holders) or a destroy refused with the pool's own `dataset is busy` (namespace holders).
`MNT_FORCE`'s reading cites the sources the read itself checked
(`do_umount` calling `umount_begin` only where the file system defines one,
and OpenZFS defining
none).

### L4: two numbering systems for "Question 3"

**Right,
and fixed**:
ten questions,
numbered once,
each named by its topic,
and no sentence refers
to a question by number alone.

### L5: re-running an arm on the backend the project uses

**Wrong at `main`.**
The earlier revision's block said an arm cannot reach its guest on the OVHcloud backend because of
the shared folder;
the package's own README says "Nothing an arm does is refused on `ovh` any
more",
and `bin/mvm-guest-shared-folder.ts` makes `/mnt/shared` once per guest.
This round ran every arm on that backend through that folder,
which is the run that settles it.

## Revisions

- The round that wrote the first text:
  the structure,
  the four questions and the guest arms of 2026-11-28.
- The same round's second text:
  the measurements quoted with the logs that answered them.
- The revision against the gate read of 2026-11-28 (`06cf14219a` and the commits after it):
  one first screen,
  one table of holder classes,
  what is built against what a release changes,
  the identity answer,
  the refusals listed,
  the values restated,
  ten questions in place of five,
  and this section.
  The round that made it also ran its own arms on an OVHcloud guest,
  and each reading below
  carries that guest's four facts.
