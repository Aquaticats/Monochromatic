# Audit ledger: process mistakes, guards and red cases

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Guards and failing-first cases that could not fail,
never reached what they guarded,
or were removed to please a census.

Part of [Audit ledger: process mistakes](audit-ledger-process-mistakes.md),
which lists every file of this record.

## M3: a guard whose first red was a link failure

Status:
fixed by the prep export `refactor(module-translation-repair): export archiveStandInFor`.
`archive-dispute-standing.unit.test.ts` first failed because `archiveStandInFor` was not exported,
which says nothing about behaviour.
Prevention:
read each `[FAIL]` reason of a red run;
a missing export gets its own prep commit first.

## M5: a new condition guarded on one branch only

Status:
fixed (guard `88fdb1923`).
`bd98bdc70` refused the judged part of the incumbent whenever a target-only run was held out,
which refused an eligible stand-in with a trailing transcript;
the guard had no eligible stand-in with a held-out run,
and review found it.
Prevention:
a guard for a new condition covers every combination of the inputs it branches on,
with a positive control that must move.

## M11: a fix for one test defect written in the shape of another

Status:
fixed in `d76d2424a` and `a9230202d`.
The T6 fix made two tests assert that the second call finishes first,
and ordered the calls by 20 ms against 5 ms sleeps,
which is the T5 shape;
`consolidate-driver` then failed 1 of 8 at 0.2 CPU.
Twice more a comment stated how a check fails before any mutation showed it:
`e40ccf6d2` said a hanging voice ends at the exchange deadline (the fixture arms none,
so the bound could never fail),
and the gate comment said the case fails on the run deadline (the file exits 13 on the unsettled wait).
Both are corrected,
with commit comments on `e40ccf6d2` and `c10d62663`.
Prevention:
an ordering claim in a test is enforced by a gate,
never by a head start;
a comment that says how a check fails is written after the mutation that shows it.

## M14: a reproduction check committed without a positive control

Status:
corrected in `7c444de3e`,
with a commit comment on `2c4207912`.
The A12 fix compared the rebuilt identity with the recorded one,
checked only against a synthetic fixture,
and was committed before any stored artifact was run through it;
every stored artifact then read as moved,
mikaela17 among them though its rows matched slice for slice.
The same shape recurred one step later,
when the rendering audit's rows-plus-measurements check
was committed with the alignment findings in it and every reproduced carve still refused.
Prevention:
a check that classifies real artifacts runs over stored ones before its commit,
with at least one that must pass and one that must fail (QPC),
and every class it can print is read.
A related shape,
`<test A> | rg --count FAIL || <test B> | rg --count FAIL`,
never ran B when A went red (S10);
and `mise run --cd <package> build` once,
against CM5.
The first `;` one also hid which of two files failed,
since both counts printed as one number.
Prevention:
a report that should run after a failing command is `a || b`,
never `a ; b`;
two independent checks are two tool calls.
Once more on 2026-09-28,
in narration:
the X15 slice-cache check was announced with an empty-directory control
the command did not contain;
its second `find`,
printing the newest file's time,
served as one,
and the narration was corrected before the result was used.
Once more on 2026-09-30 (UTC) during B45:
`ls <removed worktree> ; git worktree list | rg --count <name>`,
confirming a removal with two checks joined by `;` where they belonged in two calls.
Once more that day during B45
(its arguments were not kept beyond the file the second search read):
two `rg` searches joined by `;`,
the second over a T8 triage file in the audit's scratch folder.

## M21: a check that could not fail

Status:
corrected in `d2f0e07ef` (E4).
A test asserting that a throwing settlement bought no judge read its counter off the return value,
which a throw never delivers,
so the assertion held at its initial zero whatever was bought.
A parser test named "listing the two it does" had pinned three kinds,
and later four.
Prevention:
a counter or state a throwing path must leave untouched is held by the test outside the call;
test names describe the property,
not a count that the next change makes false.

## M27: a mutation harness that could not report a catch

Status:
caught by its own result,
2026-09-28;
the verdicts were discarded and the check rerun.
The spelling mutation script dropped every output line containing "Error:
" to skip the suite's summary line,
and every assertion failure reads "AssertionError:
",
so all 26 mutants printed `SURVIVED`.
A null result from a probe never shown able to fail is no result (the M21 family,
one level up).
Prevention:
every mutation run opens with controls,
an unchanged source that must pass and a mutant that must fail,
before any verdict is read.

## M29: a red guard asked a function that never reads the entry it guards

Status:
caught before the fix landed,
2026-09-28;
the guard was rewritten in `357f534b7`.
The R11 and R12 guards of `8da383b89` asked `communityRenderingDepartures` whether "dated" and "intensive care"
still counted as renderings of 交往 and 抢救,
but the departures block reads the community glossary only,
and both words are in the rendering glossary,
whose renderings reach the identity-context lines and nothing else.
The guards were red before the fix and would have stayed red after it,
so their red proved nothing about the finding.
Prevention:
a red guard is read case by case before the fix
(each failing case must fail for the reason its label names),
and after the fix every case must turn green;
a case that stays red after the fix is a guard defect,
not a fix defect.

## M34: a guard that looked for a word the message carries twice

Status:
caught by the mutation check,
2026-09-28 (P12);
the guard was tightened before the entry closed.
The all-dry refusal's guard asked whether the message said "Bedrock",
and the message says it twice:
once in the out-of-budget list the finding was about,
once in the refill sentence.
A mutant dropping Bedrock from the list passed.
Prevention:
a guard asserts on the clause that makes the claim,
not on the whole text;
before committing a guard of the form `includes(word)`,
`rg` the message for every other place the word appears.

## M41: a mutation spec that left out the test pinning the mutated table

Status:
happened 2026-09-28 in X12's preparation half;
the mutant was caught once the file was listed.
The first preparation spec mutated the `preparation` row of `BENCHES_BY_PHASE`
but listed only the preparation's own tests;
`corpus-run/run-seats-wait.unit.test.ts`,
which pins the whole table since `c2adcfdbf`,
was not among them,
so the mutant survived and read as a gap in the guards.
Prevention:
for every mutant,
search the tests for the mutated token and list every file naming it;
report a survivor only after that search.

## M42: red guards whose fixtures never reached two of the sites they guarded

Status:
happened 2026-09-28 in X12's preparation half;
the guards rebuilt in `d00123df3`.
`f891b358e` guarded four call sites.
Its pairing fixture had sections of equal shape,
so no section round was bought,
and its hook handed one constant roster,
so a block round reusing an earlier seating passed;
its seam fixture paired every archive block,
so the review never ran.
Every guard went red before the fix and green after it,
which proved only that some site was fixed:
the M29 family,
a red that says nothing about the site its label names.
Prevention:
a guard over several sites asserts that each site is reached
(one assertion per site,
as `sectionRoundAsked` and `blockRoundAsked` now are),
and a hook under test hands a different roster at each call,
so reusing an earlier reading fails.
Once more the same day,
in the X14 pictures seam:
`f865f524a` held Synthetic throughout,
so the first seating was never asked,
and nothing showed the injected OCR was the one read;
`35684e182` adds a case with no hold and counts the stand-in's reads.
Prevention,
added:
a case under a constant hold exercises only the re-seat;
the first seating needs a case with no hold,
and an injected dependency needs its own call counted.

## M44: a detector that said one order and kept another, guarded by a fixture that could not tell

Status:
happened 2026-09-28 in H16's `page-title-spans.ts`;
fixed in `d9a306602`.
`repeatedTitleSpans` said it listed titles by first appearance,
and tallied every heading,
then every HTML heading,
then every 《》 and 【】 span,
wherever each stood.
Its order test used 《》 alone,
one marker kind,
so it passed either way:
the M42 family,
a guard whose fixture
cannot reach the difference its label names.
The same file copied a private code-point counter,
with a `character !== ''` guard that a `for…of` over a string
can never fail,
from `page-name-glossary.ts` and `corpus-name-index.ts`,
beside a shared `codePointCount` (`code-points.ts`) that says it exists so no copy drifts;
`67243edae` removes all three copies and the three private copies of the Han test.
The rewrite then slipped a regex into the heading reader (RG1),
caught on reading the diff before lint.
Prevention:
before writing a helper,
`rg` the package for one that does the job
(`rg 'function \w*(codePoint|Han|heading)' src`);
an order claim is tested with a fixture mixing every source the order draws from.

## M50: a mutation runner that read a crashed test run as a survivor

Status:
happened 2026-09-28 (B18);
the runner fixed the same hour.
`mutants.ts` (`~/temp/agent/audit-glossary-fix/`) counted `[FAIL]` lines only.
A mutant giving the casing scan an ASCII-only continue test made it loop forever on a word opening with `Â`;
the test process died of memory exhaustion before printing any verdict,
and the runner reported the mutant SURVIVED.
Reproduced by hand,
it was a hang,
not a survivor,
and it exposed a real hazard in the scan (fixed in `f4aa85d9a`).
Earlier survivors were each closed by an added test,
so no past verdict let a gap stand;
a crash read as a survivor only ever cost an unneeded test.
The runner now counts a non-zero exit,
a signal or a timeout (180 s) as a failure and names it,
and a control that reproduces the hang reads as caught (SIGABRT).
Prevention:
a harness's verdict counts only outcomes it can observe;
a run that ends without a verdict is a result of its own,
never read as a pass,
and each harness gets one control per outcome it reports.

## M52: a positive control applied by line number after an edit moved the line

Status:
happened 2026-09-28 (B18);
caught because the control read 0,
and rerun.
The suicide floor's control rewrote its stem with `sed` addressed by line 53,
read from the file before the switch's import grew by three lines;
the `sed` matched nothing,
the harness reported no difference,
and a null control is no control.
Rerun by pattern with the replacement counted first,
the control made 21 of 131 pairs differ.
Prevention:
a control edit is addressed by the text it changes,
never by a line number,
and the command counts the changed text (`rg --count-matches`) before the measurement runs.

## M53: a test committed for a defect it never reached

Status:
happened 2026-09-28 (B18);
`94dca38d8`'s test,
fixed by `d107043be` the same hour.
The test for gloss placement beside a combining accent put the accented handle in a signature,
where the pass never establishes a signer's rendering,
so it placed nothing under either build;
the commit named a defect its own test could not show,
and a mutant restoring the old edge test survived it.
The fixture checks run for the address,
refusal and line switches against the pre-letter build
(`.cache/dist-before-letters/`) were skipped for this one.
Prevention:
a test added for a defect is run against the build before the fix,
and must fail there,
before the fix is committed;
a test that passes on both builds is rewritten until it reaches the path.

## M56: a red guard whose comment case never reached the comment path

Status:
happened 2026-09-29 in X22's red guard (`deb3406b2`),
caught by the mutation check the same hour and corrected.
The case meant to show the floor still reads a paragraph carrying an HTML comment
set the comment between the two dollar signs,
where the formula swallows it as math text,
so the formula formed whether or not comments were cut,
and the mutant that kept comments survived.
The path the cut protects is a comment outside the formula,
which the strict grammar refuses unless the comment goes first;
the case now puts it there,
and the mutant is caught.
Prevention:
the M29 rule already asks that each red case fail for the reason its label names;
a case covering a pre-processing step is paired with the mutant that removes that step
before the guard is called complete.

## M58: a red case whose fixture could not reach the check it was said to pin

Status:
happened 2026-09-29 in B24's introduced-defect screen red guards (`c8d3f480e`),
caught when the fix left the case red,
corrected in `74a55e92b`,
and the red commit carries a correcting comment.
The restatement case gave its region no issue ids,
and the screen collects prior quotes only from the issues a region names,
so the case could never reach the accepted issue's quote:
it was red before the fix and stayed red after it.
The red commit's message nonetheless said the case "pins the second fold once the first holds",
a claim no mutant had tested.
This is M29's and M56's class again,
now in a commit message as well as a test.
Prevention:
a message says a case pins a branch only after the mutant that removes the branch has been caught;
before that it says what the case fails on today,
and a case that must pass a precondition to reach its branch
is read against that precondition in the code before it is committed.
Again in T8's fifteenth batch (`2d3b02c45`),
caught while writing the fix rather than after it:
B69's red case for a record that is not a JSON object used an array,
which `isJsonRecord` lets through to be probed for fields,
so after the fix it would have reached the format-version check
and been refused with that check's reason rather than pinning its own.
The fix commit (`a56cde819`) made the record a JSON string and says why in the case,
and the red commit carries a correcting comment.
The prevention's last clause covers it:
the red case's input was not read against the guard it passes first.

## M62: a copy-check case whose candidate passed validation without the check

Status:
happened 2026-09-29 in `e4cfae398`,
caught by B26's mutation batch,
corrected in `f0ddd425d`,
and the commit carries a correcting comment.
The case offered a merged candidate against a page that had lost the same heading,
where validation finds the candidate valid,
so the model was never asked with or without the copy check;
the commit message said validation "would otherwise ask the model about the heading it lacks".
The older "REPRODUCED THE INCUMBENT" case had the same shape since it was written.
This is M53's and M58's class again:
a case that fails on nothing its branch decides.
Prevention:
a case guarding a short circuit is read against what happens without it,
by a mutant that removes the short circuit,
before its commit says what it guards;
the fixture's page is named in the test file with why it makes the candidate fail (`HEADED_PAGE`).

## M85: a refusal case rewritten to name its side, checking less than it did

Status:
happened 2026-09-30 (UTC) while writing T8's sixteenth batch,
found the same day reading the diff before `c2974bea9`,
and corrected in that commit.
The block-pairing reader's out-of-range case asserted `toThrow(BlockPairingError)` on the original side.
Rewritten to cover both sides and name each,
it asserted `toThrow` with each message instead,
and module-test hands a string straight to chai's `.throw`
(`package/module/test/src/expect-matchers-core.ts` lines 138-140),
which checks only that the message contains it
(`compatibleMessage` in chai 6.2.2's `index.js`,
lines 80-81).
The class was left unchecked,
though `readBlockPairingOutcomes` records only a `BlockPairingError` as an unusable reply
and rethrows any other.
The rule in `doc/mistake-prevention.md` (moved on 2026-10-18 to `package/translation-repair/doc/mistake-prevention.md`),
that a refusal case checks the message beside the class,
was not applied to the rewrite.
Prevention:
a rewritten assertion is compared with the one it replaces,
and a matcher that takes a class or a text checks one of them only,
so a refusal is caught with `caught`,
then checked with `toBeInstanceOf` and with `caughtValueText` whole.

## M86: a short-bench case sized from a quorum not read

Status:
happened 2026-09-30 (UTC) while writing T8's sixteenth batch,
found on the first run of both short-bench cases,
and corrected before `c2974bea9`.
Both cases refused two seats of four and expected a short bench;
`rosterQuorumSize` is half the bench rounded up
(`roster-quorum-size.ts` line 20),
so two of four is a quorum,
and each case failed at its own premise,
`expect(quorum.short).toBe(true)`,
before reaching the stage.
The benches are now five seats refusing all but `MIN_STAGE_VOICES`.
Prevention:
a fixture's size is taken from the rule it must cross,
read in the code before the fixture is written,
and the case keeps asserting its premise from the exported helper,
which is what caught this.

## M89: a guard-off build whose last mutant's case never ran

Status:
happened 2026-10-01 (UTC) during B72's reader tests,
found reading the run's output,
and corrected by a second build.
One build carried three mutants:
`treeNodes` made recursive,
`emphasisSpans` parsing strictly
and `oneLine` folding every whitespace character.
The last one's case sits in `oneLine`'s suite,
the second top-level `describe` of `emphasis-spans.unit.test.ts`.
Each test file awaits its suites at the top level,
and a suite with a failing case rejects,
so the file stopped after its first suite failed,
and the `oneLine` case never ran;
the harness prints a line per failing case and per passing suite,
so it printed nothing for that case or its suite.
A second build with that mutant alone failed the case.
Prevention:
a guard-off build mutates cases in one top-level suite per file,
and a mutant whose case prints no FAIL line is read as surviving
only once that case's suite is seen to have run.

## M97: a red case hidden behind another suite's failure in the same file

Status:
happened 2026-10-01 (UTC) while writing B77's red cases,
found when the run printed no line at all for one suite,
and fixed before the red commit (`42eb0b0dd`).
`judge-crosscheck.unit.test.ts` awaited two suites at the top level.
The census suite's new red cases failed,
so that suite rejected and the file stopped before the `statusBreakdown` suite ran:
its red case printed neither a FAIL line nor its suite's PASS line,
and could not be seen failing.
The two suites now sit under one root `describe`
(the `json-guard.unit.test.ts` pattern),
and a direct run failed exactly the two new cases with every other case passing.
This is M89's mechanism met on the red side:
there a guard-off mutant's case never ran,
here a red case never ran.
Prevention:
a red case counts as red only from its own FAIL line;
a file awaiting several suites at the top level is wrapped in one root `describe`
before a case is added to any suite but the first,
and the package's other such files are queued as a family of their own.

## M104: a read of red runs that missed a case whose line was cut before its marker

Status:
happened 2026-10-01 (UTC) in T8's twenty-second batch,
found in a guard-off log,
and the runner fixed the same day.
The scratch runner of named tests (`~/temp/agent/audit-glossary-fix/run-guards-named.ts`)
prints each line holding `[FAIL]`,
cut to 400 characters.
A case whose name ran past the cut lost its own marker,
and the reads searched for that marker,
so they skipped the line.
B88's guard-off printed three lines and the read showed two;
the count in the file's header line gave it away.
Across the 420 runner logs then in that folder,
16 printed lines in 14 logs had lost the marker
(`~/temp/agent/audit-glossary-fix/b22-cut-marker-census.mjs`).
Every log still printed the file's count of such lines in its header.
Two of the runs were traced to their messages:
B84's red message cites the suite's own "2 children failed",
and B88's red message names all three lines.
The other twelve were not traced.
Prevention:
the runner now prints `FAILED: ` before every line it keeps,
so the marker survives the cut,
and a red run is read by that prefix and checked against the header's count.

## M112: an `unreachable:` throw that left its own line cold

Status:
happened 2026-10-03 (UTC),
caught at once and recorded here.
The group batch's no-boundary guard first became a throw whose message starts `unreachable:`,
the shape `mergedAttestations` uses,
and the fresh whole-suite census then listed the throw's own line
(`census-QFKBeX`,
`src/group-run-anchor.ts` at line 172)
as code no unit test runs:
an unreachable throw is unreachable code too.
The guard is removed now,
with its invariant in the comment above `settled`.
Prevention:
where a stretch must close,
the dead branch comes out rather than throwing from it,
and an `unreachable:` throw counts as one more uncased line
(`artifact-change-sets.ts` holds that shape open,
`census-QFKBeX` lines 212 to 215).
That prevention is withdrawn:
M113 records what following it cost and the rule that replaces it.

## M113: guards removed, and named errors flattened, to close census stretches

Status:
found 2026-10-05 (UTC),
reading the MiMo trial's production diff once the trial was merged;
the sites named here were fixed the same day,
and this entry grows as the rest of that diff is read.
The trial's handover told it that every census stretch ends cased,
unreachable and removed,
or a fixed defect,
and to take the census again until its counts read zero.
A guard that must stay and can never run fits none of the three.
M112 records the trial's answer,
that the dead branch comes out,
which inverts the package's rule for a broken invariant:
a state no input produces is refused out loud,
never passed on.
The answer was applied in three shapes.

Guards taken out with nothing in their place.
Each is restored as a throw whose message opens `unreachable:` and says what was found:

- `reanchorInsertions` (`group-run-anchor.ts`) wrote `NO_BOUNDARY`,
  minus one,
  as an insertion's offset when no translation block stood on either side of it;
- `anchorOffsets` (`group-source-anchor.ts`) named `NO_OFFSET` as an anchor
  in a walk whose paired steps index past the translation blocks;
- `occursOnce` (`fidelity-alteration.ts`) answered that a number missing from its passage occurs once,
  since its second search then ran from the passage's start and found nothing;
- `extractAnswer` (`grade-sheet-read.ts`) cut the opening characters off a heading without its grade marker
  as though they were the marker;
- `scanUrlRuns` (`corpus-run/dropped-destinations.ts`) left its cursor where it stood
  for a run that consumed nothing,
  which is a loop without an end;
- `describeRows` (`corpus-run/bench-report.ts`) returned `'no rows'` for an empty group,
  and with the return gone an empty group printed `NaNms per slice`;
  `summarizeBench` groups rows by a width and a pass read off those same rows,
  so no group is empty (restored in `f3b0d5db3`);
- `restoreHeading` and `restoreSignature` (`corpus-run/contributor-name-restore.ts`)
  returned `UNCHANGED` where the rebuilt line equals the line,
  and with the arms gone such a line read as rewritten,
  with a finding restoring it to itself;
  a rebuild equal to its line means the written name already is the rendering,
  which `carriesRendering` has answered before either is called
  (restored in `f3b0d5db3`;
  the writing agent measured 98,576 Han code points through the pinyin reader
  and 400,000 seeded pages through the built pass,
  neither reaching the throws).

Named errors flattened to `nonNullishOrThrow`,
whose whole message is `Expected non-nullish value, got undefined`.
The explicit throw's own line read as cold,
and `nonNullishOrThrow`'s throwing arm lives in another package,
outside the count.
The two `unreachable:` throws of `coverage-candidates.ts`,
naming the matcher and the aligner,
are back with their words,
and in `f3b0d5db3` two more,
each `RenderingAuditInvariantError`:
`withCitedReferences` (`corpus-run/rendering-audit-settled-buy.ts`) names the entry and the slice
whose page was never read for references,
and `subjectsOf` (`corpus-run/rendering-audit-settled-subject.ts`) refuses a row that passed the decided filter and is not decided.
The trial had removed that throw behind a type predicate,
which the compiler does not check against the filter's body,
and left the `@throws` describing it;
the filter returns `boolean` again.

One conversion of a fallback into `nonNullishOrThrow` was a regression,
not a flattened invariant:
`stampOf` in the cap census,
ledger B132.

Parameters added to production functions so that a case could reach a branch,
every one defaulting to the production value,
which M70 forbids:

- `fold` on `canonicalize` and `anchorLocatedSpan` (`rendering-audit-anchor.ts`),
  which let a case break the length invariant the function checks.
  The parameter and its case are gone;
  the invariant's `RenderingAuditInvariantError` stays.
- `needle` on `deleteOneSentence` (`fidelity-damage.ts`),
  which let a case name a sentence the text lacks.
  The parameter and its case are gone,
  and the arm they reached,
  an unchanged twin reported as an undamageable slice,
  is an `unreachable:` throw:
  `anchorSentence` draws the sentence from the text
  and `spliceOutSentence` returns its input only for a sentence the text lacks.
- `anchor` on `insertBorrowedSentence` (`fidelity-damage.ts`),
  gone with its case.
  The arm the trial removed beside it stays removed:
  `applySeededErrors` throws for a splice point the text lacks,
  and a splice it makes adds the borrowed sentence,
  so its result always differs from the clean text.
- `ocrTool` on `readImageWithOcr` (`image-ocr.ts`),
  which named the reader's program so that a case could name one that is not installed.
  Beside it the trial added five cases that run the real `magick` and `tesseract`,
  which the test file's own header rules out:
  a unit test must not depend on programs and language data a machine may lack.
  A required seam for the programs the function runs is to replace the parameter and those cases;
  this entry is completed when it lands.

Removals read against their callers that stand:

- `targetSize` (`coverage-tail.ts`) dropped its insertion check:
  `readUntranslatedTail` sizes only slices `isPaired` passed,
  and `makeInsertionChunk` keeps an insertion's text empty,
  so every reachable input sizes as it did.
- `openRouterChunksOf` (`openrouter-chunk-scan.ts`) dropped its opening-brace check,
  and now agrees with the completion reader (`stream-completion.ts`),
  which parses every payload that is neither empty nor the sentinel:
  a chunk whose payload opens with spaces is read by both,
  where this scan used to skip it and lose its cost and endpoint.
  Its TSDoc still described the check;
  it describes the parse now,
  and an `openrouter-cost.unit.test.ts` case reads a cost off such a chunk.
- `anchorOffsets` reads its walk through `nonNullishOrThrow` inside a loop bounded by the walk's own length,
  an element read that never had words of its own.

Test cases the trial added that could not fail,
read by three independent reviews of its test additions:
`33ea944b4` corrects eight corpus-run test files
(`published-page-check`,
`meter-sample-read`,
`meter-dry-span`,
`final-selection-completeness`,
`list-spread-restore`,
`rendering-audit-settled-input`,
`slice-census-entry`
and `spend-read`),
each case now asserting a whole result on an input where the code it names decides it,
and case names lose the tags naming the T8 batch and cluster that wrote them,
which said nothing about what a case asserts.
`f3b0d5db3` corrects the test files of its own modules the same way.
The production defects such cases pinned as intended,
or could not see,
are B128,
B130,
B131,
B133 and B139.

Found beside these:
`groupNodesSealed` returned no runs at all,
without a word,
for a supplied walk pairing past the translation blocks
(two source blocks,
one translation block,
a paired step naming the fourth:
`{"runs":[],"sealedSourceIds":[]}` on the build of `7f069adc9`);
decided and fixed as B138.

Open when this entry was written,
closed in B197 to B200,
found by the agent that wrote `f3b0d5db3`:
the slice census sizes an added block by the block the aligner left over,
so an added paragraph of 42 or of 213 characters reads as 39,
since `estimateExpansion` (`align-blocks.ts`) divides a section's whole translation by its whole original
and the added block raises the length expected of every rendering
(the cause read off the source,
not measured in a fork;
`slice-census-entry.unit.test.ts` asserts the column only where it reads right);
`restoreHeading` cuts the page title at its last colon to find the prefix,
so a rendering that holds a colon is written twice into its heading,
`### Three: Mao: Cat` becoming `### Three: Mao: Mao: Cat` with a restore reported
(the agent's probe;
the code is older than the trial,
and which colon ends a prefix is a design choice);
`finalSelectionFindings` reads an artifact whose contest never ran as nothing to report,
an arm no production input reaches,
since `settledEntryArtifact` writes its kinds as literals;
`censusEntry` counts translation-only blocks with the deterministic aligner even under a settled recipe,
so a settled row mixes two carves (read,
not probed);
and `readLists` (`list-spread-restore.ts`) drops a list whose item has no offsets,
silently,
which would move every later list's position
(the parser sets positions on what it parses,
an inference).

Prevention:
`mistake-prevention.md`,
"Guards a census wants gone".

Closed on 2026-10-05 (UTC) for the `ocrTool` seam (`e300e6c33`,
`d550bf04f`):
the seam landed in place of `ocrTool`.
`readImageWithOcr` (`image-ocr.ts`) requires `runProgram`,
a `ProgramRunner` both decoders and the OCR reader go through,
and `decodeToPng` takes the same runner;
nothing defaults it.
The real runner is `runInstalledProgram`,
named once,
where `RUN_PICTURE_SOURCES` (`corpus-run/pass-visual-evidence.ts`) binds it through `ocrReaderOver`.
The trial's five cases ran the real `magick`,
`dwebp` and `tesseract` with `chi_sim` data,
and the older undecodable case relied on the real decoders failing;
all six are gone.
Every case of `image-ocr.unit.test.ts` about the reader now runs through `scriptedPrograms`
(`scripted-programs.test-fixture.ts`),
which starts no process
and whose stand-ins copy,
transcribe,
refuse or are missing at the paths the reader names,
and each case asserts the reading,
the programs run with their arguments,
and every line logged.
The stand-in rejections copy Node's spawn and exit failures field for field,
and two cases hold them against the rejections `runInstalledProgram` raises.
Prevention:
`mistake-prevention.md`,
"Tests touching the real world":
a program on the machine is an outside read like the corpus,
and its runner is a required parameter.

Removals read against their callers and left standing on 2026-10-05 (UTC)
(`df0bcb85a`,
`ba243b3c3`,
the verifying agent's reading):

- `attachClaimFilers` (`claim-filers.ts`) reads a claim's filers through `nonNullishOrThrow`
  behind the filter that kept the claim for that very key.
- `containerHalfPairs` (`container-half-pairs.ts`) reads the half it splices out through `nonNullishOrThrow`;
  the index comes from the same list's `findLastIndex`,
  checked against not-found first.
- `classifyDisplacement` (`displacement-class.ts`) reads a slice's class and a high slice through `nonNullishOrThrow`;
  both arrays are maps of the same readings.
- `listedNames` (`edit-prompt.ts`) reads the last name through `nonNullishOrThrow`
  in the arm taken only for two names or more.
- `declinedTargetBlocks` (`declined-target-runs.ts`) keeps its `nonNullishOrThrow` node read,
  behind the checks every production path makes first,
  `blockPairingToSteps` and `walkIntoRuns` (B138);
  the comment that credited the alignment for it now names them.
  The verifying agent found,
  on a base before B138,
  that a pairing read back from a settled artifact naming a block past the section reached that read,
  declining every block or throwing a missing-value error that named nothing,
  and refused it in this module with a `RangeError`;
  B138's `BlockPairingError` now meets that pairing upstream,
  so the merge kept B138's refusal,
  and two cases in `declined-target-runs.unit.test.ts` pin it through this module's entry.

Named errors flattened to `nonNullishOrThrow`,
one more
(`49f3ebdbf`,
`699ad2f8c`):
`readImagePair` (`image-reading-pair.ts`) named the roster slot of a reader whose exchange threw
with an explicit throw,
and the trial replaced it with `nonNullishOrThrow`
and a comment crediting a capability gate.
The slot is present because the settlements are the roster mapped one ask per slot;
the throw is back with its words,
in the `unreachable:` form,
and the comment gives that reason.

A check cut in half so that another would run:
`inspectParagraph` (`inspect-paragraph.ts`) lost the kind half of its structure check
so that the leading-block check behind it went live,
which moved a heading beside broken definitions from `not-one-paragraph` to `unparseable`
(B157).
The check is whole again,
and the leading-block check is an `unreachable:` throw.

## M115: an invariant throw counted as code no test ran

Status:
found 2026-10-05 (UTC) on review of M112,
fixed in `c1f846ddb`.
The delegated agent that wrote the change drafted this entry under the code M113,
which was taken by then.
M112's prevention said the dead branch comes out rather than throwing from it,
and that an `unreachable:` throw counts as one more uncased line.
That inverts the package's rule,
under which a broken invariant throws `new Error('unreachable: ...',)`,
and it made the census a standing reason to delete every such guard:
no honest input reaches one,
so each read as code no unit test ran for ever.
By the agent's count,
`rg --fixed-strings 'unreachable:' src --glob '!*.test.ts' --count-matches` printed 42 in 24 files,
29 of them keys and type members named `unreachable`;
the unminified coverage build held 13 such throws,
and the whole-suite census `census-aU1H0y` at `9fe80a109` left nine of them cold:
`src/artifact-change-sets.ts` 212 to 215,
`src/artifact-read.ts` 138 to 150,
`src/budget-hold-wait.ts` 188 to 191,
`src/chunk-measure.ts` 245 to 248,
`src/cited-reference-scan.ts` 272 to 277,
`src/corpus-run/published-page-check.ts` 556,
`src/dedupe-issues.ts` 160,
`src/splice-slices.ts` 263
and `src/transient-retry.ts` 584 to 585.

The census now reads each cold stretch's own text in the coverage bundle in one linear pass
(`coverage-stretch-atoms.ts`,
`coverage-invariant-throw.ts`).
A stretch in which every statement throws `new Error(...)` whose first argument is one string or template literal beginning `unreachable:`,
or a class whose name ends in `InvariantError`,
is an invariant throw;
braces,
a semicolon,
`else` and the `if` head guarding a throw may stand in it,
and anything else keeps the whole stretch cold,
as do a slash the pass cannot tell for a division or a pattern
and a function no process called.
`placeTally` places such stretches apart,
so no row,
total or baseline reading counts one,
and the report prints their count beside the cold counts and lists each by source,
lines and class thrown.
The census file is format 3 (`CENSUS_FORMAT`) and keeps them under `invariantThrows`;
`readBaselineCensus` refuses format 2 by name,
since its stretches hold invariant throws among the cold code with no text to tell them by,
and read against a run that leaves them out each would read as ran.
Replayed through the changed build with no test run
(a replay script in the agent's scratch),
`census-aU1H0y` reads library source as 83 files and 110 stretches over 273 lines,
with nine invariant throws,
which is the 90 files,
119 stretches and 309 lines it recorded.
None of the nine is of the class form,
whose six throws tests reach.

A whole-suite census of the change has not been taken on the branch yet.
The one the agent ran refused on a failing suite:
`bench-sample-draw.unit.test.ts` reaches `git` through a shim whose `git-policy-cli` build its checkout lacked.
Of the 286 `census.json` files under the package's coverage cache on this machine,
none was written after `c1f846ddb`,
so every stored baseline is format 2,
and the next census is taken without one and becomes the baseline.

Prevention:
an `unreachable:` throw stays,
the census counts it apart,
and a guard is never removed to close a stretch.
A stretch closes by a case that reaches it,
or,
where no honest input does,
it is left to the invariant count.
This supersedes M112's prevention,
and T8's rule that a span no runtime value can reach is removed as dead code is read with it:
a span that guards an invariant is not dead code.
`mistake-prevention.md`,
"Guards a census wants gone".

## M167: a brief's example claimed a failing result that nobody had run

Status:
happened on 2026-10-13 in the brief for the follow-ups to the mark rule;
disclosed by its author in the hand-back.
The brief gave an example for the writer item,
a chosen place of other words,
and said the reader refuses the section without the fix.
The reader accepts that section.
The red shape is a chosen place that holds the equal words,
beside a receiving place that holds them with a mark.
The author ran both shapes,
said so plainly,
and kept the first as a second round-trip case that was never red.

Prevention:
an example in a brief that claims a failing result is run before the brief is sent,
or is written as a question.
`mistake-prevention.md`,
"Briefs and hand-backs".

## M179: the property oracle counted the proven apostrophes as loose marks

Status:
happened on 2026-10-13 in the work on B488;
caught by the author before any result rested on it.
The oracle of the sixth review's grids counted a proven apostrophe as a loose mark,
so under the new rule it would have flagged correct carries.
The author wrote a new independent oracle,
and showed with a positive control that it detects a planted gain case and a planted attach flip.

Prevention:
when a rule changes what a property means,
the oracle is rewritten and shown to detect a planted violation before any grid is run with it.
`mistake-prevention.md`,
"Guards that cannot fail".

## M268: a test of an ordering used a fixture that does not differ between the two orders

Status:
happened on 2026-10-18 (UTC) in the test round `w188`.
The commit `bcf0394584` (`f6b787eac0` on the branch) pinned the order of equal-length candidates by code points
and not by UTF-16 units,
with a fixture of U+A730.
U+A730 lies in the basic plane at a lower code point than the surrogates,
so it sorts the same in UTF-16 order and in code point order,
and the test could not tell the two apart.
The notes do not say how it was noticed.
The method of the round requires each pin to fail under its mutation,
and this fixture could not.
`9aadabd6f1` (`3b18ef4de5` on the branch) replaced it with `a` plus U+E000 against `a` plus an astral character,
which the two orders sort differently,
and the new fixture was proved.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a fixture for an ordering is shown to give opposite answers under the two orders before the test is committed,
and the pin is then proved by its mutation,
as the round proved the replacement.
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## M269: mutation specs of the demonstration named text that had moved or changed

Status:
happened on 2026-10-18 (UTC) in the test round `w188`.
Two specs for the mutations named for n173,
and the spec for the combining mark before a match (`n171-B645-mark-before`),
had find strings that no longer matched,
or named a file that had moved (`consistency-contains-words.ts`).
The round used the specs of the demonstration (`w181`) and,
where a file had moved,
an equivalent by content;
the current specs are in `spec-w188.ts`,
outside the repository.
The notes record the specs as stale and name no further effect.

Prevention:
the notes name none.
Written in this filing and not given by the notes:
a mutation spec is applied to the base and shown to match exactly once before a proof relies on it,
and a spec that does not apply is rewritten and kept with the proofs.
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
