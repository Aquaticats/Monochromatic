# Audit ledger

Every finding of the whole-package audit the owner asked for on 2026-09-27
("audit the whole translation-repair pkg for all the mistakes we've made and fix all of them",
"Mistakes made,
ever,
for this pkg,
not just today").
Each finding carries where it is,
the evidence,
the fix,
the guard,
and its status.
A finding leaves this ledger only when its fix and guard have landed,
and its lesson then lives in the prevention doc.

Probe scripts and their outputs stay outside the repository,
under `~/temp/agent/audit-<area>/`,
because they read the pinned corpus,
which is never committed.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Audit ledger: glossaries and Canadian forms](audit-ledger-glossaries-and-canadian-forms.md):
  Every glossary entry and Canadian-forms pass measured against the corpus,
  with the mutation check and the re-probe of the glossaries.
- [Audit ledger: rule text,
  the consolidation gate and the floor replay](audit-ledger-rule-text-gate-and-floor-replay.md):
  Contradictions and gaps in the rule text every sheet carries,
  the consolidation gate finding,
  the replay of every floor over the archives and settled pages,
  and the declared-name floor that read a swap of declared names as a loss,
  with the blends its first fix passed (B699).
- [Audit ledger: history of the numbered defect classes](audit-ledger-defect-class-history.md):
  The defect classes found run by run before the audit,
  classes 1 to 185 and the ones before numbering.
- [Audit ledger: defects in the tests themselves](audit-ledger-test-suite-defects-in-the-tests-themselves.md):
  The test-suite findings T1 to T7 and T9,
  covering an assertion that never runs,
  vacuous checks,
  tests pinning wrong or retired behaviour,
  corpus text and real personal data in fixtures,
  flaky timing,
  names claiming more than they check,
  magic numbers,
  and the log every test run writes into `node_modules/.monochromatic/`.
- [Audit ledger: code no unit test runs, method,
  first censuses and early batches](audit-ledger-test-suite-unrun-code-method-first-censuses-and-early-batches.md):
  How the coverage of code no unit test runs was first measured on 2026-09-28 and 2026-09-29,
  the rule for when T8 closes and the census task that proves reach,
  the first whole-suite censuses,
  and the first eight batches of cases written against them,
  ending with the page-assembly rounds error that was decided rather than tested.
- [Audit ledger: code no unit test runs,
  stretch queue batches one to ten](audit-ledger-test-suite-unrun-code-stretch-queue-batches-one-to-ten.md):
  The stretch queue,
  batches one to ten,
  each read against a whole-suite census,
  from the census placement module to the `corpus-run` title modules,
  including the triage of the queue and the finding that the compressed build hid guards.
- [Audit ledger: code no unit test runs, stretch
  queue batches eleven to sixteen](audit-ledger-test-suite-unrun-code-stretch-queue-batches-eleven-to-sixteen.md):
  The stretch queue,
  batches eleven to sixteen,
  each read against a whole-suite census,
  from the alignment modules to the pairing modules.
- [Audit ledger: code no unit test runs, stretch queue batches
  seventeen to twenty-four](audit-ledger-test-suite-unrun-code-stretch-queue-batches-seventeen-to-twenty-four.md):
  The stretch queue,
  batches seventeen to twenty-four,
  each read against a whole-suite census,
  from the slice modules to the page modules.
- [Audit ledger: code no unit test runs, agent clusters
  and bundles before the trial](audit-ledger-test-suite-unrun-code-agent-clusters-and-bundles-before-the-trial.md):
  The clusters of the census cut and given to agents reading one commit,
  what review changed in each patch,
  and the nineteen bundles whose agents stopped at a usage limit,
  up to the hand-over to the trial model.
- [Audit ledger: code no unit test runs, trial model
  clusters from `group` to `coverage`](audit-ledger-test-suite-unrun-code-trial-clusters-group-through-coverage.md):
  The trial model's first thirteen clusters of the fresh whole-suite census,
  from `group` to `coverage`,
  each of eight to ten stretches.
- [Audit ledger: code no unit test runs, trial model clusters
  from `markup` to `corpus-run/editor`](audit-ledger-test-suite-unrun-code-trial-clusters-markup-through-editor.md):
  The trial model's clusters fourteen to forty of the fresh whole-suite census,
  from `markup` to `corpus-run/editor`,
  each of five to seven stretches except the last.
- [Audit ledger: code no unit test runs, trial model clusters from `corpus-run/roster`
  to `corpus-run/publish`](audit-ledger-test-suite-unrun-code-trial-clusters-roster-through-publish.md):
  The trial model's clusters forty-one to sixty-nine of the fresh whole-suite census,
  from `corpus-run/roster` to `corpus-run/publish`,
  each of three or four stretches.
- [Audit ledger: code no unit test runs,
  re-verification of 2026-10-05](audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md):
  The correction of 2026-10-05 of eight groups the trial model had blamed on the census,
  the verified facts about the census tools,
  the method adopted from then on,
  and the re-verification of the sites left open.
- [Audit ledger: code no unit test runs, census at `65bd3a645`
  and the runner entry files](audit-ledger-test-suite-unrun-code-census-at-65bd3a645-and-runner-entry-files.md):
  The whole-suite census of 2026-10-06 and the library clusters cut from it,
  the quote-loss refusal kept cold on purpose,
  and the runner entry files moved into modules,
  with what stayed open at the close of that passage.
- [Audit ledger: code no unit test runs,
  final census and what stays cold](audit-ledger-test-suite-unrun-code-final-census-and-what-stays-cold.md):
  The final whole-suite census of 2026-10-10,
  the two clusters cut from it,
  the entry tails round,
  the last census,
  and the stretches that stay cold with the reason for each.
- [Audit ledger: page assembly and the corpus-run driver](audit-ledger-page-assembly-and-corpus-run-driver.md):
  Findings in the page-assembly passes and the driver that runs a corpus pass,
  the archive's notices that a page is unfinished and the translators' notes that decided a contest (B698),
  an archive block revision that dropped the note its block held (B702),
  notes put back into a CRLF text with bare line feeds (B706),
  a page shipped without 30 slices of the original and nothing on it to say so (B827),
  the footnote check on the page that was relative to the archive (B828),
  a refinement rolled back with no line in the log (B829),
  alignment findings that reached the artifact and not the log (B830),
  and standings kept without contest endorsement that no `TALLY` field counted (B831).
- [Audit ledger: the page grammar](audit-ledger-source-text-under-a-note-and-the-page-grammar.md):
  The original of an untranslated passage written as literal text from a CommonMark parse,
  a page the grammar refuses with the original on it,
  a page shipped without the runs no writing makes parse and how a page on disk is judged (B832),
  and the corrections it made to A8,
  B827,
  B830
  and B831.
- [Audit ledger: the source text under a note,
  follow-ups on parser gaps and step searches](audit-ledger-source-text-follow-ups-parser-gaps-and-step-searches.md):
  The holes between the plain parse and the strict grammar that followed B832,
  the limits of that follow-up closed and the shapes a second search found,
  the wiring tests of the page on disk,
  the sixth round after the re-check of round 4,
  and the seventh round,
  the module lines from the tokenizer and the footnote reading (B832 follow-ups).
- [Audit ledger: the source text under a note, the ninth round of follow-ups](audit-ledger-source-text-follow-ups-lone-carriage-returns-and-the-full-pass.md):
  Lone carriage returns as line endings,
  the sampled live-construct check over a wider pool at one pair in four,
  the full pass over every pair and text,
  the walk fixture's hook in the grammar,
  and the limits and mutants of the round (B832 follow-ups).
- [Audit ledger: the source text under a note, the reading comparison and the real corpus](audit-ledger-source-text-follow-ups-the-reading-comparison-and-the-real-corpus.md):
  The page check's comparison of the reading of the page with the reading of the source,
  the count of real texts per residual class,
  the fixes of the classes and the checks of the fourth round (B832 follow-ups).
- [Audit ledger: the source text under a note, the block comparison and the openers in code](audit-ledger-source-text-follow-ups-the-block-comparison-and-the-openers-in-code.md):
  The comparison decided for each top-level block,
  the defects for a block it cannot decide and a run it cannot compare,
  the comments that never close taken out of code and math,
  the code spans an indent-four footnote start cuts,
  and the pins,
  mutants,
  search and real-corpus counts of the fifth round (B832 follow-ups).
- [Audit ledger: the source text under a note, tables, emphasis and raw HTML after a paragraph](audit-ledger-source-text-follow-ups-tables-emphasis-and-raw-html-after-a-paragraph.md):
  The footnote start indented four columns after a table row,
  in emphasis and link text and after a backslash hard break,
  the comment openers that cut a paragraph or stand in a footnote's fence,
  the block of raw HTML that cuts a paragraph,
  the pins,
  mutants,
  differentials,
  search and real-corpus counts of the answer to the third re-check (B832 follow-ups).
- [Audit ledger: the source text under a note, the plain reading with tables and the containers](audit-ledger-source-text-follow-ups-the-plain-reading-with-tables-and-the-containers.md):
  A link target decoded as far as it reads,
  the plain reading with GFM tables and strikethrough,
  the footnote start after a link reference definition,
  every opener that starts a block of raw HTML,
  the fence,
  the display math and the openers under a footnote in a block quote or a list item,
  the lazy lines and the raw HTML a footnote opens with,
  the placing of the block comparison,
  and the reds,
  mutants,
  differentials,
  search,
  real-corpus counts and residual classes of the answer to the fourth re-check (B832 follow-ups).
- [Audit ledger: the source text under a note, the footnote readings, round 8](audit-ledger-source-text-follow-ups-the-footnote-readings-round-eight.md):
  The image alternative text with an autolink or a code span,
  the list marker after indented code,
  indented display math after a line of a paragraph,
  the footnote that opens with a tag alone,
  the fence mark indented four columns,
  the twenty-nine minimal texts of the containers re-check,
  the pin of the tab path,
  the placing of the block comparison in the time of the places that matter,
  and the search,
  differentials,
  real-corpus counts,
  gates and limits of the round (B970 to B979).
- [Audit ledger: providers, routing and seating](audit-ledger-providers-routing-and-seating.md):
  Findings in the provider clients,
  the router and the seating of models.
- [Audit ledger: DeepSeek off-peak pricing](audit-ledger-deepseek-off-peak-pricing.md):
  The cost of a DeepSeek call booked at the rate of the hours of its start and its completion,
  where it had been booked at peak whatever the hour,
  what that figure bounds (B920),
  and the wording that still described a spanning rule the code does not have (B921).
- [Audit ledger: the repair lane](audit-ledger-repair-lane.md):
  Findings in the repair lane,
  L1 to L13 and L15.
- [Audit ledger: the repair lane's smaller items](audit-ledger-repair-lane-smaller-items.md):
  The smaller repair-lane findings collected as L14,
  each under its own letter.
- [Audit ledger: docs and comments that disagreed with the code](audit-ledger-docs-against-code.md):
  Statements in the docs,
  comments and examples that the code or the record contradicted,
  among them a function the ledger still named after its removal (D36)
  and comments of the XingZ6018 fixes that said more than the code or the rulings (D37).
- [Audit ledger: doc conventions and their guards](audit-ledger-doc-conventions-and-their-guards.md):
  Conventions the docs broke across many files and the guards written for each:
  clock times with no zone,
  task-list numbers,
  the Markdown linter,
  links that lead nowhere,
  and references by position.
- [Audit ledger: the
  Markdown linter's gate and the split of long docs](audit-ledger-markdown-linter-gate-and-the-split-of-long-docs.md):
  The gate that runs the Markdown linter over the family's docs (D35),
  the older docs it reads through name stems,
  and the docs filed by subject or by a span of dates to stay within the line limit.
- [Audit ledger: found while fixing,
  stages, seats and pairing keys](audit-ledger-found-while-fixing-stages-seats-and-pairing-keys.md):
  Defects met while fixing another finding,
  in how the stages stop,
  seat and re-seat,
  and in what the slice and pairing keys carry.
- [Audit ledger: found while fixing,
  tests, source text and live defaults](audit-ledger-found-while-fixing-tests-source-text-and-live-defaults.md):
  Defects met while fixing another finding,
  in the tests and fixtures,
  in what source files say,
  and in parameters that fell back to the live transport or the corpus clone.
- [Audit ledger: found while fixing,
  names, headings and math on the page](audit-ledger-found-while-fixing-page-names-headings-and-math.md):
  Defects met while fixing another finding,
  in how a page's names,
  HTML headings and math are read.
- [Audit ledger: copies of shared code in production](audit-ledger-copies-in-production-code.md):
  Readers,
  predicates and helpers the production code kept in more than one copy,
  each copy free to drift from the others,
  among them the pinned lint binary version beside the one `mise.lock` installs (B700),
  the toolchain cutover that broke its reader and the gate's lookup of the lint program (B980),
  and the move of that pin to 0.4.0 with the refusal text brought up to the toolchain lock (B1015).
- [Audit ledger: copies of shared code in tests and scans](audit-ledger-copies-in-tests-and-scans.md):
  Helper bodies the test files and the source scans kept in copies,
  and the scans that could not see a copy.
- [Audit ledger: code nothing reaches](audit-ledger-unreferenced-code.md):
  Imports,
  functions,
  values,
  modes and exits that no production path reaches,
  and the checks that did not report them.
- [Audit ledger: comments, TSDoc and source text](audit-ledger-comments-tsdoc-and-source-text.md):
  Defects in what the source says about itself:
  TSDoc that documents nothing or the wrong thing,
  comments citing what does not exist,
  characters in the source a reader cannot see,
  header sentences that the code did not make true,
  and the direction of the evidence in the header of the span agreement (B681).
- [Audit ledger: text by code point, and words inside words](audit-ledger-code-points-and-word-boundaries.md):
  Text read by UTF-16 unit or ordered by locale where code points were meant,
  fixed words found inside longer words,
  and a quote counted inside a longer word that ends at a joint (B675).
- [Audit ledger: folds and quote forms](audit-ledger-folds-and-quote-forms.md):
  Comparisons that used the wrong fold for their question:
  straight against curly quotes,
  layout-only differences,
  case folds,
  answers checked before they were folded,
  and an archive review quote of a screenshot refused for the picture's own line breaks,
  its dropped reply left unlogged (B715).
- [Audit ledger: text that shows nothing](audit-ledger-text-that-shows-nothing.md):
  Text a reader sees as blank that passed for wording,
  because the check asked `trim()` or asked nothing.
- [Audit ledger: defaults that stand in for an input](audit-ledger-defaults-standing-in-for-an-input.md):
  Places where a missing,
  malformed or unreadable input was replaced by a default,
  and the run went on as though the input had said so,
  among them a block with no source section reviewed against an empty one in silence (B717).
- [Audit ledger: labels and counts in messages](audit-ledger-labels-and-counts-in-messages.md):
  Messages that named the wrong thing or stated a count wrongly:
  labels that did not fit,
  counts before a fixed plural,
  one field carrying two kinds of text,
  a log line that reported nothing done and gave no reason,
  a log line that said a changed slice shipped unchanged (B733),
  and a run-off line that called a plurality a tie (B806).
- [Audit ledger: values read back from text](audit-ledger-values-read-back-from-text.md):
  Numbers,
  fields and keys the code wrote into text and then read back more leniently,
  or by a rule the writer does not keep.
- [Audit ledger: runner reports and probes](audit-ledger-runner-reports-and-probes.md):
  Report and probe commands that printed a claim their own figures deny,
  or bought work nothing could use,
  a picture reader measurement whose next step named the sample when the trusted records had limited the count (B800),
  the coverage rounds over an untranslated tail,
  examined and kept (B811),
  and the pass's per-entry lines that carried no clock time and no attempt number (B960).
- [Audit ledger: the pass's lifecycle line stamps (B961 to B964)](audit-ledger-lifecycle-line-stamps-round-two.md):
  The findings of the re-checks of B960:
  a hand-edited attempt count that ended the pass at every launch (B961),
  three tests that could not fail (B962),
  the pass's own lines and the logger's lines that carried no time or no attempt,
  and the cap wording (B963),
  and docs that claimed what the code did not (B964).
- [Audit ledger: the cap census and the Xiaomi cap](audit-ledger-the-cap-census-and-the-xiaomi-cap.md):
  A card that still named the pooled 99th percentile after the cap rule could read its calls (B990),
  and a census that named every dangling link under the agent root and buried its own summary (B991).
- [Audit ledger: NeuralWatt's day start is never waited for](audit-ledger-neuralwatt-day-start-never-waited-for.md):
  The wait for NeuralWatt's day to start again and the park of the pass between entries,
  removed on the owner's answer of 2026-10-29 (B1005),
  and the interim page a mid-page dry stop ships on his second answer of the same day (B1006 to B1008).
- [Audit ledger: footnote markers and link destinations](audit-ledger-footnote-markers-and-link-destinations.md):
  Scans for footnote markers and link destinations that disagreed with the parse,
  about where a marker is a reference or where an address ends,
  and a definition reorder that carried a note under the last definition to another (B705).
- [Audit ledger: tags, nesting and paired marks](audit-ledger-tags-nesting-and-marks.md):
  Readers of tags,
  attributes,
  paired marks,
  front matter and nesting depth,
  each reading the structure its own way.
- [Audit ledger: outcome rules and stage records](audit-ledger-outcome-rules-and-stage-records.md):
  Rules that decide what a slice ships and what a stage records:
  exits an outcome rule could not reach,
  records that dropped a round,
  work no floor could check,
  a consolidation under the vote minimum that kept the archive wording two judges called ineligible (B696),
  two archive block revisions of disjoint lines that could not both ship (B712),
  the review stage's findings that its caller dropped (B714),
  and a polish gate tie that kept the base and said nothing (B730).
- [Audit ledger: composed operations and checks on a part](audit-ledger-composed-operations-and-partial-checks.md):
  One refusal inside a composed operation that stopped the whole,
  or left no finding,
  checks that vouched for a whole text on one part of it,
  and a composite of archive revisions that refused any revision whose line count differed (B803).
- [Audit ledger: test cases that cannot fail](audit-ledger-test-cases-that-cannot-fail.md):
  Cases and guards that would pass with the behaviour they name removed,
  a pass-through that no case depended on,
  the mutations of four rounds' guards that no test fails,
  and a fixture runner of the oxlint plugin tests that dropped a plugin's throw (B688).
- [Audit ledger: survivors of
  the guard demonstrations and the tests that pin them](audit-ledger-guard-demonstration-survivors-and-their-pins.md):
  The mutations of four rounds' guards that every test passed,
  the test round that pinned the reachable ones,
  the equivalent mutants,
  the two survivors that stay unpinned (B684),
  a producer-list search that read a builder in few shapes (B861),
  the readings of that search that no case held (B862),
  the rebuild of that search over each file's syntax tree (B863),
  and the check over the family's real sources that names each shape the search leaves unread (B864).
- [Audit ledger: the check over the schema sources as a whitelist](audit-ledger-schema-source-check-as-a-whitelist.md):
  The check of B864 rebuilt from a list of seven named shapes into a whitelist anchored at the module-load boundary,
  with two totals,
  eight shapes,
  and a prevention rule that replaces the one of B864 (B865),
  then a mutant called equivalent by an argument (B866),
  and a third total for where a builder's result goes (B867).
- [Audit ledger: the check over the schema sources, code the search cannot read and
  the mutants that survived](audit-ledger-schema-source-check-unreadable-code-and-surviving-mutants.md):
  Code the search cannot read and specifiers the check cannot resolve (B868),
  and four surviving mutants with the cases they called for (B869).
  It continues B865 to B867.
- [Audit ledger: the check over the schema sources, the named place, the carriers, the callbacks and
  the loaders](audit-ledger-schema-source-check-named-place-carriers-callbacks-and-loaders.md):
  A named place that holds the module of the function it names,
  the namespace of a module that declares forwarders,
  the parameters of a function written as an argument,
  the loader of modules and the binding of Node read where they are no direct call,
  the key readers of a constructor,
  and the limits and over-reports each leaves (B953).
  It continues B865 to B869.
- [Audit ledger: files a later run reads back](audit-ledger-files-a-later-run-reads-back.md):
  Caches,
  ledgers,
  locks and stored stage results that a later run read back,
  torn,
  stale or holding what no writer writes,
  and a reader that throws where a changed locator no longer finds the slot.
- [Audit ledger: tests touching the real world](audit-ledger-tests-touching-the-real-world.md):
  Tests that reached the real corpus,
  real keys,
  the file system,
  child processes or process globals.
- [Audit ledger: clocks and durations](audit-ledger-clocks-and-durations.md):
  Durations measured on the system clock,
  test bounds on the real clock a loaded machine could outrun,
  the archive block review round that waited the readers' window for writer voices (B713),
  and the held review seat that landed before quorum under load (B813).
- [Audit ledger: output decided by arrival order](audit-ledger-output-decided-by-arrival-order.md):
  Findings,
  refusals and log lines whose content or order depended on which concurrent work ended first,
  the scan that looks for it,
  and the seat-window family,
  tests that named a seat the rotated first round no longer asked (B701),
  among them the composite cases of the archive block review (B734)
  and the run-off line cases of the translate stage (B812).
- [Audit ledger: failures under a caller's stop](audit-ledger-failures-under-a-callers-stop.md):
  Fan-outs that dropped,
  mislabelled or doubled a member's own failure once the caller had stopped the work.
- [Audit ledger: loops that copy what they built](audit-ledger-loops-that-copy-what-they-built.md):
  Folds,
  maps and scans that copied or re-read everything built so far at every step.
- [Audit ledger: building the tense check](audit-ledger-tense-check.md):
  The record of building the tense candidates:
  what the review of the tense records and sheets found loose,
  what was fixed,
  the rule that quoted words keep their speaker's tense,
  the two findings of the review of n162 about that rule:
  quotations in single curly marks and two sentences that could be read against it,
  and the quotation mask that still joined marks of different pairs,
  with the three cache versions that moved for it (B676).
- [Audit ledger: building the consistency pass,
  repeat groups and the carry](audit-ledger-consistency-pass-repeat-groups-and-the-carry.md):
  The record of building the page-level consistency pass up to the independent review of its rebuild:
  how the repeated wording is located,
  how the bench chooses and the pass carries the choice to every place,
  and what the first reviews found.
- [Audit ledger: the consistency pass,
  the span and the marks beside it](audit-ledger-consistency-pass-span-and-marks-beside-it.md):
  The fourth and fifth independent reviews of the pass:
  what the located span covers,
  which single marks beside it travel with it or refuse a splice,
  and the joint rules that keep a stand from sitting inside a word.
- [Audit ledger: the consistency pass,
  apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md):
  The sixth independent review and the apostrophe round that followed it:
  which apostrophes are attached to a word,
  the apostrophes that need no guess and travel with their word,
  and the blocks a splice carries in part.
- [Audit ledger: the consistency pass, the page-wide proof,
  forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md):
  The seventh independent review and the fixes that answered it:
  the proof that an edge mark is an apostrophe,
  read over the whole page,
  the joints after closing marks,
  the form of an arriving apostrophe,
  the floors that keep a writer from changing words that must stay,
  and the limits the lead accepted.
- [Audit ledger: the consistency pass, stack depth, the
  floors fixed point and the page form](audit-ledger-consistency-pass-stack-depth-floors-fixed-point-and-page-form.md):
  The rounds that built the whole-page reading:
  the parser's refusal at the edge of the stack,
  the floors and the gain check settled together,
  and the page that an arriving apostrophe takes its form from.
- [Audit ledger: the consistency pass,
  one reading of every mark](audit-ledger-consistency-pass-one-reading-of-every-mark.md):
  The independent grid check and the eighth review's findings that the first fix round closed:
  one meaning for a mark in the counter,
  the locator,
  the proof and the readers of form and of marks inside words,
  and the two limits that round left accepted.
- [Audit ledger: the consistency pass, the fixes of the eighth
  review and the oracle residue](audit-ledger-consistency-pass-fixes-of-the-eighth-review-and-the-oracle-residue.md):
  The findings of the eighth review that the second fix round took in hand,
  the flags of the first that the lead decided to fix after the merge,
  and the residue of the oracle's rerun that the next round fixed,
  each with the outcome of its fix.
- [Audit ledger: the consistency pass, parse boundaries,
  the letter test and the proof's page](audit-ledger-consistency-pass-parse-boundaries-letter-test-and-proof-page.md):
  The defects of the second fix round and the rounds after it:
  the stack exhaustion read as a refusal wherever it came from,
  the letter test widened for every caller,
  the reason a left-out gain states,
  and the proof's page that was a reconstruction.
- [Audit ledger: the consistency pass, the ninth review](audit-ledger-consistency-pass-ninth-review.md):
  The ninth independent review and its fixes:
  the glyphs that ship,
  the stack edge,
  the largest set of re-admitted gains,
  the double marks `„` and `‟`,
  the mark after closing emphasis,
  the cost,
  and the comments.
- [Audit ledger: the consistency pass,
  the independent oracle and the rules for marks](audit-ledger-consistency-pass-independent-oracle-and-mark-rules.md):
  What an oracle written from the rule text found after the fixes,
  the nine rules for marks the lead decided in answer,
  and the limits accepted with them.
- [Audit ledger: the consistency pass, the tenth review](audit-ledger-consistency-pass-tenth-review.md):
  The tenth independent review and the fixes that answered it:
  the gain blocked by a mark that reads free,
  the second mark carried past a footnote reference,
  a tag,
  a link or a space,
  the readers that knew four marks and eight,
  and the leading elisions.
- [Audit ledger: the consistency pass,
  the re-review and the refusal run](audit-ledger-consistency-pass-re-review-and-the-refusal-run.md):
  The independent re-review of those fixes and the round that answered it:
  the refusal run stated by what renders between two marks,
  with references,
  escapes,
  spaces and line breaks,
  the elisions that open a quotation,
  the limits accepted,
  and the container markers at the start of a line.
- [Audit ledger: the consistency pass,
  what the refusal run steps over](audit-ledger-consistency-pass-what-the-refusal-run-steps-over.md):
  What the reviews after that round found in the refusal run:
  the page-wide refusal for a reference that is a mark,
  the invisible characters,
  controls and separators it steps over,
  what it reads as a blank line,
  and the lines of invisible characters that stay inside a paragraph.
- [Audit ledger: the consistency pass,
  span edges as whole units](audit-ledger-consistency-pass-span-edges-as-whole-units.md):
  Span edges that cut a character reference or a backslash escape,
  took in text that only looks like a reference,
  or left half a word,
  and the units the edges are made of now,
  with the shown-word test and the word characters of any script that n159 landed.
- [Audit ledger: the consistency pass,
  what the word test reads as shown](audit-ledger-consistency-pass-what-the-word-test-reads-as-shown.md):
  What the review of n159 found the word test read wrongly,
  all fixed by n161:
  hidden stretches read unit by unit,
  markup that separates or annotates read as invisible,
  and inline code beside a text place,
  with the rows of the twin check that n166,
  n168,
  n169 and n173 fixed and that landed on 2026-10-18,
  and the padded inline code,
  the raw elements and the label order that the review of n162 and n168 added (B633,
  B634 and B638),
  inline code padded by line endings (B677),
  and two review findings about link labels that did not reproduce (B678).
- [Audit ledger: the consistency pass,
  elisions, quoted numbers and pairing](audit-ledger-consistency-pass-elisions-quoted-numbers-and-pairing.md):
  The pairing of single marks on a curly page:
  elisions inside a quotation,
  the quoted letter,
  nested quotations,
  opening marks beside markup and quoted numbers,
  with the tie rule and the shown-text pairing that n159 landed.
- [Audit ledger: the consistency pass,
  the prose mask and slices parsed alone](audit-ledger-consistency-pass-the-prose-mask-and-slices-parsed-alone.md):
  Typography only on the prose that the page parse shows,
  the cache versions that moved with it and with the quoted-tense rule,
  and the readers that parse a slice alone,
  with the page's definitions,
  the fragment landing and the shared memory of the masks that n159 landed.
- [Audit ledger: the consistency pass, definitions inside containers,
  fragment landings and the mask memory](audit-ledger-consistency-pass-definitions-landings-and-the-mask-memory.md):
  What the review of n159 found in those readers:
  the definitions inside containers that the shared reader still misses (open),
  a fragment landing outside its slice,
  and a text that could read another text's mask,
  the last two fixed by n161,
  with the landing that took offsets which are not whole numbers (B635)
  and the callers of the fold that read without the page's definitions (B647).
- [Audit ledger: the consistency pass,
  voices lost and places left unlocated](audit-ledger-consistency-pass-voices-lost-and-places-left-unlocated.md):
  What the run `TianqiChen66625` lost and n170 found,
  and n171 fixed:
  replies refused for the way they named a label,
  a run-off whose result the log never stated,
  a carry choice that split and was not run off,
  a chosen place that no quote located,
  an abandonment line that counted the wire,
  and the two defects that n171 found in its own span agreement.
- [Audit ledger: the consistency pass, page definitions recorded
  and carries gated by generation](audit-ledger-consistency-pass-page-definitions-recorded-and-carries-gated.md):
  What the review of B656 to B664 left open:
  the page definitions a slot needs,
  which artifact generation 18 records,
  the carry run-off that generation 18 requires,
  a refusal text that named the wrong quotes,
  and generation 19,
  which gates the stricter recount of containment and inline code (B682).
- [Audit ledger: follow-ups on emphasis definitions, cache accounts
  and the Synthetic quota shape](audit-ledger-follow-ups-emphasis-definitions-cache-accounts-and-synthetic-quota.md):
  The emphasis readers that parsed without the page's definitions (B668),
  the contradictions in the TSDoc of the consistency cache version (B669),
  the account of the pairing cache version from `d614a0c1d` to `29d606470` (B670),
  the Synthetic quota body the meter reader refused on 2026-10-18 (B671,
  seen again for a whole pass),
  a clock-zone slip in that account (B672),
  the request to read the words of a place with recorded definitions,
  which reads the page the other way and was not built (B673),
  the judge seats line that printed `wet` for a meter that said nothing (B686),
  Synthetic's removed five-hour window and concurrency limit (B691),
  and the cache-account audit that read a version set in a merge resolution as in no commit (B814).
- [Audit ledger: the package split,
  configs, scans and messages](audit-ledger-package-split-configs-scans-and-messages.md):
  What the split of the package into 36 found in the moved code and its tooling:
  a build config that lacked a plugin,
  a task that depended on a missing build,
  exports of files that do not exist,
  a scan that read nothing after the move,
  refusals that named a task that no longer exists,
  audit queries that covered only the new paths,
  workspace cycles made by tests,
  and a lint ignore that matched a package.
- [Audit ledger:
  the polish gate's claims and the order of reason and vote](audit-ledger-polish-gate-claims-and-reason-order.md):
  Sheets that withheld a claim the judges needed:
  the repair panel's accepted claims that never reached the polish gate (B731),
  the ballots whose reply shape asked for the choice before the reason (B732),
  sixteen answer schemas whose declared key order is not the order Hyper returns (B762),
  and the same claims,
  which the polish writers were never shown,
  nor the dispute note,
  nor the judges of the writers' rewrites (B805).
- [Audit ledger: what each sheet shows](audit-ledger-what-each-sheet-shows.md):
  Sheets that lacked a block their role needs to judge or write,
  the reader-protection rule that reached past the account of how a death or an attempt happened (B697),
  the cut of the translators' notes that left a bare quote line,
  joined two lines and dropped a carriage return (B703),
  the absolute naturalness review that showed its reviewers the notes (B704),
  a Han name the archive spells inside its sentence that no sheet or floor carried (B707),
  the judges,
  critics,
  auditors and pairers that saw the notes between translators (B708),
  and the composite archive candidate that reached the judges with no assessment (B716).
- [Audit ledger: readers that accept what no writer writes](audit-ledger-readers-accept-shapes-writers-never-write.md):
  Readers of stored records and directories that accepted a shape,
  a name or a value their own writer never produces,
  and a carry recount that never read a run-off beside a bench that was not asked (B680).
- [Audit ledger: references to titles](audit-ledger-title-references.md):
  How a title named in running text is recognized,
  glossed and rewritten.
- [Audit
  ledger: lists that must name every member of a type](audit-ledger-lists-that-must-name-every-member-of-a-type.md):
  Lists and switches meant to cover every member of a type that could silently miss one.
- [Audit ledger: block pairings that do not fit their text](audit-ledger-block-pairings-that-do-not-fit-their-text.md):
  Pairings of source and target blocks that a later step applied to text they did not fit.
- [Audit ledger: the environment handed to child processes](audit-ledger-child-process-environment.md):
  What a child process inherits:
  keys,
  settings and git's own variables,
  in production code and in tests.
- [Audit ledger: credentials masked at the transport](audit-ledger-credentials-masked-at-the-transport.md):
  Credential spellings that could reach a log line,
  a refusal or a listing unmasked.
- [Audit ledger: provider streams, usage counts and retries](audit-ledger-provider-streams-usage-and-retries.md):
  Provider fields read otherwise than the provider documents them:
  stream events,
  usage counts,
  and the refusals a retry decision reads,
  with the stream guards whose windows could not fire and whose progress was any body bytes (B804),
  the rounds that waited five minutes on a stream that sent only comment lines (B809),
  and the seat abandoned again and again,
  where the evidence rejected a hold (B810).
- [Audit ledger: NeuralWatt replies cut at the default effort and
  the picture energy estimate](audit-ledger-neuralwatt-replies-cut-at-the-default-effort-and-the-energy-estimate.md):
  Calls to NeuralWatt that ran at the provider's default reasoning effort and were cut at the length cap:
  the picture reader's reply the length limit cut (B718),
  a picture call sent at the provider's default effort (B719),
  the energy estimate that counted a picture as base64 text (B720),
  a text call sent at the provider's default effort (B807),
  the energy hold that read the pipeline's own use at a third of what a rotation of entries measured (B808),
  and the hold that read the GLM seat at fourteen times its use after the lift to low effort (B836).
- [Audit ledger: branches no input reaches,
  kept as quiet answers](audit-ledger-unreachable-branches-kept-as-quiet-answers.md):
  Branches no input reaches that answered quietly where a loud refusal belongs,
  most met when a coverage census asked for them to be removed,
  the four branches of the markup ranges that left a node out for want of a position (B683),
  a decision-seat branch only a test reaches (B824),
  and a re-ask catch the run-ending refusal cannot reach (B825).
- [Audit ledger: the source scans and the coverage census](audit-ledger-source-scans-and-coverage-census.md):
  What the source scans and the coverage census counted wrongly,
  could not see,
  or printed too little of when they failed,
  modules tested only through their callers,
  and tests of blank text that the scan for them did not list.
- [Audit ledger: ambient state touched by library code](audit-ledger-ambient-state-touched-by-library-code.md):
  Library code that read or wrote the process environment,
  the exit code or a module-load default,
  where a caller could not supply or see it,
  and a lookup kept under the identity of a list (B679).
- [Audit ledger: work merged from parallel briefs](audit-ledger-work-merged-from-parallel-briefs.md):
  What went wrong when two pieces of work written apart met in the same files.
- [Audit ledger: which seats a quorum counts](audit-ledger-which-seats-a-quorum-counts.md):
  Quorums that counted seats which never answered,
  could not answer,
  or were withheld,
  a rejected credential that no line named (B685),
  replies the caller abandoned that no column counted (B687),
  a refusal the router gave before any request counted as the seat's failed ask (B711),
  Gemma listed as unable to read a picture on OpenRouter before any transcription was measured (B721),
  the readers bench counted by text reach where a picture needs vision reach (B722),
  and a picture reader measurement that lived in a scratch script (B723).
- [Audit ledger: short benches and the last live seat](audit-ledger-short-benches-and-the-last-live-seat.md):
  Quorums sized on the seats that could answer and how long a round waits on the last of them:
  the short bench that waited on its last live seat,
  where the owner let the grace open,
  with the round's need read again at each refusal,
  artifact generation 20 and the cache versions that moved with the rule (B816),
  the ceiling a hold could lift a short bench to,
  which the seat reader's line never stated (B833),
  a slice bought again by the next attempt for want of a second ballot (B834),
  the contest and consolidation rounds opened for slices their bench could not settle (B835),
  and the model that was the late voice on the wide benches and the checker order
  until the owner took those seats away (B1030).
- [Audit ledger:
  short bench floors and the waits that cross them](audit-ledger-short-bench-floors-and-the-waits-that-cross-them.md):
  The ceiling that counted a provider its meter keeps dry as coming back (B837),
  the wait that judged a return against the quorum alone (B838),
  the translate lane's first-time buys on a select bench under its floor (B839),
  and the artifact's record of why such a slice was kept (B840).
- [Audit ledger: the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md):
  The owner's rulings of 2026-10-25 on `XingZ6021`:
  the panel tally that asked for a summed weight of three and decided nothing for a unanimous panel of two (B900),
  the candidate selection's minimum weight and the exceptions that worked around it (B901),
  the three non-translation votes (B902),
  a gate that heard one judge and shipped a consolidation (B903),
  the cache versions that moved (B904),
  the replay of the recorded ballots (B905),
  and the floors that stay with the choices not taken (B906).
- [Audit ledger: the weight floors removed, rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md):
  The checker count of two (B907),
  a declined round that ships only what two judges named (B908),
  an unheard gate that ships the slate's two-ballot pick (B909),
  the page title and the consistency round on two voices (B910,
  B911),
  the cache versions (B912) and the replay after the round (B913),
  the addendum:
  a tie across every offered candidate that ships only among the candidates two judges named (B914),
  and round 3:
  a page title that settles only on a rendering two voices gave (B915)
  and the sheet text about a declined round that no longer contradicts itself (B916).
- [Audit ledger: the weight floors removed, round 4](audit-ledger-weight-floors-removed-round-four.md):
  The lead's rulings on the third re-check:
  a regressed issue that needs two ballots naming `worse` (B917),
  the sheet sentence about a declined round that says a decline does not by itself keep the wording (B918),
  and a page title round stored only when every title it asked is settled (B919).
- [Audit ledger: the weight floors removed, round 5](audit-ledger-weight-floors-removed-round-five.md):
  The answer to the fourth re-check and the owner's "Two ballots for undo too":
  a strip and a rollback that need two ballots naming `worse` (B922),
  the sheet sentence about a declined round told per round (B923),
  the clause reader that refuses swapped outcomes (B924),
  the page title round asked again in full (B925),
  and the cache versions that moved (B926).
- [Audit ledger: the weight floors removed, round 6](audit-ledger-weight-floors-removed-round-six.md):
  The answer to the fifth re-check:
  the rollback's level case that is reachable and tested (B927),
  the run-off the challenge sentences omitted (B928),
  the same judges that a round does not promise (B929),
  the two sentences the sheet-wide scans never read (B930),
  the nits and the clause reader's negations (B931),
  and the cache versions that moved (B932).
- [Audit ledger: the weight floors removed, round 7](audit-ledger-weight-floors-removed-round-seven.md):
  The answer to the sixth re-check:
  the rollback's level case that no longer depends on the sheet text (B933),
  the leader clause that was false in two shapes the code does not narrow (B934),
  the scans of the two new sheet entries and the nits (B935),
  and the cache versions that moved (B936).
  Round 8,
  the answer to the seventh re-check,
  is in the same file:
  the word behind that carried one meaning (B937),
  the position scan that passed a positional comment (B938),
  the closing sentence of B935 that overstated what fired (B939),
  the measuring-sheets case that read one sheet of five (B940),
  the evidence of B933 that lived outside the repository (B941),
  the sentence of round 7 about the merge (B942),
  and the cache versions that moved (B943).
  Round 9,
  the answer to the rotation sweep of round 8,
  is in the same file:
  the archive window case that held a seat the selection round may not ask (B944),
  the chunk decline case that had a seat name the candidate (B945),
  the two refine cases that read a self-vote (B946),
  and the cache versions,
  the gates and the lesson (B947).
- [Audit ledger: the weight floors removed, round 10](audit-ledger-weight-floors-removed-round-ten.md):
  The nits the eighth and ninth re-checks left:
  a split four-seat bench settled by the seat the rotation leaves out (B948),
  the position scan that read a verb,
  a floating quantifier and "that" as the object of a comparison (B949),
  the refiner left out of the window,
  the cache versions and the gates (B950),
  the scan that reads "that" before a bare verb,
  with the items the review of round 10 left (B951),
  and the scan that reads "that" only before a noun it knows,
  with the pins and nits of the re-check of round 11 (B952).
- [Audit ledger: the two-ballot translate text
  and the lead's adoptions of 2026-10-29](audit-ledger-the-two-ballot-translate-text.md):
  The owner's "Ship the two-ballot text" after the corpus pass `XingZ6022`:
  the translate lane's winner that ships where the final round cannot seat two judges (B1020),
  the defect note that said what happened at each kept archive (B1021),
  the footnote guard that withdraws only what provably clears the orphan (B1022),
  the follow-up that does not ask the same judges the same slate again (B1023),
  the trace of a patch that resolved every accepted issue and lost (B1024),
  and artifact generation 22 with the cache versions that did not move (B1025).
- [Audit ledger: holds
  a test registers and the seat reading sleeps](audit-ledger-holds-a-test-registers-and-the-seat-reading-sleeps.md):
  A hold a test registers for a provider that the seat reading sleeps for real:
  the router's five-minute hold slept by a unit test in every gate from g138 on,
  and the scan that now reads the holds tests register (B817).
- [Audit ledger: the every-provider-dry refusal,
  what it stops and what it says](audit-ledger-the-every-provider-dry-refusal.md):
  The router's refusal for a call made with every provider out of budget:
  lost as one seat's voice for 77 s and swallowed by four catches the pass guard hid (B818),
  the NeuralWatt return it stated as 00:00 UTC (B819),
  the entry in flight that ships nothing (B821),
  the refusal raised per model (B822),
  and the seat reader's case that depends on the roster (B823).
- [Audit ledger: a stage that heard nobody, recorded as data](audit-ledger-silent-stages-recorded-as-data.md):
  A panel that heard nobody on five packets cached as examined
  because its finding no longer began with the text the gate matched,
  the cache version that moved with the rule,
  and the refine cache's two gates that asked different questions (B820).
- [Audit ledger: providers and seats a run holds out](audit-ledger-providers-and-seats-a-run-holds-out.md):
  A provider an owner ruling keeps off that every run still asked (B760),
  a seat never withheld although its streams carried nothing (B761),
  a meter line that read a held provider differently from the router (B763),
  a model the owner's answer took off the lane-contest and consolidation benches
  after the grace cut 40 of its 95 streams (B815),
  a writing-bench floor that stopped an entry for want of a named return
  though NeuralWatt's return was 48 s away (B826),
  five phases judged against a bench their stage does not seat (B860),
  and a cooldown waited out for every slice by a provider that refused at once after each (B965).
- [Audit ledger: real-clock waits left in tests after the cooldown round](audit-ledger-real-clock-waits-left-after-the-cooldown.md):
  Three unit tests that still slept on the real clock after B965,
  a ten-millisecond hold in the pass-entry overlap case and two NeuralWatt return waits in the provider package,
  replaced by a rendezvous and a fake clock,
  and the gap in the real-clock scan that never saw the provider sites (B985),
  then the two 40 ms sleeps that round left open
  and the poll slices,
  elapsed readings and clockless budgets the widened scan now reads (B995),
  then the hold-wait file that round deferred,
  eight of whose ten sleeping cases passed with the wait removed,
  and the router and constant-held fake clock forms the scan now reads (B996).
- [Audit ledger: probes that read one editor card](audit-ledger-editor-card-probes.md):
  An editor calibration with no flag naming a card (B870),
  a flag that confines the editing and not the spend (B871),
  a lone editor whose slate ships unjudged (B872),
  a relative share that moved from 52.2 to 26.7 to 10.0 percent across identical runs (B874),
  the absolute probe that reads one card's edit against a known fix and the pieces it stands on (B875 to B880),
  the per-call spend guard that replaced a gate on the first-pass total (B881 and B882),
  the end-of-section insertions it could not score (B883),
  the process slips of two rounds (B884),
  the guard that counts a throw and a retried attempt (B885),
  the bench amount and the read step over the arm logs (B886 to B889),
  the slips of a fourth round (B890),
  the decision rule held as code with a floor from an exact enumeration (B891),
  the dry run that prints what the arm will do (B892),
  and the slips of a fifth round with the cache statement (B893).
- [Audit ledger: the editor bench's paired rule and trials](audit-ledger-editor-bench-paired-rule-and-trials.md):
  The floor of B891 replaced by a paired exact test for each effort,
  with its false-pick chance and power (B894),
  the pool that holds every trial the clone offers (B895),
  the read step over three or four logs that absorbs a failed trial in the trial (B896),
  the spend guard's one-call-at-a-time assertion (B897),
  the retry-ladder hook with the cache statement of round 6 (B898),
  and the bench total of 60 USD with the replicates the rule reads as units (B899).
- [Audit ledger: the editor bench's multi-pass arms,
  the false-pick bound and the repeat agreement](audit-ledger-editor-bench-multi-pass-arms-and-repeat-agreement.md):
  The follow-ups of B899 from round 8 on:
  the planned line of an arm that walks the pool in passes,
  the false-pick bound under unequal repeat agreement,
  the lines that print each arm's repeat agreement and the fix rate of the default effort,
  and the runbook with its stop rule and pre-launch read.
- [Audit ledger: the editor bench's exact repeat test, the high fix rate
  warning and the self-contained runbook](audit-ledger-editor-bench-exact-repeat-test-and-self-contained-runbook.md):
  The follow-ups of B899 from round 10 on:
  the high fix rate warning that meets the rule's stated region,
  the exact test of the repeat warning with its measured rates,
  the lead's actions on two warnings,
  and the runbook blocks that each set the variables they read.
- [Audit ledger: the editor bench's prompt reuse,
  the re-check fixes and the bench outcome](audit-ledger-editor-bench-prompt-reuse-in-repeat-passes.md):
  The follow-ups of B899 in rounds 11 to 15:
  repeat passes answered by the prompt reuse layer instead of the model,
  each reuse booked at its spend ceiling,
  the guard and the read step that now refuse a row that made no call,
  the README's stop rule over rows that made a call,
  the fixes of the re-checks of rounds 12 to 14,
  and the outcome of the bench,
  with medium kept as the editor's effort.
- [Audit ledger: blocks and quotes read off the parse](audit-ledger-blocks-and-quotes-read-off-the-parse.md):
  Blocks,
  quotes and code spans counted by a blank-line split or a scan of their own,
  where the parse was meant.
- [Audit ledger: positions compared across texts](audit-ledger-positions-compared-across-texts.md):
  Offsets,
  headings and slices of one text applied to another that does not line up with it,
  a leading byte order mark among the causes,
  a heading series that lost the original's numbers where the archive heads it by name alone (B695),
  and a quote copied from a sheet that cut the notes between translators searched in the stored text (B709).
- [Audit ledger: repeated wording and ids on a page](audit-ledger-repeated-wording-and-ids-within-one-text.md):
  Titles,
  links,
  ids and wordings that stand more than once on a page,
  settled by their first or last occurrence without saying so.
- [Audit ledger: repeated keys in stored records and rows](audit-ledger-repeated-keys-in-stored-records-and-rows.md):
  Stored records,
  rosters,
  ledger rows and log lines that name one key more than once,
  read by the first or the last without saying so.
- [Audit ledger: command lines read by each runner](audit-ledger-command-lines-read-by-each-runner.md):
  How the runners read their command lines,
  and ids a line names that its runner cannot reach.
- [Audit ledger: operator refusals printed as faults](audit-ledger-operator-refusals-printed-as-faults.md):
  Refusals of an operator's own mistake that printed as faults of the command,
  and refusals that came after work an operator could not be told about.
- [Audit ledger: caught text carried into messages](audit-ledger-caught-text-carried-into-messages.md):
  Refusals,
  log lines and stored details that carried a caught error's own text,
  which can quote a provider body or a document.
- [Audit ledger: refusals that name too little](audit-ledger-refusals-that-name-too-little.md):
  Refusals and failure lines that named too little to act on,
  or named the wrong cause,
  a bridge that could not start and left degraded findings with exit code zero (B689),
  oxlint output that was not JSON and reached the plugin tests as a bare `SyntaxError` (B690),
  every provider's HTTP refusal named as Synthetic's,
  with no provider (B692),
  Synthetic's request-size cap claimed for every provider's reply (B693),
  and the client type every driver takes named for Synthetic (B694).
- [Audit ledger: process mistakes](audit-ledger-process-mistakes.md):
  The index of the slips in how the work was run,
  checked,
  delegated and reported:
  shell commands and tasks,
  launch checks,
  guards,
  searches,
  claims,
  edits and tests,
  questions to the owner,
  agents and briefs,
  commit gates,
  stopping and waiting,
  and docs that grow or go stale.
- [Audit ledger: where each numbered entry is filed](audit-ledger-entries-by-id.md):
  Every numbered entry of the recurring code families,
  the process mistakes and the other numbered findings,
  by id,
  with the file that holds it.
- [Audit ledger: the
  areas of the audit and the files filed under each](audit-ledger-areas-of-the-audit-and-the-files-filed-under-each.md):
  The probes of each area of the audit,
  the run it stopped,
  the T8 entry (code no unit test runs) with its census method,
  and the files each area's findings are filed in.
