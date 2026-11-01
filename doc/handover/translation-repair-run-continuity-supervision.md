# Translation repair run continuity: supervision

Part of the [run continuity index](translation-repair-run-continuity.md).

## Current stop condition

Historical evidence only.
No continuity supervisor or corpus pass is running,
and this file does not authorize recreating one.

## What stops a pass on its own

Read from `START` output at tip `c3f95de9d`,
2026-08-21:

-   Soft budget `259200000ms`,
    seventy-two hours.
    The pass stops accepting new
    entries past it.
-   Hard per-entry cap `25200000ms`,
    seven hours.

BOTH NUMBERS MOVED since this section was first written against tip
 `7a6894152`,
where they were twelve hours and three.
`#114` raised the
 per-entry cap after measuring that four to six entries hit the old one.
Anyone
 reasoning about when a pass ends on its own should read the `START` line of the
 run in front of them rather than any number written here.

So a pass now runs for days rather than for a night,
with entries settled and
 its slice cache holding only whatever was in flight.
Resuming is cheap:
 `discardSliceCache` drops each entry's cache as it settles,
so the cache only
 ever holds in-flight work.


## Two supervisors, deliberately

`~/temp/agent/resume-run.sh` is the older,
single-shot watcher.
It waits for one
 stop,
resumes once,
and exits when that resumed run ends.
It also aborts
 outright if the pass is still alive at its own six-hour deadline.

`~/temp/agent/resume-supervisor.ts` loops instead,
up to eight resumes.

They cannot race.
The supervisor refuses to launch anything while the watcher's
 pid is alive,
so the watcher always gets the field first and the supervisor
 takes over only once it exits.

The supervisor is TypeScript run directly by node,
not bash,
for two reasons.
 The repo forbids shell scripts,
and bash reads a script INCREMENTALLY:
editing
 a running watcher can make it execute a spliced mixture of old and new bytes.
 Node reads the whole file before executing any of it.

## How to stop it

```sh
touch ~/temp/agent/resume-supervisor.stop
```

Checked on every poll of the WAIT loop,
so while the supervisor is waiting for a
 pass to stop,
the file takes effect within a minute.

IT DOES NOTHING WHILE A PASS IS RUNNING,
and this was learned the hard way
 rather than reasoned about.
The supervisor spends that time inside `run()`,
 awaiting its child,
and reaches no stop check until the pass exits.
A stop file
 placed during a run sits unread for as long as the run lasts,
which can be
 twelve hours.

So there are two different operations:

-   To stop the SUPERVISOR while a pass is running,
    kill it by pid.
    Verified:
    killing the supervisor does NOT kill the pass,
    because the pass is a child
    that outlives it.
    That is what makes swapping supervisors mid-run safe.
-   To stop the RUN,
    kill the pass by pid.
    The supervisor then sees the field
    clear,
    and a stop file placed at that point prevents the next launch.

## Guards, and what each one is for

-   **Exact process match.**
    `pgrep --full --exact 'node src/corpus-run/corpus-pass.ts'`.
    A loose `--full` match without `--exact` also matches any SHELL whose
    command line contains that string,
    including the supervisor's own launcher,
    so it reports the pass alive forever and nothing ever fires.
    Measured:
    loose
    matched four processes,
    exact matched only the real one.
### The exact-match guard no longer matches the pass, measured 2026-08-21

THE GUARD IS STALE AND THE SUPERVISOR CANNOT SEE ITS OWN CHILD.
 `PASS_PATTERN` in `~/temp/agent/resume-supervisor.ts` is
 `node src/corpus-run/corpus-pass.ts`,
from when the pass ran straight from
 source.
The mise task the supervisor launches,
 `//package/module/translation-repair:corpus-pass` (moved on 2026-10-18 to
 `//package/translation-repair/corpus-pass:corpus-pass`),
now depends on `build` and
 runs `node dist/final/node/corpus-pass.mjs`.

Measured against a live pass:
`pgrep --full --exact 'node src/corpus-run/corpus-pass.ts'`
 returned nothing while `ps` showed
 `node dist/final/node/corpus-pass.mjs --only ...` running.

So `passRunning` reads FALSE whenever a pass is running,
which is the one answer
 every other guard here is built on.
The supervisor would launch a second pass
 on top of a live one.
What contains it is not the guard:
`lockRunsDir` refuses
 the second pass on the same runs directory,
and the SPIN GUARD then reads that
 fast exit as a spin and stands down.
A second pass pointed at a DIFFERENT runs
 directory has no lock to refuse it,
and would compete for provider concurrency,
 which `doc/troubleshooting/synthetic-aggregate-concurrency-stall.md` measured as
 degrading every backend above roughly seven aggregate streams.

The positive and negative controls recorded further down this document were run
 when the pattern was correct,
and they passed.
They do not carry forward
 through a change to how the task launches the pass,
which is the general
 lesson:
a process-identity guard has to be re-verified against the command line
 the task actually produces,
every time that task changes.

`#169` carries the fix.
Nothing was armed when this was found,
so no double
 launch occurred.

### Fixed 2026-08-22: the guard reads `/proc`, and `pgrep` patterns are abandoned

THE PATH WAS ONLY HALF THE DEFECT.
Correcting `PASS_PATTERN` to the built path leaves the guard exactly as blind,
because `--exact` compares the pattern against the WHOLE command line,
and every pass launched by hand carries `--only <entries>`.
Measured against the live pass,
 in order:

-   `pgrep --full --exact 'node src/corpus-run/corpus-pass.ts'`,
    the shipped pattern:
    exit 1.
-   `pgrep --full --exact 'node dist/final/node/corpus-pass.mjs'`,
    the path corrected and nothing else changed:
    exit 1.
-   `pgrep --full 'node dist/final/node/corpus-pass.mjs'`,
    `--exact` dropped:
    exit 0,
    matching the pass and also the shell carrying the pattern text.
-   `pgrep --full --exact` against the entire command line including the
    `--only` list:
    exit 0,
    matching only the pass,
    and useless because that list changes per run.

So the guard has been blind to every hand-launched pass since it was written,
and correct only for the argument-free passes the supervisor launches itself.
That is why the log carries a real `another launcher won the field` line from
 2026-08-14:
that pass had no arguments.

`passRunning` no longer asks `pgrep` anything.
It reads `/proc`,
the way `watcherAlive` in the same file already did,
and counts a process as a pass when its interpreter is `node` and one of its
 arguments is named exactly `corpus-pass.mjs` or `corpus-pass.ts`.
Matching the file NAME rather than the path is what survives the move that broke
 this.
Matching an EXACT name rather than a prefix keeps a `node --eval` one-liner that
 merely mentions the file from counting as a pass.

DO NOT RESTORE A `pgrep` PATTERN HERE.
Two separate flag interactions have each left this guard silently blind,
and both failed by returning false,
which reads exactly like a clear field.

The scan deliberately reads every argument rather than only the one node treats
 as the script.
The two errors do not cost the same.
Reporting a pass that is not there makes the supervisor wait and launch nothing.
Missing a pass that is there makes it launch a second one into the same runs
 directory and the same slice cache.
A rule keyed on argument position would go blind in the expensive direction the
 moment the task grew a node flag ahead of the script.

#### Controls for the pass guard, and why the old ones passed a broken guard

The controls recorded under "Verifying it rather than trusting it" tested whether
 `pgrep` works,
not whether the pattern matches the passes this project runs.
The positive control there used a supervisor-launched pass with no arguments,
and the negative control varied the PATTERN rather than the process state.
Both are satisfiable by a guard that cannot see a `--only` pass.

The replacement varies the PROCESS STATE,
and runs the real function through the real code path:

-   Fail first,
    per GFP:
    the shipped pattern against the live pass,
    exit 1,
    recorded before any fix was written.
-   Live positive,
    against the running pass:
    `passRunning returned true`.
-   Nothing running at all:
    `passRunning returned false`.
-   A pass carrying `--only`:
    `passRunning returned true`.
-   A shell naming `//package/module/translation-repair:corpus-pass` (moved on 2026-10-18 to
    `//package/translation-repair/corpus-pass:corpus-pass`),
    which is the launcher shape that made a loose match useless:
    `passRunning returned false`.
-   A `node --eval` one-liner naming the built path inside its code:
    `passRunning returned false`.

Every control after the live positive answers true on this host while a pass is
 alive,
because the live pass satisfies all of them.
They run inside a fresh PID namespace instead,
where it is invisible:

```sh
unshare --user --map-root-user --fork --pid --mount-proc bash -c \
  'node /decoy/dist/final/node/corpus-pass.mjs --only x & sleep 1;
   RESUME_SUPERVISOR_PASSCHECK=1 node ~/temp/agent/resume-supervisor.ts'
```

`RESUME_SUPERVISOR_PASSCHECK` exists because `RESUME_SUPERVISOR_SELFTEST` runs
 `smokeCheck` first,
which shells out to a mise task that can rebuild `dist`,
and a running pass is EXECUTING from `dist`.
That coupling is why this guard could not be checked against a live pass without
 risking the run,
and it is a large part of why it stayed broken.
The two are now separable.

-   **Smoke check before every launch.**
    The pass runs straight from SOURCE,
    so
    a session that stopped mid-edit leaves a broken module graph and the resume
    would die instantly.
    `--plan` is the documented zero-quota setup check:
    it
    loads that exact graph,
    resolves the corpus at the pinned tip,
    and
    constructs the client,
    so it also catches a corpus or key failure no build
    would.
    It runs against a throwaway runs directory,
    never the real one.
-   **Watcher IDENTITY,
    not just liveness.**
    The supervisor defers to the
    watcher by pid,
    and `kernel.pid_max` here is 4194304 with the counter
    already at 4054937,
    so pids wrap within days.
    A recycled pid belonging to an
    unrelated process would make the supervisor defer to a stranger for the rest
    of the night while reporting itself armed.
    It now reads
    `/proc/<pid>/cmdline` and requires the watcher's own script name,
    so a
    recycled pid reads as gone.
    An unreadable process reads as ALIVE,
    because
    launching a second pass is worse than waiting.
-   **Spin guard.**
    A resume that exits in under two minutes is treated as a
    spin rather than as work,
    and the supervisor stands down instead of
    spending its remaining attempts.
-   **Attempt budget.**
    Sixty resumes.
    It began at eight and that was wrong,
    for
    a reason worth keeping:
    its purpose is stopping a run that dies instantly
    from spinning,
    which the SPIN GUARD above already does.
    What the count
    actually consumed was legitimate operator restarts,
    since landing a pipeline
    change means stopping the pass so it picks the change up.
    Three of eight
    went that way inside an hour on 2026-08-13.
    Exhausting it would have ended
    the night's run silently,
    which is the one outcome every guard here exists
    to prevent.

## Verifying it rather than trusting it

Every check below was run against the live system before the supervisor was
 armed,
because a supervisor whose detection is wrong is worse than none:
it
 reports itself armed in the log and never fires.

-   Positive control:
    the real pass pattern must exit 0.
    It did.
-   Negative control:
    a bogus pattern must exit non-zero.
    It did.
-   Liveness:
    the watcher pid must read alive and a nonexistent pid must read
    dead.
    Both did.
-   Identity:
    the watcher pid must match its own script name and must NOT match
    a different one,
    and a dead pid must match nothing.
    All three did,
    so the
    check distinguishes "this pid runs the watcher" from "this pid exists".
-   End to end:
    with the stop file present,
    the supervisor logs its arming line
    and stands down,
    which exercises the whole file including module-level
    await.
-   Smoke check:
    run by hand at the current tip before arming,
    exit 0,
    client
    constructed.

The two functions that decide the night,
`run` and `smokeCheck`,
are otherwise
 never exercised until the moment they matter.
They now have a self-test:

```sh
RESUME_SUPERVISOR_SELFTEST=1 node ~/temp/agent/resume-supervisor.ts
```

It runs the real guard through the real code path and logs what each returned,
 then exits before the loop.
Confirmed:
`smokeCheck` true,
`passRunning` true,
 and the smoke log carried the child's own `PLAN ok` line,
which is what proves
 the spawn,
the working directory,
the environment override and the output
 capture all work rather than merely not throwing.
The resume itself calls the
 same `run` with different arguments,
so that path is proven too.

Editing the supervisor file cannot disturb a supervisor already running,
because
 node reads a module completely before executing any of it.
That is the property
 bash lacks and the reason this is not a shell script.

Two probes that FAILED silently while investigating this,
both worth
 remembering:

-   `pgrep --exact --list-full node | rg corpus-pass` reported nothing while the
    pass was running.
    `ps --no-headers -eo pid,etime,args` found it at once.
-   `find <dir> -type f -newermt '-60 minutes'` matched nothing while eleven
    files had been written in that window.
    `touch --date='60 minutes ago' <ref>`
    followed by `find -newer <ref>` reported them correctly,
    and a thirty-day
    reference served as the positive control.

Both would have supported the same wrong conclusion,
that the run was dead.

## The verification run in flight, started 2026-08-21

This one is NOT under either supervisor.
It is a short bounded run started by
 hand for `#138`'s verification at the user boundary,
and it needs different
 handling from a production pass.

### What is running

-   The pass,
    over six entries into a throwaway directory:

    ```sh
    TRANSLATION_REPAIR_RUNS_DIR=~/temp/agent/vub-run1-20260821 \
      mise run //package/module/translation-repair:corpus-pass \
      -- --only lintong,gaoyanger,keyword233,Weideriche_,Zha_Ke,Acheron
    ```

    Moved on 2026-10-18 from `//package/module/translation-repair:corpus-pass`
    to `//package/translation-repair/corpus-pass:corpus-pass`.

    Log at `~/temp/agent/vub-run1.log`.
    Find it with
     `pgrep --full 'corpus-pass.mj[s]'`.

-   A capture poller,
    `~/temp/agent/vub-cache-capture.mjs`,
    copying
     `vub-run1-20260821/slice-cache` into `~/temp/agent/vub-cache-capture` every
     200 milliseconds.
    Log at `~/temp/agent/vub-capture.log`.
    Find it with
     `pgrep --full 'vub-cache-captur[e].mjs'`.

The bracket in both patterns is not a typo.
A plain `pgrep --full` matches the
 shell running the `pgrep` itself,
which is recorded in
 `doc/troubleshooting/pgrep-wait-loop-matches-itself.md` and cost two wrong
 readings while setting this up.

One command answers all of it with no pattern to get wrong:
`node ~/temp/agent/vub-status.mjs`.
It scans `/proc` the way the pass guard now does,
matching on the interpreter and on the script's base name,
so it cannot match the shell that invoked it and arguments cannot defeat it.
It prints the settled entries,
the entry still holding a cache,
the captured file count,
and one line per watched script with its pids.

PREFER IT TO A HAND-WRITTEN PATTERN.
A hand-written `pgrep` was read as a dead poller on 2026-08-22 because the
 pattern named a script that never existed,
which the bracket does nothing to catch.
A status check that under-reports invites restarting a poller,
or worse a second pass into the same runs directory and the same slice cache.

### Why a poller exists at all

`runOneEntry` calls `discardSliceCache` the moment an entry reaches its
 artifact,
by design.
That makes a plain second pass unable to exercise the
 resume path:
it either skips the entry on its artifact,
or re-runs it with no
 cache.
Copying the cache aside while the pass runs is what makes a restored
 second run possible.
`doc/planning/the-third-rendering.md` carries the full
 reasoning under "Verifying the stage at the user boundary".

### The constraint a taking-over session must respect

LIFTED 2026-08-22 08:12Z.
Run 2 finished,
`DONE processed=6 of pending=6`,
so the freeze this section describes no longer binds.
The rest of the section is kept because it explains WHY the digest matters,
which the next cached run will need again.

The historical constraint read:
DO NOT EDIT ANYTHING UNDER `package/module/translation-repair/src/` (moved on 2026-10-18 to
`package/translation-repair/*/src/`) UNTIL RUN 2
 HAS FINISHED.
The slice cache is keyed on the pipeline digest,
which
 `digestPipeline` takes over the built output directory.
An edit moves the
 digest,
every namespace discards,
run 2 re-buys everything,
and the
 verification reports a validator failure that never happened.

Docs are safe:
`tip` moves and the digest does not.
Two builds of unchanged
 source were measured to reproduce
 `sha256-tree-v1:2384524b15c2482c37db147b9654b0036eeebfba7e24b6297854d7bcddef4cc0`.

### How to finish it

1.  `node ~/temp/agent/vub-report.mjs ~/temp/agent/vub-run1-20260821/artifacts`
     parses every artifact back through the shipped whole-artifact parser and
     reports each consolidation slice.
2.  `node ~/temp/agent/vub-capture-check.mjs <artifacts> <capture>` says whether
     the capture holds every settlement worth resuming.
    It is a LOWER BOUND,
    not
     a verdict,
    because `slate-kept-standing` persists on a decision the artifact
     does not record.
3.  Copy the capture,
    never move it,
    into a fresh runs directory's
     `slice-cache`,
    repoint the poller at that directory,
    and run the same
     `--only` line into it.
    A re-bought settlement is the diagnostic evidence and
     the discard will delete it the same way.
4.  Run 2 passes on the per-slice accounting registered under
     "What run 2 must re-buy",
    NOT on a count of
     `reportStreamProgress` lines.
    Those lines are tagged `translation-repair`
     rather than an entry,
    so they cannot be attributed to a slice at all,
    and
     a zero was never reachable.
5.  `node ~/temp/agent/vub-compare.mjs <run1-artifacts> <run2-artifacts>`
     confirms the consolidation records are byte-identical.
6.  Kill the poller.
    It loops forever and will not stop on its own.

### How to stop it instead

Kill the pass by pid.
Its artifacts and its captured cache both survive,
and the
 verification can proceed on whatever subset settled.
A failed or killed entry
 keeps its cache in the run's own directory,
by the same rule that discards a
 settled one,
so re-running that entry in place is a resume case reached
 organically rather than restored.

Nothing in the repository depends on this run.
It writes only under
 `~/temp/agent/`,
outside git,
because a runs directory holds corpus text from an
 unlicensed source.

## What run 2 must re-buy

REGISTERED 2026-08-22 from run 1's capture and artifacts,
before run 2 started.

THIS ACCOUNTING COVERS `Acheron` and `Weideriche_` ONLY,
the two entries settled when it was written.
Four more entries were still to come.
EVERY FURTHER SETTLED ENTRY NEEDS THE SAME ACCOUNTING BEFORE RUN 2 STARTS,
from `~/temp/agent/slice-terminals.mjs` over its artifact and
 `~/temp/agent/capture-decisions.mjs` over its capture.
Reading this list as complete would call a legitimate re-buy a failure.
The criterion this replaces,
zero `reportStreamProgress` lines,
would have
 reported a failure that never happened.

### Why zero was never reachable

`consolidationWorthResuming` in
 `package/module/translation-repair/src/consolidate-driver.ts` (moved on 2026-10-18 to
 `package/translation-repair/corpus-pass/src/consolidate-driver.ts`)
 REFUSES TO PERSIST a `slate-kept-standing` settlement whose decision is
 `declined-indecision` or `declined-rejection`.
Those two are `UNSETTLED_DECISIONS`,
and the reason is given beside them:
a
 thin panel is a fact about a provider on one night,
not a property of the
 question,
and freezing it into the cache would answer every later resume of
 that entry with that night.

So a declining slice is never written to the cache at all.
No poll rate catches
 it,
and run 2 re-asks it BY DESIGN.
`Acheron` slice 0 is exactly that case,
measured in run 1's log at `2026-08-22T02:17:44.907Z`:
`translate stage: winner short of the minimum vote weight; keeping the
 incumbent`.

### What the capture actually holds

Read with `~/temp/agent/capture-decisions.mjs`,
which prints the `decided`
 field the artifact does not carry.

- `Acheron` slice 1 and slice 2 persisted.
   Both `consolidated`,
   both
   `judged` from a `fresh` origin,
   both at gate `usable` 6.
   They are
   identified by `rewrapped`,
   false on slice 1 and true on slice 2,
   which
   matches the artifact.
- `Weideriche_` persisted one of its two slices.
   `consolidated`,
   `judged`,
   `fresh`,
   gate `usable` 6.
   Both its slices carry `rewrapped` true,
   so
   the envelope does not say which one.

### What must re-buy, and why

- `Acheron` slice 0,
   `slate-kept-standing`,
   REFUSED BY RULE.
   Never cached,
   so nothing was lost.
- `Acheron` slice 3,
   `no-standing-text`,
   RACED AWAY.
   The rule persists it,
   the capture does not hold it,
   and the entry is already discarded from the
   live cache,
   so it is gone for good.
- One `Weideriche_` slice,
   RACED AWAY,
   on the same evidence.

Two entries,
two race losses,
one each.
The poller loses roughly one file per
 entry to `discardSliceCache`.

### The instrument, and its positive control

A resumed slice is SILENT.
`consolidateDocument` reads
 `const settlement = resumed ?? await settleFresh()`,
so a cache hit calls
 neither `produceConsolidations` nor `settleConsolidation` and neither tag can
 appear.
A fresh purchase calls both.

Count `[<entry>] [consolidateDocument] [settleConsolidation]` and the same with
 `[produceConsolidations]`.
Run 1's positive control is 27 and 2 for `Acheron`
 across 4 slices,
19 and 1 for `Weideriche_` across 2.

### The pass criterion

Per slice,
against run 1's artifact:

- `Acheron` slice 1 and slice 2 MUST be byte-identical.
   A difference there
   means the resume path is broken,
   which is what this verification exists to
   decide.
- `Acheron` slice 0 and slice 3 MAY differ,
   and a difference is not a
   failure.
- One `Weideriche_` slice MUST be byte-identical and one MAY differ.
   Which is
   which is not predictable from the capture,
   so either assignment passes.
- Both entries MUST show fewer fresh-purchase lines than run 1,
   since half
   their slices resume.
   Equality with run 1 means nothing resumed.

Keep `Acheron` in run 2 rather than dropping it for a cleaner result.
Slice 0
 is the only place the decline-refusal path can be watched executing on real
 state.

### Scope run 2 to entries that settled

Only entries holding a settled artifact when run 1 ends belong in run 2's
 `--only` line.
An entry killed mid-flight keeps a partial cache in the run's own directory,
which is a snapshot rather than a resume,
and it cannot be predicted from an
 artifact that does not exist.

### Run 1 baseline per entry, captured before run 2 starts

Counted from `~/temp/agent/vub-run1.log` at 05:21Z,
with 4 of 6 entries settled.

- `Acheron`:
   27 `settleConsolidation`,
   2 `produceConsolidations`,
   30 lines tagged `consolidateDocument`.
- `Weideriche_`:
   19,
   1,
   21.
- `Zha_Ke`:
   12,
   1,
   14.
- `gaoyanger`:
   9,
   2,
   12.
- `keyword233`:
   still in flight,
   so it has bought nothing yet.
- `lintong`:
   not started.

The accounting closes exactly.
The four settled entries sum to 77 `consolidateDocument` lines,
which is the total the log carries.
That is the positive control for this count,
and it is why these figures can be trusted as a baseline rather than as a filter
 that happened to match.

Run 2 must show fewer than these on every entry it resumes.
Equality means nothing resumed.
