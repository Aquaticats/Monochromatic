# Audit ledger: blocks and quotes read off the parse

Blocks,
quotes and code spans counted by a blank-line split or a scan of their own,
where the parse was meant.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B42: the quote guard counted quotes on a blank-line split

Status:
fixed 2026-09-30 (UTC),
found closing the coverage census's translate-slice stretch for the quote-loss refusal (ledger T8's sixth batch).
The stretch looked unreachable:
the floor requires every top-level block of the page as it stands in the candidate,
of the same kind and in the same order,
so a floor-valid winner keeps every top-level blockquote.
It was reachable,
because the guard did not count what the floor reads.
`quoteBlockCount` split a text on blank lines and counted the chunks opening with `>`,
and the parser disagrees twice:
a quote opening on the line after a paragraph's is a blockquote to the parser and the floor,
and was none to the count;
a quote inside a container tag sits inside a block the floor compares only as a whole,
and was none to the count unless blank lines set it apart.
A scratch probe over the build from before the fix found a floor-valid candidate the guard refused in two shapes,
a paragraph then a quote,
and a quote in a list item,
each with no blank line opening the quote.
Over the stored translate records on disk
(2,958 records,
8,295 distinct texts,
read through the floor's own reader,
`~/temp/agent/audit-glossary-fix/b42-reach-floor-reader.mjs`):
the old count was below the parse's in 26 texts and above it in none,
and 20 texts carry a quote below the top level,
which the raw parse finds inside a container tag or inside another quote
(`b42-nested-quote-kinds.mjs`).
Of the 4 stored quote-loss refusals,
one,
hulicaijia24 slice 2,
kept the archive over a replacement the parse reads as carrying its one quote;
replayed through the fixed guard,
it ships,
and the other three,
XingZ60 slice 88 in two runs and slice 29,
still refuse
(`b42-stored-refusals-replay.mjs`).

The reporter was wrong beside it.
`alignmentRefusals` rebuilt the refusal's sentence by recounting the record's `outputText`,
the whole archive,
where the guard compared the archive's judged part with the target-only transcript held out,
so a slice with a held-out transcript would have reported the transcript's quotes as the judged part's
(read from the code;
no stored refusal carries a held-out transcript).
The sentence also said "1 quoted passages".

Fix:
the slice skeleton counts blockquotes at every depth off the parse (`quotedPassages`),
under the strict grammar and under plain markdown alike,
and the guard reads both texts as the floor reads a page
(`readPageSkeleton`:
strict first,
plain markdown where it refuses).
A quote inside another quote counts,
so a winner turning a quoted reply into the letter's own words is refused,
as a top-level quote turned into a paragraph is refused by the floor;
that is a behaviour change rather than a repair,
since such a winner shipped before it.
A text neither grammar reads keeps the archive,
since a check that could not run has not shown the quotes survive.
A quote-loss refusal stores the counts it compared,
the reporter names them,
the cache reader recomputes a refusal without them,
and the sentence puts each count in its number.
`quoteBlockCount` is gone;
`topLevelBlocks` stays for the target-only run,
which splices text and so needs each block's text.
Cases:
`quote-preservation.unit.test.ts` for the counter and the sentence,
`translate-slice-guards.unit.test.ts` for a floor-valid winner dropping a quote from inside a container tag
(refused,
the counts stored)
and one keeping its quote on the line after a paragraph's
(shipped),
and `slice-cache-store.unit.test.ts` for the stored counts.
The translate records change,
so the cache-account audit before a launch takes this change beside B36 to B41.

Queued:
the floor itself still compares top-level blocks only,
so a candidate dropping a quote inside a container tag passes the floor and is refused only after the judges chose it,
keeping the whole archive,
where a floor finding would send it back to its translator first.
Moving the count into the floor changes every lane's floor,
so it lands after the replay over the pinned human archives and settled pages that every floor had before landing
shows what it would refuse there.
Landed as B110.

## B68: the container deficit counted blocks by blank-line split

Found by the fourteenth coverage batch,
reading the same file as B67.
`countBlocks` counted the blocks one side writes inside a container as runs of non-blank lines.
The package's rule for a question about what a container holds is to ask the parse the floor reads
(`mistake-prevention.md`,
"Structure read off the parse",
after B42),
and a blank-line split disagrees with it wherever the two texts write one block differently.
In the red cases,
a list written loose in the original and tight in the archive admitted an absent passage into a container the archive renders whole,
the same list tight in the original and loose in the archive refused one the archive lacks,
and a fenced block with a blank line inside admitted one as well.
Whether a pinned page has such a container is not measured:
none of the 56 settled artifacts in the worktree's run directories carries an admission finding,
nor the coverage finding every admission writes,
so the artifacts do not record them,
and the logs that print them are mixed with unit-test logs.

Fixed (`8ec62ce62` red,
`19eb118ab`):
`insertion-container-blocks.ts` reads each slice's side,
cut to the part inside the container,
with `readSliceSkeleton`,
and counts its top-level blocks.
A passage costs the blocks the container counted for its slice,
so the deficit and what it is spent on are one measure.
A half the parse refuses leaves the container uncounted,
admitting nothing,
with a finding naming the side,
the slice and the parser's detail
(`insertion-container-deficit-unread`);
both documents parse whole,
so that arm is not expected,
and it is a finding rather than a stop because a run always ships.
The count gives up one thing,
recorded as a choice:
a list the archive renders with two of its three items reads one block against one,
which is the roster's carried and partly-carried verdicts' business.
Counting by blank-line split again failed the three list and fence cases and nothing else.
The census after the fix (`census-VmhQQH`,
1,467 passes,
taken clean)
reads against `census-o4RZ7w`:
ran 0,
still cold 1,164,
cold since then 0,
not loaded 6,
and per-file counts compared between the two census files change nowhere;
both container modules are loaded and hold no cold stretch.

Closed in B100:
this entry left three readers open
(`displacement-ratio.ts` `contentBlockCount`,
`line-structure.ts` `isLineStructured`,
`translate-address-drop.ts` `blocksOf`)
and the target-only run;
a census found two more in the page passes,
and the fix found the suicide floor reading the same split.

Recurrence:
a count of what a passage holds reads the parse the floor reads,
and one measure serves both sides of a comparison and what the comparison is spent on.

## B100: six more readers counted blocks by blank-line split

Found 2026-10-01 (UTC) reading the readers B68 left open,
a census an agent took over the package's source
(`b68-open/report.md` in the audit's scratch folder),
with red cases in `2ab7849c7`
and the fix in the commit adding this entry.
B68 named four readers that split a passage on blank lines where the floor reads the parse;
the census found six,
and the fix found a seventh caller:

- `line-structure.ts` `isLineStructured` called five loose list items line-structured,
  where the floor reads one list;
- `target-only-run.ts` `splitTargetOnlyRun` protected nothing when a quoted transcript followed its anchor on the next line,
  since the split joined anchor and quote into one block that matched no source block,
  and it rejoined the blocks it kept with one blank line;
- `displacement-ratio.ts` `contentBlockCount` counted a fence with a blank line inside as two blocks and a loose list as three;
- `translate-address-drop.ts` `blocksOf` gave the two sides of a fence with a blank line inside different counts,
  so the address floor compared the texts whole and summed an address the block that switched did not hold;
  `translate-suicide-drop.ts` read the same split,
  so a paragraph and the published-work quotation on the line after it were one block,
  and the quotation's 自杀 was read as the passage's own;
- `corpus-run/prose-ranges.ts` closed a stray backtick on a code span past a heading,
  shielding prose and leaving the code span open to the page passes;
- `corpus-run/canadian-spelling-capital.ts` `opensSentence` took a capital opening a paragraph on the line after a heading for a name.

Each now reads the parse.
The slice readers read `parseSliceBody`,
the grammar the floor reads;
a passage it refuses is read whole by the address and suicide floors (`blocksOrWholeText`),
as an unequal count already was,
is not line-structured,
and has nothing protected by the target-only run.
The page passes read the page grammar through `proseBodyTree`:
comments and lone container tags masked,
then the strict grammar,
then plain markdown where it refuses,
as `parse-document.ts` reads a page.
Plain markdown refuses a body only where its nesting exhausts the parser's stack;
such a body reads as having no blocks,
so no code span or sentence start is read off it,
and any other failure propagates.
`splitTargetOnlyRun` now cuts the archive's own bytes at the anchor's end,
and `restoreTargetOnlyRun` appends the protected run as the archive wrote it,
so a retention and a restore both rebuild the archive byte for byte.
`contentBlockCount` throws `SliceBlockCountRefusalError` on a refused side rather than counting some other way;
only the displacement and window-trial probes count blocks,
since the contest size note now reads its ratio tails off character counts alone (`ratioImplausibility`),
and so parses no rendering.
`markdown-blocks.ts` `topLevelBlocks` lost its one caller and is removed;
`parsedTopLevelBlocks` gives each block's bytes,
kind and offsets.

A replay ran each reader's old build and new build over the same inputs
(`b68-fix/replay/b68-replay.mjs` in the audit's scratch folder):
the slices in 266 artifact files under 405 run directories
(three unreadable directories skipped,
6,037 usable rows),
and the 92 English pages beside their 92 Chinese pages at the pinned corpus commit,
with one synthetic control per reader,
each of which differed as built to.
Its first run found two faults in the fix,
both corrected before the figures here were taken:

- the parse masks a lone `<details>` or `</details>` to read the rest,
  so the target-only run's anchor skipped one and anchored a block early
  (`XingZ6012` slice 88);
  `parsedTopLevelBlocks` now lists each masked tag as a block of its own,
  in its place;
- on a candidate the slice grammar refuses,
  the address floor read the texts whole and its finding came before the parse floor's,
  standing in for the one that names the fault (`BI4PBV` slice 3);
  the address floor now leaves such a candidate to the parse floor.

The corrected replay:

- `isLineStructured` over 1,381 inputs:
  five answers moved,
  each from line-structured to not,
  three slices of one fidelity run losing HTML comments as blocks
  and two losing lone container tags
  (`XingZ6010` slice 76 and `XingZ6012` slice 88);
- `splitTargetOnlyRun` over 756 inputs:
  four splits moved
  (`Mio10` slices 15 and 16,
  `Mio7` slice 13
  and `XingZ6012` slice 88),
  each by the two line breaks before the protected run alone,
  and the judged part restored with the protected run rebuilt the archive in 756 of 756 inputs under both builds;
- `contentBlockCount` through `sliceSizeOf` over 3,677 inputs:
  183 block counts moved,
  read by the probes alone,
  and none threw;
- `droppedAddressFindings` over 18,541 inputs,
  `protectedRanges` over 3,678
  and `canadianizeText` over 18,633:
  no answer moved.

The full suite found a third fault,
which no replay input carried.
The Han residue floor reads every original,
page and candidate through `protectedRanges`,
which the old backtick scan let read any text;
on a page of 16,000 nested quotation markers
(`translate-floor-ground.unit.test.ts`)
plain markdown exhausted its stack inside `proseBodyTree`,
and the `RangeError` left the floor.
`parseMarkdownBody`'s TSDoc called plain markdown total;
it now names the overflow,
`requireMarkdownRefusal` (`parse-mdx.ts`) narrows a catch to it,
`proseBodyTree` reads such a body as having no blocks,
and `translate-skeleton-page.ts`,
which read any failure of the plain grammar as a refusal,
now reads that one alone.
`prose-ranges.unit.test.ts` carries the page,
beside the same code span one quote deep;
with `proseBodyTree`'s catch made to rethrow,
that case failed on the `RangeError`,
and the catch was then restored.
The suite also held three expectations the fix moved:
the protected run's leading line breaks (`archive-dispute-standing.unit.test.ts`),
the log line counting them
(`translate-slice.ts` now says the count takes in the line breaks before the run)
and the inventory of classes whose message is safe to forward,
which gains `SliceBlockCountRefusalError`.

No cache version moves:
the cache-account audit over the worktree's runs and the main checkout's
read 13,714 slice-cache records in 391 run directories,
the newest written 2026-09-27T04:26Z,
and each of the seven versions was set after it.

What enforces it:
`src/blank-line-splits.unit.test.ts` (moved on 2026-10-18 to
`package/translation-repair/source-scan/src/blank-line-splits.unit.test.ts`),
among the source scans,
fails on a production call to `split`,
`indexOf`,
`lastIndexOf`
or `includes` handed a string holding a blank line,
a constant the file declares with one,
or a regular expression matching two line feeds.
Its fixture case runs first.
With `folded.split('\n\n',)` planted in `isLineStructured`,
its package case failed naming that line alone (`line-structure.ts:108 split`),
and the line was then removed.
Out of its reach:
a reader that walks lines and treats an empty one as a boundary,
and a blank line built at run time.

Calls made here are open to veto:

- a passage the slice grammar refuses is read whole by the address and suicide floors,
  not line-structured,
  and protects nothing,
  rather than stopping the slice;
- the page passes read a body the strict grammar refuses as plain markdown;
- the protected run carries the line breaks between the anchor and the run;
- a paragraph or heading at any depth opens a sentence for the spelling pass;
- the probes' block count throws on a refused side rather than counting it another way;
- a candidate the slice grammar refuses is left to the parse floor by the address floor;
- HTML comments and lone container tags no longer count as blocks for line structure;
- lone container tags are blocks for the target-only run;
- a body neither grammar reads has no code span or sentence start read off it,
  its backticks prose to the page passes and the Han residue floor;
- plain markdown's refusal is a `RangeError`,
  and any other failure inside it propagates.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B110: the floor compared top-level blocks only, so a quote nested in a container could be dropped

Queued by B42,
red in `734f7da63`,
fixed in the commit adding this entry.
The translate floor (`validateTranslatedSlice`) and the archive-revision shape check (`revisionShapeFindings`)
compared only the top-level block sequence,
so a candidate keeping a container tag,
a list,
a footnote or a blockquote
while turning a quote nested inside it into prose passed both.
In the translate lane only the quote guard after judging caught it,
keeping the whole archive where a floor finding would have sent the candidate back to its translator first;
in the archive block review nothing caught it.

The floor now compares quoted passages at every depth,
read off the strict parse the floor already makes (`SliceSkeleton.quotedPassages`),
against the page as it stands,
or the original where there is none
(`quotedPassageFloorFindings` in `translate-validate-blocks.ts`),
beside the block comparison rather than inside it;
its finding states both counts and asks for every quoted passage to stay one.
A revision with the block's own shape is withheld when it carries fewer quoted passages than the block;
one of another shape is already withheld for its shape,
so it gets one finding,
not two.

Before landing,
a replay counted what the any-depth comparison refuses
over 1,277 pinned archive-against-original slices and 5,520 settled would-ship slices,
none unreadable.
Every slice it refuses there the block comparison already refuses:
5 against the original in the first population,
and in the second 3 against the original and 1 against the page,
each a top-level kind or count change.
No slice drops a nested quote under a matching top level,
and a positive control,
the one nested quote in `hakureico` slice 0 stripped of its markers in memory,
is caught.
So no pinned or settled verdict moves.

The cache versions do not move:
translate 15,
consolidation 20 and lane contest 6 were each set after the newest slice-cache record,
and the cache-account audit with the agent runs added finds the newest of 13,714 records still the one of 04:26 UTC on 2026-09-27,
so each account records the change as riding inside its number.
The patch that prepared this fix bumped all three,
against the accounts' own convention.

The quote guard after judging (`settleTranslateSlice`) stays.
A read of every path a winner takes to `stageResult.text` found each fresh candidate floored against the same `incumbentText`,
read by the same `readPageSkeleton` the guard reads,
so on a Markdown slice the guard no longer refuses anything the floor passed,
and the B42 case that reached it now settles on the archive through the floor
(`translate-slice-guards.unit.test.ts`).
Two paths still reach it:
a front-matter slice,
whose floor checks YAML and counts no quotes,
though no pinned front matter carries a line Markdown reads as a quote
(276 of 276 people pages read);
and a winner the invisible-variant fold changes after the floor read it,
which no measurement has looked at.

Open:
whether the quote guard should read front matter at all,
since the declared-name guard beside it skips it
(it should not,
and does not since B113);
whether the floor should read the folded text that ships rather than the text before the fold
(it should,
and does since B112);
and,
once both are settled,
whether the guard becomes a throwing invariant or keeps a case that reaches it
(neither:
B113 keeps it as a refusal that ships the archive,
which no settled slice reaches).
No case reaches its `refused-quote-loss` branch since this fix.

Reach:
both red cases fail at the build of `734f7da63` and pass here.
That red commit carried a TSDoc block on an `expect` statement in each of its two cases,
which the TSDoc attachment scan refused when the full suite first ran over this fix;
both are line comments here.
The red commit had been checked by lint and its two test files alone,
not by the source scans.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B113: the quote guard read front matter as Markdown

Found 2026-10-02 (UTC) while settling B110's open question on front matter,
red in `146039a2f`,
fixed in the commit adding this entry.
`settleTranslateSlice`'s quote guard counted quoted passages with the page parser on every slice it guarded,
front matter included,
while the declared-name guard beside it skipped front matter
and the front-matter floor checks YAML.
In YAML a line opening with `>` inside a block scalar is the scalar's content.
A probe against the build of `7314a8606`:
a page of front matter alone prepares into one front-matter slice;
`countQuotedPassages` reads a block scalar line `  > Meow` as one quoted passage on the archive's side
and none on a rendering that writes the motto as a plain scalar;
`validateTranslatedSlice` passes that rendering.
Driven through `settleTranslateSlice`,
the slice settled as `refused-quote-loss` and kept the whole archive.
No pinned verdict moves:
no pinned front matter carries such a line
(276 of 276 people pages read for B110).

The guard now reads Markdown slices only,
as the declared-name guard does.
With B110's any-depth count in the floor and B112's fold at intake,
no slice the pipeline settles reaches `refused-quote-loss` now.

Decided for quality and open to veto:
the refusal stays,
shipping the whole archive,
rather than becoming a throw,
since a throw would fail the entry where the owner's rule is that a run always ships;
no case reaches it,
so the coverage census reads that branch cold,
expected and recorded here.

The cache versions do not move:
the audit run for B112 found no slice-cache record newer than any version,
so the change rides inside translate 15.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse".

## B130: a code span after an escaped backtick left unprotected

Red in `b1fd99dc2`,
fixed in `52c5ca829`.
Found on 2026-10-05 (UTC) reading the T8 trial's cases in `prose-ranges.unit.test.ts`,
where an independent reviewer found one asserting an empty result for a bare double backtick,
a result that holds with or without the check the case named.
`constructEnd` (`corpus-run/prose-ranges.ts`) answered "no construct" for any backtick whose previous character was a backtick.
The check dates from `1873b23dc`,
when code spans were found by a hand scan.
They come off the parse now,
in `codeSpans`,
keyed by the starts the parse gives its spans,
and the one span whose opening backtick follows a backtick is the span after an escaped backtick,
which the check hid:
in a sentence holding an escaped backtick directly before a code span,
the parse reads the span at offsets 8 to 14 and `protectedRanges` returned none,
so a prose rewrite could reach inside it.
The red run printed 1 [PASS] and 2 [FAIL] lines,
the case failing with `expected [] to deeply equal [ { start: 8, end: 14 } ]`.

No page was harmed as far as the corpus shows:
at the pinned commit no Markdown file holds an escaped backtick directly before a backtick
(`rg` counted 0 files,
against 7 holding any backtick);
a model's wording could.

The fix:
the backtick arm asks the parse's spans alone.
The file's cases for a bare double backtick and for a fenced block return the same ranges with the check gone.
The trial's other four cases in the file asserted a length,
or a length of zero or more;
each asserts its whole range list now,
on inputs the reviewer's mutants tell apart
(a closing brace inside a line comment,
inside a template literal,
and inside a block comment that never closes).

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse" and "Guards that cannot fail".

## B157: a heading beside definitions the strict grammar refuses read as unparseable

Red in `49f3ebdbf`,
fixed in `699ad2f8c`.

Found on 2026-10-05 (UTC),
reading the MiMo trial's production diff once the trial was merged.
`inspectParagraph` (`inspect-paragraph.ts`) parses a paragraph twice:
alone,
to decide whether it is exactly one paragraph,
and with the document's definitions appended,
so that its references resolve.
The trial's T8 change (`f8cf67026`) cut the kind half out of the first check,
so that the second `not-one-paragraph` check would run,
and read the paragraph's kind off the definitions parse.
A single block that is no paragraph then reached the definitions parse:
on the build of `22afb6c89`,
a heading beside definitions the strict grammar refuses returned `unparseable`,
where the code at `28303c42a` returned `not-one-paragraph` before the definitions were parsed
(read off the source,
since no build of that commit is kept).
The function's own TSDoc says that question must not be influenced by anything appended to the paragraph.

The fix:
the alone check refuses again a text that is not one paragraph on its own,
as at `28303c42a`.
The second check is a throw whose message opens `unreachable:` and names the block the definitions parse led with:
the definitions are appended after a blank line,
which ends a paragraph,
so they cannot change its first block.
A probe ran forty texts that parse alone as one paragraph against twenty-six definitions shapes,
1040 pairs,
and moved no first block;
joined by one newline instead,
the same pairs moved 116,
setext headings and a table among them.

Recurrence:
`mistake-prevention.md`,
"Guards a census wants gone".

## B197: a settled census row counted its added blocks by another carve than the one that sliced it

Red in `6f61ba5dd`,
fixed in `72f5be638`.

Found on 2026-10-05 (UTC),
as two items ledger M113 left open.
`censusEntry` counted translation-only blocks with `alignBlocks` even under a settled recipe,
so a settled row was sliced by the recorded pairing and counted by the deterministic aligner.
A fixture whose pairing declines the last translation block sliced it out,
84 characters against a baseline of 125,
and counted no such block.
The same column read 39 for an added paragraph of 44 characters and for one of 227.

M113 recorded a cause for that reading off the source:
that `estimateExpansion` divides a section's whole translation by its whole original,
so an added block raises the length expected of every rendering.
Measured by the fixing agent,
the cause holds in part.
In a section of five blocks,
forcing the clean expansion changes which block is left over for an addition of 213 characters
and not for one of 42,
which is a near tie in length between neighbouring blocks;
in a section of one original block the column reads the original's own rendering,
39 characters,
whatever was added.

The fix:
the count reads the steps that sliced the section,
`blockPairingToSteps` where the recipe holds a pairing for it and `alignBlocks` otherwise,
and a block a pairing renders for an original is not counted.
Under a recorded pairing the column reads 44 and 227;
a deterministic row still reads the aligner's leftover.

Open to the owner's veto:
`estimateExpansion` and the aligner are unchanged.
Estimating the expansion from the whole entry recovered an invented added block more often when it was long
and less often when it was short
(the fixing agent's probe over 145 sections of the pin,
1,434 insertions at each size:
1,279 recovered against 1,190 at 213 characters,
and 1,231 against 1,265 at 42),
and it would move alignments on every page.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
a figure describing a carve is read off the steps that made the carve.

## B325: the slice parse welded the paragraphs a line holding only a mark parts

Red in `75ea14bff`,
fixed in `3ce947dce`.

Found on 2026-10-06 (UTC) while settling the one archive with marks for B324.
`parseDocument` blanks a line that shows nothing and is not blank to the parser before it parses,
and `parseSliceBody` did not,
so a slice holding such a line read as one block fewer than its page:
by the agent's measurement,
3,
2 and 3 blocks for the 4,
3 and 4 of three slices of `people/Toka_ls/page.en.md`.
The corpus pass folds the archive first,
so it did not reach production there;
the bench sampler slices raw text and does.

The fix:
`parseSliceBody` masks invisible lines first,
as the page parse does.

Open to the owner's veto:
a candidate line of a zero-width space alone is a paragraph break in the slice grammar.

Recurrence:
`mistake-prevention.md`,
"Structure read off the parse":
two parses of one text apply the same masks in the same order.
