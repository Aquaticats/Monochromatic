# Lane outcome vocabulary handover: the version 2 reader, its schema guards and the reviews that closed the invariants work

The corrected parser contract,
the frozen comparison rules and the version 2 parser,
the build order and the next-actions list with its landed items,
the closing of the invariants review (#103) and the translate driver guard trap (#95),
and the independent review of both guards.

Part of [Handover: the lane outcome vocabulary and what is mid-flight](two-lane-outcome-vocabulary.md),
which lists every file of this record.

## The parser contract was corrected before the parser was written

A review of the CONTRACT itself,
not of code,
found five clauses wrong.
All five are corrected in the planning doc under
"What the version 2 parser must require,
and what it may tolerate",
each saying what it replaces.
The short version:

-   "Every field is REQUIRED" contradicted the writer,
    which keeps raw lane results LIVE and additive
    on purpose.
    Version 2 needs a FROZEN EVIDENCE CORE naming the few raw fields a reader verifies,
    or the version means whatever today's TypeScript says.
-   Exactness follows SCHEMA OWNERSHIP,
    not nesting,
    which the old wording left to be inferred
    from a lane object holding a strict ledger and a tolerant raw result side by side.
-   The blocked status is NOT recomputable:
    a blocked run and an unblocked one produce the same ledger
    when no slice decided differently.
    The reader checks a compatibility matrix instead.
-   A standalone reader cannot re-prepare the texts,
    because the artifact stores none of the identity's
    inputs.
    Verification TAKES a preparation rather than claiming to build one.
-   Generic dispatch accepts an explicit version 1 rather than refusing a version it can read,
    and `readArtifactChangeSets` refuses version 2 explicitly,
    since it answers with one singular
    change set per artifact and version 2 has two lanes and no singular anything.

## The comparison rules are frozen now, which they were not

The sixth finding was a defect in shipped code,
and it is fixed:
the vocabulary froze the WORDS a comparison row may use,
and the RULES deciding a row
were still the live comparator's,
which the builder called and projected.
A later change to how a verdict is decided would have reinterpreted every artifact on disk
under an unchanged version number.

`artifact-v2-comparison.ts` derives the persisted comparison from version 2 ROWS,
which is what a reader holding only the file can run.
The live comparator keeps refusing ledger pairs that cannot be compared at all.
The builder runs BOTH and refuses a disagreement,
so the day the live rules move,
a corpus pass stops rather than writing artifacts
that quietly mean something new.

Two corrections landed on that module afterwards,
both defects in code shipped the same day:

-   Row equality compared `JSON.stringify` on both sides while its own comment said key order
    did not matter.
    It passed only because both sides came from literals written in one order.
    The reader compares rows parsed OFF DISK,
    where the order is whatever the file has,
    so the exported function was a trap aimed at the parser that has to reuse it.
    Equality is now field by field over all eleven fields and into the unions,
    in `artifact-v2-row-equality.ts`,
    shared by writer and reader so the two cannot disagree
    about what agreement means.
    The key-order case carries a positive control:
    it asserts the reordered row's bytes actually differ,
    so it would fail against the old reading.
-   `compareLanesV2` did not cross-check `sourceText`,
    which the live comparator does refuse on.
    Harmless in the writer,
    where the live comparator runs first,
    but the READER runs only the
    frozen module,
    so a refusal missing there is a refusal the reader does not have.

`assertDerivationsAgree` is the one guard here with no strip demonstration,
and it cannot have one:
both derivations live inside the builder,
so no fixture reaching `buildSettledArtifactV2` from
outside can make them disagree.
Its accept and refuse paths are both covered directly in
`artifact-v2-comparison.unit.test.ts`.
Recorded rather than left unremarked,
the same way the
`CLEANUP` capture's limits were.

## The version 2 parser landed, all eight steps

Written to the corrected contract,
one commit per step,
each with lint,
types and its own tests green
before the next started.
What exists now:

-   `artifact-exact-guard.ts`:
    `requireExactKeys`,
    `requireOneOf`,
    `requireArtifactJsonValue`,
    `requireArtifactJsonRecord` (both refuse `null` at every depth),
    and `requireOpenRecord`,
    which
    accepts it.
    That pair IS the tolerance boundary.
-   `artifact-v2-read-contract.ts`:
    parsed types plus the frozen evidence core.
    Both lane statuses are literal copies rather than imports of the live unions.
-   `artifact-v2-read-vocabulary.ts` and `artifact-v2-read-rows.ts`:
    exact parsers for the three
    unions and the three rows.
    The tolerant row parser refuses a RESERVED field on the wrong member
    (`acceptedText` on an outcome that decided nothing) while tolerating unknown ones.
-   `artifact-v2-read-evidence.ts`:
    both lanes' tolerant evidence parsing,
    in ONE file rather than one
    each as the sketch had it,
    because they differ by three fields and share the index-list reading.
-   `artifact-v2-read-row-relations.ts`,
    `artifact-v2-read-set-relations.ts`,
    `artifact-v2-read-comparison.ts`:
    evidence against ledger BY POSITION,
    slice indices DISTINCT
    within a ledger,
    index sets against the rows that produce them,
    blocked COMPATIBILITY,
    per-row
    `assertWordingCoherent` and `assertDeliveryCoherent`,
    and the recorded comparison against
    `compareLanesV2`.

`assertSlicesDistinct` was added after the eight steps,
from the adversarial list the task carried:
a ledger naming one slice twice passed every other relation,
because all of them join BY POSITION.
Two rows both naming slice 5 agree with their own evidence,
agree with the other lane,
and still
match the prepared slice count,
so the file described a document with one slice reported twice and
another missing and nothing said so.
It checks DISTINCT rather than ascending or contiguous:
the writer renumbers slices by design
(`#100`),
so a reader assuming `0` to `length - 1` would refuse a valid future artifact.
-   `artifact-v2-read.ts`:
    the orchestrator,
    `parseSettledArtifactV2`.
-   `artifact-read.ts`:
    dispatch only,
    returning a generation-discriminated reading;
    version 1 parsing moved to `artifact-v1-read.ts` keeping every exported name.
-   `artifact-v2-corpus-verify.ts`:
    `verifyArtifactV2AgainstPreparation`,
    taking a rebuilt preparation.

Verified at the boundary rather than only in fixtures:
`settleEntry` wrote a real artifact into a
throwaway directory and the parser read it back,
both lanes,
recomputed comparison and all nine raw
repair fields intact (`scratchpad/probe-v2-roundtrip.mjs`).
Two facts fell out of the work and are now pinned by tests:

-   A LEGACY artifact is not version 1 with its version field removed.
    The slice count and the two index sets arrived WITH that field,
    so an artifact carrying them and
    no version is refused rather than read as older.
-   The scripted critic reply in the probes does not satisfy the wire contract,
    so those runs lose
    every voice (`stage-quorum-unmet (critic 0/6)`) and the repair outcomes read `incumbent-fallback`.
    That is `#112` working,
    not over-firing;
    do not read those probe outputs as a clean run.

## Next actions, in order

1.  ~~**The version 2 parser**~~,
    landed;
    see above.
    Original note kept for the shape of the requirement:
    `settleEntry` writes version 2 and `artifact-read.ts` read version 1 only.
    Read the CORRECTED contract in the planning doc rather than re-deriving it,
    and start from the file seams the review proposed:
    generic dispatch stays in `artifact-read.ts` and returns a generation-discriminated reading;
    version 1 parsing moves to its own file;
    version 2 gets a read contract (parsed types plus the frozen evidence core),
    exact parsers for the vocabulary,
    one tolerant parser per raw lane result,
    the relations (ledger against result,
    index sets,
    blocked compatibility,
    recorded comparison against derived),
    a top-level orchestrator,
    and a separate corpus verifier taking a supplied `PreparedDocumentPair`.
    The frozen comparison it needs already exists as `artifact-v2-comparison.ts`,
    with `artifact-v2-row-equality.ts` for the recorded-versus-derived check.
    Do NOT return `SettledArtifactV2`:
    its raw results are typed by live pipeline shapes,
    which is exactly what a historical reader must not depend on.

    BUILD ORDER,
    each step committed with its own tests,
    lint and types green before the next:

    1.  `artifact-exact-guard.ts`:
        `requireExactKeys`,
        the `callConfig` JSON value guard that
        refuses `null` recursively,
        and the open raw-result guard that ACCEPTS it.
        A sibling rather than more of `artifact-guard.ts`,
        which is already at 239 lines.
    2.  `artifact-v2-read-contract.ts`:
        the parsed types and the frozen evidence core.
        Core per `sliceTexts` row is `chunkIndex`,
        `incumbentKind`,
        `incumbentText`,
        `outcome.kind`,
        and `acceptedText` where decided.
        Lane evidence adds the counts and index sets each result already reports:
        repair carries `status`,
        `sliceCount`,
        `shippedChunkIndices`,
        `withdrawnChunkIndices`,
        `findings`;
        translate adds `changedSliceCount`,
        `refusedSliceCount`,
        `withdrawnSliceCount`,
        and its `status` of `complete` or `unfilled`.
    3.  `artifact-v2-read-vocabulary.ts`:
        exact parsers for outcome,
        delivery,
        decision comparison,
        ledger row,
        comparison row.
    4.  One tolerant evidence parser per lane,
        in its own file.
    5.  `artifact-v2-read-relations.ts`:
        evidence against ledger BY POSITION,
        index sets against
        the rows that would produce them,
        blocked compatibility,
        per-row `assertWordingCoherent`
        and `assertDeliveryCoherent`,
        recorded comparison against `compareLanesV2`.
    6.  `artifact-v2-read.ts`:
        top-level exact parse and orchestration.
    7.  Dispatch:
        version 1 parsing out of `artifact-read.ts` into `artifact-v1-read.ts`,
        `artifact-read.ts` left holding the four-case dispatch and returning the
        generation-discriminated reading.
    8.  `artifact-v2-corpus-verify.ts`,
        taking a supplied `PreparedDocumentPair`.
2.  ~~**The mixed-generation trap**~~,
    landed as `pass-schema-guard.ts` (`ed80db5cc`).
    Its premise was WRONG as first written here;
    the corrected account is kept below because it is
    what the guard was built to,
    and because the correction is the reusable part.
    `settledEntryIds` does read FILENAMES only (`pass-settled.ts`),
    so the scheduler
    skips an entry settled by any generation.
    What the note missed is that `corpus-pass.ts:232` already calls `assertResumableGeneration`
    (`pass-generation-guard.ts`) before anything is settled,
    and that guard refuses a directory
    whose artifacts record any pipeline digest but this build's.
    A build that writes version 1 cannot share a digest with one that writes version 2,
    since the digest is over BUILT OUTPUT,
    so the realistic trap is already refused
    with `GenerationDriftError` and a three-way remedy.
    What is genuinely uncovered is narrower:
    `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT=yes` disarms the foreign-digest refusal,
    and its message promises that a rate over the pool stays usable if it names a required commit,
    which is true of build drift and false of SCHEMA drift,
    where half the files cannot answer
    the two-lane questions at all.
    Hand-assembled directories reach the same place without the opt-in.
    The precedent for the fix is in that guard's own comment:
    drift is applied to the foreign-digest check ALONE because unplaceable and legacy artifacts
    are different problems with different remedies "neither of which drift is an opinion about".
    Schema version is a third such problem.
    Re-running skipped entries still costs real money,
    so the guard REFUSES and names the ways
    forward rather than deciding anything.
    Found while checking that guard,
    and FIXED:
    the version 1 constant was named
    `SETTLED_ARTIFACT_SCHEMA_VERSION` and documented as "Schema generation the pass writes today",
    while `pass-entry.ts:249` writes 2.
    It is `ARTIFACT_SCHEMA_VERSION_V1` now,
    beside `ARTIFACT_SCHEMA_VERSION_V2`,
    so the guard importing what the pass writes cannot pick up the wrong one (`262526dcb`).

    What landed:
    `assertResumableSchemaGeneration` and `censusBySchema`,
    called from
    `corpus-pass.ts` immediately after `assertResumableGeneration`,
    refusing any settled artifact
    whose generation is not the one this pass writes.
    Not overridable,
    by the sibling guard's own rule.
    Three answers count as foreign:
    another version,
    no version field at all
    (the 2026-08-15 generation,
    which records a digest and so is neither unplaceable nor legacy),
    and a version this build cannot read.
    Verified at the user boundary rather than only in fixtures:
    the real `corpus-pass -- --plan` against a throwaway runs directory holding one version 1
    artifact refuses with `SchemaGenerationError` naming `schema version 1: 1 settled, CatEntry1`,
    and the same driver against a fresh directory still plans 92 entries.
    Both runs spent zero quota;
    plan mode makes no model call and the refusal precedes scheduling.
3.  ~~The discriminated `unrecorded` reading collapsing back into an absent optional property~~,
    landed as `b8cdd5eba`.
    It was in `artifact-v1-read.ts` rather than in `artifact-read.ts`:
    `ParsedAcceptedIssue.repair` is now the `RecordRepairReading` itself.
    The collapse survives in exactly one place,
    `draw-entry-load.ts`,
    where a grading candidate is
    built for the SAMPLE FILE;
    that file's shape is on disk in draws a human is grading,
    so widening
    it is a persisted-format change rather than a reader change,
    and it is left to whoever decides
    to make it.
    Suite after:
    exit 0,
    348 passing groups.

`buildSettledArtifact` (version 1) has lost its last production caller and stays:
readers still parse version 1 artifacts,
and the corpus directory holds them.

## The invariants review is fully closed (`#103`)

Its three remaining items were re-read rather than trusted,
and two were already built:

-   The translate lane already enforces what hearing nobody means,
    at `translate-document.ts:367`,
    before the record is kept.
-   The resumed branch already refuses an unheard cached record.
    It was untested and its finding was the only one written as a bare sentence while its siblings
    carry a tag,
    so a build that started refusing every cached slice would have read as a run with no
    cache.
    Now `translate-discarded-unheard-slice chunk N; ...`,
    pinned by a driver test shown to
    fail with the branch stripped (`112295ed2`).
-   The `unrecorded` collapse,
    fixed and described in the next-actions item 3 above.

## A GFP trap worth knowing about in the translate driver (`#95` closed)

`#95`'s last open nuance,
the unconditional `settledByKey.set`,
was already fixed in code when
re-read:
the memoization sits inside the persist branch,
so an unheard record is not reused by an
in-run twin.
It had no test,
and the first test written for it was WRONG in a way that passed.

A CALL COUNT DOES NOT SEPARATE THE TWO BEHAVIOURS.
The resume branch refuses an unheard record
whatever put it there,
so an unconditionally memoized twin costs two questions exactly like a
non-memoized one.
Asserting the call count passed with the memoization moved out of the persist
branch,
which is what the GFP run showed and what saved the case from being a guard that guards
nothing.

What differs is whether the second twin ever MEETS a record it has to discard,
so the case asserts
an empty `translate-discarded-unheard-slice` finding list instead (`9d399f763`,
corrected in
`9dde08ed7`).
Two protections stacked this way will hide each other from any instrument that reads only the cost.

## An independent review of both guards, and what it changed

Ran through `sol` on the two guard files plus the two reader relations files,
with the context it
could not see stated in the prompt.
It found one defect in code landed the same night,
and three more worth acting on.
All four are fixed (`7e7738b6d`,
`ae52ca975`):

-   **Distinctness was the wrong invariant.**
    Every relation in the version 2 reader joins BY
    POSITION,
    so a ledger naming slice 1 then slice 0 agrees with its own permuted evidence,
    with the
    other lane under the same permutation,
    and with the slice count,
    while every consumer zipping
    rows against the preparation reads each row against the wrong slice.
    The reasoning that chose
    distinctness (the writer renumbers,
    so do not assume `0` to `length - 1`) is sound and does not
    reach that far:
    renumbering produces GAPS,
    and gaps are still increasing.
    Now strictly
    increasing,
    measured on `prepareDocumentPair` first.
-   **A label is not a shape.**
    The schema guard read `artifactSchemaVersion` and never the body,
    so
    a version 1 artifact relabelled as version 2 passed it,
    was skipped by the scheduler,
    and failed
    only in whichever reader asked it a two-lane question.
    Every artifact declaring the generation
    this pass writes is now parsed with that generation's reader.
-   **The census was keyed by its own prose**,
    so three unversioned generations collapsed into one
    phrase and a corrupt file collapsed with a sound artifact of an unreadable generation,
    which is
    how a refusal could tell an operator to preserve a corrupt file.
-   **The refusal order offered a remedy that does not work**:
    the build guard ran first,
    told the
    operator to set the drift opt-in,
    and the schema guard then refused anyway while the run logged
    a resume that never happened.
    Split into what nothing can override,
    then the shape,
    then the
    overridable build check.
    Verified at the boundary.

NOT ACTED ON,
and recorded so it is not re-derived:
sol also notes there is no directory-wide
snapshot,
so a concurrent writer publishing a version 1 artifact between the census and the
scheduler still produces a mixed directory.
The runs directory already takes an exclusive lock
(`lockRunsDir`),
which covers our own passes;
a foreign writer is out of scope of that lock and
would need one shared with it.
