# Audit ledger: code no unit test runs, method, first censuses and early batches

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

How the coverage of code no unit test runs was first measured on 2026-09-28 and 2026-09-29,
the rule for when T8 closes and the census task that proves reach,
the first whole-suite censuses,
and the first eight batches of cases written against them,
ending with the page-assembly rounds error that was decided rather than tested.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Function-level coverage census of 2026-09-28

Measured so on 2026-09-28 at `1ea1e350a` (`~/temp/agent/audit-glossary-fix/coverage-exports.mjs`,
result in `coverage-exports.json`):
835 test processes,
7,044 bundle functions,
1,407 exported functions.
Three exported functions never ran:
`DecisionsCardMissingError`,
`UnpositionedContainerError` and `spendCeilingOverrideNote`.
Five exports match no bundle function by name
(`isAsciiLowerLetter`,
`isJsonRecord`,
`seatJudges`,
`seriesFor`,
`textSettingOf`),
so the measure cannot say whether they ran.
The name census's 94 was mostly functions reached through others.
The branch gap is the larger one:
371 exported functions that ran hold 952 blocks no test reached
(175 functions with one such block,
130 with two or three,
57 with four to ten,
9 with more,
led by `repairChunk` at 22),
and blocks inside functions the package does not export are not yet counted.
The 952 was a listed-range sum over exported functions alone;
it and the five unmatched names are superseded by the 2026-09-29 measurement.

## Block coverage remeasurement of 2026-09-29

Remeasured on 2026-09-29 at `cd3f14810`,
with every block rather than exported functions' alone:
the unit suite under `NODE_V8_COVERAGE` against a build carrying source maps
(862 test processes;
1,356 passes counted by marker and no failures,
where a count of lines reads 1,355 because one output line held two tests' lines),
each bundle cut at every range boundary any process reported,
each piece given the count of its innermost enclosing range in each process and summed over processes,
and the pieces no process ran merged and mapped to source lines through the maps.
Summing only the ranges each process lists,
as the 2026-09-28 measurement did,
calls blocks cold that ran:
V8 drops a nested block whose count equals its parent's
(`MergeNestedRanges` in V8's `src/debug/debug-coverage.cc`),
so a block one process lists as cold can have run in another that does not list it.
Of the 2,371 blocks in every function that sum calls cold on this run,
1,061 ran.
That measurement also keyed each function by its start offset alone,
so a function opening its chunk merged with the chunk's top-level script and took its empty name:
those are the five unmatched exports,
and all five ran
(`isAsciiLowerLetter` 35,657,557 times,
`isJsonRecord` 1,286,565,
`seatJudges` 14,
`seriesFor` 9,
`textSettingOf` 3).
Two controls hold the mapping:
the three exports no test calls land on their own declarations
(the constructors at `src/model-card-derive.ts:271` and `src/unwrap-container.ts:162`,
`spendCeilingOverrideNote` at `src/corpus-run/spend-ceiling.ts:132`),
and the spans in `src/repair-chunk.ts` read as whole branches
(ternary arms,
early returns with their log lines),
though a span can start a line late where the minifier folds a log call and a return into one statement.
Neither control could show a span that crosses from one module into the next,
since each reads within one function,
and such spans were recorded under their first source alone (M67),
so the counts of spans,
lines and files in this entry until the M67 rerun leave out every source after the first;
counts of functions never called are unaffected,
a function being mapped by its own first character.

## What no unit test ran on the mapped build, by class

What no unit test ran on that build,
by class,
read through the mapping M67 corrected.
In library source,
1,223 spans over 3,229 lines in 449 files,
holding 72 functions no test calls.
In the six runner entry files tests load,
27 spans over 798 lines,
holding 14 such functions.
The 35 runner bundles no test loads carry 12,928 physical lines in 38 files,
counted apart from the spans;
three of those files are library source only those runners import
(`src/corpus-run/audit-sensitivity-input.ts`,
`src/corpus-run/runs-layout.ts`,
`src/corpus-run/pass-republish.ts`).
Other workspace packages built into the bundles add 233 spans over 3,906 lines in 59 files,
which are theirs to test rather than this entry's.

## When T8 closes, and the `coverage-census` task that proves reach

T8 closes when every library span has a case that runs it and checks what it does (TCV),
a throw on malformed input included
(driven by an input cast past its type,
asserting the error class),
and a span no runtime value can reach is removed as dead code under this ledger;
the runner entry files and the runners no test loads come after the library.
A batch proves its reach by rerunning its own test files under coverage against a mapped build:
every span it claims must read a count above zero there.
The whole census reruns before the launch checklist.
The first 2026-09-29 measurement ran from scratch scripts;
the census is now the package task `mise run coverage-census`
(`f81b58992` to `7253426c8`).
It builds with a source map beside every chunk (`coverage-census:build`),
runs the unit suite or the test files named after `--` with `NODE_V8_COVERAGE` set,
refuses a failing suite,
tallies and maps the cold code as the scratch census did
(until M67),
prints the report,
writes `census.json` under `~/.cache/translation-repair/coverage`,
deletes the raw coverage,
and runs the normal build again.
Given the same coverage and build as the scratch census,
its functions reproduced every total of it
(`t8-port-control.mjs` in the audit's scratch folder),
which proved the two agreed and not that either was right:
both recorded a stretch under its first character's source (M67).
Since `25dc7e9f6` the census splits each stretch where its source changes,
writes census format 2,
refuses a baseline of an earlier format,
and refuses to report where an uncalled function's first line sits in no stretch of its own source.
A batch proves its reach with `--baseline <census.json>` (format 2) and `--source <file>` for each source it claims,
or,
where the batch edits a source it claims,
by a format 2 census of its own test files in which each claimed stretch is gone:
a claimed source loaded with no stretch left,
or,
where the batch claims only some of a source's stretches,
each claimed line outside every stretch left.

## First whole-suite census at `e22373347` and the first two batches

The census's first whole-suite run refused,
the suite failing on the census's own commits (M59 again,
M63),
and a later one showed stretches in the census's own modules (M66).
The whole-suite census at `e22373347` (1,381 passes,
`census-7rB6TM`) was the baseline the batches were read against until M67:
library source holds 1,232 stretches over 3,239 lines in 453 files,
with 72 functions never called;
the six runner entry files tests load hold 27 stretches over 798 lines,
with 14 never called;
the 36 bundles no test loads carry 13,389 physical lines in 39 sources,
three of them library source only those bundles carry;
other workspace packages hold 228 stretches over 3,898 lines in 59 files.
Its stretch,
line and file counts leave out the sources after the first in every stretch crossing modules,
and a format 2 baseline replaces it;
its lists of functions never called and bundles never loaded stand.
The first batch,
`7253426c8`,
closed the census's own stretches,
read through a census of its three test files that showed `coverage-tally.ts`,
`coverage-lines.ts`,
`coverage-file.ts` and `build-entries.ts` loaded with no stretch left;
that reading went through the M67 mapping,
and the format 2 baseline repeats it.
The second batch,
`f2c96d7dd`,
gave cases to the refusals of `decisionsCardOf`,
`flattenContainers`,
`footnoteIdentifiers` and `clusterReadings`,
and removed two throws no input reaches (`AtomFloorError`,
`AnchorRegionError`) and one v8-ignored guard.

## Format 2 whole-suite census at `77de8e685`

The whole-suite census at `77de8e685` (1,386 passes,
`census-qilSwP`,
format 2) is the baseline from M67 on:
library source holds 1,207 stretches over 3,473 lines in 445 files,
with 63 functions never called;
the six runner entry files tests load hold 27 stretches over 799 lines,
with 14 never called;
the 36 bundles no test loads carry 13,418 physical lines in 39 sources;
other workspace packages hold 230 stretches over 4,278 lines in 61 files.
In it the first batch's four sources,
and every census module,
are loaded with no stretch.
The second batch's sources are loaded with no stretch but two:
`src/model-card-derive.ts` keeps lines 374 and 429 to 435,
table checks of its own file batch rather than the refusal the batch claimed,
and `src/unwrap-container.ts` kept three,
two of them the refused container's fragment arms
(a fragment's name,
which the MDX extension writes as null,
and `<>` in the message)
and one a container whose children are not a list.
Cases for all three followed,
and a census of their test file against this baseline reads the three as ran and `src/model-card-derive.ts`,
the untouched control,
as still cold.
A stretch's line range runs from the lowest line its characters map to to the highest,
so where the minifier moves an early return behind the code after it,
the range can take in lines that ran:
the bundle writes that non-list return as `isNodeList(children)?…:!1`,
and the separator before `!1` maps to the last line of the arm before it,
so the stretch read 278 to 284 where only 278 was cold.
This entry then said the error was only ever toward calling code cold,
an inference no build without the minifier had tested,
and it was wrong (M79):
compression also folds guards into expressions V8 gives no range,
so a guard no test reaches read as run.
The census has measured an unminified build since.

## Third batch against the format 2 baseline: uncalled named functions

The third batch gave cases to named functions no test called,
each an exchange or helper production reaches:
the prefixing logger's `error`,
`fatal`,
`flush` and `trace`;
the culled-seat guard's `chatJson` and `decide`,
refused for a culled model and passed through otherwise
(the guard is now exported for its test);
the router's `decide`,
refused with no decisions client and while OpenRouter reads dry,
and marking the meter on a budget refusal;
the seat tally's `decide`;
the Anthropic scanner's `servedBy`,
which the drain asks on every Hyper stream;
the pacer's default sleeper,
driven on the real clock through a wait and an abort;
and `sealedEnd`,
which only the scorer walk reaches,
with an original left behind the last seal and nothing after it.
`sealedEnd`'s hand-written unreachable throw became `nonNullishOrThrow`,
and the provider barrel's `//endregion` moved to its end,
past the two export blocks that had landed after it.
A census of the batch's test files against the format 2 baseline reads every claimed stretch as ran,
and `src/group-merge.ts`,
edited,
holds neither `sealedEnd` nor the seal branch calling it in a fresh census of those files.
The census reports raw truth and honours no v8 ignore hint:
a guard a hint hid is restructured in its file's batch,
as `flattenContainers`' was.

## Fourth batch against the format 2 baseline: five `corpus-run` files

The fourth batch,
`be6490163`,
gave cases to five `corpus-run` files:
`spendCeilingOverrideNote`,
and `resolveSpendCeilingUsd` reading its dial from the environment;
the two-lane artifact reader's sealed archive-original spans,
and its refusals of an empty span list,
a span ending before its start,
a digest that is not one,
and a front matter authority other than the archive's;
`loadEntry`'s happy path,
through a corpus reader it now takes as a parameter,
on an invented page rather than the clone;
and `widthControlHolds` over a panel backing the intact passage and one backing the passage missing a sentence.
The `not-an-object` reconcile fault,
which no parsed artifact could raise,
was removed.
A census of its test files against the format 2 baseline reads every claimed stretch as ran,
`src/model-card-derive.ts`,
untouched,
as still cold,
and the edited `draw-entry-load.ts` and `draw-reconcile.ts` as holding no stretch.

## Fifth batch against the format 2 baseline: the run's outside readers

The fifth batch began with M68,
and then the run's outside readers,
which only `pass-entry.unit.test.ts` had reached,
and only through the live key,
caches and corpus.
`outsideReadsFrom` builds them over an environment,
a transport,
a corpus pin with its lister and reader,
and a clock that is the wall clock unless one is handed in;
`RUN_OUTSIDE_READS` is its call over the process's own.
Its cases buy a work title and a linked page through a stub transport into a throwaway cache,
stamped by the wall clock and then by a handed clock,
search and read nothing without a key,
and read the names at the pin handed over.
A fresh format 2 census of `pass-outside-reads.unit.test.ts` (`census-BsBdqy`) loads `src/corpus-run/pass-outside-reads.ts` with no stretch left
(`9a03a4852`).
Two mutants were each caught by one case alone:
readers that stamp every record by the wall clock,
a handed clock ignored,
failed the fixed-clock stamps;
names read at the run's pin in place of the one handed over failed the pins the lister and reader saw.

## Sixth batch against the format 2 baseline: title cache, relabel probe reader and provider gate

The sixth batch gave cases to three more `corpus-run` files.
`openPageTitleCache` now round-trips a stored round through a throwaway directory,
and its guard refuses a stored value that is not a round,
or holds a title or finding of the wrong shape,
one field at a time.
The relabel probe's artifact reader renders a recorded tally under its envelope
(a count never written as 0,
a count of another type as itself),
reads a probe block written as null or holding no regions as no tallies,
and refuses a tally with no envelope,
an issue severity and a claim category outside the taxonomy,
each by its path;
the test file's `PROBER` constant had claimed a recorded tally no fixture carried,
and now says it is the roster.
The provider gate for measured arms took an optional transport,
defaulting to the live one,
and read its keys and the Bedrock ledger's place from `process.env`;
its tests set keys by writing `process.env`.
`assertRequiredProvidersReady` now requires `env` and `transport` (the pass hands over `process.env` and `fetchTransport`),
and its tests hand over their own,
the Bedrock case a ledger in a throwaway directory;
new cases cover an absent flag,
a flag with no value,
an empty value or an unknown provider,
nothing required,
a meter the transport cannot read,
and Bedrock wet and dry.
The gate's catch rethrew a `RequiredProviderError` nothing under a meter read raises,
and dropped every other failure without a word;
the rethrow is gone,
and the failure is logged before the arm is refused as `meter unavailable`.
A census of the three test files against the format 2 baseline (`census-5MbbbV`) reads the six stretches of the two unedited sources as ran,
and loads the edited `required-providers.ts` with no stretch left
(`4138c5ade`).

## Seventh batch against the format 2 baseline: `run-config.ts`

The seventh batch closed `run-config.ts` with X23.
Its baseline stretches were the checker-independence call's fallbacks for the run's refiners and self-certification,
which `RUN_MODELS` always sets
(it is now typed `RunRepairModels`,
the contract with both required,
and the fallbacks are gone);
the stand-in caller for an absent provider,
removed as unreachable;
the exhausted meter answered when the first provider has no key;
the Bedrock and OpenRouter arms and the decisions client no case had built without the suite's keys;
and the payload store.
New cases build on Hyper alone and read the exhausted meter,
build on all four stand-in keys and read Bedrock wet under its credit and dry at none,
refuse a seat whose only provider has no key without asking anyone,
and replay a prompt from a payload directory in a second client without asking.
A fresh format 2 census of `run-config.unit.test.ts` and `provider-router.unit.test.ts` (`census-IHg9So`) loads `run-config.ts` and `run-providers.ts` with no stretch left;
`provider-router.ts` keeps the baseline's lines 506 to 509 (now 542 to 545),
left to the batch of anonymous callbacks and library stretches,
beside lines the router's other test files run.

## Eighth batch against the format 2 baseline: `ordinal-style.ts`

The eighth batch closed `ordinal-style.ts`,
whose fifteen baseline stretches the series pass's fixtures had left:
the arabic and roman forms read and rendered,
a prefix with no number word,
the "No." leader,
digits past the word tables,
the no-number style rendering nothing,
and the Han numeral's refusals of a second ten,
a second digit in one place and a character that is no numeral.
It had no test file of its own;
`ordinal-style.unit.test.ts` drives each directly,
and reads back every style it renders for every number a series carries.
Writing it showed the roman reading took any word of roman letters as a numeral,
so a capitalised "DID" or a malformed "IIII" before a colon read as a number
that a series re-rendered in another style would have replaced;
it now takes only the numeral `roman` writes for a number from one to ninety-nine.
No stored page carries either
(`t8-roman-headings.mjs` in the audit's scratch folder read 894 headings on 23 published pages and 251 on 92 archive pages,
and its positive control file found one of each kind).
A fresh format 2 census of the new test file (`census-c3xfXm`) loads the edited `ordinal-style.ts` with no stretch left
(`08eb4cba0`).
The mutant restoring the letters-only reading failed the roman refusal case alone,
at "Mittens DID" read as roman.
With it the corpus-run named functions no test called are done.

## Page-assembly rounds error decided rather than tested

`PageAssemblyRoundError` (the baseline's `page-assembly-rounds.ts` lines 37 and 46 to 59,
and the throw at 308 to 311) was decided rather than tested.
The page-assembly rounds threw it when a round took back a slice an earlier round had taken back.
A round's lane rows exclude every slice taken back before,
so only a pass-made row can bring one back,
and two passes do write rows on every slice the page carries
(`canadianizePage`,
`correctPinyinPage`);
the guard takes back every row when a structural break has no single culprit,
so such a row can be taken back again.
No fixture or stored run is known to reach it.
Were one to,
the throw stopped the entry's page,
against the owner's rule that a run always ships,
and without it the rounds would make and lose the same row forever.
The rounds now settle on a round that takes back nothing new:
each further round must add a slice to those taken back,
so they still end,
and the page they settle on is consistent,
since the slice stays withdrawn and ships the archive's text
and `guardPageAssembly` writes no pass row the settling round's guard took back.
A quality call under the standing directive,
recorded for the owner to veto.
`freshlyTakenBack` carries the rule and `page-assembly-rounds.unit.test.ts` drives it:
a round's new take-backs each once,
none for a round that took back only what earlier rounds had,
and none for a round that took back nothing.
A fresh format 2 census of that file and `page-assembly-guard.unit.test.ts` (`census-gBeTc1`) loads `page-assembly-rounds.ts` with no stretch left;
`page-assembly-guard.ts` keeps its baseline lines 114 to 116,
`namesIt`,
which needs a pass-restored row and a guard-trimmed row on one page,
for the batch of anonymous callbacks and library stretches
(`1891634ee`).
The mutant whose filter in `freshlyTakenBack` keeps every slice failed the first two cases;
the third,
a round that took nothing back,
reads the same under both.
