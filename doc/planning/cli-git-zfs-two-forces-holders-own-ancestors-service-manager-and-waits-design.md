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
  a stop ends the unit,
  its control group and its namespace,
  and a unit whose task was killed outside its manager comes back and holds the clone again;
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
- decisions 8 to 10 rest on step 22's measurements and on nothing of this round;
  this round's own timings are stated as readings of a guest that is not the owner's stack.

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
STACK zfs-module=2.4.1-1ubuntu5
STACK zfs-userspace=zfs-2.4.1-1ubuntu5.1
STACK out-ns=mnt:[4026531832]
STACK out-user-ns=user:[4026531837]
STACK listns=-1 errno=EINVAL
STACK ids-a-task-holds=10
```

The four facts every reading below carries are kernel,
OpenZFS,
systemd and account.
The owner's machine is kernel 7.3,
OpenZFS 2.4.4,
systemd 262 and uid 1000.
This guest matches none of the first three,
so **no reading below is a reading of the owner's stack**,
and each passage that needs one says what stays open.

Readings,
each with what answered it:

- **The stack itself**:
  kernel `7.0.0-28-generic`,
  OpenZFS 2.4.1 (module and userspace,
  the distribution's),
  systemd 259,
  account `root`.
  `keep/arm-stack.log`.
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
  the unmount inside answers `EINVAL`,
  and the destroy answers `gone`
  once the task is ended (`keep/arm-holders.log`).
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
  the stop ends the unit,
  its control group and its namespace,
  and a unit whose task is killed outside its manager comes back and holds the clone again
  (`keep/arm-unit-stops.log`,
  `keep/arm-restart-unit.log`).
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
(`keep/arm-unmount-under-the-caller.log` of the earlier round).
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
  OpenZFS 2.4.1 and systemd 259,
  as root and as `ubuntu`.
  Each reading below carries those four facts,
  and none is called a
  reading of the owner's stack.
- **OpenZFS 2.4.4** as the recipe builds it:
  the recipe's build and the round's arms did not both run in the time this round had beside them,
  and the readings that need a pool were taken on the distribution's 2.4.1.
  What stays open:
  whether `zfs destroy`'s busy rule differs in the last three patch releases,
  which the round's source reading at `zfs-2.4.4` and this round's arm at 2.4.1 do not settle by
  themselves.
- **systemd 262**:
  no image on that backend carries it,
  so the locked-unit reading is 259's.
  What that leaves open is stated in decision 4.
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
