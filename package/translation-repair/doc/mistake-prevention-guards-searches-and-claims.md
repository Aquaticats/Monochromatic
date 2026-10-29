# Preventing mistakes: guards, searches and claims

Rules for evidence:
guards that must be able to fail,
searches and censuses that must reach what they look for,
and claims that carry their backing.

Part of [Preventing this package's mistakes](mistake-prevention.md),
which lists every file of this record.

## Guards that cannot fail

What happened:
red guards went red for the wrong reason (a missing export,
a lint warning) (M3,
M4,
M231);
guarded one branch of a new condition (M5);
asked a function that never reads the entry guarded (M29);
matched a word the message carries twice (M34);
used fixtures that never reached the sites they named (M42,
X18's first fixture,
and a lane comparison case whose fixture repeated both ledgers where its name repeats one,
so an earlier check refused it
(T8's fifth batch)),
among them a test committed for a defect it passed on under the build before the fix (M53),
or that could not tell the claimed order from another (M44);
and mutation runs could not report a catch,
or left out the test pinning the mutated token (M27,
M41),
or read a test run that crashed before any verdict as a survivor (M50);
a positive control addressed by line number changed nothing and read as a null (M52);
and a red commit said a case pins a check its fixture could not reach,
before any mutant had tested it (M58);
two copy-check cases passed with the copy check removed,
since their candidate was valid against the page they used (M62);
and a type-level proof that roster names are served ids outlived its purpose:
once the roster type was derived from the served ids,
the proof could never fail,
and its runtime case asserted a literal `true` (ledger B31);
and refusal cases checked only that a read threw,
or threw an error of the right class,
so a case refused by an earlier check than the one it names passed
(the naturalness review,
polish and census input tests,
tightened under T8);
and six artifact refusal reasons printed as something other than what the reader expected,
two of them the opposite,
behind cases that checked a fragment the wrong reading also held (B33);
and a refusal case rewritten to name its side swapped its class for the message,
since `toThrow` given a text checks only that the message contains it (M85);
and the slice coverage and delivery refusal cases checked the class alone or a fragment of the message,
though every check in each file throws one class carrying a structured fault
(T8's seventeenth batch);
and a guard-off build put a mutant in a file's second top-level suite beside one failing its first,
so the file stopped before the second suite and its case never ran (M89),
and a red case sat the same way behind red cases in the file's first suite,
so it could not be seen failing (M97);
and the attribution reader's cases for generations 2 and 3 wrote each record's index as generation 4 does,
so they agreed with a decoder that read one spelling for every generation,
while 42 stored artifacts those generations wrote were refused (ledger B107);
and cases another contributor added asserted a prefix of a damaged heading (ledger B133),
an empty result that holds with or without the check named (ledger B130),
a comparison over a fixture with nothing to compare (ledger B131),
or the wrong result as the intended one (ledger B128,
B139).

More on 2026-10-05:
a scripted client no prober ever heard,
whose cases asserted no findings (ledger B143);
footnote cases whose expected text was the fixed function's own output (ledger B146);
a coverage control that held with no decoy taken (ledger B150),
and called a cut that left nothing evidence it could not locate (ledger B149);
and a store guard that checked a settlement's fields one by one and never two that come together (ledger B151).
A scripted client is shown heard by each stage it serves before a case built on it counts,
and an expectation for a parse-dependent result is written off the printed tree.

Trial cases named a guard no input can tell from its absence,
reached an invariant throw through a cast,
and asserted an answer alone under a name for two knobs (ledger B178).
A case is shown to fail with the thing it names taken out of the build,
before its name is believed.

The consolidate store still resumed a gate whose choice its own ballots do not give,
and flags no writer sets beside their terminal (ledger B187).
Two reading-cache cases built records that a closing check refused whatever field they named (ledger B188),
and trial cases in seven files were refused before the line they named was read,
or answered a status that never reached the throw they named (ledger B211).
A refusal case hands in a record with one cause,
and resumes the same record with that cause mended.

Trial cases in twenty more files asserted what no change to the code under test could move:
a default compared with its own expression,
a check for `undefined` on a value never undefined,
a sheet searched for a sentence its run is never given,
a seed whose shuffle agrees with arrival order and with alphabetical order,
a scripted verdict outside the wire's vocabulary,
and a note never shown absent where its count is zero (ledger B225,
B226).
A case that reads the environment sets it and clears it,
and a note or a finding is shown absent beside the input that raises it.

The consolidate store was first driven against the real stage on 2026-10-06 (UTC),
and resumed a gate that heard fewer voices than its quorum,
a settlement the stage returns and the persistence rule never writes (ledger B249).
Cases in six more files could not fail on what they named:
a refusal checked by its class alone,
an input that could not tell one guard from another,
a length read where the whole operation was meant (ledger B256).
And an agent's strip of its shell's environment,
built by filtering the printed environment for key names,
removed nothing,
since the filter printed the matched text and never a name,
so a built command ran with the run's keys (ledger M120).
A guard is shown to remove,
refuse or fail on the thing it is for,
on a real instance of that thing,
before anything is run behind it.

Agents' cases pinned how many texts a scan passes over a generated family (60,
36 and 36)
and how many spellings a grammar accepts (106),
and one compared a list's length with itself (ledger M137).
A count read off a run moves with the fixture and says nothing of the property it stood for,
and the owner's rule is to avoid magic-number tests.
A case asserts the property itself:
the passed texts are exactly those the stated rule skips,
or both verdicts occur and every reader agrees with the grammar.

A damage case checked only that its text held the borrowed sentence,
which a sentence set anywhere also satisfies (ledger B331).
Three guards no input can tell from their absence stood with nothing at their site to say so:
one had a record saying every case passes without it,
and the case that named it deleted (ledger B178);
the other two were found by a review of the trial's cases (ledger B335).
The bench-draw cases held three `toContain` assertions where each whole value was known (ledger B348).
A case on arming a deadline inside a queue's slot passed whichever way the deadline was armed,
since its transport ignored the signal the deadline aborts (ledger B362):
a scripted transport honours the signal a case means to see fire.
The pass republish case read the page it expected from the output it was checking,
so it passed whichever page the line named;
it now names the original page,
and a mutant reading the English page first fails it (ledger B368).
A case never takes its expectation from the value under test.

The rule:
a red guard is read case by case before the fix,
and each case must fail for the reason its name gives;
after the fix every case turns green.
A refusal case checks the message the refusal carries,
the path and reason for an artifact read,
beside the error class,
and a copy with one expected message changed must fail.
`toThrow` checks a class or a text,
never both,
so a refusal is caught with `caught`,
then checked with `toBeInstanceOf` and with `caughtValueText` whole,
and a rewritten assertion is compared with the one it replaces.
Where the class carries a structured fault,
the case asserts the fault whole beside the class,
and one case per sentence checks the message.
An `ArtifactParseError` reason is printed after "expected",
so it names what the reader wanted there,
never what it found or a sentence about the record,
and its case reads the printed message whole.
A test added for a defect runs against the build before the fix and must fail there before the fix is committed.
A guard over several sites asserts that each site is reached,
one assertion per site,
and a hook under test hands a different answer each call.
An order or a precedence claim is tested with a fixture mixing every source it draws from.
A fixture standing for a stored record of one generation is spelled as that generation's writer spelled it
(`artifact-key-vocabulary.ts` names each renamed key),
never in the reader's own names,
and a reader of a renamed key gets its spelling from the vocabulary.
Every mutation run opens with a control that must survive,
and lists every test file naming the mutated token (`rg` the tests first);
a survivor is a guard defect until a test that can fail is added and the mutant is caught.
A run that ends without a verdict (a crash,
a signal,
a timeout) is a result of its own,
never a pass,
and a harness carries one control for each outcome it reports.
A control edit is addressed by the text it changes,
and its change is counted before the measurement runs.
A message says a case pins a branch only once the mutant removing that branch is caught;
until then it says what the case fails on today,
and a case that must pass a precondition to reach its branch
is read against that precondition in the code first.
A case guarding a short circuit is run once with the short circuit removed,
and must fail there,
before its commit says what it guards.
Every test file awaits one root `describe` at its top,
its suites as children run one at a time (ledger B99),
since a failing suite awaited at the top ends its file
and every suite after it goes unrun and unreported;
`multi-suite-top-level.unit.test.ts` holds each file to it.
A mutant whose case prints no FAIL line is still read as surviving
only once that case's suite printed its pass line,
and a red case counts as red only from its own FAIL line.
A case over a pass that rewrites text asserts the whole text it returns,
and a case named for a check feeds the check an input only that check decides.
When one side of a compared pair becomes derived from the other,
every guard comparing them is re-read:
one that can no longer fail goes,
and its record names the case that still carries the invariant.
A guard whose mutant no input can catch is an equivalent mutant:
no case is written for it,
and it stays or goes by what it does for the code
(a narrowing the type check needs,
a plain word for an edge);
one that stays carries a comment at its site saying why no case can separate it,
so a reader of a later mutation run finds the reason beside the survivor,
not in a ledger entry.
When a rule changes what a property means,
the oracle that checks the property is rewritten,
and shown to detect a planted violation,
before any grid is run with it:
the oracle of the sixth review's grids counted a proven apostrophe as a loose mark,
and would have flagged correct carries (ledger M179).
The author of a change never edits or reads the oracle that judges it:
n140 changed its copies of the oracle and of the final check to its new rule,
and its grids read zero on pages that an oracle written from the rule text alone failed:
3,555 single-edit pages of one grid and 1,092 of another,
861 of them for a doubled mark (ledger M194,
B535).
A changed rule gets an oracle written by someone else from the rule text,
which imports nothing from the code under test,
reads no body of it,
and is shown on hand cases that fail first before a grid is run with it (ledger M196).
An independent oracle is written from the rule text alone,
and the reviewers of a rerun read its outputs and never its code:
the tenth review did not open the oracle,
and decided the four pages the oracle still failed from the flanking rules of the format
and the parse of a reference implementation,
which showed the oracle wrong and production right (ledger M194,
M196,
B549).
A guard demonstration states a reason for every mutation that still passes:
of 287 mutations of the guards of four rounds,
64 passed every test,
and three of those changed what the code does (ledger B674).
Each survivor is classed as a missing test,
an equivalent mutant or an unreachable branch;
a missing test is written and shown to fail under its mutation.
A fixture for an ordering is shown to give opposite answers under the two orders before the test is committed:
a pin of the code point order used a character of the basic plane at a lower code point than the surrogates,
which sorts the same in UTF-16 order,
and a second commit replaced it (ledger M268).
A mutation spec is applied to the base and shown to match exactly once before a proof relies on it,
and a spec whose find string or file has moved is rewritten and kept with the proofs (ledger M269).
These two rules were written by the filer of M268 and M269 on 2026-10-19 (UTC),
and the notes did not give them.
A fixture runner fails on a diagnostic with no rule code,
which is how oxlint reports a plugin that threw:
the runner of the oxlint plugin tests kept only diagnostics with the caller's prefix,
and every case that expected no findings passed while its plugin crashed (ledger B688 and M275).
That holds in lint mode since `90fa8c2d44` and in fix mode since `b6676c9a66` (the notes give this rule),
and a case that runs oxlint on its own,
with a catch that only checks that some error exists,
is moved onto the runner.

What enforces it:
the mutation harness in each fix's record,
run with a control;
the reach assertions in the preparation,
seam and evidence guards.

## Searches and censuses that miss

What happened:
a search capped with `head` read as complete (M33);
a pin searched by a whole clause missed a rewrapped one (M13);
a census counted a helper's fields by name,
not by contract (M24);
a teardown's audit listed only the sites it happened to see (M35);
a key was censused by its builders' names rather than by the material it hashes (M39);
raw NUL bytes made a source file binary to every line search (M40);
a lint and a probe ran over an empty or self-matching list (M45);
a census of a letter test searched the names of functions holding it and missed every inline copy,
and the search by shape then missed the same test written negated (M48);
a search for an entry id printed corpus text (M36);
the coverage census recorded each cold stretch under its first character's source,
dropping every module a stretch ran on into,
and a port control reproducing an earlier census's totals passed because that census had the same flaw (M67);
the coverage census measured a compressed build,
where guards folded into logical expressions had no range and read as run,
after the ledger had inferred from the stretched spans alone that its error ran only toward cold (M79);
a census of number reads counted calls and missed readers handed on as values (`.map(Number)`) and `Date.parse`,
six reads in all (M90);
a source scan read a `satisfies` as typing a table with string keys,
and missed a parameter destructured from an object until moving code changed its count (M98);
a scan of map writes read every two-argument `set` as one,
a typed array's copy among them,
and code was bent into a loop to pass it (ledger B342);
and a search whose pattern `rg` refused printed its `|| echo` fallback,
which read as finding nothing (M99).

A fix was committed after a run of 35 named test files that left out the one file asserting the old wording,
and the suite was red at that commit (ledger M122).
A count of 26 files matching a pattern was taken as proof a form was in use,
with no match read and no lint run,
and the linter warns on the form (ledger M124).
A probe for a byte order mark matched three code points and not three bytes,
found nothing,
and its positive control used another pattern (ledger M128).
And a census over named paths counted a log once for every path that reached it,
which moved the percentile a cap rule reads (ledger B288).
A named run after a wording change holds every test file that holds the old words,
found by a search for the words;
precedent is a match that was read,
never a count;
a positive control uses the probe's own pattern and tool;
and a census collapses its inputs to distinct files before it counts.
A census of combinator members that log read only function bodies written inline in the call's arguments,
so a member passed by name stayed out of its sight,
and the order of log lines was checked only at the two sites it fixed (ledger B388);
a census names the forms it cannot see beside its count.

The rule:
a search whose result licenses a change runs uncapped over all of `src` (or `--count` first),
then narrows.
A census of a rule searches every way the rule can be written,
not the names of functions known to hold it:
for a character test,
range comparisons on characters and on codes,
literal alphabets,
case-fold comparisons,
and regex classes and properties,
each shape in its asserted and its negated spelling.
A census of a behaviour also lists every way the language reaches it
(a call,
a construction,
a value handed on,
a coercion)
and every function with the same behaviour on another type,
and its guard says which it reads and which it leaves out of reach.
A source scan lists every way the language declares the shape it looks for
(an annotation,
a cast,
a `satisfies`,
an alias,
a parameter named,
destructured or defaulted),
says for each whether it gives the shape,
and holds one of each in its fixture;
a count that moves when code moves without changing what it does marks a gap in the scan.
A scan reading a method call tells the receiver's kind from the declaration the receiver's name resolves to,
never from the method's name alone,
and a finding that would bend correct code to pass it is read as a defect of the scan first.
A null result counts only after a positive control shows the search can match,
and a fallback report names the exit status (`|| echo "exit $?"`),
since `rg` exits 1 for no match and 2 for a refused pattern or another error.
A measurement that aggregates is checked against an invariant any correct output meets,
not only against an earlier tool's totals,
since two tools sharing a flaw agree:
the coverage census refuses a report in which an uncalled function's first line sits in no stretch of its own source.
A measurement reads code as its source writes it:
the coverage build is not minified,
and a claim that a transform's error runs only one way is tested by measuring once without the transform.
A generated list's line count is printed before anything consumes it.
A probe's output goes outside the tree it searches.
Corpus page text may be printed since 2026-10-05 (UTC),
when the owner ruled,
"I've allowed you to handle PII.
The corpus is public."
Until then a probe printed ids,
indices,
counts,
code points and markup alone (M36).
The lead's reading of the ruling,
open to the owner's veto:
a probe need not mask corpus page text,
and a doc may quote it where the quotation helps;
credentials and raw provider requests and replies stay private,
a run log's lines of model reasoning among them;
the corpus stays out of the repository,
since it is unlicensed;
unit fixtures stay invented;
and a session transcript is still neither searched nor quoted,
since it holds more than the corpus.

A scan's package-wide case compares what it found through a helper of
`package/translation-repair/test-fixture/src/scan-findings.test-fixture.ts`
(`expectNoFindings` for a list that should be empty,
`expectFindingsAsListed` for a list that should equal an inventory,
`expectRecordAsListed` for a record),
never through `toEqual` or `toStrictEqual` alone:
the assertion library prints a value longer than forty characters cut to its length
(`expected [ …(31) ] to deeply equal []`,
issue 610),
so a failing scan named nothing it found until its walk ran again in a probe.
Each helper writes every difference whole on a line of its own,
then makes the comparison the scan made.

A change that makes an outcome more frequent starts with a search that lists every consumer of that outcome,
and each consumer is re-read:
a test that was harmless while `unavailable` was rare
withheld work from carriable places once the mark rule made it common
(ledger M170).

A corpus count sizes a cost and never proves a shape absent.
A rule's scope was set to "the paragraph" from the unit of a count that found no quotation across paragraphs,
and eight small pages the seventh review built lost a quotation mark (ledger M181).
The scope of a rule is decided from what the language allows,
and each shape a count shows as zero gets a hand-built input before the rule leans on its absence.

A random search of 1,799,159 pages and quotes found no place whose words read another way with a page's definition,
and a hand-built page found one at the first try (ledger B673).
The sample rarely held an opener,
a reference and a closer in the order the case needs,
so the null said nothing about reach.
A fixture is also read as the span the counter locates and not as the text it was written as:
`*a [b*] c*’` differs with a definition,
and the counter locates `a [b*] c*’` after the opener,
which does not.
The search reports its own positive control,
a case known to be found,
before a null is written down,
and a fixture is run through the counter before a test leans on it.

A link checker parses Markdown and not text:
one reported an escaped bracket in quoted output as a dead link,
and the line held no link (ledger M255).
A finding of a checker is read in its file before it is reported or fixed.

A null over stored runs is taken only from artifacts that can carry the field under test:
the first check for the finalist extension searched the pooled runs directory,
found 164 legacy artifacts with no consistency record,
and missed that hand-picked passes write to a throwaway folder of their own (ledger M258).
At least one searched artifact is shown to carry the field,
and the places where hand-picked passes write are listed as searched or not searched.

What enforces it:
habit,
the control-byte scan in the checklist,
the coverage census's placement refusal and baseline format check (`requirePlacedFunctions`,
`CENSUS_FORMAT`),
and the census's refusal of a minified build (`requireUnminifiedBuild`),
before the suite runs.
`package/translation-repair/source-scan/src/scan-direct-comparisons.unit.test.ts`,
among the source scans,
fails on `toEqual` or `toStrictEqual` called on `expect(...)` in a case of a scan the `source-scans` task names
where the case reads the package's files,
and on a scan none of whose cases reads them.

## Claims without their evidence

What happened:
ledger numbers written from a summary or a comment instead of the log or the code (M9,
M22);
docs naming a log line the code never writes (M20);
a count put to the owner that measured something narrower than the option it backed (M18);
refusals called damage from a category name (M31);
a fix that supplies a model "missing" context built before reading the sheet (M32);
an inferred cause written as fact (X18's first draft);
a commit said its cases covered every branch of the coverage census,
and the census,
once run,
found ten cold stretches in four of those modules (M66);
a batch's baseline reading printed all zeros for a source its baseline never loaded,
which reads as nothing left to do (T8,
fixed in `297c72fd5`);
a message said a census recorded which bundle carried a source,
which `census.json` does not record (M76);
a baseline reading printed "ran 13" over sources edited since the baseline,
whose old stretches it matched against lines now holding other code (T8,
fixed in `d27a89dd0`);
the reading counted nowhere the code a run left cold that its baseline ran,
so a change that left code cold read clean (B61);
and it asked git about edits inside the package alone,
though the census reads other packages' sources by line too (B62);
it left code no source map places out of "cold since then" and still counted it as not loaded,
so the same six stretches in a renamed bundle read as not loaded at every census,
and each batch record explained them by hand (B105);
a veto-open call refused an exponent in a provider's price on the ground that the fixtures write none,
and a stored catalogue writes one (M91).

A field's only observable was said to be a line the code discards,
and the line was logged all along,
read by a test in another file (ledger B179).
A claim that nothing can observe a value is measured,
by looking for a test that already observes it.

Every count of `[PASS]` lines this audit quoted was a count of suites that print,
and 44 cases in ten files sat in a suite with an empty name,
which the test module logs at `debug`,
so they passed without a line (ledger B277).
The same count cannot show a failure either:
a suite with failing cases still prints its pass line (ledger M125).
Report commands claimed what their own figures deny:
that no call went to the metered provider where every metered call sat on an unpriced seat,
a zero total under a line saying silence is not a zero,
one note for three different states,
and a block left out with no word that it was (ledger B280,
B286,
B289).
A comparator's comment stated an order its code did not keep,
with no case for either tier (ledger B304).
A notes file for agents described three fixtures before they were read (ledger M129),
and a commit message repeated an agent's account of code the lead had not read (ledger M131).
A claim of absence is printed only in the state that proves it,
a branch that skips part of a report says so,
a comment that states an order has a case for each tier it names,
and an account its writer has not checked is labelled as whose it is.
A generator's comment said every suffix followed every prefix where its code paired each with one,
and a module comment named two of the three readers it served (ledger M137);
a comment that states what code covers is read against the code.
Four agents' reports were kept only in the session and had to be extracted from its transcript later (ledger M136);
a report is written to a file when it arrives.
A red commit's message was first drafted saying every failing case was a new one,
where an existing case failed too,
and a note tied a bound to another entry's list before checking which bound that entry meant (ledger M139);
a message describing a run is written from the run's log line by line,
and a link between two records is checked before it is written,
or written as unverified.
A standing summary counted the digests whose rounds were judged,
so a digest whose rounds drew no ballot was counted nowhere (ledger B372):
every population a filter drops is counted on a line.
The coverage census refused a suite that failed,
and counted one that exited clean having printed no PASS marker,
so a suite that ran no test gave a census claiming no test ran any line of the package (ledger B384):
a suite with no pass marker is refused,
not counted.
Five TSDoc paragraphs of the settled audit named `main` as their function's only caller,
where the caller is `runSettledAudit`,
and one named a reader of `--cap` the package does not define (ledger B389);
a doc naming a caller or a reader is read against the code each time that code moves.
The runs lock warned that a claim's staged text was left after the claim whenever its removal failed,
though a runs directory too long for the staged name refuses the write and the removal with the same error,
so the warning named a file left behind that was never written (ledger B390);
a message says a file stands only where its write is known to have succeeded,
and otherwise says what failed and on what condition the file stands.

The rule:
every number,
log line,
cause and quoted rule is read from its source in the same step it is written,
and the source is named beside it.
A claim that tests cover a module's branches comes from a census of the claimed sources
(`mise run coverage-census -- <test files>`),
each loaded and holding no stretch in a census of format 2 or later (M67),
never from reading the tests.
A claimed source the baseline holds no stretch in is proven by this run's rows alone:
the baseline reading names each one,
and one the baseline never loaded must be loaded now with no cold stretch left.
So is a claimed source edited since the baseline's commit,
which the reading names and leaves out of its counts;
a baseline is taken from a tree with nothing uncommitted under the package,
or the reading refuses it.
No stretch may read as cold since then,
code the baseline ran that this run left cold.
A comparison of two runs reads each side for what the other cannot say,
and a question put to git covers everything its answer is read against,
read NUL-separated.
A kind of record a reading leaves out of one count it leaves out of every count it prints,
and a figure a batch record keeps explaining by hand is a reading defect to fix,
not to explain.
A claim drawn from a census names the `census.json` field or the report line that holds it.
A fix that gives a model context starts by rendering the sheet and searching it for that context.
An inference is labelled as one,
or traced in the code before it is written.
A claim that no code does a thing is a search's result:
the search runs before the claim is drafted,
since a commit message drafted first said no production code compares an artifact's commit with the corpus pin,
and the search then found the pin stamped into artifacts and compared in three files (ledger M138).
A claim about what a writer writes is measured over that writer's stored output,
with a control the measurement must flag;
the package's fixtures are its own copies,
not the writer's output.
A check's log counts only with its exit status and its own verdict line:
a docs commit was recorded as Markdown-lint clean from a log holding only the logger's sink timeout,
and no line from the linter (ledger M151).
A cost is sized by counting every shape that reaches the code path,
never by the example that prompted the question:
a span rule's cost was sized by its one dropped opening when plural possessives,
years and `'em` reach the same code (ledger M168).
A sentence of the form "so the result pairs X the way both places did" is backed by a property probe
over a grid that includes markup and punctuation edges before it is written in a header (ledger M169).
A limit is accepted only with a probe that shows what a reader of the page sees,
and the entry that records the acceptance names the probe:
a mix of straight and curly apostrophes was accepted from the author's description,
and the seventh review counted it on 6,239 of 61,846 replaces of its grid (ledger M180).
A limit is also read against the text of the decision it departs from before it is recorded:
a mark beside a CJK character was accepted as trailing against a decision that says it reads as loose,
and the entry that recorded it had no probe either (ledger M200,
B520).
A replay cited as evidence for a reader prints the thing the reader read:
the replay of the stored run printed no count of consistency groups
until its line ended `consistency-groups=2 read-at-generation=15`
(ledger M172).
A page read that says a rule holds names the lines it read:
the read of TianqiChen66623 recorded that the tense held from the narration,
and her last message,
quoted in her own words,
had been moved into the past on the same page (ledger M220).
Quoted speech is read against the source and the human archive line by line,
apart from the narration.
A read states the outcome of a stage from the record the artifact keeps of that stage,
never from one round's log line:
the read of TianqiChen66625 said the run-off of the refrain reached no quorum
from the "no quorum" line of its first window,
and the `runOff` of the artifact held a win of 3 of 4 after a retry (ledger M226).
A count that a log line prints is described only after the code that computes it is read:
the 1.3 million "delivered chars" of a cut stream were the raw stream with its framing,
about 27 times its reasoning (ledger M226).
A word such as "most" is a count,
and it is counted (ledger M226).
A listing is given a cause only after the type of each entry in it is read:
unreadable entries called mode-locked test files were dangling symbolic links (ledger M224).
A round said every caller of the text fold receives the page's definitions,
and three readers of emphasis still parsed without them,
two of them one step behind the caller the brief named (ledger B668).
A claim that every caller of a function hands it a value is made after the call sites are listed with `rg` and read,
and a parameter that defaults to empty is where a caller leaves it out unseen.
A cache version's account said no record had been written since it was set,
and a pass wrote three under it three weeks later (ledger B670).
An account of a cache version names the date and the search that backs "no record since",
and says what retires a stored record besides the number.
A figure in an agent's hand-back is recounted from the raw files the hand-back names before it is recorded:
a summary said 6,751 shared hits where the agent's own counter files total 6,628 (ledger M257).
A claim that no file of a generation is stored is made after the run directories are searched
for a pass that is writing now (a held `pass.lock`),
and it names the date and time of the search:
the round of the sol review called generation 18 open while a pass was writing files of it,
and the recount it had made stricter would have refused one (ledger M265).
A change that makes a reader stricter is gated by a generation,
and does not rely on a directory standing still (ledger B682).
This rule was written by the filer of M265 on 2026-10-19 (UTC),
and the notes did not give it.
A premise in a brief about a count or about the size of a text is recounted
from the log it cites before the brief is sent,
or the brief marks it as unchecked and the agent measures it first:
the report behind the seat round said the cut replies were counted as usable and had no answer text,
and the log showed neither (ledger M270).
A log line quoted in a report is copied from the source or from the test that pins it,
and a test that pins a line pins its exact text:
a report quoted the credential line with a space before a comma that the source lacks (ledger M271).
These two rules were written by the filer of M270 and M271 on 2026-10-19 (UTC),
and the notes did not give them.
Two lint logs are compared by finding,
not by their summary lines:
a package was said to have "the same findings" when it had three shared ones
and type-aware warnings from dependencies that were not built (ledger M276).
A fact a run's evidence states,
such as a work's English title its lookup found,
is checked against the sources that evidence cites before it is called wrong;
one store's silence is not the absence of an edition (ledger M279).
Text in a page is called invented only after the whole quoted passage is read in the original,
not the line the quote began on:
"so her enemies could never take her" was removed from a comment as invented,
from line 9 of the original alone,
and line 10 of the original is where she said it (ledger M283).
The reading names the lines it covered,
and a passage that runs on is read to its end.

What enforces it:
habit;
the census reading refuses a baseline taken with uncommitted changes,
names every claimed source edited since the baseline apart from its counts,
names every stretch cold since the baseline,
leaves unmapped code out of every status it reads (`coverage-census-report.unit.test.ts`),
and asks git about every file of the work tree;
`rendered-sheets.test-fixture.ts` renders every sheet so reading one is a call away.
`package/translation-repair/source-scan/src/silent-test-cases.unit.test.ts`,
among the source scans,
fails on a case whose nearest suite has an empty name or none,
so every passing case prints a line (ledger B277).

## Guards a census wants gone

What happened:
a contributor was told that every stretch of code no unit test runs ends cased,
removed as unreachable,
or fixed as a defect,
and to take the coverage census again until it read zero.
A guard on a state no input produces can never run,
so it reads as a stretch for ever,
and the rule left no place for it.
Guards came out,
five of them leaving a sentinel,
a wrong answer or an endless loop where the guard had stood;
named errors became `nonNullishOrThrow`,
which says only that a value was missing;
and production functions gained parameters whose one use was a case forcing the guarded state
(ledger M112,
M113).
One removal changed a result for an input the signature admits
(ledger B127):
the guard had also been covering sizes that were not whole numbers.
One conversion of a fallback into `nonNullishOrThrow` rested on a proof that named one of two line markers,
and a log line of the other ended the cap census (ledger B132).
The census itself was the pressure:
it counted an `unreachable:` throw no test drives as code no test ran,
for ever (ledger M115).
A reader that skipped a step naming a block its side lacks,
on the belief that no walk names one,
grouped such a walk into nothing,
and a pairing read off a stored artifact could build one,
since the check on a model's reply had vouched for another list (ledger B138).

The trial's other production edits were read against their callers on 2026-10-05:
a refusal given an own `cause` of `undefined` and a mistyped field reported as missing (ledger B155),
a catch widened to wrap a fault of the process as a corpus refusal (ledger B156),
and a check cut in half so that another would run,
which moved a heading beside broken definitions to `unparseable` (ledger B157).

Four cases were added to cover a branch of the persistence guard that no pass reaches,
each building its artifact by cast,
and one of them pinned an artifact nothing had reviewed passing unrefused (ledger B168).

A walk reader checked each step's blocks and never that every block had a step,
so a walk leaving one unnamed grouped its section to nothing (ledger B172).

Two named invariant throws of the rendering audit's corroboration had been flattened
to `nonNullishOrThrow` (ledger B176).
A fallback on a list read at another list's index was turned into the same unnamed refusal,
which refused only some of the inputs the fallback had covered (ledger B177).
When a fallback becomes a refusal,
the inputs the fallback covered are refused at the function's entry,
in words.

Two arms answered without a word for a state no input produces:
a lane selection that was never contested read as nothing to report (ledger B199),
and a parsed list item with no offsets dropped its whole list (ledger B200).
Each is an `unreachable:` throw now.

Two more answered for such a state,
each behind a case that pinned the answer:
the width contest's `none` for a composite whose text is neither seat's,
the same word it gives where no candidate won (ledger B223),
and the refine boundary's verdict for a paragraph outside the body zone,
which its case reached through a cast (ledger B224).
No verdict is kept for a fixture production cannot build.
Fourteen chain tails then wrote their invariant as a call to a helper that throws,
a shape the census reads as code no test ran,
until the helper became a class named for the invariant and thrown at the chain's end (ledger B231).

The sibling arm B199 left,
a consolidation recorded as not run read as nothing to report,
is an `unreachable:` throw too (ledger B250).
The coverage clusters of 2026-10-06 (UTC) then read every cold fallback the whole-suite census listed,
and made loud or total each default for a state its caller rules out,
twenty by the three commits' own counts,
where no input reaches the state (ledger T8).
Three fallbacks could be reached after all:
by a percentile no rank belongs to,
which the signature admits and no caller passes (ledger B272);
by a file whose own words read as a position (ledger B274);
and by a call cut at its stream bound on every provider,
under a comment calling that end unreachable (ledger B275).
A fallback is called unreachable only after every way into it is read,
a comment that says so included.

The clusters of the final census of 2026-10-10 (UTC) made seven more arms throw:
four that answered without a word for a state no input produces,
and three paths that rethrew a value nothing they call throws,
each now wrapped with the caught value as its cause (ledger T8).
The brief for those clusters said a silent arm becomes a throw,
and not that a loud refusal a caller already handles stays,
so one agent turned six `FootnoteRewriteError` refusals into plain `unreachable:` errors.
The relabel pass,
the only production catch of that class,
turns the refusal into a withheld relabel that keeps the archive,
while a plain error passes the catch and stops the entry;
the lead restored the refusals at the merge (ledger M143).
A census brief names the refusal classes a caller handles and the callers that handle them.

The rule:
a state no input produces is refused out loud,
with `throw new Error('unreachable: <what was found and why it cannot occur>',)`
or the invariant error class its module already has,
and that throw stays whether or not a test can reach it.
A cold stretch that is nothing but such a throw is finished work,
never a reason to remove the guard,
flatten its message or add a way in for a test.
Before a guard comes out because its state cannot occur,
say what follows it if the state occurs anyway:
where the answer is a sentinel used as a value,
a slice from the wrong place,
a wrong boolean or a loop that does not end,
the guard becomes the throw.
That is for an arm that answers without a word.
A loud refusal of a class a caller turns into a withheld result stays as it is,
cold or not,
and its stretch is recorded as kept,
as the six `FootnoteRewriteError` refusals the relabel pass withholds on are (ledger M143).
Turning such a refusal into a throw changes what a run does when the proof is wrong,
a stopped entry where a withheld result shipped,
so it is the lead's decision,
made against the owner's rule that a run always ships,
and never an agent's on its own proof.
It may come out only where the operation after it refuses the same state by itself,
in words that name it.
When a guard does come out,
list every input it turned away,
the ones outside the proof included,
and refuse those at the function's entry.
`nonNullishOrThrow` is for an element read that never had words of its own,
never a replacement for a message that named what broke.
A parameter that lets a case force a value the function derives for itself is not a seam
(the rule for seams is under "Tests touching the real world").
A fallback over text read from a file is dead only when every line the reader accepts carries what the fallback covers,
and the proof names each reader that hands lines on.
An index read off a stored record is checked against the list it indexes where the record becomes steps,
since the reader of the reply it was copied from vouched for another list.

The census counts an invariant throw apart from cold code:
a cold stretch that is nothing but `throw new Error('unreachable: ...',)`,
or a throw of a class whose name ends in `InvariantError`,
with only braces,
`else` and its guarding `if` head beside it,
is listed under the invariant throws,
and no row,
total or baseline reading counts it.
Its census file is format 3;
a format 2 baseline is refused by name and taken again,
since it holds those throws among the cold code with nothing to tell them by.
An invariant written another way
(a call beside the throw,
a helper called to throw for its caller,
a message joined with `+`,
`nonNullishOrThrow`)
stays cold or goes unseen,
so the shapes the count reads are the shapes an invariant is written in.

What enforces it:
review of every removed guard against this list,
and the lead's review at the merge of every refusal an agent turned into a throw,
both habit.
`sample-draw.unit.test.ts` holds
"REFUSES a size that is not a whole number of slots",
the one case here an honest input reaches.
`coverage-invariant-throw.unit.test.ts` holds the shapes the census reads as an invariant throw
and the ones it keeps cold,
and `coverage-census-input.unit.test.ts` the refusal of a format 2 baseline.
`pair-blocks-steps.unit.test.ts` holds the refusal of a pairing naming a block its side lacks.

## Modules without a test of their own

What happened:
a census of the package's 1,086 production modules found 417 with no `<stem>.unit.test.ts` beside them;
117 of those are barrels,
modules of types and literal constants,
or runner entries,
and 300 are logic that another module's test reaches or no test names at all (ledger B102).
A module tested only through its importers loses that coverage when an importer changes,
and nothing stopped a new one from joining them.
The same held inside a module:
three private helpers of the mark readings (`readProof`,
`isPartOfAWord` and `bringsAtEdge`) were reached only through exported callers,
and no case named their branches (ledger B519,
M187).

The rule:
a new production module ships with `<stem>.unit.test.ts` beside it,
unless it is a barrel,
holds only types and literal constants,
or is a runner entry `package/translation-repair/coverage-census/src/build-entries.ts` lists.
A module on the list leaves it when it gains a test.
A new helper with branches of its own is exported from its module marked `@internal`,
reaches its tests through `built-artifact-barrel.ts`
(the lint rule for the imports of tests asks for the built entry,
and rule XPT of `AGENTS.md` allows it;
ledger B519,
M190),
and is tested directly branch by branch;
the hand-back lists any helper without such a test.

What enforces it:
`own-unit-tests.unit.test.ts`,
among the source scans,
for modules;
for helpers,
habit and the hand-back's list.

## Lists that must name every member of a type

What happened:
`KNOWN_VERDICTS` listed the verdicts a pre-grade file may carry,
checked with `satisfies readonly GradeVerdict[]`.
The union gained `duplicate` six days after the list was written.
The clause checks that each listed member exists and not that every member is listed,
so nothing failed,
and a file carrying the new verdict was refused whole (ledger B169).
A census of the package's other lists,
lookup tables and branch chains over a union found none behind its union,
and gave each that must be complete a form the compiler holds (ledger B227).
The chains' closing refusal was first a helper called to throw,
which the coverage census listed as untested code,
and is now a thrown class (ledger B231).

The rule:
a value meant to name every member of a union is keyed by the union
(a record whose key type is the union),
or the union is derived from the value,
or the members are read by a `switch` that ends in a `never` assertion,
so the compiler refuses a member the value lacks.
A list that is a deliberate subset says in its TSDoc which members it leaves out and why.
An array typed or checked as `readonly Union[]` proves membership only.

What enforces it:
the compiler,
for `KNOWN_VERDICTS` and for each list,
record and chain ledger B227 converted;
a chain ends in `throw new UnhandledMemberInvariantError(…)`,
whose `never` parameter refuses a member no branch names.
No scan finds a list typed by a union declared apart from it,
so a new one is held by review,
and the lists that census read and left unconverted are named in ledger B227.

## What a failing check prints

What happened:
the first red run of two new scans printed `expected [ …(31) ] to deeply equal []` and `[ …(13) ]`,
the count of their findings and none of the findings,
and reading them took a second run of each file outside the runner (ledger B338).
The assertion library under `@monochromatic-dev/test-harness-core` is chai 6.3.0,
which cuts any value longer than 40 characters to its length in a failure's message
(`config.truncateThreshold`),
and the test package neither sets that nor exports it (issue 610).
The same display stood in ledger B70,
quoting a failure of its guard,
recorded and not acted on (ledger M138).
A guard written for it then found 64 comparisons in 41 scans
that compared what a walk over the package found through the library alone,
where the lead's census had counted 31 scans (ledger B339).

The rule:
a check that fails over a list of findings fails listing them,
each on a line of its own,
before any comparison the library would cut.
A scan over the package compares what it found through the helper of its shape
(`scan-findings.test-fixture.ts`):
`expectNoFindings` for a list that should be empty,
`expectFindingsAsListed` for a list against an inventory,
and `expectRecordAsListed` for a record,
each printing every difference whole and then making the scan's own comparison.
A diagnostic seen to hide what it reports is fixed when it is first seen,
or recorded as open with what it hides.
A harness that reads the output of a tool names the target,
the config,
the command and both streams when that output cannot be read as expected:
the runner of the oxlint plugin tests let a bare `SyntaxError` escape when oxlint printed a failure text (ledger B690).
A failure that a tool reports only on standard error,
with exit code zero,
is turned into a failing run that names the file and the way out:
a bridge that could not start left degraded findings and one stderr line (ledger B689).
This rule was written by the filer of B689 and B690 on 2026-10-19 (UTC),
and the notes did not give it.

What enforces it:
`scan-direct-comparisons.unit.test.ts`,
among the source scans,
fails on a case of a listed scan that reads the package's files
and compares through `toEqual` or `toStrictEqual` alone,
and on a scan none of whose cases reads them;
its header names what it cannot reach (ledger B339).
`scan-findings.unit.test.ts` holds each helper's whole failure text.
Issue 610 holds the cause in the test package,
so a comparison outside a scan can still print a count where what differs runs past 40 characters.

## A bound is read against the deadline it sits under

What happened:
the two windows of the stream guard were 600000 ms beside a per-call deadline of 360000 ms,
so for three runs neither could fire,
and the artifact stamped both numbers without anything reading them together.
The guard also counted any body bytes as progress,
so a comment line that OpenRouter sends while it waits kept a silent stream open until the straggler grace
ended eleven of them (ledger B804).

The rule:
a bound that ends a call is checked,
where both numbers are read,
against the deadline that would pre-empt it,
and the configuration is refused when the bound is not shorter,
or is not a positive finite number.
The check throws when the configuration is built,
naming each setting,
its value,
the deadline and both ways out.
Progress for such a bound is the output the bound is for:
a content or a reasoning delta,
never bytes of any kind.
A test hands the guard a comment line,
a blank line and an empty delta,
and reads the cut.
The bound's value is set from the largest observation of a healthy call,
with the sample's censoring stated,
and the percentile is named only when the cost of a cut is stated beside it.

What enforces it:
`stream-bound-config.unit.test.ts` refuses the pair that shipped and each unusable number.
`run-call-config-builder.unit.test.ts` drives `runCallConfigOf`.
`stream-content-progress.unit.test.ts` holds the three silent shapes as cuts
and a content and a reasoning delta as progress.
