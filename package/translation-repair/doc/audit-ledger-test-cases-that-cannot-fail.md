# Audit ledger: test cases that cannot fail

Paths written `src/...` in this record name the layout before 2026-10-18,
when the package `package/module/translation-repair` was split into the packages under `package/translation-repair`.
`doc/audit/translation-repair-package-split-map-2026-10-18.json` gives each file's current path.

Cases and guards that would pass with the behaviour they name removed.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B33: artifact refusals that say the opposite of what they expect

Status:
fixed 2026-09-30 (UTC),
found the same day by T8's fourth batch,
whose probe printed each refusal of the consolidation reader whole.
`ArtifactParseError` prints `artifact parse failed at <path>: expected <reason>.`,
so a reason is read as what the reader wanted there.
Six were written as something else.
Two said the opposite of what the reader wants:
text under a terminal that changed nothing printed "expected text to ship,
from a slice whose terminal is … and settled on no change",
and a contested slice outside the comparison printed "expected index naming no recomputed comparison row".
Four were sentences that do not follow "expected":
the three refusals of a consolidated terminal's shipped record
("expected a slice whose terminal is consolidated must carry the text it ships",
and two that ship an archive or incumbent instead),
and a polish whose changed flag disagrees with its text
("expected whether final text differs from baseText").
No test caught them,
since every case quoting one checked a fragment
("settled on no change",
"must carry the text it ships")
that the wrong reading also contains.

The census (`reason-census.mjs` in the audit's scratch folder,
over the package's own parser)
read all 145 reasons the package's source passes to the class;
a text search finds 146 constructions,
the extra one the example in the class's own TSDoc.
The other 139 read as what was expected.
Phrases naming a state
("present in one slice",
"unfilled,
since …")
read acceptably after "expected" and stay.

Fix:
`74387b886` rewords the five in the consolidation and contest readers
("expected no text to ship,
since a slice whose terminal is … settled on no change";
"expected an index naming a slice the recomputed comparison holds";
"expected the text a slice whose terminal is consolidated ships",
and so on),
and `ebaa97908` makes the polish flag's reason name the value
("expected true,
which is whether final text differs from baseText").
Each test quoting one now checks the whole message.
No scan can tell a well-formed reason from another,
so the prevention doc's rule is to read each new refusal's message whole in its test.

## B50: a test named for an exit the code stopped taking

Found by the eighth coverage batch (census-dUOMFS):
the census found the settlement's absence exit cold,
yet a case in `consolidate-settle.unit.test.ts` was named for it,
"the absence the judged round raises over an ineligible standing".
Its run's log shows the judging returning "nothing proposed" instead:
since B39,
a withheld slate on which a voice was heard keeps its wording rather than raising,
so the case's input no longer reaches the exit its name promised,
and the name kept that exit looking covered.
Renamed in the B50 commit to the path it takes,
with the `translate-no-candidate` finding asserted;
whether the absence exit can still be reached at all is its own entry.

Recurrence:
when a change moves an input to a different exit,
rename or repoint every case named for the old one;
a case name is a coverage claim no runner checks.

## B99: suites after a failing one in the same test file never ran

Found 2026-10-01 (UTC),
after it had hidden a case three times (M89,
M97,
and one of B92's red cases),
fixed in the commit adding this entry.
module-test's `describe` throws when a child fails,
and the package's test files awaited each suite at the module's top,
so the first failing suite's throw ended the module
and no suite after it ran.
The runner prints a line per failing case and per passing suite,
so a suite that never ran printed nothing,
and a file reported its first failing suite as its only one.
A codemod scanning the package's 906 test files
(`multi-suite/wrap-suites.mjs` in the audit's scratch folder)
found 291 awaiting more than one suite at the top.

Each now awaits one root suite,
`describe({ name: '', concurrency: 1, children: [...] })`,
whose children are the suites the file held,
in their order:
the root runs them one after another,
as the top-level awaits did,
and each child keeps the concurrency it set or takes `DEFAULT_CONCURRENCY`,
the default it had before,
which `module-test` now exports.
The codemod wrapped 276 files.
It refused 15 where a declaration sat between two suites;
an agent hoisted each such declaration unchanged ahead of the root,
in its order,
after reading that no suite before it reads what it names
and that the one with an effect (`prompt-payload-store.unit.test.ts`'s `writtenVersion()`,
which writes a file in a directory of its own) changes nothing an earlier suite sees.
A hoisted declaration now runs before any suite,
so one that throws stops every suite in its file rather than only the later ones.
Four files whose every suite already set its concurrency imported `DEFAULT_CONCURRENCY` for nothing after the codemod;
the unused-imports scan found them in the full suite,
and they were removed.

What enforces it:
`src/multi-suite-top-level.unit.test.ts`,
among the source scans,
fails on a `.test.ts` file holding more than one `await describe(...)`
held only by top-level statement shapes,
the shape `global-writes-sequenced.unit.test.ts` reads as a file's top-level suite;
the two scans now share `parentsOf` and `ancestorsOf` from `source-scan.test-fixture.ts`.
Its fixture case runs first and passed from the start.
Its package case failed naming the 15 hand-wrapped files before they landed and passed after;
with a second top-level suite appended to `code-points.unit.test.ts`,
it failed naming that file alone,
and passed once the line was removed.
Out of its reach:
a suite awaited inside a function the file's top calls,
and `describe` imported under another name.

Calls made here are open to veto:

- the root runs its suites one at a time,
  keeping the order the top-level awaits gave,
  rather than side by side;
- a declaration between two suites moves ahead of the root rather than into a case.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B143: a scripted client no prober ever heard

Fixed in `9c2eb03d9`,
the test commit of B141.

Found by the same review.
`scriptedSettleClient` in `refine-slice-settle.unit.test.ts`,
written by the trial,
offered every exchange one list of replies and returned the first its guard accepted.
The list ended with `EMPTY_INTRODUCED_DEFECT_REPORT`,
the probe's result object,
which no stage's wire accepts,
and held a checker reply only under `worseTally`.
So every run through it ended on `stage-quorum-unmet (introduced-defect-probe 0/2)`,
a run without `worseTally` heard no checker either,
and its comment said the replies were ones each call's guard accepts.
No case asserted the findings,
so nothing showed it.
No case passed because the prober went unheard:
the sheet cases asserted text recorded before any reply was validated.
One half of the recheck case passed on the unheard checker round,
which is the recheck defect.

The fix:
the client scripts by the stage's structured-output name,
each reply checked against the guard the built package exports for that stage,
and every scripted case asserts the settlement's findings whole,
so a stage the script fails to answer shows as a lost voice.
The cases run on a checker bench of three,
the floor the refine phase holds a run to.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail":
a scripted client is shown heard by each stage it serves,
by a case asserting that stage's findings or its heard count,
before any case built on it is read as evidence.

## B146: cases whose expectation was the function's own output

Fixed in `adce3f9a8`.

The three B123 bounds cases took their expected text from what the fixed function returned,
so they asserted the URL rewrite as right,
and one case name said the reference "stays in its text" while the tree held it in the link.
Two more cases named a shape as no marker under a map that named another label,
so a misread could not fail them.
The corrected cases set the reference off from the literal by a space,
the glued form asserts the refusal a tokenized twin gives,
and the two inventory cases assert `documentLabels`.
Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail":
an expectation for a parse-dependent result is written off the printed tree,
and a case that names a shape as inactive asserts the inventory,
or a map that names the shape.

## B149: the coverage control called a cut that left nothing evidence it could not locate

Red in `9c03075d7`,
fixed in `341211543`.

Found 2026-10-05 (UTC),
reviewing the MiMo trial's test cases (review part 4,
F3).
`withoutSpans` (`corpus-run/coverage-control.ts`) answers blank for a document no span was in,
and blank again when the spans were the whole document.
`tryCase` read every blank as `evidence-not-locatable`,
documented as a carried verdict none of whose spans could be found.
That state no input produces:
`judgeCoverage` calls a passage carried only on a full claim it anchored,
and fills `evidence` with the translation's own text for each such claim,
the same translation `tryCase` cuts.
A roster quoting the whole translation was refused as not locatable with all three spans found.

The fix:
the refusal reason is `cut-left-nothing`;
`tryCase` checks every span is non-blank and in the translation before it cuts,
and throws `unreachable:` where one is not;
the reason's union and documentation say so.
`coverage-control-probe.ts` read the old reason without naming it and said such a case's evidence could not be found;
its lines were brought in line in the same commit,
and `d233f882f` reworded one of them that the count-nouns scan caught.
The census note in the T8 entry on code no unit test runs keeps the old name,
with a note.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B150: a coverage control held without a decoy that ran

Red in `9c03075d7`,
fixed in `341211543`.

Found 2026-10-05 (UTC),
reviewing the MiMo trial's test cases (review part 4,
F5).
A page with no room for a decoy cut clear of the anchored spans records `no-room` and no absence vote.
`coverageControlHolds` took the decoy half over every row,
so such a row counted as a decoy that was taken and moved nothing.
Three rows with no room and a roster following the quote held,
with no decoy ever asked;
one decoy that drew the vote beside two such rows held too.

The fix:
the result carries `decoysTaken`;
the control holds only where at least one decoy was taken,
and no more than half of the decoys taken drew the vote;
a control that took none prints why it cannot hold.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B178: trial cases on the audit and the OpenRouter client that could not fail, or ran on claims no screen builds

Changed in `96ef8323e`,
a commit of test files alone.

Found on 2026-10-05 (UTC)
by the first and fourth reviews of the trial's added cases
(the first review's findings 5 and 14,
the fourth's F14 and its notes on weak assertions and test seams).
In `rendering-audit-corroborate.unit.test.ts`,
one case paired an omission that spans the candidate with one that leaves it unused,
a pair `screenRenderingAudit` cannot hand on,
and named a guard no input can tell from its absence:
with the guard cut out of the built `intersects`,
every case of the file still passes.
Two cases read a length and a voice count where the whole result is knowable,
and the trial's six cases sat in one suite and repeated one claims literal five times.
In `rendering-audit-screen.unit.test.ts`,
a case reached `quotesRequired`'s invariant throw with a category cast past its type
and claimed to catch a category added to the vocabulary and forgotten in the rule,
which a made-up word cannot do.
In `openrouter-client.unit.test.ts`,
the case named for the deadline and `maxAnswerChars` asserted the answer alone,
and the `/credits` refusal was read by class and status.

The fix:
the impossible pair and the cast case are deleted.
`quotesRequired` keeps its export behind a case that asks it for every member of `RENDERING_AUDIT_CATEGORIES`
and asserts the whole table,
so a list added to the vocabulary without the rule fails there.
The corroboration cases assert whole results over one fixture anchored on the file's own texts,
each in the suite of the function it tests.
The knob case asserts,
for a call with both knobs and a call with neither,
the answer bound on the exchange and whether its signal is the caller's;
the refusal case asserts the class and the whole message.
Eleven mutants of the built package,
one per thing a case names,
each fail the case that names it (the fixing agent's runs).

Open to the owner's veto:
the both-unused overlap case pins the member order the build returns,
second voice first,
where `corroborate` keeps arrival order and nothing reads either.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B179: nothing asserted the endpoint and the cached tokens on a completed call's spend line

Changed in `96ef8323e`,
a commit of test files alone.

Found on 2026-10-05 (UTC),
checking the fourth review's note on `reportedSpendFieldsOf`.
The trial cut the three conditional spreads of `chatText` (`openrouter-client.ts`)
into an exported helper and cased the helper,
on the reason,
recorded under T8,
that the cached field's only observable is a spend line `chatText` discards.
The line is logged by `reportSpend`,
and `client-log-context.unit.test.ts` already reads such lines off `console.info`.
With `chatText` handing the helper no endpoint,
or no cached count,
`openrouter-client.unit.test.ts` printed no failure;
only the cost was held,
by the meter cases.
The extraction itself changes nothing:
280 combinations of the three readings give the same fields as the inline code at `28303c42a`.

The fix:
a case reads the spend line a completed call writes for a stream reporting a cost,
an endpoint and cached prompt tokens,
and asserts it whole.
Each of the three mutants now fails it (the fixing agent's runs).
The helper and its case stay.

Recurrence:
`mistake-prevention.md`,
"Claims without their evidence":
"the only observable" is a claim to measure,
by looking for a test that already observes the thing.

## B188: two reading-cache cases refused their records for a reason they did not name

Changed in `7f99edb6a`,
a commit of test files alone.

Found on 2026-10-05 (UTC)
by the agent that fixed B149 to B153.
Two cases of `corpus-run/reading-cache-store.unit.test.ts`,
older than the trial,
built an unavailable record without `transient: false`,
so the closing check of `isPairedReading` refused each whatever field it named,
and each stayed green with its own guard removed.

The fix:
each record has one cause,
the reason `reader-failed` or readings that are strings,
and each case also resumes the same record with that field valid.
With the reason guard removed the first case fails,
and with the readings guard removed the second does
(the fixing agent's runs,
the source restored each time).

The module TSDoc of `corpus-run/coverage-control-vote-change.unit.test.ts` said one scripted answer served every round,
where the file holds three scripted clients:
the blind client,
the sheet-reading client with its two conditions,
and the denying client.
It says so now;
nothing else in that file changed.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B211: trial cases in seven files that could not fail on what they named

Changed in `bbed06b87`,
a commit of test files alone.

Found on 2026-10-05 (UTC)
by the fourth review of the trial's added cases.
No production file changed:

- `corpus-run/name-gloss-restore.unit.test.ts`:
  both cases were refused before the line shape was read,
  since the archive carried the name once and the shipped text never carried it,
  and the comma case asserted that two lengths were at least zero;
  each now has a fixture the line shape decides and a control that restores,
  with the restored row and the finding whole;
- `nudged-reask.unit.test.ts`:
  typed fixtures replace casts,
  the path where both answers fail returns a value the second answer cannot be mistaken for,
  and new cases hold a usable first answer,
  which buys no second ask,
  and the request the second ask receives,
  the nudge last and the first provider excluded;
- `provider-router.unit.test.ts`:
  the last-refusal case answered 503,
  which is no budget refusal,
  and so stopped at the first provider;
  it now refuses on budget at both providers and asserts the last refusal whole,
  and a new case holds a failure that is no budget refusal rethrown at once;
- `restoration-judge.unit.test.ts`:
  the whole judgments record,
  where `votes` shows the ballot that named one seed only;
- `hyper-client.unit.test.ts`:
  the answer-length knob and the replaced signal are read off the recorded exchange,
  not searched for in a JSON string;
- `corpus-name-index.unit.test.ts`:
  the class,
  the whole message and the kind;
- `corpus-run/tag-attributes.unit.test.ts`:
  whole readings where it asserted mapped names,
  and the batch tag is gone from six case names.

Each changed or added case failed against a mutation of the production line it names
(the fixing agent's runs,
the mutants restored).
Its one stated gap:
the unclosed-quote half of the first name-gloss case cannot fail on one guard alone,
since two guards refuse that line.
The commit was not gated alone.
The whole suite next ran two commits on,
on `cf50117c5` in a side checkout:
1,607 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 36 source scans passing.

Open to the owner's veto:
the 503 case stands beside the older 500 case and does not replace it;
and the `hyper-client` case asserts that the signal was replaced,
not how long the deadline is,
which the wire does not show.

Open:
two older `refine-wire` cases use a dog sentence as fixture text,
an older `corpus-name-index` case asserts mapped names,
and an older `provider-router` case asserts the class alone.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B225: trial cases in twelve files that could not fail on what they named

Changed in `0b687fbdf`,
beside the fixes of B222 to B224.

Found on 2026-10-05 (UTC)
by the first review of the trial's added cases.
No production file changed for these:

- `corpus-run/roster-card-render.unit.test.ts`:
  a row asserted to be other than `undefined`,
  which `listingRowFor` never returns,
  is asserted whole,
  and the Bedrock card is asserted as its five fields where two were read;
- `corpus-run/cap-override.unit.test.ts`:
  the environment case compared the default call with its own expression,
  so a reader ignoring the environment passed it;
  it now sets the variable through a disposable and reads 15,
  clears it and reads the fallback,
  in the suite of the function it tests,
  which runs one case at a time;
- `corpus-run/window-trial-slice.unit.test.ts`:
  the insertion case searched the translator's sheets for a sentence an insertion run is never given;
  it now reads the judges' sheets,
  every one saying nothing stands where the target is an insertion and none saying so where a rendering exists,
  and the draw refusal is asserted by class and whole message;
- `han-title.unit.test.ts`:
  the line-end refusal is asked for under the form where it alone keeps a two-line title out;
- `corpus-run/cache-account-read.unit.test.ts`:
  a declaration that runs to its line end joins the refused list,
  and the line-end case moves to the suite of the function it tests;
- `corpus-run/runs-lock.unit.test.ts`:
  the holder's message is asserted whole in a case of its own,
  where `toContain` checks had been appended to another case,
  and the lock line is asserted as its whole text;
- `coverage-stage.unit.test.ts`:
  scripted verdicts said `all`,
  a word outside the wire's vocabulary,
  and now say `full`;
  the misattribution case asserts the whole verdict;
- `introduced-defect-wire.unit.test.ts`:
  the one-neighbour case asserts the whole nearby block,
  the empty half included;
- `model-content.unit.test.ts` and `provider-budget-refusal.unit.test.ts`:
  cases move under the function they test,
  and the payment refusal and the stated wait each gain cases of their own;
- `seeded-error.unit.test.ts`:
  both refusals are asserted by class and whole message;
- `refine-eligibility.unit.test.ts`:
  the case behind B224 is deleted.

The fixing agent's runs,
not rerun by the lead:
thirteen mutations of the production lines the changed cases name,
each turning its case red and restored from a byte copy.
Its stated gaps:
the change from `all` to `full` was not shown red by a mutation,
and the branch of `lockRunsDir` that names the holder after a lost race is still run by no case.
The whole suite's run on `f7653073b` is recorded in B222.

Open when this entry was written,
closed in B256 (`042a9245d`):
that branch,
and dog-themed fixture text left in two older cases
(`seeded-error.unit.test.ts`,
`introduced-defect-wire.unit.test.ts`).
A follow-up holding both was launched on 2026-10-06 (UTC)
and merged in that commit.
Two cases reach both arms of the lost takeover;
the rebase case of the first file is cat-themed and its case with a dog needle is deleted;
and a search of both files for `dog` at `f8e99b84d`,
letter case ignored,
finds nothing,
where the same search for `cat` finds lines in each.

Open to the owner's veto:
`concurrency: 1` on the suite whose case writes a process variable.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B226: trial cases in eight more files that asserted less than their names said

Changed in `27726ecc0`,
a commit of test files alone.

Found on 2026-10-05 (UTC)
by the third review of the trial's added cases.
No production file changed:

- `sample-draw.unit.test.ts`:
  the case ordering one entry's issues used a seed
  whose shuffle agrees with arrival order and with alphabetical order,
  so a sort by id passed;
  it uses a seed that puts `issue-b` first and runs both arrival orders,
  and `candidateOf` no longer casts a `sourceAnchor` the type does not hold;
- `sample-grading.unit.test.ts`:
  two casts that hid fields the types lack are gone,
  the repair is asserted to be the same object,
  and the byte-count refusal is asserted by class and whole message;
- `corpus-run/contributor-name-authorities.unit.test.ts`:
  the case is named for the missing signature it tests and asserts the whole map with its origin;
- `corpus-run/editor-standing-read.unit.test.ts`:
  the runs-directory case passed on any directory,
  since the summary prints its count whatever was read;
  it pins the first summary line and the fixture's own refusal,
  and the seated run asserts the whole report;
- `corpus-run/editor-width-report.unit.test.ts`:
  rows shipped on the narrow arm alone and on the wide arm alone join the two that could not tell the arms apart;
- `corpus-run/probe-telemetry-report.unit.test.ts`:
  each note is shown absent where its count is zero;
- `corpus-run/slice-cache-namespace-resume.unit.test.ts`:
  a bare rejection becomes the permission message,
  and a case that depends on no file mode shows a raising guard surfacing;
- `fidelity-alteration-variant.unit.test.ts`:
  two case names say what their cases test and lose the batch tag.

Each changed case went red against one production break made for it
and green once the break was restored from a byte copy
(the fixing agent's runs,
not rerun by the lead);
the two renamed cases have no red run.
The whole suite's run on `f7653073b` is recorded in B222.

A correction to the commit's message and to the agent's report,
both of which leave open a field `RankedCandidate.issueKey` as written and never read.
`RankedCandidate` (`sample-draw.ts`) has no such field at `27726ecc0`:
the field stands on `KeyedCandidate`,
where `byIssueKey` reads it to order one entry's issues,
as B127 records.
A follow-up launched on 2026-10-06 (UTC) was handed the same claim.

Open:
the permission case still depends on running as a user a file mode can refuse.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B256: cases in six test files could not fail on what they named, and two housekeeping leftovers

Tests in `042a9245d`,
housekeeping in `51ccd8680`.

Found on 2026-10-06 (UTC) by the pass B254 records,
over cases earlier entries called weak and a branch that had no steady case.
Each of these passes before and after:

- `refine-wire.unit.test.ts`:
  two cases use cat fixtures and assert the whole operations
  (envelope id,
  base hash,
  new text),
  where they read a length and one id.
- `corpus-name-index.unit.test.ts`:
  three cases assert the whole lines,
  the names and the sorted list of paths asked.
- `provider-router.unit.test.ts`:
  four refusal cases assert the whole message,
  where they read the class alone or a part of it.
- `seeded-error.unit.test.ts`:
  the rebase case asserts the seeded text and the whole applications.
  The case "throws SeedApplicationError on absent and ambiguous needles" is deleted:
  it asserted the class alone,
  and "REFUSES A NEEDLE THE TEXT LACKS" in the same file holds both messages whole (read by the lead).
- `introduced-defect-wire.unit.test.ts`:
  the refusal case sends a region that is the string "1".
  The old input,
  the string "one",
  could not tell the type guard from the integer guard.
- `corpus-run/runs-lock.unit.test.ts`:
  two new cases reach both arms of the lost takeover,
  a rival's lock that reads back and one that does not,
  through a child process that wraps `rename`,
  so the rival's lock appears the instant the stale one is evicted.

The agent's measurements,
not rerun by the lead:
with a guard or a wording of the production code mutated,
each new case fails on its mutant,
and the old files pass the mutants for the router's hold wording and two refusal reasons,
the wire's type guard,
the dropped base hash in two refine cases and the lock's holder arm.

Two leftovers closed in `51ccd8680`.
Nine production files cited a scratch path in a comment;
each now cites the ledger entry or the document holding the figure,
or states the fact without a path,
and a search for the scratch root over production sources returns nothing (the agent's search).
Fifteen names were re-exported twice across the barrels;
the second statement of each is removed (`index.ts`,
`pipeline-barrel.ts`,
`bedrock-barrel.ts`,
`sheet-barrel.ts`),
with 2,885 distinct explicit names and the built index's 1,938 runtime export keys unchanged (the agent's count).

One item of the pass was a false premise.
The lead's brief,
repeating commit `27726ecc0`'s message,
asked about a field `RankedCandidate.issueKey` as written and never read.
That type has no such field at that commit;
the field stands on `KeyedCandidate`,
where the sort of one entry's issues reads it
(B226 records the correction,
posted on `27726ecc0` as commitcomment-203602282).

Open to the owner's veto:
the deleted case;
making the lock's race by wrapping `rename` in a child process;
the barrel deletions.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B331: a damage case checked only that its text held the borrowed sentence

Fixed in `e1c84094e`.

Found by a cross-check of the four review reports on the trial's test cases against the branch,
one of six findings the cross-check left open.
The case that skips an empty donor sentence and borrows the first usable one
(`fidelity-damage.unit.test.ts`)
asserted that the damaged text contains the borrowed sentence,
which a sentence set anywhere,
or a wrong count or detail line,
also satisfies.

The fix:
the case asserts the whole attempt:
its kind,
the damaged text with the sentence set after the clean text's second sentence,
the changed count and the detail line,
each count derived from the borrowed sentence's length.
The lead measured that value from the built library before writing it
(by the lead's notes,
89 changed characters,
the borrowed sentence's length).
The whole suite on `e1c84094e`,
run in a side checkout:
2,017 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and 44 source scans passing.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail":
a case over a pass that rewrites text asserts the whole text it returns.

## B335: three guards no case can tell from their absence

Changed in `e1c84094e`.

Found by the same cross-check:
`list-spread-restore.ts`,
`fidelity-damage.ts` and `rendering-audit-corroborate.ts` each hold a guard
a case reaches and passes without.
Each is an equivalent mutant,
so no case can be written:
lists pair only at the same item count,
so a gapless list is gapless on both sides of its pair and reads alike either way;
every text includes the empty string,
so the refusal of an empty sentence holds without its line;
and comparing missing bounds is false,
so an unused side intersects nothing without its line,
which the type check needs to reach the bounds at all.
The lead measured the last two in Node.

The fix:
each guard stays,
with a comment at its site saying why no case can separate it.
B178 had recorded that every case of its file passes with the third guard cut out
and deleted the case that named it,
leaving the guard with no word on why it stays.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B348: the bench-draw cases asserted fragments where the whole value was known

Changed in `de6ff17`,
a commit of test files alone.

Found on 2026-10-06 (UTC) by the agent writing the runner observations.
`corpus-run/bench-draw.unit.test.ts` held three `toContain` assertions where the whole value was known:
the empty-pool refusal,
the middle-width case
and the one-seat refusal.

The fix:
each asserts the whole value:
`BenchDrawError: a bench sample cannot be drawn from no slices` as the refusal's `String`,
`{ widths: [2, 3, 4, 5, 6], repeated: 4 }`,
and `BenchReportError: a roster of 1 cannot be benched: nothing to vary`.
The runner observations change rewrites `BenchReportError`;
by that change's own report it prints the same text,
and if it lands with another the one-seat refusal's expectation moves with it.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B362: the synthetic client's case on arming the deadline inside the slot could not fail

The guard's red is `1f983c322`,
and the case changed in `4cca8c301`;
its red is the agent's control.

Found on 2026-10-06 (UTC) by the census of B358.
In `synthetic-client.unit.test.ts`,
"arms the exchange deadline inside the slot,
not at dispatch",
each of two calls slept 150 ms against a deadline of 220 ms,
and the transport ignored its signal,
so the case passed whichever way the deadline was armed.
By the agent's control,
the old case still passed with the deadline firing first.

The fix:
the first call holds the one slot for 300 ms,
the second waits in the queue past its 100 ms deadline and answers at once,
and the transport refuses an exchange whose signal has aborted.
By the agent's controls,
the new case passes,
and fails when the deadline counts from dispatch.

Open to the owner's veto:
The case was reshaped to mend a check that could not fail,
in a file the agent was already editing.

Recurrence:
`mistake-prevention.md`,
"Tests on the real clock" and "Guards that cannot fail".

## B368: the republish case accepted either page

Pinned in `bb2a8b9ad1e164974e54d2750432a25b267ef0de`,
whose change is fixed in `cfac5b981`;
nothing at the red commit can fail the case,
since main's `readPinnedPair` already reads in input order,
so its red is the agent's mutant.

Found on 2026-10-06 (UTC) by the review of the runner observations change,
its second finding.
The pass republish case read the page name it expected from the output it was checking,
so it passed whichever page the `REPUBLISH LEFT` line named.
It now expects `page.md`,
the original page,
and its name says so.
By the agent's mutant,
which lists `page.en.md` first in `readPinnedPair`,
the case fails,
as does the case writing the missing page,
since the swap trades source and target;
the file was restored and checked unchanged with `git diff --quiet`.

Open to the owner's veto:
The `REPUBLISH LEFT` line keeps its shape and its `because=refused by <Class>`,
and now always names `page.md` when both pages fail;
the report names no alternative.

Recurrence:
`mistake-prevention.md`,
"Guards that cannot fail".

## B436: the repeat census printed its positive controls and exited 0 whatever they read

Red `c5a5e78991`,
fixed in `36ada23b21`.

`repeat-census` now prints its whole summary,
then ends at the stated refusal code naming each control that read false,
an absent control entry counting as failed (`failedControlsOf`,
with tests of its own).

## B626: the definitions that the refinement replies hand on had no case that depends on them

Found on 2026-10-15 by the review of n159
(n160,
finding Y8),
after the lead's notes had listed it as owed.

Outcome written on 2026-10-15.
Fixed by n161 with a case that is red only by guard removal,
and landed on 2026-10-15 in the tests `5fe3173cc5` and the code `277145a862`.

`src/refine-stage-replies.ts` hands the page's `definitions` to `restoreTypography`,
and the case in `src/refine-stage-replies.unit.test.ts` passed them to `resolveReply`
and asserted nothing that depends on them.
The author's account of n159 listed the pass-through as not guarded by a test,
and the cache account of `REFINE_CACHE_VERSION` 7 (B623) relies on it.
The reader sees nothing of this directly:
without the pass-through a shortcut reference whose definition stands in another slice
would be curled at the reference only,
the defect of B615 on the refinement path.

The lead ruled on 2026-10-15:
add the behaviour test.

Fixed by n161:
`src/refine-stage-replies-definitions.unit.test.ts` (new)
uses a shortcut reference `[dogs' bowl]` whose definition stands in another slice.
It passes on the build before the fix,
since the pass-through exists,
so its failure is shown by guard removal:
with `definitions,` removed from the call,
1 row passes and 2 fail.

## B674: 64 of 287 mutations of the guards of four rounds passed every test, and three change what the code does

Found after the split of 2026-10-18 (UTC) by the guard demonstrations of the rounds n168,
n169,
n171 and n173
(`w181`;
the tables are `w181-notes/report-tests.md` and `w181-notes/report-tables.md`,
which give the reason for each mutation).

Outcome written on 2026-10-19 (UTC).
The demonstrations ran 287 mutations of the guards.
223 failed at least one test and 64 still pass.

Three survivors change what the code does,
and no test sees it:

- `n169-locate-shown-definitions` changes the result on 266 of the 21,870 rows that the demonstration generated.
- `n171-B639-lf-last-digit` makes the reader refuse a bare label `12`.
- `n171-slot-apostrophes`,
  for which the notes name no input.

The other survivors fall in three kinds,
each with its reason in `report-tables.md`:
paths no test pins
(the log line of the run-off,
the key and stage of the run-off,
and recount checks that shadow each other),
equivalent mutants,
and unreachable branches.

The agent's Write of `report.md` was refused by the tool,
and its hand-back is the report (M261).

Status:
open.
The handover records that a test round (`w188`) pinned every reachable survivor in 15 commits that change tests only,
on `95a57f628d`,
and that they are not yet on the branch;
its notes land with that code and are not read here,
so this entry does not say whether each new case fails under its mutation.

Updated on 2026-10-19 (UTC):
the test round landed as 15 commits,
`4df68c685e` to `58b9d98583`,
and its notes are read in B684.
Each survivor it pinned has a case that its notes record as failing under the mutation,
and the three that change behaviour are pinned to what the TSDoc and headers of the code state.
The other survivors are equivalent mutants,
four unreachable branches that are now invariant errors (B683),
and two that the round could not pin.
The entry is closed for the pinned survivors,
the equivalent ones and the four branches,
and open for the two left unpinned.
B684 lists each.

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".

## B688: the fixture runner of the oxlint plugin tests dropped a plugin's throw, and cases passed

Found in the request memo round (`w190`),
where the spread-bridge pair `50ccee0440` and `5dce27bb59` was held:
two valid-fixture cases passed only because `runOxlintFixture` hid a plugin's throw (M275).

Outcome written on 2026-10-19 (UTC).
Oxlint reports a plugin that threw,
and a source it could not parse,
as a diagnostic with no rule code.
The runner in `package/oxlint-plugin/test-support/src/index.ts` kept only the diagnostics whose code starts with the
caller's prefix,
so that diagnostic was dropped,
and any fixture that expected no findings passed while the plugin crashed.
The autofix cases did not use the runner at all:
each ran oxlint with the fix flag itself,
with a catch that only checked that some error existed,
and read back a file that a crashed plugin would have left unchanged.

Fixed in lint mode by the redo of the spread bridge (`w194`):
red `04f54e0ca1`,
fixed in `90fa8c2d44`.
The runner throws and names the target and every message of such diagnostics.
Two cases of `run-oxlint-fixture.unit.test.ts` failed before the fix:
one for a plugin that throws on a valid source,
and one for a plugin that throws although another rule of it reported.
None fails after;
the two controls pass before and after.

Fixed in fix mode by the fix-mode fixture round (`w195`):
red `169ab8a678`,
fixed in `b6676c9a66`,
under the same rule.
`runOxlintFixture` gains a `fix` option and an `environment` option (`1c0fdc0f6e` and `48c87f8813`),
and every fix-mode spawn in the plugin tests moved onto it
(`5928802c78`,
`484741ba4e`,
`eac85debe9` and `e1debca690`,
in the packages `no-restricted-syntax`,
`tsdoc`,
`prefer-readonly-parameter-type` and `stylistic`).
The worktree range was `d16aaf67f7` to `483e74e462` on `729de4182d`;
the landing is `169ab8a678` to `20ab2696b9`,
pushed on 2026-10-19 (UTC) after gate g109.
The gate had 144 of 145 family steps green and a suite of 2532 cases with 0 failing.
Its one red step was the `lint:oxlint` task of the repair package,
which aborted in libuv's `init_once` during a host-wide thread-creation failure (`EAGAIN`) between 02:38 and 02:40 UTC,
and ran green again at `483e74e462`;
the notes say the main session recorded that incident in
`doc/troubleshooting/fedora-44-fold-emulator-user-task-ceiling.md`
in `174bddfeac` on `main`,
a commit that is not in the history of the checkout this entry was written in.

Not done:
the red of the cases before their move onto the runner was not run again.
Left open,
each filed as an issue:

- Issue 1777:
  the `check-alignment` findings of `tsdoc`.
- Issue 1778:
  the findings of `stylistic`.
- Issue 1779:
  the Synthetic quota shape that B671 leaves open.
- Issue 1780:
  the allow-list of the request memo of `w190`.

Gate g110 passed with the known lint findings of `tsdoc` (14) and of `stylistic` (2 and 1).

Recurrence:
`mistake-prevention-guards-searches-and-claims.md`,
"Guards that cannot fail".
