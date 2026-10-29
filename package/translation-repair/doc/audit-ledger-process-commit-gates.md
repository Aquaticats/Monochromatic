# Audit ledger: process mistakes, checks run before a commit

Commits made without the suite,
the lint,
the scans or the type check that would have refused them,
a round landed without the round before it (M264),
and a note committed before the Markdown check had run on it (M307).

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M2: a wording change committed without the full suite

Status:
fixed in `0cf2017b3`.
`e8f0b0369` reworded house-policy sentences and bumped a cache version with single-file runs only;
three pins broke (the version 17 key literal,
the corner-bracket sentence,
the place-as-means sentence).
Prevention:
any sheet wording or cache version change runs the full suite right after its commit
(GCE puts the commit first) and before the next change builds on it,
and a version bump repins its key literal in the same commit.

## M4: a guard committed red with a lint warning

Status:
fixed in the following fix commit.
`88fdb1923` carried a `no-mixed-operators` warning
because the lint step sat in an `&&` chain whose `rg` succeeded on warnings.
Recurred on 2026-09-27:
the F-9 guard went in red with seven type errors,
because it passed `sourceText` to a floor that did not yet take it.
Prevention:
commit only after the lint line reads `Found 0 warnings and 0 errors`;
a guard that needs a new parameter gets a prep commit adding the parameter first.

## M10: lint run on changed files only

Status:
fixed in `13c3f51af`.
Every batch linted the files it touched,
so two `unicorn(consistent-function-scoping)` warnings from `0aa800ab4` (2026-09-03)
stood in `provider-budget.ts` and `openrouter-client.ts` for 24 days;
a whole-package run (`oxlint-wrapper.mjs src`,
1666 files,
about 30 to 50 s) showed them at once.
Prevention:
before a batch is called done,
lint the whole `src` tree and read its summary line.

## M12: a floor change committed with its own test file run, not the full suite

Status:
fixed in `b1dc34af2`.
The F-9 fix (the neutral-pronoun floor reads the original) ran only `translate-neutral-pronoun.unit.test.ts`;
`translate-validate.unit.test.ts` pinned the old behaviour and failed until the F-5 work ran it.
The ledger edit that moved M9 also cut M8 in two,
because the region was read short of M8's end;
M8's prevention sat under M11 until this entry's commit.
Prevention:
a floor change runs the full suite before its commit,
as M2 already asks of sheet wording;
a region is read to the next heading before text is inserted after it.

## M23: a red guard committed with its lint warnings unread

Status:
corrected in `f83c4a6b6`,
2026-09-27 (E5).
The panel guard was linted and committed in one `&&` chain;
the lint wrapper exits 0 on warnings,
so six `strict-void-return` warnings landed in `f12a29a3d`
and were read only in the output afterwards.
Prevention:
lint is its own call,
and its `Found N warnings and M errors` line is read before anything is staged;
warnings count as findings here (LN8),
so a chain that gates on the exit code gates on nothing.

## M45: a lint over an empty file list read as clean

Status:
happened 2026-09-28 while fixing X19;
caught by the next command's output.
The file list for the lint came from `git status --short -- .` run in the package directory,
which cli-git's `require-root` guard refused,
so the list was empty;
the lint that read it printed "Found 0 warnings and 0 errors".
The earlier run over the same empty list had printed findings in exactly the changed files,
so the wrapper with no file argument lints something wider,
and a clean result from it says nothing about the list.
Prevention:
a command that consumes a generated list prints the list's line count in the same call first
(`wc --lines`),
and git runs from the repository root (the M1 family's git rule).
The same hour the slice-cache age check (M28) wrote its output to `slice-cache-newer.out` under the tree it searched,
a name its own `-path '*slice-cache*'` matched,
and printed one file written "today";
reading the path showed it was the output itself,
before any conclusion was drawn.
Prevention:
a probe writes its output outside the tree it searches,
or under a name its pattern cannot match.

## M46: a commit whose type check ran before its last edit

Status:
happened 2026-09-28;
`010e36bc5` failed the package's type check,
fixed by `84c748aba`.
The type check ran after the guard's first edits;
the barrel export added last collided with an existing export,
and lint and the tests passed,
so nothing caught it before the commit,
which named no breakage.
Prevention:
lint,
type check and the named tests run after the final edit of a commit,
in that order,
and a commit follows only a clean run of all three.

Recurred 2026-10-01 (UTC) in T8's twenty-first batch,
with a commit message rather than a type check:
`9c007f857`'s message was checked for task and issue numbers,
then a line was appended to it and the commit ran in the same call,
so the check never read the text that was committed.
The committed message read afterwards (`git log -1 --format=%B`) holds none.
Prevention:
the message check is one of the checks that run after the final edit;
any write to a message file starts the check again,
and the commit goes in a call of its own.

## M54: doc commits that never ran the Markdown linter

Status:
happened across the audit;
found 2026-09-28 (D23) and fixed the same hour.
Every commit to this ledger,
the prevention doc and the package's other docs was checked by reading,
by a search for task-list numbers,
and sometimes by a render,
but never by the repository's Markdown linter,
which the repository's lint task runs over every Markdown file.
Prevention:
a Markdown change runs `mise run lint:markdown <files>` before it is staged,
and a `--fix` is rendered against the committed version before it is committed.

## M59: error classes added and marked with only their own tests run

Status:
happened 2026-09-29,
caught by the full suite after B24 and B25,
fixed in `2ea511960`.
`735566798` marked `CacheAccountLogError` and `CacheAccountReadError` safe to print,
and `5bf2388ff` added `ReferenceLineHeadError`,
which writes its own sentence;
each commit ran its own test files and no full suite ran between them.
`message-names-only.unit.test.ts` reads every class in `src`,
so it failed on all three:
two marked classes the inventory did not list,
a `line` part it did not name,
and a class writing its own sentence with neither the marker nor a withheld reason.
This is M12's class again:
a change whose guard is a package-wide scan,
tested only where it was written.
Prevention:
every source commit's test run includes the package-wide source scans
(`message-names-only`,
`dead-functions`,
`duplicate-bodies`,
`tsdoc-example-scan`,
`fixed-length-cuts`,
`clock-time-zones`,
`task-list-numbers`,
`living-doc-links`
and `log-root-scan`),
and a ledger entry is closed only after a full suite has passed on its last commit.

Recurred 2026-09-29 in the coverage census (`f81b58992` to `5fb0af40e`),
with the prevention written and not followed:
four census error classes marked safe to print,
and a copy of the Han residue floor's private line-start helper,
each commit running only its own test files.
The census's first whole-suite run refused on five FAIL markers from `message-names-only` and `duplicate-bodies`;
`e22373347` fixed both.
The scans now run as one task,
`mise run source-scans`,
which builds and runs all fifteen that read the whole of `src` or the docs
(the nine named,
with `position-references`,
`caught-errors-kept`,
`exported-function-names`,
`floor-inputs-stated`,
`text-accumulators` and `rendered-sheets-census`);
its first run passed all fifteen.

Recurred 2026-10-01 (UTC) during B75,
with the task written and not run:
`97aeee2ca` wrote "answers it as below zero" in `command-line.ts`'s region comment,
which `position-references.unit.test.ts` reads as a reference by position,
and none of B75's source commits ran `source-scans`;
the scan failed first on the docs commit's guard run,
naming that line and the same phrase in the B75 entry.
Both now say "under zero",
B73's wording for a count typed with a minus sign.
The task's first run then failed `message-names-only.unit.test.ts` as well:
it listed five sites forwarding a caught error's text into a marked class,
and `97aeee2ca` had removed one,
the coverage census's own `parseArgs` catch.
`5a3e38037` fixes both;
`source-scans` then exits 0 with all 20 suites passing.
The prevention stands:
`mise run //package/module/translation-repair:source-scans` (moved on 2026-10-18 to
`//package/translation-repair/source-scan:source-scans`) runs before each source commit,
beside its named tests.

Recurred 2026-10-01 (UTC) at the close of T8's twentieth batch,
through a gap in the rule itself:
the rule named source commits,
and the batch's docs commit `fb16c5128` ran only the Markdown linter.
Its B84 entry said a commit comment "gives the count above",
which `position-references.unit.test.ts` reads as a reference by position;
the full suite on `7b61bb928` failed that one package-wide case.
The clock-time,
task-list-number,
living-doc-link and position-reference scans read the docs as well as `src`,
so a docs commit can fail them as readily as a source commit.
Prevention:
`source-scans` runs before every commit that touches the package's source,
docs,
README or task file,
and the task's description and `mistake-prevention.md` now say so.

## M105: a commit across 139 sources verified by named test files alone

Status:
happened 2026-10-01 (UTC) in `de323fc97` (B98),
found by the next full suite,
fixed in the commit adding this entry
and corrected by a comment on `de323fc97`.
B98 chose the noun after `corpus-run/bench-report.ts`'s slice count by the count,
so a group of one slice now prints "1 slice".
`corpus-run/bench-report-groups.unit.test.ts` cut each summary line at " slices"
and expected the bare count,
so its group of one cut nothing and the case failed.
The commit's runs were the test files its list named,
and the list did not name that file;
its message called them "the 21 test files the change touches",
which reads as every test file reaching the change.
The case now cuts at the line's parenthesis and expects the noun with its count,
so it also holds the group of one to the singular.
Prevention:
a change across many sources runs the full suite before its commit,
and a message names a named-file run as the files it lists,
never as the files the change reaches.

## M114: a commit left without its lint, and a setup command read by its task status

Status:
both happened on 2026-10-05 (UTC) and were caught the same day.

The commit.
`f0f449765` corrected eight test files and was made on named tests alone;
its message said the whole suite,
the lint and the scans would run with the following change.
Work went to agents instead,
and the gates started only when a reviewer's advice asked for them.
Run on that commit itself,
they passed the suite and the scans and read `Found 1 warning and 0 errors.`:
an expectation written `3072` where the lint wants `3_072`
(`openrouter-cached-tokens.unit.test.ts`).
`30c9bd8e6` fixed it,
sixteen minutes after the commit by the two commit stamps.
Twelve agent worktrees then stood at `f0f449765` (`git worktree list`),
so each agent's lint reported the warning and each had to be told it was known.

The setup command.
The script that creates agent worktrees was run under `mise exec --`,
where `git` is the plain program and not the repository's wrapper;
plain `git` does not know `--no-worktree-copy`,
and the script stopped at its first worktree,
twice.
Each run was in the background behind `|| echo "exit $?"`,
so its task reported exit code 0.
The report was taken for the result,
neither log was read,
and an agent was launched into a worktree that did not exist.
It was found when a `cd` into another of those worktrees failed;
the worktrees were then made without `mise exec --` and the agent was told.
`mistake-prevention.md` already had the rule,
under "Tasks,
builds and bulk output" (ledger B103):
a background run is read by its own log,
never by the task's reported status.

Prevention:
a commit that defers its gates starts them in the background in the call after it;
"with the next change" names no time.
A command that sets up what a later step uses is read by its log before that step starts,
and a path an agent is sent to is listed in the call that sends it.

Recurred on 2026-10-05 (UTC),
the lead's own:
merging `341211543`,
the lead edited `coverage-control-probe.ts` beside the agent's patch
and committed after `lint:types` and the named tests,
without the source scans;
the commit's message said so.
The count-nouns scan then failed the whole suite and the scans on `ba243b3c3`,
on a line the lead had written,
and `d233f882f` reworded it.
A hand edit made while merging is checked like any other change:
the scans run before the commit,
not only in the gates after it.

## M123: a red commit made without the source scans

Status:
happened 2026-10-06 (UTC) in `f45278048`,
found on the fix tree,
fixed in `71974714f`
and corrected by a comment on `f45278048` (commitcomment-203621236).
The red commit's message names three test files as failing on purpose.
The `duplicate-bodies` scan failed there too:
two test files,
written by two agents and landed in the last two test commits,
each projected an envelope with one body.
The lead had run the named test files and the type check before that commit,
and not the scans.
M114 records a commit left without its lint;
a red commit adds test files as well,
and the scans read those.
The fix commit adds `envelope-span.test-fixture.ts`,
which both test files use.
Prevention:
the source scans run before a red commit too,
and a message that names what fails names every failure of its tree.
`mistake-prevention.md`,
"Lint and edits".

## M124: a count of matching lines taken as proof a form was in use, and a commit without its lint

Status:
happened 2026-10-06 (UTC) in `71974714f`,
found by the whole-suite run on that commit,
fixed in `230623bec`.
The fixture M123 records exported `envelopeSpanOf`,
which two test files passed by reference to `map`.
Before choosing that form the lead counted lines matching a pattern for it in the package's tests,
took 26 matching files as proof the form was in use,
read none of them,
and ran no lint.
The linter warns on the form where it cannot see the function's arity across an import
(`no-array-callback-reference`),
and the run on `71974714f` ended its lint with one warning.
`envelopeSpansOf` takes the envelopes and maps them itself.
The commit went out without a lint run,
which M114 already forbids.
Prevention:
precedent is a match that was read,
beside the check that enforces the form,
never a count;
and lint runs before a commit that adds a helper,
not the scans and the type check alone.
`mistake-prevention.md`,
"Searches and censuses that miss" and "Lint and edits".

## M133: a doc changed in a fix commit went out without its Markdown lint

Status:
happened 2026-10-06 (UTC) in `3d36ba7b9`,
found before the next commit,
fixed in `c8196cf6c`.
The start guard's fix added a paragraph to `doc/configuration.md` (moved on 2026-10-18 to
`package/translation-repair/doc/configuration.md`).
The lead ran the Markdown lint for docs-only commits,
and this was a code commit that touched a doc,
so the lint was not run;
it reported three `semantic-line-breaks` findings on the paragraph,
on lines 191 and 200.
Prevention:
the Markdown lint runs on every doc a commit touches,
whatever else the commit holds
(`mise run //:lint:markdown <paths>`).
`mistake-prevention.md`,
"Lint and edits".
Recurred on 2026-10-11 (UTC) in `ee0706879`:
the lead fixed the findings of a lint run and committed without running the lint again,
and one line the run had flagged was left unfixed in `doc/design-commitments.md` (moved on 2026-10-18 to
`package/translation-repair/doc/design-commitments.md`)
(a bold item title broken after its comma by the next commit).
The rule now reads:
the lint's last run before a commit is after the last edit,
and its output holds no finding.

## M164: the lead checked the split's test edits without the source scans

Status:
happened on 2026-10-12 (UTC).
Before committing the docs split,
the lead ran the seven tests that read the docs,
the package lint and the Markdown lint,
but not the source scans,
though the shared facts every brief cites name them as a step after any edit under `src`.
The split exported `withParts` from `living-docs.test-fixture.ts`
while only that module calls it,
and gate g81 failed the dead-declaration scan (ledger B118) on it.
Fixed in `33f4599514`.

Prevention:
the lead's own edits under `src`,
fixtures and tests included,
take the same final measurement as a brief asks of an agent:
the touched files by name,
the source scans,
the lint,
then the whole suite or a gate.

## M173: the commit path lists went stale before the commit

Status:
found on 2026-10-13 in the merges of the sixth-review round.
The lists of paths for each commit were written at merge time,
and went stale as soon as the next author added files.
The second round of the linter migration generated its lists from `git status --porcelain --untracked-files=all -z`
just before the commit,
and found 100 entries in two lists.

Prevention:
path lists for a commit are generated from the worktree status immediately before the commit,
and at no other time.
`mistake-prevention.md`,
"Commit messages".

## M264: a round recorded in the handover as done was never landed, and its successor conflicted

Status:
happened on 2026-10-18 (UTC) with the two rounds of `w186`.
The first round (`074599761e` to `dcf55b5eae`,
B668 to B671) was recorded in the handover as done,
and was never cherry-picked onto the branch.
Landing the second round (B672 and B673) alone conflicted on the ledger file that the first round had created,
and the conflict showed that the first had not landed.
Both rounds landed together after gate g104
(the gate was green on `67c9b1accb`,
with the landing pushed as far as `ce7be13112`).

Prevention:
before a range is landed,
its base is checked to be an ancestor of the branch,
and every earlier round from the same worktree is found in the branch's `git log`.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Work merged from parallel briefs".

## M266: a test-only round was reported done before the scans of the package family had read its tests

Status:
happened on 2026-10-18 (UTC) in the test round `w188`.
The round ran `build`,
`lint:types`,
`lint` and `test:unit` for the seven packages it touched,
and did not run `source-scan`.
Its `lint:types` and `lint` findings (types in two test files and an `await` in a loop) came after the pins,
and were fixed in `ce66f2cb07` and `58b9d98583`.
When the round was ported onto the branch of generation 19,
`source-scan` found two problems in its tests:
two copies of the ballot builder in the run-off recount tests of `artifact`,
now `ballotsQuotingSecond`,
and two copies of the torn-down body in the drain tests of `provider-stream`,
now `tornDownResponseOf`.
They were fixed in `5c1d1aef1b` with the same ballots and the same body.
The same run found a third problem of the sol round's own:
a word of position in the name of a test in `consistency-markup-ranges-unplaced.unit.test.ts`,
which that commit also fixed.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a round that adds or changes tests runs the `source-scan` task of the package family
beside `lint:types` and `lint` before it reports,
because the scans read tests for duplicate bodies,
words of position and times without a zone.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Lint and edits".

## M287: a bench change landed with the run package's tests only, and the corpus-pass tests that read the bench failed at load

Status:
happened on 2026-10-22 (UTC) in `tr-w277`,
at the change of B815.
`9639e19d85` took `hf:Qwen/Qwen3.8-27B` off the late-judge bench
and ran the run package's tests only;
it did not run the corpus-pass package's tests.
`pass-seams-reseat.unit.test.ts` there reads `judgeSeatsFor` and has a fixture guard
that throws when the fixture views seat the same late judges
("so no case here can tell a wired hook apart"),
and with Qwen gone a Synthetic dry-out takes no late judge,
so the guard threw at load and the file ran no case.
`89067fb49a` fixed it,
and its message says it was committed before its rerun finished.
That rerun's log (outside the repository,
`~/temp/agent/tr-fan/w274-audit/reseat/corpus-pass.log`) holds `JUDGE SEATS` lines from the file's cases,
so the file loaded and ran,
and its six `[FAIL]` lines name two other cases.
Measured on 2026-10-22:
of the 31 views with at least one provider dry,
4 take a late judge (`hf:moonshotai/Kimi-K3`),
Synthetic and Hyper dry with OpenRouter wet and any of NeuralWatt and Bedrock,
and the contest and consolidation reseat cases now read a Synthetic-and-Hyper-dry view.

A second miss in the same rerun:
`eb4db609ff` moved `CONSISTENCY_CACHE_VERSION` to 7 and `PAGE_TITLE_CACHE_VERSION` to 6 (B816),
and the two cases that name them in the corpus-pass package,
`pass-consistency-cache.unit.test.ts` (version 6 and its pinned key) and `pass-page-titles.unit.test.ts` (version 5),
are the two failures of that log.
`205eed4f57` moved both pins.
This is the shape of M284 again:
the check after a version change ran the declaring packages,
and corpus-pass declares neither constant.

A third miss,
found when the unit tests of 16 packages ran on `205eed4f57`:
five more packages failed on pins the same commits moved.
The pinned keys of the repair slice,
refine,
lane-contest,
consolidate and translate slice caches still held their old hashes,
and 22 expected lines in three `verify-published` test files of the run package still read `read-at-generation=19`,
though `19dc276377` writes generation 20.
`aed73b727b` moved them,
with each new hash read off the failing run,
and the five packages passed on the rerun.
The same check found three more slips.
The cache-bump script behind `eb4db609ff` had split the code span
`git merge-base --is-ancestor` across two TSDoc lines in three accounts.
Two generation-20 TSDocs said "one voice below them",
which the position-references scan refuses.
The real-clock scan did not list the grace windows and the stuck-seat timer the B816 cases added.
`8de619e404` fixed all three.

A fourth miss,
found by gate g164 on `b1a6ea4e10`:
the commits that applied the owner's answers of 2026-10-22 ran neither the provider package's tests nor the linter.
The Qwen hold `late-judge-dropped` sat on its card,
and `roster-seats.unit.test.ts` in the provider package,
which holds every seat's holds against its card,
still listed only `openrouter-dropped`,
so two of its cases failed.
The linter found three files past `max-lines`
(`model-cards.ts` at 303 counted lines,
`stage-quorum.ts` at 302,
`living-docs.test-fixture.ts` at 306),
each pushed over by lines the answers added,
and two warnings in the stage package:
`STUCK_MS = 1000` without a digit separator,
and the last-seat grace condition on one line.
`a6eb72b182` fixed all five:
the claim lists the new hold,
and each long file moved one concern into a sibling
(`openrouter-card-sides.ts`,
`stage-gather.ts`,
`markdown-link-targets.test-fixture.ts`).
This is M46 again:
lint and the named tests run after the final edit of every commit,
and an owner's answer applied as a data edit is such a commit.

Recurrence:
M284,
which records the same check for cache versions;
the prevention of M284 names constants that key a cache,
and nothing in it named a bench.

Prevention:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Which seats a quorum counts":
a change to a bench or a roster
(a hold on a model card,
a roster list,
a seat set that `judgeSeatsFor` builds)
runs the unit tests of every package that reads `judgeSeatsFor` or the bench constants,
and not only the package it edits.
For the third miss,
`mistake-prevention-stored-files-and-cached-decisions.md`:
the runs that follow a version move start after the bump commit,
and a move of the artifact generation also searches the old number as text,
since a printed generation sits in tests that name no constant.
For the fourth miss,
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Lint and edits":
a hold or list entry added for an owner's answer runs `lint:oxlint` on the package it edits,
and a hold runs the provider package's tests as well.
The search is `rg --files-with-matches '<NAME>' package/translation-repair --glob '*.ts'`
for `judgeSeatsFor` and each constant that moved;
on `9ed9b5d14a` it finds corpus-pass,
probe,
repair,
run,
stage and translate-stage.

## M288: about 30 gates took five minutes longer than the gates before them, and nobody compared the time

Status:
happened from g138 (`3637592bd4`,
2026-10-21 (UTC)) and was found on 2026-10-22 (UTC) in `tr-w290`,
by an implementer who asked why the corpus-pass package's `test:unit` took five minutes.
The defect is B817.

What happened:
in every gate up to g137 (at `7c9d5fa35f`) that package's `test:unit` took 43 to 85 s.
From g138 it took 305 to 313 s in every gate,
about 30 gates in all,
and each gate read its `[FAIL]` count (zero) and went on.
A case of `pass-reseat.unit.test.ts` slept a 300000 ms hold for real and then passed (B817).
The rotation sweep runs that package's tests on every salt,
so the case ran on all ten.

Why nothing flagged it:
every check a gate runs reads a count,
and the count did not move.
The wait was constant,
so it never looked like a flaky case,
it sat inside one block of parallel cases,
so no case reported it,
and no case failed.
A gate's wall time was read as a cost and not as a measurement:
305 s in one gate was the same 305 s in the next,
and a figure that does not change from gate to gate is not noticed.
Nothing in the gate list compared a package's time with the gates before it.

How it was found:
the log of the package's `test:unit` prints each block's duration after its name,
and the block of the lanes hooks read 300.3 s,
which is the router's hold of 300000 ms to within the start-up of the block.
A duration that matches a constant of the code under test is a sleep.

Recurrence:
M2 and M12,
which record a gate read by its exit code and not by its `[FAIL]` lines,
and B813,
a real-clock wait that held in every run and was wrong in one.
Those mistakes read a result too weakly;
this one read a result and skipped its duration.

Prevention:
`mistake-prevention-tests-clocks-and-arrival-order.md`,
"Tests on the real clock",
and step 2 of "Before a run launches" in `mistake-prevention.md`:
a gate compares each package's `test:unit` time with the same package's time in the gates before it
whenever the time jumps,
reads the block durations in the log,
and takes any block whose duration matches a constant of the code under test as a sleep to be found.
The real-clock scan reads the holds a test registers for a provider (B817).

## M293: a count was expected from the corpus-pass test runner, and it prints none

Status:
found on 2026-10-22 (UTC) by the implementer of the review fixes of B818,
who recorded it in the notes of that round.

The test runner of the corpus-pass package prints no count of cases or files,
so a gate that looks for "N passed" finds nothing,
and a gate that takes the absence for a pass has read nothing.
The same output also prints a `[PASS]` line for a suite with failing cases (M125),
so that line alone proves no more than the count did.

Prevention:
`mistake-prevention-tasks-builds-and-bulk-output.md`,
"Tasks,
builds and bulk output":
that package's gate reads the exit code,
the `[FAIL]` lines (zero),
and the `[PASS]` lines of the suites the change added or touched,
by name (M288,
and `AGENTS.md` rule TRV:
a green run counts only when its output holds the expected suite and test names).

## M294: the gate scanned the tree before a merge of main, and the merge brought in three doc lines that fail the source scan

Status:
fixed on 2026-10-22 (UTC).
The three lines were fixed in `b18a4f7a40`,
and the land script now holds the check that would have refused them.
Found when main failed `source-scan` after the land merge `ecbbae7fa3`;
gate g171 read the repaired tree green at 2803 tests and none failing.

What happened:
the gate that cleared the landing scanned the tree as it stood before main was merged into the translation-repair branch.
The merge `ecbbae7fa3` (main at `1d46232457` into the branch) brought in three doc lines
that the source scans refuse:
a position reference with no named target in the evidence file of the `CuspariaKLSY23` read,
another in its stages file,
and zone-less clock times in the stages file of the `zheermao23` read.
Each side had passed its own scans.
The scans that read docs (clock times,
position references,
the Markdown linter) read the whole tree,
so a merge of docs can fail them though neither parent does,
and main stayed red until `b18a4f7a40` named the targets and zoned the times.

Why nothing flagged it:
the scan was run on the parents and not on the result,
and a merge commit is the one tree no author and no gate had seen.

Prevention:
the land script runs `//package/translation-repair/source-scan:test:unit` on the merged tree whenever it merges main,
and refuses to push when it fails.
`mistake-prevention-docs-and-references.md`,
"The Markdown linter",
holds the rule and its limit:
the scan imports the gate's `dist` builds,
so a merge that changes markup,
typography or document structure needs a fresh gate and not the scan alone.

Recurrence:
M250 and D35,
where docs-only commits landed without the one task that reads the line limit and main went red.

## M307: a note was committed before the source-scan Markdown check had run on it

Status:
fixed on 2026-10-29 (UTC) in `d4fbf7d6e9`,
recorded the same day from the lead's notes.

The lead's own note in B965
(`audit-ledger-providers-and-seats-a-run-holds-out.md`,
commit `832383a4c4`)
failed the `semantic-line-breaks` check at its lines 980 and 981.
Each of the two lines ran on past a comma inside a parenthesis.
`d4fbf7d6e9` breaks them at their clauses and changes no fact.
The cause was that the note was committed
before `mise run //package/translation-repair/source-scan:test:unit` had run on it,
and the note was one the lead wrote,
not an agent's.

Prevention:
`mistake-prevention-docs-and-references.md`,
"The Markdown linter":
that task runs before the commit of any Markdown edit,
the lead's own included,
whatever the size of the edit.

Recurrence:
M300,
where a brief named the root lint task and a ledger went to a gate with two findings,
and M133,
where a fix commit changed a doc without the Markdown lint.
