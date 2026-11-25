# Design stance

This page records how the owner wants this package to be designed and judged.
It is read before changing the wrapper,
its tests,
its gate image or its release tooling.

## We control all the code

The owner said,
on 2026-11-01:

> I think we can make the git wrapper much better.
> Because we control all the code.
> Don't take "Good" as perfect as "Good" breaks down whenever it meets real world.
> Happened so many times already.

Everything in this package is ours:
the wrapper,
its tests,
the gate image build,
the launchers,
the mutation and fuzz tooling,
and these docs.
When a limit,
a delay or a failure comes from our own code,
the fix goes in our code.
Do not route around it,
do not document it as a known limit,
and do not accept a workaround where the cause can be removed.

The owner has said the same thing in other words earlier in this work:

- "Fix even pre-existing issues of the pkgs you own."
- "Shouldn't the proper fix be making our code faster?"
- "We control the parallel launcher,
  so the right move is to fix it instead."
- On flaky tests:
  they "should be fixed properly".

From lines 48306 to 48321 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The ceiling is about 16 s, and the three steps over it (2026-11-08)

The owner was shown the scratch `git commit` steps of this repository's commit timing log
(886 records that name the step,
twelve of them past the 3,006 ms ceiling then in force),
chose "About 16 seconds",
three times the slowest ordinary step (5,444 ms),
and added:
"About 16s,
and investigate what happened for those 3 commits.
If we can't figure it out,
then we have a defect and should log more."
"Those 3 commits" are the steps that would still run past 16 s:
26,256,
36,651 and 171,383 ms.

From lines 48912 to 48923 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

Recorded here only (no other home found):
- "We also have the advantage of git itself being OSS so we're not just wrapping a black box."
  (2026-11-03T07:36:28-05:00).
  Practice:
  for every Git step the wrapper runs or replaces,
  read Git 2.56.0's own source for what it locks,
  waits on and can exit
  nonzero for,
  cite file and line,
  and where a step moves in process port Git's logic so the result equals Git's.
  A v2.56.0 clone is at `~/temp/agent/git-v2.56.0-2026-11-03`.

## Concurrent committers will keep growing

The owner also said:

> Assume the chance different people/agents trying to commit at the same time is going to grow more and more.

A design that is adequate for today's commit rate is a hypothesis about tomorrow's,
not a result.
What grows with the number of committers is burstiness,
tail latency,
wasted optimistic work and the cost of every failure mode under contention.
The measured baseline at that time was 582 commits in 24 hours across all branches (`git log --all`),
one per 148 s;
the owner's premise is that burstiness,
not that average,
is what grows.

## "Good" is not "perfect"

A design,
a test,
a measurement or a release that looks good against the cases we thought of
is a claim about the real world,
and the real world here is hostile in ordinary ways:
loaded hosts,
slow fsync,
killed processes,
closed terminals,
huge repositories,
other tools' state in the same repository,
and many committers at once.
Each of the following looked good until it met that world.
Each is recorded here with its measurement.

- **A private index that kept no stat data.**
  An explicit-path commit rebuilt its private index with a plain `read-tree`,
  so the commit step re-hashed every tracked file.
  The commit step took 2377 ms of a 2740 ms wrapped commit on a 20,218-file clone,
  and ran more than 640 s live on this repository under load.
- **One default bound for every Git child process.**
  A single 1800 s bound looked safe and hid a stall behind silence.
  The per-class bounds that replaced it were themselves questioned,
  because the cost of an index refresh grows with the number of stat-dirty files.
- **Tests and a performance fixture that were too small to tell the truth.**
  The performance fixture holds 30 files and its commit scenario was not enforced,
  so the 87 percent finding above stayed invisible.
  Branch tests that restored Git state by running more Git took 34 to 44 s each,
  and 1.3 s for thirty tests once each case started from a fresh copy.
- **A gate image that recompiled everything.**
  Any edit under the package tree recompiled 121 crates.
  After layering,
  a test-file edit rebuilds one crate:
  982 s became 153 s.
- **Locks held across unbounded work.**
  A waiter could give up after 95 s on a holder that was still legitimately working,
  and waiters retry every 20 ms with no order among them.

## The directory a command starts in is an input

The current directory belongs to the whole process.
`cargo test` runs a crate's tests as threads of one process,
so a control that moved it made every other control read that control's fixture,
or a fixture it had already deleted,
and issue 6454 measured exactly that:
seven controls called `std::env::set_current_dir`,
and a module run read another control's directory.

The release executable reads its own directory once,
at the entry,
(`src/native/foundation/start_directory.rs`),
and hands it down as an input:
`ResolutionInputs::current_directory` for the search for the real Git,
`ShippedChecks::start_directory` for the guard,
the shell directory,
the worktree plans and the policies,
`CommitCommand::start_directory` for a commit's own relative options,
`RepairEnvironment::start_directory` for a tool's relative search path,
and the `directory` argument of a program lookup.
A caller that starts somewhere else names that directory instead,
which is how a control starts the code under test in its own fixture,
and what the shipped commands decide does not change:
production passes the directory the process was started in,
except for the `git-lfs` lookup,
which is given the directory the started program will run in.
That lookup exists so the pre-check and the start agree about which file runs,
and the started program reads its own relative `PATH` entries where it runs;
the two directories differ only for a command moved by `-C` or for a push whose repository is not the
wrapper's directory,
and there the lookup now names the file the child itself would.

`source_guard/no_test_changes_the_process_directory_tests.rs` fails any source of this crate that calls
`std::env::set_current_dir`,
so a control that needs a fixture to be the working directory passes its path rather than moving the process.

A path that still reads the process directory directly is a gap to close when a decision depends on it:
the repository reader,
the discard audit record's directory,
the forwarded lock notice,
the repair environment's own snapshot,
and the directory a `-C` chain reaches for an interactive upload.
None of them is reached by a control that used to move the process.

## What this asks of contributors

These are working suggestions derived from the owner's words and from the history above.
They are revisable;
the quotes above are the requirements.

- Read a green result as evidence about the cases it covered,
  and say which cases it did not exercise.
- Measure on a realistic size and under the load there is:
  a large repository,
  a loaded host,
  several committers at once,
  with alternating arms and a same-binary control for the spread.
  A benchmark on a toy fixture is not a result.
- Prefer a design whose failure is bounded and visible
  over one that is fast only when everything goes right.
- When a "good enough" decision rests on today's load,
  write down the load it assumed,
  so that the decision can be reopened when the load changes.
- When the cause of a limit is in this package or in its build and test tooling,
  find the cause and remove it here.

## No locks: a resident service serializes commits

The owner decided the commit architecture on 2026-11-01.
Their words,
in order:

> Thoughts on locks:
> I believe they're an anti-pattern.
> I still think the server client architecture has potential.

> "child or resident proces" - another reason for server client architecture.
> Some policies are meant to be ran async.

> What is going on?
> Didn't I say locks are bad?

The last one answered a proposal that still put a kernel lock around the serial commit section.
The direction is a resident single-writer service that owns commit serialization and the work that runs after a commit
(push,
standards review,
maintenance).
No lock,
lock file,
owner record or polling loop belongs in the commit path,
and no second writer falls back to a lock when the service is down.
The design is in `doc/planning/cli-git-resident-service.md`.
Working suggestions from the coordinator,
revisable with evidence:
the wrapper stays a thin client for mutating commit-path commands,
the operating system starts the service and the wrapper never does,
and a client whose service is unreachable fails fast with an actionable explanation.

The owner added a rule for the failure path,
in these words:

> socket activation (OS starts it) or a loud,
> clear failure - no,
> never leak internal things that could be resolved to the user.

A client that cannot reach the service has exactly two outcomes:
the operating system starts the service on demand and the commit proceeds,
or the client stops with a plain message that says what the user can do.
Whatever the system can resolve itself (a version mismatch,
a stale socket,
a crashed service,
a slow start,
a client killed mid-request)
is resolved without the user,
and no internal state reaches the user.
