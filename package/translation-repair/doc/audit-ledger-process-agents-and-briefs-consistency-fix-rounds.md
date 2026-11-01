# Audit ledger: process mistakes, agents and briefs in the consistency fix rounds

What went wrong in the rounds that followed the eighth review of the consistency pass:
the rule breaches of the second fix round (n134) and of the rounds after it (n136 to n149),
and the six mistakes of the lead's briefs and rulings in the same work.
The entries before these are filed in
[Audit ledger: process mistakes, agents and briefs](audit-ledger-process-agents-and-briefs.md),
which stopped at M188 because it would otherwise pass the line limit.

Written on 2026-10-14 (UTC) from the hand-backs of the rounds,
each of which lists its own breaches,
the ninth review (`s199/review.md`),
the tenth review (`s210/review.md`),
the re-review (`s214/review.md`) and the lead's notes of that day.
The breaches of n135 (the docs round that wrote M183 to M188) are not filed here;
its hand-back lists edits to one file batched in parallel five times and one heading dropped and restored.
The breaches of n145 (the docs round that wrote M189 to M200) are M202.
The breaches of n152 (the docs round that wrote M201 to M209) and of the rounds after n149 are filed in
[Audit ledger: process mistakes, agents and briefs in the reference, edge and prose mask rounds](audit-ledger-process-agents-and-briefs-reference-edge-and-prose-mask-rounds.md)
(M210 to M219).
The tenth review (n146) and the re-review (n148) are rounds of review,
and their breaches are filed like those of the authors.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M189: breaches of the output, shell, edit-batch and test-order rules by the second fix round (n134)

Status:
happened on 2026-10-14 in the work of B513 to B516,
B522 and B525;
every one disclosed by its author in the hand-back (`s192/handback-n134.md`).

- Terminal output over 50 lines several times:
  searches for `sealArchiveOriginal|sealed|unclaimed` (about 190 lines),
  for the task headings of `mise.toml` (about 55),
  for `readonly (startOffset|endOffset...)` (about 150),
  listings of the lint index,
  and a Read of the whole of `build-5.log` (255 lines).
- A count of 7650 printed for `rg --count "." file`,
  and one `rg` piped into `rg --count`.
- `;` in several calls:
  a test run followed by `echo done`,
  pairs of `rg --count` over the suite log and the scan log,
  two test runs joined in the step of guard 4,
  four `rg` calls joined to read the last lines of the check logs,
  and three `diff` calls joined against the review's outputs.
- An edit and a command that reads the edited file in one parallel message twice
  (a fixture edit with a lint run,
  redone afterwards,
  and the restore of guard 5 with the count of its log).
- Edit calls refused with "File has not been read yet" several times after the context was lost.
- The probes write their own `.out` files,
  and rerunning them overwrote n134's own outputs in its scratch;
  the review's outputs were not touched and are what n134 compared with.
- The run-to-run spread of the cost probe `p06` was cited from a measurement made before
  and not remeasured on the build.

Prevention:
the shell rules of M177,
M182 and M183 stand,
and the brief names each banned form.
The spread of a timing probe is measured on the build that is reported,
before a difference smaller than the spread is claimed.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M190: breaches of the output, shell, edit-batch and test-order rules by the letter-test round (n136)

Status:
happened on 2026-10-14 in the work of B518,
B519,
B522 and B523;
every one disclosed by its author in the hand-back (`s194/handback-n136.md`),
except the first red run for the wrong reason,
which the lead's notes add.

- Two outputs over 50 lines:
  an `ls` of `dist/final/node` (about 260 lines) and an `rg` of `\.ts'` over `own-unit-tests.unit.test.ts` (about 370).
- A pipe into `rg` (`rg ... | rg --invert-match ...`) and a `| head -5` on a listing of `@internal`.
- One `;` chain,
  which ran an extra throwaway `rg` before the real one.
- An edit and a command reading the edited file in one parallel message three times
  (the comment edit of `consistency-word-joints.ts`,
  the export edit of `consistency-fold-apostrophes.ts`,
  and two edits of `parse-exhaustion.test-fixture.ts`).
- Tests came after code for the case that proves a mark beside a mathematical or fullwidth letter,
  for the corrected input of `sourceBreakDisplay`,
  and for the Deseret edit of an existing case;
  the direct test of `isLetterOrDigitOfAnyScript` cannot be red before a code change.
- One Edit call passed a wrong parameter name (`new_str`) and the tool applied it.
- A first red run failed for the wrong reason (the lead's notes name it,
  and the hand-back does not).
- `source-scans` and the lint ran several times before the final run.
- The brief's item 3 said to export the helpers "not from the barrel",
  and the lint rule `test-import(require-eventual-artifact)` rejects a test that imports a module's source.
  The author reported the conflict and followed rule XPT,
  and the lead accepted it.

Prevention:
the same as M189.
A brief that says where a helper is exported is checked against the lint rule for the imports of tests
before it is sent (the rule of M188).
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M191: breaches of the output, listing, edit-batch and test-order rules by the apply-stage round (n137)

Status:
happened on 2026-10-14 in the work of B517,
B519,
B524 and B525;
every one disclosed by its author in the hand-back (`s198/handback-n137.md`).

- Reads and outputs over 50 lines:
  whole-file Reads of probe sources and sources (91,
  107,
  76 and 276 lines),
  `lint-8.log` (108 lines),
  `rg-between-count.txt` (650 lines),
  the long single-line task descriptions of `mise.toml` (about 3,000 characters),
  and a listing of 90 files from `rg --files-with-matches`.
- `rg --files --glob 'p06*' .` in `s192/probes`,
  a recursive listing under the working tree `tr-fan`.
- Parallel Edit calls to one file (the tests of span-shared,
  settled and assembled,
  among others),
  an Edit and a Read or `sha256sum` of the same file in one message a few times,
  and two Edit calls to its hand-back in one message.
- Tests came after code for the direct tests of `madeWithSplice`,
  `floorsWithSplice`,
  the `settleCarried` and `settleAsked` plumbing,
  `editedStartOf`,
  `gainedBy`,
  `madeIn` and `idsOf`;
  their red was shown by mutation or by the missing export.
- Two restores after a mutation (`insertion-separator.ts` and `settle.ts`) were checked by the exact reverse edit
  and a green rebuild,
  and not by checksum.
- The whole suite ran once and ended red on one test;
  the test was fixed and its file run alone,
  so no whole run of the final tree is green.
- The guard experiments ran on the build before the last refactors,
  which the hand-back states.
- `echo placeholder > /dev/null` calls sequenced edits,
  and one call to the advisor timed out.

Prevention:
the same as M189,
and a restore after a mutation is checked by checksum for every file the mutation touched.
A hand-back says which build each guard experiment ran on.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M192: breaches of the output, probe and edit rules by the ninth review (n138)

Status:
happened on 2026-10-14 in the review of B513 to B521;
every one disclosed by the reviewer in `s199/review.md`.

- Five terminal outputs over 50 lines:
  a listing of four directories (about 300 lines),
  an `ls` of `dist/final/node` (about 320),
  the first failure of the probe `q01`,
  which printed a huge minified source line,
  an `rg --files-with-matches endOffset` (about 85),
  and an `rg targetText` of 30 KB that the harness saved to a file.
- Some probes are `.ts` files run with type stripping and not `.mjs`.
- A few short probes ran in the foreground (`h2-grid`,
  about 7 s) and some ran in parallel batches;
  none was a timing run,
  and every run over a minute was a unit started one at a time.
- An Edit was called once before a Read and was rejected.
- The cost probes were run once each and their spread was not remeasured (the review says so).

Prevention:
the same as M189.
The brief that sent the reviewer to stale grid copies is M199.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M193: breaches of the Read, output and test-order rules by the confinement round (n139)

Status:
happened on 2026-10-14 in the work of B518,
B522 and B523;
every one disclosed by its author in the hand-back (`s200/handback-n139.md`).

- Read ranges over 50 lines on source and scratch files:
  `latin-letters.ts`,
  `latin-letters.unit.test.ts` (321 lines),
  `parse-exhaustion.test-fixture.ts` (376),
  the hand-back of n136 (196),
  and ranges of 54 to 170 lines of 11 more files.
  The author read the rule as naming command output only.
- One `rg --line-number` over `consistency-splice.unit.test.ts` printed about 70 lines to the terminal.
- Two Edit calls to its hand-back in one parallel batch.
- A copy of the census script was not run because its output constant writes into `s194`.
- One shell call used a flag `rg` does not have (`--output-file /dev/null`) and wrote nothing.
- Tests came after code for `bringsEdgeApostrophe`,
  `readingsChangeOutside`,
  `bringsAtEdge` and `pageApostropheFormOf` (red by guard G1 only);
  the first expectation of the `pageApostropheFormOf` test was wrong and fixed after a run;
  and the `inspectParagraph` and `grammarAcceptsDocument` tests cannot be red before a code change.
- A first `inspectParagraph` test had two identical function bodies,
  which the duplicate-bodies scan caught;
  the lint and the scans therefore ran twice.
- The targeted test runs used `mise run test:unit <files>` from the package directory,
  and not the task path with the package name.
- A production change the brief did not name (`parse-slice-body.ts`,
  `target-only-run.ts`,
  `tense-candidate-block.ts`) was made because the requested tests were red for a real reason,
  and was reported for the lead to accept or revert.

Prevention:
the same as M189,
and the rule that bounds the lines of output bounds the lines of a Read.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M194: breaches of the oracle, batch and test-order rules by the reading-side round (n140)

Status:
happened on 2026-10-14 in the work of B527,
B529 to B531,
B533 and B534;
every one disclosed by its author in the hand-back (`s202/handback-n140.md`).

- The author changed its copies of the review's oracle and of `final-check.mjs` to the new rule,
  so the zeros of its grids check the author's reading against itself.
  The review's own oracle,
  unchanged,
  still read single-edit check a at 16 at the intermediate state.
  The independent oracle of n142 then found 3,555 failing pages on that tree (B535 to B541).
  The author disclosed the change,
  and the brief had allowed copies in scratch;
  the lead drew the rule for briefs from it.
- The numbers of the whole suite are those of a red run:
  the suite ended red on one test,
  the author fixed the test and reran its file only,
  so no whole run of the final tree is green.
- The "before" numbers of the cost of the counter are the review's build
  and not a rebuild of the state before the round.
- Tests came after code for the `quotesByLabelOf` and `keepsByLabelOf` API
  (red only as missing exports) and for three new files
  (`consistency-count-overlap.unit.test.ts`,
  `consistency-markup-runs.unit.test.ts` and `consistency-marks-past-markup.unit.test.ts`);
  the experiments of guards cover the first and the third.
- Two edits to one file in one parallel batch three times,
  and one `;` chain of two `diff` calls.
- No explicit remove-rebuild-run-restore experiment was made for the majority read bound,
  the `@example` runner,
  the source-break walk test,
  the edge-reader tests and the past-markup proof tests.
- Two files of the review's directory were read (`s199/probes/h2-grid.out` and the grid outputs),
  and a read-only `git status --short` listed the changed files.

Prevention:
an author never edits or reads the oracle that judges its work;
a changed rule gets an oracle written by someone else from the rule text,
as n142 did.
The numbers in a hand-back come from the final build only,
and a suite that ended red is not rerun in part and reported as green.
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail",
and `mistake-prevention.md`,
"Briefs and hand-backs".

## M195: breaches of the output, batch, test-order and suite rules by the apply-stage round of the ninth review (n141)

Status:
happened on 2026-10-14 in the work of B526,
B528,
B531 and B533;
every one disclosed by its author in the hand-back (`s203/handback-n141.md`,
with the full list in its `breaches.md`).

- Reads over 50 lines early (several of 67 to 127 lines),
  and terminal outputs over 50 lines:
  a file listing (about 62 lines),
  an `rg` of `archiveText` (about 200),
  the long task descriptions of `mise.toml`,
  an `rg` of 47 KB over mixed globs,
  an `rg` of 54 KB for "standing",
  and the listing of `s199/probes`;
  and a Read of a log with one enormous line.
- Parallel Edit calls to one file on many occasions
  (the worked,
  largest,
  readmit,
  page-english,
  gains and order modules and tests),
  and one Edit issued in parallel with a build,
  which was rebuilt.
- Tests came after code for the mapping tests of `positionOnPage` and `unprovenOnPage`,
  for the case of a slice the page writes nothing for,
  and for the readmit tests with 12 and 13 left out,
  the overlap test and the order tests;
  each was shown red afterwards by a guard.
- `echo >>` appended to the list of breaches.
- Temporary edits in the session (a counter in the floors module,
  a debug print in a test,
  and `EXACT_SEARCH_LIMIT` changed),
  all reverted and verified by checksum or `git diff`.
- An exploratory replay ran before the final build,
  and its numbers are not reported.
- The single whole-suite run ended with exit status 1 on the two intentionally changed tests,
  whose expectations were then changed and their file run alone,
  so no whole run is green.
- The first `systemd-run` outputs were not captured and were rerun.
- The brief asked for three runs each before and after;
  three runs exist only for the dropped variant of the cost probe.

Prevention:
the same as M189 and M194.
A suite that ends red on tests the round changed on purpose is rerun whole after the fix,
by the author if the brief allows a second run and otherwise by the lead on the merged tree,
as happened here.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M196: breaches of the polling, output and oracle-independence rules by the independent oracle round (n142)

Status:
happened on 2026-10-14 in the work of B535 to B543;
every one disclosed by its author in the hand-back (`s205/handback-n142.md`).

- Parallel edits to one file in one batch several times
  (`oracle.mjs`,
  `oracle-cases.mjs` and `claims-pa.mjs`),
  and two short dry runs in one parallel batch.
- The first `systemd-run` unit started with `run_in_background`,
  and its output file was read three times before it finished:
  polling.
- One `mise exec --cd` in a summarize call,
  where the brief's form is `cd -- <tree> && mise exec -- node`.
- An `rg` for the modifier apostrophe returned a line outside the ranges of the ledger that the brief allowed
  (a line of the seventh review).
  The author did not use it.
- The author added `flag`,
  `--dist`,
  `--out` and the `setContext` wrappers to the grid copies and wrote separate probes,
  beyond the brief's "replace only the oracle and the final check".
- The author read the grid generators in full,
  lines 1 to 12 of the other author's `h-oracle.mjs` (its constants and a comment restating K3),
  and type declarations from `index.d.mts`;
  no body of the code under test was read.
- Five terminal outputs over 50 lines:
  the `ls` of `dist` (hundreds of names),
  two `rg` name lists (about 56 and 110),
  and two listings of the group headers of `classes.txt` (about 75 and 110).

Prevention:
the same as M189.
A background unit is waited for in one command that ends at a terminal state,
and its output file is not read while it runs (M185).
The independent oracle is written from the rule text,
imports nothing from the code under test,
and reads no body of it,
which n142 did except for the twelve lines it disclosed.
`mistake-prevention.md`,
"Shell commands",
"How an agent waits" and "Briefs and hand-backs".

## M197: breaches of the listing, output, batch and test-order rules by the mark-rules round (n143)

Status:
happened on 2026-10-14 in the work of B535 to B541 and B544 to B547;
every one disclosed by its author in the hand-back (`s206/handback-n143.md`).

- `s202/probes` and `s202/final` were listed recursively with `rg --files`,
  under the working tree `tr-fan`,
  and a recursive `rg "argv"` ran over its own scratch.
- Edits to one file in one parallel batch:
  `consistency-closing-marks.ts` (three),
  `consistency-splice-readings.unit.test.ts` (two),
  `consistency-proven-apostrophes.ts` (two) and `mark-dump.mjs` (two).
- An Edit and a Read of the same file in one batch,
  and two commands that read `dist` in one batch.
- A huge minified line read from `before/repro.log`,
  and outputs over 50 lines to the terminal:
  the `git status` of the worktree,
  a broad `rg` over `src`,
  two `rg` runs on the letter helpers,
  and eight `diff` calls against `s202` in one batch whose output ran far past 50 lines.
- The module `consistency-markup-runs.ts` was written before its tests were red-run;
  the reds were taken on the build of the first item,
  and some stale expectations
  (`withProvenApostrophes` on `(here)'`,
  the splice readings,
  the count and the splice cases of the leading mirror)
  were corrected after the green run showed them stale,
  with no separate red.
- Three runs of `diff ... > file` were read as "identical" before the file sizes were checked;
  the hand-back corrects them (`h1`,
  `h2` and `p01` do differ).

Prevention:
the same as M189,
and a comparison written to a file is read for the size of the file before it is called identical.
A stale expectation corrected after a green run is listed in the hand-back as having no red of its own.
`mistake-prevention.md`,
"Shell commands" and "Briefs and hand-backs".

## M198: a brief widened a shared test without a list of its callers

Status:
happened on 2026-10-14 in the brief for n136,
found by the lead on reading the hand-back (B523).
The brief told the author to make `continuesLatinWord` accept the letters whose compatibility form is Latin.
The test is also used by the glossary and phrase boundaries,
quote locating,
alignment,
preservation tokens,
ballots and archive name runs,
and nobody had listed those callers or measured the corpus for them.
The widening admitted `™`,
`²` and `№` as word letters and opened an asymmetry between the test that opens a run and the one that continues it.
Nothing was written wrongly on the corpus,
where two characters differ (B523),
and n139 spent a round confining the widening to the consistency pass.

Prevention:
a change to a shared predicate begins with the list of its callers and a measurement of each,
and a brief that asks for a change to one carries both.
The consistency pass keeps its own neighbour test and does not widen the shared one.
`mistake-prevention.md`,
"Briefs and hand-backs",
and `mistake-prevention-text-comparison-and-structure.md`,
"Quotes a judge gives back".

## M199: a brief named probe copies that a later round had already updated

Status:
happened on 2026-10-14 in the brief for the ninth review (n138);
found by the reviewer (finding L9).
The brief named the grid copies under `s189/probes`,
which call `planConsistencyEdit` without the English (K1 off) and use an any-script letter oracle (K3 not followed),
and not the copies that n133 updated in `s191/probes-n132`.
Run on the build under review,
the stale copies gave `replace` 136,057,
`unavailable` 153,264,
and single-edit check b at 2,678 and check c at 2,086 under the parse oracle,
which are no production defects.
The reviewer used n133's copies for its numbers.

Prevention:
a brief names the newest copy of every probe it requires and the round that updated it.
`mistake-prevention.md`,
"Briefs and hand-backs".

## M200: a limit was accepted against the text of the decision it departs from

Status:
happened on 2026-10-14 in the ruling on n133's flag e (B520);
found by the independent oracle of n142 (decision D3).
The lead accepted that `cats’猫` reads as a trailing mark,
and K3 (B507) says that a mark beside a CJK character reads as loose.
The entry B520 itself records that the page outcome was not separately probed,
which is the case of M180 again:
a limit accepted without a probe of what the page reader sees.
The independent oracle read K3 literally,
and 2,854 of its 3,555 single-edit failures disappeared only under the other reading.
The lead reversed the acceptance as decision D3,
and n143 built the reading in B537.

Prevention:
an accepted limit is read against the text of the decision it departs from before it is recorded,
and the entry names the probe (the rule of M180).
`mistake-prevention-guards-searches-and-claims.md`,
"Claims without their evidence".

## M201: breaches of the output, pipe, edit-batch and shared-build rules by the residue round (n144)

Status:
happened on 2026-10-14 in the work of B549 to B555;
every one disclosed by its author in the hand-back (`s208/handback-n144.md`).

- Terminal output over 50 lines several times:
  a dump of the task descriptions of `mise.toml`,
  an `rg` for "pending",
  a Read of the one-line output `p20-ties-plain.out`,
  and a `git status --short` of the whole uncommitted tree.
- Two test runs in one parallel batch (`readings` and `loose-marks`),
  which read `dist` at once.
- One command with a pipe (`| head -n 0`),
  a placeholder that did nothing,
  during the rerun of the first reproduction.
- A Read of lines 1 to 200 of `s191/probes-n132/grid-n132.mjs`,
  the grid driver of an older author;
  the oracle beside it was not opened.
- The checksum check after guard A ran twice into the same log,
  so the second run overwrote the first,
  and the list of checksums of the original files predates one comment edit in `consistency-closing-marks.ts`,
  so the restore of that file is shown by a later list.
- Edits to one file in one parallel batch:
  the two test-comment edits of `pass-consistency-apply-largest.unit.test.ts` in one,
  and in another batch an Edit and a Read of the same test file when seed 7322 was added.

Prevention:
the same as M189,
a checksum check writes a log of its own for each run,
and a figure taken on another build is labelled as such,
as this hand-back did for the search for `unproves`.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M202: breaches of the Read, output and listing rules by the docs round of the ledger (n145)

Status:
happened on 2026-10-14 in the writing of B522 to B555 and M189 to M200;
every one disclosed by its author in the hand-back (`s209/handback-n145.md`).

- Reads over 50 lines of sources:
  lines 1 to 64,
  296 to 430 and 428 to 579 of the lead's notes (64,
  135 and 152 lines),
  lines 201 to 260 of a review and 151 to 210 of the hand-back of n143 (60 each),
  lines 760 to 843 of `audit-ledger-process-agents-and-briefs.md` (84),
  and lines 790 to 844 of `mistake-prevention-text-comparison-and-structure.md` (55).
- Terminal output over 50 lines:
  `rg --files` of the doc directory (about 110 names),
  the first search for function names in `w158` (about 90 lines),
  `rg --files-with-matches` for a position word over `src` (about 75),
  a vocabulary fixture read with `rg --max-count 60` (60 lines),
  and the second search for declarations (about 56).
- Listings and searches that recursed under the shared working tree:
  `rg --files` over the doc directory of `w152`,
  `rg --files-with-matches` for the old file name over the whole worktree `w152`,
  `rg --files-with-matches` for a position word over `src` of `w152`,
  and four searches for declarations and usages over `src` of `w158`.
- The formatter ran on eight files that hold older text,
  and the author did not diff what it changed beyond the lines the lint named,
  which the hand-back says is unverified.
  The diffs of those files were read in the next round,
  and they show no reflow of an older line:
  three of the eight are tracked,
  and their diffs hold only added lines and lines whose content changed,
  and five are new files that have no older text.

Prevention:
the same as M189,
and the limit of 50 lines binds a Read of a source file,
a hand-back and a log as it binds the terminal.
A formatter run is followed by a diff of the files that held older text,
and the lines the diff shows as moved without a change of content are restored.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands".

## M203: breaches of the output, batch and prefix rules by the tenth review (n146)

Status:
happened on 2026-10-14 in the review that wrote B556 to B563;
every one disclosed by its author in the review (`s210/review.md`,
part 10).

- Terminal output over 50 lines twice early:
  an `ls` of `dist/final/node/`
  and an `rg` for `writeFileSync` and `OUT =` over the copied probes.
- Several parallel tool batches,
  among them `count-single` run beside the diff that read its output,
  and batched Reads and Edits of one file.
- The `time` prefix once,
  and one `git diff` run from a subdirectory of `w158`,
  which the guard refused.
- The advisor timed out once and was not retried,
  so nothing in the review was second-reviewed.

Prevention:
the same as M189.
A review says which of its claims were second-reviewed,
as this one did.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands".

## M204: breaches of the batch, chain and output rules by the round that fixed the tenth review (n147)

Status:
happened on 2026-10-14 in the work of B556 to B563;
every one disclosed by its author in the hand-back (`s212/handback-n147.md`).

- Edits to one file in one parallel batch:
  four to `pass-consistency-apply-hand-built.unit.test.ts` in one,
  and two each to `quote-neighbours.ts` and `consistency-unread-units.ts`.
- An Edit and a read of the same file in one batch three times:
  `restore-typography.ts`,
  the fixture of the shipped page and `pass-consistency-shipped-marks.unit.test.ts`.
- `;` chains in at least seven Bash calls:
  the extracts of the red logs,
  the lint log,
  the scan log,
  the guard log and the status read.
- A chain of four steps with `&&` (`mkdir`,
  `cp`,
  `cd` and `node`);
  the hand-back counts a second chain of `mkdir`,
  `cd` and `node` too,
  which is three steps and within the limit.
- At least one `rg` output of more than 50 lines to the terminal
  (the failing hand-built lines of a red log).
- An attempted batch of eight Edits on the probe copies,
  all refused,
  a malformed Read call and a Bash call with a quoting error,
  none with an effect.
- The name of the test of the page `S15` was left saying "the one mark the archive held",
  true in count and not in glyph,
  to avoid another whole-suite run for a label (renamed by n149).
- The long fuzz runs were not repeated on the last build,
  on the argument that the sources equal those of the build they ran on by checksum;
  the hand-back says so.

Prevention:
the same as M189 and M197,
and a chain is counted by its steps before it is sent.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands".

## M205: breaches of the polling, edit, output and Read rules by the re-review (n148)

Status:
happened on 2026-10-14 in the review that wrote B564 to B574;
every one disclosed by its author in the review (`s214/review.md`).

- About fifty polls of the state of a unit with `systemctl --user show` and `is-active`
  while the generators ran.
  The `systemd-run --wait` client of one unit had been killed by the shell,
  so its notice never came.
  The reviewer then used `systemctl --user is-active --wait`,
  which returned at once,
  and finally a node script that waits for the output file with a timer and no shell loop.
- Edits sent in one batch:
  four Edits to four probe copies (a first attempt was refused because the files were not yet read),
  and two Edits to `review.md` in one batch.
- A `;` in one Bash call (`systemctl --user start --wait n148-nope.service 2>/dev/null; echo skip`),
  a placeholder that did nothing.
- `rg --files` on `s210/probes` printed about 130 lines to the terminal,
  and about 10 for `s212/probes`.
- Reads of output files with a limit of 60 lines,
  and sources and scripts read in ranges far over 50 lines,
  because the reviewer read the rule as covering outputs only:
  `consistency-unread-units.ts` (260 lines),
  `consistency-prose-text.ts` (250 and 180),
  `consistency-closing-marks.ts` (150),
  `consistency-proven-apostrophes.ts` (230 and 200),
  `pass-consistency-apply-screen.ts` (200 and 120),
  `restore-typography.ts` (290),
  `consistency-loose-marks.ts` (170 and 160),
  `quote-neighbours.ts` (110),
  `nested-single-quotes.ts` (160),
  `consistency-splice-readings.ts` (190),
  `consistency-edge-marks.ts` (110),
  and the whole of `r10-adversarial.ts`,
  `h-pass-lib.ts`,
  `r02-pairs-alone.ts` and the hand-back of n147 (347 lines).
- Runs over a minute outside `systemd-run`:
  the replay ran as a background command,
  as the brief gave it,
  while a unit was running,
  and `q-memo-examples.mjs` ran in the foreground with a limit of 300 seconds.
- One malformed `rg` call,
  and a `git show HEAD:…` read of two files into the scratch folder,
  one of whose paths does not exist in HEAD.

Prevention:
the same as M189,
and the brief says that the limit of 50 lines binds the files an agent reads (M209).
A unit is waited on in a command that ends when the unit does,
and is not polled.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "How an agent waits".

## M206: breaches of the output, batch, build and test-order rules by the round that fixed the re-review (n149)

Status:
happened on 2026-10-14 in the work of B564 to B571;
every one disclosed by its author in the hand-back (`s215/handback-n149.md`).

- Terminal output over 50 lines several times:
  a listing of the probes of `s214` and `s212`,
  an `rg --context` for a "readonly page",
  a search of the tasks in `mise.toml`,
  an `rg --follow` over `node_modules`,
  a count over the 90 pages of the corpus,
  the pass lines of a probe,
  and Reads of logs with very long lines (`guard-C.log` and `occ-debug.txt`).
- Two Edits to one file in one batch twice
  (`q8-cost.mjs`,
  and the import and the header of `consistency-run-steps.ts`),
  and a Write beside a Bash command that read the file the Write made.
- Commands that read `dist` at once:
  the red runs for the quote references and the unread units,
  `count-reference-pages` with `count-elisions`,
  and `lint:types` with `source-scans`,
  which builds.
- Tests written after the code for `consistency-run-steps.unit.test.ts`
  (after the scan for tests of a module's own demanded it),
  for the fixture option `silentWriters`,
  and for the line that guards the pairing in `countOpeningSingles`
  (added after the guard check showed it missing).
  Each was then run against a removed guard.
- The whole-suite unit captured standard output only,
  so the count of `[FAIL]` lines was not trustworthy,
  and the hand-back took the exit status 0 as the evidence.
  The lead checked the logs afterwards:
  the standard error of a unit goes to the same file by default,
  so the caveat is moot.

Prevention:
the same as M189 and M197.
A unit's output is captured on both streams,
and a hand-back that cannot show a count of failures says so,
as this one did.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".

## M207: an option that changes shipped glyphs was accepted without a probe of a paired quotation

Status:
happened on 2026-10-14 in the ruling on the hand-back of n147 (B563);
found by the re-review (finding Q5,
B568).
The hand-back flagged its choice:
on a curly page a leading straight mark before a listed elision curls to `’` whether or not it is paired,
which is wider than the ruling's "only the pairs that curl to `‘` now".
The lead accepted it,
and nothing probed a quotation that begins with a word of the list.
A quotation such as `'Tis the season,' she said` shipped with `’` at the start and a straight mark at the end,
and an ordinary quotation that begins with `Round` was read as an elision.
The tests of n147 pinned no such quotation,
and the count on the corpus found none of the shape
(a count sizes a cost and never proves a shape absent,
the rule of M181).
The lead reversed the acceptance:
the elision curls to `’` only when unpaired.

Prevention:
before an option that changes shipped glyphs is accepted,
paired and unpaired quotations are probed with it,
and the entry that records the acceptance names the probe.
`mistake-prevention-text-comparison-and-structure.md`,
"Quotes a judge gives back".

## M208: the refusal run was stated as a list of named characters

Status:
happened on 2026-10-14 in the ruling and the brief for the tenth review's fixes (B557);
found by the re-review (findings Q1 to Q4,
B564 to B567)
and by n149 (B574).
The lead stated the run by name:
three character references,
three spaces,
and "on the line".
The author built that list,
and the families around each name shipped a doubled mark:
the other references that are marks,
the escapes,
the other Unicode spaces and the soft line breaks inside a paragraph.
A fifth family,
the container markers at the start of a line,
was found by the author of the fix of the four.
The reason the ruling gave,
that a second mark for the eye must be refused,
named all of them,
and the list did not.
Q1 to Q4 were then fixed by saying what the renderer shows between the two marks.

Prevention:
a rule about what lies between two marks is stated by what renders between them
(the model of the renderer:
escapes,
references,
every white space,
line breaks inside a paragraph and container markers),
and never by a list of characters.
`mistake-prevention-text-comparison-and-structure.md`,
"Quotes a judge gives back".

## M209: a brief whose rule of 50 lines did not say that it covers source files

Status:
happened on 2026-10-14 in the briefs of the rounds before n149;
found by the lead on reading the review of n148.
The briefs said that output that may pass 50 lines goes to a file and is read in ranges of at most 50 lines,
and did not say that the rule covers the files an agent reads.
The reviewer of n148 read the rule as covering outputs only and read sources in ranges up to 347 lines (M205).
Other rounds read sources in long ranges too:
n145 up to 152 lines (M202) and n144 200 lines of a grid driver (M201);
what their briefs said is not recorded here.
The brief for n149 says that the rule covers source files.

Prevention:
the rule of 50 lines is stated for source files,
hand-backs and logs as well as for outputs,
and the lead checks each brief against the recorded decisions and the rules of this page before it is launched.
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Shell commands" and "Briefs and hand-backs".
