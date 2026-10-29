# Audit ledger: process mistakes, lint, edits and tests

Slips in editing and checking code:
lint left unread,
edits aimed by a stale line number,
tests that touched the real world,
and seams added the wrong way.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M6: edits attempted on files not read

Status:
recurring,
harmless (the tool refuses).
Twice more on 2026-09-27 (`tally-resolution.unit.test.ts`,
`roster-fixture.ts`),
both files viewed with `sed` rather than the Read tool.
Three times more on 2026-10-01 (UTC) during B75
(`required-providers.unit.test.ts`,
`probe-candidates.unit.test.ts`,
`mistake-prevention.md`),
each viewed with `rg` or `sed`.
Once more on 2026-10-01 (UTC) during B77 (`sheet-barrel.ts`).
Prevention:
read the region with the Read tool before editing it;
a `sed` or `rg` view does not count as a read.

## M26: a lint autofix changed what the code does

Status:
caught before it reached a run;
corrected in `70a2a73ca`,
2026-09-28.
`oxlint --fix` applied `unicorn(prefer-set-has)` to a string:
`prefix.includes('- ')` became
`new Set(prefix).has('- ')`,
a set of single characters that no two-character list marker can match,
so a list item's first word never opened a sentence.
The red guard caught it;
the commit before the fix had been made after the autofix,
unreviewed.
The date modules' autofix (`e9065faef` to `2efc90630`) was read line by line afterwards and is layout only.
Prevention:
commit before `--fix`,
read `git diff` after it for anything but layout,
and run the guards on the fixed code before trusting it.

## M38: stray marker lines in guard scripts, stripped by a pattern that ate indentation

Status:
happened 2026-09-28,
three times;
fixed in `6ebd37697` and before the other commits.
The scripts that wrote the X12 guards and the X14 seam test carried stray `"""` lines inside TSDoc blocks.
Two scripts removed them with `replaceAll('   """\n', '')`,
which on a nine-space line also took six spaces
of indentation from the line after,
leaving TSDoc closers at fifteen spaces:
one landed in `5acdd36c6`,
five more were caught before their commit by a search for over-indented closers.
No check reads indentation (X9),
and lint passed each time.
Prevention:
remove a stray line whole (`sed '/^\s*"""$/d'`),
never by a substring;
after any scripted edit,
search the touched files for over-indented `*/` lines and for the marker itself.

## M43: a unit test run under the run's keys reached the live web

Status:
happened 2026-09-28 with H16's red guard;
fixed structurally in `d82dfe559` (X19).
The guard drove `preparePassEntry` end to end with an original naming a title,
and nothing in the test said which of the preparation's reads leave the process.
The suite inherits every decrypted key from the root `mise.toml`,
so the preparation's own lookup bought a search and wrote a record to the real cache (X19).
The same hazard had been met once before and handled in one test only,
by asserting its fixture named no title (`pass-footnote-lifecycle.unit.test.ts`).
Prevention:
a new test that drives a production entry point lists the reads that entry point makes
outside the process (`process.env`,
`fetch`,
`homedir()`,
a pinned corpus) before it first runs,
and hands each a fixture;
a seam that reaches the network or a real cache is required,
never optional,
so the type checker refuses a test that forgets it.

## M47: a type declared while the same shape was already exported under the same name

Status:
happened 2026-09-28 (B11),
fixed in the next commit.
`index-pair-list.ts` declared `IndexPair { source, target }` for the shared correspondence-list guard,
in a change whose whole purpose was removing copies;
`pair-agreement.ts` already exported that type by that name.
Prevention (the prevention doc's copies family):
before declaring a type or helper,
search for its name and for its shape (`rg 'type <Name>\b'`,
and the field names together),
and reuse what is there.

## M61: a comparison key that assumed its parts never throw and move nothing twice

Status:
happened 2026-09-29 while writing B26's fix,
caught before its commit.
The first `wordingKey` read every text through `foldSoftBreaks`,
which parses the document and throws `FrontMatterParseError` on a fenced block whose YAML does not parse,
as its own TSDoc says;
the repair turn's copy check reads a model's reply before any floor,
so a comparison that never threw could have stopped a slice.
The key's own front-matter case threw on its first run.
The same draft dropped the lane wraps' byte check after the wrap
on the strength of the wrap moving nothing on a second application,
which had been measured over twelve passages and never over the stored runs.
Prevention:
before a comparison reads text through a function,
that function's thrown errors are read and each gets a defined answer
(`pageLayout` lays such a text out as written);
a property a change relies on is measured over the stored records first
(the wrap moved 1,876 of 7,462 distinct stored texts and none on a second pass),
or the change is written so it does not rely on it
(the lane wraps compare the wrapped text).

## M65: a stand-in command whose text carried the marker the census counts

Status:
happened 2026-09-29 in `37e1dd8e8`,
caught by the first run of its test file,
fixed in `5fb0af40e`.
The failing-command case of `runSuite` ran a stand-in whose command text held a literal `[FAIL]`,
and `runSuite` warns with the command it ran,
so the suite's own log carried a failing marker.
A census of the whole suite counts those markers,
and would have refused every run as a failing suite.
Prevention:
a fixture that prints a marker tooling counts builds the marker when it runs,
and a new test file's first run is read for markers in its log,
not only for its exit.

## M68: a seam required of two functions and supplied by their caller

Status:
happened 2026-09-28 in `d82dfe559`,
which made the outside reads a required parameter of `preparePassEntry` and `runPassPreparation`
while `runEntryPipeline` went on handing `runPassPreparation` the run's readers itself;
found 2026-09-29 by listing every reference to `RUN_OUTSIDE_READS` while the T8 census reached `corpus-run`,
fixed the same day.
`settleEntry` took no readers,
so `pass-entry.unit.test.ts` prepared each of its entries through `RUN_OUTSIDE_READS`:
the Exa key the suite inherits,
the real lookup and reference caches under `~/.cache/translation-repair`,
and the corpus clone at the pin.
Its fixtures link `https://example.test/cat-record`,
and the reference cache holds a record for that address fetched 2026-09-16T16:26:05Z,
status `error`,
failure `CRAWL_UNKNOWN_ERROR`,
a field that holds the endpoint's own error tag
(the record read 2026-09-29;
it predates the seam,
when preparation read the key directly).
Of the 14 reference records it is the only one at a fixture address,
and of the 58 lookup records the newest whose title a test file holds is X19's own,
written at 19:42 UTC,
before that fix landed at 19:56 UTC
(`m68-fixture-records.mjs` in the audit's scratch folder;
its first version compared whole queries,
missed X19's record,
and was rewritten to compare the title inside the query before its null was read).
Every later run of the file read that cached failure instead of asking again,
so its sheets carried an unread-reference line no fixture chose,
and on a machine without the record the file would have bought the read again.
The type check X19 relied on could not see the gap:
a parameter is required only of the functions that declare it,
and a caller supplying the run's value itself satisfies it.
`settleEntry` and `runEntryPipeline` now require `outsideReads`,
the pass hands over `RUN_OUTSIDE_READS`,
every entry test hands over `NO_OUTSIDE_READS`,
and a case in `pass-entry.unit.test.ts` hands over recording readers
and checks each was asked about the entry's original and nothing else
(`3f6df0396`).
A mutant `settleEntry` handing the pipeline inert readers of its own in place of the ones it was handed
failed that case alone,
at its first reader count;
the restored build passes it.
Prevention:
when a seam becomes required,
every reference to the production value is listed (`rg RUN_OUTSIDE_READS src`),
and each one below the function a run itself calls moves to a parameter;
the seam is done when the production value is named only by that function,
TSDoc examples,
and runner commands that build a live client beside it.

## M69: a real-clock case that needed the machine to keep pace with a 20 ms window

Status:
happened 2026-09-29 in `ee6b012c2`,
found the same day as the only failing case of the whole unit suite run after `3510c8336`,
fixed the same day.
The case filled a pacer's 20 ms window on the real clock,
slept through it,
then started a third take and aborted it at once,
expecting the abort to end a sleep.
Under the whole suite's load the window had emptied before the third take began,
so the take found room,
never slept,
and resolved;
the case had passed every time its own file ran alone.
The case is now two:
a short window whose second take is checked by a one-sided bound
(it cannot finish before the first take leaves the window,
and a stall only lengthens it),
and a 60 second window whose second take is certainly asleep when the abort lands.
Prevention (T5):
a real-clock case asserts only bounds that load can widen and not break,
a case that needs a caller asleep uses a window no stall outlasts,
and a new timing case counts as passing only once the whole suite has run it.

## M70: a corpus seam added optional a day after the rule said required

Status:
happened 2026-09-29 in `be6490163`,
found the same day by X24's census,
fixed the same day.
To give `loadEntry`'s happy path a test,
that commit took the corpus reader as a parameter and defaulted it to `readCorpusFile`,
the clone,
a day after M43's prevention had said a seam reaching the corpus is required and never optional.
Every caller that left it out read the clone;
the runner meant to,
and the unit file's refusal cases did not reach the read.
The parameter is now required (X24).
Prevention:
a seam added to make a function testable is checked against `mistake-prevention.md`'s real-world rule before it is written,
and never takes the production value as its default.

## M71: a line-range `sed --in-place` over two files, applied to each

Status:
happened 2026-09-29 in B30's wrapper group,
caught before any commit.
One `sed --in-place` call with a line range named two files
(`probe-telemetry.ts` and `probe-barrel.ts`),
meaning the range for the first;
GNU `sed` with `--in-place` treats each file separately
and applied the range to both,
deleting 19 barrel lines and a TSDoc opener in the telemetry file,
which the build refused as a parse error.
The barrel was restored from `HEAD` and the opener rewritten.
Prevention:
a line-range edit names one file per call,
and a scripted edit prints its diff before any build reads it.

## M75: removals that left the imports they no longer read

Status:
happened three times on 2026-09-29
(`3510c8336`,
`66cd08426`,
`9f49c2ed5`),
found the same day by B32's guard and fixed in `c077cc881`.
Each commit removed the last read of a name its file imported
and left the import:
ten bindings across nine files.
The prevention doc already said a removal counts the removed names' uses,
because no configured check reports an unused import;
none of the three commits applied it.
Prevention:
`src/unused-imports.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/unused-imports.unit.test.ts`) now fails on an unread import,
and it runs in `mise run source-scans`,
which every source commit runs.

Recurred 2026-10-01 (UTC) before B103's commit,
which records it.
An agent's patch deleted 42 test helpers and left 21 imports only they had read.
Lint passed,
and so did a named run whose list held the 41 changed test files without the source scans,
against the prevention doc's rule that a source commit's named tests include them;
the full suite's run of the scan found the imports.
Prevention added:
an applied patch that deletes code runs `mise run source-scans` before its named tests,
and an agent asked to delete code is asked to count the deleted names' remaining uses.

## M80: a narrowed caller left its callee's arm with nothing to reach it

Status:
happened 2026-09-30 (UTC) in `a589d1fde`,
caught by the batch census at `b546d1b2d`
and removed in `0fcb75c7b`.
That commit made the translate stage ask for the floor's findings only where the archive did not pass,
and left `floorFindings`' arm for a pass in place,
so the census that was to show the batch's stretches closed showed a new one of the batch's own making.
Prevention:
a change that narrows the condition a call sits under reads the callee's arms against the new condition in the same commit,
and narrows the callee's parameter type where the language can say it,
as `floorFindings` took only a verdict that was not a pass,
until B43 made the stage read the verdict through `requireComparedVerdict` and removed it.

## M83: a lint's findings left unread because the command printed its log only on failure

Status:
happened 2026-09-30 (UTC) during the ninth batch's `51b888ac2`,
caught before the commit,
and first misdiagnosed.
The Markdown lint of `doc/slice-context.md` (moved on 2026-10-18 to `package/translation-repair/doc/slice-context.md`)
ran as `astral-safe-lint.mjs <path> > <log> || tail <log>`,
the shape every build and lint command in this audit takes.
`astral-safe-lint.mjs` exits 0 whatever it finds,
so the `tail` never ran,
and its log,
which read `semantic-line-breaks 3`,
went unread.
The run was then taken for one against the main checkout,
since the shell starts every command there and this one had no `cd`,
and it was run again from the worktree;
that run's three findings were fixed before the commit.
The diagnosis was wrong:
the wrapper joins every path onto the worktree root (its `repo` constant),
the main checkout holds no such file,
and the first run had linted the edited file.
The ninth batch's record first carried the wrong diagnosis as this entry;
it was corrected before commit by reading the first log and the wrapper.
Prevention:
a tool that exits 0 with findings is read by its output,
not its exit status
(`astral-safe-lint.mjs`,
the census,
`run-guards-named.ts`):
its log is printed or searched every time,
and a diagnosis of why a run misread is checked against the tool's source before it is written down.

## M87: a test's idiom moved into a fixture file that lints as source

Status:
happened 2026-09-30 (UTC) while writing T8's sixteenth batch,
found by the package lint before `c2974bea9`,
and corrected in that commit.
The short-bench client came from a copy in a unit test,
written with arrow functions and `async` bodies without `await`.
Moved into `refusing-seats-client.test-fixture.ts`,
it drew three `no-arrow-function` errors and six `require-await` warnings:
the shared lint config relaxes those rules only for `**/*.{test,bench}.ts`
(`testOverride` in `package/config/oxlint/src/overrides.ts`),
so a `*.test-fixture.ts` file is held to the source rules.
Prevention:
code moved from a test into a fixture follows an existing fixture's idiom
(`archive-selection.test-fixture.ts` for a scripted client),
and the lint runs before the move is used.

## M88: shared readers extracted with no unit test of their own

Status:
happened 2026-10-01 (UTC) in `f0bad6d25`,
found the same day reading the prevention doc against B72's census,
and corrected in `3429c8229`.
`f0bad6d25` moved `treeNodes` into `mdast-tree-nodes.ts`
and wrote `emphasisSpans` and `oneLine` into `corpus-run/emphasis-spans.ts`,
each with two callers,
and tested them only through the italic-title pass and the formula floor.
The prevention doc's rule that a shared reader gets a unit test of its own when it is extracted
stood under "Text by code point",
where a reader of structure is not looked for.
Writing those tests found `oneLine`'s TSDoc saying every whitespace run reads as one space,
where the code folds the space and the line feed alone (ledger B72).
A probe at `3429c8229`
(`shared-untested-probe.mjs`,
its control `code-points.ts` read as tested)
counts 167 of the 562 production modules imported by two or more others,
barrels aside,
with no unit test file of their own.
Prevention:
the rule now stands under "Copies of shared code",
where moving a helper is looked for;
a source scan failing on a new shared module without its own test,
over a classified list of the 167,
is open.

## M93: line numbers for in-place edits worked out rather than read

Status:
happened 2026-10-01 (UTC) during B75,
three times,
each seen in the diff before any commit and restored.
Inserting `argv: process.argv` and doc lines across the runner entries with `sed --in-place`,
line numbers were worked out from offsets and from listings taken before an insertion higher in the same file:
`sentinel-probe.ts` had its doc comment garbled
(rewritten through the scratch `replace-lines.mjs`),
`coverage-census.ts` had an `@param` placed in the wrong comment and a line written over
(restored),
and a delete in `score-probe.ts` took one line more than meant,
the opening of a doc comment
(restored).
Prevention:
a line number for an in-place edit is read from `rg --line-number '' <file> | sed --quiet 'A,Bp'`
taken just before the edit,
one file per call;
after an insertion,
the numbers below it are read again rather than shifted by hand.

Recurred 2026-10-01 (UTC) in T8's twenty-first batch,
on a scratch probe rather than a package file:
a `sed --in-place` aimed at the probe's `require` line took line 9 from memory,
where line 9 opened a doc comment,
and garbled it;
the tool's own diff showed it and the next call restored both lines.
The prevention stands,
scratch files included.

## M95: scratch readers of lint output that could miss a finding

Status:
happened 2026-10-01 (UTC) during B75,
twice,
with no finding missed.
The scratch `lint-findings.mjs` paired findings with locations through a pattern written with a Unicode dash
where oxlint prints an ASCII one,
so it could match no location line;
it was corrected to `,-\[`.
And the summary check `rg 'Found [0-9]+ warnings'` cannot match oxlint's singular
"Found 1 warning and 0 errors.";
on the one run that printed it,
the findings reader showed the warning and the summary check printed nothing,
which was read back from the log before it was fixed.
Prevention:
the summary is read with `warnings?`,
and the findings list and the summary are read together,
each a check on the other.

## M100: a test fixture that replaced a process global while the runner ran cases side by side

Status:
happened 2026-10-01 (UTC) during B78,
found by B78's first guard-off,
and fixed in `b1604a2f8`.
`wall-clock-stub.test-fixture.ts`,
as B78's red commit wrote it,
replaced `Date.now` for the whole process while a case held it.
It was written without reading how module-test schedules cases:
a suite runs its children side by side,
sixteen at a time by default (`package/module/test/src/describe.ts`).
The fix commit's checks passed,
since on the fixed tree no package code reads `Date.now`;
the logger does,
for its stamps (`package/module/logger/src/create-logger.ts`),
so a sibling's log lines during a stub window carried the stepped time,
as the FAIL lines of the fixture's own guard-offs show (`2026-09-30T23:00:00.000Z`).
The first guard-off of B78 (`monotonicMs` reading `Date.now`) showed the cost:
`request-pace.unit.test.ts` printed its B78 case's FAIL line and then did not exit;
the guard runner waited on it for eleven minutes until it was killed by hand,
and run alone it stalled again after three PASS lines.
The mechanism is an inference,
not confirmed:
a real-clock pacer case beside the stepped one read the stubbed clock,
about eight hours behind the real one,
and its pace loop slept toward that.
A regression back to the system clock would have hung the whole suite rather than failed it.

The stub now goes through the case's own sandbox,
`ctx.sinon.stub(Date, 'now')`,
which module-test answers only to code running in the owning case's async context
and restores when the case ends (`sandbox-slot.ts`,
`execution-node.ts`).
`wall-clock-stub.unit.test.ts` holds a stepping case and a sibling that waits for it,
and the sibling,
bracketing its read with `new Date()` stamps,
reads the real clock.
Guard-offs on the fixed fixture,
restored with `git diff` printing nothing after:
an added process-wide assignment beside the sandbox stub was refused by the sandbox once another case's stub was live
(`SandboxOwnershipError`),
so the stepping case failed and its sibling timed out,
the wrong route;
with the sandbox stub replaced by the plain assignment,
the sibling failed its own assertion,
its read sorting as `2026-09-30T23:00:00.000Z` ahead of the real stamps.
With the stub scoped,
the first guard-off of B78 rerun exited every file with its FAIL lines.
The scratch guard runner (`~/temp/agent/audit-glossary-fix/run-guards-named.ts`) now ends a file after 300 s and prints that it did.

Prevention:
before a fixture replaces a global,
read how the runner schedules cases,
and scope the replacement to its case (module-test's `ctx.sinon` for a method)
or sequence the cases that share it;
a guard-off whose process does not exit is a finding of its own,
and a guard runner bounds each file.
The package's other tests that replace process globals
(`process.env` variables,
`console` methods,
`process.exitCode`),
in 29 files by one search,
were classified and fixed as B79.
17 of those files already set `concurrency: 1` on at least one suite,
several with a comment giving this reason
(`cli-refusal`,
`slice-cache-discard-log`,
`rendering-audit-settled-print`),
so the package knew the hazard;
no doc stated it,
and the stub was written without reading those files either.
The prevention doc's "Globals a case replaces" now states it.

## M103: TSDoc blank lines written in the form their file does not use

Status:
happened 2026-10-01 (UTC) in T8's twenty-second batch,
found before the third commit of the batch's fixes and corrected in it.
`anthropic-completion.ts` writes every blank line inside a TSDoc block as one space;
the blocks written by `33bffdb7b` and `08f56f71e` used empty ones,
34 of them in the file at `08f56f71e` beside 20 spaced,
and the lint and the source scans pass either form.
The same slip was corrected once before,
in `d811fcf2b`,
where the edit tool had dropped the spaces.
It is the cause again:
the B90 fix's TSDoc for `completion-shape.ts` was written with one-space blank lines
and landed with 6 empty ones,
so a line holding only a space does not survive the edit tool.
Prevention:
before committing a file whose TSDoc changed,
count its blank lines inside TSDoc blocks by form and keep the one the file used.
A source scan holding each file to one form would make this a check rather than a habit;
none is written yet,
and 264 of the package's 2,017 TypeScript files under `src` mixed the two forms when this was recorded.

The check held for the rest of the batch:
before the B93 and B94 fix commits it found 4,
4 and 7 empty lines the edit tool had left
and restored them.

## M111: a comment naming its check by where it sits

Status:
happened 2026-10-03 (UTC),
caught at once and recorded here.
The T5 recurrence's fixture comment told the reader "the check below covers them",
and `position-references.unit.test.ts` turned the suite red on it before the change landed.
The comment names "the no-leak check" in words now.
Prevention:
a comment names the check it discusses in words,
never by where it sits,
since an insertion silently repoints a position.

## M117: a line-range `sed --in-place` aimed by a count off a print that had lost a line

Status:
happened 2026-10-05 (UTC),
while merging the footnote mention fix on the main checkout,
seen in the next read and repaired before any commit.
To replace two lines of a TSDoc in `footnote-mentions.ts`,
the lines were printed with `sed --quiet` and without their numbers,
against M93's prevention,
and counted from the range's first address.
The print opened on a blank line,
which the tool's output trimmed,
so the count started on the wrong line,
and `sed --in-place '202,203c\…'` replaced lines other than the two meant.
The block was repaired with the Read tool and one Edit.
M93 records line numbers worked out and not read;
here they were read,
off output that no longer matched the file line for line.
Prevention:
a replacement of more than one line is a Read and an Edit,
which match by text and fail where the text is not there.
Where `sed --in-place` is used,
each printed line carries its own number from the file,
by M93's form or by `cat --number <file> | sed --quiet 'A,Bp'`,
never a count from the range's start.
`mistake-prevention.md`,
"Lint and edits".

## M135: escapes written through a tool's parameters landed as the characters they name

Status:
found 2026-10-06 (UTC);
each instance in the commits named here was caught before its commit or mended in the merge that brought it.
A tool that writes a file from a model's parameters decodes a backslash-u escape written there
into the character it names.
The lead's own edit meant to write `\u002D` wrote a hyphen,
which showed when two edits were refused as changing nothing;
a commit message draft written the same way held a literal U+FEFF,
found by a search for invisible characters before the commit.
The same decoding wrote the agents' literal characters:
nine byte order marks in the parse agent's front matter cases,
written as escapes in `75ea14bff`;
two full-width commas in its TSDoc examples of `corpus-run/dropped-destinations.ts`,
written as escapes again in `3ce947dce`;
and a hyphen for the JSON escape of one in three comment lines of a patch not merged at `3ce947dce`.
By the lead's inference,
not measured,
the same decoding wrote a soft hyphen and byte order marks that stand literally in older test files,
which are not yet escaped at `3ce947dce`.
Prevention:
an escape is written through a shell edit,
with the backslash doubled in the edit's replacement
(`sed` with `\\u` where the file is to hold `\u`),
and every file the lead or an agent wrote is searched,
before it is committed,
for invisible characters and for an escape that came out as its character;
the brief agents read says so.
`mistake-prevention.md`,
"Escapes and the layer that reads them".

## M178: an author ran the package's auto-formatter over the shared tree

Status:
happened on 2026-10-13 or 2026-10-14 in the work on B488 to B491;
disclosed by the author in its hand-back.
The author ran `format:oxlint` twice over the package in the shared tree,
and the formatter reformatted files in the package.
The hand-back does not list them,
and the tree held other authors' uncommitted work.

Prevention:
an author runs a fixer only on the files it changed,
or reports every file the fixer touched.
`mistake-prevention.md`,
"Lint and edits".

## M289: the tests of the run-ending change were green because the pass guard sat in every call path

Status:
happened on 2026-10-22 (UTC) in the first round of B818,
found by the review of that round (its note N7 and its should-fix S2).

The first tests built their clients with the pass guard (`PassDryStop`) already in the call path.
The guard aborts the shared signal before the error reaches any catch,
and every layer that checks `signal.aborted` rethrows,
so a catch that swallowed the refusal still ended the run through the guard.
Four catches could swallow it with no guard in the path:
the picture readers' settle,
the stage round's grace window,
the overlapped map's rethrow of the earliest failure,
and the seat reader's own refusal.
The tests and the claim "checked in the stage catches" held for the stage calls and for nothing else.
Two run-level cases were weaker still:
the new case of `corpus-pass-run.unit.test.ts` overrides `settle`,
so it never runs `pass-entry.ts` and shows nothing of the in-flight entry's classification,
and `entry-error-outcome.unit.test.ts` feeds the error class to the classifier directly,
so nothing showed that the class arrives there unwrapped.
Any layer that wraps the error instead of rethrowing it would have filed the entry as `resumable-failure`,
printed a REATTEMPT line before the stop line,
and left every test green.

The fix:
a case for each catch with a client built without the guard,
asserting that the refusal reaches the caller unchanged (reds `b1f1196379` and `51e18a1831`),
and a `settleEntry` case that asserts the entry hands the refusal that ended it to the run (B818).

Prevention:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"A refusal that ends the run":
a test of the claim "this error ends the run" builds its client without the guard,
asserts the error reaches the caller unchanged,
and adds a positive control,
the same case with an error that does not end the run,
which must still be contained.
A test that overrides the function under claim shows the stub.

## M290: a scripted merge of two top-level describe blocks broke three test files, and the formatter left whitespace the source scan lists

Status:
happened on 2026-10-22 (UTC) in the second round of B818 (`3dcd82ed6d`),
repaired by `462a2d8772`.

The merged test files take one top-level describe.
A scripted merge by text substitution (a Python rewrite) broke three of them,
and the merge left trailing whitespace on blank lines inside object literals.
`oxlint --fix` does not strip that whitespace;
the source scan for trailing whitespace does.
`462a2d8772` stripped the whitespace "as the whitespace scan lists".

Prevention:
`mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Scripted merges of test blocks":
a merge of two describe blocks is edited by hand or with a structural tool,
never by text substitution,
and the source-scan `test:unit` runs after any scripted file merge.

## M291: a block was added to files near the line limit, and three extractions followed the code

Status:
happened on 2026-10-22 (UTC) across `068d523090` and `3dcd82ed6d`.

The `max-lines` rule (300 lines,
comments and blank lines skipped) forced three extractions after the code was written:
the voice-loss and shortfall findings moved to `stage-voice-lost.ts`,
the reader contain step to `image-reading-contain.ts`,
and the seat refusal to `run-seats-refusal.ts`.

Prevention:
`mistake-prevention-moves-splits-and-rewrite-scripts.md`,
"Files near the line limit":
before a block goes into a file near the limit,
count its lines the way the rule counts them and plan the extraction first,
as its own commit.

## M292: a type guard written on a destructured catch binding tripped TS1230 and the rule against throwing non-errors

Status:
happened on 2026-10-22 (UTC) in `068d523090`,
and was repaired in the same commit.

A type guard on a destructured catch binding tripped the type checker's TS1230 and the lint rule `only-throw-error`.
The form that passes takes `error: unknown` and narrows it with a predicate:
`isDryRefusal` in `stage/src/stage-run-end.ts` returns `error is EveryProviderDryError`.

Prevention:
`mistake-prevention-shell-edits-commits-and-agents.md`,
"Lint and edits":
a guard for a caught value takes it as `unknown` and returns a predicate,
and the caller rethrows the narrowed value.
