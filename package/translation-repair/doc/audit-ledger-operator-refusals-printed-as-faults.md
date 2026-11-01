# Audit ledger: operator refusals printed as faults

Refusals of an operator's own mistake that printed as faults of the command,
and refusals that came after work an operator could not be told about.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B221: seven refusals of the operator's own mistake printed as faults of the command

Red in `63567b4f5`,
fixed in `843bd6ce0`.

Found on 2026-10-05 (UTC),
as the item ledger B183 left open and what the agent found beside it.
`reportingRefusals` (`corpus-run/cli-refusal.ts`) prints a `StatedRefusalError` as the command declining,
its sentence alone,
and anything else as a fault,
with the fault line,
the frames and the fault's exit code.
Seven classes carry an authored,
marked sentence about something the operator named or set,
and each printed as a fault:
`CensusBaselineError` (`corpus-run/coverage-census-input.ts`),
`SpendCeilingOverrideError` (`corpus-run/spend-ceiling.ts`),
`BedrockCreditOverrideError` (`bedrock-ledger.ts`),
`UnsafeSeedError` and `GradedSheetExistsError` (`corpus-run/sheet-path.ts`),
and `EmptyPoolError` and `MixedGenerationError` (`corpus-run/artifact-pool-refusal.ts`).

The fix:
each extends `StatedRefusalError`,
its message word for word as it was,
so the command prints the sentence alone and exits with the stated-refusal code.
`corpus-run/cli-refusal.unit.test.ts` holds one case for each class,
asserting the printed line and the exit code,
and `corpus-run/coverage-census-input.unit.test.ts` holds the baseline's class.
All eight failed on the lead's red tree,
the seven reading exit code 5 where 6 was expected.

The lead's part:
`StatedRefusalError` takes an optional `cause`,
kept on the error and never repeated in its message,
committed in `2b0bfad75` with a test file of its own;
`CensusBaselineError` hands its cause to it,
where the agent's patch set the property by hand after construction.

Left as they were:
`HardCapOverrideError` and `RunsDirectoryBusyError`,
the same shape,
in files another change held;
`SheetBindingError`,
whose message quotes values read off files and is unmarked,
so it needs an authored message first;
and `CoverageFileError`,
`SourceMapFileError` and `CorpusReadError`,
which the agent read as faults of the environment without running each command.
A follow-up is prepared for these six and was not launched when this was written.
It was launched and merged since:
B254 (`51ccd8680`) makes five of the six stated refusals,
and keeps `CoverageFileError` a fault with its reason said.
`LedgerShapeError`,
`BedrockLedgerShapeError` and `ArtifactParseError` are damaged data and stay faults,
which an existing case pins.

The whole suite ran on `843bd6ce0` in a side checkout,
which holds the fixes of B214 to B221:
1,615 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
a refusal's class says whether it is the operator's mistake or the command's fault,
and a case drives each such class through the printer.

## B254: five refusals an operator can mend printed as faults of the command

Fixed in `51ccd8680`,
with their cases.

Found on 2026-10-06 (UTC) by a housekeeping pass over items earlier entries left open.
`HardCapOverrideError` and `RunsDirectoryBusyError` extended `Error`,
so `reportingRefusals` printed "This is a fault in the command" with frames and exited 5,
for a ceiling the operator set and a second pass started beside a live one.
`SourceMapFileError` and `CorpusReadError` did the same,
for a build without a source map and a clone or pin without the commit,
each with a message that already told the operator what to do.
`SheetBindingError` took a finished message that quoted the seed,
the corpus pin and the draw digests read off the sheet and the manifest,
was unmarked,
and printed as a fault.

The fix:
the first four extend `StatedRefusalError`,
messages unchanged,
and print one line at exit 6;
`CorpusReadError` keeps its kind and its cause.
`CoverageFileError` stays a fault,
since the census writes that file itself.
`SheetBindingError` is rebuilt from a closed kind,
a closed sheet label and the two paths the operator named,
is marked,
and is a stated refusal that says to name a pair one draw wrote.
The two scorers pass the paths (`score-probe.ts`,
`score-agreement.ts`).
The case list `OPERATOR_MISTAKES` in `corpus-run/cli-refusal.unit.test.ts` holds each class's printed line and code,
and every case in `sheet-binding.unit.test.ts` asserts the whole message.

The agent's measurement,
not rerun by the lead:
the four classes printed at exit 5 with frames before and at exit 6 with one line after.
Measured by the lead:
38 named test files pass,
with the 37 source scans and a clean type check after a fresh build.
The whole suite ran on `51ccd8680` in a side checkout:
1,629 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 37 source scans passing.

Open to the owner's veto:
the four classes and `SheetBindingError` made stated refusals;
`CoverageFileError` kept a fault;
the binding refusal no longer printing the differing seed,
pin or digests,
which the operator reads in the two files it names.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".
This closes the six classes B221 left as they were:
five became stated refusals here,
and `CoverageFileError` stays a fault with its reason said.

## B278: three report commands faulted on an unreadable log and totalled a log named twice

Red in `5e116395a`,
fixed in `387af02b7`.

Found on 2026-10-06 (UTC) while moving the four report runners into modules a test can load
(T8,
the runner entry files).
`run-timing-report`,
`spend-report` and `meter-report` read each named log with `readFile`.
A log path that could not be read left the command as a fault,
exit 5,
naming nothing.
A log named twice was totalled twice by `run-timing-report` and `spend-report`,
so every call,
round and credit doubled.
The line count was a split on the newline,
so the closing newline every log a run wrote ends with counted as one more line.

The fix:
the three share `report-log-read.ts`.
`readLogTexts` refuses an unreadable log as a stated refusal,
exit 6,
naming the path as typed and the filesystem's code.
`refuseRepeatedLogs` refuses a path named twice,
the two paths compared once made absolute,
in the two commands that total;
`meter-report` keeps accepting one,
since it collapses exact repeats by design.
`linesOfLog` drops the empty piece after a closing newline.
The cases are in each command's suite "<command> as built",
which runs the built command in a child process over files the case writes,
and in the test files of the run modules.
Measured by the lead on the red tree:
`ledger-report` passes as it stands,
and the other three as-built files fail by the assertions named
(three cases,
two and one).

For B278 to B281,
measured by the lead on the fix tree:
35 named test files pass,
with the 40 source scans and a lint that finds nothing,
its type check included.
The agent's measurement,
not rerun by the lead:
a census over its 16 test files lists none of the four bundles among those no test loaded,
and no cold stretch in the entry files or the new modules beyond two `unreachable:` throws.
The whole suite first ran over this change on `53e25b458`,
with the two runner clusters that followed it:
1,821 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 40 source scans passing.

Open:
two paths that reach one log through a symbolic link are not seen as one.

Open to the owner's veto:
refusing a log named twice,
where totalling it twice was the behaviour.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal",
for the fault,
and "Readers of the package's own log lines".

## B284: the score commands' refusals reached the operator as faults with no words

Red in `a13f71ba8`,
fixed in `1088e72d9`.

Found on 2026-10-06 (UTC) while moving the five score runners into modules (T8).
Each of these ended at exit 5 with a stack and no words:

- `score-verify` and `score-probe` over a sheet and a manifest of different lengths;
- a manifest row with a wrong position;
- `score-attribution`,
  `score-probe` and `score-crosscheck` over a runs directory with no `artifacts` directory;
- a graded sheet that could not be read,
  absent or a directory,
  in `score-verify`,
  `score-agreement` and `score-probe`,
  which named no path.

The fix:
each is a stated refusal,
exit 6.
The length refusals name the counts;
the position refusal no longer quotes the file's value;
`requireArtifactsDir` (`score-artifacts-dir.ts`) names the directory and the filesystem's reason;
and `readSheetText` (`score-sheet-text.ts`) names the path,
the reason and what to name instead.
Measured by the lead on the red tree:
`score-probe` and `score-crosscheck` pass against the unmoved commands,
and `score-attribution`,
`score-verify` and `score-agreement` fail by assertion
(two cases,
two and one).

For B284 to B287,
measured by the lead on a tree that also held the files of `53e25b458`:
44 test files pass,
this cluster's 22 among them,
with the 40 source scans and a lint that finds nothing.
The tree of `1088e72d9` is that one less the files of `53e25b458` and its lines in two files both commits edit;
it was not built on its own,
and its message says so.
The agent's measurement,
not rerun by the lead:
a census over its 22 test files lists none of the five bundles among those no test loaded,
and no cold stretch in the entry files or new modules beyond the throws.
The whole suite's run over this change is the one B278 records.

Open,
to rule on:
`resolvePool` (`artifact-pool.ts`) reads the environment itself;
`score-verify` counts a sheet answer of `Duplicate` as unscored,
where `score-agreement` reports duplicates apart.

Open to the owner's veto:
`requireArtifactsDir` as a listing of its own before the gather's;
full paths in the two readers' refusals.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B290: two refusals of `cache-account-audit` printed as faults of the command

Fixed in `53e25b458`,
with its cases,
which the agent ran red in its worktree.

Found by the move B288 records.
The audit read every tracked file with no check that the file existed,
so a removal not yet committed ended a check made before a launch as a fault,
exit 5.
A cache version declared in a form the audit cannot read raised `CacheAccountReadError`,
whose message tells the operator how to write the declaration,
and it printed as a fault of the command with frames.

The fix:
the missing file is a stated refusal naming the file,
and `CacheAccountReadError` extends `StatedRefusalError`.
`cache-account-slice-report.ts` takes the runs directory where it looked it up,
and has a test file of its own.

Open,
to rule on:
`unaccountedCommits` and `printSliceCacheAccount` compare times to the second,
so a version set in the same second as a record reads as not set after that record;
telling the two apart needs ancestry and not seconds.

Open to the owner's veto:
`resolveRunsDir()` read before the audit,
where it was read after the version report.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B296: the samplers' and cards' refusals reached the operator as faults

Red in `17a66fa66` for the two draws,
fixed in `097b6ac40`,
with the listing cases run red by the agent in its worktree.

Found by the move B295 records.
Each of these ended at exit 5:

- a listing body that was not JSON,
  or a catalog listing of the wrong shape,
  with a stack;
- `draw-sample` and `damage-sample` over a runs directory with no `artifacts` directory;
- a final draw whose sample carries no recorded repair,
  which refused in clear words but as a fault.

The fix:
each is a stated refusal,
exit 6.
The listing refusals name the URL and quote neither the body nor the parse failure.
`listSettledNames` names the directory and the variable that sets it.
`UnmeasurableRepairError` (`sample-grading.ts`) extends `StatedRefusalError`,
the lead's choice of the two ways the agent set out,
so the draw calls `assertRepairMeasurable` with no catch around it.

Measured by the lead on the red tree for B296 and B297:
four of the six as-built test files pass against the unmoved commands,
and `draw-sample` and `damage-sample` fail in two cases each by assertion,
exit 5 or 0 where 6 is expected.
The agent's measurement,
not rerun by the lead:
a census over its 35 test files lists none of the six bundles among those no test loaded
and no cold stretch in the new modules;
two entry files each keep one cold character,
the continuation after an awaited call that a keyless run refuses before it returns
(`model-health.ts`,
`damage-sample.ts`).
The whole suite's run on `097b6ac40` is recorded in B295.

Open,
to rule on:
`resolvePool` (`artifact-pool.ts`) reads the environment and prints through `console.log` itself,
and `loadEntry` (`draw-entry-load.ts`) takes a reader but reads the run's corpus pin itself.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B302: `corpus-pass` resolved its cap and its spend ceiling before its refusal boundary

Red in `ea16d3b51`,
fixed in `9a68e3c78`.

Found on 2026-10-06 (UTC) while moving `corpus-pass` into modules a test can load (T8).
The cap for one entry and the spend ceiling were resolved in module constants,
before the refusal boundary ran.
An unreadable `TRANSLATION_REPAIR_HARD_CAP_MINUTES`,
or an unreadable spend ceiling,
exited 1 under a dump of bundled source,
although both refusals are stated ones.

The fix:
the entry reads the cap,
the ceiling and both windows inside its run callback,
in that order,
before the lock is taken.
The procedure is `corpus-pass-run.ts`,
which takes the environment,
the clock,
the tip,
the client factory,
the settlement and the corpus pin as required parameters and reads none of them itself.
The lead read the moved procedure against the one it replaces:
the steps run in the order they ran.

Measured by the lead on the red tree for B302 and B303:
`corpus-pass.unit.test.ts`,
eight cases that hold against the command as it stood,
prints one pass line and no fail line;
`corpus-pass-refusals.unit.test.ts` fails in its four cases and passes in none.
The agent's measurements,
not rerun by the lead:
a census over its eleven test files shows no cold stretch in the entry file,
in the sources it loads or in any new module,
and one invariant throw (`corpus-pass-order.ts`).
Not reachable without a key,
by the same agent:
a run to its end as built,
since the run client is built before the `--plan` return and before the queue;
wet meters over the real transport;
a real settlement of an entry.
The whole suite ran on `9a68e3c78` in a side checkout,
which holds the fixes of B302 to B304:
1,924 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 41 source scans passing.

Open,
seen by the agent and left:
`run-config.ts` resolves the corpus pin at load,
so an unreadable `TRANSLATION_REPAIR_CORPUS_COMMIT` exits 1 with a stack,
the same shape as the cap;
the `ONLY` line says the ordering is bypassed although the entries asked for are still ordered;
the detail of an `INCOMPLETE` line says to check that the clone exists for a page that is simply absent;
and the pair target on the `DONE` line is a literal 92.

Closed on 2026-10-10 (UTC) for the corpus pin by B394 (`d415f1ddac6b11f68193865985a69842ff9387d2`):
each entry reads the pin inside its refusal boundary,
and an unreadable value is a stated refusal at exit 6.
The other three were not reproduced at that round's base,
`791196bdc8458c54962cd834b0ac1a58851fdf78`,
since `cfac5b981` had already met them.
The `ONLY` line says the entries named are run in the pass's own order,
which the case
"RUNS THE ENTRIES --only NAMES IN THE PASS'S OWN ORDER,
resuming one holding cached slices ahead of one the command line and the corpus both name first"
(`corpus-run/corpus-pass-select.unit.test.ts`),
added by that round and passing at its base,
now pins.
The detail for an absent page says "check the path,
or pin a commit that has it",
and an unreadable clone prints no `INCOMPLETE` line,
since it is a stated refusal.
The `DONE` line counts `artifacts=N/M` over the complete pairs the walk found,
and no literal 92 is left in production code.
What those changes left open is under B394.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
a setting that can refuse is read inside the boundary that prints refusals,
never in a module constant.

## B303: the artifact guards' five refusals printed as faults of `corpus-pass`

Red in `ea16d3b51`,
fixed in `9a68e3c78`.

Found by the move B302 records.
An artifact nothing can place,
and four more refusals of the artifact guards,
each printed as a fault in the command,
at exit 5 under frames,
for a message that tells an operator what to do next.

The fix,
the lead's on the agent's work:
the five guard classes
(`UnplaceableArtifactError`,
`LegacyPipelineError`,
`GenerationDriftError`,
`SchemaGenerationError`,
`MislabelledArtifactError`)
extend `StatedRefusalError`.
The agent had caught the five in the pass and rethrown each inside a `StatedRefusalError`.
A refusal that is always an operator's is one by class,
as the samplers' is (B296),
and the catch,
its cause chain and two inventory entries it wanted are not in the commit.
`corpus-pass-guards.unit.test.ts` holds each refusal as its own class,
a stated refusal,
with its whole message.
The whole suite's run on `9a68e3c78` is recorded in B302.

Open to the owner's veto:
the guard classes as stated refusals.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
a refusal that is always the operator's is a stated refusal by class.

## B307: an empty corpus and a roster too narrow to judge printed as faults of the bench commands

Red in `cd78288d7` for the empty corpus,
fixed in `f8e99b84d`,
with the roster's case run red by the agent in its worktree.

Found by the move B305 records,
and for the empty corpus by the cluster of B299,
which saw it in a file it did not own.

- `sampleBenchSlices` (`bench-sample.ts`) threw a plain `Error` for a corpus that yields no slice,
  which `producer-calibrate` and `roster-bench` both printed as a fault at exit 5.
  It raises a `StatedRefusalError` naming how many entries the pin lists.
  The cases and the change are the lead's.
- `roster-bench` printed the refusal of a roster of fewer than two as a fault in the command,
  at exit 5 under frames.
  It states the refusal before the corpus is drawn.

`refuseUnjudgeableRoster` restates a `ProducerRosterError`,
a class that elsewhere is a fault and so is not stated by class.
The whole suite's run on `f8e99b84d` is recorded in B305.

Open to the owner's veto:
the narrow-roster refusal as a stated refusal and not an invariant;
the clock and the report writer as parameters of the roster bench.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B311: `coverage-census` run where no build stands printed a listing error as a fault

Red in `a9d447691`,
fixed in `8206fa5c6`.

Found on 2026-10-06 (UTC) by the agent moving the command.
Run from a directory with no `dist/final/node`
(the wrong working directory,
or no build yet),
the census listed the build directory with a bare read,
whose `ENOENT` went up uncaught,
and the command printed `refused by Error` with stack frames at exit 5,
the exit of a fault in the command.

The fix:
`requireCoverageBuild` (`corpus-run/coverage-census-build.ts`) lists through `namesIn`
and refuses as stated at exit 6,
naming the directory,
the filesystem code (`ENOENT`,
`ENOTDIR`)
and the task that builds with source maps.
The lead made the census's logger a required parameter of its run and of its build check,
owned by the entry file,
where the agent had made it optional with the module's own as the default.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
a directory the operator did not make is a stated refusal,
never an uncaught listing error.

## B382: a refused fidelity reference request printed a stack at exit 5

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
from the code,
since the lead's brief named it with no source text (M142).
`runFidelityProbe` (`corpus-run/judge-fidelity-probe-run.ts`) handed what the command line asked for
to `reviewedFidelityRequest`,
whose `FidelityReferenceError` reached `reportingRefusals` as a fault,
printed under frames at exit 5,
where the command's other refusals of what was typed print one stated line at exit 6.
The class is a fault elsewhere,
where a reference read from the clone fails its reviewed hashes.
By the agent's run against the base build,
three as-built cases of `fidelity-reference-cli.unit.test.ts` exit 5 where they expect 6,
with one stderr line and nothing on stdout:
an entry with no reviewed reference,
unreviewed context,
and the default damage families for an entry with no reviewed alteration;
and the case of `judge-fidelity-probe-run.unit.test.ts` on unreviewed context fails.

The fix:
`requestedSpecs` restates a `FidelityReferenceError` from the request check
as a `StatedRefusalError` carrying its message,
with the caught error as its cause,
and rethrows anything else;
its TSDoc says an empty checked-in manifest,
or one repeating an id,
is restated too,
though that refusal is not the operator's.
The site is listed in `FORWARDING_SITES` (`message-names-only.unit.test.ts`)
and held in `caught-value-text.unit.test.ts`,
both added before either scan flagged it.
A request a transport refused still logs `voice lost`,
and a trial whose judges are all lost logs `roster declined`,
as its siblings do.

The comment the fix wrote at `requestedSpecs` cited "ledger B313" for this defect,
an entry on a settled run file's row that was no object;
it cites this entry since `dc23e67cc1a4db79842412a448fd35c693ad433c`.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal".

## B391: the width probe built its client before anything a keyless operator could be told

Red in `cb553634188266b72019e60b6bd285d5746b0d32`,
fixed in `752be22cf8f4d875333bf1d54efdb90955d619a1`.

Found on 2026-10-10 (UTC) by the agent working the runner entry tails,
from the open items of the T8 passage "The runner and git cluster of the final census".
`editor-width-probe.ts` built its run client with `createRunClient()`
before it resolved the runs directory,
and `runEditorWidthProbe` (`corpus-run/editor-width-probe-run.ts`) took the built client,
so a process holding no key refused at the client
and never printed the count,
the draw,
or the refusal of an empty draw,
none of which needs a provider;
and the rest of `main` stayed cold in every keyless child.
By the agent's run against the unchanged production code,
three cases of `editor-width-probe-run.unit.test.ts` fail,
among them the new
"BUILDS THE CLIENT ONLY ONCE THE DRAW HOLDS A SLICE,
so a process holding no key prints the header and the draw and then refuses with what building the client said,
asking the panel nothing";
and two as-built cases of `editor-width-probe.unit.test.ts` fail:
the keyless refusal,
now expected after the header and the draw,
and
"REFUSES DRAW B OF A ONE-SLICE SAMPLE as stated and exits 6 before any client is built,
so the refusal that needs no provider is the one a keyless operator reads".
The as-built cases run over a throwaway runs directory,
lookup cache and corpus clone (`benchWorld`),
since the draw now comes before the client and would otherwise read the operator's own clone.
The red commit's message counts 9 of its 23 named test files failing against the unchanged production code,
four of them for want of the modules they import (B392);
the other 14 pass,
handing a repository the old signatures ignore (B393).

The fix:
`runEditorWidthProbe` takes `newClient`,
as `runRosterBench` does,
and calls it once,
after the draw is known to hold a slice and before `controlHolds` asks the panel;
the entry passes `createRunClient`.
By the agent's census over the round's 24 named test files,
the entry's one cold line is the closing `},);` of its awaited run,
which is wiring (the T8 passage).

Run on the merged tree of this round,
B394's and B398's:
lint clean,
49 scans passing,
and the suite at 2,047 passing with no failure,
14 more than B390's 2,033:
5 from this round's suites,
3 from B394's and 6 from B398's.
At the merge the lead resolved two TSDoc example lines,
in `corpus-run/draw-sample-pool.ts` and `corpus-run/draw-sample-run.ts`,
keeping both the pin of B394 and the repository of B393.

Open to the owner's veto:
one client,
built right before the control;
the alternatives are one client per stage,
or one built at the start as before.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B710: a rotation sweep that found failing cases ended in a stack trace

Red in `bab539ba71`,
fixed in `f127703f46`;
the bad-option refusals red in `02ce0ade0d`,
fixed in `92de2fce80`.

Named in the lead's brief of 2026-10-21 (UTC):
a task that is run to read its findings ended them in a thrown error.
The entry of `test:rotation-sweep` (`source-scan/src/rotation-sweep/rotation-sweep-cli.test-fixture.ts`)
threw a bare `Error` when `rotationSweepMain` returned false,
so Node printed the entry file's source line and frames after a report that already named the cases.
The sweep had run to its end and found what it exists to find;
nothing was broken,
and the operator was shown a fault.

The fix:
`rotationSweepMain` returns the exit code.
A clean sweep ends on `rotation sweep: clean` and returns 0.
A sweep that found cases writes the same report on standard output,
then a refusal on standard error that names each case,
its package,
the salts that fail it
(or that it is red with no shift)
and the command that reruns one salt,
and returns 6,
the code `reportingRefusals` gives a stated refusal.
`source-scan` does not depend on `glossary`,
where `StatedRefusalError` lives,
and `reportingRefusals` serves the declared `corpus-run` commands,
so the sweep takes the pattern's shape and not its class:
a found failure is a verdict,
and only an error out of the machinery
(the canary,
a package with no test file)
is thrown,
with its stack.
`rotation-sweep.unit.test.ts` drives `rotationSweepMain` over an injected sweep:
four of its five new cases fail on the red commit,
and the fifth,
a fault thrown unchanged with its stack,
passes there by design.

The exit code is assigned at the entry file's top.
The scan of process globals (`global-writes-sequenced.unit.test.ts`) refuses that in any fixture,
and the linter refuses `process.exit`,
so the scan now names the one fixture that is a process's own entry (`PROCESS_ENTRIES`),
allows its writes outside a case only,
fails if the listed file is gone,
and has a case showing the same write elsewhere still found.
That is a change to a guard,
made on purpose and listed for the lead's veto.

Measured on the fix commit by the real task on a throwaway always-failing test file in `translate-stage`,
removed afterwards:
`mise run //package/translation-repair/source-scan:test:rotation-sweep -- --package translate-stage --salt 1`
printed the report,
the refusal and no frame,
and exited 6;
without the file it ended on `rotation sweep: clean` and exited 0.

The command line is refused the same way.
`sweepRequestOf` threw a bare `Error` for an unknown option,
a salt past 9,
a salt that is no number and an option without a value,
so a mistyped option printed a stack.
It now throws `SweepOptionRefusalError`,
and `rotationSweepMain` writes its sentence alone to standard error,
naming the option typed and the valid forms,
starts no sweep and returns 6.
Any other error out of reading the command line is a fault and is thrown unchanged.
One case of `rotation-sweep.unit.test.ts` drives the four bad command lines
and failed on the red commit `02ce0ade0d`.
The new class writes its own sentence,
so the message inventory of `message-names-only.unit.test.ts` lists it as withheld with its reason,
which the fix commit missed and the commit after it supplied.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal":
an entry that can find something an operator is meant to fix returns a code,
and a case drives the found and the faulted paths through the entry's function.
