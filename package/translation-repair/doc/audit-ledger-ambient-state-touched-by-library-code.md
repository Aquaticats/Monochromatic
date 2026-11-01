# Audit ledger: ambient state touched by library code

Library code that read or wrote the process environment,
the exit code or a module-load default,
where a caller could not supply or see it.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B379: the ledger and meter reports wrote the exit code on the caller's process

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
among the observations the lead's brief handed on.
`reportLedger` (`corpus-run/ledger-report-run.ts`) and `reportMeters` (`corpus-run/meter-report-run.ts`)
wrote `process.exitCode` inside the procedure,
so a caller importing either had its own process status changed.
By the agent's census of the 42 distinct entries `src/build-entries.ts` (moved on 2026-10-18 to
`package/translation-repair/coverage-census/src/build-entries.ts`) lists
(it names `coverage-census.ts` twice),
37 set no exit code themselves,
leaving 4,
5 and 6 to `reportingRefusals` (`corpus-run/cli-refusal.ts`),
and after the fix five set one:
`ledger-report`,
`meter-report`,
`model-health` and `verify-published` from what their procedure returns,
and `editor-standing-read` inside its own `reportStandings`.
By the agent's search with `git log -S 'process.exitCode'` over `src/corpus-run` (moved on 2026-10-18 to
`package/translation-repair/*/src/corpus-run`),
23 commits touched such a write,
and two removed one,
each on purpose:
`97aeee2ca` took `ASKED_WITHOUT_A_SEAT` out of `ledger-report.ts`,
a case the command-line reader now refuses,
and `354f6bae7` took `PUBLISHED_TREE_DISAGREES` out of verify-published,
which exits 0 with its findings;
the moves of `387af02b7` and `8206fa5c6` kept every code.
By the agent's run against the base build,
seven cases of `ledger-report-run.unit.test.ts` and three of `meter-report-run.unit.test.ts` fail,
each expecting the code returned and none left on the process,
for example `expected { lines: [ …(2) ], …(2) } to deeply equal { lines: [ …(2) ], returned: 1, …(1) }`.

The fix:
`reportLedger` returns `NOTHING_TO_READ` (1),
`LEDGER_INCOMPLETE` (2) or the new `LEDGER_READ` (0),
and `reportMeters` returns `NOTHING_RECORDED` (1) or the new `REPORTED` (0);
each entry sets `process.exitCode` from what its procedure returns,
and `meter-report.ts` gains a `main`.
`captureCodedReport` (`corpus-run/report-run-capture.test-fixture.ts`) reads what a report returned
beside what it left on the process,
and the cases expect the code returned and the process code unset.

The red commit fails 26 of the 68 named test files against `b7244804f`'s production code,
one of them without running a case,
since it imports `readPoolPolicy`,
which the fix adds (B380);
the other 42 pass either way.
Run on the merged tree before the fix commit:
lint clean,
49 scans passing,
and the suite at 2,032 passing with no failure,
two PASS lines more than B378's 2,030,
from the two suites `artifact-pool-conflict.unit.test.ts` gained.

Open:
the logger package's own sink verification bound of 5,000 ms timed out 12 times in the agent's suite run,
without failing a case,
as it did in the run B378 records;
the agent ran no suite at the base to compare.

Open to the owner's veto:
`editor-standing-read` still sets `NOTHING_RECORDED` on the process itself,
since its procedure lives inside the entry file;
the alternative is to split it into a `-run.ts` procedure that returns the code,
as its siblings are.

Closed on 2026-10-10 (UTC) by B392 (`752be22cf8f4d875333bf1d54efdb90955d619a1`),
which took the alternative:
`reportStandings` (`corpus-run/editor-standing-read-run.ts`) returns `NOTHING_RECORDED` (1) or `STANDINGS_READ` (0),
and the entry sets `process.exitCode` from what it returns.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B380: the artifact pool read its commit policy from the environment inside library code

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
in its census of direct `process.env` reads in production code,
78 at the base.
`resolvePool` (`corpus-run/artifact-pool.ts`) read `TRANSLATION_REPAIR_REQUIRED_COMMIT`
and `TRANSLATION_REPAIR_POOL_ALL` itself,
under every reader that pools settled artifacts,
so a case chose its pool by writing the process's environment,
and a shell that exported either variable chose the pool a library caller measured.
The contradictory pair,
a required commit with `TRANSLATION_REPAIR_POOL_ALL=yes`,
was a plain `Error`,
which the commands printed as a fault at exit 5 under frames,
for a sentence telling the operator which variable to unset.
By the agent's run against the base build,
the as-built conflict cases of `score-attribution`,
`score-crosscheck`,
`score-probe`,
`draw-sample` and `damage-sample` exit 5 where they expect 6,
and `artifact-pool-conflict.unit.test.ts` fails to load,
with a `SyntaxError` for the missing export `readPoolPolicy`.

The fix:
`readPoolPolicy({ env })` returns a `PoolPolicy`,
its `requiredCommit` and `poolAll` read as before (`?? ''` and `=== 'yes'`);
`resolvePool` takes the policy as a required value
and refuses the pair as a `StatedRefusalError` with the same text;
and the readers over it
(`gatherAttributionEntries`,
`gatherProbeReadings`,
`collectShippedRegions`,
`readDrawPool` and the five procedures)
take the policy too.
The five pooling entries read it from `process.env` and hand it down.
By the agent's census after the fix,
82 direct reads stand:
the read in `resolvePool` is gone,
and five are new,
each in one of those entries' own functions.
`pool-policy.test-fixture.ts` holds the policy of an invoker who set neither variable,
and the pair as a built command's child is handed it and as the pool refuses it;
`artifact-pool-names.unit.test.ts` no longer clears the process's variables around its case.

Open to the owner's veto:
The contradictory pair is a stated refusal at exit 6,
where it was a fault at exit 5;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world" and "Two layers reading one refusal".

## B381: three runs-directory readers read the environment, so relabel and verify ignored the directory handed them

Red in `3fa45b1377073e74042d66fdb033f2533abbb7cc`,
fixed in `9d558da592162f9765fb03a2280af9477ada147d`.

Found on 2026-10-10 (UTC) by the agent working the runner leftovers,
in its census of readers that reach the environment through a function.
`readArtifactRecords` (`corpus-run/probe-relabel-artifact.ts`),
`writeWidthReport` (`corpus-run/editor-width-report.ts`)
and `writeBenchReport` (`corpus-run/bench-report.ts`)
each called `resolveRunsDir()`,
which reads `TRANSLATION_REPAIR_RUNS_DIR`.
`runProbeRelabel` and `runProbeVerify` are handed a runs directory,
while their gatherers,
`gatherRelabelCases` and `gatherControlCases`,
read through `readArtifactRecords`,
so they read whatever the variable named,
not the directory their procedure was handed.
By the agent's run against a build holding these readers' base code,
each case pointing the variable elsewhere and handing the filled directory in:
`readArtifactRecords` fails with `RunJsonUnreadableError: could not read whiskers.json as JSON (ENOENT)`;
`writeWidthReport` writes outside the directory it is handed;
`writeBenchReport` leaves no `roster-bench/rows.json` there (ENOENT);
`runProbeRelabel` and `runProbeVerify` fail with `expected [ { …(2) } ] to deeply equal [ { …(3) } ]`;
and the relabel case,
control and gathering suites fail with ENOENT.
In that run the base build's width report cases tried to write under the worktree's default runs directory,
`node_modules/.monochromatic/translation-repair-runs`,
and failed with ENOENT;
by the agent's uncapped listing,
nothing was written there.

The fix:
the three readers take `runsDir`,
as do `gatherRelabelCases` and `gatherControlCases`,
and `runProbeRelabel` and `runProbeVerify` hand them their own directory;
`runRosterBench`,
`runEditorWidthProbe` and `runWidthDraw` take `runsDir`,
and the entries `roster-bench.ts` and `editor-width-probe.ts` pass `runsDir: await resolveRunsDir()`.
By the agent's census after the fix,
every production caller of `resolveRunsDir` is an entry or is handed the function
(`coverage-probe-run.ts` takes it as a parameter).
`runsDirPointedAt` (`corpus-run/runs-dir-pointed.test-fixture.ts`) stays for the three cases
showing that a directory handed in wins over the variable,
and the gathering,
bench report,
relabel case and relabel control tests no longer write the variable.

Not shown red:
`roster-bench-run.unit.test.ts`'s changed expectation,
which records the runs directory each report write is handed,
never ran against the base,
and no base build was left to run it on;
the red commit's message does not say whether that file is among the 26 it shows failing.
The width loop and width run tests pass `runsDir` to fakes that do not record it,
so only the type checker holds that pass-through.

Open,
a defect left to the owner:
`run-config.ts` reads the corpus pin at module load
(`RUN_CORPUS_PIN_SETTING`,
through `readCorpusPinSetting`,
which reads `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` and `TRANSLATION_REPAIR_CORPUS_COMMIT`),
75 production files import that module by the agent's count,
and an unreadable value refuses at import,
exiting 1 with a stack outside `reportingRefusals`.
B302 records it as still open.

Open to the owner's veto:
`roster-bench` and `editor-width-probe` resolve the runs directory once at start,
not at each report write;
the variable means the same,
but with it unset a git failure in `resolveWorktreeRoot` now refuses before any quota is spent,
not after the first row;
the alternative is to keep the resolution lazy by handing the procedures a resolver.
The pipeline-wide reads stay where they are,
since moving each would change the signature of every stage on its way:
the straggler grace,
read by `resolveStragglerGraceMs` in the default of every round (`stage-round.ts`),
where the alternative is to hand the grace from the entry into each stage;
the writer grace,
read through `readWriterGrace` inside the consolidate,
translate,
refine and repair editor stages,
where the alternative is to thread it from the entry;
the slice overlap,
read by `readPassOverlap` inside `settleEntry` for each entry,
where the alternative is to read it once at the corpus-pass entry;
the contest records,
whose `recordContest` (`candidate-ledger.ts`) reads `TRANSLATION_REPAIR_RUNS_DIR` under `selectBestCandidate`,
where the alternative is to thread a runs directory through candidate selection;
the calibration grace,
whose `adoptCalibrationGrace` (`grace-override.ts`) reads and writes `process.env` for `editor-calibrate`,
where the alternative is to return the grace and pass it down;
and the outside reads,
`RUN_OUTSIDE_READS` (`pass-outside-reads.ts`) handing the live `process.env` on at module load,
where the alternative is to build them in the corpus-pass entry.

Closed on 2026-10-10 (UTC):
the corpus pin read at load,
and the outside reads,
by B394 (`d415f1ddac6b11f68193865985a69842ff9387d2`);
and the straggler grace,
the writer grace,
the slice overlap,
the contest records and the calibration grace by B398 to B400 (`477bbd6601509ac08d867e858af680a59c4f91eb`),
each by the alternative this entry named.
The resolution of the runs directory once at start still stands for `roster-bench` and `editor-width-probe`;
since B391 the width probe resolves it in `main`,
before its run builds the client.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B393: the artifact pool asked the package's own checkout for ancestry

Red in `cb553634188266b72019e60b6bd285d5746b0d32`,
fixed in `752be22cf8f4d875333bf1d54efdb90955d619a1`.

Found on 2026-10-10 (UTC) by the agent working the runner entry tails,
from the veto on the artifact pool case in the T8 passage "The runner and git cluster of the final census".
`resolveCommit` and `tipContains` (`corpus-run/artifact-generation.ts`)
defaulted `repository` to the directory of the built bundle,
and `resolvePool` (`corpus-run/artifact-pool.ts`) handed them none,
so the case
"RESOLVES A REQUIRED COMMIT NAMED BY A REVISION ..."
read this checkout's `HEAD` and `HEAD~1`
and failed on a checkout of one commit,
and no caller of `resolvePool` could ask another repository.
The rewritten case builds a throwaway repository of two commits with `makeNamingArchive`;
by the agent's run against the unchanged production code,
it failed with "Cannot place pipeline commit ... against required commit ...".

The fix:
`repository` is a required value on `resolveCommit`,
`tipContains`,
`selectEligible`,
`resolvePool`,
`gatherAttributionEntries`,
`gatherProbeReadings`,
`collectShippedRegions`,
`readDrawPool` and the five pooling procedures
(`printAttribution`,
`printCrosscheck`,
`printProbeScore`,
`drawGradingSample` and `sampleDamage`);
the five entries pass `import.meta.dirname`,
and every built bundle sits flat in `dist/final/node`,
so they name the checkout the default named.
`selectEligible` asks the repository nothing when no commit is required,
and cases with no required commit pass `UNASKED_REPOSITORY` (`corpus-run/pool-policy.test-fixture.ts`),
a path no directory stands at.

Open to the owner's veto:
`repository` is required even where no commit is required;
the report names no alternative.
Four cases of `corpus-run/artifact-eligible.unit.test.ts` still read this checkout's root commit and `HEAD`,
through `gitBounds`,
and hand the checkout in as `THIS_CHECKOUT`;
the alternative is a throwaway repository.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B394: the corpus pin was resolved at module load

Red in `b42882f3a74c02b8e77549e8075a18ccbad0a221`,
fixed in `d415f1ddac6b11f68193865985a69842ff9387d2`.

Found on 2026-10-10 (UTC) by the agent reading the corpus pin at each entry,
the open item B302 and B381 left.
`run-config.ts` held `RUN_CORPUS_PIN_SETTING`,
built by `readCorpusPinSetting` while the module loaded,
and `RUN_CORPUS_PIN`,
its pin;
and `pass-outside-reads.ts` built `RUN_OUTSIDE_READS` from them at load,
over the live `process.env`.
So an unreadable `TRANSLATION_REPAIR_CORPUS_COMMIT` or `TRANSLATION_REPAIR_CORPUS_CLONE_DIR`
exited 1 under bundled source before `reportingRefusals` ran,
in 34 of the 43 built bundles,
`index.mjs` among them,
whether or not the command reads the corpus.
Seventeen entries read the pin in code besides `corpus-pass`:
`coverage-control-probe`,
`coverage-probe`,
`displacement-probe` (its clone alone),
`draw-sample`,
`editor-calibrate`,
`editor-width-probe`,
`judge-fidelity-probe`,
`probe-relabel`,
`probe-verify`,
`producer-calibrate`,
`recall-benchmark`,
`roster-bench`,
`sentinel-probe`,
`slice-census`,
`translate-probe`,
`window-trial-probe`,
and `rendering-audit-settled` through its arguments.
Seven library modules read the constant in code:
`draw-entry-load`,
`pass-decline`,
`pass-entry-artifact`,
`pass-seated-pictures`,
`pass-outside-reads`,
`rendering-audit-settled-args` and `window-trial-protocol`.
By the agent's run against the base build,
each of the 17 commands gave code 1,
nothing on stdout and stderr beginning with the bundled source,
where `corpus-pin-entries.unit.test.ts` holds each at exit 6 with its whole stated line and nothing on stdout;
the same file holds a command that never reads the corpus,
`ledger-report`,
running past the setting.
The red commit's message counts 10 of its 15 named test files failing against the unchanged production code;
the other five pass,
the `--only` order case among them (B302's closing note),
and the rest hand a pin the old signatures ignore.

The fix:
`readCorpusPinSetting({ env, fallback })` reads the environment it is handed,
as `readPoolPolicy({ env })` does since B380;
`readRunCorpusPinSetting({ env })` replaces both constants,
its fallback a private constant of `run-config.ts`,
the same clone path and `CORPUS_COMMIT_SHA`.
Each of the 18 entries calls it inside its run callback,
at the callback's start by the agent's account,
except `corpus-pass`,
where it follows the cap,
the ceiling,
both windows and the overlap (B400)
and comes before the runs directory and the lock.
The library takes the pin as a required value:
`settleEntry`,
`runEntryPipeline`,
`loadEntry`,
`readDrawPool`,
`protocolDigest({ headSha, pin })` and `readSeatedPictures` take `pin`;
`readAuditArguments` and `runSettledAudit` take `runCloneDir`;
and `recordEntryDecline`,
`settledEntryArtifact` and `settledPageArtifact` take `corpusSha`.
`RUN_OUTSIDE_READS` is gone,
and `corpus-pass.ts` and `rendering-audit-settled.ts` call `outsideReadsFrom` in their own functions;
the removal is in this commit,
though the fix commit of B398 to B400 names it too,
having made the same change on its own branch (M144).
The pinned commit and its default are unchanged.
Cases also stand in `corpus-pass-refusals`,
`corpus-pin-override`
(rewritten so no case writes `process.env`),
`run-config`,
`draw-entry-load`,
`pass-decline`,
`pass-entry`,
`window-trial-protocol` and `rendering-audit-settled-args`.
By the agent's census over its 15 test files,
`readRunCorpusPinSetting`,
`readCorpusPinSetting`,
`loadEntry`,
`protocolDigest`,
`readAuditArguments` and `recordEntryDecline` show no cold stretch,
and the changed files it lists cold are callers whose suites were not in its run.
The whole merged tree's run is recorded in B391.

Open,
the lead's list,
not fixed:
the TSDoc in `corpus-run/corpus-pass-lines.ts` says "Until 2026-10-06" for the ordering wording and the literal 92,
where both left the code on 2026-10-10 (`cfac5b981`);
`doc/configuration.md` (moved on 2026-10-18 to `package/translation-repair/doc/configuration.md`) still names
`RUN_CORPUS_PIN`,
the removed constant;
and the `INCOMPLETE` detail for an ordinary absent page still tells the operator to "check the path",
a path the pass builds itself,
where the alternative is to reword it locally in `passIncompleteLine` and `readSide`.

Closed on 2026-10-10 (UTC) for the dates and the configuration page,
by `34f669155d47ad0de95d674b21d8740b51ddff0d`:
the TSDoc in `corpus-run/corpus-pass-lines.ts` says "Until 2026-10-10" for both,
and the corpus pin paragraph of `doc/configuration.md` (moved on 2026-10-18 to
`package/translation-repair/doc/configuration.md`) names `readRunCorpusPinSetting`
and says each command reads the pin once,
at the start of its run and inside its refusal boundary,
where it said the pin was read when `run-config.ts` loads.
The `INCOMPLETE` detail stays open.

Open to the owner's veto:
commands that never read the corpus now run past a malformed setting;
the alternative is to read it in every entry.
`rendering-audit-settled` refuses a malformed commit even when `--clone` is given,
as at the base;
the alternative is to read the clone half alone.
The 17 as-built cases sit in one file;
in `corpus-pass` the pin is read after the windows and the overlap and before the runs directory and the lock;
and `judge-fidelity-probe` reads the pin before `digestPipeline`;
the report names no alternative to these three.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal" and "Tests touching the real world".

## B395: the draw's entry loader read its source at the module constant

Red in `b42882f3a74c02b8e77549e8075a18ccbad0a221`,
fixed in `d415f1ddac6b11f68193865985a69842ff9387d2`.

Found by the move B394 records.
`loadEntry` (`corpus-run/draw-entry-load.ts`) read each entry's zh source at `RUN_CORPUS_PIN`,
whatever pin `drawGradingSample` was handed,
so the bands a draw built and the commit its sheets record could come from two pins.
The case is
"READS THE ENTRY'S SOURCE AT THE PIN IT IS HANDED,
never at one of its own"
(`draw-entry-load.unit.test.ts`).

The fix:
`loadEntry` takes `pin`,
which `readDrawPool` hands it from the draw.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B396: the window trial's protocol digest folded in the module constant

Red in `b42882f3a74c02b8e77549e8075a18ccbad0a221`,
fixed in `d415f1ddac6b11f68193865985a69842ff9387d2`.

Found by the move B394 records.
`protocolDigest` (`corpus-run/window-trial-protocol.ts`)
folded `RUN_CORPUS_PIN` into the digest the trial ledger resumes on,
whatever pin `runWindowTrial` was handed,
so a trial at another pin pooled rows bought at the first.
The case is
"CHANGES WITH THE CORPUS COMMIT IT IS HANDED,
so rows bought at another pin are never pooled"
(`window-trial-protocol.unit.test.ts`).

The fix:
`protocolDigest({ headSha, pin })` folds in the pin it is handed,
its clone and then its commit.

Not checked by a run,
an inference:
the digest's key order is the order the pin reader writes,
so a ledger keyed before the change still resumes at the same pin.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B397: the artifact and the decline record named the commit of the module constant

Red in `b42882f3a74c02b8e77549e8075a18ccbad0a221`,
fixed in `d415f1ddac6b11f68193865985a69842ff9387d2`.

Found by the move B394 records.
`settledEntryArtifact` (`corpus-run/pass-entry-artifact.ts`) and `recordEntryDecline` (`corpus-run/pass-decline.ts`)
named `RUN_CORPUS_PIN.commitSha` on the artifact and on the decline record,
whatever pin the pass read the entry at.
The cases are
"NAMES THE CORPUS COMMIT IT IS HANDED on the record,
never one of its own"
(`pass-decline.unit.test.ts`)
and
"NAMES THE COMMIT OF THE PIN IT IS HANDED on the artifact it writes and on a decline record,
never one of its own"
(`pass-entry.unit.test.ts`).

The fix:
both take `corpusSha`,
as does `settledPageArtifact`,
from the pin `settleEntry` is handed.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B398: the stage windows were read at every round and inside the writer stages

Red in `1e8ad4c070a15a130344f7bed7d668db6b59c6a9`,
fixed in `477bbd6601509ac08d867e858af680a59c4f91eb`.

Found on 2026-10-10 (UTC) by the agent working the pipeline-wide environment reads,
the alternatives B381 left open.
`resolveStragglerGraceMs` (`grace-override.ts`) read `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS`
in the `graceMs` default of `runGatherRound` (`stage-round.ts`),
at every round;
and `writerRoundGraceMs()` (`writer-grace-override.ts`) read both window dials
inside the four writer stages
(consolidate-produce,
translate-produce,
refine and repair-editor).
So a case chose its windows by writing the process's environment,
no signature showed that a command depends on them,
and an unreadable dial refused a probe at its first round,
not at its launch.
By the agent's run against the base build,
a judges-only selection with the round dial at `300s` refused
although the case handed in its windows (`candidate-select.unit.test.ts`);
a refine stage with only the writer dial at `300s` refused with
"TRANSLATION_REPAIR_WRITER_GRACE_MS must be ...";
and eight `readWriterGrace` cases fail,
since it ignored the `env` it is handed.
The red commit's message counts 12 of its 156 named test files failing at run time
against the unchanged production code;
the rest are red only to the type checker,
which gives 540 errors in 152 test and fixture files against the base declarations,
and none in production.

The fix:
one value,
`StageSettings` (`stage-settings.ts`),
holding the `WriterGrace` and the `ContestLedger` (B399),
threaded along the one chain the reads share:
by the agent's reference walk from `runGatherRound` with oxc,
160 declarations in 128 files,
reached from 19 entries.
The 18 entries other than `editor-calibrate` call `readStageSettings({ env: process.env })` once;
`editor-calibrate` reads its windows through `readWriterGrace` with its own round fallback (B399).
`runGatherRound`,
`gatherStageVoices` and `runWindowedRounds` require `graceMs` with no default,
so a gather with no window does not compile;
reader stages pass `stageSettings.grace.roundMs`,
and the four writer stages pass `stageSettings.grace.writerMs`.
`resolveStragglerGraceMs` and `resolveWriterGraceMs` require `raw`;
`readWriterGrace({ env, roundFallback })` keeps the refusal order that names the round dial first;
`writerRoundGraceMs` is gone,
and so are the optional test seams `graceMs` on `reviewAbsoluteNaturalness` and `contestLaneSlice`.
The 17 probes now refuse an unreadable window in their entry,
before any round.
At the merge the lead resolved eleven conflicting hunks between this round and B391's and B394's,
in comments,
examples and one duplicate block (M144),
and added the overlap and the settings the merged signature requires to B394's pin case in `pass-entry.unit.test.ts`,
in the fix commit.
The whole merged tree's run is recorded in B391.

By the agent's census of direct `process.env` reads in production code,
82 at the base and 99 after,
the nine library reads of the items this round moved are gone,
and every new read is in an entry's own function.
By the agent's census over its 13 test files,
none of `stage-settings.ts`,
`corpus-run/pass-overlap.ts`,
`grace-override.ts`,
`writer-grace-override.ts` and `corpus-run/slice-overlap.ts` has a cold stretch,
and the cold stretches in the files it touched are older code.

Open:
"only entries read the environment" holds for these reads and for the bases of child environments,
not for the whole package.
`resolveHardCapMinutes` (`corpus-run/cap-override.ts`) and `resolveSpendCeilingUsd` (`corpus-run/spend-ceiling.ts`)
read the environment in their default parameters,
and `readDriftOptIn` (`corpus-run/pass-generation-guard.ts`) in its body;
`corpus-pass.ts` is each one's only production caller,
by a search made for this entry.
The alternative is a required `raw` or `env`,
as done for the windows.
A scan gap the move opened:
`real-clock-bounds.unit.test.ts` keys `graceMs`,
but not the `roundMs` or `writerMs` of a `StageSettings`.
Tests hand windows only through `SCRIPTED_STAGE_SETTINGS`
or `scriptedStageSettingsWaiting({ graceMs })` (`stage-settings.test-fixture.ts`),
whose `graceMs:` key the scan reads,
and the `WriterGrace` literals of `editor-calibrate-run.unit.test.ts` are out of its reach;
the alternative is to add both keys and list the literals of a reader's output.

Closed on 2026-10-10 (UTC) by B401 and B402
(`7285b3c98c55a4654832943f3c444569cb536182`,
and `e749bca51e21079a2ee4289ad20ae73206900c7e` for B401's production code):
`corpus-pass.ts` reads the cap,
the ceiling and the drift opt-in over `process.env` and hands each in,
and none of the three library functions reads the process's environment (B401);
and the scan keys `roundMs` and `writerMs`,
with each value it finds under them listed with its reason (B402).
The example this entry's open item named could not have fired:
the `WriterGrace` literals of `editor-calibrate-run.unit.test.ts` are `HANG_STOP_MS`,
which the scan leaves alone,
and were so in this entry's red commit.
`resolveRunsDir()` (`corpus-run/run-config.ts`) still reads `process.env` itself,
recorded under B401.

Open to the owner's veto:
one `StageSettings` value for both windows and the ledger;
the alternative is separate `graceMs` and `contestLedger` parameters.
Tests run their rounds under the hang stop,
60,000 ms for both windows in `SCRIPTED_STAGE_SETTINGS`,
instead of the 120,000 and 180,000 ms they took from the environment;
so the note case in `editor-calibrate-run.unit.test.ts` reads `60000ms ... 60000ms` where it read `5678ms ... 1234ms`,
and no longer tells the two numbers apart;
the alternative keeps 1234 and 5678 as small bounds on the real clock.
`CorpusPassInput` keeps `graceNote` and `writerNote` beside the new fields,
and `RepairChunkRequest` (`repair-chunk-request.ts`) and `PassPrepareRequest` (`corpus-run/pass-prepare-request.ts`)
are modules of types alone,
split out because `repair-chunk.ts` reached 306 code lines and `corpus-run/pass-prepare.ts` 302,
against the limit of 300;
the report names no alternative to these two.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world" and "Tests on the real clock".

## B399: the contest ledger and the calibration grace read and wrote the process's environment

Red in `1e8ad4c070a15a130344f7bed7d668db6b59c6a9`,
fixed in `477bbd6601509ac08d867e858af680a59c4f91eb`.

Found by the work B398 records.
`recordContest` (`candidate-ledger.ts`) read `TRANSLATION_REPAIR_RUNS_DIR` under every `selectBestCandidate`,
whose callers are `editor-ensemble.ts`,
`refine-stage.ts`,
`translate-judge.ts` and `judge-fidelity.ts`,
so contests landed wherever the process's variable pointed,
and a case chose where by writing it.
`adoptCalibrationGrace` (`grace-override.ts`) wrote `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` back into the process
when the launch had left it unset,
so that every round would read the calibration's window there,
and the variable stood at `300000` after a call.
By the red commit's message,
the contest ledger's cases fail at the base,
the ledger writing where the process's variable points and not where the ledger handed in says,
and the calibration grace's cases fail on the environment it ignores and the variable it writes.

The fix:
`ContestLedger` is `{ kind: 'unkept' }` or `{ kind: 'kept', runsDir }`,
read by `readContestLedger({ env })`,
where unset and empty both keep nothing,
as before;
`recordContest` writes where the ledger it is handed says,
and `selectBestCandidate` hands it the one `StageSettings` carries.
`adoptCalibrationGrace({ env })` returns the grace and writes nothing;
the calibration lane's rounds get `readWriterGrace` with `CALIBRATION_STRAGGLER_GRACE_MS` as the round fallback.
`editor-calibrate.ts` reads the overlap,
the calibration grace and both windows in functions of its own that the run calls where it used them before,
and reads the ledger in `main`.
No production code writes `process.env`:
a search over `src` outside tests and fixtures for an assignment to it
finds none at `477bbd6601509ac08d867e858af680a59c4f91eb`,
and the same search finds the write in `grace-override.ts` at `791196bdc8458c54962cd834b0ac1a58851fdf78`.
The key of `blank-text-tests.unit.test.ts` is renamed to `candidate-ledger.ts#readContestLedger`.

Open to the owner's veto:
the ledger is read from the raw variable,
not from the pass's resolved `runsDir`;
the alternative would start recording contests in a launch that named no runs directory,
which changes what the setting means.
`editor-calibrate` reads its settings where the run used them,
so the tested output and the order of refusals are unchanged;
the alternative reads them all at the start of `main`,
which puts a window or overlap refusal before the refusal of a bad slice count.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B400: the slice overlap was read for each entry, after the lock

Red in `1e8ad4c070a15a130344f7bed7d668db6b59c6a9`,
fixed in `477bbd6601509ac08d867e858af680a59c4f91eb`.

Found by the work B398 records.
`settleEntry` (`corpus-run/pass-entry.ts`) read `TRANSLATION_REPAIR_SLICE_OVERLAP` through `readPassOverlap`,
for each entry,
so an unreadable value refused the pass at its first entry,
after the runs lock was taken and the layout made,
and after `runPassQueue`,
which has no catch,
had counted and written that entry's first attempt;
and a case chose an entry's overlap by writing the process's environment.
By the agent's run against the base build,
a `settleEntry` handed overlaps 1 and 2 with the process's variable at `several` refused;
and eight `readOverlap` cases fail,
since it ignored the `env` it is handed.

The fix:
`readPassOverlap({ env })` (`corpus-run/pass-overlap.ts`) returns the `OverlapSetting`,
read once in `corpus-pass.ts` after both windows and before the corpus pin,
the runs directory and the lock;
`settleEntry` takes the setting
and logs the unchanged `OVERLAP <id> value=... source=...` line through `entryOverlap`;
`readOverlapSetting` and `readOverlap` (`corpus-run/slice-overlap.ts`) take `env`.
So the refusal now comes at launch,
before the lock,
and no attempt is recorded.
B394's fix had read the pin right after the windows;
this round put the overlap between them.
The allowlist line of `own-unit-tests.unit.test.ts` for `corpus-run/pass-overlap.ts` is gone,
since the module has a test of its own;
`window-dial-unreadable.test-fixture.ts` is new,
and `slice-overlap-dial.test-fixture.ts` gains `PASS_FALLBACK_OVERLAP`.

Recurrence:
`mistake-prevention.md`,
"Two layers reading one refusal" and "Tests touching the real world".

## B401: the cap, the ceiling and the drift opt-in were read by library code

Red in `7285b3c98c55a4654832943f3c444569cb536182`,
fixed in `e749bca51e21079a2ee4289ad20ae73206900c7e`.

Found on 2026-10-10 (UTC) by the agent working the open items B398 left.
`resolveHardCapMinutes` (`corpus-run/cap-override.ts`) and `resolveSpendCeilingUsd` (`corpus-run/spend-ceiling.ts`)
read `process.env` in the default of `raw`,
and `readDriftOptIn` (`corpus-run/pass-generation-guard.ts`) read it in its body;
`corpus-pass.ts` is each one's only production caller,
and the barrels re-export them.
By the agent's run against the base build,
`readDriftOptIn` handed `env: {}` with the process's variable at `yes` returned true,
and the walk case handed `yes` returned false:
two cases of `pass-generation-guard.unit.test.ts` fail at the base,
the one test file of the four named that the red commit's message counts failing.
The two resolvers already honoured a `raw` they were handed,
so their move is a signature change the type checker holds:
by the agent's account,
the two base cases that read the environment
("READS THE ENVIRONMENT when the caller names no raw text ..." in `cap-override.unit.test.ts`
and "READS THE DIAL FROM THE ENVIRONMENT when no value is handed in ..." in `spend-ceiling.unit.test.ts`)
give TS2741 against the new declarations,
and the new case in each file,
"READS ONLY THE TEXT IT IS HANDED,
never the process's own variable",
passes at the base.

The fix:
`raw` is required on both resolvers,
with no default that reads the environment;
`readDriftOptIn({ env })` reads the environment it is handed,
taking `env` rather than `raw`,
since `ALLOW_DRIFT_VAR` is private to its module;
and `corpus-pass.ts` reads all three over `process.env`,
in the order it read them before,
the opt-in now after the corpus pin and ahead of `resolveRunsDir()`,
a move no refusal can observe,
since the read cannot throw.
What each setting means and every refusal text are unchanged.
The TSDoc of `runCorpusPass` lists the drift opt-in among the reads made inside the refusal boundary,
and names the four library modules that read no environment for them.
This closes the first open item of B398.
The lead's gate on `e749bca51e21079a2ee4289ad20ae73206900c7e` passed:
the suite at 2,047 passing and 0 failing,
lint with no warnings or errors,
and 49 source scans passing.

Open,
seen by the agent and left:
`resolveRunsDir()` (`corpus-run/run-config.ts`) still reads `process.env` itself,
left since every production call of it is in an entry file,
or in a procedure an entry hands it to as a function
(`resolveRuns`,
or `resolveRunsDir` for `runCoverageProbe`),
by a search made for this entry;
and `streamBoundMs`,
which `bedrock-client.ts` arms a stream with,
is not a key of the real clock scan,
though no test or fixture sets it.
The stream bound was measured and closed on 2026-10-11 (UTC) by B416,
which found three test files arming a card's bound.

Open to the owner's veto:
the drift opt-in is read ahead of the runs directory,
where reading it after the runs directory,
inline as before,
is the alternative;
`readDriftOptIn` takes `env`,
where taking `raw` and exporting `ALLOW_DRIFT_VAR` is the alternative;
and the cap and ceiling cases point the process's own variable elsewhere to show the read never consults it,
so they write `process.env` and run one at a time,
where relying on the type checker alone is the alternative.

Recurrence:
`mistake-prevention.md`,
"Tests touching the real world".

## B679: the shown-label lookup kept its answer under the identity of the list

Found on 2026-10-18 (UTC) by the sol review of the production diff of five landed rounds
(`s245/sol-review/answer-gpt61.md`,
finding 6),
and reproduced as a defect of the interface,
since no production caller changes a list.

Outcome written on 2026-10-19 (UTC).
Red `eb5b23e45c`,
fixed in `12ba110124`.

The cause:
`shownLabelOf` (`consistency/src/consistency-label-form.ts`) cached the lookup of each list of shown labels
in a module `WeakMap` under the identity of the array.
The cache came in with B639 (`29d6064708`).
A list that its owner changed after one lookup kept answering for the labels it had held at the first.
Every caller passes a fresh array
(`consistencyLabels`,
`runOffChoiceFinalistsOf`,
the candidate labels),
and the parameter is typed `readonly string[]`,
so the cache was safe in production and wrong for the interface it exposes.

Fixed:
the module `WeakMap` is gone.
`shownLabelReaderOf` reads a list once and answers for the names put to it,
`shownLabelOf` is one lookup of a reader,
and `shownOnceLabelsOf` (`consistency-wire.ts`) looks up all the names of a ballot field through one reader,
so a ballot that names many places still reads a list once per field.
Counts and ballots are unchanged on every real page.

Test:
a case in `consistency-label-form.unit.test.ts` changes one shown label between two lookups.

Prevention:
the notes name none,
and this filing adds none for a defect that no caller reached.
