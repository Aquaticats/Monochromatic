# Restart corpus accumulation whenever a behaviour fix lands

## Decision

When a fix changes pipeline behaviour,
kill the running corpus accumulation and restart it into a fresh artifacts directory at the new `HEAD`.
Do not let a running pass finish first.

Stated by the repository owner on 2026-08-13:
"Always kill and restart.
We'll actually run out of actionable items at some point."

RESTATED BY THE OWNER ON 2026-08-19,
unprompted and mid-action,
as
"Always stop and restart."
It arrived while a 92-entry pass launched minutes earlier was being killed,
because reading `#107` immediately after launching turned up a measured and specified fix for
damage that ships in the output.
A rule restated a second time under a live instance of the case it governs is a settled rule,
so an agent facing this choice again should act rather than deliberate.

## What this replaces

The alternative under consideration was to freeze a pipeline commit,
accumulate to completion against it,
and only then measure.
That was rejected.

The argument for freezing was that restarting on every fix means accumulation never completes.
The owner's answer is that the fix stream is finite:
the queue of actionable defects drains,
and once it does the accumulation completes on its own.
Freezing optimises for finishing a pass,
which is not the goal.
The goal is a pool that describes the pipeline as it now stands.

## Why a stale pool is worse than a restarted one

A pass stamps `tip`,
the repository commit at pass start,
onto every artifact it writes.
`readHeadSha()` runs once in `corpus-pass.ts` and the value is reused for the whole pass,
so a pass that outlives a fix keeps stamping the pre-fix commit on everything it settles.
Every entry it produces after that point is unusable for any measurement that needs the current pipeline,
and it costs the same budget as a usable one.

Measured on 2026-08-13,
immediately before this decision:
the accumulation directory held 22 settled entries across four recorded tips,
and none of the four contained the chunk or union governance fixes that had landed that evening.
The pool of entries settled under the current pipeline was zero while the directory looked full.

## What makes the failure visible rather than silent

Three pieces,
all in `package/translation-repair/*/src/corpus-run/`:

-   `artifact-generation.ts` partitions settled artifacts by the `tip` each recorded.
    Nothing read that field before;
    six readers globbed the directory and pooled whatever was in it.
-   `artifact-eligible.ts` refuses a pool spanning generations unless a required commit is named
    or mixing is asked for explicitly,
    and refuses an empty pool outright.
    An empty pool is the same failure at its limit:
    a denominator shrunk all the way to nothing while the number above it still renders.
-   `artifact-pool.ts` applies that policy from the environment,
    so the four rate-producing readers cannot each forget it separately.

A scheduler asking which entries already have an artifact must still see all of them,
or the pass re-runs settled work.
`corpus-pass.ts` reads the directory unfiltered for that reason and is correct to.
The filter is for readers that produce a rate.

## Also rejected: salvaging older entries per measurement

The suggestion was that eligibility is a property of the question rather than of the directory,
so a measurement whose code path no intervening fix touched could still pool older entries.

It does not hold here.
Verse governance reaches 49 of 275 chunks across 31 entries,
so a draw can land in a governed slice whatever file the fix touched.
The test would have to be "does this sample intersect the changed behaviour",
not "did the fix touch a different file",
and nothing computes that.
Recorded because the idea is sound in principle and may apply once the fix stream slows.

## What counts as a behaviour fix

Only a change that alters what a pass WRITES.
A change confined to the readers does not,
and restarting for one would burn the accumulation over a typo.

The two families are disjoint and can be checked rather than judged.
`corpus-pass.ts` imports none of `artifact-eligible.ts`,
`artifact-pool.ts` or `artifact-generation.ts`;
those are reached only through `pipeline-barrel.ts` and the four rate-producing readers.
Before restarting,
confirm the changed file is on the path the pass actually runs.

Worked example from the session that set this policy:
the empty-pool guard landed before pass14 started and is in its tip,
while a diagnostic wording fix landed after it started and did not trigger a restart,
because `artifact-eligible.ts` cannot change a single byte any artifact records.

## Operational note

Restart into a NEW directory,
via `TRANSLATION_REPAIR_RUNS_DIR`.
Restarting into the existing one would skip every already-settled entry,
which is precisely the set that needs redoing.

This is now enforced rather than remembered.
Before a pass settles anything,
`assertArtifactsPlaceable` and `assertBuildGenerationResumable` in `pass-generation-guard.ts`
(with the schema check `assertResumableSchemaGeneration` between them)
refuse when the artifacts already present record a different built pipeline from the one this invocation would stamp
(the guard compared commits when this was decided;
it has compared build digests since 2026-08-14).
`TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT=yes` opts out for a deliberately mixed directory.

## The resume trap this closes

Restarting on a fix was only half the problem.
The other half is that a pass stopping at its soft budget is continued by a fresh invocation,
and that invocation reads `HEAD` again.
Any commit in between,
including one this policy says needs no restart,
changes what the resume stamps.
Four such resumes are the entire explanation for the four-tip directory described above.

The soft budget was therefore also raised from 720 minutes to 4320.
Measured from artifact mtimes:
about 27 minutes per entry over a clean stretch,
about 53 averaged across a span including stalls.
At 92 pending entries that is 41 to 81 hours,
so twelve hours settled roughly 13 to 26 and required four to seven resumes to finish.
The budget must not be the thing that fragments the pool.
