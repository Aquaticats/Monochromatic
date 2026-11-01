# Translation repair history 2026-08-21 to 2026-08-23: the consolidation reaches the document

The session of 2026-08-21:
the consolidation wired into the entry pass,
four defects fixed on the way,
including a settlement that could delete a slice,
the verification run in flight and the rule not to edit source during it,
and four measurements that changed the tasks that asked for them.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-21: the consolidation reaches the document, and four measurements that changed their own tasks

THIS DOCUMENT WAS 208 COMMITS STALE when this section was written.
The whole third-rendering arc,
`#130` through `#168`,
is recorded in
`doc/planning/the-third-rendering.md`,
which is canonical for it.
What follows is the state a taking-over session needs,
not a retelling of those commits.

### The consolidation is wired, and its record can no longer delete a slice

`pass-entry.ts` calls `consolidateDocument` after the lane contest,
under the entry's own signal,
and writes what it settled into the version 2 artifact.
Four defects were fixed on the way and each is worth knowing about:

-   The artifact dropped `settlement.text` entirely,
    so a consolidation that won both rounds could not affect any output.
    It now carries `shipped` as a NAMED ABSENCE,
    `{ kind: 'consolidated', text }` or `{ kind: 'unchanged' }`,
    because one terminal state carries the empty string
    and a consumer writing a bare field per slice would delete every declined-contest slice.
-   The resume rule read `gate.usable > 0` where its own policy says quorum,
    and persisted `slate-kept-standing` regardless of the decision behind it.
-   `contest.` and `pairing.` were never registered as cache namespaces,
    so every repair-lane generation change silently deleted an entry's contest ballots
    and its whole LLM-assisted pairing,
    both bought from the roster.
    Proven end to end on a throwaway directory.
    The list is now derived from one `CLAIM_BY_ROLE` record.
-   A resumed settlement was trusted unread.
    `openConsolidateCache` now validates the terminal,
    the text,
    the floor,
    every verdict and every gate ballot before bytes off disk can become corpus text.

State at this commit:
lint clean,
501 suites,
0 failures.

### What is running, and the one rule that protects it

`#138`'s verification at the user boundary is IN FLIGHT.
`doc/handover/translation-repair-run-continuity.md` carries the operating detail
under "The verification run in flight",
including why a cache poller exists and how to finish or stop it.

The rule that matters here:
DO NOT EDIT `package/module/translation-repair/src/` (moved on 2026-10-18 to `package/translation-repair/*/src/`) UNTIL
RUN 2 HAS FINISHED.
The slice cache keys on a digest taken over the built output,
so any source edit discards every namespace,
and run 2 would re-buy everything and report a validator failure that never happened.
Documentation commits are safe and were used throughout this session for exactly that reason.

### Four measurements that changed the task that asked for them

Each of these was owed by a pending task,
cost no quota,
and moved the answer:

-   `#164` asked which of two renderings is better where the gate's ballots disagree with the panel.
    The question dissolved.
    `unsupported` and `dropped` are lists of rendering NAMES,
    so a ballot naming both cancels,
    and 74 of the 76 asymmetric ballots
    name faulty exactly the rendering they voted against.
    The field restates the vote.
    Closed with nothing built.
-   `#162` asked whether the wrap splits a line on line-structured slices.
    The band pair has ZERO such slices,
    so the first measurement was a null from a probe
    that could not have shown anything.
    Corpus-wide it splits 189 of 211,
    breaking 470 of 1091 existing lines.
-   That produced `#167`:
    none of the three shipped wrap sites is gated on `lineStructured`,
    so the pipeline tells producers to keep verse lines and then breaks 43 percent of them at assembly.
-   `#163`'s band of 2.09 to 4.24 fires on 27 percent of the corpus's own archive slices.
    Refuted.
    The tail starts near 0.8 and 10,
    and block-count disparity is a second,
    unit-free instrument:
    88 percent of slices carry equal block counts.

The shared lesson,
and it recurred three times in one session:
A NULL FROM A PROBE THAT CANNOT SHOW THE DIFFERENCE MEANS NOTHING.
The band pair had no line-structured slice;
a GFP walked the same list it was testing;
`prepared.slices` carries `source` and `target` objects rather than `targetText`,
so a key mismatch reported zero structured slices out of 1259.
Each was caught by running a positive control first.

### Open, in the order they matter

-   `#138` item 4,
    the verification,
    is the only thing blocking that task.
-   `#165` owes a count of `slate-kept-standing` settlements carrying a declining decision.
    The artifact records the TERMINAL and not the decision,
    so this reads off the captured cache rather than the artifacts.
-   `#166` has its consumer split enumerated:
    `readArtifactSubjects` and `damage-region-v2` ask shipped questions and are wrong as they stand;
    `settled-tally` is honest but incomplete.
-   `#167`,
    `#163`,
    `#162` all need a decision and then source changes,
    so all wait for run 2.
-   `#168` and `#169` came out of infrastructure rather than the pipeline:
    a picture ceiling above the gateway's undocumented body cap,
    and a supervisor whose process guard cannot see the pass it launches.
