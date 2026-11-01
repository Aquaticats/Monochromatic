# Audit ledger: tests touching the real world

Tests that reached the real corpus,
real keys,
the file system,
child processes or process globals.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B79: tests that wrote a process global while module-test ran other cases beside them

Found by the census M100 queued.
module-test runs a suite's children sixteen at a time unless a suite says otherwise
(`package/test-harness/core/src/describe.ts`),
so a case that sets an environment variable,
diverts `console.log` or moves the working directory across an await
shares that value with every case running beside it,
and each restore puts back what it found,
which can be a sibling's value when cases finish out of order.
29 test files wrote a process global
(assignments,
deletes,
and `Reflect` or `Object` writes,
one of them split over two lines).
Read file by file,
16 already ran every writing case in a suite run one at a time,
several with a comment giving this reason;
the other 13,
which the scan committed red (`b9edc1542`) listed as 134 findings,
fell into these groups:

- three where restores finishing out of order could leave a sibling's value set:
  `candidate-ledger` and `run-config`'s `resolveRunsDir` suite on the runs directory,
  and `artifact-pool-conflict` on the pool variables,
  one of its cases setting them only after an await;
  `recordContest`,
  `resolveRunsDir` and `resolvePool` read their variable before their first await,
  so each case read its own value,
  and no later suite in those files reads the variable,
  so no assertion failed on it;
- `run-config`'s `readHeadSha` suite moved the working directory with `process.chdir` across an await while its sibling ran,
  which passed only because `readHeadSha` does not depend on the working directory,
  the very thing the moving case checks;
- `bench-report` never put the runs directory back nor removed its `mkdtemp` directory,
  harmless only because its case is the file's one;
- seven whose writes sit in windows with no await between setting and restoring,
  or in the only case of their suite:
  `transient-retry`,
  `artifact-pool-names`,
  `bench-report-groups`,
  `corpus-pin-override`,
  `pass-generation-guard`,
  `probe-telemetry-report` and `spend-ceiling`,
  safe as written and one await away from not being;
- `artifact-placement`,
  whose captures chain to each other and filter by file name,
  built for cases running at once;
  a restore out of order left a closed wrapper forwarding to the real reporter;
- `writer-grace-override`,
  whose two clearers sat in a table that nothing calls by name,
  so no reading of the text could follow them to a case.

`bench-sample-draw` sets `PATH` only in the text of a program a child process runs,
which is not this process,
and the scan reads it as text.

Fixed (`cf17f1dd0`):
every suite whose cases write a process global runs at `concurrency: 1`,
with a comment naming the global;
`bench-report`'s case restores the variable and removes its directory when it ends;
`clearDial` holds one static delete per dial in an `if` chain
(the lint configuration bans `switch`),
which also retires a `Record` keyed by text;
and `artifact-placement`'s capture no longer claims its suite runs cases at once.

The scan,
`global-writes-sequenced.unit.test.ts`,
reads every test and test fixture for an assignment or `delete` on a member chain off
`process`,
`console`,
`globalThis`,
`Date`,
`Math`,
`performance`,
`Intl`,
`JSON` or `crypto`;
`Reflect.deleteProperty`,
`Reflect.set`,
`Reflect.defineProperty`,
`Object.defineProperty` and `Object.assign` with such a first argument;
and `process.chdir` or `process.umask` with an argument.
It places a write in a function declared by name at every call of that name,
followed out to a fixpoint,
and a write in a method or callback where that function is made;
a write passes only in a case whose every enclosing suite,
out to the one awaited at the file's top,
runs at concurrency 1,
written there or inherited.
A write outside any case,
in a function no call by name reaches,
or in a suite not awaited at the top is reported.
A stub through the case's own sandbox (`ctx.sinon`) is no write form,
since module-test scopes it to the case.

Calls made here are open to veto:

- every writing case is sequenced,
  the seven safe today among them,
  so the rule reads off the text and still holds when an await is added;
- a suite sequenced inside one that is not counts as running beside others,
  since its parent dispatches it with its siblings;
- a writer no call by name reaches is reported rather than passed;
- console captures stay as they are in sequenced suites
  rather than moving to `ctx.sinon` stubs,
  which would scope them by construction
  but rewrite capture helpers that work,
  where sequencing is the package's own answer in 16 files already;
- no red case per file:
  whether a restore leaves a sibling's value depends on the order cases finish,
  so a check of the variable after the suite could pass by luck,
  and the scan's package case is the deterministic red.

Guard-offs on the fixed tree,
each restored with `git diff` printing nothing after:
`readHeadSha`'s suite back at concurrency 16 failed the package case with exactly its two `process.chdir` writes;
`Reflect.set` dropped from the write calls,
following no caller of a named helper,
and reading only the innermost suite
each failed the fixture case
(the `Reflect.set` finding missing,
then the write a helper chain reaches,
then the write in a sequenced suite inside a concurrent one,
12 findings for 13).

The full suite on `99cf575d7`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,501 PASS lines and no FAIL line.

Out of the scan's reach,
and named in its module note:
a write through an alias (`const env = process.env`),
a global shadowed by a local of the same name,
and a case made by a function the suite's text only calls.
Package source is not read:
production code that writes a process global is a different question,
and none was looked for here.

Recurrence:
a case that writes a process global runs in a suite run one at a time,
out to the top of its file,
or replaces a method through its own sandbox;
`global-writes-sequenced.unit.test.ts`,
which `source-scans` runs,
fails on any other.

## B108: test helpers left their temporary directories behind

Found 2026-10-01 (UTC) by a census of the package's test code,
fixed in the commit adding this entry.
A parse of the 947 test and fixture files found 108 calls that make a temporary directory,
in 70 files.
43 of them,
in 41 files,
could leave the directory on disk:
a helper returned the bare path with nothing to remove it,
built a disposer no caller bound,
removed the directory only when the case did not throw,
or wrapped such a helper,
and `artifact-generation-git.unit.test.ts` made two at module scope and never removed them.
The system's temporary directory held 35,060 entries when this entry was written,
with the leaking helpers' prefixes in the thousands
(`artifact-placement-` 4,294,
`artifact-generation-` 3,205,
`runs-lock-` 2,576).

Each now goes through `scratchDir` (`scratch-dir.test-fixture.ts`) bound with `await using`,
directly,
through a helper that returns its directory with a disposer delegating to `scratchDir`,
or through a helper that takes the directory a caller made that way;
`freshLedger` in the three `window-trial-*` test files no longer awaits anything and is synchronous.
Over one run of `artifact-placement.unit.test.ts`,
the file as it stood at `8cc9a3650` left 25 directories behind and this commit's left none;
over one run of all 41 changed files,
no leaking helper's prefix gained an entry,
and the directory's total stayed at 35,060.
All 41 changed files pass,
and a separate reviewer read the change and found nothing in them to fix.
The helpers that already removed their directories every time,
65 calls,
still make them themselves rather than through `scratchDir`.

The directories already left behind are not deleted here:
they sit in the system's temporary directory,
outside the repository,
and removing them is the owner's call.

What enforces it:
nothing yet.
A scan refusing any temporary directory in test code made other than through `scratchDir` waits on moving the 65 remaining calls onto it.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B117: test temp directories made outside `scratchDir`, or bound only after their setup ran

Queued by B108,
red in `9473905a0`,
fixed in `cdd36a98d` and `8ea5e7fa5`.
B108 fixed 43 leaking test directories and left the family unguarded.
`temp-dirs-scratch.unit.test.ts` now holds two rules over tests and fixtures:
no direct `mkdtemp` or `tmpdir` call outside `scratch-dir.test-fixture.ts`
and the 8 kept sites it names with why;
and every `scratchDir` call is the initializer of an `await using` declaration,
since a bare binding leaves a window between the directory's making and anything bound to remove it.
At `9473905a0` the first rule found 57 calls,
the 8 kept sites among them,
and the second 9.

A read-only agent's patch moved them:
`scratchDirWith` runs a setup against a fresh directory's path and removes the directory if the setup throws,
and two-directory helpers make the second inside a `try` that removes the first.
Before committing,
the patch needed:
its two rules folded into one top-level suite,
since two let a failure of the first stop the second (B99);
a lint-refused `String()` of untyped node fields replaced by the shared readers;
five setups returning `Record<PropertyKey, never>`,
which lint refuses,
replaced by `scratchDirPrepared` for setups that hand back nothing,
and by plain `scratchDir` bindings at the twelve callers of two helpers with no setup;
an environment variable written inside a setup,
a function only ever passed as a property,
which the global-writes scan cannot follow by name,
moved into `runsDirPointedAt`,
called by name and disposed before the directory;
23 TSDoc blocks left on return statements turned into line comments;
and an import its move orphaned removed.

The patch also undid work.
The agent's copy of `entry-pictures.unit.test.ts` predated B115,
and its diff was taken against the files as they stood when it finished,
so rebuilding its base from that diff gave the current file,
and the three-way merge copied the agent's file whole,
bringing back the local `sliceOf` B115 had moved to `content-slice-of.test-fixture.ts`.
The widened duplicate-body scan caught it;
the import is restored in `8ea5e7fa5`.
A line-level check of the nine temp-dir files any commit since B108 changed
(lines those commits added that the merge no longer held,
and lines they removed that it held again)
found no other.
The patch also wrote one setup into two files,
now `scratchDirOfEmptyRecords`.

The full suite at `8ea5e7fa5` left no new entry in `/tmp`
(`find /tmp -maxdepth 1 -newer` a marker touched before the run lists `/tmp` itself alone,
and `tmpdir()` is `/tmp` under `mise`);
`/tmp` still holds 35,065 top-level entries on 2026-10-02 (UTC),
most named by the test helpers B108 found leaking
(4,294 `artifact-placement-*`,
2,883 `artifact-generation-*`,
2,576 `runs-lock-*` the largest),
which this audit leaves for the owner to delete.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world"
and "Copies of shared code".

## B140: a logger's own report read as a line the package logged

Red in `7ecd53897`,
fixed in `4640ab83a` and `22afb6c89`.
Found on 2026-10-05 (UTC) when the whole suite on `4008eeb99` failed the `lookup-cache.unit.test.ts` case
"MISSES on a file cut short inside its record,
and WARNS that it does not parse,
naming the file and none of its text"
with `unreachable: the console sink wrote a line without its level and timestamp tags: logger internal error: sink verification failed for entry 3: Timed out after 5000ms: sink 3 verify`.
The same case had passed in the two whole-suite runs before,
on `971c8bec8` and `fa2813de4`.
`warnLinesDuring` (`console-warn-lines.test-fixture.ts`,
added in `8c7426a44`) diverts `console.warn` around a piece of work
and reads every line the console sink wrote.
The package logger is built on a process's first line and verifies its sinks then;
a sink it cannot verify it reports on `console.warn` itself,
untagged.
Where a test process logged its first line inside the divert,
that report landed in it,
and the fixture threw on a line without the sink's tags.
Entry 3 is the file sink,
and another session was running mutation and fuzz jobs on the machine at the time
(both from `7ecd53897`'s message).
The fixture's message called the state unreachable;
a loaded machine reached it.
Reproduced without load in a throwaway checkout of `4008eeb99` by making the log directory read-only:
the same case failed the same way,
the report naming `EACCES`.

The red case runs the fixture in a child process whose working directory holds a regular file
where the file sink wants its log directory,
so the sink's verify fails at once and the child's first line is logged inside the divert:
0 [PASS] and 2 [FAIL] lines at `7ecd53897`.

The fix:
`4640ab83a` flushes the built package's root logger before the fixture replaces `console.warn`,
so the logger verifies its sinks first
and a report of one it could not verify reaches the console it was written for,
outside the divert.
A line without the console sink's tags still makes the fixture throw,
since it is no line the package logged,
and the message no longer calls that unreachable,
because the logger can still report a sink failing during the work.
A test process holds a second logger,
module-logger's default,
which the test framework writes through,
while the build bundles its own copy;
`22afb6c89` flushes that one too.
Which of the two reported in the run on `4008eeb99` is not known,
and a verify of the second running past its limit into an open divert was not reproduced:
an attempt in a child process did not land the report inside the divert.
Flushing it before the divert waits its verification out whenever it started,
so the order no longer rests on timing.
These facts are from the three commits' messages;
the whole suite had not run on `22afb6c89` when its message was written.
It ran afterwards in a fresh checkout of `22afb6c89`:
1585 [PASS] and 0 [FAIL] lines,
the lint clean and 35 scans passing.

Open:
the logger's report itself keeps coming.
`logger internal error: sink verification failed for entry 3: Timed out after 5000ms`
stands 113 times in the suite and scan logs of the five whole-suite runs from `cf50117c5` to `94de8258e`,
each run made in a side checkout while other agents worked on the same machine,
and every one of those runs passed.
Why the file sink's verification runs past its limit on a loaded machine was not looked into.

Recurrence:
`mistake-prevention.md`,
"Globals a case replaces".

## B185: the picture sources defaulted to the corpus and the real programs

Fixed in `17646f215`,
with no red case:
the change makes a parameter required,
and the type check named the one call that left it out (the fixing agent's run).

Found on 2026-10-05 (UTC)
by the agent that replaced the `ocrTool` seam (ledger M113),
among the seams it saw and did not fix.
`readPassVisualEvidence` (`corpus-run/pass-visual-evidence.ts`) defaulted `pictureSources` to `RUN_PICTURE_SOURCES`,
and `readSeatedPictures` (`corpus-run/pass-seated-pictures.ts`) took it optionally,
so a test that left it out read pictures from the pinned corpus and ran the real programs.
One did:
the case of `corpus-run/pass-entry.unit.test.ts` that stops an entry on a picture with no reviewed evidence
reached `readCorpusBytes` against the pinned checkout for an invented entry,
and with a refusing stand-in handed in it ended `resumable-failure` where it had ended `stopped`
(the fixing agent's runs).

The fix:
`pictureSources` is required in `readPassVisualEvidence`,
`readSeatedPictures`,
`createPassPictureReader`,
`runEntryPipeline` and `settleEntry`,
as `outsideReads` is;
`RUN_PICTURE_SOURCES` is named once,
where `corpus-pass.ts` calls `settleEntry`.
Tests pass `NO_PICTURE_SOURCES` (`corpus-run/pass-picture-sources.test-fixture.ts`),
whose gatherer and reader reject with a message naming the seam,
and the case that stops an entry hands in a gatherer that finds no bytes,
which is what it is about.

Open to the owner's veto:
the seam runs through `settleEntry`,
which 31 lines of its test file now hand it,
where the other way folds it into `PassOutsideReads`.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B330: eight log lines that print a provider's failure had no case through their own line

Red in `03ec6a4fc`,
fixed in `9a32c49`.

Found on 2026-10-06 (UTC) by the census of B329.
The lines of `image-reading-pair.ts`,
`corpus-run/required-providers.ts`,
`corpus-run/model-health-report.ts`,
`corpus-run/coverage-probe-run.ts`,
`corpus-run/window-trial-pick.ts`
and the abort warnings of `translate-slice-attempt.ts`,
`repair-refine-step.ts` and `repair-slice-buy.ts`
had no case reaching them with a status failure the real client raised over the real transport.
By the agent's reading,
no line was wrong:
each names the status,
the provider or model and the provider's own words,
and through the masked transport holds no credential.
The abort warnings are reached only where the abort fires while the provider refuses,
since `attemptStageCall` rethrows an error under an aborted signal unchanged.

The fix:
`provider-status-failure.test-fixture.ts` builds the failure
through the real `createSyntheticClient` over the real `fetchTransport`,
against a stubbed `fetch` whose invented provider echoes the key three ways,
and one case per line holds the whole line;
their red is the mask's gap of B329 showing through them.
`buyRepairSlice` is exported through `repair-chunk-barrel.ts` for its first unit test file,
and the own-unit-tests allow-list lost its line for `repair-slice-buy.ts`.
The suites that stub `fetch` for the process run one case at a time.
The lead's change:
the fixture's count of base64 characters its prefix fills alone (5) is derived from the prefix and named group sizes,
where the agent's file wrote the number.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B332: the real corpus pin stood as fixture data in nine test files

Fixed in `e1c84094e`.

Found by the same cross-check,
which named one file (`corpus-run/editor-standing-read.unit.test.ts`);
the lead's search found nine,
writing the pin in full or cut to seven or nine characters,
five of them in the seven-character form.
Each case records,
prints,
parses or abbreviates the id,
and none reaches code that checks it against the pin,
so the real commit stood where any object id does.
The owner's ruling of 2026-10-05 (UTC) that the corpus is public covers corpus text,
and by the lead's reading of it unit fixtures stay invented.

The fix:
each of the nine writes an invented id (`feedface` repeated,
or cut as the case cuts it),
and every one passes with it.
Comments that cite the pin as where a count was measured keep it,
since that is what they record,
and so do the production examples that show the package's own pin.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B334: a read fault a case made with a file mode held for every user but the superuser

Fixed in `e1c84094e`.

Found by the same cross-check.
The slice cache's resume case (`corpus-run/slice-cache-namespace-resume.unit.test.ts`)
made a file unreadable with mode `000`,
which a superuser reads anyway,
so the case failed whenever the suite ran as root.

The fix:
the file is made one byte past the largest Node's `readFile` reads,
2 GiB,
held sparse so it takes no disk;
Node refuses it from its size before reading,
whoever runs the suite,
and the case asserts that refusal's whole text,
`RangeError [ERR_FS_FILE_TOO_LARGE]: File size (2147483648) is greater than 2 GiB`
with the size written from the case's own constant.
The lead measured the refusal's text on Node 26.10.0 with a sparse file of that size.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B386: three as-built suites started their command through their own path

Changed in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
passing before and after `9d558da592162f9765fb03a2280af9477ada147d`;
nothing in it can be red.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers.
`verify-published.unit.test.ts`,
`editor-standing-read.unit.test.ts` and `sink-names-only.unit.test.ts`
started their built commands through `runKeyless` with a path each built itself
(`VERIFIER`,
`STANDING_ENTRY` and a `command` and `args` pair),
not through the shared starter,
`runBuiltCommand`,
which adds the `TRANSLATION_REPAIR_STARTED_BY` marker the built command's guard reads;
the working directory,
`PACKAGE_ROOT`,
is the same.

The change:
the three start through `runBuiltCommand`,
and `VERIFIER` and `STANDING_ENTRY` are gone.
By the agent's run against the base build,
the three pass with their case names as at the base,
and pass after the fix.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B801: `remark-math` could not be imported from source, because the override that dropped `katex` left its import dangling

Fixed in `47b4634eeb9afabc1aaa8401a99f2e4dc7432a1b` (the shim) and `1a61a85cfe2128224549a15e1fb7266bc02b48d0` (the override);
red in the commit before them.

Found on 2026-10-21 (UTC) when a test run from a worktree outside the scratch area's directory tree
failed at load in every test file that reaches `document-structure/src/parse-mdx.ts`.
`parse-mdx.ts` imports `remark-math`,
which imports the root of `micromark-extension-math`.
That package exports only `"."`,
and its `index.js` re-exports `mathHtml` from `lib/html.js`,
which runs `import katex from 'katex'` and reads `katex.renderToString` when the module loads.
`c92efa816c` overrode `'micromark-extension-math>katex': '-'` for GHSA-238p-pmpm-9mq7,
so `katex` was not installed,
and the import threw `ERR_MODULE_NOT_FOUND: Cannot find package 'katex'`.
Built bundles were not affected:
rolldown drops `lib/html.js` because the package declares `sideEffects: false`,
which is why the corpus runs kept working.

The change:
`package/pnpm-override/shim-katex` resolves as `katex`,
exposes `renderToString` and the other members of katex's surface as functions,
and throws `KatexNotInstalledError` from every call or construction;
the override points at it.
By the agent's run,
one `document-structure` unit test file failed at load with the `ERR_MODULE_NOT_FOUND` above before the override,
and that file and the whole `document-structure` unit task pass after it;
the shim's own cases include loading `remark-math`,
which failed the same way before the override.
The built `corpus-pass` bundle has the same pipeline digest before and after.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Tests touching the real world";
`doc/troubleshooting/dependencies.md`,
"`katex` (dropped for GHSA-238p-pmpm-9mq7;
substituted via a throwing workspace shim)".

## B802: unit tests passed only because a parent directory of the worktree held a `node_modules`

Status:
the cause is found and the rule is recorded;
a search of `package/translation-repair` and `package/dev-script` found no guard
that refuses a run whose parent directories hold a `node_modules`.

Found on 2026-10-21 (UTC) while tracing B801.
Every scratch worktree sat under one directory that held its own `node_modules`,
left by another session's npm install,
and that install held a `katex`.
Node resolves a bare import by walking up the directory tree,
so `micromark-extension-math/lib/html.js` found that `katex` although the workspace had none.
The install was rewritten at 2026-10-21 17:10:03 UTC without `katex`,
and from then on 1,172 test files of 26 packages failed at load in worktrees under that directory.
`c92efa816c` had checked its removal of `katex` the same way,
so its note that the unit suite passed,
math cases included,
rested on a directory outside the repository.

The fix of B801 added no guard against this.
It was made in a worktree where no parent directory holds a `node_modules`,
which is what showed the defect as it is.

The first whole gate run in such a worktree,
on 2026-10-21 (UTC) after the shim,
found a second site:
`consistency/src/consistency-shown-kinds.unit.test.ts` walked to the parser's list of HTML block names
by resolving `remark-parse` from its own file,
and `consistency` declares no `remark-parse`,
so three cases failed with `Cannot find module 'remark-parse'`.
The walk now starts at the document structure package,
which `consistency` depends on and which declares `remark-parse`.
Every other case of the suite passed there,
so no third site was found by that run.

Recurrence:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Tests touching the real world".
