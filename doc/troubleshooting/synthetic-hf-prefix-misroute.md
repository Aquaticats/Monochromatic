# Synthetic refuses every Charm-Hyper-only model with an `hf:` prefix error

Status:
FIXED on 2026-08-26 in `8b289c3ab`,
with the guards committed first in `e0010019f`;
task `#235`.
Kept because the symptom recurs the moment a run is launched without both keys,
and because three instruments lied while it was being diagnosed.

## The symptom, as an operator sees it

A run finishes.
It exits zero,
prints no refusal line and no fault line,
and settles every slice it was given.
Nothing in the summary says anything is wrong.

Under that,
half the model roster produced nothing at all.

The tell in the log is a round line that reads the same every time:

```text
round: 5/10 heard
```

Five of ten,
on every round,
for the whole run.
A genuine provider wobble moves that number around.
A number that never moves is not a wobble,
it is a class of model failing outright.

## What is actually happening

Synthetic is being asked to serve models it does not host,
and it says so plainly,
once per call:

```text
SyntheticHttpError: provider API returned HTTP 400:
{"error":"Your model name should start with an hf: prefix; for example: ...
```

`SyntheticHttpError` in that output is the class renamed `ProviderHttpError` on 2026-10-19.

Synthetic serves models named with an `hf:` prefix.
Five of the ten seats in the roster are Charm Hyper endpoint labels,
which carry no such prefix,
and Charm Hyper is the only provider that serves them:

```text
qwen3.8-max
minimax-m3
gemma-4-26b-a4b-it
deepseek-v4-pro-0813
deepseek-v4-flash-0731
```

Measured over one four-slice calibration,
those five failed on a hundred percent of their calls,
25 of 25 each,
and the five `hf:` models failed on none of theirs.
The split is exact.
It is not load,
not credits,
and not a rate limit;
it is the wrong provider being asked a question it cannot answer.

## Why the run does not fail, and why that is the worse half

Quorum is five voices of ten,
and it is met on the nose.
`package/translation-repair/stage/src/stage-quorum.ts:167` sets
`quorumNeeded = Math.ceil(modelIds.length / 2)`,
and line 327 tests `voices.length >= quorumNeeded`.
Ten models need five,
and five is exactly what survives,
so every round stands,
every slice settles,
and the command reports success.

That is the arithmetic to check first on any roster change.
A roster of ten tolerates losing one provider's half.
A roster of nine or eleven would not have tolerated it,
and this defect would have announced itself on the first round instead of hiding for a run.

This is the multi-provider resilience from `#199` working as designed.
It was built so a provider going away could not stop a run,
and it does not stop one.
What it was not built to do is notice that the loss is total and permanent
rather than momentary,
and say so.

The result is a run that is completely well-formed and quietly worthless:
a ten-model calibration carried out by five models.
The calibration's closing coverage line did name the five seats
(`WROTE NOTHING AT ALL: ... covers 5 of 10 seats`,
from `producer-silence.ts:222`),
but it carries no counts and no cause,
it is stdout prose under exit 0,
and the pass and the other CLIs print nothing of the kind;
that,
not total silence,
was the gap the fix closed.

## How to check whether a run has this

Since `8b289c3ab`,
read the end of stderr.
Every command prints one `SEAT <model> asked=N usable=N unusable=N threw=N` line per seat,
and a `SEATS DARK: K of N seats asked produced nothing usable this run: ...` line when any seat never answered.
A run with a `SEATS DARK:` line is not a comparison of the roster,
whatever its exit code.
A run launched without both keys no longer reaches a call at all:
it exits 6 with `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY is not set; run under mise so sops injects it`.

For a log written before the fix,
count the distinct values of the heard fraction across the run.
One distinct value,
below the roster size,
for every round,
is this defect.

Attribute per model before concluding.
Anchor the search on the emitter's own template rather than on free text,
and print only the extracted fields:
run log lines carry model output,
and this corpus must not be pasted anywhere.

The two anchors worth grepping are the provider's `hf:` prefix message
and the stream progress marker.
Counting both per model gives the failure rate per seat,
and this defect shows as a clean hundred percent on some seats and zero on the rest.

## The cause, located

`createRunClient` in `package/translation-repair/run/src/corpus-run/run-config.ts`
read the second provider's key and,
when it was empty,
warned once and returned the Synthetic client alone.
That single-provider client then received every roster id,
the five Charm Hyper labels included,
and the Synthetic client had no check that it serves the id it is asked for.
`budget-routing.ts`,
which the first draft of this document pointed at,
never ran:
the routing client was never built.

The key was empty because the run was launched with `node dist/final/node/editor-calibrate.mjs` directly,
not under `mise run`,
which is what decrypts `.env.local.json` into the task's environment.
Both keys were in the worktree's encrypted file the whole time;
an earlier draft of the handover claimed fork worktrees carry no secrets,
and that was wrong (this worktree's file is byte-identical to the main worktree's).
The one `warn` line that named the missing variable was the only trace,
and nobody was grepping for it.

## What the fix did

1.  `createRunClient` requires both keys.
    A missing or empty one is a `RunConfigError`,
    which now extends `StatedRefusalError`,
    so the CLI boundary repeats the variable name and exits 6 with no frames.
    There is no one-provider run to fall back to,
    because half the roster is served only by Charm Hyper.

2.  The Synthetic client refuses any id outside its catalog before the wire,
    on both `chatText` and `chatJson` (`syntheticServes`,
    `SyntheticModelNotServedError`).
    Serving capability is a property of the pair,
    so it is checked before any availability logic.

3.  Every call through the factory's client is counted on `RUN_SEATS` (`seat-tally.ts`),
    and `reportingRefusals` prints the seat report at the end of every command,
    on every exit path.

4.  `createRunClient({ transport })` is a seam for wiring tests:
    a Charm Hyper label reaches `hyper.charm.land/v1/messages` and never `/chat/completions`,
    with Synthetic live and its meter unreadable.

Each of five fix lines was removed in turn,
the package rebuilt,
and its suite failed
(2,
3,
2,
4,
and 2 failing lines),
then the line was restored and the suite passed again (`GFP`).
The negative control,
a bare `node` launch with a fake Synthetic key and no Hyper key,
exited 6 with the variable name,
no frames,
and no `SEAT` lines.

## Three instruments that lied while this was being diagnosed

Kept here because each cost real time,
and each will cost it again.

1.  **Run log stamps are UTC;
    this machine runs EDT.**
    A line stamped `02:11:48` was written at `22:11:48` local.
    Read as local time,
    a live and healthy run looks like it died twenty hours ago.
    Compare against `date -u`.

2.  **Node reports `comm` as `node-MainThread` here,
    not `node`.**
    A `/proc/*/comm` scan matching exactly `node` returns nothing,
    which reads as proof the process is gone.
    Match a `node*` prefix,
    or match on `cmdline`.

3.  **`pgrep -f PATTERN` matches the shell running it.**
    A wait loop whose own command line contains the pattern never exits.
    This has caused three separate incidents in this work,
    one of which ended with a cleanup loop killing its own shell and returning exit 144.
    List the pids first,
    confirm the kind through `/proc/<pid>/comm`,
    then kill by number from a command that does not contain the pattern.
