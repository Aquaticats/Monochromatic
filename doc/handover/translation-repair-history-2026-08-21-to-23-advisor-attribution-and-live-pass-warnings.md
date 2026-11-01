# Translation repair history 2026-08-21 to 2026-08-23: advisor attribution corrected and warnings for a live pass

The section on false citations shown to be false when measured from the session transcript,
the block type that made advisor calls invisible to a tool inventory,
the standing rule that replaced the old one,
and three warnings the advisor raised about the boundary verification:
cache replay,
tails that must reproduce,
and builds during a live pass.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-22: advisor attribution corrected from the transcript, and warnings for the boundary verification

### The false-citation section was itself false, measured from the transcript

Earlier revisions of this section claimed advisor guidance in this session was
never obtained,
then corrected that claim four times,
each correction replacing
one attribution with another.
All of it was wrong,
in the same direction,
and the primary record settles it.

Measured 2026-08-22 from the session transcript at
`/home/user/.claude/projects/-var-home-user-Monochromatic/e94557c6-4127-423c-9303-371e578fb92f.jsonl`,
214701 lines,
reading counts and line numbers only because that file carries
corpus passages.
It holds 254 advisor calls.

THE CAUSE IS A CONTENT BLOCK TYPE.
`advisor` is recorded as a
`server_tool_use` block,
never a `tool_use` block.
An enumeration that walks `tool_use` blocks therefore reports zero advisor
calls for a transcript holding 254,
while reporting every other tool
truthfully.
That is exactly the shape of the claim each correction rested on:
"only Bash,
`ToolSearch` and `TaskUpdate` were called".

POSITIVE CONTROL,
and the reason this is cause rather than theory:
the probe
that found it made the same mistake first,
filtering `.type=="tool_use"` and
returning 0 against a file it then reported 254 for under
`.type=="server_tool_use"`.
Same file,
same session,
two filters,
and only the second one can see the call
type in question.
The null was an artefact of the filter,
not a fact about the session.

WHY SUMMARIES KEEP LOSING THEM.
Advisor calls cluster immediately before
compaction.
Across the 105 compaction boundaries that have a preceding call,
the median gap
is 24 lines and 61 percent sit within 25 lines of the cut.
The guidance places a call where work completes,
and compaction cuts where work
completes,
so the call most likely to fall outside a summary is the one whose
findings the next segment is about to act on.

WHAT THE COMMITS ACTUALLY FACED,
counted as advisor calls preceding the line
where each commit first appears:

-   `d79d999c3`,
    the first correction:
    248 before it,
    nearest 198 lines back.
-   `39108d02c`,
    Commit A of `#163`:
    248 before it,
    nearest 915 lines back.
-   `81f2a9173`,
    the second correction:
    249 before it,
    nearest 74 lines back.
-   `2008ccc17`,
    opening Commit B:
    250 before it,
    nearest 316 lines back.
-   `0c19e2461`,
    the third correction:
    251 before it,
    nearest 219 lines back.
-   `8a83da109`:
    253 before it,
    nearest 53 lines back.

So "no advisor call preceded the `#163` guard" and "no advisor call preceded
Commit B either" are both false,
and the sentences they were written to correct
were true when first written.
The four corrections each deleted a true attribution.

THE STANDING RULE,
REPLACING THE ONE THIS SECTION USED TO CARRY.
The old rule
said never to write that an advisor said anything unless that call appears in
the current transcript.
That rule produced the four false corrections,
because a compacted transcript
does not show the call and the summary standing in for it cannot see the block
type.
Replace it with:

-   A summary's tool inventory is not evidence of absence.
    It enumerates
    `tool_use` blocks,
    and `advisor` is not one of those.
-   To settle whether a call happened,
    read the jsonl and count
    `server_tool_use` blocks named `advisor`.
    Counts and line numbers only:
    that file carries corpus passages.
-   Occurrence and wording are separate claims.
    A summary that dropped a call
    also dropped what it said,
    so an attribution written after a compaction can
    be right that the call happened and wrong about its content.
    Check the
    second the hard way,
    by what the surrounding work could not have produced
    alone.
-   Corrections stay recorded rather than amended away,
    because the false
    sentences were pushed and a reader of the history would otherwise carry
    them forward.
    That part of the old rule survives,
    and this section is
    itself an instance of it.

The advisor call at transcript line 213023,
219 lines before `0c19e2461`,
produced three findings about how the `#163` boundary verification could
quietly fail rather than about the note itself:

-   A cache hit at a tail slice would replay a ballot cast BEFORE the note
    existed,
    so the verification would read pre-note judges while crediting the
    outcome to the note.
    `laneContestRunShape` covers `modelIds` and
    `identityContext`,
    never policy text,
    and a repair-unchanged candidate is
    byte-identical to the archive across runs,
    so the collision is plausible
    rather than theoretical.
    RESOLVED:
    the store is `slice-cache/` INSIDE
    `TRANSLATION_REPAIR_RUNS_DIR`,
    so a throwaway runs dir is fresh by
    construction and every ballot this run reads it also wrote.
-   The tails must reproduce before ballots mean anything.
    If the repair lane
    changes `dogesir_#3` this time instead of returning the archive,
    no note
    fires and the run verifies nothing,
    which is a finding to record rather
    than a failed verification.
-   Running a `//package/module/translation-repair` (moved on 2026-10-18 to the packages under
    `package/translation-repair`) mise task that BUILDS,
    while a pass is live,
    can destroy it.
    rolldown names chunks by content
    hash and the runner's stage chunks load on demand,
    so a rebuild can delete
    a chunk the live process has not yet imported.

    NARROWED BY MEASUREMENT AFTERWARDS,
    and this refinement outranks the
    sentence above it,
    which was stated broadly enough to cover every task in
    the package.
    `lint` and `lint:types` are LEAF tasks with no `depends`,
    so both are safe during a live pass,
    and `lint` writes only
    `dist/final/types/tsconfig.tsbuildinfo`,
    never a node chunk.
    Confirmed by
    reading the package `mise.toml` and by `mise tasks deps` against a positive
    control:
    `corpus-pass`,
    which declares `depends = ["build"]`,
    does print it,
    so an empty result for `lint` is a real absence rather than a broken probe.
    `test:unit` declares no `depends` either,
    which is why `buildAndTest`
    exists to sequence them:
    it does not rebuild,
    so running it against a stale
    `dist` passes vacuously rather than testing what was just edited.

    Editing `src/` mid-pass is safe:
    the generation stamp is a digest over
    `dist/final/node`,
    and never reads source.
