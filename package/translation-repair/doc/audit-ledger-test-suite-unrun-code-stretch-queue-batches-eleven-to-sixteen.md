# Audit ledger: code no unit test runs, stretch queue batches eleven to sixteen

The stretch queue,
batches eleven to sixteen,
each read against a whole-suite census,
from the alignment modules to the pairing modules.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Stretch queue, eleventh batch: alignment modules

The eleventh batch's baseline is that census
(`95ca288cd`,
`census-BF4g7o`):
library source holds 900 stretches over 1,821 lines in 321 files,
with 19 functions never called.
By the first construct
(`t8-triage-bf4g7o.txt`),
the queue is 348 returns,
192 nullish fallbacks,
158 ternaries,
100 throws
and 102 others.
Two clusters tie at 26 stretches:
`align` (5 files,
39 lines)
and `corpus-run/artifact` (8 files,
35 lines).
The tie goes to the most cold lines,
so the eleventh batch takes `align`.

The eleventh batch took the alignment modules against `census-BF4g7o`
in three code commits,
`c5fd0b03e` to `791bc45fe`,
and no entry of its own.
Its census at `791bc45fe`
(`census-5PAw2O`,
1,446 passes,
taken clean)
reads against `census-BF4g7o`:
ran 0,
still cold 1,227,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 5.
The five edited sources are the cluster's,
and each "loaded it and left 0 cold stretches";
`table-cell.ts`,
new since the baseline,
is among the census file's loaded sources
and holds no cold stretch and no function never called.
Library source went from 900 stretches to 874:
per-file counts compared between the two census files give the cluster's 26
(`align-headings-optimal.ts` 10,
`align-headings-forced.ts` 7,
`align-blocks-walk.ts` 5,
`align-headings-grid.ts` 3,
`align-blocks.ts` 1),
all five files now at none,
and no other library file changed its count.
The 6 not loaded are the unmapped `wording-key` chunk,
6 stretches before and after.
Three files of other packages moved,
and no commit since `cce456093` edits either package:
`with-timeout.ts` (`async-time`) went from none to 1,
`create-logger.ts` (`logger`) from 14 to 16
and `error-format.ts` (`logger`) from 3 to 1.
Read by line in the eight census files from `census-qTbkjB` on,
those three files hold the same stretches in each but `census-BF4g7o`,
whose run alone reached the timer callback in `with-timeout.ts` (lines 57 to 59),
the sink verification failure in `create-logger.ts` (240 to 247 and 366 to 373)
and `error-format.ts`'s lines 31 to 58 but for 41,
49,
50 and 52.
So the tenth batch's "ran 3" are the `with-timeout.ts` stretch and the two `create-logger.ts` ones.
Every test at `95ca288cd` is in this suite unchanged
(the batch's test changes are 34 added lines in `align-blocks.unit.test.ts` and the new `table-cell.unit.test.ts`),
so by inference a timeout and a sink failure of that one run reached them,
not code a commit reached,
and they are cold again here.
The reading printed none of that:
it reads only the baseline's stretches,
so the stretches that went cold again,
and a cold stretch in a source new since the baseline,
are counted nowhere (B61,
found with B62).

How the rows closed,
by commit:
`c5fd0b03e` replaced the unreachable fallbacks with narrowings that throw,
most through a new `tableCell` read (`table-cell.ts`)
(`grid` 210,
247 and 249,
`optimal` 111,
219,
297,
365,
387 and 406,
all seven of `forced`,
`walk` 183,
189-190,
201-202 and 208-209),
dropped the optimal scan's checks for a cell no filled table leaves unreached (`optimal` 203 and 345),
called `pairScore` where the scan wrote its body out (`optimal` 351 and 360)
and let each traceback move name its own step (`walk` 313-320);
`40b2cd1ff` cased two blocks with no characters (`align-blocks.ts` 238);
and `791bc45fe` summed both sides' characters through one `charsAcross`,
where `estimateExpansion` had written the same reducer twice.
The duplicate-body guard did not report that copy:
the reducer's body is 42 characters once comments and whitespace go,
under the guard's floor of 80.
Every row but 238 is an index read into a table whose size fixes the index,
and 238 guards input the exported scorer accepts though no parse writes it.
With that guard removed the empty-block case alone failed,
and the guard was restored with no diff.
The batch's control copy,
the align-blocks test with the empty-block case expecting a non-finite score,
failed on that case alone and was deleted.

## Stretch queue, twelfth batch: `corpus-run` artifact modules

The twelfth batch's baseline is that census
(`791bc45fe`,
`census-5PAw2O`):
library source holds 874 stretches over 1,782 lines in 316 files,
with 19 functions never called.
By the first construct
(`t8-triage-5paw2o.txt`),
the queue is 347 returns,
171 nullish fallbacks,
158 ternaries,
99 throws
and 99 others.
`corpus-run/artifact` leads alone,
with 26 stretches in 8 files over 35 lines,
so the twelfth batch takes it.

B61 and B62 were fixed before the twelfth batch began
(`48c580d16` to `4304d167d`),
so the batch reads against a census taken after them:
the whole suite at `4304d167d`
(`census-7D4Vp9`,
1,447 passes,
taken clean)
reads against `census-5PAw2O`:
ran 0,
still cold 1,228,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 0.
No stretch ran,
none went cold
and no source with a stretch in either census changed,
so the queue and the twelfth batch's cluster stand as read from `census-5PAw2O`;
per-file counts compared between the two census files agree in every source.
The reading's unmapped fix (`5af5eeec8` and `ce32e438d`) came after this census
and changes nothing it read,
since both censuses carry the unmapped chunk under the same bundle name.

The twelfth batch took the artifact modules of `corpus-run` against `census-7D4Vp9`
in ten code commits,
`21d3654d5` to `99eed862e`,
and two entries of its own,
B63 and B64.
Its census at `99eed862e`
(`census-aoSuO0`,
1,458 passes,
taken clean)
reads against `census-7D4Vp9`:
ran 2,
still cold 1,164,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 14.
The 2 that ran are `artifact-two-lane-project.ts`'s two stretches,
the one cluster source the batch never edited.
Of the 14 edited sources,
seven are the rest of the cluster,
and each "loaded it and left 0 cold stretches";
the other seven were edited by B63,
B64 and the plural-word helper,
and per-file counts compared between the two census files give each of them the count it had
(`attribution-read.ts` 11,
`editor-standing-read.ts` 13,
`verify-published.ts` 6,
`line-structure-guard.ts` 5,
`page-republish.ts`,
`pool-generation.ts` and `rendering-audit-settled-input.ts` 1 each).
Library source went from 874 stretches over 1,782 lines in 316 files to 848 over 1,747 in 308:
the cluster's 26
(`artifact-eligible.ts` 6,
`artifact-two-lane-rebuild-rows.ts` 5,
`artifact-generation.ts`,
`artifact-placement.ts`,
`artifact-pool-refusal.ts` and `artifact-provenance.ts` 3 each,
`artifact-two-lane-project.ts` 2,
`artifact-two-lane-verify.ts` 1),
all eight files now at none,
and no other library file changed its count.
Entry files keep 30 stretches,
over 754 lines where they covered 778:
`editor-standing-read.ts` holds its 13 over 114 lines where it held them over 138.
Other packages' sources are unchanged at 61 files and 324 stretches.
The 6 not loaded are the unmapped `wording-key` chunk,
now named `wording-key-BponvTmf.mjs` where it was `wording-key-DoHfCZX0.mjs`,
6 stretches under each name.
The bundles no test loaded are still 36,
carrying 38 sources over 13,323 lines where they carried 39 over 13,450:
`runs-layout.ts`,
which `runs-layout.unit.test.ts` (new in the batch) loads,
is no longer among the library sources only those bundles carry.

How the rows closed,
by commit:
`21d3654d5` moved the plural word chosen inline into one `wordForCount` (`count-word.ts`),
with both arms cased there
(`eligible` 214,
222 and 230,
`pool-refusal` 201 and 202);
searching library source for that inline choice found it in eleven modules,
all moved,
with no output meant to change.
`8860dc52c`,
following B64,
reads the entry id back through `entryIdOfArtifact`,
and `assertArtifactProvenance`'s arm for a name without the suffix went with it,
since its callers now pass the `ArtifactFileName` type
(`provenance` 266-267).
`06fe7faae` replaced the fallbacks no input reaches with code that has no such branch,
or with `nonNullishOrThrow` where the invariant is the whole argument:
`eligible` 168 and 505 (every admitted entry is placed;
each generation is paired with its count once),
`generation` 413 and 415 (nano-spawn's own `SubprocessError` and its typed `exitCode`),
`pool-refusal` 319,
`provenance` 103 and 309 (the pool records tip and digest together,
so `assertArtifactProvenance` takes one `expected` record or none,
and the test for a tip checked alone went with the state it described),
`rebuild-rows` 101-102 and 116 (rows and slices read as pairs)
and `verify` 135-137.
`99eed862e` cased the reachable ones:
`eligible` 520-521 (a generation the required commit excludes),
`generation` 211-212 (a file named `.json` alone),
`placement` 88,
92 and 94 (an id recorded as null,
an array,
an object),
`project` 80-82 and 90-91 (a member that is not an object,
one with no kind)
and `rebuild-rows` 46,
50 and 89,
through the first direct test of `carveDivergence`,
which now takes the rows and slices it reads rather than the whole artifact and preparation.
The batch's control copy,
the artifact-placement test with the null-id case expecting `(absent)`,
failed on that case and its describe alone and was deleted.

## Stretch queue, thirteenth batch: lane modules

The thirteenth batch's baseline is that census
(`99eed862e`,
`census-aoSuO0`):
library source holds 848 stretches over 1,747 lines in 308 files,
with 19 functions never called.
By the first construct
(`t8-triage-aosuo0.txt`),
the queue is 339 returns,
167 nullish fallbacks,
146 ternaries,
97 throws
and 99 others.
Two clusters tie at 21 stretches:
`lane` (7 files,
59 lines)
and `corpus-run/insertion` (7 files,
50 lines).
The tie goes to the most cold lines,
so the thirteenth batch takes `lane`.

B65 was fixed before the thirteenth batch began
(`e987110a1` to `6583c32c4`),
so the batch reads against a census taken after it:
the whole suite at `6583c32c4`
(`census-EiesJ9`,
1,464 passes,
taken clean)
reads against `census-aoSuO0`:
ran 0,
still cold 1,194,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 6.
Per-file counts compared between the two census files differ in one library source,
`slice-cache-store.ts`,
from 1 stretch to none
(the ENOTDIR catch B65 removed),
so library source holds 847 stretches over 1,745 lines in 307 files,
with 19 functions never called;
the unmapped `wording-key` chunk is renamed again,
6 stretches under each name.
By the first construct
(`t8-triage-eiesj9.txt`),
the queue is 339 returns,
167 nullish fallbacks,
146 ternaries,
96 throws
and 99 others,
and `lane` still ties `corpus-run/insertion` at 21 stretches with the most cold lines (59 against 50),
so the thirteenth batch takes `lane` against `census-EiesJ9`.

The thirteenth batch took the lane modules against `census-EiesJ9`
in four code commits,
`da74173a1` to `de9ced3f0`,
and one entry of its own,
B66.
Its census at `de9ced3f0`
(`census-lPGpK8`,
1,465 passes,
taken clean)
reads against `census-EiesJ9`:
ran 4,
still cold 1,180,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 5.
The 4 that ran are the two cluster sources the batch never edited,
`lane-contest-eligibility.ts` (3) and `lane-contest-key.ts` (1).
The 5 edited sources are the rest of the cluster,
and each "loaded it and left 0 cold stretches".
Library source went from 847 stretches over 1,745 lines in 307 files to 826 over 1,686 in 300:
the cluster's 21
(`lane-contest-driver.ts` and `lane-slice-sets.ts` 4 each,
`lane-contest-eligibility.ts`,
`lane-contest-stage.ts`,
`lane-slice-coverage-error.ts` and `lane-slice-text.ts` 3 each,
`lane-contest-key.ts` 1),
all seven files now at none,
and per-file counts compared between the two census files change in no other file.
Entry files keep 30 stretches over 754 lines,
and other packages' sources are unchanged at 61 files and 324 stretches.
The 6 not loaded are the unmapped `wording-key` chunk,
renamed again (`wording-key-W_GpcIy_.mjs`),
6 stretches under each name.
The bundles no test loaded are still 36,
carrying 38 sources over 13,322 lines,
as in `census-EiesJ9`.

How the rows closed,
by commit:
`da74173a1` and `65deb5167` (B66) cased the driver's rows for the claims and the note the judges are shown
(`driver` 426-429,
435-436 and 446-447)
and the repeat faults
(`slice-text` 334-340 and 342-348,
`coverage-error` 249 and 251).
`67225cfb4` replaced the fallbacks no input reaches with code that has no such branch:
`driver` 388-390 (`nonNullishOrThrow`:
`projectLanes` is the only producer of `ProjectedLanes`,
and its `compareLanes` builds the comparison from the repair ledger row by row),
`stage` 270 and 313 (a `SeatedBallot` type for the ballots the stage builds,
each from one seat's reply),
`slice-text` 220-225 with `coverage-error` 245 (a decision carries its wording as a string,
so the `typeof` check and its fault kind went)
and `slice-sets` 265,
286,
291 and 314 (each list paired with its indices once,
rather than looked up by position).
`de9ced3f0` cased the reachable ones:
`key` 296-300 (a dispute note keys apart from none,
and from another note),
`eligibility` 182 (a ballot choosing neither lane under eligibility),
367-371 (an original no grammar reads)
and 469-470 (front matter with no fence),
and `stage` 463-468 (a short bench).
By reading `settleVotes`,
which counts only the two lanes' votes,
keeping or dropping a neither ballot at `eligibility` 182 cannot change the choice,
so that case drives the line and cannot tell the two apart.
The same commit rebuilt the driver test's `metadataProjection` from whole rows,
where it had been cast from a partial shape against B63's rule,
and gave `catProjection` its TSDoc back from above `metadataProjection`'s.
The batch's control copy,
the contest key test with the dispute case expecting the same key as no note,
failed on that case alone and was deleted.

## Stretch queue, fourteenth batch: `corpus-run` insertion modules

The fourteenth batch's baseline is that census
(`de9ced3f0`,
`census-lPGpK8`):
library source holds 826 stretches over 1,686 lines in 300 files,
with 19 functions never called.
By the first construct
(`t8-triage-lpgpk8.txt`),
the queue is 333 returns,
162 nullish fallbacks,
141 ternaries,
92 throws
and 98 others.
`corpus-run/insertion` leads alone
(21 stretches,
7 files,
50 lines,
ahead of `prompt` at 20),
so the fourteenth batch takes it.

The fourteenth batch took the insertion modules of `corpus-run` against `census-lPGpK8`
in four code commits,
`1af921d31` to `bc15e9a28`,
and one entry of its own,
B67.
Its census at `bc15e9a28`
(`census-o4RZ7w`,
1,466 passes,
taken clean)
reads against `census-lPGpK8`:
ran 0,
still cold 1,155,
cold since then 5,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 8.
The 8 edited sources are the seven cluster files,
each "loaded it and left 0 cold stretches",
and `mask-container-tags.ts` (B67),
which "left 4 cold stretches",
the 4 it had.
The 5 cold since then are `module-or-throw` code the batch brought in
(`nonemptyOrThrow` 1,
`getSize` in `size.ts` 4),
not loaded before,
so other packages' sources go from 61 files and 324 stretches to 63 and 329.
Library source went from 826 stretches over 1,686 lines in 300 files to 805 over 1,636 in 293,
with 17 functions never called where 19 were
(the two sort comparators the cases now call):
the cluster's 21
(`insertion-carried-shift.ts` 6,
`insertion-carried-decide.ts` 4,
`insertion-container-deficit.ts` 3,
`insertion-carried-anchor.ts`,
`insertion-carried-fold.ts`,
`insertion-container-halves.ts` and `insertion-referenced-definitions.ts` 2 each),
all seven files now at none,
and per-file counts compared between the two census files change in no other library file.
Entry files keep 30 stretches over 754 lines.
The 6 not loaded are the unmapped `wording-key` chunk,
renamed again (`wording-key-C7_uuYqL.mjs`).
The bundles no test loaded are still 36,
carrying 38 sources over 13,322 lines.

How the rows closed,
by commit:
`1af921d31` and `ad58185f7` (B67) cut a container half at the lone tag's own offset,
and the arm for a tag the text search could not find went with the search (`deficit` 184).
`0bae4490f` replaced the fallbacks no input reaches with code that has no such branch,
or with a narrowing that throws where the invariant is the argument.
In the carried fold,
positions were looked up again after the lookup that found them:
`pairedNeighbourToward` now returns each neighbour with its slice narrowed to a paired one,
the fold decision carries the carrier,
the receiver and the carried slice,
and `shiftSlices` takes them
(`decide` 215-218,
`fold` 153,
`shift` 80,
130 and 272-273);
the carried slice is read once with `nonNullishOrThrow`,
and the check that it is the recorded insertion went,
since the rows are built from the insertion slices at those positions
and a fold rewrites only a carrier's source (`decide` 226-229);
a carried passage always names evidence,
since `judgeCoverage` calls one carried only on a majority of full votes,
whose matched regions are the evidence (`decide` 146-149,
`nonemptyOrThrow`);
a typed `find` and `filter` read the placed regions' holders (`decide` 177);
and `abutting` takes the two chunks in either order,
reading the gap from the smaller end to the larger start,
so neither caller orders them by position and overlapping spans count as abutting
(`anchor` 237,
`shift` 139-144).
Outside the fold:
`halves` 117-118 and `definitions` 143 read the slice at an admitted position with `nonNullishOrThrow`,
and `definitions` 224 reads the referrer in the predicate that finds the label.
`bc15e9a28` cased the reachable ones:
`anchor` 198-201 (evidence in an archive paragraph no paired span holds),
`fold` 303 (the chained shape listed bath first,
both folding on the first pass,
the order the case's name already claimed),
`shift` 121 (a new mirrored shape where the carrier follows the passage)
and 124 (no paired slice on the carrier's far side),
`deficit` 172 (an archive half not writing the tag)
and 434-439 (two absent rows out of document order),
and `halves` 99-104 (admitted positions out of order),
through the halves admission's first direct test.
The batch's control copy,
the halves test with the finding naming the later admitted slice,
failed at that assertion alone and was deleted.

## Stretch queue, fifteenth batch: prompt modules

Reading the deficit also found B68:
it counts a container's blocks by blank-line split,
where the rule its prevention section states asks the parse.
B68 comes before the fifteenth batch,
whose baseline is the census taken after its fix.

B68 was fixed before the fifteenth batch began
(`8ec62ce62` and `19eb118ab`),
so the batch reads against the whole suite at `19eb118ab`
(`census-VmhQQH`,
1,467 passes,
taken clean),
which leaves library source at 805 stretches over 1,636 lines in 293 files,
with 17 functions never called,
as `census-o4RZ7w` did.
By the first construct
(`t8-triage-vmhqqh.txt`),
the queue is 321 returns,
162 nullish fallbacks,
134 ternaries,
92 throws
and 96 others.
`prompt` leads alone
(20 stretches,
2 files,
43 lines,
ahead of `pair` at 19),
so the fifteenth batch takes it.

The fifteenth batch took the prompt modules against `census-VmhQQH`
in five code commits,
`2d3b02c45` to `f7838ffb3`,
and one entry of its own,
B69.
Its census at `f7838ffb3`
(`census-OGAQiJ`,
1,468 passes,
taken clean)
reads against `census-VmhQQH`:
ran 0,
still cold 1,144,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 2.
The 2 edited sources are the cluster's two files,
each "loaded it and left 0 cold stretches".
Library source went from 805 stretches over 1,636 lines in 293 files to 785 over 1,593 in 291,
with 17 functions never called as before:
the cluster's 20
(`prompt-payload-store.ts` 16,
`prompt-uniqueness-client.ts` 4),
both files now at none,
and per-file counts compared between the two census files change in no other library file.
Other packages keep 63 files and 329 stretches,
and entry files 30 stretches over 754 lines.
The 6 not loaded are the baseline's unmapped `wording-key` chunk (`wording-key-C7_uuYqL.mjs`).
The bundles no test loaded are still 36,
carrying 38 sources over 13,322 lines.

How the rows closed,
by commit.
Which arm of a one-line stretch was cold is inferred from the stretch's width and the tests that existed,
since the census's bundle is not kept;
the batch's census is what shows each closed.
`2d3b02c45` and `a56cde819` (B69) gave each of the store's refusals a reason
and cased every one:
the read that fails on something other than absence (`store` 123-129,
with the cause arm of 66),
each field of the stored reply (177 to 199,
212,
221 and 223),
the envelope and the format version (277 and 279),
text that is not JSON (287-293,
now a narrow `try` of its own)
and a write the filesystem refuses (316-323).
The arm after the read for a symbol other than the absence sentinel went,
since `readPayloadText` returns no other (`store` 268-270).
`2d6237b3e` typed the prompt digest's input:
`canonicalPromptValue` is only handed the model id and the messages,
whose content is a string or a list of parts that are records of strings,
so its arms for null,
numbers,
booleans and anything else went (`client` 44,
48,
50 and 71-73).
Its keys now sort by UTF-16 code unit rather than `localeCompare`,
which follows the host's locale;
a scratch copy of both serializers over six requests
(text and vision content,
keys written in both orders)
printed "samples 6,
differing 0",
and its control showed the two orders disagree on `a` and `B`,
so digests the payload store already holds still name their records.
`d811fcf2b` gave the batch's new TSDoc blocks the blank-line form their files use,
which the edit tool had dropped.
`f7838ffb3` cased a stored refusal replayed whole (`store` 205)
and asserts every B69 refusal as `tallyErrorText` prints it.
The batch's control copy,
the store test with the format-version case expecting the `reply.text` reason,
failed that case alone and was deleted.

Two findings are left open for work of their own.
Ten more production orderings,
in eight files,
sort with `localeCompare`
(`corpus-run/bench-draw.ts`,
`select-candidate.ts`,
`aggregate-claims.ts` twice,
`corpus-run/coverage-census-baseline.ts`,
`corpus-run/coverage-census-report.ts`,
`corpus-run/coverage-census-print.ts`,
`corpus-run/sentinel-probe.ts`,
and `corpus-run/coverage-tally.ts` twice),
where `critic-attribution.ts` and `repair-stages.ts` already state the code-unit rule;
each is to be read for whether its order decides an outcome,
keys a record or only prints.
And blank lines inside TSDoc blocks are written empty in 570 source files,
with a single space or the indentation in 1,049,
and both ways in 266
(a scratch count over the package's 1,975 source files);
nothing enforces either,
and the edit tool writes them empty,
so one form per file with a source scan is queued.

## Stretch queue, sixteenth batch: pairing modules

By the first construct
(`t8-triage-ogaqij.txt`),
the queue for the sixteenth batch is 316 returns,
162 nullish fallbacks,
132 ternaries,
89 throws
and 86 others.
`pair` leads alone
(19 stretches,
9 files,
39 lines,
ahead of `slice`,
`archive` and `corpus-run/run` at 18),
so the sixteenth batch takes it.

The sixteenth batch took the pairing modules against `census-OGAQiJ`
in two code commits,
`b0924bd4d` and `c2974bea9`,
and no entry of its own.
Its census at `c2974bea9`
(`census-PgyHZ8`,
1,470 passes,
taken clean)
reads against `census-OGAQiJ`:
ran 1,
still cold 1,125,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 8.
The 8 edited sources are the cluster's files other than `pair-blocks-stage.ts`,
each "loaded it and left 0 cold stretches",
and the 1 that ran is that file's stretch,
which the report does not list
(`coverage-census-print.ts` prints only the rows that did not run)
but the per-file counts compared between the two census files place there.
Library source went from 785 stretches over 1,593 lines in 291 files to 766 over 1,554 in 282,
with 16 functions never called where there were 17:
the cluster's 19
(`pair-agreement.ts` 1,
`pair-blocks-read-outcomes.ts` 1,
`pair-blocks-stage.ts` 1,
`pair-blocks-steps.ts` 1,
`pair-blocks-wire.ts` 1,
`pair-definition-order.ts` 3,
`pair-media-adjacency.ts` 4,
`pair-sections-stage.ts` 3,
`pair-sections-steps.ts` 4),
all nine files now at none,
and per-file counts compared between the two census files change in no other library file.
The function no longer listed is the `prefix` callback of `pair-sections-stage.ts`
(baseline line 353),
which the contested-source case now calls.
Other packages keep 63 files and 329 stretches,
and entry files 30 stretches over 754 lines.
The 6 not loaded are the baseline's unmapped `wording-key` chunk (`wording-key-C7_uuYqL.mjs`).
The bundles no test loaded are still 36,
carrying 38 sources over 13,322 lines.

How the rows closed,
by commit.
Which arm of a one-line stretch was cold is inferred from the stretch's width and the tests that existed,
since the census's bundle is not kept;
the batch's census is what shows each closed.
`b0924bd4d` removed the arms no input reaches.
`bestVoted` is only handed the agreed pairs naming a source drawn from those same pairs,
so the top-ranked is read with `nonNullishOrThrow` (`agreement` 209).
The heard filter of `readBlockPairingOutcomes` is now a type guard,
so the loop's second heard check went (`read-outcomes` 110).
Each original's targets are held in a `Set`,
so a pair given twice names its block once,
and the in-loop carried check,
which followed a check that none was carried,
went (`blocks-steps` 166).
`blockAt` reads the block a pair names,
so the second lookup and its absent arm went (`definition-order` 253),
and a definition's opening label is read with `nonNullishOrThrow`
rather than dropping the pair from the relabel unsaid (263):
a block is zoned as a definition only for the parser's `footnoteDefinition` node,
whose text opens with its label.
That reading is measured as well:
a scratch probe (`definition-label-probe.mjs`) parsed every page of the pinned corpus,
the original,
the archive and the traditional-Chinese page of each of the 92 entries,
and read a label off each of their 158 definition blocks
(52,
50 and 56),
none unlabelled;
definitions indented by one and by three spaces,
lazily continued,
of two paragraphs and adjacent read a label too,
and a definition block whose text was shifted by one character read as unlabelled,
so the count could show one.
The media-adjacency reads of a pair's blocks (`media` 153)
and of a run's ends (205)
are range-checked upstream
(`readBlockPairing`,
or cached reads keyed by these blocks' text;
`unclaimedRuns` over `targetBlocks.length`),
so both use `nonNullishOrThrow`;
the owner is read as `[owner, ...others]` (269);
and the only symbol `mediaOwner` returns is the unresolved sentinel,
so the unknown-sentinel throw went (356).
In `pair-sections-steps.ts`,
targets,
previous and next hold one entry per original section (`sections-steps` 289,
290,
299 and 300),
so those fallbacks are `nonNullishOrThrow`,
and `scanNearest`,
which copied its answers so far at every section,
appends in one loop.
Both pairing readers' catches narrow through a helper in the repository's `require<Class>Refusal` form
(`requireSectionPairingRefusal` for `sections-stage` 191,
and `requireBlockPairingRefusal` to match it).
`c2974bea9` cased the reachable ones:
an index past the end on the translation side (`blocks-wire` 392-394),
each side's refusal now checked for its class and exact message;
a short bench on each stage (`blocks-stage` 234-245,
`sections-stage` 305-310):
five seats,
whose quorum is three,
with the router refusing all but the two a stage closes on,
the findings exactly as `shortBenchStageFinding` words them;
a source two voices pair one way and two another (`sections-stage` 353-355),
which keeps no pair and says so with the stage's own prefix;
two definition pairs sharing an original,
given with the later archive block first,
read as not crossing (`definition-order` 153);
and a pair given twice emitting its translation block once.
Both helpers are tested directly.
The short-bench cases share a new fixture,
`refusing-seats-client.test-fixture.ts`.

Three corrections came before `c2974bea9`.
Both short-bench cases were first written on a bench of four with two refused,
and failed at their own premise,
`expect(quorum.short).toBe(true)`:
`rosterQuorumSize` is half the bench rounded up,
so two of four is a quorum and that bench is not short (M86).
The rewritten out-of-range case first asserted each side's message alone,
dropping the class check the original side had,
though `readBlockPairingOutcomes` records only a `BlockPairingError` as an unusable reply
and rethrows any other class;
it was read before the commit and checks both (M85).
The fixture's first form used arrow functions and `async` without `await`,
which lint refuses outside a test file;
it follows `archive-selection.test-fixture.ts` (M87).
The batch's control copy,
the definition-order test with the shared-source case expecting `crossing: true`,
failed that case alone and was deleted.
With the tie-break on the archive side removed from `crosses`,
and with the targets held in a list rather than a `Set`,
each new case failed alone in its file,
and both guards were restored.

Two findings are left open.
`scanNearest` rebuilt a list inside a fold,
the quadratic shape rule ITR in `AGENTS.md` rules out,
and `text-accumulators.unit.test.ts` sees only text grown in a loop,
so the package's folds are to be read for a step that copies its accumulator,
with a source scan for the shape.
And `prepare-block-pairing.ts` keeps a `prefix` callback no test calls
(line 162 at `c2974bea9`),
the block-pairing counterpart of the one this batch cased;
it belongs to its own cluster.

By the first construct
(`t8-triage-pgyhz8.txt`),
the queue for the seventeenth batch is 310 returns,
157 nullish fallbacks,
130 ternaries,
86 throws
and 83 others.
Three clusters tie at 18 stretches:
`slice` (7 files,
112 lines),
`archive` (9 files,
36 lines)
and `corpus-run/run` (2 files,
18 lines).
The tie goes to the most cold lines,
so the seventeenth batch takes `slice`.
