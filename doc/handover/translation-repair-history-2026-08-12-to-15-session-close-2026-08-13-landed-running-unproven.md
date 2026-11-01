# Translation repair history 2026-08-12 to 2026-08-15: the close of the 2026-08-13 session

What landed and was pushed by the end of the 2026-08-13 session,
what was running,
and what was still unproven.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session close: what landed, what is running, what is unproven

### Landed and pushed

-   Judge crosscheck built and then STOPPED on its own measurement.
    `seatJudges`,
    `buildCrosscheckCensus` and `score-crosscheck` are shipped and tested;
    `#31` closed because panel parity bounds the whole measurement at about 3%.
-   Naturalness eligibility widened:
    the filter refused every paragraph
    containing a newline,
    discarding 782 soft-wrapped paragraphs to protect 29
    with real hard breaks.
    Verified after the change,
    120 eligible before
    against 404 now.
-   Verse addendum corrected:
    it asserted the CURRENT TEXT is line-structured
    while the predicate reads the SOURCE,
    and on the one entry it exists for
    those disagree (21 blocks at median 22 against 18 at median 101).
-   `--only Id1,Id2` on the corpus pass,
    so one entry can be run when that entry
    is the evidence.
    Runs into a throwaway `TRANSLATION_REPAIR_RUNS_DIR` by
    instruction,
    so a hand-picked document never enters a pool later draws treat
    as natural accumulation.
-   Every run report now prints `SOURCE <dir>` first.
    `score-crosscheck` read the
    wrong run and printed clean zeros that read as "nothing to report".
-   prefer-readonly findings 33 to 10 across two merges and six named types.
    Issue `#424` filed on the two remaining complaints;
    both were fixed on main
    within the session and the diagnostic now names the producing callable and
    line.

### Running

Two passes:
the main accumulation on the widened lane,
and `Toka_ls` alone in
 `translation-repair-runs-verse` for `#79`.
The acceptance check is written and
 structural only,
at `scratchpad/verse-check.ts`:
line count preserved,
no line
 emptied.
The recorded before-state is 95 corpus lines rendered as 101,
55 lines
 changed,
one emptied outright.

### Unproven, and stated as such

-   That the corrected addendum changes what the editor does.
    The before-state
    sits at tip `95f72e591`,
    several pipeline versions back,
    so a clean result
    means the CURRENT pipeline is sound on that entry and attributes nothing to
    the wording.
-   That the pipeline can supply missing translation on a near-empty section.
    `shi_Yumiaoya` is queued behind the verse run for exactly that,
    and the
    answer is three sections read directly rather than any rate.
-   Nothing ENFORCES the line rule.
    `preservation-check` catches deletion and
    says in its own header that substitution passes,
    and `#79` is substitution,
    so a deterministic guard would catch one fabrication of the three.
