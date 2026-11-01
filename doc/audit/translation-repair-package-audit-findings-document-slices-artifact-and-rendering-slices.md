# Package audit: findings of the document, slices, artifact and rendering slices

The findings of the slices that read the corpus and store or render what a run decides:
document,
slices,
artifact and rendering.

Part of [Translation-repair package audit](translation-repair-package-audit.md),
which lists every file of this record.

## document-1, MAJOR, verified: any corpus read failure silently drops the entry from a pass

`src/corpus-run/corpus-pass.ts:392-396` continues past any `CorpusReadError`,
and `src/corpus-source.ts:239-243` wraps every `execFile` rejection as one,
with no kind field:
a non-zero git exit,
a spawn failure,
and a `maxBuffer` overflow all read as the expected missing side.
No line is written on either side,
so the entry is absent from `pending=` and nothing says why;
a resumed pass then ranks bands over a shrunken `settled`.
No corpus-pass test references `CorpusReadError`.
Tracked as `#252`.

FIXED 2026-08-26 in `8af2b2bde`:
`CorpusReadError.kind` is read off git's stderr,
measured against git 2.55
(a path absent at the commit says `does not exist in` and is `missing-object`;
an unreadable clone,
a spawn failure,
an oversized blob or a failed listing is `other`),
`isMissingCorpusObject` is the one guard a corpus walk may step
past,
and all five catchers use it (the pass,
the recall benchmark,
the picture gatherer,
the window trial probe,
the slice census).
The pass's walk lives in `src/corpus-run/pass-eligibility.ts`,
names each incomplete entry by id and absent side,
and the pass prints one `INCOMPLETE <id>: <side> page absent at the pin (...)` line per entry.
Guards:
the corpus-source tests assert the kind on a missing path,
a missing clone and a failed listing;
the
eligibility tests sort a throwaway clone into a pair,
a settled size and a named gap,
name a missing original as
the source side,
and propagate an unreadable clone.
Collapsing the classifier fails 2 and 3 cases;
re-widening the walk's catch fails 2.

## document-2, MAJOR, verified: a transient reader failure is cached as a permanent `unavailable` verdict

`src/document-readings.ts:166-183` persists every paired reading whatever its kind;
`src/image-reading-pair.ts:380-388` records a reader that threw as `reader-failed` and counts it absent,
and `:432-440` turns fewer than two readings into an `unavailable` pair verdict.
On resume `:150-158` serves that verdict from the cache with only `resumed, unavailable` in the log,
until the generation marker retires the cache on a rebuild,
never on the provider's recovery.
That is provider trouble degrading the pipeline durably,
which the owner's standing rule forbids.
Tracked as `#253`.

FIXED 2026-08-26 in `17bcaf46c`:
an `unavailable` pair verdict carries `transient`,
true when any reader produced
nothing for a reason that may not hold tomorrow (`reader-failed`,
`empty-reply`,
`too-short`,
`reads-as-refusal`),
false for a disagreement or a roster that cannot read the picture;
`readDocumentPictures` persists only resumable
verdicts (`isResumableReading`),
and `openPictureReadingCache` refuses a transient record and any record written
before the field,
so the one reader failure that left a picture unread is read once more.
Guards:
a throwing reader marks the verdict transient and unresumable,
a disagreement stays stable,
the document
reader persists nothing under a failing client and one record under a disagreeing one,
and the cache refuses the
transient and the older-shape records.
Flattening the mark fails 3 and 2 cases,
dropping the persist guard fails 2,
re-admitting transient records fails 3.

## document-3, MAJOR, verified: a CRLF original silently disables the line-structure family

`src/line-structure.ts:57-66` splits blocks on `\n\n`,
which a CRLF document never contains,
so every slice counts as one block,
fails `MIN_BLOCKS`,
and `isLineStructured` answers false:
no editor addendum,
no inheritance,
no line-count guard.
`src/markdown-blocks.ts:11-16` already measured the population:
one of 184 files at the pin,
`people/gqt/page.md`,
the source side,
which is the side the predicate reads.
`src/mask-invisible-lines.ts:260` splits on `\n` alone,
so an invisible line on that page keeps its `\r` and is never masked;
`src/quote-normalize.ts:205-212` treats `\r\n` as two breaks,
so every wrapped critic quote there is refused (document-4).
Tracked as `#254`.

FIXED 2026-08-26 in `7a2a21ed8`:
`readCorpusFile` folds every CRLF to LF through `foldCarriageReturns`
(`src/line-endings.ts`,
which counts what it folded),
`isLineStructured` folds again for callers that read text by
other means,
and the invisible-line mask judges each line without its carriage return and blanks around it,
keeping
the return and the length.
Measured on the page at the pin:
141 endings folded,
1 block before the fold and 45 after;
the page is prose,
so the
predicate answers false for the right reason now,
and it carries no invisible-only line.
Document-4 no longer meets corpus text,
since the quote normalizer sees LF;
its two-break reading of a literal CRLF
stands for text read by other means and is recorded here rather than changed,
because a length-preserving
normalizer cannot collapse a two-unit ending to one space without an offset map.
Guards:
the fold counts and shrinks by exactly the count and leaves LF and a lone return alone;
a CRLF page reads
back LF;
CRLF verse is line-structured;
a CRLF invisible line is blanked with its return kept and the region
covering the mark alone.
Undoing the read fold,
the splitter fold,
or the mask's body judgement fails 2 cases each.

## document-5 to document-12, MINOR, verified where cited

The unread-signals doc describes a `mirrored` flag and a proportional fallback that `chunk-document.ts:425-444`
no longer has (`equalShape` now,
and the fallback is deleted);
`entry-filter.ts:58,79` throws operator mistakes as bare `Error`;
`image-reading-past-refusal.ts:76` justifies `REFUSAL_ASK_LIMIT` with the projection its own module note refutes;
`locate-quote.ts:151,171,253` writes up to 60 characters of a critic's quote into a `quote-not-found` finding
(corpus text in logs,
no decision recorded);
`readingAnchors`,
`sharedAnchorCount`,
and `quotedTranscript` have no production caller;
per-character accumulator rebuilds in `footnote-graph.ts:106,190`,
`image-reading-sense.ts:180,187`,
`reading-refusal.ts:156`,
and `protected-atom.ts:259-273` (`RG2`,
linear in practice);
bare `Error` for the footnote overflow refusal and several unreachable states,
one defaulted with `?? ''`;
`image-ocr.ts:261,275` discards both decoder errors with `void error` (`LG2`),
and `MIN_OCR_CHARS` is a literal copy of `MIN_READING_CHARS`.

FIXED 2026-08-26 in `8ac0fff65`,
with the tightened entry-filter guard in `b19bd23b0`.
document-5:
`doc/troubleshooting/translation-repair-unread-signals.md` names `equalShape` and `alignHeadingsForced`
and describes the proportional fallback as deleted.
Prose only.
document-6:
both `--only` refusals are `StatedRefusalError`;
the suite's three THROWS cases assert the class.
Guard:
both throws reverted to bare `Error` fail all three THROWS cases;
restored,
passes.
document-7:
`REFUSAL_ASK_LIMIT` is justified by the `#124` measurement (20 extra calls against 119 first asks,
one
recovery on the third ask);
the refuted projection is gone.
Prose only.
document-8:
`needlePreview` reports the flattened needle's length and Latin token count and never its text,
so a
`quote-not-found` finding carries shape only;
the decision is recorded in the module note.
Guard:
text quoting restored fails five locate-quote cases,
among them `COUNTS a paragraph-length needle rather than
quoting any of it`;
restored,
passes.
document-9:
`readingAnchors`,
`sharedAnchorCount`,
`quotedTranscript` and their constants are deleted with their
describes and barrel exports;
`MIN_READING_CHARS` stays,
exported.
Deletion;
no guard applies.
document-10:
`footnote-graph.ts` collects digit parts into a list joined once and slices the GFM identifier from its
start;
`reading-refusal.ts` builds each word in a list closed by `closeWord()`;
`protected-atom.ts` walks with a
cursor and a run start and slices each atom once.
Each is one linear pass with no accumulator rebuild.
Guards:
the identifier start shifted by one fails two footnote-graph cases;
the closing `closeWord()` dropped fails
two reading-refusal cases (`` ACCEPTS `note` AS ITS OWN WORD `` among them);
the atom slice shortened by one fails four
inspect-paragraph cases;
each restored,
passes.
document-11:
the overflow refusal is `FootnoteOverflowError`,
marked and in the inventory;
`chunk-insertion.ts`
reads the source chunk through `nonNullishOrThrow` instead of defaulting to `''`.
Guard:
the marker removed fails the two inventory cases (`KEEPS exactly the classes the inventory records` among
them);
restored,
passes.
The `nonNullishOrThrow` change has no guard:
the defaulted branch was unreachable.
document-12:
`decodeToPng` logs each decoder's refusal through `refusalText` at debug,
and `image-ocr.ts` imports
`MIN_READING_CHARS` instead of copying it.
Logging;
no guard.

## slices-1, MAJOR, verified: the slice-cache store cannot refuse a silence-settled record

Folded into `#238` as its store side:
`src/corpus-run/slice-cache-store.ts:48-66` and `:423-427` are shape-only guards ending in `Array.isArray(...)`,
no guard reads `heardCriticIds.length` or a `stage-quorum-unmet` finding,
and the artifact reviewer confirmed the owned artifact shape carries no per-stage heard count either,
so the slice caches are the only place that decision lives.
Contrast `lane-contest-driver.ts:56-60`,
whose `worthResuming` persists only quorum-met outcomes.

## slices-2, MAJOR, verified: the contest-ledger readers are blind to the ordinal collision

Folded into `#246` as its reader side:
`ledger-directory.ts:177-185` sorts by file name and asserts that is contest order,
`ledger-read.ts:365` reports the file count as the contest count,
and `ledger-parse.ts:494-498` reads the `at` stamp that no reader compares.

## slices-3, MAJOR, verified at the call shape: the slice census carves with the abandoned pairer

Folded into `#249`:
`slice-census-entry.ts:159-162` aligns with `alignDocumentSections` and `:225-232` subdivides with no `blockPairing`,
so the `CENSUS` lines size a translate lane over slices the pass no longer produces.

FIXED 2026-08-26 in `459482191`:
`censusEntry` takes the settled recipe,
feeds the section pairing to alignment and
each section's block pairing to subdivision,
and labels every row `settled-complete`,
`settled-partial` or
`deterministic`;
the driver reads each entry's recipe from the runs directory and opens with a `CENSUS carve:` line
naming how many rows are which,
legacy artifacts counted apart.
Guards:
a recipe that unpairs the middle section moves the unpaired counts,
and one that declines the middle block
shrinks the sliced translation characters;
dropping either spread fails 2 cases.
Measured:
92 complete pairs,
6 settled-partial rows against the 2026-08-17 runs directory and 86 baseline.

## slices-4 to slices-9, MINOR, verified where cited

Raw `console.log` in `slice-cache-namespace.ts:553-556`;
the proportional merge loop at `slice-pair.ts:360-471` is unreachable while its TSDoc calls it the live fallback;
nine index-and-count error classes across the delivery,
coverage,
comparison,
and assembly modules lack
`messageNamesOnly`,
so the boundary prints only their names;
the repetition containment rule (`assembly-repetition.ts:420-422`,
`assembly-adjacent-repetition.ts:289-294`)
compares space-joined strings with `includes`,
so a phrase inside a longer span across word boundaries is dropped;
`SpanAnchor` names two declarations and the explicit `index.ts:37` export shadows the rendering-audit one;
`draw-entry-load.ts:162-189` throws its three reconcile refusals as bare `Error`,
muting the counts they carry.

FIXED 2026-08-26 in `c2d2473db` (slices-4,
5,
7,
8,
9),
`9f0145fcb` and `95882d3fe` (slices-6).
slices-4:
`discardNamespace` announces through `tagged({ tag: discardNamespace.name, l })`,
same wording.
Guard:
`console.log` restored fails `COUNTS what it removed and NAMES who filled it`,
which asserts the
`[discardNamespace]` tag;
restored,
passes.
slices-5:
the proportional merge loop and `totalRunChars` are deleted,
the target runs with them,
and the TSDoc names
the three live shapes:
aligned when both sides carry blocks,
one slice per source run at the insertion boundary,
one
re-indexed slice otherwise.
Deletion;
`slice-pair` passes unchanged and `lint:types` is the guard.
slices-6:
the nine classes forward no message.
`SliceRecordContradictionError` takes `{ lane, sliceIndex, changed }`;
the other eight take a fault union (`SliceIndexingFault`,
`SliceCoverageFault` with `BlockPlacementFault`,
`DeliveryInvariantFault`,
`DeliveryCoherenceFault`,
`LaneComparisonFault`,
`LaneSliceCoverageFault`,
`AssemblyContractFault`,
`SliceDeliveryFault`) and word it through a sentence function in their own module
(`lane-comparison-fault.ts`,
`assembly-contract-fault.ts`,
`slice-delivery-fault.ts` are new siblings;
the rest sit
beside the class).
All nine are marked and in the inventory,
so the boundary and the `TALLY` line print the slice and
the counts.
Block ids in a coverage fault are the parser's positional `block/N` tokens,
recorded on the type.
The
named slice sets carry a closed `NamedSliceSetLabel` and their clauses live beside the sentence,
so no caller string
reaches a message.
The reviewer's fix shape (add the marker) was not available:
the inventory refuses a marked class
that forwards its message,
`StatedRefusalError` being the sole exception.
`slice-delivery.ts` split into
`slice-delivery-decide.ts` and the fault module,
and the lane exports moved to `lane-barrel.ts`,
both for the
file-length limit.
Each of the nine suites gained a `WORDS its refusal` case pinning the sentence and the marker.
Guards,
part one:
the marker removed from `SliceCoverageError` fails three inventory cases;
a word dropped from
`indexingSentence`,
the two phrases swapped in the record class,
and the separator changed in
`deliveryInvariantSentence` each fail their suite's `WORDS` case.
Part two:
the marker removed from
`SliceDeliveryError` fails three inventory cases;
a word dropped from `coherenceSentence` fails `WORDS` and `REFUSES a
gap that carries wording anyway`;
the clause dropped from `comparisonSentence`,
the wrong clause looked up in
`laneCoverageSentence`,
and the two numbers swapped in `assemblySentence` each fail their suite's `WORDS` case;
all
restored,
all suites pass.
slices-7:
`holdsPhrase` pads both phrases with spaces before the substring test,
in both the document and the
adjacent check.
Guard:
plain `includes` restored fails `KEEPS a phrase that is a character substring of a longer finding across a word
boundary` and `KEEPS wording that is a character substring of longer shared wording across a word boundary`;
restored,
passes.
slices-8:
the rendering-audit type is `RenderingAuditSpanAnchor`;
the root barrel exports one `SpanAnchor`.
Rename;
`lint:types` is the guard.
slices-9:
`DrawReconcileError` (`corpus-run/draw-reconcile.ts`),
marked,
names the entry (deliberately,
as the pass's
`INCOMPLETE` lines do) and the counts,
and carries the stray value's `typeof` name in place of its `JSON.stringify`.
Guard:
the marker removed fails the two inventory cases;
a bare `Error` restored fails `REFUSES as
DrawReconcileError`;
restored,
passes.

## artifact-1 and artifact-2, MAJOR, verified: both comparison checks embed whole rows in their messages

Folded into `#237`:
`artifact-two-lane-read-comparison.ts:150-154` puts `JSON.stringify(row)` (archive English and both lanes' text)
into an `ArtifactParseError`,
a class marked `messageNamesOnly` at `artifact-guard.ts:26`,
so `refusalText` forwards the row verbatim at every CLI that reads a settled artifact;
`artifact-two-lane-comparison.ts:330-335` puts both rows into the unmarked `ArtifactComparisonError`,
which the pass's TALLY and CLEANUP lines print 200 characters of.
The trigger is a stored row disagreeing with the frozen rules,
which is exactly what the guards exist to catch.

FIXED 2026-08-26 in `c19d203c6`:
both checks name the differing fields through `comparisonRowDifferences`
(`comparisonRowsEqual` derives from it) and never a row;
the read test asserts the message names `laneRelation`
and carries no `"repairText"`;
guard fails with `laneRelation` dropped from the checks (2 failing lines per suite).

## artifact-7, MAJOR, verified structurally: the round-three instruments cannot read their own draw

`probe-relabel-artifact.ts:362-383` reads through `repairLaneRecordsOf` (`artifact-repair-lane-records.ts:83-121`),
which calls `parseSettledTwoLaneArtifact` directly and refuses any artifact without a numeric schema version;
the reviewer measured,
with structural output only,
that all 17 round-three entries are unversioned legacy
artifacts that `readSettledArtifact` reads as `kind: 'legacy'`.
Loud,
not silent:
exit 5 at the first item.
Tracked as `#257`,
to be decided with `#247` and `#249`.

FIXED 2026-08-26 in `f199b70cf` and `bab444350`:
`readArtifactRecords` dispatches on the generation through `readSettledArtifact`,
the lane for version 2 and the root for the legacy and version 1 generations,
with two cases pinning the legacy path.
`f199b70cf`'s message claimed both cases passed;
one was failing at commit time
(the legacy parser requires `repairDisposition` and `refined` once a record carries regions,
and the fixture lacked them),
which `bab444350` fixed and says so.
Guard shown to fail with the dispatch forced to the lane (both legacy cases refuse at `artifactSchemaVersion`),
restored,
passed.
Live run of `probe-relabel` against the round-three draw,
2026-08-26,
exit 0:
`RELABEL rebuilt 5 distinct damaged regions`,
`gathered 10 unflagged control regions`,
fifteen regions each under `issues-withheld`,
`issues-rendered`,
and `issues-absent`,
none under the old label,
three checker seats asked 45 and usable 45,
no dark seat.
Summed over the fifteen regions the arms raised `removal` 1 (withheld),
3 (rendered),
1 (absent)
and `corroborated` 0 each;
on real regions carrying six to seventeen prior issues the rendered prompt raised more,
not fewer,
removal claims,
the opposite of the cat fixture's direction,
which is a fact for `#68`'s successor to read,
not a verdict.

## artifact-3 to artifact-6, MINOR, verified where cited

`artifact-two-lane-project.ts:68-70` stringifies a `never` member carrying text fields
(unreachable while the unions stay exhaustive);
the consolidation reader (`artifact-two-lane-read-consolidate.ts:368-375`) refuses only a duplicate index,
never a missing or extra slice,
unlike the contest and index-set readers;
`artifact-placement.ts:243-247,288-291` echo a malformed `id` or digest whole to stdout;
`resolveCommit`,
`tipContains` (shallow-clone and unknown-commit throws),
`keepEligible`,
`parseRegionTally`,
and the three placement stdout lines have no test.
The reviewer also answered `#238`'s question:
the owned artifact shape carries no per-stage heard or configured count and no `stage-quorum-unmet` marker;
those strings travel only inside the tolerant raw lane result.

FIXED 2026-08-26 in `12b2af6c1`.
artifact-3:
`refuseUnknownMember` names the member by its `kind` and sorted field names,
never by value;
a member
that is not a record is named by its `typeof`.
Guard:
`JSON.stringify` of the member appended fails `NAMES THE FIELDS of an unknown member and none of their values`;
restored,
passes.
artifact-4:
`parseConsolidation` takes the artifact's `laneSelection` and holds a settled stage to exactly the
contest's slices in its order (`assertConsolidationCoversContest`,
mirroring `assertContestCoversEligible`);
the
whole-artifact reader hoists the selection and passes it.
Measured first:
all 28 artifacts on this machine carrying
the field (schema versions 2,
3 and 4,
under `~/temp/agent`) name exactly their contest's slices in its order,
one of
them naming none,
so the refusal rejects nothing on disk.
Five cases:
the empty settled stage beside an empty contest,
a missing slice,
an extra slice,
out-of-order records,
and records beside a pending selection.
Guard:
the coverage call removed fails the four refusal cases;
restored,
passes.
artifact-5:
`shapeOf` names a recorded value by type and,
for a string,
length;
the three `POOL` lines print that in
place of the value.
Four cases capture the lines through a chained `console.log` wrapper (the suite's cases run
concurrently,
so a wrapper that restored the real reporter outright cut its siblings out mid-case) and pin both the
shape and the absence of the value.
Guard:
`JSON.stringify` restored on the id line and the unreadable-digest line fails `NAMES A MISMATCHED ID BY SHAPE
ALONE`,
`NAMES AN ABSENT ID` and `NAMES AN UNREADABLE DIGEST BY LENGTH ALONE`;
restored,
passes.
artifact-6:
`resolveCommit`,
`tipContains` and `isShallowRepository` take a `repository` seam defaulting to the
package's own worktree;
`artifact-generation-git.unit.test.ts` builds a three-commit history and a `--depth 2` clone
in `mkdtemp` directories (identity passed per call,
never the pinned corpus clone) and drives the abbreviation,
the
unknown name,
the ancestor,
the same commit,
the clean negative,
the shallow refusal and the unknown-commit refusal;
the exit codes were probed on the same fixture before the cases were written (negative 1,
unknown 128,
shallow flag
true).
`artifact-pool-keep-eligible.unit.test.ts` and `artifact-probe-tally.unit.test.ts` cover `keepEligible` (five
cases) and `parseRegionTally` (seven,
including the attribution-only projection and the tolerant `preExisting`).
`resolveCommit` and `keepEligible` are exported through the generation barrel as `@internal`.
Guards:
the shallow check disabled fails `REFUSES TO ANSWER THE SAME NEGATIVE IN A SHALLOW CLONE`;
`malformedIds`
dropped from the filter fails `CARRIES A MALFORMED ARTIFACT THROUGH` and `KEEPS AND DROPS IN ONE LISTING`;
the
count-versus-claims throw disabled fails `REFUSES A COUNT THAT DISAGREES WITH THE CLAIM LIST` and `HOLDS A DECLARED
preExisting COUNT TO THE CLAIMS TOO`.
The six mutations were applied together on one build;
each suite failed only
its own cases and the whole-artifact and eligibility suites stayed green as controls;
all restored,
all pass.
Whole-suite `buildAndTest` after `12b2af6c1`:
804 PASS,
0 FAIL,
exit 0.
The reviewer's `#238` answer (no per-stage heard or configured count in the owned shape) is acknowledged as recorded;
it asked for no change.

## rendering-1, MAJOR, verified: the provenance verdict is wrong on every roster-paired artifact

`rendering-audit-settled-input.ts:288` re-prepares with `prepareDocumentPair({ sourceText, targetText })`,
no `blockPairings` and no `sectionPairing`,
while production carves through `prepareDocumentPairWithRoster`;
the identity check hashes every slice's placement and refuses on any difference,
so every artifact whose roster pairing moved a slice reads `verification=refused`.
The audited text is taken from the artifact itself,
so every measurement column is right;
only the guard is void,
in the one case it exists for.
Tracked as `#255`.

FIXED 2026-08-26 in `d2c580995` (on the recorded section pairing from `26f346de8`):
`readArtifactSubjects`
re-prepares through `rebuildPreparation`,
so a roster-paired artifact rebuilds to its own identity and reads
`verified`;
the verdict gains `unverifiable`,
carrying the recipe halves the file lacks,
for a mismatch beside a
gap,
while a mismatch under a complete recipe stays `refused`;
the driver prints the missing halves beside the
objection.
Guards:
an artifact built over a crossed section pairing reads `verified` with the bare carve as positive control;
an old-style file over a different pair reads `unverifiable` naming both halves;
the moved-slicing case now carries a
complete recipe and still reads `refused`.
Removing the section half,
the block half,
or the gap rule fails 2 cases each.

## rendering-2, MAJOR by contract, verified: one stdout line prints an arbitrary caught message

Folded into `#237`:
`rendering-audit-settled.ts:292` prints `REFUSED: ${verification.detail}`,
which `rendering-audit-settled-input.ts:204` fills with `caughtValueText(error)` and no marker check;
bounded today to hashes and counts,
but the verifier's `translating()` wrapper launders inner messages
into the marked `ArtifactParseError`,
the `#244` shape.

FIXED 2026-08-26 in `c19d203c6`:
`verifySettled` records `refusalText` as the detail.
No guard of its own:
the path is one call,
and an unmarked inner throw cannot be injected through the public reader.

## rendering-3, MAJOR, verified: `--run` and `--against` written last are read as absent

`rendering-audit-settled-report.ts:190-197` returns `''` for both an absent flag and a valueless one,
so `--run` written last reports the newest run and `--against` written last prints no across-run band;
the sibling args module records the same collapse as a fixed defect for `--cap` and `--only`,
and the report module has no test.
Tracked as `#256`.

FIXED 2026-08-26 in `68062ccf7`:
the report reads both flags through `readReportArguments` in the args module,
which shares `valueAfter` and its refusal with `--cap` and `--only`;
a flag at the end of the line or followed by
another flag is a stated refusal naming the flag (`--run needs a value written after it`,
exit 6,
no frames,
verified
at the CLI boundary).
Guards in the args test:
nothing named gives two empty lists,
both named are read in any order,
`--run` at the end
refuses,
`--against` followed by `--run` refuses;
a lenient reader fails 3 cases.

## rendering-4 to rendering-12, MINOR, verified where cited

`asked=` counts answered rows (`settled-read.ts:420`),
so per-model voice loss is unreadable from the report;
relocation candidates are counted per claim pair,
not per slice pair (`settled-relocation.ts:161-183`);
`auditOne` builds a client per subject (`settled.ts:122`,
the calibrate-5 shape);
four operator refusals thrown as bare `Error`
(`settled.ts:428`,
`settled-input.ts:391-392`,
`settled-report.ts:96,161`);
the row and digest notes say a run file carries no text while `report` persists document spans
(`settled-row.ts:127-150`),
which the `#219` sanitization must treat as corpus-bearing;
a negative `--cap` audits the whole archive (`settled.ts:216`);
model-supplied category and verdict words are stored verbatim in `dropped` (`rendering-audit-screen.ts:288,393`);
`settled.ts` and `settled-report.ts` have no test;
invariant throws are bare `Error`.

FIXED 2026-08-26 in `744890056` (rendering-4,
5,
6,
7,
8,
9,
10, 12) and `50ffffc07` (rendering-11,
and the
regression `744890056` shipped).
rendering-4:
`rateByVoice({ rows, roster })` seats the run's recorded roster first,
so `asked` is the subject count
for every member and `answered` is what arrived;
`VoiceRate` gains `answered`,
the report reads `roster` off the run
file (empty for a run written before it was kept) and `printVoices` prints `asked= answered= lost=`.
Guard:
`asked` collapsed onto `answered` fails `COUNTS A LOST VOICE AGAINST THE ROSTER`;
restored,
passes.
rendering-5:
`distinctSlicePairs` counts (run set,
entry,
omission slice,
addition slice) tuples and the heading
reads `claim pairs=N slice pairs=M`;
the read suite carries the three-by-two case (six claim pairs,
one slice pair).
Guard:
the slice-pair count made to add the claim count fails `COUNTS the candidates in its heading` and `PRINTS
FEWER SLICE PAIRS THAN CLAIM PAIRS`;
restored,
passes.
rendering-6:
`main` builds one client per run,
only once something is bought,
and `auditOne` takes it.
Guard:
the roster emptied at the audit call fails `BUILDS ONE ROW` and `SHOWS THE ROSTER A DECLARED IDENTITY`,
since the scripted client records who was asked;
restored,
passes.
rendering-7:
the four refusals are `StatedRefusalError`.
Guards:
the mixed-layout site reverted fails `REFUSES A DIRECTORY CARRYING BOTH LAYOUTS` (the class is asserted);
the no-rows site reverted fails `REFUSES A FILE CARRYING NO ROWS ARRAY`,
and the built report then exits 5 on a
rowless run where it exits 6 restored;
the newest-run site reverted fails `REFUSES A PROBE THAT HAS NEVER RUN`;
the empty-archive site reverted fails `REFUSES AN EMPTY ARCHIVE` at the built command;
all restored,
pass.
rendering-8:
the row module says `report` carries document spans and model prose,
so a persisted run of this
probe is corpus-bearing and `textIdentity` is the one field built to carry none;
the digest module's note is
scoped to the identity field;
the handover's `#219` section names the run files.
Notes;
no guard.
rendering-9:
`readCap` refuses a cap below zero with the operator's word repeated;
`NO_CAP` stays internal.
Guard:
the refusal loosened fails `REFUSES a cap below zero`;
restored,
passes.
rendering-10:
`boundedWord` keeps the first token of a model-supplied category or verdict,
at most 32 characters,
in both drop reasons.
Guard:
the word repeated whole fails `BOUNDS THE WORD IT REPEATS`,
`BOUNDS A LONG SINGLE TOKEN` and `BOUNDS AN
UNKNOWN VERDICT`;
restored,
passes.
rendering-11:
`rendering-audit-settled.unit.test.ts` covers `capped`,
`eligibleSubjects`,
`printPopulation` and
`auditOne` (scripted client,
identity threading);
`rendering-audit-settled-report.unit.test.ts` covers
`readRunRows`,
`newestRun` and `printAcross` on throwaway runs and runs both built commands at the boundary;
the
corroborate case that looped over a possibly empty result now pins two groups of two,
measured first.
Both `main`s
are exercised only through the built commands.
Guards:
`capped` widened by one fails `BUYS NOTHING at zero` and `BUYS THE PREFIX`;
the roster never read fails
`READS the rows, the archive the run named and the roster it asked`;
the REFUSED line dropped fails `SAYS WHAT A
REFUSED VERIFICATION OBJECTED TO`;
the identity withheld fails `SHOWS THE ROSTER A DECLARED IDENTITY`;
all
restored,
pass.
rendering-12:
`RenderingAuditInvariantError` (`rendering-audit-invariant.ts`) at the six sites,
unmarked and
recorded in the inventory's withheld list.
Guard:
the class name dropped fails `NAMES ITSELF`;
restored,
passes.
THE REGRESSION `744890056` SHIPPED:
exporting the two CLI entry modules through the audit barrel made the bundler
fold each into a shared chunk,
and the built `rendering-audit-settled.mjs` and `rendering-audit-settled-report.mjs`
became re-exports whose `import.meta.main` read false,
so both commands printed nothing and exited 0 for that one
commit.
`50ffffc07` moves the testable halves to `rendering-audit-settled-buy.ts` and
`rendering-audit-settled-runs.ts`,
with the probe name shared from the row module,
and keeps the entries as entries.
Guard:
a marker export from each entry re-added to the barrel fails `REPORTS A NAMED RUN and exits 0` and `REFUSES A
FILE THAT IS NOT A RUN`,
with both built files carrying no `import.meta.main`;
restored,
passes.
Rule for the
package:
nothing an entry module declares may be exported through a barrel.
Method note:
a failing `describe` rejects and stops its suite,
so the guards were proved one describe per suite
file per round;
the first round's two mutations masked six others until they were re-run singly.
