# Audit ledger: code nothing reaches

Imports,
functions,
values,
modes and exits that no production path reaches,
and the checks that did not report them.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B16: 41 import bindings nothing referenced, and no check that reports one

Status:
fixed in `3817a99c9`,
2026-09-28;
the missing check is repository configuration,
outside the package,
and issue #578 asks for it.
`noUnusedLocals` is off in `package/config/typescript/tsconfig.options.json` and the linter reports none either,
so the provider-client merge (B12) left four `p-limit` imports behind unnoticed until counted.
A parser census over the package's 1,837 source and test files (`~/temp/agent/audit-glossary-fix/unused-imports.mjs`)
found 41 bindings no code references;
36 are removed across 24 files,
and the 5 kept are TSDoc `{@link}` targets (`ArtifactParseError` in three readers,
`FidelityReferenceError`,
`IssueEvidenceConflictError`).
The census now reports those 5 and nothing else.

## B20: functions nothing calls, and no check that reports one

Status:
fixed,
2026-09-28.
A census of every top-level function in the package's non-test source
(`.cache/dead-functions.mjs` in the package,
rolldown's parser)
found one private function its file never names beyond its declaration
and one export no other file names;
a control copy of the source with one planted function of each kind reported both.
- `cb30e516e`:
  `isAnchored` (`coverage-verdict.ts`) had been the evidence filter until `d6db46519` made full-coverage votes the only evidence,
  and `uniqueRosterModelIds` (`consolidation-naturalness-state.ts`) served the naturalness correction loop `0f1898942` removed;
  both are gone,
  with the import only the second read.
- `549f15bbd`:
  `dead-functions.unit.test.ts` fails on either kind,
  counting a test as naming an export;
  its fixtures first show it finds each kind and leaves a function called in its file,
  named in another source file or a test,
  or exported by a list.
  Writing the fixtures caught a flaw in its own whole-word check
  (the last occurrence read no following character,
  so a longer word counted),
  pinned before the package-wide case could pass on it.
  Both source guards now read through `source-scan.test-fixture.ts`.
  Mutants planting a dead private function,
  a dead export,
  and restoring a merged copy are each caught;
  a comment control survives.

## B30: functions only tests reach

Status:
fixed 2026-09-29,
found the same day while reading T8's cold stretches;
the values the same reach leaves unread are B31.
`consolidationFailureEvidence` held four cold stretches,
and nothing outside its own test calls it:
the single consolidation attempt (`1ba8f713a`,
2026-09-01) stopped building the prior-failure evidence it feeds,
and the consolidate sheet kept a branch for evidence no caller sets.
B20's guard never reported it,
because it counts a test,
and a barrel's re-export list,
as naming an export.

Measured at `e3ce445ed` by reach from production
(`t8-production-reach.ts` in the audit's scratch folder):
the roots are every non-test source file's module-level code,
which is where the runner entries call their main and where tables run on import;
a top-level function is live once a live context names it,
and its body is then a live context;
import and export lists name nothing.
Matching is by name,
so a name two functions share keeps both live,
and the error is only toward calling code live.
Its control planted a chain no root reaches,
an export that a barrel re-exports and a private function calls,
that private function,
and the export calling it,
which a barrel also re-exports,
beside a function a module-level constant calls;
it reported the chain's three and kept the fourth.
Every one of the 44 build entries sits under `src/`,
and no file outside this package imports it.
It lists 57 functions,
by what they are:

- Wrappers over parts production calls apart,
  tested in place of the parts:
  `assertResumableGeneration` (the pass runs a schema check between its halves),
  `verifiedAttestations` (the stage logs each verdict between its parts),
  `stripStubMarkers`,
  `comparisonRowsEqual`,
  `laneContestChoiceMayShip`,
  `groupNodesAligned` (production passes seals)
  and `corroboratedCount` (replaced by the prober count,
  `83f913a65`).
  Two comments still name the generation wrapper as what runs at pass startup
  (`corpus-pass.ts`,
  `artifact-provenance.ts`).
- Modules built beside the live path and superseded:
  the source-first grouper and orphan reflow (`group-source-first.ts`,
  `reflow-orphans.ts`),
  built unwired on 2026-08-15 "before it decides a corpus",
  whose job the sealed grouper's block-scale insertion took on 2026-08-23;
  the version 1 artifact writer (`buildSettledArtifact`),
  unused since both lanes settle each entry (`04c6d85cf`),
  whose round-trip case says it reads what the pass writes;
  the non-streaming completion parser (`extractCompletion`),
  unused since streaming (`33d3331d4`);
  and the critic benchmark (`runCriticBenchmark`,
  `computeScorecard`,
  `isRetryableAttempt`,
  `isTruncatedAttempt`,
  `prepareBenchmarkEntry`),
  which no runner calls.
- Functions whose caller changed course:
  `consolidationFailureEvidence` and its three helpers (`1ba8f713a`);
  `assertPublishableTranslation` (`c1db6f729` publishes a page with an unfilled passage as a recorded gap);
  `verifyArtifactAgainstPreparation` (the rendering audit checks measurements alone since A12b,
  `da9ca20b0`);
  `referenceLabels` (`ac179a4e4` reads footnotes from the document's syntax);
  `donorTextsFor` (`6354dd16a` calibrates on a reviewed donor);
  `estimateRequestWeight`,
  which routing never read;
  `renderJudgedRate`,
  whose runner changed;
  and `categoryFamily`.
- Test support shipped as package source:
  the TSDoc example scan and its text scanner (`tsdoc-example-scan.ts`,
  `source-text-scan.ts`),
  the catalogs' label lookups and Hyper counterpart lists,
  and `cardOf`.
- `resetRunSpend`,
  a documented seam that zeroes the process-wide spend meter between tests.

Fix:
the guard counts reach from production,
fixtures first,
and fails on these before any is touched.
A wrapper goes and its cases call the parts in production's order;
a superseded module or a function whose caller changed course goes with its cases,
once no other export of its file is live;
test support moves to `.test-fixture.ts` files;
and `resetRunSpend` is the guard's one allowed seam,
since no fixture can reach module-private state without an export,
a choice the owner may veto for an injected meter.
The consolidate sheet's branch goes with the evidence,
and the rendered-sheets census shows whether any sheet changes.

Done:
the guard (`src/dead-functions.unit.test.ts` (removed before 2026-10-18)) went in red in `eda99b687`
and passes from `32e07afb7`,
with `resetRunSpend` its one allowed seam.
`363b0ee60` removes the seven wrappers,
each case now calling the parts in production's order;
the generation guard's drift permission is required,
and the pass passes what it reads from the environment.
`94ccbc67c` names the guard the pass runs in the restart decision.
`9170b7158` moves the test support into `.test-fixture.ts` files,
`2fdcaddb2` removes the superseded modules with their cases,
and `9f49c2ed5` the functions whose callers changed course.
`32e07afb7` removes the consolidation recovery evidence,
the sheet branch no caller fed,
and the sheet-leak floor's label for its heading;
`consolidationNeedsRecovery`,
which read as asking for another strategy,
is now `standingKeptUnendorsed` beside its one caller.
No sheet changes:
the consolidate writer's sheet,
rendered through the wire before and after over the fixture's subject,
a bare one,
one reaching every optional block
and one whose texts carry fence runs,
is identical in all four,
where the old wire with prior-failure evidence set differs
(`b30-wire-compare.ts` in the audit's scratch folder).
The label's removal rides inside consolidation 20 and lane contest 6,
with an account in each key:
the heading is in no stored slate,
artifact or cache record,
and no slice-cache file was written after 04:27 UTC on 2026-09-27.
The whole suite passes on `32e07afb7`:
1,384 describe blocks pass and none fails
(a PASS line is one block,
naming its passing cases,
`describe.ts` in `module-test`;
this entry first called them cases,
M74).
Against the census baseline's 1,397 blocks,
the 25 gone belong to removed functions,
were renamed with them,
or are the guard's old block,
and the 12 new ones are the renamed and split blocks,
the guard's rewritten one
and the renamed rule's
(`b30-suite-diff.mjs`,
which pairs the two logs' PASS lines by tags and first case name,
so a case removed from a block that keeps others does not show).

## B31: values only tests read

Status:
fixed 2026-09-29,
found the same day by B30's reach probe run over top-level values
(`t8-production-reach.ts --values`).
Beside `resetRunSpend`,
the probe listed 23 top-level values no production code reads,
and the guard that went in red found `GAP_PENALTY` and `OPENROUTER_DECISION_IDS` as well:

- `ABSOLUTE_NATURALNESS_CONFIRMATIONS_REQUIRED`,
  which reads as configuration,
  while the confirmation asks exactly once whatever it says.
- `HYPER_ORIGIN_NAMES_ARE_SERVED` and its type,
  a proof that cannot fail:
  it stood over a hand-listed roster type (`cc7ccfc00`),
  and since that type became an `Extract` over the served ids
  (`ea0024e3c`,
  2026-09-16)
  the conditional can never reach `never`;
  its runtime case asserts a literal `true`.
- `SEATED_BEDROCK_JUDGES` and `SEATED_OPENROUTER_JUDGES`,
  read only by the seat tests,
  whose TSDoc holds the two models' seating measurements,
  and the three origin buckets they are built from
  (`HYPER_ORIGIN_ROSTER_IDS`,
  `BEDROCK_ONLY_ROSTER_IDS`,
  `OPENROUTER_ONLY_ROSTER_IDS`,
  with their three private spelling lists).
- The 13 role-named seats of `roster-fixture.ts`,
  unit-test support the package build ships.
- `GAP_PENALTY` in `align-blocks.ts`,
  documented beside the kind scores it is set against and read by nothing:
  `align-blocks-walk.ts` declared its own copy,
  "mirrored from the scoring module's calibration",
  and read that one,
  both since the files' first commit (`35c4a4aac`),
  so retuning the documented one would have changed nothing.
- `OPENROUTER_DECISION_IDS`,
  whose only production reader was a type (`typeof` it).

Fix:
the guard counts values by the same reach,
red first;
the constant and the proof go,
with the runtime cases that cannot fail,
since "SERVES every roster id from at least one catalog" carries the invariant the proof named;
the seated sets and the buckets move to a test fixture,
and their measurements to the two models' cards;
the seats move to a `.test-fixture.ts` file;
the walk reads the documented gap penalty;
and the decisions spelling becomes a type union.

Done:
the guard went in red in `e91ce423b`,
renamed `src/dead-code.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/dead-code.unit.test.ts`):
it tracks every top-level function,
class and value bound to a plain name,
roots production at the module-level statements that declare nothing,
and counts a value named only in type positions as unreached.
It named the 25 and no class.
`2efbdbde9` removes the proof,
its type,
the constant,
and the two roster-reach cases that could not fail
(the Hyper catalog refuses to load unless its rows are exactly the Hyper spellings);
the walk imports the documented gap penalty,
whose value is unchanged;
and the decisions spelling is a type union,
a choice under the owner's standing quality directive that the owner may veto,
the model-cards case now checking that no two cards share a decisions spelling.
`34f7db3b6` moves the seated sets and the Bedrock bucket to `roster-buckets.test-fixture.ts`,
drops the Hyper and OpenRouter buckets as no test needs them,
moves the two seating records to the cards,
and turns the model-cards case that restated the buckets into one about a Bedrock-spelled card.
`e28dd54f2` moves the seats to `roster-seats.test-fixture.ts`,
215 test and fixture files importing them from it.
The guard passes from `e28dd54f2`,
and the whole suite on it passes 1,384 describe blocks with none failing;
against B30's run on `32e07afb7`,
every PASS line pairs but the guard's and the seat test's,
renamed,
while the two roster-reach cases left a block that kept a third.

## B32: imports nothing reads

Status:
fixed 2026-09-29,
found the same day:
B30's last group removed four type imports from the consolidate wire by eye,
and no configured check reports one
(GitHub issue 578:
the repository's lint leaves `no-unused-vars` off,
and its TypeScript configuration leaves `noUnusedLocals` false).
TypeScript 7's package ships the native compiler alone,
with no compiler API a scratch check could call,
so the measurement used the source scan's parser
(`b32-unused-imports.ts` in the audit's scratch folder):
an import binding is unread when its local name stands nowhere in its file outside the import declarations,
type positions and export lists counted.
Its control,
a fixture with one unread value import and one unread type import beside read ones,
reported exactly those two.
Over the package's 1,915 files,
tests and fixtures included,
it found 15 unread bindings,
all in package source.
Five are error classes a TSDoc `@throws {@link …}` line names,
the only place a file names an error its callee throws,
and each line holds:
`artifact-repair-read.ts`,
`corpus-run/artifact-rounds-read.ts` and `corpus-run/artifact-vote-read.ts` call the `require*` helpers of `artifact-guard.ts`,
which throw `ArtifactParseError`;
`dedupe-issues.ts` calls `mergeDuplicateEvidence`,
which throws `IssueEvidenceConflictError` through `mergeClaimEvidence`;
and `corpus-run/judge-fidelity-probe.ts` calls `reviewedFidelityTrials`,
which throws `FidelityReferenceError`.
Those imports stay.
The other ten are read nowhere:
`footnoteMarkerLabels` and `ChunkPair`,
left by `9f49c2ed5` (B30,
which removed the functions reading them);
`fetchTransport` in the five provider clients,
left by `66cd08426` (X24,
which made every client's transport required);
and `ProviderRecord`,
`ModelCaller` and `ProviderName`,
left by `3510c8336` (X23,
which built the run client on a handed transport).
`git log -S` on each name in its file names the commit.

Fix:
`src/unused-imports.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/unused-imports.unit.test.ts`) went in red in `40add2b70`,
reading every package file through the source scan
and counting a name a TSDoc link names as read;
it named the ten and joined `mise run source-scans`.
`c077cc881` removes them;
the five clients now import only types from the transport module,
so those imports are `import type`.
The guard passes from `c077cc881`,
and the whole suite on it passes 1,385 describe blocks with none failing,
the one new block the guard's.

## B47: a refine mode no production code built, kept alive by tests

Found by the eighth coverage batch (census-dUOMFS),
where `consolidation-polish-round.ts` asked the same question three times:
whether the mode was `required-naturalness-correction`.
No production code builds that mode:
`rg` over `src` without tests or fixtures finds only reads.
`git log -S` names `0f1898942` from 2026-09-01,
"remove the naturalness correction loop",
which removed its only producer;
the mode,
its branches in the refine prompt,
the refine selection context and the polish gate sheet,
its prior-correction rendering,
and its `no-correction` outcome all stayed,
built by tests alone,
and the rendered-sheets fixture rendered a slate no run could send.
Production builds two modes:
comparative and objection correction.

Fixed in `6a3e6cffa`,
a quality call open to veto:
the mode and everything only it reached are gone,
and so is `disposition` on the refine stage and polish round results,
which without `no-correction` equalled `changed`
and which no production code read.
The cases that built the mode now pin the objection mode where it had the same thing to pin:
fencing,
the base label,
and a declined or split round keeping the text.

No cache version moves,
on a measurement rather than a reading:
a probe rendered 58 sheets from the builds before and after the edit:
every fixture sheet,
and the gate,
the selection context and the refine prompt in each live mode,
with and without declared names and references.
49 are byte-identical;
the 9 that differ are the removed mode's own renders,
the probe's positive control.

Recurrence:
when a commit removes a producer,
`rg` for every constructor of the value it produced,
and remove each branch,
type member and test that only that value reached.
A test that builds a value production cannot build keeps dead code looking covered.
Left as it is:
`consolidation-polish.ts` still records `correctionCount: 0` and `corrections: []` on every review;
those are stored artifact fields with readers,
so changing them is a schema change for its own entry.

## B51: an archive-keeping exit for an absence no judging raises

Found by the eighth coverage batch (census-dUOMFS),
behind B50:
the settlement's exit for an absence the slate judging raised over a withheld standing
(`consolidate-settle-judge.ts` returning it,
`consolidate-settle.ts` keeping the archive on it)
was cold,
and no input reaches it.
The judging raises over a withheld standing only on a slate with no candidate and nobody heard;
with a candidate it ships its preference past every decline,
by the owner's "Preference + polish" of 2026-09-27,
and its retry raises only while a run-off still narrows.
The settlement keeps the archive before judging where no proposal survived the floor and no lane text is offered,
the lane offer drops blank texts and any the rule does not pass,
and the candidate builder keeps every lane text that shows something;
so a slate that reaches the judges has a candidate or a surviving proposal.
Two inputs were tried and neither reached the exit:
one blank surviving proposal returns "nothing proposed" (B50),
and no voices at all settles `incumbent-only` first.

Fixed as a quality call open to veto,
on the shape ledger B43 set for the floor:
the judging returns only decisions,
and an absence over a withheld standing throws `WithheldSlateAbsenceError`,
a fault in this code,
through one narrowing at the one call site (`requireWithheldSlateDecided`),
cased directly in `consolidate-settle-judge.unit.test.ts`.
An absence over an eligible standing passes through as before.
The owner's rules that a run always ships are about text;
like B43's disagreement,
this names two rules of the code falling out of step,
which no text can cause.
Rides inside `CONSOLIDATE_CACHE_VERSION` 20:
no slice-cache file was written after 04:27 UTC on 2026-09-27,
where the midnight control finds 494.

Recurrence:
when an owner rule turns a raise into a return,
trace every caller's handler for that raise
and retire the ones it left unreachable,
with a named fault where the rule's premise is another module's.

## B52: an unsettled polish nothing built, and three guards against it

Found by the eighth coverage batch (census-dUOMFS),
at the driver's raise of `NaturalnessCompletenessError` for a polish of kind `unsettled`.
No production code builds that kind:
`rg` over `src` without tests finds only its definition and its readers,
and `git show` of `consolidation-polish.ts` just before `0f1898942`
shows the one construction that commit removed,
"remove the naturalness correction loop" of 2026-09-01.
Its TSDoc kept it "for old records and defensive guards",
but no record can carry it:
the artifact projection's output holds only `settled` and `not-run`,
the consolidation cache refused to keep an `unsettled` settlement,
and no slice-cache file has been written since 04:27 UTC on 2026-09-27,
before the cache version the current keys carry.
Three guards read it:
the driver's raise,
the persistence check refusing it,
and the artifact projection's throw.

Fixed as a quality call open to veto:
the kind and its three guards are gone,
the projection loses the slice index it took only to name the throw,
and the one test that built the kind,
a persistence case for a correction loop that no longer exists,
went with it.
`NaturalnessCompletenessError` stays,
thrown by the artifact completeness check.
The cost log's `unsettled` exit stays readable and is no longer written:
an `rg` over the agent runs found `exit=unsettled` in a validation log of 2026-08-29.

Left as it is:
the consolidation cache's shape check trusts a cached settlement's `polish` without reading it,
a gap for the cache store's own entry;
and the settlement type still admits a gate's terminal with no gate and a declined slate with no decision,
states only a type split by terminal would rule out.

Recurrence:
a variant kept "for old records" names the records;
check each store it could live in before keeping it,
and delete it with its guards when none can hold it.

## B118: test declarations nothing read, and fixture exports nothing imported

Red in `5f3e63e67`,
fixed in `cacfa20fb`.
B30 and B31 guard package source by production reach,
and B32 guards imports,
but nothing read the declarations of test files and fixtures:
the repository's lint leaves `no-unused-vars` off
and its TypeScript configuration leaves `noUnusedLocals` false.
Four declarations were read nowhere:
`SUPPORTING_READERS` and `NAME_QUOTE` (`archive-naming.test-fixture.ts`),
`sliceCarrying` (`fidelity-damage.unit.test.ts`)
and `withoutBreaks` (`semantic-wrap.unit.test.ts`),
the last two named only inside their own `@example`.
Twenty-four fixture exports no other file imported,
each read only inside its own fixture,
from constants such as `ARCHIVE_PATH` to the `ScratchDir` and `ParsedSource` types;
an `export` only its own file reads offers nothing,
and hides the next dead one behind it.

`dead-test-declarations.unit.test.ts` reads every test file and fixture.
An unexported top-level declaration fails unless its name stands where it is read:
never as a property,
member,
class or interface member,
enum member,
specifier name or label,
and never as another binding,
such as a parameter or a nested declaration.
A fixture export fails unless another file imports or re-exports it by name.
A default export,
a namespace import and a dynamic `import()` of a relative path fail as forms the scan cannot follow;
the package's tests use none.
A read of a nested binding that shadows a top-level name,
and two declarations reading only each other,
still count as reads,
since telling them apart needs scope resolution.
At `5f3e63e67` the scan found 28,
the same 28 names the agent's TypeScript-checker census of the same tree listed.

A read-only agent drafted the fix and the scan.
Its fix was applied as written;
its scan was reworked before the red commit:
the draft counted a property key or member name spelling a declaration's name as reading it,
counted a fixture export its own file reads as live,
which left the 24 exports unguarded,
and carried a default-import branch that could mark nothing read.

Recurrence:
`mistake-prevention.md`,
"Lint and edits".
