# Translation repair history, first handover document: overnight of 2026-08-13 and the round three grading steps

The record of the 2026-08-13 session,
in which the alignment fix was refuted and rebuilt and the recurring walls were reduced,
with the steps for grading round three and the ruling that kept the probe in shadow mode.
The same night is also recorded,
with the aligner fix that was never wired in,
in `translation-repair-history-2026-08-12-to-15-overnight-2026-08-13-census-of-unread-signals-and-open-decisions.md`.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Overnight of 2026-08-13: alignment fix rebuilt and the recurring walls

WHAT CHANGED OVERNIGHT 2026-08-13,
kept as the record of that session:

-   `#74` was REFUTED and is now REBUILT.
    The old fix could never have worked:
    `alignHeadings` cannot leave two headings unpaired at all,
    because a
    zero-affinity pairing scores 0 while two gaps cost `2 * GAP_PENALTY`,
    and
    the designed penalty was bounded by exactly that quantity.
    Attempts six
    (lexicographic scoring with an ambiguity path) and seven (the preamble as
    an empty-labelled unit) are prototyped and measured against PRODUCTION:
    90 of 92 entries pair identically,
    `XingZ60` keeps 12 of 13 pairs and loses
    only the wrong one,
    and the single refused entry,
    `XIEPT2`,
    holds 82
    characters of English against 6994 of Chinese.
    It is NOT LANDED,
    because
    with no translate stage `XIEPT2` would get nothing at all,
    and that is the
    destination decision `#70` owns.
-   The dominant cause of alignment fallback is an ASYMMETRIC PREAMBLE,
    not a
    missing section:
    exactly 5 entries corpus-wide,
    and they are 5 of the 7
    that fall back.
    That was the bigger half of the work and it was long
    assumed mechanical.
-   Prerequisite 3 is MISPAIRING,
    not unsupported content,
    but the first
    version of that answer overclaimed and was corrected in place.
-   Option B's cost is 1.56x the editor calls and 3.9x its output,
    not a
    multiplication of the run,
    because the editor already fires on 64% of
    slices.
-   `#72` stands,
    through TWO alarms.
    The first pooled `slice-cache` with
    `artifacts`;
    the second was `Futajuhuacha` supplying 7 of 8 hits.
    The
    monitor now alarms only when a share survives dropping its largest single
    contributor,
    which is the rule covering all three false alarms this
    session.
-   `quote-not-found` now records WHICH quote missed,
    not only that one did
    (`b8c678e0a`),
    which takes down one of the four recurring-wall instances.
    Landed without restarting `pass13`;
    a detached watcher at
    `~/temp/agent/continue-pass13.sh` RESUMES that run when its budget expires,
    carrying its settled entries forward rather than re-doing them.
    Cancel with
    `pkill --full continue-pass13.sh`.

## Steps for the round three grading and the probe shadow-mode ruling

WHEN FIFTEEN SETTLE:
1.    `mise run //package/module/translation-repair:draw-sample -- --final` (moved on 2026-10-18 to
      `//package/translation-repair/probe:draw-sample`).
    It now writes THREE files:
      both sheets and `sample-manifest-<seed>.json`.
    The manifest is not optional and cannot be regenerated later;
    see "The draw recorded nothing about what it drew".
2.    Hand the user `doc/runbook/translation-repair-round-three-grading.md`.
    Detection sheet FIRST and alone,
      then the repair sheet.
3.    `score-agreement` for the precision gate (bar 0.9;
    round one 0.560/0.636/0.680,
      round two 0.740/0.787/0.800).
4.    `score-probe --repair-sheet PATH --manifest PATH` for the probe.

SUPERSEDED 2026-08-07:
TASK 53 IS ANSWERED,
do not re-ask it.
Put to the user
with `AskUserQuestion`,
they chose to keep the probe in SHADOW MODE.
The four
options,
the full ranking,
and the condition that reopens the question live in
`doc/decision/introduced-defect-probe-gating.md`,
which is canonical.
`refutedByHuman` from step 4 is the evidence that reopens it,
and revising that
document is what to do with it.

When scoring the probe,
subtract `refinedJoined` before reading any other
count:
on those positions the probe judged wording the naturalness lane
replaced,
so they compare two different texts.

DO NOT start the recall re-measure (task 51) while a pass is running;
it
contends for the same quota.
