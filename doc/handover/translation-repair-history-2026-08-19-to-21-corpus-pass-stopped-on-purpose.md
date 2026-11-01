# Translation repair history 2026-08-19 to 2026-08-21: the end-to-end corpus pass launched, then stopped on purpose

The launch of a full pass at 23:00Z and its stop eight minutes later,
the reasons for stopping,
the standing rule to stop and restart,
what gated the relaunch,
and the shell mechanics learned while killing the runner.

Part of [Translation repair history: 2026-08-19 to 2026-08-21](translation-repair-history-2026-08-19-to-21.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The end-to-end corpus pass was launched, then STOPPED ON PURPOSE, and why

STATE AS OF 2026-08-19 23:08Z:
NO PASS IS RUNNING.
Nothing to monitor,
nothing to resume,
and no restriction on editing package source.
If a recovery session reads only one line
of this section,
that is the line.

It was launched at 23:00Z over 92 entries and killed at 23:08Z with four minutes of work
and ZERO settled artifacts,
so the discarded cost is one partial entry.
The dead directory `~/temp/agent/corpus-pass-full-20260819` holds a stale `pass.lock`,
a generation-keyed slice cache and that partial work.
It is worth nothing;
the relaunch
uses a FRESH directory,
because the rebuilt `dist` stamps a different digest anyway.

### Why it was stopped four minutes in

Reading `#107` immediately after launching turned up a fix that is measured,
bounded,
specified,
and NOT BUILT WHERE IT IS NEEDED,
for damage that is legible in shipped output:
`lintong` ships a farewell offering the same thing twice,
and `saurikissa` slice 7 ships a
sentence severed after its preposition.
The bound is measured over 1260 slices:
80 flagged,
51 contiguous runs,
longest run 3,
and every relocation pair adjacent,
so showing the judge
slices n-1,
n and n+1 provably covers every case in the pool.

WHAT IS AND IS NOT ALREADY BUILT,
established by reading the source rather than the tracker,
which says "not yet built" without the qualifier:

-   the judge's `neighbouringSourceText` and `neighbouringIncumbentText` parameters EXIST,
    and their own TSDoc names `#107` as the reason
-   the TRANSLATE lane is fully wired:
    `translate-document.ts` computes both with
    `neighbouringSource` and `neighbouringIncumbent` and feeds the call AND the slice-cache
    key from one value
-   the REPAIR lane has NO neighbour wiring at all:
    `translate-repair.ts` and the repair
    stages contain no occurrence of `neighbour`

That split is exactly the lane split the damage shows,
which is the confirmation that this
is the right target:
`lintong` counts 2 and 2 in the repair lane against 1 and 1 in the
translate lane,
and the severed sentence at `saurikissa` slice 7 is repair-lane too.
Do not confuse `translate-repair.ts`,
which is the translate lane's conversation with a
model whose candidate failed validation,
with the repair LANE,
whose entry point is
`repairPreparedDocument` in `repair-translation.ts`.

Measuring a pipeline corpus-wide for four days,
when a known defect in it already has a
specified fix,
spends the time to learn the rate of something we were about to remove.
The arithmetic favours stopping and it is not close:

WITHDRAWN 2026-08-25.
The owner removed the release date this table reasons against,
because working under it produced worse results and sometimes slower ones.
The comparison is kept because it records why "stop then run" was chosen,
but the schedule half of it is void and must not inform any later decision.

```text
run then fix   baseline describing a superseded pipeline,
               fix plus verification plus a second pass lands late, no slack
stop then run  fix in hours, 5 flagged entries verify overnight,
               relaunch sooner, full pass done sooner, real slack
```

THE USER RATIFIED THIS AS A STANDING RULE while it was happening,
in three words:
"Always stop and restart."
So this is not a one-off judgement call to re-litigate.
When a
fix ought to land before a long measurement,
kill the measurement and restart it after.
Do not let a multi-day run finish on code that is already superseded.

### What gates the relaunch, and what does not

ONLY `#107` GATES IT.
The fix list does not grow while the pass waits.
`#98` was already measured to have zero instances on this corpus under a validated positive
control,
so it changes no pairing here and is correctly gated on heading scoring instead.
`#90`,
`#91`,
`#94` and `#96` have no measured shipped damage and stay post-pass.

HARD RELAUNCH DEADLINE (WITHDRAWN 2026-08-25,
kept only as a record of what was decided then).
If the window fix has not cleared its gates by then,
relaunch on the current tip and land the fix afterwards.
A perfect pipeline that never gets
measured is worth less than a measured one.

The gates are checkable rather than felt:

-   `lintong` repair lane drops from 2 to 1 for each of the two distinguishing noun phrases
-   `saurikissa` slice 7 no longer ships the severed sentence
-   the contested-cut replacement rate at unflagged slices does not fall away from about 0.95
-   suite,
    lint and types stay green
-   judged-context cost is read,
    not assumed:
    n plus or minus 1 roughly triples the context,
    and `#92` found cost tracks CLAIM COUNT rather than size,
    so confirm claim counts hold

The instruments already exist:
`~/temp/agent/join-107.mjs`,
`~/temp/agent/severed-sentence-census.mjs`,
and the current code reproducing the `lintong`
duplication is the positive control.
Run the five flagged entries with `--only` into a
throwaway `TRANSLATION_REPAIR_RUNS_DIR`.

### Mechanics worth not rediscovering

The runner installs no signal handler,
so a plain TERM ends it and it died in two seconds.
The first TERM appeared to be ignored for a much dumber reason:
bash's BUILTIN `kill` has no
`--signal` long option,
so `kill --signal TERM <pid>` fails with "invalid signal
specification",
and the failure was invisible because stderr had been sent to `/dev/null`.
Use `kill -s TERM <pid>`.
This is the `LF2` case where no long form exists and the short
flag stays.

`pgrep --full 'corpus-pass.mjs'` MATCHES ITS OWN COMMAND LINE and will report a live runner
that is really the pgrep.
Use `pgrep --full '^node .*corpus-pass\.mjs'`.
