# Preventing mistakes: the real world, clocks and arrival order

Rules for tests and code that meet the world outside them:
real files,
keys and processes,
the clock,
process globals,
and the order concurrent work ends in.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Tests touching the real world

What happened:
a unit test prepared an original naming a title,
and the preparation bought a live web search with the key the suite inherits from `mise`
and wrote the answer into the real lookup cache (M43,
X19).
The fix made the seam required of the preparation,
but the entry driver above it went on handing down the run's readers itself,
so the entry tests still prepared through the key,
the real caches and the corpus clone (M68).
The run client and the provider gate read the provider keys,
the Bedrock ledger's place and the transport from the process by default,
and their tests set some keys and built on the rest the suite inherits (X23).
43 test helpers made a temporary directory and could leave it behind,
and the system's temporary directory had gathered thousands under their prefixes (ledger B108).

The T8 trial added an OCR reader parameter that defaulted to the real program and five cases that ran
`magick`,
`dwebp` and `tesseract`;
the reader now requires its program runner,
and its cases run scripted programs (ledger M113).
A program on the machine is an outside read like the corpus,
and its runner is a required parameter.

The picture sources still defaulted to the corpus and the real programs
in the functions that hand them to that reader,
and one case read the pinned checkout for an invented entry (ledger B185).
A default is looked for at every function that hands a seam on,
not only where the seam is first read.

Each test that started a child process built the child's environment itself,
or passed none,
so the children of 27 test files and fixtures could inherit the provider keys the suite holds under `mise`,
and one fixture,
merged unread,
started a node child on the built package with the parent's environment whole (ledger B291,
M121).
None called a provider.
`nano-spawn` merges the environment it is given into the parent's,
so a variable left out of that object is still there:
an environment for a merging spawner states each name it removes,
with the value `undefined`.
An agent then ran a built command from its own shell,
behind a strip of its own making that removed nothing,
and bought six provider calls (ledger M120).
A test starts a child only through `package/translation-repair/adjudication/src/child-environment.test-fixture.ts`
(`spawnKeyless`,
`runKeyless`,
`runBuiltCommand`),
which removes every variable whose name ends in `_API_KEY` and every `TRANSLATION_REPAIR_` setting,
and only then sets what the case names.
A built command is started the same way by an agent as by a test,
and an environment is never stripped,
filtered or rebuilt by hand.
Three as-built suites still started their command through `runKeyless` with a path each built itself,
without the start marker `runBuiltCommand` adds (ledger B386);
a test starts a built command through `runBuiltCommand` alone.
A shared brief called the logger's sink verification lines noise,
not a failure,
and the first run of the whole-suite census at `116bf6f4b` then failed nine as-built cases on such a line,
since each pins the stderr of the child it starts whole (ledger M145,
T9).
A brief names a known defect of another package as a defect,
with the cases it is known to fail,
and tells the agent to report such a case and run it again,
never to ignore it.

Production code then started its own children with the run's whole environment:
git children,
the picture program and the coverage census's suite child (ledger B314);
and the corpus listing's git child kept the repository variables its guard meant to drop,
since the spawner merged the names left out back in from the parent (ledger B315).
Every production start passes the environment `childEnvironment` builds.
A built command then gained a guard of its own:
in a process that holds a key,
it runs only when `TRANSLATION_REPAIR_STARTED_BY` names it,
which its task sets,
and the test fixture,
or an operator by hand where a pass forbids a rebuild (ledger B319).

A review of the trial's cases found the real corpus pin written as fixture data in nine test files,
where no code the cases reach checks it against the pin (ledger B332),
and a case that made a file unreadable with mode `000`,
which the superuser reads anyway,
so the case failed whenever the suite ran as root (ledger B334).
Eight log lines that print a provider's failure had no case reaching them
through the real client over the real transport (ledger B330).
The corpus git children kept the caller's locale variables,
and git translates the refusals the corpus reader tells apart by their English words:
under a German `LANGUAGE`,
`LANG` or `LC_MESSAGES` a path absent at a held commit printed its refusal in German (ledger B347).

The process's own state was also read and written inside library code,
not at the entries.
The ledger and meter reports wrote `process.exitCode` inside their procedures,
so a caller importing either had its process status changed (ledger B379).
The artifact pool read its commit policy from the environment inside `resolvePool`,
so a case chose its pool by writing the process's variables (ledger B380);
and three readers of the runs directory read the environment themselves,
so the relabel and verify procedures read whatever the variable named,
not the directory they were handed (ledger B381).
A procedure returns its exit code,
and the entry sets it on the process.
Library code takes a setting as a value from the entry,
and the entry reads the environment;
a read left inside library code is recorded with why it stays (ledger B381).
The same held deeper in the pipeline.
The corpus pin was resolved while `run-config.ts` loaded,
and seven library modules read it as a module constant,
so the draw's entry loader,
the window trial's digest,
an entry's artifact and its decline record used that constant whatever pin they were handed (ledger B394 to B397).
The straggler and writer windows were read at every round and inside the writer stages,
the contest records' directory under every selection,
and the editor calibration wrote its window back into the process for its rounds to find (ledger B398,
B399);
the slice overlap was read for each entry (ledger B400).
Each is now read by the entry file and handed down as a value,
and no production code writes `process.env`.
The cap,
the spend ceiling and the drift opt-in were still read in library code,
in the defaults of `resolveHardCapMinutes` (`corpus-run/cap-override.ts`)
and of `resolveSpendCeilingUsd` (`corpus-run/spend-ceiling.ts`),
and in the body of `readDriftOptIn` (`corpus-run/pass-generation-guard.ts`);
`corpus-pass.ts`,
each one's only production caller,
now reads all three over `process.env` and hands them in (ledger B401).
One read stays in library code:
`resolveRunsDir()` (`corpus-run/run-config.ts`) reads `process.env` itself,
and every production call of it is in an entry file,
or in a procedure an entry hands it to as a function (ledger B401).
The width probe built its run client before its draw,
so a process holding no key refused before every refusal that needs no provider,
and its as-built cases never reached the draw (ledger B391).

The artifact pool's case for a required commit named by a revision read this checkout's `HEAD` and `HEAD~1`,
since `resolveCommit` and `tipContains` (`corpus-run/artifact-generation.ts`)
took a repository that defaulted to the package's own directory,
and `resolvePool` handed them none,
so the case failed on a checkout of one commit (ledger T8,
the runner and git cluster of the final census).
The repository is now a required value from the five pooling entries down to `resolveCommit` and `tipContains`,
and the case builds a throwaway repository of two commits (ledger B393).
One suite still reads this checkout's own git:
four cases of `corpus-run/artifact-eligible.unit.test.ts` read its root commit and `HEAD` read-only,
through `gitBounds`,
and hand the checkout in as `THIS_CHECKOUT`.
It stands against the rule that an outside read is a required seam,
awaiting the owner's word,
where a throwaway repository is the alternative (ledger B393).

The rule:
before a test drives a production entry point,
list what that entry point reads outside the process
(`process.env`,
`fetch`,
`homedir()`,
the pinned corpus) and hand each a fixture.
A seam that reaches the network,
a real cache or the corpus is a required parameter,
never an optional one.
When a seam becomes required,
every reference to the production value is listed (`rg 'outsideReadsFrom\(' src`),
and each one in a function the entry calls moves to a parameter:
the process's own values are named in the entry file,
inside its refusal boundary
(`createRunClient`,
the `outsideReadsFrom` calls of `corpus-pass.ts` and `rendering-audit-settled.ts`,
`readStageSettings`,
`readRunCorpusPinSetting`,
`readPassOverlap` and `readDriftOptIn` over `process.env`,
and the cap and ceiling texts `corpus-pass.ts` hands to `resolveHardCapMinutes` and `resolveSpendCeilingUsd`),
and in TSDoc examples;
no module builds one while it loads (ledger B394).
A client a run builds is handed in as a factory
and built where the run first needs it,
after every refusal that needs no provider (ledger B391).
A test makes a temporary directory only through `scratchDir`,
bound with `await using` where it is made or by every caller of a helper returning it,
so the directory goes when the case ends,
however the case ends.
A helper that fills the directory before returning it makes it through `scratchDirWith`
(a setup with fields to hand back)
or `scratchDirPrepared`
(a setup handing back nothing),
which remove the directory if the setup throws;
a helper that only makes it is no helper,
and its callers bind `scratchDir` themselves (ledger B117).
An environment variable a case points at its directory is written by a function the case calls by name,
after the directory stands,
and its disposer is bound after the directory's,
so the variable is restored first.
A fixture writes invented data where any value would do:
a commit id no code checks against the corpus pin is an invented one (`feedface` repeated),
and the real pin stands only where a comment records where something was measured
or a production example shows the package's own pin.
A fault a case makes holds whoever runs the suite:
a file a case must fail to read is one Node refuses by its size
(a sparse file of 2 GiB),
never one a mode forbids.
A line that prints a provider's failure is reached by a case
through the real client over the real transport with a stubbed `fetch`
(`provider-status-failure.test-fixture.ts`).
A reader of another program's words fixes the language that program speaks
where the environment for its child is built
(`LC_ALL=C` for the corpus git children),
and a case runs the real program under another locale
beside a control showing that the program translates there.
The corpus reader's search ceiling held the parent of the clone directory as named,
while git compares the list with its real working directory,
so a clone directory that is a link read the repository it points into (ledger B369);
and git splits that list at colons,
so a parent path holding one voided the ceiling (ledger B370).
A path handed to another program is resolved as that program resolves it,
and a value written into a list the program splits at a delimiter is refused,
before the program runs,
when it holds the delimiter.

What enforces it:
the type checker:
`preparePassEntry`,
`runPassPreparation`,
`runEntryPipeline` and `settleEntry` require `outsideReads`,
and tests pass `NO_OUTSIDE_READS` (`corpus-run/pass-outside-reads.test-fixture.ts`);
`readPassVisualEvidence`,
`readSeatedPictures`,
`runEntryPipeline` and `settleEntry` require `pictureSources`,
and tests pass `NO_PICTURE_SOURCES` (`corpus-run/pass-picture-sources.test-fixture.ts`);
`configureProviders`,
`runClientFrom` and `assertRequiredProvidersReady` require `env` and `transport`;
`outsideReadsFrom` requires the environment,
the transport and the corpus readers;
the five provider clients require their transport,
and every corpus reader its pin or reader (X24);
`runGatherRound`,
`gatherStageVoices` and `runWindowedRounds` require `graceMs`,
every stage on the way to them takes `stageSettings`,
and tests pass `SCRIPTED_STAGE_SETTINGS` (`stage-settings.test-fixture.ts`,
ledger B398,
B399);
`settleEntry` requires the pin and the overlap setting (ledger B394,
B400);
`resolveHardCapMinutes` and `resolveSpendCeilingUsd` require `raw`,
and `readDriftOptIn` requires `env` (ledger B401);
and the pool and its readers require `repository`,
which tests that require no commit pass as `UNASKED_REPOSITORY` (`corpus-run/pool-policy.test-fixture.ts`,
ledger B393).
A seam added to make a function testable never takes the production value as its default (M70).
`package/translation-repair/source-scan/src/temp-dirs-scratch.unit.test.ts` (ledger B117),
among the source scans,
fails on a direct `mkdtemp` or `tmpdir` call in a test or fixture outside the kept sites it names with why,
and on a `scratchDir` call that is not the initializer of an `await using` declaration.
`package/translation-repair/source-scan/src/test-children-keyless.unit.test.ts` (ledger B291),
among the source scans,
fails on a test file or fixture,
other than the shared one,
that imports a spawning module or calls a spawning function;
the shared fixture's own test starts real children that print the names they see.
`package/translation-repair/source-scan/src/production-children-keyless.unit.test.ts`,
among the source scans,
fails on a production start of a child that does not pass the environment `childEnvironment` builds
(`child-process-environment.ts`,
commit `4049d9eb2`,
ledger B314 and B315,
which close the item ledger B291 left open);
`package/translation-repair/corpus-pass/src/production-children-environment.unit.test.ts` starts each site's real child
and reads from the child's own side that no key or setting arrived.
`package/translation-repair/corpus-pass/src/corpus-git-context.unit.test.ts` asserts the corpus environment whole
and reads real git's refusal in English under a German locale,
beside a control that fails on a host whose git does not translate (ledger B347);
`package/translation-repair/adjudication/src/corpus-source.unit.test.ts` refuses a clone directory that links to a plain
directory inside a repository,
still reads through a link to a real clone,
and refuses a clone whose parent's real path holds a colon (ledger B369,
B370).
`task-runner-guard.ts` refuses a built command started in a process that holds a key
unless that process names the command (ledger B319),
and `package/translation-repair/source-scan/src/task-runner-guard-reach.unit.test.ts`,
among the source scans,
fails a task that starts a built command without the marker,
a built command no task starts,
and an entry file that hands the guard no environment.
A process that imports the built library directly is outside the guard,
and a shell in a worktree still holds the keys.
`corpus-run/ledger-report-run.unit.test.ts` and `corpus-run/meter-report-run.unit.test.ts`
assert the code each report returns and that it leaves the process's code unset (ledger B379);
the cases of the relabel artifact reader,
the width report and the bench report point the variable elsewhere and hand a directory in (ledger B381);
and the as-built conflict cases of the five pooling commands hold the pool's refusal at exit 6 (ledger B380).
The cases of the window readers,
the contest ledger and the overlap readers hand each its own environment
beside a process that holds another value (ledger B398 to B400),
as do the cases of the cap,
the ceiling and the drift opt-in (ledger B401),
and `corpus-run/corpus-pin-override.unit.test.ts` writes no `process.env` (ledger B394).
`corpus-run/editor-width-probe-run.unit.test.ts` holds the order of the draw,
the client and the control,
and its as-built suite runs over a throwaway runs directory,
lookup cache and corpus clone (`benchWorld`,
ledger B391).
No source scan finds an environment read,
an exit-code write or a hand-built path to a built command outside where these rules put it,
by the names of the scans the `source-scans` task runs;
the censuses that found them were scratch scripts,
so the rest is habit.

A test or gate result counts only from a worktree with no `node_modules` in any parent directory,
because Node finds a bare import by walking up the directory tree and a stray install there answers for a missing package
(ledger B801,
B802).
A dependency removal or override is checked by loading every unbundled entry of the packages that reach it,
the unit test files among them,
and not only by the digest of a built bundle,
which drops code the source still imports.

## Tests on the real clock

What happened:
a case filled a 20 ms window on the real clock and expected a take started after it to still be asleep;
under the whole suite's load the window emptied first,
and the case failed only there (M69,
T5).

Two stage cases expected a voice delayed 30 ms to land inside a 100 ms grace window,
which a loaded machine could outrun (ledger B318).
Their windows are 60,000 ms,
which the round never waits for since it ends at the last answer,
and each case fails at a window of 0,
which shows the window is what keeps the late voice.

The consolidation driver test bounded each call and each drive at 5,000 ms over milliseconds of scripted work,
and its TSDoc said nothing bought a call where a case bought calls;
on a loaded machine the case driving two slices serially and then two at once
failed with a `TimeoutError` its name never states,
twice,
and passed in every other run of the whole suite (ledger B340).
Its bound is 60,000 ms,
documented as a stop for a hang that no case waits for.

The census that followed,
on 2026-10-06 (UTC),
read every bound the tests and fixtures set on the real clock,
948 of them outside seven files deferred to the changes in flight on them
(ledger B358).
The bounds that only stopped a hang now share one constant;
the bounds a case reaches bound work that cannot end before them;
a seat that slept past a window,
a head start that ordered two slices,
and a ceiling on elapsed time gave way to gates and to what the case can read off the abort (ledger B359,
B360);
an attempt that slept 200 ms under its own bound of 300 ms gave way to one that answers at once,
the time passing in a backoff longer than the bound (ledger B361);
a parse's time limit moved to the process's CPU clock (ledger B363).
A case on where a deadline is armed could not fail,
since its transport ignored the signal;
it now holds the slot past the deadline and refuses an aborted exchange (ledger B362).
The census also found that a per-call deadline handed to a stage over a hand-scripted client is never armed,
since the provider clients arm it (ledger B364):
what fires in a test is a signal,
a cap,
a window
or a stream bound
that the test arms itself or hands to production code that arms it.

The seven deferred files were read on 2026-10-10 (UTC),
34 findings in 15 keys,
six more than the census counted from cases added since its base (ledger B378).
Six grace windows in the round test bounded rounds that end before them,
since every case there passes at a window of 0,
and went to the hang stop.
The round test held its quorum mark under 125 ms past the first answer,
which failed in 2 of 4 runs beside other new cases (ledger B365),
and a whole-roster round's grace under its window;
a stall of 300 ms broke each.
The quorum mark is now held to the span from the round's start to the next turn of the event loop,
which the microtasks before the mark cannot outlast,
and the grace to the span from the last answer to a timer armed after that voice's own and as long;
each new shape passes under the stall,
and fails when quorum waits for the slow voice,
or when the round waits out a window.

A scripted review client held a straggler for 25 ms counted from its own request,
so that it landed after quorum,
and two bounds entries said it lands there "on any machine" (ledger B813).
The stage's own work between the peers' answers and the quorum tally is real time too,
and in one run of the rotation sweep it outlasted the 25 ms:
the held seat landed before quorum and the readers' window had no straggler to cut.
The hold is 1000 ms now.
A client-side barrier that holds the straggler until every other seat has answered holds nothing,
since the other seats answer synchronously and the slow part is the stage after the client returns.

A case registered the router's own hold of 300000 ms for a provider
on a view whose checker bench was short of quorum,
and the seat reading sleeps the shortest hold it is told of when a bench is short,
so the case slept five real minutes and then passed (ledger B817).
The corpus-pass package's `test:unit` went from 43 to 85 s in the gates up to g137
to 305 to 313 s in every gate from g138 on,
and about 30 gates were read as green and went on without anyone noticing,
since each gate counted its `[FAIL]` lines and no gate compared the time (ledger M288).
No timer call stood in the case for a scan to read,
because a hold reaches the sleep through a client's reader of holds.
The fix is that no case registers a hold of a second or more:
a case needs a hold only to be running,
and registers 40 ms.

The rule:
a real-clock case asserts only bounds that load can widen and not break;
a case that needs a caller asleep uses a window no stall outlasts;
a grace window a case relies on is one the round never reaches,
and the case is shown failing at a window of 0;
a window a round ends before is the hang stop,
and the case is shown passing at a window of 0;
a bound that only stops a hang is `HANG_STOP_MS` of `hang-stop.test-fixture.ts`,
60,000 ms,
never a number of its own;
a bound a case means to reach bounds work that cannot end before it on any machine,
work that ends only on its own abort or a sleep armed after the bound and longer than it;
a meter or a sleep that must overlap or order other work is shown in place within one turn of the event loop,
or gives way to a gate the case opens;
elapsed time is asserted only as a lower bound,
or as no more than a span the case reads itself over the same work,
up to a turn of the event loop or a timer armed after the work's own and as long,
which no stall can make shorter than what it bounds;
synchronous work is timed on the process's CPU clock;
a new timing case counts as passing only once the whole suite has run it;
a hold a case registers for a provider is the short one,
since the seat reading sleeps the hold it is told of (ledger B817);
a gate that reads a package's `test:unit` time compares it with the same package's time in the gates before it,
and looks into a jump before the gate counts as green,
by reading the block durations the log prints after each block's name
and taking a duration that matches a constant of the code under test as a sleep (ledger M288).
A bound on scripted work is a stop for a hang,
which no case waits for,
documented as such where it is declared;
never a few seconds over milliseconds of work,
which a loaded machine outruns.
A case that needs a run stopped brings its own controller,
so the stop for a hang never doubles as the stop a case asserts.
A real-clock hold in a fixture,
which keeps a seat back so that it lands after a mark,
is sized against the stage's own work between the other answers and that mark under load,
and is far over it;
a few tens of milliseconds counted from the held seat's own request is not.
A bounds entry never says a seat lands after a mark "on any machine"
unless a gate or an abort the case controls makes it so (ledger B813).

What enforces it:
`real-clock-bounds.unit.test.ts`,
among the source scans,
fails a bound a test or fixture sets on the real clock other than the shared hang stop
unless it is listed with its class and reason,
and fails a listed bound or a deferred file the source no longer holds,
with no file deferred since ledger B378;
the whole unit suite run before a batch's work is called done.
`real-clock-bounds.unit.test.ts` also reads the holds tests register for a provider (ledger B817):
a hold of 1000 ms or more written onto a rig's `holds`,
or given to a provider in a record that a holds reader returns or a constant named for a hold holds,
fails the scan unless it is listed with why it sleeps nothing,
and the three listed (`stop-landing-hook.test-fixture.ts`,
`budget-hold-wait.unit.test.ts`,
`run-seats-read.unit.test.ts`) each name the reason.
It flags by size and cannot tell a hold that is slept from one that is not,
since that depends on whether a bench is short,
and it does not reach a hold built from a name it does not read or handed on by a parameter.
It keys a case written as an anonymous function `<module>`,
so it counts the holds of a file and does not locate one.
Nothing in the repository compares a package's test time with its previous gates:
that stays a habit of the gate (ledger M288).
No scan reads an upper bound on elapsed time;
review holds each to a span the case reads itself.
Since the stage windows became a `StageSettings` value (ledger B398),
tests hand them through `SCRIPTED_STAGE_SETTINGS` or `scriptedStageSettingsWaiting({ graceMs })`
(`stage-settings.test-fixture.ts`),
whose `graceMs:` key the scan reads.
The scan also keys `roundMs` and `writerMs`,
the two windows a `WriterGrace` carries inside a `StageSettings`,
from which every stage hands one to its round as `graceMs` (ledger B402);
the values it finds under them that keep their own,
a window handed on from its caller,
a reader's output asserted whole,
or a grace read back in its launch note,
are listed with their reasons.
A calibration grace's `effectiveMs` and a note's `builtInMs` are not keys,
since production only prints them.
The scan keys a model card's `streamBoundMs` too,
and reads a `createBedrockClient` imported from the built package under any local name
whose options name no `streamBoundMsOverride`,
since the client arms each card's own bound on the real clock around every attempt unless handed one;
a stream bound of `'unbounded'` is left,
as the override tests hand it (ledger B416).
A test builds a Bedrock client with the override,
`'unbounded'` unless the case reaches the bound.
Open,
as the follow-up merged on 2026-10-10 (UTC) left it (ledger B358,
B378):
the logger package's own sink verification is bounded at 5,000 ms,
which timed out 15 to 34 times per whole-suite run in the agents' runs that counted it,
and 12 times in one run of the source scans,
without failing a case.
A timeout can fail a case all the same:
the logger then writes `logger internal error: sink verification failed ...` into the process's stderr,
and in the first run of the whole-suite census at `116bf6f4b` that line failed nine as-built cases,
which pin their child's stderr whole (ledger T9,
M145).
It is the logger's defect,
open as GitHub issue 573;
a case it fails is reported and run again,
never ignored (the rule in "Tests touching the real world").

## Durations on the system clock

What happened:
durations,
deadlines,
holds,
paces and budgets were differences of `Date.now()` readings,
and the system clock is the time of day a person or a time daemon can set.
Set back an hour,
the pacer asked a take to wait an hour more than its window,
the budget view held a refuser an hour on a 300 ms backoff,
and an entry's artifact recorded a duration of an hour below zero,
which its reader refuses;
set forward,
the repair benchmark read its budget spent and skipped an entry (ledger B78).
The fidelity probe read the start it records after reading every reference,
so the time those reads took was left out of the run it dates (ledger B383).
A run's start is read before its first work.

The rule:
a difference of two readings inside one process reads `monotonicMs` (`monotonic-clock.ts`),
whose clock no setting of the system time moves;
the system clock is read only as a stamp,
`new Date().toISOString()`,
for a moment that outlives the process.
A `monotonicMs` reading counts from near the process's start,
so it is never stored,
and zero marks nothing as absent:
a stamp for "never" is negative infinity.
A timer can end a millisecond before a monotonic deadline,
so code that must not start early reads the clock again after its sleep.

What enforces it:
`wall-clock-reads.unit.test.ts`,
which `source-scans` runs,
fails on the system clock read as a number in source or tests
(`Date.now` in any form,
`getTime` or `valueOf` called,
a unary plus or `Number` on a date,
`performance.timeOrigin`,
`Temporal.Now`),
and the red cases in each site's test file step the system clock through `wall-clock-stub.test-fixture.ts`.
Out of the scan's reach:
those globals reached through `globalThis` or another name,
dates compared with `<` or `>`,
and durations between stamps a process wrote.
Where a run reads its start is held by the fidelity probe's case alone,
which counts the reference reads made by each clock reading (ledger B383);
elsewhere it is habit.

## Globals a case replaces

What happened:
the stub for the system clock replaced `Date.now` for the whole test process,
while module-test runs a suite's cases side by side,
sixteen at a time by default.
A pacer case beside the stepped one read the stubbed clock,
and a guard-off that should have failed a case hung its file instead;
the logger's stamps in sibling cases took the stepped time too (ledger M100).
Thirteen other test files wrote environment variables,
`console` methods or the working directory in suites that ran their cases at once,
where a restore finishing out of order could leave a sibling's value behind (ledger B79).
A fixture that diverted `console.warn` to read the lines a piece of work logged
caught the logger's own report of a sink it could not verify,
since the logger starts on a process's first line and that line fell inside the divert;
on a loaded machine the report failed a cache case,
and the fixture's message had called the state unreachable (ledger B140).

The rule:
a case that replaces a global replaces it only for itself:
a method through the case's own sandbox,
`ctx.sinon.stub(target, 'method')`,
which module-test answers only to code in that case's async context and restores when the case ends;
any other write to a process global
(an environment variable,
a `console` method,
`process.exitCode`,
the working directory)
happens only in a suite run one case at a time,
`concurrency: 1` on it or on every suite around it out to the file's top,
even when no await falls between the write and its restore today.
Before writing such a fixture,
read how the runner schedules cases.
A case that replaces a process-wide writer starts every logger that writes through it before the replacement:
the built package's logger and the test framework's,
each flushed so its sinks are verified,
so a logger's report of its own reaches the writer it was meant for
and never reads as a line the work logged.
A guard-off whose process does not exit is a finding,
and a guard runner ends each file after a bound.

What enforces it:
`global-writes-sequenced.unit.test.ts`,
which `source-scans` runs,
fails on a write to a process global in a test or test fixture
that is not inside a case every suite around which runs one case at a time,
following helpers called by name to their cases;
`wall-clock-stub.unit.test.ts` holds a sibling case that reads the real clock while another case holds a stepped one;
`console-warn-lines.unit.test.ts` runs the warn-lines fixture in a child process whose file sink cannot verify
(added in `7ecd53897`).
Out of the scan's reach:
a write through an alias such as `const env = process.env`,
and a case made by a function the suite's text only calls.

## Output decided by arrival order

What happened:
one checker round read twice gave its findings and readings in two orders,
by which seat lost its first answer,
and two cases had sorted the findings,
which hid it (ledger B253).
The author table of a score report ordered equal counts by the directory listing,
which is not sorted (ledger B287).
And one as-built case failed in two whole-suite runs and passed when its file ran alone:
the command read two pages at once,
both reads failed,
and the refusal named whichever ended first,
which a probe of 100 such pairs measured at 98 against 2 (ledger B300,
B316).
Every concurrent read set a census found reports the first failure in input order since,
fifteen of them with no case of their own (ledger B317).
The verifier printed its entries' lines in the order its reads finished (ledger B310),
and a test helper put the recheck's findings and ballots in order before comparing them,
which hid a regression of the roster order the stage promises (ledger B328);
a second test file did the same through `readingsInSeatOrder` (ledger B343).
The bench draw's skip line kept a fixed-length opening of the refusal,
cut inside the commit hash before the page was named,
and printed the lines of several entries in the order their reads ended (ledger B316).
A gather round raced its asks,
and once the caller stopped it two asks could reject with failures of their own,
so the round reported whichever ended first
and the other failure reached no line (ledger B352).
That fix logged the failure it replaced by its class alone,
the status and the provider's words dropped,
and only for the ask that ended first:
a refusal ending after an ask that ended in the caller's reason reached no line,
and a stop after quorum stood let the wait on the grace window swallow every failure;
four cases pinning the provider's words in the line a stop writes failed at the gate (ledger B365).
The runner observations change put the bench draw's rethrow in the catch of a `Promise.all`,
so an entry was skipped or refused by which of its reads ended first (ledger B367).
Until review,
the scan written to hold the rule passed what it was meant to fail:
a listed reason's anchor held to the call's top-level statement,
so any name in the same `if`,
`for` or `try` block held it (ledger B349);
a catch that handed its error on through an alias,
a cast or a nested object read as quiet (ledger B350);
`Promise` reached through `constructor`,
a namespace or a doubled `globalThis` went unseen (ledger B351);
and an unlisted call written before a listed one took its entry,
the finding naming the listed call's line (ledger B354).
Forms the scan already read were planted only then (ledger B355),
and the section the change wrote for its own facts was folded into this one,
rather than a second section on one rule (ledger B357).
The displacement probe and the settled audit read in input order
and still logged each entry's line from inside its member,
as the member ended,
so one input logged its lines in a different order on each run (ledger B388).
The editor standing read listed its directories and read its artifacts under `Promise.all`,
and printed each "no artifacts under",
off-roster or refused line from inside the member that found it,
so one set of archives gave its stderr lines in completion order (ledger B392).
The repair of invalid candidates re-asked each author and read the re-asks in input order,
but wrote no line for a re-ask that failed as the caller stopped,
so every failure but the first in input order was lost (ledger B403).
The sibling round had been fixed for the same shape;
this fan-out was found by a census of the callers,
after the ledger had recorded it as an open reading.
That census,
run over every caller,
found two more fan-outs with no line:
the vision readers,
where the abort guard stood before the only place a rejected settlement was written,
and a pinned reference read (ledger B404,
B405);
the same pass found the required-provider gate
and the meter line reading the caller's own stop as a meter failure,
with the provider's words dropped (ledger B407).
The last site,
the overlapped map,
whose jobs fail in the code they call,
was then run red in six of its nine callers and fixed in the helper (ledger B408);
the same run found the lookups for cited references and work titles writing the caller's own reason as a fetch failure
and answering it as data (ledger B409).

The rule:
what a command prints,
records or reports does not depend on which of several concurrent operations ended first,
nor on the order a directory was listed in.
Concurrent reads collect their results,
and printing is sequential,
in the order of the input.
Where several started operations may fail,
what is thrown is the first failure in input order,
through `allInInputOrder` (`all-in-input-order.ts`),
never `Promise.all`'s first by the clock.
A combinator call outside the helper is listed with the reason no completion order can show
(its members cannot reject,
each catches its own failure,
they share one abort reason,
or their settled results are read in input order)
and with what in the code that reason rests on,
held to the call it excuses:
the statement making the call and the code feeding it,
every catch inside the call,
or the handler of a `try` around it.
A failure a shared reason replaces is logged,
not dropped:
each ask's own failure is written by the ask that owns it,
as it ends,
with the provider's words a log line may carry (`exchangeFailureLogText`);
a round that throws a shared reason in place of a failure is never the only place that failure is logged.
A walker that skips some failures steps past a missing object alone,
decided at the read,
and reports every other failure first in input order.
Voices are ordered by roster,
ties by a stated key,
and a case asserts the order exactly,
never after a sort of its own.
A log line about one member of a combinator is printed after ordering,
from what the member returned,
never from inside the member;
a progress line kept inside one says so.

What enforces it:
`promise-combinators-in-order.unit.test.ts`,
among the source scans,
fails on a combinator called in production code outside the helper's own call and the listed sites,
on every way it sees to reach a combinator other than by a plain named call,
on a call in a function it cannot key,
on a listed site holding more or fewer calls than listed,
and on a listed reason the code no longer holds;
its module doc names what is out of its reach,
such as a reflective or computed read off a value other than `Promise`.
`all-in-input-order.unit.test.ts` holds the helper,
and each moved site with a parameter over what it reads has a case scripting the later member failing first,
`settled-carve.unit.test.ts`,
`bench-sample-draw.unit.test.ts` and `stage-round.unit.test.ts` among them;
`stage-round.unit.test.ts` also holds the line of a refusal ending before an ask that ended in the caller's reason,
after one,
and after quorum stood,
each whole (ledger B365);
`repair-edit-stages.unit.test.ts` holds the roster order whichever seats lost their first answer,
`refine-recheck.unit.test.ts` and `refine-slice-settle.unit.test.ts`
assert the recheck's order as the stage returns it,
and `corpus-run/score-crosscheck-authors.unit.test.ts` holds the author table's ties under both arrival orders.
`displacement-probe-run.unit.test.ts` and `rendering-audit-settled.unit.test.ts` hold their per-entry lines
in input order when the later member ends first (ledger B388).
`corpus-run/editor-standing-read-run.unit.test.ts` holds each refused artifact in the order its archive was named
when the later archive's read ends first,
and two directories that cannot be listed in the order they were named (ledger B392).
`translate-repair.unit.test.ts` holds the line each failing re-ask writes as the caller stops the repair,
with the later-listed seat's first,
none for a re-ask ending in the caller's own reason,
and the first failure in input order thrown (ledger B403).
`image-reading-pair.unit.test.ts` holds a reader's line in roster order,
with one that ended before the stop,
`fidelity-reference-read.unit.test.ts` the line of each pinned page,
`corpus-run/required-providers.unit.test.ts` the gate's refusal with the caller's own reason and no line for it,
and `provider-meter-state.unit.test.ts` the meter's status and the provider's words and a stopped read written as stopped
(ledger B404,
B405,
B407).
`overlapped-map.unit.test.ts` holds each failure the helper does not throw,
written in position order whichever ended first,
none for the thrown one or for the caller's reason,
the same with no stop at all,
including a failure that is no error,
and no second line for a buy's failure the twin memo wrote;
`twin-memo.unit.test.ts` holds no abandoned-buy line for a buy ending in the caller's reason,
with the key withdrawn and the waiting twin asking for itself,
and the line with the provider's words for a buy's own failure;
`translate-repair.unit.test.ts` and `translate-slice-attempt.unit.test.ts` hold a re-ask's failure once
on its seat's line when the repair runs as a buy inside the map,
and `corpus-run/run-seats-read.unit.test.ts` holds one line for a seat reading's stop (ledger B411,
B412,
B413);
`repair-slice-buy.unit.test.ts` and `repair-refine-step.unit.test.ts` each hold the seat reading's refusal once,
on the reading's own line,
with their abandoned line bare,
when the stop lands as the pass's own re-seat hook reads the bench (ledger B415);
the three abort catches (`translate-slice-attempt.ts`,
`repair-slice-buy.ts` and `repair-refine-step.ts`) write the same line for a marked failure;
the lane drivers' tests (`consolidate-driver`,
`lane-contest-driver`,
`translate-document`,
`refine-phase`,
`repair-translation` and `corpus-run/pass-insertion-admission`) each land the stop inside a later job's hook
through `stop-landing-hook.test-fixture.ts`,
and `work-title-lookup.unit.test.ts` and `cited-reference.unit.test.ts` hold the lookups refusing with the caller's reason
while a member's own failure keeps its line,
and each page's or title's line in input order when a later one ends first (ledger B408,
B409,
B414).
A helper that reads many jobs takes the caller's signal and logger as required parameters,
so a caller cannot leave the stop unread;
a failure that a lower layer wrote is marked there,
by every writer that lets it out (the twin memo,
`repairOneCandidate` and the seat reading),
and the helper skips marked failures instead of guessing from their class.
A member that writes a line returns an outcome instead of writing from inside the member,
so the lines are written once every member has ended,
in input order.
A layer that holds the caller's signal and sees the caller's own reason writes nothing for it:
the reason is the caller's,
never the member's failure.
When a fix closes one fan-out of a shape,
the same change lists every other caller of the helper that rethrows unwritten under a stop,
and says which are covered;
the census is part of the fix.
No scan finds a result ordered by arrival,
nor a line logged from inside a member.

## Orders and seats that a prompt hash decides

What happened:
the bench is asked in an order rotated by the hash of the prompt,
with a window of quorum plus one seats (`stage-fanout-window.ts`),
so any edit to a sheet moves the hash,
and with it the order the voices answer in and which seats stand in a round's window.
The quoted-tense rule changed the text of the sheets (ledger B627),
and ten tests that had pinned the old order or the old seats failed in the whole suite,
all of them green in the run before.
They named three voices in the order a report lists them,
read slate entries by position,
compared ballots,
claims,
claimants or renderings by position,
looked for the first seat asked by position,
or expected particular editor seats to be seated
(`hf:moonshotai/Kimi-K3` was seated where `glm-5.3` had been).
n161 made each independent of the rotation,
and wrote the reason in a comment.
The review of n162 then found that six of them had lost a guard that had nothing to do with the order (ledger B631):
two parsed both sides of an on-disk format and wrote them out again,
one took the expected order from the function under test,
two accepted any six of the seven seats that can edit,
and one looked up two named voices and no longer failed on an extra entry.

The rule of the rotation came back with the reader-protection scope of 2026-10-20,
which lengthened the house rule every sheet splices:
the consolidate deep-reply case and the corpus-pass contest re-seat case named a seat the first round no longer asked
(ledger B701).
The census of the other cases that name a seat or an order is closed:
it found nine members across `stage`,
`corpus-pass`,
`probe` and `translate-stage`
(the contest re-seat case among them),
five pinned in the tests and four reached by seams that now take `fanOut` (ledger B701).

The rule:
a test never pins an order or a seat set that a prompt hash decides,
since any sheet edit moves the hash.
It compares by seat:
ballots,
claims,
claimants and renderings sorted by model id in code-point order,
and a seat found by its name.
A set of seats that a case needs is pinned by an injected start of the rotation,
and was first checked for membership in a list of seats,
which let a worse set pass.
The content checks stay whole:
what a ballot says,
which credit lines match the seats seated
and how many distinct seats sit are asserted as before.
The rule of "Output decided by arrival order" stands:
an order that a stage promises (roster order,
input order) is asserted exactly and never after a sort of the case's own,
and what a command prints does not depend on which operation ended first.
This rule covers only the order in which a rotated bench is asked and the seats its window holds,
which no stage promises.
A test made free of the order keeps everything else it pinned:
the text of a file format stays compared as text,
the order a report promises is derived by the documented rule and asserted,
the whole slate is compared and not two entries found in it,
and a case that must seat particular voices pins them by an injected start of the rotation
(`StageSettings.editorBenchStart`,
which no production launch gives)
and not by a membership check (ledger B631).
A case that needs one named seat asked passes `fanOut: 'whole-bench'` to the stage,
or pins `benchStart`,
and never relies on the rotated window to reach it (ledger B701).
The same holds for a case that reads a named seat's own output or ballot:
the rendering of the one interested translator,
or a judge's scripted ballot.
It asks the whole bench in every stage it drives,
the slate and the judging both,
because a seat the first round leaves out comes back empty and the case then reads an absence
(ledger B701,
B812).
The run-off line cases of the translate stage asked four translators at the default window,
so the first round asked three of them by the hash of the prompt.
Under salts 2 and 6 of the sweep the interested translator was left out,
its own word came back empty,
and the judges agreed where the cases expect a decline.
The composite cases of the archive block review (ledger B712) asked the whole bench and did not pin the start,
so the numbers of the candidates followed the sheet text,
and a change to the reply instruction failed five of them (ledger B734):
a seam that takes `fanOut` takes `benchStart` too.
A seam that drives a stage takes `fanOut` and hands it on,
so a case can ask it:
`runArchiveBlockReviewStage` and `runPassContest` did not,
until `20a379f77a` and `a169580740`,
and the translate stage,
the pairing preparation and the corpus-pass admission and preparation seams did not,
until `1305066161`.
A change to the rule that sizes a window moves the rotation into cases that never met it,
with no sheet text edited (ledger B944 to B946).
`1dee3ca797` sized the selection fan-out by the ballots a winner needs and not by a summed weight,
so a bench of four that was asked whole became a window of three,
and the seat left out followed the hash.
`d42e79be74` updated the asked count of one case to three and ran no sweep.
The next sweep found four cases that passed at salt 0:
a straggler held back by seat,
a judge that named a candidate by seat,
and two cases that read the self-vote of the refiner.
The rule is the same and the remedies are the ones named here:
hold or script a judge by its place in the order of call where the case needs one judge of those asked,
and seat only as many judges as the window asks where the case reads a named seat's own finding
and the stage has no seam for a whole bench.
A case that scripts the asked count itself fails when the count changes,
and that failure is the signal to run the sweep.
Before a test is rewritten to ignore an order,
its diff is read for each assertion it drops,
and a guard is removed to show the new test still fails for it.

What enforces it:
habit and review.
`model-order.test-fixture.ts` (`sortedByModelId`)
holds the comparison,
and the ten test files of ledger B627 use it.
`PINNED_EDITOR_BENCH_START` in `editor-bench-start.test-fixture.ts`
seats the six editor voices of the calibration cases,
and the guards of the seam were shown to fail by removal (ledger B631).
No scan finds a test that pins a rotated order,
and none could,
since the order is decided by a hash at run time.
The rotation sweep finds a scripted case that names a seat without either by running it (ledger B701).
The census searched once,
on 2026-10-20 (UTC),
by running every family suite under ten shifts of the rotation,
and the guard built on 2026-10-21 (UTC) repeats that search as a task:
`mise run //package/translation-repair/source-scan:test:rotation-sweep`.
It costs ten full runs of the family's unit tests
(824 s,
builds included,
on 2026-10-21 (UTC)),
so it is not run on every commit.
It runs before a commit that changes the text of a sheet lands,
or the quorum,
the spare or the rule that sizes a window (ledger B944 to B946),
and in the pre-launch checks (`mistake-prevention.md`,
"Before a run launches",
step 13).
A case it lists is fixed by the rule of "Orders and seats that a prompt hash decides",
and the fix is shown to fail again under the salts the sweep named when its pin is taken out.
Between sweeps the habit and review are still the guard.
