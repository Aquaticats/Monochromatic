# Translation repair history 2026-08-07 to 2026-08-12: what the draw estimates and how sheets are joined to verdicts

Why round-robin across entries protects the draw,
the probe join that attached the wrong record's verdict,
the smaller draw-integrity fixes,
what the precision figure estimates,
the checks made for the first scoring run,
the check that a re-draw cannot overwrite the draw manifest,
and the digest that binds sheets to an exact draw.

Part of [Translation repair history: 2026-08-07 to 2026-08-12](translation-repair-history-2026-08-07-to-12.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Entry count per band is not what protects the draw; round-robin is

Do not read `POOL band=large entries=5` as the safety property.
The large band's candidates are wildly unequal:
 of its 449 accepted issues,
 198 are Dethelly's and 167 are Futajuhuacha's,
 so two entries hold 81% of the band.

That concentration does not reach the sample.
`selectFromBand` in `package/module/translation-repair/src/sample-draw.ts` (moved on 2026-10-18 to
`package/translation-repair/house-rule/src/sample-draw.ts`)
 groups candidates by entry,
 ranks each entry's issues among themselves,
 and sorts by rank BEFORE entry,
so the draw takes one issue from every entry before any entry's second.
With 16 large-band slots over 5 entries each entry gets about three,
 whichever entry brought 198 candidates and whichever brought 12.
`sample-grading.unit.test.ts:376` pins this:
 a pool where `Heavy` holds five candidates and `Light` holds one,
 drawn to two slots,
 must contain both.

The consequence for judgment:
 when deciding whether the pool is ready for the gate sheet,
 count ENTRIES per band,
 because that is what sets the spread,
and ignore the accepted-issue totals,
 because round-robin has already flattened them.
The earlier worry about 2 large-band entries was still correct,
 but for the right reason:
 two entries meant eight slots each,
 not that they held most of the candidates.

## The probe join attached the wrong record's verdict, and the aggregates never showed it

Found by review,
not by symptom.
A guard written to test a hypothetical collision fired immediately on the live
 run's own artifacts,
 in `Acheron`.

`score-probe` joins a graded sheet position to a probe verdict through the issue
 id:
 position to issue id through the manifest,
 then issue id to reading.
That second map was built from each reading's `regions[].issueIds`.
A region names EVERY issue it serves,
 and the README already says one replacement can serve several accepted issues,
so a shared envelope appears in the readings of every record it served and names
 all of them.
Handing those pairs to the `Map` constructor keeps the LAST one,
 so an issue could resolve to a different record's reading.

This is the ordinary case,
 not a rare hash collision.
It happens whenever an envelope served more than one issue,
 which is the merging behavior the pipeline is built around.

Two things this did NOT affect,
 both verified rather than assumed:

-   The aggregate figures.
    `summarizeProbeTelemetry` deduplicates by `envelopeId` before judging,
     so `regions`,
     `majorityIntroduced`,
     and `minorityIntroduced` never read the broken map.
    Re-running after the fix returned `entries=15 shippedRecords=647
    regions=210 majorityIntroduced=1 minorityIntroduced=18` unchanged,
    so every figure quoted in
     `doc/decision/introduced-defect-probe-gating.md` stands.
-   Any measurement taken so far.
    The join only runs with `--repair-sheet` and `--manifest`,
     which is task #60,
     and #60 has never been run.

The fix takes ownership from the record at parse time,
 where it is exact:
 `readArtifactProbe` now returns `owned`,
 pairing each reading with its own record's issue id,
 and a duplicate id throws rather than overwriting.

The lesson worth keeping is narrower than "check your joins".
The regions list was a plausible-looking key that was never an identity.
Nothing downstream could have detected it,
 because a wrong-but-well-formed reading produces counts that look exactly like
 right ones.
When a join key is derived rather than carried,
 ask what happens when the derivation is many-to-one.

## Three smaller draw-integrity fixes landed alongside

-   Preliminary draws used the GATE seed,
     differing from the final only in file name.
    Since a preliminary is re-run as the pool grows,
     each one previewed the gate sample,
     and choosing when to finalize after seeing them would be selecting the
     sample on its contents.
    Preliminary now draws with a derived seed.
    No contamination occurred:
     this round's preliminary sheets were never read.
-   Final sheets are now created exclusively (`flag: 'wx'`).
    `resolveSheetPath` refuses a path that exists,
     but that check and the write were separate steps,
     and human grades exist nowhere else.
-   The pool report gained `contributing=` and `perEntry=`.
    This immediately corrected a reading:
     the small band shows `entries=5` but `contributing=4`,
     because `ArtsEpiphany` settled `unchanged` and accepts nothing.
    Judge readiness on contributing entries,
     never on the raw entry count.

## What the precision figure estimates: entries, not issues

Worth stating plainly because two reviewers raised it independently and nothing
 recorded it.

The draw allocates slots per BAND,
and within a band round-robins across
 ENTRIES.
So an entry contributing 12 candidates and one contributing 198 receive about
 the same number of slots.
Their per-issue inclusion rates therefore differ by more than an order of
 magnitude,
and the pool itself is lopsided:
 small holds 38 accepted issues,
 medium 160,
 large 449,
while the sample splits roughly 17 / 17 / 16.

The number the gate reads is therefore **entry-balanced precision within each
 size band**.
It is not an unbiased estimate of precision over the accepted-issue population,
 and it was never meant to be.
Two properties are being bought with that:

-   A band's figure describes the band rather than its largest entry.
    Round two reported 0.740 / 0.787 / 0.800 per band,
    which is only meaningful
     if a band's number is not dominated by whichever entry happened to be
     prolific.
-   Rounds stay comparable.
    Round one and round two were drawn this way,
    and changing the estimand now
     would make round three a different measurement wearing the same name.

The figure to quote alongside it is the per-entry composition the POOL lines
 print,
because that is what says whether a band's spread is real.
Do NOT reweight the sample by inclusion probability to recover an
 issue-weighted precision without saying so explicitly:
 it would answer a different question from the one rounds one and two answered.

## Two things verified rather than assumed, for whoever runs #60

**The join runs,
and it resolves every position.**
The ownership fix,
the
identity check,
and their tests were all in place while the command itself had
never been executed.
Exercised against the preliminary pair at zero quota:

```text
AGREEMENT joined=50 probeFlagged=0 refutedByHuman=0 sharedWithHuman=0 flaggedUnscored=0 unflaggedFailures=0
```

`joined=50` is the figure that matters.
The zeros are expected on an ungraded sheet,
 but `joined=0` would also have printed as a clean run,
so check that number first rather than the ones beside it.

**`--repair-sheet` and `--manifest` need ABSOLUTE paths.**
The mise task runs
from the package directory,
not the repo root,
so a path relative to the runs
directory fails with `ENOENT` after the summary has already printed.
Build them
from `$(pwd)` at the worktree root.

**The entry-balanced estimand does cover round one.**
The claim recorded above
rests on rounds one and two sharing the draw,
and task #32 is titled "draw
50-issue uniform sample",
which reads like a contradiction.
It is stale wording
from before the tooling was designed:
`sample-draw.ts` has exactly ONE commit
(`da6d66fa2`,
2026-07-25),
has never been modified since,
and already contained
`selectFromBand`'s round-robin.
Round one's sheet is dated 2026-07-26,
after it.
So both graded rounds were drawn entry-balanced and the comparability argument
holds.

## A re-draw cannot overwrite the draw manifest, and why binding sheets to a draw by digest is still worth building

### Checked rather than assumed, for whoever picks up #63

The suspicion that a re-draw could overwrite the MANIFEST while `wx` protected
 the sheets is FALSE.
`corpus-run/draw-sample.ts` uses one `writeFlag` for all three outputs,
and a
 final draw sets it to `wx`,
so a final draw creates every output exclusively or
 creates none.
The digest binding is still worth building,
for a different reason:
 the sheets print no issue id anywhere,
so a header is the ONLY thing that can
 tie a sheet to the items it was drawn from.

## Task 63 landed: sheets bind to a draw, and the reader stops faking claims

Both halves are built,
tested,
and exercised on the real command path.

### The binding

Sheets are joined to their manifest BY POSITION,
and the only check was that
 both declared the same seed and the same corpus pin.
That check cannot do the job it was asked to do.
The draw is deterministic in its SEED but not in its POOL,
the pool grows with
 every entry that settles,
and so one seed at one corpus commit names a
 different set of items at different times.
Two draws can agree on seed,
on pin,
and on item count while describing
 different issues;
 the join would then mislabel every verdict and error nowhere.

Now all three outputs carry a digest over the ordered item identities
 (`position`,
`entryId`,
`issueId`),
plus the seed and pin,
under a
 `sample-draw/v1` domain prefix.
It is computed ONCE per draw,
from the manifest object both sheets are rendered
 beside,
so the three files cannot disagree about the thing that exists to prove
 they agree.
Canonicalized through `JSON.stringify`,
never a delimiter join:
 an entry id containing the delimiter would otherwise let two different draws
 hash alike,
which is the SYB failure in miniature.

`parseSampleManifest` RECOMPUTES the digest rather than trusting the stored
 string.
A digest never checked against its own contents proves only that two files carry
 the same characters,
so editing the items and leaving the digest alone would
 still match a sheet carrying the stale value.

Positions are now checked against where each item sits.
`requireCount` admits zero and admits any ordering,
while both scorers read
 grades by ARRAY INDEX and take the issue id from the item at that index,
so a
 manifest recording another order described one join while the code performed a
 different one.

### Two defects found on the way, neither in the original task

`score-agreement` had NO manifest check at all,
and looked up pre-grades under
 a fixed default seed while `--sheet` could point anywhere.
An earlier round's graded sheet scored against this round's pre-grades would
 have reported a confident agreement rate between unrelated draws.
It now derives the pre-grade path from the seed the sheet declares,
and
 validates a manifest.

The first version of that check sat AFTER the early return taken when no
 pre-grades exist,
so it never ran.
The unit tests passed the whole time.
What caught it was running the real command against a deliberately mismatched
 pair:
two preliminary draws from pools of 304 and 364 candidates at the same
 seed and pin,
then scoring the first sheet against the second manifest.
It printed the precision line and no refusal.
The check now runs before anything is reported.

### Legacy sheets are scoreable, and say so

A missing digest is a NOTE,
not a refusal.
Round three was drawn before the binding existed,
and a final draw refuses to
 overwrite itself precisely because a sheet may already carry hours of grading,
 so refusing would strand work nothing can reproduce.
Verified:
both scoring commands still run against the real round-three files and
 print the weaker-binding note.
The round-three files were NOT backfilled with a digest.
Writing into a sheet the user may open at any moment buys a retroactively
 trusted association,
which is not what the digest is for.

### The reader half

`ProbeClaimAttribution`,
`TelemetryRegionTally` and `TelemetryProbeReading` name
 what the artifact reader actually returns.
It parses `modelId` and `admissibility` and drops every quote field,
because
 those carry unlicensed corpus text into a summary meant to be pasteable;
 it used to satisfy the full claim type by writing `''` into all five text
 fields,
which is a claim shaped exactly like a complete one.
A caller reading `claim.evidence` could not tell "not parsed" from "quoted
 nothing",
and only the first is ever true,
since the screen cannot admit an
 unanchored claim as corroborated.

Region parsing moved to `artifact-probe-tally.ts`.
That was forced rather than chosen:
the refinement audit had pushed
 `artifact-probe-read.ts` to 326 code lines against a 300 cap,
which the package
 lint reported and the previous session did not re-run after landing it.

### A methodology trap worth carrying forward

`*.unit.test.ts` files import `../dist/final/node/index.mjs`,
the BUILT bundle,
 and `lint:types` does NOT type-check them.
So `mise run //package/module/translation-repair:test:unit` (moved on 2026-10-18 to
`//package/translation-repair/<package>:test:unit`) on its own tests the
 PREVIOUS build,
and a green run right after a source edit means nothing.
Use `buildAndTest`.
Two green runs were collected here before that was noticed,
and neither had
 executed a line of the new code.

### Two follow-ups from the task 63 review, one closed and one recorded

CLOSED:
absence was accepted ASYMMETRICALLY.
A legacy sheet paired with a NEW manifest passed under the weaker check,
as did
 a bound sheet whose manifest carried nothing.
One draw writes all three files in one instant and always computes a digest now,
 so a one-sided pair was assembled from two draws,
which is the case the binding
 exists to refuse.
It throws now,
and `requireSheetSeed` replaced the
 `identity.seed || DEFAULT_SAMPLE_SEED` fallback for the same reason:
 measured first,
every grading and repair sheet in the runs directory carries a
 `Draw seed` header (only the gate verdicts do not),
so the fallback was
 unreachable for real input and only ever a way to place an unplaceable file
 under whichever round is current.

RECORDED,
NOT CLOSED:
the pre-grades file carries no draw identity at all.
It is a bare position-to-verdict map with keys `"0"` to `"49"`,
joined by
 position like everything else here.
Deriving its path from the seed the sheet declares is enough for round three,
 because the one-shot draw guard means exactly one draw ever held that seed.
It is not enough in general,
and the fix is a schema change to a file that
 currently exists once,
on disk,
in the middle of the measurement it feeds.
Do it when #48 and #60 close,
not before.
