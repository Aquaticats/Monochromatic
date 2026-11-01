# Audit ledger: code no unit test runs, stretch queue batches seventeen to twenty-four

The stretch queue,
batches seventeen to twenty-four,
each read against a whole-suite census,
from the slice modules to the page modules.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Stretch queue, seventeenth batch: slice modules

The first finding the sixteenth batch left open became B70 and was fixed before the seventeenth batch,
in four commits
(`42d7a0f08`,
`be0897f00`,
`f033d19e8`,
`66b77b040`);
the `prefix` callback stays open.
The seventeenth batch reads against the census at `66b77b040`
(`census-DapwaX`,
1,472 passes,
taken clean),
which reads against `census-PgyHZ8`:
ran 0,
still cold 1,104,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 7.
The 7 edited sources are B70's
(`apply-patch.ts` 2 cold stretches,
`archive-italic-title-restore.ts` 1,
`archive-stub.ts` 2,
`dropped-destinations.ts` 3,
`list-spread-restore.ts` 7,
`published-page-check.ts` 4
and `markup-atom-preservation.ts` 1).
Library source went from 766 stretches over 1,554 lines in 282 files to 765 over 1,547 in 282,
with 15 functions never called where there were 16:
compared file by file,
only `archive-italic-title-restore.ts` moved,
from 2 stretches to 1,
since `lastFirst`,
the comparator ordering its titles last first that no test called,
went with that splice.
By the first construct
(`t8-triage-dapwax.txt`),
the queue is 311 returns,
156 nullish fallbacks,
129 ternaries,
86 throws
and 83 others,
and the tie is unchanged:
`slice`,
`archive`
and `corpus-run/run` at 18 stretches each,
with 112,
36
and 18 lines,
so the seventeenth batch still takes `slice`.

The seventeenth batch took the slice modules against `census-DapwaX`,
18 stretches over 112 lines in 7 files,
in five code commits,
`774710cb0` to `3bc823c13`,
and found B71 on the way.
Its rows read three ways.

The unreachable ones went.
`slice-pair` 76,
a hand-written throw on an empty run,
became `nonNullishOrThrow`,
which still throws.
`slice-pair` 318-350,
a one-slice fallback for a section one side of which carries no blocks:
every caller takes its pairs from `alignDocumentSections`,
whose `chunkByHeadings` chunks each hold a node,
and a target is such a chunk or an insertion anchor,
so the function now returns early for an anchor and groups both sides otherwise.
The test that named the fallback ran through the grouping,
its translation being a bare heading;
it is reworded and now checks it gets one slice.
`slice-delivery-decide` 230-235,
`nonNullishAccepted`'s throw:
`decideDelivery` returns the document's text beside the delivery,
so the shipped row reads it there,
and the fault kind went with its sentence (`slice-delivery-fault` 270).
`slice-delivery-decide` 187-188,
a gap for a decision agreeing with an archive that holds nothing:
the coherence check refuses wording beside an absent archive and an empty decision there,
so `decideDelivery` now runs that check itself and the arm went.
`slice-delivery` 327-332,
a missing wording after the count check,
became `nonNullishOrThrow`,
and its fault kind and sentence went (`slice-delivery-fault` 294).
`slice-cost-read` 244 and 292 went with B71's one-pass reader.

The reachable ones have cases (`1845b72c2`):
declined and sealed blocks placed in a slice (`slice-coverage` 207-220,
410-417 and 441-449),
a withdrawn set naming a slice the lane never reached (`slice-delivery-decide` 112),
a wording naming another slice than its position's (`slice-delivery` 342-349),
a record calling the archive present at an anchor (`slice-delivery` 368-374,
`slice-delivery-fault` 302-306)
and a picture carrying no text (`slice-pictures` 208-215).
Both files' existing refusal cases asserted the class alone or a fragment of the message,
and every check in each file throws one class;
they now assert the refusal's structured fault whole.
One fixture put `acceptedText` beside a wording's outcome rather than in it,
so its outcome never changed;
it now sets the outcome.

One case was red on purpose:
the anchor refusal read "slice 1 is present of archive wording by its lane record and the other way by its prepared chunk",
and `3bc823c13` rewords it to say which record says what.

The batch's control copy,
the pictures test with its new case expecting `not read`,
failed that case alone and was deleted.
With each guard turned off in one build
(the digit check in `countField`,
the declined check,
the index and kind checks in `buildSliceDelivery`,
the withdrawn name in `decideDelivery`,
the no-text finding),
exactly the case covering it failed in each file,
and the tree was clean after the restore.
The sealed-blocks check was not turned off:
the session's permission classifier refused that edit,
so its case is unmeasured against a guard-off.
The write tool,
like the edit tool,
wrote a rewritten file's spaced TSDoc blank lines empty;
each was put back by line address before lint,
and `tsdoc-blank-lines.mjs` read every edited file's form after.

Three findings are open from this batch.
`armSliceCost` measures a slice as a difference of `Date.now()` readings,
the wall clock,
and the package does so for most of its elapsed times
(`performance.now()` in two files);
each site is to be read for whether it measures work,
which wants a monotonic clock,
or a budget the owner reads in wall time.
B78 has since closed that family,
reading every duration inside a process on a monotonic clock,
the budgets among them,
a call recorded there open to veto.
The package has about fifty number reads from text in 32 production files,
several on operator-typed variables;
each is to be read for an empty,
signed,
hexadecimal,
exponent or unsafe value read as a number nobody wrote.
B73 has since closed that family.
And `coverageSentence` and `blockPlacementSentence` print a count of one before a plural noun,
which the open family of counts before a fixed plural covers.

Its census at `3bc823c13`
(`census-vi1FHB`,
1,472 passes,
taken clean)
reads against `census-DapwaX`:
ran 8,
still cold 1,103,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 5,
each of the 5 loaded and left with 0 cold stretches.
Library source went from 765 stretches over 1,547 lines in 282 files to 747 over 1,435 in 275,
with 15 functions never called as before:
the cluster's 18 stretches and 112 lines,
its 7 files no longer listed.
Compared file by file,
the 8 that ran are the cluster's 5 in the two unedited files
(`slice-coverage` 4,
`slice-pictures` 1)
and 3 in other packages no commit here touched
(`with-timeout.ts` 1 and `create-logger.ts` 2,
whose `onTimeout` and `markEntryUnavailable` left the never-called list),
while the logger's `reportLoggerInternalError` (`error-format.ts` 31) left it too:
called this time,
its one cold stretch became 3 inside the same range,
so the reading counts none as newly cold.
Those three move with timing the suite does not fix
(a logger sink verification that timed out appears in this batch's lint output),
an inference not measured here.

## Stretch queue, eighteenth batch: archive modules

By the first construct
(`t8-triage-vi1fhb.txt`),
the queue for the eighteenth batch is 306 returns,
156 nullish fallbacks,
127 ternaries,
77 throws
and 81 others.
Two clusters tie at 18 stretches:
`archive` (9 files,
36 lines)
and `corpus-run/run` (2 files,
18 lines).
The tie goes to the most cold lines,
so the eighteenth batch takes `archive`.

B72 closed before that batch,
and its census at `3429c8229` (`census-apleN5`) is recorded under B72.
Its triage (`t8-triage-aplen5.txt`) leaves the queue at 306 returns,
156 nullish fallbacks,
126 ternaries,
77 throws
and 81 others:
the stretch B72 removed was a ternary in `corpus-run/archive`,
a cluster of its own,
so `archive` and `corpus-run/run` still tie at 18 stretches,
and the eighteenth batch takes `archive`.
B73 and B74 followed,
and three findings were to close first (the elapsed-time family,
a flag written twice and prototype keys).
B75 closed the repeated flag and found a fourth,
an `--only` naming no entry,
which B76 closed,
and B77 closed the prototype keys,
so the elapsed-time family closes first,
and the batch reads against the census taken once it closes.
B78 closed it (`a3f1f2c55` to `e2b0962fd`),
so the census taken next is the batch's baseline.

That census at `36e3f36cc`
(`census-dKGOhZ`,
1,500 passes,
taken clean)
reads against `census-apleN5`:
ran 0,
still cold 967,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 36.
Library source went from 746 stretches over 1,432 lines in 274 files to 730 over 1,404 in 274,
with 15 functions never called as before.
The 36 edited sources are B73 to B78's,
whose baseline lines name other code,
so the reading cannot match their stretches;
compared file by file instead
(`~/temp/agent/audit-glossary-fix/b78-edited-cold.mjs`,
each file's cold stretches and lines in both censuses),
35 hold as many cold stretches as before or fewer,
and one holds more:
`corpus-run/command-line.ts`,
0 then 2,
both arms of `usageLine` that print a position which repeats (246)
and a single position that may be left out (248),
code B75 added that no case prints,
though the command table declares both shapes
(`unit test file` and the slice counts).
They are added to this batch.
By the first construct
(`t8-triage-dkgohz.txt`),
the queue is 302 returns,
150 nullish fallbacks,
122 ternaries,
76 throws
and 80 others;
`archive` and `corpus-run/run` still tie at 18 stretches,
and the eighteenth batch takes `archive`,
36 lines in 9 files to `corpus-run/run`'s 18.

The eighteenth batch took the archive modules against `census-dKGOhZ`
and the two `corpus-run/command-line.ts` usage-line arms added to it,
20 stretches in 10 files,
none of whose sources had changed since that census
(`git diff --stat 36e3f36cc` over them printed nothing),
in seven commits,
`9a362ba09` to `67f85061a`,
and no entry of its own.
Its rows read three ways,
and rows in four files hid defects.

The defects,
each committed red before its fix:

- `archive-footnote-relabel` 206-207,
  the ambiguity detail's `?? ''` fallback,
  cold because no case had two archive labels meet one original label.
  The detail ended "where an earlier slice mapped [^X]",
  X an original label in one conflict and an archive label in the other with nothing saying which,
  in the parser's case-folded key (`[^tabby]` for a label both documents spell `[^Tabby]`),
  and "slice" was the later claim's kind of place,
  so the widening test asserted "an earlier slice" for a relation a definition pair gave.
  `mapLabels` now keeps the claim that first mapped each label and says
  "where <place> mapped that archive label to original [^X]" or
  "where <place> mapped archive [^W] to that original label",
  in the claim's spelling;
  the correspondence's `unit` field,
  read nowhere else,
  went (`9a362ba09`,
  `371c49efe`).
- `archive-revision-footnotes` 70,
  a block the archive does not carry,
  for which the footnote floor passed every revision unread,
  a branch its TSDoc kept for fixtures a caller composed and no test reached;
  production hands the floor an exact slice (`corpus-run/archive-block-repair.ts`),
  so the floor now throws `BlockOutsideArchiveError`,
  a marked class with one fixed sentence (`9a362ba09`,
  `371c49efe`).
- `archive-footnote-order` 311,
  the throw when moved definitions reparse differently.
  A bounded probe over the build (`b18-order-fuzz.mjs`:
  2,000 documents from a fixed-seed grammar of definition contents,
  indents,
  gaps and surrounding text,
  the reverse order asked)
  read 1,351 moved,
  493 left standing,
  156 such throws and no other error,
  and an enumeration of two-definition pages (`b18-order-minimal.mjs`) 148 of 2,312 with no empty definition among them;
  one shape is a definition whose fence was written unindented,
  whose lines a paragraph definition moved after it takes as a lazy continuation.
  `relabelArchiveFootnotes` read the throw as a rewrite refusal and kept the whole archive,
  the sound rename with it,
  under a message blaming the rename,
  where the move's other refusals leave the definitions standing with a note.
  The move now refuses the same way,
  and the pass applies the rename and says the definitions stand
  (`bd59138ad`,
  `837c393e2`);
  the probe on the fixed build read 1,351 moved,
  649 standing (406 with a note) and none thrown.
- `archive-revision-shape` 66 and 77-81,
  cold for want of a block or revision the slice grammar refuses.
  Writing the cases found the refusals saying "the block is paragraph and the revision is paragraph,
  paragraph",
  a list where a count belongs;
  they now say "1 block (paragraph)",
  the translate block floor's form (`becf044c4`,
  `67f85061a`).

The unreachable ones went (`783c2f420`).
`archive-footnote-closure` 217-223 refused the closure's own result for merging two archive labels,
which the construction cannot do:
the supplied relations are injective (`readFootnoteClosureInput` refuses any that are not),
the eliminated pair joins labels no relation names,
each displaced label takes a distinct fresh label outside both namespaces,
and an archive label nothing moves is no relation's destination,
or it would have been displaced;
`footnote-rewrite-map.ts` also refuses a merging map where it is applied.
The check went,
the TSDoc states the proof,
and `archive-footnote-closure-injective.unit.test.ts` closes every relation set over every pair of nonempty sets drawn from three labels
(1,178 inputs,
247 closed,
42 taking the forced pair,
24 displacing a label,
0 merging,
by `b18-closure-reach.mjs`)
and asserts it reaches closed,
forced and displaced results.
`archive-footnote-relabel` 339,
a hand-written throw for a position the two lists of one length cannot lack,
is `nonNullishOrThrow`.
`archive-dispute` 458 dropped an empty archive wording from a disputed slice's refused wordings:
a dispute needs accepted issues,
an anchor's repair outcome carries none (`repair-slice-settle.ts`),
a present slice holds a parsed block's text,
and `disputedWordingFindings` never matches an empty wording anyway.

The reachable ones have cases (`becf044c4`):
the singular contributor label and its full-width colon,
which fit no prefix,
a picture and a comment closed and not (`archive-block-evidence` 139,
145 and 146);
each naturalness responsibility starved of all but one seat's replies,
the scripted client cutting one sheet's replies,
with only that responsibility's quorum line recorded (`archive-block-naturalness` 131-135);
guard rows for a reply that is no record,
a quote that is not text,
no replacement and no finding (`archive-block-review-wire` 198,
202 and 204);
`logArchiveDisputes` warning once per dispute in slice order,
and `archiveDisputeNotesOf` keying a note to each disputed slice and no other (`archive-dispute` 517 and 634-642);
and refusals ending with a position taken one or more times,
any number of times,
and one that may be left off (`command-line` 246 and 248).
The two revision floors are exported for their own test files.

Two corrections came on the way.
Lint refused the enumeration's `slice()` copy as a spread and then the spread as a needless copy,
two rules meeting over a loop that appends to the list it reads;
the subset builder now maps before it appends.
And lint asked for the new class's constructor TSDoc and for two nested calls split in the new relabel case.

The batch's control copy,
the revision-shape test with its first case expecting a finding the floor does not write,
failed that case alone and was deleted.
With each guard turned off,
one per test file per build,
exactly the case covering it failed in each file
(build one:
the contributor reader's branch,
the finding type check,
the dispute warning,
the revision refusal of an unreadable text,
the repeating usage arm,
the backward relabel conflict,
the fresh labels kept out of both namespaces,
and the reparse refusal of a move,
which failed the reorder case and the pass case;
build two:
the defect-discovery quorum line and the dispute notes),
and `git diff` printed nothing after each restore.
The revision floor's new refusal was not turned off:
the session's permission classifier refused that edit,
so its case is unmeasured against a guard-off.

Two findings are open from this batch.
`isVerifiableEditorialArchiveBlock` accepts a block whose first words are a label,
a picture or a comment,
or any of whose lines is a contributor line,
so prose following a label in one block passes as apparatus;
how often the pinned archives carry such a block is to be measured before any tightening;
B81 measured none and closes it.
And `revisionFootnoteFindings` reads each revision against the archive as it came,
while `archive-block-repair.ts` composes the revisions it accepts into one text,
so two revisions harmless alone could together leave a footnote defect no check sees;
B80 closes it.

Its census at `67f85061a`
(`census-JXU9Js`,
1,505 passes and no FAIL line,
taken clean)
reads against `census-dKGOhZ`:
ran 10,
still cold 1,067,
cold since then 0,
not loaded 6,
claimed sources with no stretch there 0,
sources edited since then 6.
The 10 that ran are the batch's stretches in the four files it left unedited
(`archive-block-evidence` 3,
`archive-block-naturalness` 2,
`archive-block-review-wire` 3,
`corpus-run/command-line.ts` 2);
the 6 edited are its other files,
five loaded and left with 0 cold stretches,
and `archive-revision-shape.ts` with 1:
the "nothing" arm of `countedBlocks` that `67f85061a` added,
which a revision of white space alone reaches,
since the review guard admits one and only an empty revision is a removal;
`f9babaec4` cases it,
and with the arm's word changed that case failed alone in its file.
Library source went from 730 stretches over 1,404 lines in 274 files to 711 over 1,367 in 265,
with 14 functions never called where there were 15,
the one no longer listed being `archiveDisputeNotesOf`'s `toNote`.

## Stretch queue, nineteenth batch: `corpus-run/run`

By the first construct
(`t8-triage-jxu9js.txt`),
the queue for the nineteenth batch is 293 returns,
149 nullish fallbacks,
117 ternaries,
74 throws
and 78 others.
`corpus-run/run` leads at 18 stretches
(2 files,
18 lines),
before `corpus-run/heading` at 17,
so the nineteenth batch takes `corpus-run/run`.
The two findings this batch left open close first,
and the batch reads against the census taken once they close.

The full suite on `7f7b1610a`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,505 PASS lines and no FAIL line.

The nineteenth batch took `corpus-run/run` against `census-EJl6yz`.
Its reading found four defects,
red in `71f7720a9` and fixed in `43e444768` and `8175dc866`
(ledger B82);
`8c622b09e` cased three reachable arms that already behaved,
and the census's line-340 stretch is an attribution note,
since the existing no-slash ratio case reaches that refusal.
The closing census,
`census-IqLEk4` at `8175dc866`,
read against `census-EJl6yz`:
ran 0,
still cold 1,046,
cold since then 4,
not loaded 6,
sources edited since then 2.
The 4 cold since then are the other packages' timing-dependent stretches recorded for earlier batches
(`with-timeout.ts` 57-59,
`create-logger.ts` 240-247 and 366-373,
`error-format.ts` 31-58).
Of the edited sources,
`run-timing-read.ts` left 0 cold stretches
and `run-timing-parse.ts` 3,
all reachable frame refusals the fix added,
which `37f637675` cases.
Library source went from 711 stretches to 693 in 263 files.

## Stretch queue, twentieth batch: `corpus-run/heading`

By the first construct
(`t8-triage-iqlek4.txt`),
the queue for the twentieth batch is 292 returns,
132 nullish fallbacks,
118 ternaries,
73 throws
and 33 others.
`corpus-run/heading` leads at 17 stretches
(3 files,
25 lines),
so the twentieth batch takes `corpus-run/heading`
against `census-IqLEk4`.

The full suite on `143e786eb`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,509 PASS lines and no FAIL line.

The twentieth batch took `corpus-run/heading` against `census-IqLEk4`.
Reading its cold stretches found B83,
a heading restore that shipped a section the page never replaced as empty text,
and reading B83's family found B84,
two passes pairing a slice's headings by position without checking the count
(both recorded in their own entries).
The rest were reachable arms,
cased in `4c0039a57` and `c6464ab59`
(`heading-title-lines.ts` had no test file and has one now),
and unreachable fallbacks,
which throw now.
The closing census,
`census-STKkse` at `4c0039a57`,
read against `census-IqLEk4`:
ran 6,
still cold 1,003,
cold since then 0,
not loaded 6,
sources edited since then 8.
Of the edited sources,
`heading-series-unify.ts` left 0 cold stretches where the baseline held 10,
`heading-collision-restore.ts` 1 where it held 4
(the return cased in `c6464ab59`),
five of the six other files the B83 and B84 fixes touched left fewer than the baseline held,
and `assembly-page-text.ts` kept its one,
the stretch the baseline had at line 315 standing at 389 below the added reader;
the stretches left in those six files belong to other clusters.
Library source went from 693 stretches in 263 files to 666 in 260.

## Stretch queue, twenty-first batch: `assembly`

By the first construct
(`t8-triage-stkkse.txt`),
the queue for the twenty-first batch is 286 returns,
120 nullish fallbacks,
112 ternaries,
72 throws
and 76 others.
`assembly` leads at 16 stretches
(8 files,
27 lines),
before `anthropic` at 16 over 22 lines,
so the twenty-first batch takes `assembly`
against the census taken once this batch's docs close.

The full suite on `7b61bb928`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1512 PASS lines and failed the package-wide case in `position-references.unit.test.ts`:
the B84 entry this batch's docs commit wrote pointed at a count by position
(M59 records the recurrence).
`ac44c5378` names the count,
and the full suite on it printed 1512 PASS lines and no FAIL line.

The twenty-first batch took `assembly` against `census-zkJwfY`
(taken at `b9da490fb` once the twentieth batch's docs closed:
665 library stretches in 259 files,
`assembly` 16 stretches in 8 files).
Reading its cold stretches found B85,
an advancing withdrawal that compared refusal offsets on two different pages,
and checking what B85's fix compares found B86,
a strict refusal read at the start of what the parser left behind
(both recorded in their own entries).
Of the 16 stretches,
five were reachable arms,
cased in `4af8a6107`:
a container's opening half withheld when its closing half ships nothing,
a doubled definition whose carriers' originals both define it,
a line opening with a reference and no colon,
a repeated run of twelve function words,
and a page where no single withdrawal moves the refusal.
Ten could not run:
the moved-while-nothing-survived check and its sentence,
which the re-splice before it already made (`a67db99e3`),
and eight lookups of maps built over the very keys they were asked for,
two in `assembly-integrity.ts` and six in `assembly-repetition-span.ts`,
which throw now (`0c1c1fe12`).
The last,
the reducer that picks the furthest of several advancing withdrawals,
still runs in no case;
B85's entry gives the route by which it can.
The arms B85's fix added are cased in `c17feba5b` and `9244d67d3`
(the first message claimed one arm its case does not reach;
M102 records it).
The closing census,
`census-dnr81C` at `9244d67d3`,
read against `census-zkJwfY`:
ran 3,
still cold 1,003,
cold since then 1,
not loaded 6,
sources edited since then 7.
The three that ran are the cased arms in sources the batch left unedited
(container halves,
orphan trim,
repetition).
Of the edited sources,
`assembly-contract-fault.ts`,
`assembly-integrity.ts`,
`assembly-invariant.ts`,
`assembly-repetition-span.ts` and `strict-refusal-offset.ts` left 0 cold stretches,
`assembly-structural-withdrawal.ts` 1
(the reducer),
and `parse-mdx.ts` 2,
the two arms for a cause that is no `Error` the baseline held at its lines 53 and 109
(cluster `parse`).
The stretch cold since then is `request-pace.ts:128`,
in a source and test no commit of the batch touched:
it ran at the baseline and in the two censuses taken during the batch,
so some test reaches the default sleeper on a path that depends on time,
and that is now its own task.
Library source went from 665 stretches in 259 files to 650 in 251.

## Stretch queue, twenty-second batch: `anthropic`

By the first construct
(`t8-triage-dnr81c.txt`),
the queue for the twenty-second batch is 281 returns,
112 nullish fallbacks,
111 ternaries,
71 throws
and 75 others.
`anthropic` leads at 16 stretches
(2 files,
22 lines),
before `corpus-run/attribution` at 15 over 18 lines,
so the twenty-second batch takes `anthropic`
against the census taken once this batch's docs close.

The full suite on `a44cf50c4`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1512 PASS lines and no FAIL line.

The twenty-second batch took `anthropic` against `census-lxT62v`
(taken at `73bd4426c` once the twenty-first batch's docs closed:
649 library stretches in 251 files,
`anthropic` 16 stretches in 2 files).
Reading its cold stretches against the Anthropic streaming and prompt-caching documentation found B87 to B91,
and writing cases for the arms left lenient found B93 and B94
(each recorded in its own entry).
B92,
`isJsonRecord` passing arrays where its callers refuse anything but an object,
surfaced in the same cases and is queued as a family of its own;
the whole-message case for a payload that is not a frame still feeds it an array,
so this cluster is not clear of that family.
The arms left lenient,
each a metadata frame missing a part the answer does not need,
are cased in `08635488c`
(its message calls a fabricated zero count an absence;
the comment on that commit corrects it,
and B94 fixes the zero).
Fifteen guard-offs over eight builds,
one guard per test file per build,
each made a case for its fix fail
(B87,
B88,
B89,
B90,
B91,
B93 and B94),
and each was restored with `git diff --stat` empty.
The closing census,
`census-Hq0z31` at `a41e6218b`,
read against `census-lxT62v`:
ran 0,
still cold 982,
cold since then 0,
not loaded 6,
sources edited since then 5.
Of the edited sources,
`anthropic-completion.ts` and `anthropic-delta-scan.ts` left 0 cold stretches,
and `completion-shape.ts`,
`hyper-client.ts` and `stream-completion.ts` kept the 1,
3 and 4 the baseline held,
which belong to other clusters;
`anthropic-whole-message.ts`,
added by the batch,
left none.
Library source went from 649 stretches in 251 files to 633 in 249.

## Stretch queue, twenty-third batch: `front`

By the first construct
(`t8-triage-hq0z31.txt`),
the queue for the twenty-third batch is 270 returns,
110 ternaries,
109 nullish fallbacks,
70 throws
and 74 others.
`front` and `corpus-run/attribution` lead at 15 stretches each,
over 20 lines in 3 files and 18 lines in 2;
the more lines decide it,
as they did for the twenty-first batch,
so the twenty-third batch takes `front`
against `census-1B1Mb4`,
taken at `06b905d3e` once this batch's docs closed
(633 library stretches in 249 files,
ran 0 and sources edited 0 against `census-Hq0z31`).

The full suite on `06b905d3e`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1513 PASS lines and no FAIL line;
the 13 lines holding the word are passing cases' names
and the census runner's own case printing a failure on purpose.

The twenty-third batch took `front` against `census-1B1Mb4`.
Reading its cold stretches found the YAML refusal arms unreachable:
yaml's `parse` throws a `YAMLError` carrying a code and a position,
or a plain `ReferenceError` or `TypeError` for an anchor fault,
so `dcb850374` reads refusals through yaml's own class
and reads a position through `nonNullishOrThrow`.
It also found B97,
a list-shaped alias that skipped the identity rule
(recorded in its own entry).
The reachable arms are cased in `999aaa60a`,
whose new cases `7254593ed` makes name their targets rather than point at them.
The closing census,
`census-NkULde` at `ae5be43fe`,
read against `census-1B1Mb4`:
ran 11,
still cold 755,
cold since then 0,
not loaded 6,
sources edited since then 71,
the batches' fixes from B92 to B100 among them.
Of the cluster's three sources,
all edited since the baseline,
`front-matter-comment-authority.ts` left no cold stretch
and the other two left one each:

- `front-matter.ts` `parseFrontMatterYaml` rethrew a caught value that is not an `Error`,
  which yaml's `parse` never throws;
  it now hands the caught value to `errorOrThrow` (`module-or-throw`),
  which fails naming what arrived,
  so no arm a case cannot reach is left
  (issue 583 records that `errorOrThrow`'s TSDoc says `instanceof Error`
  where its body calls `Error.isError`);
- `front-matter-translation.ts` `visibleIdentityOf` read an alias that is neither text nor a list of text as another schema,
  an arm B97 added after the baseline with no case;
  `front-matter-slice.unit.test.ts` now carries one:
  a one-item alias list holding a mapping holds no identity rule,
  where the same list holding the name as text refuses a candidate dropping it.

Library source went from 633 stretches in 249 files to 605 in 246.

## Stretch queue, twenty-fourth batch: `page`

By the first construct
(`t8-triage-nkulde.txt`),
the queue for the twenty-fourth batch is 248 returns,
109 nullish fallbacks,
108 ternaries,
63 throws
and 77 others.
`corpus-run/attribution` leads alone at 14 stretches over 17 lines in 2 files,
so the twenty-fourth batch takes it,
against a census taken once this batch's docs close.

That census,
`census-3G9C58` at `d0cca211d`,
holds none of those 14:
B106 (`8cc9a3650`) and B107 (`837c18669`) cased the attribution reader's arms
before the batch began.
Library source stood at 590 stretches over 1291 lines in 242 files,
15 functions never called.
By the first construct `page` leads at 13 stretches in 4 files
(`page-headings.ts`,
`page-name-glossary.ts`,
`page-title-lexicon-wire.ts`,
`page-visible-text.ts`),
none edited since `d0cca211d`,
so the twenty-fourth batch takes `page` against `census-3G9C58`.

The twenty-fourth batch took `page` in `dc9f0449b`,
from a read-only agent's copies of `8ea5e7fa5`.
Twelve stretches are cased:
`page-headings.ts` and `page-visible-text.ts` have no test file and no export reaching the package index,
so their arms are cased through `pageNameLines` in `page-name-glossary.unit.test.ts`,
the shape the twenty-third batch used for `front-matter-translation.ts`;
the two `isPageTitleLexiconWire` refusals are cased in `page-title-lexicon.unit.test.ts`.
One stretch was unreachable:
`signaturePairs` returns early unless source and target hold the same count of signatures,
then reads the target at each source index,
so its `partner === undefined` return could never run;
it now reads through `nonNullishOrThrow`.
`pairsInOrder` carried the same dead disjunct under the same length check,
which the census could not show,
since the condition's other disjuncts run;
it reads through `nonNullishOrThrow` too.
Before committing,
the agent's two new comments,
which named the length check as "the check just made",
were rewritten to name the function that returns early;
nine new cases that each restated the glossary's header line,
which one case already pins,
were cut to compare the pairs under test;
and the agent's report of a "phantom" empty-text link in `linksOf`
was checked against micromark 4.0.2,
which reads `[猫[](url)` as a literal `[猫` and an empty link,
as `linksOf` does,
so `linksOf`'s `@returns` now says so instead.
