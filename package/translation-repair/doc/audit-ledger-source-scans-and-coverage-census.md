# Audit ledger: the source scans and the coverage census

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

What the source scans and the coverage census counted wrongly,
could not see,
or printed too little of when they failed,
and modules tested only through their callers.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B61: the census reading counts only the baseline's stretches

Found by the eleventh coverage batch,
reading `census-5PAw2O` against `census-BF4g7o`.
`baselineStatusesOf` (`corpus-run/coverage-census-baseline.ts`) reads each baseline stretch as ran,
still cold or not loaded,
and in a reading of every source `editedClaimsOf` names only the edited sources the baseline holds a stretch in.
So two kinds of cold stretch are counted nowhere:
one in an unedited source that overlaps no baseline stretch,
code the baseline's run reached and this run did not,
and one in a source added since the baseline.
A batch whose change left code cold would read clean.
At `census-5PAw2O` three stretches of the first kind stand,
`with-timeout.ts` 57 to 59 and `create-logger.ts` 240 to 247 and 366 to 373,
and the printed reading shows none of them;
comparing per-file counts in a scratch script (`t8-batch8-file-diff.mjs`) showed them,
and that comparison misses a stretch going cold where another in the same file ran.

Fixed (`48c580d16` red,
`5ebe2ab62`,
and `4304d167d` for the sort's line order):
`coldSinceOf` names each stretch of this run holding a line no baseline stretch of its source held,
read line by line,
so a stretch reaching past the baseline's lines or across the gap between two is named too,
with whether the baseline loaded its source;
the reading prints each as "cold since then",
and in a reading of every source `editedClaimsOf` reads the sources either census holds a stretch in.
Through the built package,
`census-5PAw2O` read against `census-BF4g7o` holds 4 stretches cold since then:
the three of the first kind,
and `error-format.ts` 31 to 58,
of which the baseline held only lines 41,
49,
50 and 52.
Reading only the baseline's sources in `editedClaimsOf` failed the added-source case alone;
reading a stretch as held where any one of its lines was failed the reaching-past case alone.
The census at `eafa894be` (`census-tBNOKA`) printed "cold since then 0"
and left one cold stretch in the reading itself,
the sort's line order,
which `4304d167d` cased;
without the order that case alone failed.
A review found the reading still named code no source map places
(`5af5eeec8` red,
`ce32e438d`):
the census records it as `(unmapped) <bundle>`,
lines 0 to 0,
so a rebuild that renames the bundle gives unchanged code a new source,
and `census-BF4g7o` read against `census-VW2tHL` named six such stretches cold since then.
The reading leaves unmapped code out,
since it has no lines to compare and the census's own report counts it,
and the name now has one writer and one reader (`unmappedSourceOf`,
`isUnmappedSource`);
without the check that case alone failed.

Recurrence:
a comparison of two runs reads each side for what the other cannot say.

## B62: the census asks git about edits inside the package only

Found with B61.
`sourcesEditedSince` (`corpus-run/coverage-census-commit.ts`) runs `git diff --name-only --relative` from the package directory,
which lists no path outside it,
and with no claimed sources asks about `.` alone.
The census reads the sources of other packages and dependencies the suite loads as well
(61 files at `census-5PAw2O`,
among them `module-logger`'s,
`module-async-time`'s and `cli-markdown-lint`'s),
so an edit to one of them since the baseline leaves its baseline stretches matched by line against code that moved,
the misreading `d27a89dd0` fixed for the package's own sources.
No reading so far was touched:
the oldest census file in the cache was taken at `e22373347` (2026-09-29),
and between no two consecutive census files does a commit edit a source outside the package that the earlier one holds a stretch in
(`b62-other-edits.mjs`,
41 pairs;
its `git diff` lists the logger's files across `ddccea85d`,
the control).

Fixed (`fff46159d` red,
`eafa894be`):
`sourcesEditedSince` names every file of the work tree changed since the commit,
relative to the package as the census names sources,
and its `sources` parameter is gone,
since every reading asks after the sources it reads.
The red case found a second fault in the same call:
read line by line,
a name git quotes (one holding a double quote,
a newline or a Han letter) came back quoted,
octal escapes and all,
and matched no source;
git's output is now read NUL-separated,
which git prints verbatim.
Through the built package,
the check against `ddccea85d~1` names the seven `module-logger` files git lists across that commit.
Limiting the diff to the package failed the whole-tree case alone,
and so did splitting its output on newlines.
One narrower gap of the same kind stays,
by choice:
`packageCommit` reads a tree as clean from `git status` under the package alone,
so a baseline taken while another package's source held uncommitted edits,
later discarded,
would match that source's baseline stretches against code that differed when the census ran.
Reading the whole tree for cleanliness would refuse a census whenever any file of the work tree outside the package is dirty,
however unrelated.

Recurrence:
a question put to git covers everything its answer is read against,
and its output is read NUL-separated.

## B102: 300 production modules with no unit test of their own

Found 2026-10-01 (UTC) by a read-only census an agent ran over the package's tracked source
(`own-tests/census.mjs` in the audit's scratch folder),
guarded in the commit adding this entry.
Of 1,086 production modules,
669 have a `<stem>.unit.test.ts` beside them and 417 do not.
Of the 417,
44 are barrels,
38 hold only types and literal constants,
and 35 are runner entries `src/build-entries.ts` lists;
the other 300 are logic,
139 of them named by some other module's test
and 161 named by no test at all,
reached only through whatever imports them.
A module tested only through its importers loses that coverage without a word when an importer changes,
and nothing stopped a new module from joining them.

What enforces it:
`src/own-unit-tests.unit.test.ts`,
among the source scans,
holds the 300 in a sorted `ALLOWLIST`.
It fails on a production module with no own test,
no exemption by shape
and no line there,
and on a line whose module gained an own test,
turned out exempt
or is gone,
so the list only shrinks.
The exemptions are read off each module's own syntax tree:
a barrel,
a module of types and literal constants,
or a runner entry named among `src/build-entries.ts`'s string literals.
Its fixture case runs first and passed from the start;
with `src/active-footnote-markers.ts` taken out of `ALLOWLIST`,
its package case failed naming that module alone as unlisted,
and the line was then put back (300 lines again).
Out of its reach:
whether a module's tests exercise every branch,
which the coverage census measures,
and a runner entry known only by a top-level `await` or a `process.argv` read.

Calls made here are open to veto:

- an own test is the exact `<stem>.unit.test.ts` beside the module;
- the 300 stand listed rather than each getting a test first,
  so they are worked down by the coverage batches and the list shrinks as they go.

Recurrence:
`mistake-prevention.md`,
"Modules without a test of their own".

## B105: the census reading counted unmapped code as not loaded after every rebuild

Found 2026-10-01 (UTC) reading the twenty-fourth batch's census,
red in `07b5bbd49`,
fixed in the commit adding this entry.
The census records code no source map places as `(unmapped) <bundle>` at line 0,
and the bundle's name carries a content hash a rebuild moves.
`ce32e438d` left such code out of "cold since then",
but `baselineStatusesOf` (`corpus-run/coverage-census-baseline.ts`) still read every baseline stretch,
so the same six stretches of the `wording-key` chunk read as not loaded at every census whose chunk hash moved:
`census-xSHq2Q` against `census-NkULde` holds them at the same bundle offsets,
1946 to 2352,
under `wording-key-FnMvcg3m.mjs` and `wording-key-k661Sfmz.mjs`,
and the reading printed "not loaded 6".
Ten passages of the T8 batch records explained those six by hand
(`rg 'not loaded are|chunk is renamed' doc/audit-ledger.md` (moved on 2026-10-18 to
`package/translation-repair/doc/audit-ledger.md`) before this entry).
The reading now leaves unmapped code out of every status,
as it already did of "cold since then";
the census's own report still counts it on its unmapped line.
With the filter taken back out,
the new case failed alone in its file,
and passed once it was restored.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence".

## B277: 44 test cases in ten files passed without printing a pass line

Scan red in `0634aa8bc`,
fixed in `63f8f838a`.

Found on 2026-10-06 (UTC).
An agent closing housekeeping items reported that `fidelity-reference-read.unit.test.ts` "prints nothing and exits 0",
in a file that was not its own,
and the lead followed it up.
`describe` of the test module logs the passing children of a suite named `''` at `debug`,
so a case whose nearest suite has an empty name passes without a `[PASS]` line.
A failure there still prints and still fails the file (the agent's control run),
so nothing was hidden but the count.
Every count of `[PASS]` lines this ledger quotes is a count of suites that print,
and such cases were in none of them
(the T8 entry already records that the count is of named suites and not of cases).

The fix:
`silent-test-cases.unit.test.ts` finds every `it` whose nearest `describe` has an empty name,
or none,
and names the file,
the suite chain and the case.
Its package case failed with 44 findings in ten files.
`deepseek-v41-admission.unit.test.ts` and `deepseek-v41-routing.unit.test.ts`
kept every case in a root suite named `''`;
the roots take a name.
Five files each kept a case loose beside their named suites
(`displacement-class`,
`edit-wire`,
`photo-reference`,
`restoration-judge-wire` with two,
`transient-retry`);
each case sits in a named suite of its own.
The four `fidelity-reference*` test files are rewritten under named suites.
Every refusal case there asserts the whole message and names the boundary that fired,
through fixture helpers that spell the message out and do not build it with the class under test,
and the command-line case no longer names a real page:
it reads the entry that lacks an alteration from the checked-in manifest.
The scan joined the `source-scans` task,
which ran 40 files from then.

The agent's measurements,
not rerun by the lead:
38 single-site mutations of the fidelity-reference modules,
each caught by a rewritten case,
23 of them by none of the cases they replace;
with every file run by name against one build,
1,644 `[PASS]` lines where 1,628 stood,
the rise being suites that print now and did not.
No production file changed.
The whole suite ran on `63f8f838a` in a side checkout:
1,672 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 40 source scans passing.

Not reached,
by the agent's account:
a suite named by a variable or a member reads as named whatever it holds,
and `describe` or `it` imported under another name is not followed.

Open:
the six allow-list lines of `own-unit-tests.unit.test.ts` for the fidelity-reference modules stay,
since the one test file reaches several modules and splitting it is a change of its own.

Open to the owner's veto:
the root names and the six wrapper suite names;
the row for an unsupported damage family kept through `as never`,
as the case of a loud guard production cannot reach.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence".

## B338: a failing scan over the package printed how many findings it had and not what they were

Changed in `2339cf5a6`;
issue 610.

Found on 2026-10-06 (UTC):
the first red run of the scans of B336 and B337 printed only
`expected [ …(31) ] to deeply equal []` and `[ …(13) ]`,
and reading the findings took a second run of each file outside the runner.
By the lead's inference,
the 13 are the 12 lines of B337 and the span its scan's first draft flagged.
`@monochromatic-dev/test-harness-core` builds `expect` on chai 6.3.0,
whose `config.truncateThreshold` defaults to 40
(`node_modules/.pnpm/chai@6.3.0/node_modules/chai/index.js:860`)
and is handed to loupe as the length a value is cut to (`index.js:928`);
the test package neither sets it nor exports chai's `config`,
so this package cannot change it.
The same display stands in B70,
quoting a failure of that entry's guard
(`expected [ …(9) ] to deeply equal [ …(8) ]`),
recorded and not acted on (M138).

The change:
`scan-findings.test-fixture.ts` (`expectNoFindings`) fails with every finding on a line of its own
before the comparison runs,
and the two scans fail through it.
Issue 610 records the cause in the test package.

Open:
by the lead's census,
31 of the 43 scans compare a whole list or record,
so each can fail with a count where what differs runs past 40 characters;
four of them
(`dead-test-declarations`,
`silent-test-cases`,
`temp-dirs-scratch` and `trailing-whitespace`)
assert as the two new scans did,
and the red commit says they move onto the fixture in a later commit.
Closed since by B339:
every scan's package-wide comparison goes through a helper that prints what differs,
and a scan holds them there.

Recurrence:
`mistake-prevention.md`,
"What a failing check prints".

## B339: forty-one scans compared what they found through the library, which printed it cut to a count

Red in `be46dce42`,
fixed in `9b028204d`;
issue 610.

Found on 2026-10-06 (UTC),
the item B338 left open.
A guard written for it,
`scan-direct-comparisons.unit.test.ts`,
found 64 comparisons in 41 scans
(62 `toEqual` and 2 `toStrictEqual`,
by the lead's count of its findings on the red tree)
that compared what a walk over the package found with an inventory or with nothing
through the assertion library alone,
which prints any value longer than 40 characters cut to its length.
B338's census by the lead had counted 31 scans.

The fix:
`scan-findings.test-fixture.ts` holds a helper for each shape of comparison,
each throwing with every difference whole on a line of its own
before it makes the scan's own comparison,
so a scan passes and fails on the inputs it did:
`expectNoFindings` for a list that should be empty (46 sites),
`expectFindingsAsListed` for a list against an inventory,
counting copies and printing both lists where only their order differs (12),
and `expectRecordAsListed` for a record,
key by key,
a list under a key diffed entry by entry (6).
`scan-findings.unit.test.ts` asserts each helper's whole failure text;
by the agent's measurement each of five mutants of the helpers fails the case meant for it,
and a finding planted in a scratch file failed four migrated scans of different shapes with its own text.
The guard,
among the source scans,
fails on a case of a scan the `source-scans` task names
that reads the package's files and compares through `toEqual` or `toStrictEqual` on `expect(...)`,
and on a scan none of whose cases reads them.

Open:
issue 610 stands,
since the cut is the library's and the test package neither sets its threshold nor exports it;
and the guard's header names what it cannot reach:
a reader bound to a constant,
a fixture reached through a namespace import,
a comparison inside a function the case calls,
`expect(...).not`
and a case built by a helper.
It reads names,
not scopes,
so a local name that shadows a reader errs toward a finding.

Recurrence:
`mistake-prevention.md`,
"What a failing check prints"
and "Searches and censuses that miss".

## B342: the map-write scan read a typed array's copy as a write over an unread key

The scan changed in `de6ff17`,
and the copy it had forced restored in `a7eca7f`.

Found on 2026-10-06 (UTC),
the scan's misreading B329 left open.
`blind-map-writes.unit.test.ts` listed every two-argument `set` whose function never read the key,
so a typed array's `set(source, offset)`,
which copies a source into the array and writes no key,
read as a map write,
and `credential-views.ts#decodeView` copied its offsets one element at a time to pass the scan.

The fix:
the scan resolves a plainly named receiver through the scopes that hold the write,
nearest first,
and leaves out a write whose nearest declaration makes a typed array
(a constructor,
`from` or `of`)
or annotates one;
a member,
a name a pattern binds without a type,
or a name it cannot follow stays a possible map write.
One fixture case holds each form of typed-array declaration.
Another holds a Map named like a typed array in another function and in an outer scope;
it passes on the old scan by design,
and by the agent's measurement fails under a resolution by name across the whole file.
`decodeView` copies with `set` over a `subarray` again,
which throws where a copy runs past the array,
where an index write past a typed array is dropped without a word.

Open to the owner's veto:
only a plainly named receiver is resolved,
so a typed array reached through a member is still reported.

Recurrence:
`mistake-prevention.md`,
"Searches and censuses that miss":
a scan reading a method call tells the receiver's kind from its declaration,
never from the method's name alone.

## B349: a listed reason's anchor was held to the call's top-level statement

Red in `d1e6af41c`,
fixed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass,
before its merge.
`promise-combinators-in-order.unit.test.ts`,
the scan that holds every Promise combinator in production code to a listed reason,
took a call's own statement from the top level of its function,
so inside an `if`,
a `for` or a `try`
any name anywhere in that block held the anchor.
Two calls in one `if` block,
the first resting on a name only the second uses,
passed,
and the `nextSettled` entry passed with `throwIfAborted` moved into the `try`,
where no rejection reaches it.
By the agent's measurement,
the scan as it stood missed the three defects planted inside an `if`,
a `for` and a `try`
in the case "HOLDS A NAMED ANCHOR TO THE INNERMOST STATEMENT MAKING THE CALL".

The fix:
the scan holds a name to the innermost statement making the call,
and looks for feeding code among the statements that come before the call's own in each enclosing block,
and in the head of a loop whose body holds the call.
A name written as `{ inHandler }` must stand in the handler of a `try` whose block holds the call,
and `nextSettled` now rests on that;
the cases for that form are coverage only,
since the old scan cannot express it.
By the agent's measurement,
seven edits to copies of the production source the scan reads each give exactly their finding,
one of them moving `throwIfAborted` out of `nextSettled`'s handler into the `try`,
and the unchanged source gives none.
At the merge the scan's package-wide case moved onto `expectNoFindings`,
which `scan-direct-comparisons.unit.test.ts` requires of a listed scan (B339),
and the scan joined the source-scans task.

Open to the owner's veto:
A loop's head counts as code feeding the call.
The readers of a settled result are taken from the first statement list at or around the call's statement.
A call in a class field initializer has the whole class declaration as its statement.
`Anchor` is `string | { inHandler }` rather than a new reason.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B350: a catch that handed its error on read as quiet

Red in `d1e6af41c`,
fixed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
A `catches-own` reason in the combinator scan held when any one catch under the call was quiet.
The handing check saw the error only as a direct argument,
so an alias,
a cast,
an array,
a nested object or a destructured parameter read as quiet,
and one quiet `try` beside one that throws held the reason.
By the agent's measurement,
the scan as it stood missed six of the plants of the case
"HOLDS A REASON RESTING ON A CATCH TO EVERY CATCH INSIDE THE CALL":
the alias,
the cast,
the array,
the object inside an argument,
the destructured parameter
and the quiet catch beside a throwing one.

The fix:
every catch under the call must be quiet,
with at least one present;
a destructured parameter is never quiet;
and any read of the binding counts as handing it on,
except a direct argument of `refusalText`,
`refusalOf` or `errorName`,
or the value of a property of an object literal that is such an argument.
By the agent's measurement,
each of three production edits gives one finding:
the error handed to `describeFailure` in `attribution-read.ts`,
the error cast inside `refusalOf` in `ledger-directory.ts`,
and a second,
throwing `try` in `cited-reference-lookup.ts`.

Open to the owner's veto:
A destructured catch parameter is never quiet,
where the review would treat one as quiet when none of its names is handed on;
by the agent's search,
production code holds none.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B351: the combinator scan did not find `Promise` reached without its name

Red in `d1e6af41c`,
fixed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
The scan read `Promise` by its identifier and off `globalThis`,
so `Promise.resolve().constructor.all`,
`globalThis.globalThis.Promise.all` and `ns.Promise.all` gave no finding,
and its doc did not list them as out of its reach.
By the agent's measurement,
the scan as it stood gave none of the 13 findings expected of the case
"FINDS PROMISE REACHED WITHOUT ITS NAME BEFORE THE COMBINATOR".

The fix:
a member named `Promise` or `constructor` read off any value is a finding,
as is a member named like a combinator read off anything but `Promise`,
called or not,
save `AbortSignal.any`;
so is each of those names destructured off a value other than `Promise` or the global object.
The module doc names what stays out of reach:
`eval` and `Function` text,
a computed key that is no string literal or a reflective read off a value other than `Promise` or the global object,
and another module's alias handed on or read by such a key.
By the agent's search,
production code reads no `.constructor` or `.Promise`,
and the only combinator-named member it reads off anything but `Promise` is `AbortSignal.any`,
in `stage-round.ts` and `synthetic-transport.ts`.

Open to the owner's veto:
`const { any, } = AbortSignal` is a finding while `AbortSignal.any` is allowed.
The combinator-name rule reads any such member off anything but `Promise`,
called or not.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B354: a count finding of the combinator scan named the wrong call

Red in `d1e6af41c`,
fixed in `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass.
An unlisted call written before a listed one took the entry by position,
and the finding named the old,
listed call's line.
By the agent's measurement,
the scan as it stood named only the listed call in the planted file of the case
"COUNTS EACH SITE'S CALLS AGAINST ITS ENTRIES".

The fix:
a site whose call count differs from its entries gives one finding naming every call's place,
and none of its entries is checked against a call it was not read against;
the calls of an unlisted key are still reported one finding each.

Open to the owner's veto:
The finding names every call of such a site,
where the review offered naming the first call no entry was read against;
the agent's reason is that the scan cannot know which call an entry was read against.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B355: forms the combinator scan reads were not planted in its fixture

Planted in `d1e6af41c`,
coverage only,
passing before and after `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass;
no defect.
`global`,
`globalThis['Promise']`,
the optional,
non-null and parenthesized callees,
a sequence callee,
a cast,
`.apply`,
`.bind` and a tagged template are now planted in the case "FINDS EVERY WAY",
each with its finding.
By the agent's measurement,
the scan before the change gave all 27 expected findings.

Recurrence:
`mistake-prevention.md`,
"Output decided by arrival order".

## B384: the coverage census counted every line as unrun for a suite that ran no test

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
from the code,
since the lead's brief named it with no source text (M142).
`runCoverageCensus` (`corpus-run/coverage-census-run.ts`) refused a suite that failed,
and counted one that exited clean having printed no PASS marker,
so a suite that ran no test gave a census claiming every line of the package was code no test ran.
By the agent's run against the base build,
the new case's scripted suite of no passes,
no failures and exit 0
printed a census of nothing,
its first line `coverage-census at abc123def: the unit suite, 0 passes`,
and wrote `census.json`.

The fix:
the census throws a `StatedRefusalError` when the suite printed no pass,
before it tallies or places anything:
"the suite printed no PASS marker,
so it ran no test;
its log is" the log's path,
and why such a census is not counted.
The case asserts the whole message,
no tally,
no placement and nothing printed.

The comment the fix wrote beside the refusal cited "ledger B313" for this defect,
an entry on a settled run file's row that was no object;
it cites this entry since `dc23e67cc1a4db79842412a448fd35c693ad433c`.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence" and "Tasks,
builds and bulk output".

## B430: three repeat modules were tested only through their caller

Red `cda53b4338`,
fixed in `566de2e511`.

`repeat-groups-bounds.ts`,
`repeat-groups-runs.ts` and `repeat-groups-wording.ts` sat on the own-test allowlist,
reached only through `repeatGroupsOf`,
the shape that hid the defects of B103.
They,
with `repeat-groups-refrain.ts` and `repeat-groups-order.ts` split off to meet the line limit,
now have unit tests of their own,
and the allowlist lines are gone.

Kept by the lead under the owner's instruction of 2026-10-11 to decide without asking:
an HTML comment is whitespace;
an opening mark that never closes quotes to the end of its paragraph;
invisible characters stay as wording;
overlapping slices place an occurrence in the lowest `sliceIndex`;
inline code is read on one line;
a lone carriage return is no line end;
emphasis marks stay as wording;
quotation marks no body paragraph of the corpus uses are not read;
the helpers are exported through `repeat-groups-barrel.ts`.
Changed by the lead's decision in a later commit:
a repeat that touches quoted speech anywhere stays out,
as the owner's answer of 2026-10-11 reads;
no two groups claim overlapping text;
the census fails when a positive control fails.

## B437: ten modules of the consistency pass were tested only through their callers

Red `f134107681`,
fixed in `a7a2a62c28`.

`artifact-two-lane-read-consistency-coherence.ts`,
`-fit.ts`,
`-occurrence.ts`,
`-parts.ts`,
`-verdict.ts`,
`pass-consistency-apply.ts`,
`-ask.ts`,
`-cache.ts`,
`-settle.ts` and `pass-settlement.ts`
stood on the own-test allowlist of `own-unit-tests.unit.test.ts`,
the shape that hid the defects of B103 and B430.
Each now has a test of its own and its allowlist line is gone.
Two fixtures serve them:
`consistency-group.test-fixture.ts` holds valid recorded groups changed one place per case,
and `consistency-refusal.test-fixture.ts` the whole text of an artifact refusal.
Every refusal case breaks one valid record one way and pins the whole message.
A mutation run over the ten modules left one equivalent mutant
(a located rendering carries no reason,
so the reason check throws either way)
and one choice no case can tell apart without the real clock
(the gather's grace is the round's window).
The cases found the defects of B438 to B440.

## B646: three tests that ask whether a text is blank were not listed with their reasons

Found on 2026-10-18 by the landing gate named g99b,
on the tree at `f9169093e6`,
which holds the work of n171 and n173.

Outcome written on 2026-10-18.
Fixed in `94ebefa1e3`.

The gate failed one file,
`src/blank-text-tests.unit.test.ts`,
the scan for tests of whether a text is blank (B40,
B128 in
[Audit ledger: text that shows nothing](audit-ledger-text-that-shows-nothing.md)).
It found three tests that the scan's list (`HELD_TESTS`) did not hold,
all in code that n171 added:
- two tests of `containsWords` in `src/consistency-span-agreement.ts`,
  which ask of an empty needle,
  since an empty text is found at every offset (B642 and B644);
- one test of `trailingNumberOf` in `src/consistency-label-form.ts`,
  which returns none when the digits it gathered are empty,
  which is the package's own sentinel (B639).

The rest of the suite passed,
2,482 cases,
lint had 0 warnings and 0 errors,
and the type check was clean.

Fixed in `94ebefa1e3`:
the two entries are listed,
with the reason each stands
(the empty needle,
and the sentinel).
The scan is the guard,
and it did its job at the gate.
That the scan was not among the checks that n171 reported before the landing is not shown by the sources:
no hand-back of n171 is among them,
and the message of `0199690299` says that its whole suite had not run on that tree.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B1035: scratch-parent.ts came from main with no unit test and six lint warnings

Found on 2026-10-29 in the tr-w376 tree,
where `origin/main` was merged into the branch.
`src/scratch-parent.ts` of `slice-reading` came from main in `c1367ce73b`,
added by another session without a `scratch-parent.unit.test.ts` beside it
and without a pass of the package's linter.
Two gates of this package refuse that:
the own-unit-tests scan of B102
(`src/own-unit-tests.unit.test.ts` in `source-scan`)
lists a module with no own test as unlisted,
and `slice-reading:lint:oxlint` treats warnings as failures.
The linter reported six warnings,
all in the one expression that picks the base directory and in the two calls after it:
five of `stylistic(argument-per-line)`
(lines 43 to 49 as main has them)
and one of `typescript(dot-notation)` on `process.env['XDG_CACHE_HOME']`.

The owner ruled on 2026-10-29:
"If scratch-parent.ts still doesn't have test and lint fixes,
you may now fix it and merge main into rebased."
Main still had no test for it,
so it was fixed on the branch side only.

Outcome written on 2026-10-29.
Red test in `c77504ec85`,
fix in `ad8b347e65`.

What changed:
- `scratchParent` takes `cwd`,
  `env` and `home` as one destructured object,
  each defaulting to the process's own
  (`process.cwd()`,
  `process.env`,
  `homedir()`),
  so the function can be asked about a throwaway working directory,
  environment and home,
  and no case reads or writes the real ones.
- The five calls that put several arguments on one line put one argument per line,
  which is what the rule asks,
  so that a changed argument is one changed line.
- `XDG_CACHE_HOME` is read as `env.XDG_CACHE_HOME`.
  `noPropertyAccessFromIndexSignature` is off in `package/config/typescript/tsconfig.options.json`,
  so the type check accepts the dot form and nothing is suppressed.
- The package index exports `scratchParent`,
  because the package's tests load the built entry
  and cannot reach a module the index leaves out.
- The environment is typed `Readonly<NodeJS.ProcessEnv>`,
  as the package types it elsewhere;
  the first draft,
  `Record<string, string | undefined>`,
  tripped `no-restricted-syntax/no-nullish-union`.

The four cases of `scratch-parent.unit.test.ts`
(each in a throwaway directory under the test scratch root):
inside a project the parent is `<node_modules>/.monochromatic/translation-repair-slice-reading`
and exists,
whatever the cache variable says
and without making the cache;
outside any project it is `<XDG_CACHE_HOME>/monochromatic-dev/translation-repair-slice-reading`,
and the home is not touched;
with the cache variable unset it is `<home>/.cache/monochromatic-dev/translation-repair-slice-reading`;
asked twice,
it returns the same path and does not fail.
Red:
before the change the file failed on import,
since the index did not export the function,
and the other 59 suites of the package passed.
Green:
60 suites,
lint 0 warnings and 0 errors,
type check clean,
and the B102 ratchet case of `source-scan` passes with no `ALLOWLIST` change.

Callers checked:
`rg 'scratchParent' package/translation-repair` finds one caller,
`src/image-ocr.ts`
(`await scratchParent()` with no argument),
which gets the process's own working directory,
environment and home as before.
The same expression stands in the `scratch-parent.ts` of other packages
(`bash-poke`,
among them);
those were not touched.

Recurrence:
a module taken in from another tree is held to the package's own gates before it lands,
the gates being its own test file and a clean `lint:oxlint`.
