# Audit ledger: copies of shared code in tests and scans

Helper bodies the test files and the source scans kept in copies,
and the scans that could not see a copy.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B101: the source-scan tests kept their own copies of the shared helpers

Found 2026-10-01 (UTC) reading the scan tests for helpers B99 had not hoisted,
fixed in the commit adding this entry.
Each scan test that parses the package's source took its tree helpers from `source-scan.test-fixture.ts`
and also defined some of them again:
15 files held a `fixture` builder returning the object it was handed,
6 held their own `identifierName` and one the same body as `identifierNameOf`,
2 held one `literalText`,
and `command-line-reads.unit.test.ts` held its own `nodesUnder`
and an `unwrapped` that returned the bare inner node.
The duplicate-body scan reads production source alone
(`duplicate-bodies.unit.test.ts` filters test files out),
so none of these copies could fail it.

The 26 local definitions are gone:
the 40 builder calls are the object literals they returned,
`literalText` joins the fixture beside `identifierName`,
and `command-line-reads.unit.test.ts` reads the fixture's `unwrapped` through its `.inner`.
Copies that differ in behaviour stay where they are:
`count-nouns.unit.test.ts`'s `literalText` unwraps first,
`floor-inputs-stated.unit.test.ts`'s `unwrapped` reads fewer wrapper kinds,
and the two `callsNamed` differ in unwrapping.

The duplicate-body scan's own logic,
run over the 946 test files and fixtures after this change,
found 96 groups holding 266 copies.
Six are whole test-case bodies:
three repeated across the glossary test files,
and three repeated within one file each
(`fidelity-splice.unit.test.ts`,
`grace-override.unit.test.ts`
and `mask-invisible-lines.unit.test.ts`).
They are queued as a family of their own,
to be hoisted,
deleted or listed with a reason before the scan widens to test files.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B103: test files kept their own copies of the shared test helpers

Found 2026-10-01 (UTC) by a read-only census an agent ran over every unit test and fixture,
by each helper's name and by its body's shape under any name
(`fixture-migrate/report.md` in the audit's scratch folder),
fixed in the commit adding this entry.
Three fixtures held a helper whose header said test files carry their own copies
and new cases import the fixture's:
`candidateCarrying` in `translate-ballot.test-fixture.ts`,
`capturingLogger` in `capturing-logger.test-fixture.ts`
and `scratchDir` in `scratch-dir.test-fixture.ts`.
The copies the change removes number 42 under 13 names:
six readings of a judge's ballot
(five `pickCandidate` and a `correctedCandidate` with its needle fixed),
19 throwaway directories under 10 names
(eight `scratchDir`,
three `throwawayCacheDir`,
and `makeArchive`,
`throwawayTree` and others),
differing only in the prefix and the field naming the path,
and 17 capturing loggers,
13 the fixture's own under three names for the array
and four that made the array themselves and returned it beside the logger.
The duplicate-body scan reads production source alone,
so none of them could fail it.

Each file now imports the fixture:
the wrapper's call passes its needle,
callers of the four that made their own array call the new `capturingLoggerPair`,
and handles are read through `.path`.
The agent's patch needed more,
none of it found by the tests it changed.
Lint found seven files keeping an empty `import {} from 'node:fs/promises'` its import cleanup left,
one case destructuring the old `publishDir` field
(the type check's TS2339)
and `capturingLoggerPair`'s return type on one line.
The full suite's unused-imports scan found 21 imports only the deleted helpers had read
(14 `Logger`,
6 `join`
and 1 `mkdtemp`),
after lint and a run of the 41 changed test files had passed (M75);
the full suite's background task reported exit 0,
the status of the `|| echo` fallback its command ended with,
and the log's `[FAIL]` markers were what showed the failure.
Reading found that `sink-names-only.unit.test.ts` had lost the comment saying why the disposer removes a mode-000 file,
now restored beside the `chmod`.
The three fixture headers claimed every copy had moved;
they now say the copies moved and name the kinds that stay.

Copies that differ in behaviour stay where they are,
each queued:

- `corpus-run/pass-entry.unit.test.ts`'s `pickCandidate` matches any of several needles;
- `consolidate-driver.unit.test.ts`'s own `candidateCarrying`
  and the `numberedOn` inside a case in `consolidate-settle.unit.test.ts`
  read a serialized request body,
  where a line break is the two characters `\n`,
  and are the same code as each other;
- `repair-assemble-withdrawal.unit.test.ts` and `stage-call.unit.test.ts`
  each record warnings alone over their file's tagged logger,
  the same code twice;
- helpers that make a directory and never remove it are a leak,
  not a copy,
  and get an entry of their own.

Found while writing this entry:
`mistake-prevention.md`'s "Copies of shared code" still said nothing fails on a shared module without its own test,
a sentence the B102 scan made false;
B102 wrote a section of its own and left that one standing.
It now names the scan.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code"
and "Current-state docs".

## B111: test files kept 90 groups of copied helper bodies

Queued by B101 and B103,
fixed in part in the commit adding this entry.
The duplicate-body grouping,
run over the package's test files and fixtures at the parent commit
(948 files,
bodies of 80 or more characters with comments and whitespace aside),
finds 90 bodies kept in two or more places,
247 copies in all.
After this change it finds 58 groups and 137 copies over 966 files.
A read-only agent wrote the patch:
32 groups move into 18 new `.test-fixture.ts` files
(scripted clients,
stream frames,
refusal readers,
glossary lookups,
lane evidence and others),
and 80 test files import them.
Two helpers that shared one name did different things,
so each fixture names its own:
`rejectionOf` hands back whatever a call rejected with,
and `statedRefusalMessage` reads the message of the package's own stated refusal.
The stage-recovery-round test's meow reply,
its guard and its format move onto the shared trivial-reply fixture with them.

The patch needed more than its review found.
A fixture file gets the package's production lint rules,
not the test-file override,
and lint over the patched tree reported 18 errors and 33 warnings:
arrow functions,
missing return types,
type-only imports without `import type`,
positional parameters,
callbacks passed by reference,
unchecked type assertions,
and a named import nano-spawn does not export (TS2614).
Reading found an import the patch had written inside a template literal,
names that collided once two copies met in one file,
and TSDoc blocks left on statements,
which the TSDoc attachment probe still found three of after the first fixes.
All are fixed here;
the assertions are type guards or `Error.isError` checks,
and the glossary tests call `seededAmong` rather than passing `isSeeded` to `filter`.

One move was wrong in kind and is undone.
The patch hoisted `collectingInto`,
which replaces `console.log`,
out of the two report tests whose suites run one case at a time.
The global-writes scan follows a writer to its cases by calls within one file,
so in a fixture its write has no case.
Run with that fixture restored for the measurement,
the scan fails on both of its writes,
each `console assigned, in collectingInto, which no call by name reaches`;
the source scans had not yet run over the patched tree.
Both copies stand beside their sequenced suites again.

Open:
the 58 remaining groups,
among them five capturing `console.log` that the case's own sandbox
(`ctx.sinon.stub(console, 'log')`)
could replace in one fixture,
and eight glossary cases whose bodies match because each reads its own file's term list.
The duplicate-body scan widens to test files once each is hoisted or listed.
Its grouping strips whitespace inside string literals as well,
so two bodies differing only in a literal's spaces group as one;
the widened scan has to compare literals as written.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code"
and "Globals a case replaces".

## B114: test files kept helper bodies twice within one file

Queued by B111,
fixed in the commit adding this entry.
Of the 58 groups B111 left,
19 sit within one file.
A read-only agent read every copy and found no test defect:
no pair of cases whose identical bodies left one case's claim untested.
Ten groups were one helper written twice and are hoisted to one function in their file
(`critic-wire`,
`candidate-select`,
`archive-block-review-stage` with four copies,
`provider-router`,
`repair-translation`,
`translate-retry`,
`translate-selection-sheet`,
and in `corpus-run` `run-timing`,
`artifact-two-lane-read` and `insertion-container-deficit`).
Six are one scenario split into cases that assert different things of it,
the class of an error in one and its message in another,
and stay.
Three are not copies at all:
the grouping strips every character `trim` reads as space,
inside string literals too,
and `trim` reads U+FEFF as space,
so a cut leaving one space against one leaving two (`fidelity-splice`),
a line of U+FEFF alone against one mixing it with spaces (`mask-invisible-lines`)
and an empty override against one of three spaces (`grace-override`)
group as one body.

The grouping over the tree with this change finds 48 groups and 115 copies over 967 files.
The duplicate-body scan's widening to tests has to compare literals as written
and lists the six split scenarios with their reason.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B115: test files kept helper bodies in two or more files

Queued by B111,
fixed in the commit adding this entry.
A read-only agent wrote the patch for the groups B114 left that span files:
17 new `.test-fixture.ts` files hold the hoisted helpers
(delta routing,
container tag ownership,
coverage-control cases,
the cap refusal,
scripted replies,
wet budgets,
a recorded transport,
issue-author builders,
slice and anchor pairs,
attribution proposer ids,
seed coverage,
insertion slices,
an unfilled-anchor artifact
and lane candidates),
and 41 test files import them.
Helpers that shared a name but built different things get names of their own
(`preparedPairAt` and `slicePairOf` for two different `pairOf`).
Three of the 41 files had changed under B114 after the agent read them;
those three were merged three-way against the commit it read,
with no conflict.
The grouping over the tree with this change finds 20 groups and 55 copies over 985 files.

`consolidate-proposal-wrap.unit.test.ts` held six helpers nothing called:
its one suite tests the wrapper alone,
and the helpers drove the stage,
which `consolidate-slate-carries-shipping-text.unit.test.ts` covers with live copies.
The agent deleted four with their constants and imports;
two more,
`asSent` and `judgeBallot`,
named only in their own TSDoc examples,
are deleted here.
No configured check reports a declaration nothing reads,
the same gap B32 closed for imports;
a census of the package's test files for more is open.

The patch needed more than its review found,
as B111's did.
Lint over the patched tree reported 13 warnings and an error,
all in the new fixtures or in callers of them:
the callback-arity rule resolves a local function's parameters but not an imported one's,
so two `map(toLaneCandidate)` calls now pass a named wrapper;
stubs with no `await` return a settled promise instead,
the budget stubs as module functions,
as `pass-outside-reads.test-fixture.ts` writes them;
a void function carried `@returns`,
and two expressions broke the one-step-per-line and one-property-per-line rules.
The unused-imports scan then found four type imports the hoists had orphaned,
`ModelTransport` in two client tests and `BudgetView` in two router tests,
which lint does not report.
The agent kept two type assertions it could not check without a compiler,
`as ChunkPair` and `as unknown as WouldShipSource`;
both literals type-check without them,
and both are gone.

The 20 groups left are not helpers waiting to move.
Six are the split scenarios B114 kept,
and three are B114's literal-whitespace groupings.
Two are the glossary cases B111 named,
each reading its own file's term list.
Five replace a process global
(`console.log` collectors and environment restores),
which the global-writes scan follows only within one file.
Four have the same text over different bindings:
`refusalOf` asks a different command-line reader in each file,
`ask` sends each file's own messages,
`sliceOf` cuts each file's own document,
and `capturingLogger` wraps each file's own tagged logger,
the pair `capturing-logger.test-fixture.ts` already names as kept apart.

Open:
the duplicate-body scan widens to test files,
comparing literals as written
and listing the remaining groups with their reasons
(closed by B116);
the census of test declarations nothing reads
(B118).

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B116: the duplicate-body scan read production source alone and grouped literals by their visible characters

Queued by B101,
B111 and B115;
red in `de7c6b099`,
cleared in `5b43c5f45` and `8ea5e7fa5`.
The scan read no test file,
so a helper copied between tests,
or a test restating a module's body as its expectation,
could not fail it.
And its grouping dropped every character `trim` reads as space,
inside string and template literals too,
so bodies building different text grouped as one (B114).
It now reads every file under `src`,
and compares each literal token as written
while dropping the spacing of the code around it;
a fixture case shows the change,
and a scratch copy comparing literals the old way fails that case.

Over the tree at `de7c6b099` the widened scan found 26 groups where 4 frozen copies are listed.
Seventeen were the groups B115 left,
less its three literal-whitespace groupings.
Five more only the widened scan could see.
Two were short predicates that each spelled a long fixture string twice,
in `consolidate-driver` and `consolidation-polish`,
past the 80-character floor once the string's spaces count;
each now reads one named constant,
used where the string is offered and where it is sought.
Three were tests carrying a production body.
B115's own lane-candidate fixture was a copy of the function `buildTranslateCandidates` wrote inline;
`translate-candidates.ts` now exports it as `laneCandidate`,
and the fixture is gone.
`run-seats.unit.test.ts` restated `run-config.ts`'s translator filter as its expectation,
which can fail only if the formula itself changes;
it now states translators as the writers less the seats dropped from the translator role,
a relation between two exported lists.
`slice-implausible.unit.test.ts` restated `sliceSizesOf`'s map;
it now spells out each slice's size from its own texts.

The six split scenarios B114 kept are one named function each in their files,
each case still asserting its own claim;
in `repair-contract.unit.test.ts` one case's assertions are a superset of the other's,
and both stay.
The groups over different bindings take the binding as a parameter:
the glossary cases' term lists,
the command-line reader `refusalOf` asks,
the messages `ask` sends,
the document `sliceOf` cuts,
and the logger `capturingLogger` wraps.
The console captures stub `console.log` on the case's own sandbox,
which the global-writes scan reads as no write,
and read their lines off the stub's recorded calls with `util.format`;
the agent's version faked `console.log` with a rest-parameter arrow function,
which fixture lint refuses.
Three groups stay,
listed as `GLOBAL_WRITER_COPIES` with their reasons:
each writes an environment variable the code under test reads itself,
and the global-writes scan follows a writer within one file only.

The scan now lists 7 groups,
the 4 frozen copies and those 3.
The agents' fixtures needed lint fixes before they passed
(a duplicated import,
chain and import layout,
an inline object type),
of the kinds B111 and B115 record;
`repair-assemble-slice.test-fixture.ts` also gained a refusal for a slice wording its document does not hold,
where the copies addressed offset -1.

Open:
the global-writes scan following a writer through its imports,
which would let the three kept groups share one fixture.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B120: the global-writes scan followed a writer only within its own file

Red in `c9639c9ff`,
fixed in `dbefa8aec`.
B116 left it open:
the scan placed a write inside a function declared by name at the calls of that name in the writer's own file,
so a writer moved into a shared fixture read as one no call by name reaches wherever it was used,
and three groups of environment writers stayed copied between test files
(`GLOBAL_WRITER_COPIES` in `duplicate-bodies.unit.test.ts`).
Each test file is now read once into its parents,
its calls by name
and the names it imports by name from a relative path.
A named writer's callers are its own file's calls of the name,
plus the calls of the name an import binds,
alias or not,
in every test file importing it from the writer's file.
Out of reach,
and named in the scan's module header:
a writer reached through a namespace import or a re-export.
The package's tests use neither,
and the dead-declaration scan already fails a namespace import.
`resolveSpecifier` moved into `source-scan.test-fixture.ts`,
which both scans import.
The three `GLOBAL_WRITER_COPIES` groups can now move into a fixture;
that move is queued.
Closed since by B385 in `3fa45b1377073e74042d66fdb033f2533abbb7cc`.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".

## B385: runner tests kept near copies of helpers a shared fixture could hold

Changed in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
passing before and after `9d558da592162f9765fb03a2280af9477ada147d`;
nothing in it can be red.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers.
Its census read every top-level helper in the package's tests and fixtures,
reduced each body to its syntax shape
(node kinds,
operators and property keys kept,
identifiers and string,
template and number values dropped),
and grouped shapes of length 300 or more:
111 groups,
41 of them with members in two or more runner files.

The change:
the red commit names four fixtures,
`fresh-run-seats`,
`pool-policy`,
`runs-dir-pointed` and `slice-overlap-dial`,
with `runProbeOver` in the probes-b fixture.
`fresh-run-seats.test-fixture.ts` holds `withFreshRunSeats`,
whose body stood identical in the cli-refusal and run-config tests (group 104);
`runs-dir-pointed.test-fixture.ts` holds `runsDirPointedAt`,
identical in four tests (group 24);
`slice-overlap-dial.test-fixture.ts` holds the overlap dial of the slice-overlap and pass-entry tests,
a pair outside the census's 41;
and `probes-b-built-command.test-fixture.ts` holds `runProbeOver` and `ONE_ENTRY_CORPUS`,
from the `runOverOneEntry` of the coverage-probe and coverage-control-probe tests (group 12),
which the translate-probe test now uses too,
its second case keeping a corpus of its own for the `commitSha` it needs.
`pool-policy.test-fixture.ts` holds the values of the pool change (B380),
and replaced the variable clearer of `artifact-pool-names.unit.test.ts`,
which had no copy.
`GLOBAL_WRITER_COPIES` is gone from `duplicate-bodies.unit.test.ts`:
its three groups were `runsDirPointedAt`,
the `restore` nested in it and the overlap dial's restore,
kept apart while the global-writes scan followed a writer within its own file,
which it no longer does (B120);
the case's name no longer mentions them.
By the agent's run against the base build,
the moved files pass with their case names as at the base,
the renamed duplicate-bodies case aside.

Left,
each with the agent's reason,
read from a diff it ran:
group 3,
two cache stores answering `get` and `has`;
group 4,
two row scenarios with different counts;
group 8,
two console methods diverted;
group 43,
clients scripting different sheets;
and group 67,
different corpora,
pages and pictures.
Identical raw bodies the duplicate-bodies scan passes,
though it reads the whole package,
since it compares bodies of 80 or more characters with whitespace and comments aside,
and these fall under that floor once normalized,
by the agent's reading:
group 28,
`textsOf` and `idsOf`,
80 to 92 characters raw,
and group 72,
the Canadian helpers,
72 characters raw.
Read by names and members only:
group 38,
reports driving different procedures;
groups 14,
60,
88 and 98,
two-lane cat and kitten families for different scenarios;
group 53,
width clients scripted by model and by stage;
group 103,
`noReaches` over different reach types;
and 23 groups of shape alone,
helpers with different subjects and literals
(31,
32,
45,
52,
62,
65,
68,
73,
75,
76,
79,
80,
81,
82,
84,
91,
94,
99,
107,
108,
109,
110 and 111).

Open to the owner's veto:
Group 25's environment setters,
`withDriftVar` and `withRunsDir`,
set different variables and stay apart;
the alternative is a new setter keyed by the variable's name.

Recurrence:
`mistake-prevention.md`,
"Copies of shared code".
