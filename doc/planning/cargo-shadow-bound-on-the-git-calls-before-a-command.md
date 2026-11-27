# What `cargo-shadow` should do when its Git calls before a command pass their bound

**Unapproved and non-final.**
This is a proposal for the project owner to decide from.
It was written by an agent in the Pi harness (the model it ran is named `writer`)
on 2026-11-27T03:52:11-05:00,
in the worktree of the branch
`docs/planning-issue-6555-proposal-for-the-bound-on-git-calls-before-a-cargo-command`.
It answers [issue 6555][issue],
changes no code,
no test and no SPEC,
and declares nothing complete:
the owner declares completion.
Each claim in this document says what it rests on:
the kept logs of 2026-11-26,
the source of `package/cargo-shadow/cli` as `main` stands,
a real invocation of the installed `cargo-shadow` on this host,
or a measurement under "What was measured".

What this proposal cannot say yet:

- What the stopped Git calls of 2026-11-26 were waiting on.
  None of the 25 round logs kept the child's state,
  and there is no recording of those calls on that host state to read now.
- Whether the 130.6 s call measured here was Git's own work or the work of the wrapper installed as `git`.
  Its standard error was not captured,
  and its neighbour in the same probe did print that wrapper's message about an owed ignored-state copy.
- Why the package's own file reader declined here,
  which is what makes it start Git at all:
  the reader returns a reason and the code drops it (`package/cargo-shadow/cli/src/git_base_resolution.rs:642`).
- What any bound should be,
  if a wall-clock bound stays.
  That is one of the questions at the end,
  and it belongs to the owner.

## What happens today

### The two calls and why they are there

`cargo` in this repository is `cargo-shadow`,
a shim in front of the real `cargo`.
Before it runs the command it was given,
it decides which revision is the verification base and reads the lockfiles that revision holds,
because the trust policy compares the workspace's `Cargo.lock` with the lockfile of that base
(`package/cargo-shadow/cli/src/git_base.rs:2`).

The base is meant to come from the repository's own files,
without Git (`package/cargo-shadow/cli/README.md:156`):

> The base is read from the repository's files without starting git

and Git is meant to answer only what the files cannot
(`package/cargo-shadow/cli/README.md:172`):

> Git is started once for everything the files cannot answer exactly,
> and that one start (`git cat-file --batch-check`) answers every candidate base:

So the first call is a fallback:
one `git cat-file --batch-check` is asked which commit each candidate base names
(`package/cargo-shadow/cli/src/git_base_resolution.rs:144`,
`package/cargo-shadow/cli/src/git_base_resolution.rs:423`),
in priority order `@{upstream}`,
`origin/main`,
`origin/master`,
`HEAD`
(`package/cargo-shadow/cli/src/git_base_resolution.rs:546`).
The second call is one `git show <commit>:<crate dir>/Cargo.lock` per crate directory
(`package/cargo-shadow/cli/src/git_base.rs:166`,
built at `package/cargo-shadow/cli/src/git_base.rs:181`),
each a separate Git process with a bound of its own.

Three facts about where they sit on the path of a command decide the rest of this document.

- The base is resolved whenever `trustLockfile` is true,
  before the install step decides anything (`package/cargo-shadow/cli/src/shadow_step.rs:728`),
  so the first call is on the path of every command,
  even one whose remembered pass would skip the policy check.
- The lockfile reads happen only when the remembered pass does not hold,
  and then once per crate directory.
  Measured here:
  39 reads for one command on a fresh pass,
  and none on the next run of the same commit.
- A per-call bound multiplies.
  Today a command can be stopped 40 times over,
  in sequence,
  and the last of those stops has a 60 s bound of its own.

### The bound and what stands beside it

The number is `GIT_CALL_SECONDS`,
60
(`package/cargo-shadow/cli/src/git_refs_environment.rs:103`).
It is used for the base read (`package/cargo-shadow/cli/src/git_base_resolution.rs:235`),
for the lockfile read (`package/cargo-shadow/cli/src/git_base.rs:193`),
and for `git check-ignore` (`package/cargo-shadow/cli/src/git_base.rs:267`).

Rule `LQM` (`AGENTS.md:1589`) says
"Shipped resource limits (unit caps,
timeouts,
size bounds)
cite a workload measurement or owner decision beside the value".
In form the rule is met.
What stands beside this number is this doc comment
(`package/cargo-shadow/cli/src/git_refs_environment.rs:88`):

> The bound of one Git call,
> in seconds.
> Measured 2026-11-04 on this repository (load average 6.2 to 6.9 on 32 CPUs,
> capped scopes,
> 3 runs each,
> `~/temp/agent/bench_bounds.py` and `bench_online.py`):
> `git cat-file --batch-check` with the four inputs took 0.007 s to 0.018 s,
> `git rev-parse --verify HEAD^{commit}` 0.007 s to 0.008 s,
> `git show HEAD:<dir>/Cargo.lock` 0.008 s to 0.009 s and `git var GIT_CONFIG_SYSTEM` 0.006 s to 0.007 s,
> all in a warm repository.
> A minute is about 3,000 times the slowest of these and leaves room for a cold disk or a promisor fetch,
> neither of which was measured.
> The owner's rule of 2026-11-03 justifies that a bound exists;
> the number is a Delegated pick under the owner's
> "Pick options as if I had picked them and invite veto when I say I'm back"
> (handover "Picks made under the owner's delegation"),
> veto open.

The README repeats it with the same measurement and the same caveat
(`package/cargo-shadow/cli/README.md:1055`).
Read against the days this repository now has,
the measurement understates the work in two ways.

- It timed the system Git.
  The `git` on this host's `PATH` is the repository's own wrapper
  (`/home/user/Monochromatic/.cache/monochromatic-git-policy-cli-bin/git`,
  version 0.6.1-alpha.44),
  it announces itself as "shadowing git at /usr/bin/git",
  and `cargo-shadow` starts it,
  because it starts plain `git`
  (`package/cargo-shadow/cli/src/git_base.rs:181`,
  `package/cargo-shadow/cli/src/git_refs_environment.rs:127`).
  Everything that wrapper does before and after the real Git is inside the bound.
- It timed a host at load average 6.2 to 6.9.
  Under the host state measured here the same call took 80 ms in one window and 130.6 s in another.

### What the stop is and what it says

The deadline is a wall clock,
taken before the child is started
(`package/cargo-shadow/cli/src/bounded_run.rs:406`),
so it covers the scope the child is started in,
the write of the launcher record to disk,
and the call.
At the deadline the run is stopped (`package/cargo-shadow/cli/src/bounded_run.rs:476`),
its whole control group is killed,
and the call returns only once the kernel says nothing is left in it.
The text is built at `package/cargo-shadow/cli/src/bounded_run/bound_and_error.rs:118`
(`bounded_run_bound_and_error.rs:112` before `4f53921f3e` moved the file under `bounded_run/`,
2026-11-27T02:39:05-05:00):

> `{what}` did not finish within `{limit:?}`;
> it and every process it started were stopped.
> It may have changed files before it was stopped;
> this tool did not restore or remove anything because of that.
> Run `{rerun}` again when it can finish

`{what}` is the call with its arguments:
"`git cat-file --batch-check` (cargo-shadow asks it which commit to compare with)"
(`package/cargo-shadow/cli/src/git_base_resolution.rs:237`) or
"`git show <object>` (cargo-shadow reads the lockfile of the verification base)"
(`package/cargo-shadow/cli/src/git_base.rs:195`).
So a stop says which call it was and how long it was given,
and says nothing about what the call was doing:
not the child's process state,
not the file it was reading,
not whether it had moved a byte.
The issue's third ask,
"a stop of either says what it waited for and for how long",
is met for the second half only.

### What follows each of the two stops

The two stops end differently,
and neither ends as a plain refusal.

A stopped base read fails the install step,
which becomes `Preflight::Failed` and is printed at
`package/cargo-shadow/cli/src/shim.rs:313`:

```text
        eprintln!("{message}");
        if !asserts_lock(&arguments) {
            eprintln!(
                "cargo-shadow: warning: the install step did not complete, so `--locked` is added to your cargo command; cargo stops with its own lock error where it would have changed Cargo.lock"
            );
        }
        arguments = with_locked(&arguments);
```

So the person sees the bare stop text,
and then a warning about `--locked` unless the command already had `--locked`.
The rounds of 2026-11-26 pass `--locked` themselves,
so 42 of the 44 base-read stops recorded that day are followed by no warning at all:
one bare line,
and then cargo ran.
The command also loses its base for the rest of that run:
the remembered-pass key becomes `unresolved:HEAD`
(`package/cargo-shadow/cli/src/git_base_resolution.rs:540`,
`package/cargo-shadow/cli/src/shadow_step.rs:736`),
whatever base actually failed,
so nothing later can tell which base was unknown.

A stopped lockfile read degrades instead of failing
(`package/cargo-shadow/cli/src/git_base.rs:208`):

> A lockfile that cannot be read from the base is read as absent there,
> which checks more,
> never less.

with the warning at `package/cargo-shadow/cli/src/git_base.rs:216`,
and an empty text,
so every locked version counts as one that no lockfile held at the base.
That direction is safe for the policy and unsafe for the command:
[the kept troubleshooting record][troubleshooting]
records the round where two old releases,
`cpufeatures@0.3.1` and `rowan@0.15.18`,
then refused a build whose own package had not changed,
and the round ran the same command again and it passed.

### Why the first call is needed at all

The file reader is meant to answer the base without Git,
and it answers for the plain layouts.
It declines for a list of reasons the README carries
(`package/cargo-shadow/cli/README.md:174`),
including a configuration `include`,
an `extensions.*` variable,
and an object that is not seen to be a commit stored whole
(`package/cargo-shadow/cli/src/git_objects.rs:14`).
When it declines,
the reason is dropped
(`package/cargo-shadow/cli/src/git_base_resolution.rs:642`):

```text
    // The files answer for the plain layouts.
    if let Ok(native) = native_base_counting(root, in_ci, environment, &starts) {
```

A real invocation on this branch shows it declining:
`cargo-shadow install --dry-run` started Git 40 times,
one `git cat-file --batch-check` and 39 `git show`,
traced with `GIT_TRACE2_EVENT`.
Which of the listed reasons applied is not recorded anywhere,
because `Ok`/`Err` is all the caller keeps.

## What was measured

### The kept logs of 2026-11-26

The round folders of 2026-11-26 under the run scratch of that day were read whole
before the day's expiry (they expire on 2026-11-27 at about 15:26),
over every `*.log` and `*.txt` file they hold.

- 49 stop lines in 29 round folders.
  44 of them are the base read,
  in 25 folders;
  5 are the lockfile read,
  in 5 folders;
  one folder holds both.
- Counted only over the `keep/` folder each round kept,
  the same tree gives the figures the issue quotes:
  21 folders with a base-read stop,
  5 with a lockfile-read stop,
  25 folders in all.
  The difference is the 4 folders whose only record of a stop is in their `scratch/`.
- 4 further lines in a round of 2026-11-22 contain the same sentence inside a longer message a test printed.
  They are quotations,
  not stops,
  and they are not counted.
- Of the 44 base-read stops,
  2 are followed by the `--locked` warning and 42 by nothing.
- All 5 lockfile-read stops are followed by `verifying <n> versions newly locked since the base`,
  that is by the widening of the check the troubleshooting record describes.

They say nothing about why.
No log kept the child's state,
the file it was reading,
or how long the call had been going before the deadline,
and the spans they cover are the rounds' own,
not this host's state on that day.

### The two calls under this host's load

Every reading in this section ran inside `systemd-run --user --scope --slice=agent-work.slice`
with `MemoryMax=4G`,
`CPUQuota=200%`,
`TasksMax=256`,
`IOWeight=1` and `RuntimeMaxSec`,
against the worktree of this branch,
with the pressure read from `/proc/pressure/io`,
`/proc/pressure/cpu` and `/proc/loadavg` at each reading.
The arms were alternated,
so a slow window is not one arm's.

The first window (2026-11-27T03:29 to 03:31 local,
load average 103 to 232,
`io some avg10` 42% to 93%,
`io full avg10` up to 79.6%):

- `git cat-file --batch-check` with the four inputs:
  130.6 s,
  1.5 s,
  1.9 s,
  2.7 s,
  3.0 s.
- `git show HEAD:package/git-policy/cli/Cargo.lock`:
  1.6 s,
  1.2 s,
  2.2 s,
  2.8 s,
  22.7 s.
- `git show HEAD:package/cargo-shadow/cli/Cargo.lock`:
  1.0 s,
  1.1 s,
  2.0 s,
  3.4 s,
  7.6 s.
- `git var GIT_CONFIG_SYSTEM`:
  1.2 s to 3.7 s.
- `git --version`,
  which reads no object at all:
  1.4 s to 2.9 s.
- The scope start `cargo-shadow` makes before a child
  (`systemd-run --user --scope --quiet --collect --property=Delegate=yes ... -- /bin/true`):
  18 ms to 460 ms.

The second window (03:38,
load average 110 to 129,
`io some avg10` 51% to 55%;
`io full avg10` was not recorded in this window):

- `git cat-file --batch-check` with the four inputs:
  8.6 s,
  then 80 ms,
  81 ms,
  83 ms,
  93 ms,
  95 ms.
- `git show HEAD:package/git-policy/cli/Cargo.lock`:
  103 ms,
  113 ms,
  118 ms,
  118 ms,
  123 ms,
  254 ms.
- `git --version`:
  49 ms to 108 ms.

The same call gave 80 ms in the second window and 130.6 s in the first,
in the same repository,
minutes apart.
The first window's `io full avg10` fell from 26% to 0.3% across it,
and its calls went from 1.2 s or more to their own 80 ms range as it fell,
which is the association the issue drew between the disk stall and these calls.
Two things must be said about it.
The pressure was read before each call,
so a stall that began while a call ran shows only in the next reading:
the 130.6 s call began at `io full avg10` 26.05%,
and the next reading,
23 s later,
was 79.61%.
And it is an association across 11 readings of the base read and 11 of the lockfile read,
not a proven cause.
The 130.6 s reading is one call whose standard error was not kept;
the 8.6 s one beside it did print the Git wrapper's line
"the ignored-state copy an interrupted `git worktree add` owed ... is still owed",
so at least part of that call was the wrapper's own work and not the read.

The `git --version` arm is the load-bearing one for this proposal.
A call that reads nothing took 1.4 s to 2.9 s in the first window and 49 ms to 108 ms in the second,
which is the wrapper's own cost,
not Git's.
Against 2.9 s,
a 60 s bound is 20 times the fixed cost of starting Git on this host,
not the 3,000 times the doc comment reasons with,
and that fixed cost is the floor every bound has to be sized from on a loaded host.

### One command's whole set of calls

`cargo-shadow install --dry-run` on this branch,
2026-11-27T03:35 local,
load average 111,
`io some avg10` 60%:

- 40 Git starts:
  1 `git cat-file --batch-check` in 98 ms,
  and 39 `git show` calls.
- The 39 reads summed to 1.46 s,
  with a median of 33 ms and a maximum of 130 ms.
- The whole run's trace spans 8.3 s,
  and 8.29 s of that is one `git status --short`,
  which was the probe's own last command.

A second run of the same command one minute later:

- 1 Git start,
  the base read,
  in 142 ms.
- No lockfile read at all:
  the remembered pass under `.cache/cargo-shadow/verified/` held,
  so the 39 reads are already paid once per remembered pass and not once per command.
- 215 s of wall clock,
  almost all of it in cargo calls,
  not in Git.
  That is why the two Git calls are cheap in seconds and expensive in risk:
  what they can do is stop,
  and a stop of either is what the issue records.

### What could not be measured

- A run of cargo commands under the load of 2026-11-26.
  Synthetic host load is banned,
  so there is no interleaved control at that load and no range from it.
- What the stopped calls of that day were waiting on,
  for the reason given under "The kept logs of 2026-11-26".
- The exit status of 143 that the issue attributes to two rounds' commands after the first message.
  No log under the round folders read here contains it,
  so who sent the signal,
  and at what bound,
  is not recorded.
- The 21.4 s hand measurement the troubleshooting record carries.
  It was made by an earlier round with about twenty concurrent agent builds and is cited there,
  not repeated here.
- Whether the wrapper's owed ignored-state copy is what any of the 25 stops cut off.
  The wrapper writes into a worktree while it copies,
  which is what the stop's own sentence warns about ("It may have changed files before it was stopped"),
  and no log says whether one was running.
- The lockfile read's cost for a crate whose lockfile is large.
  All 39 reads here were between 2 ms and 130 ms,
  and the largest lockfile in the tree was not separated out.

## The options

Each option's example is what that option would print,
made up for this proposal.
None of them is a capture,
and no claim about real behaviour rests on one.

### Option A: judge the calls by the progress meter the wrapper already uses

The repository has already answered this question once,
for the Git the wrapper starts,
and the answer is written down
(`package/git-policy/cli/SPEC.md:10178`):

> The rule:
> no run ends because it has taken long.
> A run ends when it has shown no sign of work for its quiet window.
> The signs are the ones the push reads (`src/native/push/push_progress.rs`):
> bytes moved by the process and everything it started,
> processor time,
> a task able to run or waiting for the disk,
> and,
> for a run that holds a connection to a remote,
> the open connection itself.

Option A applies that rule to the two calls:
read `/proc/<pid>/io` and `/proc/<pid>/stat` for the child and everything it started,
and end the call only when they show no sign of work for the quiet window,
60 s being the window the wrapper uses for a quick query.
The package already watches one observable event,
the child's exit,
through a pidfd
(`package/cargo-shadow/cli/src/bounded_run_containment.rs:650`),
so this adds the second and third observables (bytes moved,
processor time) rather than a new mechanism.
A call stalled on this host's disk is "waiting for the disk",
which that rule counts as a sign of work,
so the stalls that produced the 25 logs would be waited for instead of stopped.

- Pro:
  no working call is stopped,
  which is the outcome the issue asks for,
  on every call and every path.
- Pro:
  it is the project's own recorded rule and mechanism,
  not a new invention,
  so the argument is already settled.
- Pro:
  it also covers `git check-ignore` and `git var`,
  and any Git call a later change adds.
- Con:
  it needs a ceiling anyway,
  because `/proc/<pid>/io` can be unreadable,
  and then nothing is measured and only the ceiling ends a call.
- Con:
  a Git blocked on a lock held by another process looks idle (no bytes,
  no processor time,
  runnable state),
  so it is stopped after the quiet window,
  and that is the one case this option gets wrong.
- Con:
  the quiet windows of the wrapper are themselves labelled unmeasured,
  so the number it borrows is unmeasured too.
- Cost:
  about a day,
  since the meter,
  its procfs reads and its controls would be new to this package.

At the terminal,
on a good day,
nothing is printed.
On a slow day:

```text
$ cargo test
cargo-shadow: waiting for `git show 2eb4228f0fcf:package/git-policy/cli/Cargo.lock` (48s, 41 KiB moved, 3s of processor time)
```

and if it is truly hung:

```text
`git cat-file --batch-check` (cargo-shadow asks it which commit to compare with) showed no sign of work for 60 seconds and was stopped; it had run 71s
```

### Option B: a wall-clock bound sized from a measurement under this load, announced while it waits

Keep a wall clock,
but size it from this host and say what is happening.
Concretely:
an announcement after a few seconds,
a bound of several minutes measured on a loaded host,
and a stop text that names the state the child was in and the file it was reading.

- Pro:
  the smallest change of all the options,
  and every part of it is already written:
  the bound is one constant,
  and its printing is one place.
- Pro:
  it keeps the owner's "nothing should ever wait forever" exactly as it stands,
  with no new mechanism to argue about.
- Pro:
  an announced wait answers the third ask fully,
  for a person who was not watching.
- Con:
  a wall clock cannot tell work from a hang,
  so the next heavier day stops working calls again.
  Measured here,
  one call took 130.6 s,
  which is 2.2 times a five-minute bound and would have been stopped by 60 s twice over.
- Con:
  it leaves the count of stops in the hands of the host,
  which is the defect the issue names.
- Cost:
  an hour of code,
  plus a measurement round on a loaded host to put a number beside it.

At the terminal,
on a slow day:

```text
$ cargo test
cargo-shadow: waiting for the base read (`git cat-file --batch-check`, 42s so far)
cargo-shadow: waiting for the base read (`git cat-file --batch-check`, 2m 42s so far, reading /home/user/Monochromatic/.git/objects/pack/pack-050b2f34a59c1bcff9fbf1f819c53aff67e5893a.idx)
```

and at the bound:

```text
`git cat-file --batch-check` (cargo-shadow asks it which commit to compare with) did not finish within 300s; it was reading .../pack-050b2f34....idx and had moved 0 bytes in 291s
```

### Option C: make the file reader answer, so Git is not started for the base

The first call exists because the reader declined.
Make it answer more of what it now declines for,
and make it say why when it still declines.
The README's list of decline reasons is the work list:
the object stored whole is the one that matters most,
because a deltified commit in a pack is ordinary in a repacked repository.

- Pro:
  removes one Git start from every command,
  with no bound,
  no waiting and no message.
- Pro:
  the reader is already tested against Git itself
  (`package/cargo-shadow/cli/src/git_differential_tests.rs`),
  so correctness has a harness.
- Pro:
  saying why it declined costs a log line and answers the first question anyone debugging this asks.
- Con:
  the reader must agree with Git exactly,
  and every new case is a chance to disagree about which commit a base names,
  which is the wrong direction to be wrong in.
- Con:
  it fixes one of 40 starts.
  The 39 lockfile reads still run,
  and each still carries the 60 s bound.
- Con:
  the case that declines here was not identified,
  so the size of the work is unknown,
  not merely large.
- Cost:
  days,
  because a deltified commit needs the delta chain reconstructed and differentially tested.

At the terminal,
on a good day,
nothing is printed.
When it still declines:

```text
cargo-shadow: the base was read from git, not from the files: the configuration sets an include (`~/.gitconfig:3`), and that reader does not follow includes
```

### Option D: cache the base and its lockfiles per commit

The second call is repeated work with an immutable answer:
the lockfile a commit holds never changes.
Store the text of each crate's base lockfile under the commit it was read at,
in the cache directory the package already owns,
and read it from disk next time.
The package already does exactly this for the policy pass itself
(`.cache/cargo-shadow/verified/`,
`package/cargo-shadow/cli/README.md:149`),
and the measurement under "One command's whole set of calls" shows it works:
the second run of the same command made no lockfile read at all.
The base resolution can be cached the same way,
keyed by the reference file's own size and modification time or by the commit it names.

- Pro:
  removes the larger share of the calls,
  39 of 40,
  and the whole of their risk,
  from every repeat command.
- Pro:
  a cache hit cannot stall on the disk the way Git can,
  and it cannot be stopped.
- Pro:
  it makes a command faster on a loaded host,
  which is the direction the package's own measurements already take.
- Con:
  the first command after a base moves still pays all 39 reads,
  and that is the command a round starts with.
- Con:
  a cache is a new thing to invalidate and to clean:
  a commit that no longer exists,
  a lockfile whose cached read was itself degraded by a stop,
  and the retention the `lean` command would need over a new directory.
- Con:
  it caches what Git said,
  so the first read has the same bound as today unless it is paired with A or B.
- Cost:
  about half a day,
  most of it in the invalidation rules and their controls.

At the terminal,
on the second command in a commit,
nothing is printed.
On the first:

```text
cargo-shadow: reading 39 lockfiles at 2eb4228f0fcf (cached for the rest of this commit)
```

### Option E: refuse the command when the base cannot be read

Do not widen the check and do not lose the base:
a base that could not be read is an unknown base,
and an unknown base refuses the run with the reason.

- Pro:
  the trust check never silently widens,
  which removes the worst outcome of the two stops.
- Pro:
  one small change,
  in one place,
  and the message already exists in the right words.
- Con:
  a slow host becomes a refused build,
  so the round's work stops instead of running slowly,
  and the person has to retry by hand.
- Con:
  it does not stop either call from being stopped,
  and the command it stops costs the round every step that command would have run.
- Cost:
  an hour.

At the terminal:

```text
$ cargo test
cargo-shadow: the verification base of package/git-policy/cli could not be read (`git show 2eb4228f0fcf:package/git-policy/cli/Cargo.lock` did not finish within 60s), so the versions in that lockfile are unknown and the command was not run. Run `cargo test` again when the base can be read
```

### Option F: do what `git` and `cargo` themselves do, which is wait and say so

Both tools this package stands in front of have no bound on local work.
Git waits on a read,
a pack,
a lock and a disk for as long as it takes and prints progress when it can.
Cargo waits on its package cache lock and says so:
`Blocking waiting for file lock on package cache`,
the line that fills these rounds' logs.
Option F copies them:
drop the bound on the two calls,
print what is being waited for,
keep the pipe workers cancelled.

- Pro:
  it never stops working calls,
  and it matches what the tools it wraps do,
  so nothing is surprising about the behaviour.
- Pro:
  the message is trivial to write and needs no measurement.
- Con:
  with no bound there is nothing left to tell a hang from a slow disk,
  and "nothing should ever wait forever" is simply broken.
- Con:
  a command that hangs prints one line and then nothing,
  which is exactly the shape a person not watching cannot diagnose.
- Con:
  the stop machinery the package already has (kill the group,
  wait for `populated 0`)
  would be used only for the user's own command.
- Cost:
  an hour,
  and it is the only option here that needs no measurement at all.

At the terminal:

```text
$ cargo test
cargo-shadow: waiting for `git show 2eb4228f0fcf:package/git-policy/cli/Cargo.lock` at the verification base; this is local disk work and is not timed out
```

and then,
for however long the disk takes,
nothing more.

### Option G: carry the resolved base in the environment from the caller

The base is resolved once per change of branch and commit,
and the base read happens once per command.
Let the caller resolve it,
put it in `CARGO_SHADOW_TRUST_BASE`
(the variable the package already reads,
`package/cargo-shadow/cli/src/git_base.rs:91`),
and every command inherits one base without resolving anything.

- Pro:
  removes both calls from the path of every command at once,
  with no new code path in the shim.
- Pro:
  the base then agrees across every command of a round,
  which is a correctness gain the current per-command resolution does not have.
- Con:
  every caller has to do it,
  and a caller that forgets it gets today's behaviour,
  so the board is half converted forever.
- Con:
  a stale value is silently wrong,
  and it is wrong in the direction that widens or narrows the check without saying so.
  The base would move under the round without any command noticing.
- Con:
  the environment is inherited by everything the command starts,
  so a nested shadow build sees a base it never chose.
- Cost:
  an hour in the shim plus a change to every caller in the repository,
  and to CI.

At the terminal,
on a good day,
nothing is printed.
When the caller resolves the base:

```text
$ cargo test
cargo-shadow: comparing lockfiles with 2eb4228f0fcf (CARGO_SHADOW_TRUST_BASE)
```

## Ranking

Ranked by what each option delivers on a loaded host and for a person who was not watching.
Effort is a cost stated in each option and never a reason for a place here.

Ranking:
D > A > C > B > E > F > G.

- **D over A**:
  D deletes the work,
  A only judges it.
  D removes 39 of the 40 starts measured here,
  and with them the seconds,
  the exposure and the stop,
  while A still pays every call and only makes its ending honest.
- **A over C**:
  A covers every call,
  at every path,
  including the ones a later change adds,
  while C removes one start and leaves the 39 that carry the same bound.
  A is also the project's own recorded rule,
  which is the smaller argument to win.
- **C over B**:
  deleting a call beats bounding it.
  B is still a wall clock,
  so on the next day like 2026-11-26 it stops a call that was working,
  which is the defect in the issue.
- **B over E**:
  B keeps the command running and tells the person what happened,
  while E turns a slow host into a stopped build and gives the person a retry to do by hand.
- **E over F**:
  E ends with the real cause and a refusal the person can act on,
  while F ends with one line and then silence,
  and breaks the "nothing should ever wait forever" rule outright.
- **F over G**:
  F's failure is visible (a command that never returns is at least identifiably stuck),
  while G's failure is silent:
  a base nobody re-resolved,
  giving a check against the wrong commit with nothing printed.

What this ranking would do,
in order:
D first (the 39 reads stop being on every command),
then A (what reads remain end only when they stop working),
then C (the base read leaves the path),
with B's announcement folded into A as a smaller part of the same change.
That is a decision for the owner,
not for this proposal.

## Questions only the owner can answer

1. May a Git call before a command take minutes on a loaded host,
   as long as it is working?
   The answer decides whether A is allowed to wait at all,
   or whether a wall clock stays and B is the shape.
2. If a wall clock stays,
   what number,
   and from which measurement?
   The only readings this round has are 80 ms to 130.6 s for the same call on the same repository,
   so a number needs a policy before it can be written down:
   a multiple of the worst reading,
   or of the fixed cost the `git --version` arm shows.
3. Is the wrapper's "no sign of work for its quiet window" rule the project's answer here too,
   or is it only the wrapper's?
   A applies it to a package that does not currently share the wrapper's meter.
4. May this package keep a cache of base lockfile texts under `.cache/cargo-shadow/` (option D),
   and may `lean` prune it by the same rules it prunes the rest?
5. When the base cannot be read,
   should the command be refused (option E),
   or run with `--locked` as it does today?
   The troubleshooting record argues for a refusal;
   nothing recorded decides it for the shim.
6. Does the same rule apply to `git check-ignore` and `git var GIT_CONFIG_SYSTEM`,
   which carry the same 60 s bound and answer questions nobody is waiting to see the answer of?

[issue]: https://github.com/Aquaticats/Monochromatic/issues/6555
[troubleshooting]: ../troubleshooting/cargo-shadow-base-lockfile-read-times-out.md
