# Audit ledger: building the consistency pass, repeat groups and the carry

The record of building the page-level consistency pass up to the independent review of its rebuild:
how the repeated wording is located,
how the bench chooses and the pass carries the choice to every place,
and what the first reviews found.
The fourth and fifth reviews are filed in
[Audit ledger: the consistency pass, the span and the marks beside it](audit-ledger-consistency-pass-span-and-marks-beside-it.md),
the sixth review and the apostrophe round in
[Audit ledger: the consistency pass, apostrophes and block floors](audit-ledger-consistency-pass-apostrophes-and-block-floors.md),
and the seventh review in
[Audit ledger: the consistency pass, the page-wide proof, forms, floors and limits](audit-ledger-consistency-pass-page-wide-proof-forms-floors-and-limits.md).
Findings of those reviews that belong to a named family are filed under that family.

Split on 2026-10-14 (UTC) out of `audit-ledger-tense-check-and-consistency-pass.md`,
which had reached 994 lines and held the tense check and the consistency pass in one file.
Entries keep their ids and their text,
except that B450 now names the read it was found in,
which the old file left to whichever entry stood before it.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B423: nothing located the repeated wording a page-level consistency pass would read

Red `0916175337`,
fixed in `7bd373eb3f`.

Found on 2026-10-11 (UTC) in the read of TianqiChen66622,
whose source repeats one refrain four times and one sentence twice,
and whose page renders the refrain four ways and the sentence two ways.
The owner asked for a page-level consistency pass,
and nothing in the package located the repeats it would read.

The fix:
`repeatGroupsOf` (`repeat-groups.ts`) reads the body paragraphs of a page through `parseDocument`
and returns maximal repeats of twelve or more characters
and refrain frames
(a shared prefix of four or more characters,
a shared suffix of three or more,
and a varying slot of one to eight),
each occurrence placed in the slice that holds it whole.
An occurrence no slice holds,
as a sealed original or a refused section leaves it,
is listed in the group's `unplaced` with its offsets and why;
the first draft threw `unreachable:` there,
and both shapes reach it.
A group with a quoted occurrence,
or with every occurrence on one line,
is left out.
Hardened on the way to the merge:
an opening quotation mark counts as inside the quote,
a stray closing mark no longer drives the depth below zero,
an HTML comment inside a paragraph is no wording,
and lengths count code points where the first draft counted UTF-16 units.

The `repeat-census` task (read-only,
spends no quota) reruns the design's line reading
beside the package's detection over the 92 pinned entries.
At `7bd373eb3f` the line reading finds 76 groups on 48 pages
(68 maximal,
8 refrain);
the package finds 6 groups on 5 pages with no occurrence unplaced,
the line reading's 11 body-only groups less the same-line and quoted ones;
the live page's four refrain lines and two sentence lines are both found.

Decided by the owner on 2026-10-11 (UTC):
quoted repeats stay out of the groups.
Kept by the lead under the owner's instruction of the same day:
occurrences no slice holds whole are listed rather than refused;
lengths and slot bounds count code points while the line reading keeps UTF-16 units;
a paragraph holding `{` or an inline tag is left out whole,
where the alternative cuts only the tag;
the census modules sit on the own-test allowlist;
`maximalRunsOf` is exported through the index as a test seam;
same-line repeats stay out of the groups,
as the design recommended.

## B426: the repeat reader tested raw regions the parse had already hidden

Red `cda53b4338`,
fixed in `566de2e511`.

Found on 2026-10-12 (UTC) by an independent review of B423.
The wording reader ran its markup test and its inline cuts on the raw paragraph,
so a brace or tag inside an HTML comment,
inline code or a bare URL dropped the whole paragraph,
though the parse reads none of them as an expression or tag;
a URL or backtick inside a comment opened a cut that ran past `-->` and ate the wording after it;
a comment inside a repeat ended the run;
and an edge paragraph of a container carried the container's tags into the markup test,
which 37 paragraph ranges of the pinned corpus hold.

The fix:
the reader blanks comments and container tags,
reads them as whitespace,
and runs the markup test with hidden ranges and inline cuts blanked.
The groups over the 92 pinned pages did not change.

## B428: a repeat's slice depended on the order of the slice list

Red `cda53b4338`,
fixed in `566de2e511`.

`placementOf` took the first slice in list order that held an occurrence's start,
so with overlapping slices it reported `crosses-slice-end` for an occurrence another slice held whole.
The order of groups that start at the same offset was fixed by construction but stated nowhere.

The fix:
an occurrence goes to the slice with the lowest `sliceIndex` among those holding it whole;
`compareFoundRepeats` (`repeat-groups-order.ts`) states the group order.

## B431: a repeated source line could ship rendered several ways on one page

Red `f59e887105`,
fixed in `c08beb9682`.

Found on 2026-10-11 (UTC) in the read of TianqiChen66622,
whose page rendered one refrain four ways and one sentence two ways;
the owner asked for a page-level consistency pass the same day.

The fix:
after the slices settle and the consolidation runs,
`runPassSettlement` (`corpus-run/pass-settlement.ts`) runs `runPassConsistency`.
For each repeat group (B423) whose places read differently,
it asks the wide bench one sheet,
reason before choice,
and counts the ballots in code (`consistency-count.ts`,
`consistency-majority.ts`).
A strict majority's quote,
located exactly once in each place's English,
or its refrain pattern carrying the place's own slot,
replaces the other places by code (`consistency-splice.ts`);
no model writes text.
A place the majority keeps for its context,
a tie,
no majority,
a gather below quorum
and an unlocated quote
leave the page as it stands,
recorded.
Each changed slice passes every floor with its current wording as incumbent,
a slice where the archive stands included,
or keeps its wording with a warning and a finding.
Any trouble inside the pass is contained and the page ships.
Artifact generation 15 carries the `consistency` section,
which joined the generation before any pass wrote one,
so the number did not move;
the reader recounts it from the ballots and refuses it in generation 14 and earlier.
The would-ship reader ranks the pass's text under the page assembly and over the polish.
The pass has its own cache namespace at `CONSISTENCY_CACHE_VERSION` 1.

Kept by the lead under the owner's instruction of 2026-10-11 to decide without asking:
the floors here are Markdown only,
since no repeat group holds front matter;
no second quoted-passage guard,
since the publication rule already refuses a lost quoted passage;
the earlier group wins an overlap,
though the groups are disjoint since B435.
Changed by the lead's decision,
in work still open when this entry was written:
the ten modules the pass's tests reached only through callers get tests of their own (B437);
`ConsistencyInvariantError` carries its message;
the consistency sheet shows the tense candidates (B419).

## B434: a repeat that carried quoted speech was reported when its run began in narration

Red `c5a5e78991`,
fixed in `36ada23b21`.

The owner answered on 2026-10-11 (UTC) to leave quoted repeats out,
yet `repeatGroupsOf` judged a run quoted only by where it began,
so a run that began in narration and carried speech was reported,
and the consistency pass (B431) could have rewritten part of a quotation.
Every character of every occurrence is now asked,
the marks themselves included,
for maximal repeats and refrains alike (`repeat-groups-quotation.ts`).
No group of the pinned corpus left.

## B435: repeat groups could claim overlapping text

Red `c5a5e78991`,
fixed in `36ada23b21`.

Periodic text,
a repeat nested in a longer one that fewer places share,
and a refrain frame over clauses a maximal repeat covers
each yielded groups whose occurrences overlap,
so the consistency pass would have spliced two renderings into one span.
`disjointRepeatsOf` (`repeat-groups-overlap.ts`) keeps the group covering the most wording code points,
drops any group whose occurrence shares a character with a kept one,
breaks ties by the stated group order,
and runs after the quoted and same-line filter and before placement.
The first draft weighed UTF-16 units and so counted text beyond the first plane double.
The pinned corpus held no overlapping group before or after.

## B450: a code splice could not carry the bench's choice

Found in the read of TianqiChen66622 that B431 records.
Even located,
the repeated sentence's first place reads "She also wanted",
and its "also" renders the `也` that stands before the repeat on source line 35 and not on line 55,
so splicing it into the second place would have added a word the source lacks there;
the word sits inside the sentence,
so no contiguous quote leaves it out.
The refrain's vote split 2 to 2 between two frames that put the slot on either side of "girl",
so no splice of one frame carries every place's own slot as English.
No floor catches an added word or a misplaced modifier.

Changed by the lead's decision under the owner's instruction of 2026-10-11 to decide without asking,
in work still open when this entry was written:
writers render each receiving place following the chosen rendering,
changing only what the place's own source requires,
and the bench chooses among the current English,
the code splice where it locates,
and the writers' candidates,
reason before choice,
with every floor on the winner;
a tie for first is run off among the tied finalists,
as the owner's rule of 2026-10-11 does for contests,
and a second tie leaves the page as it stands.
Rejected:
loosening the matcher alone,
which would have shipped the "also";
and a splice of the refrain's winning frame,
which would have shipped a misplaced slot.
Open to the owner's veto:
the run-off,
whose alternative is that a tie leaves the page.

Built in `6e5afaa41a` (red) and `3085924af4`,
with the page consistency phase leaning on the writer bench in `4ec390fe87` and `f8d47c38aa`;
the artifact moved to generation 16 and `CONSISTENCY_CACHE_VERSION` to 2.
A keyless replay over TianqiChen66623's stored ballots located every place:
the refrain's 2 to 2 tie goes to a run-off between the third and fourth places,
and the repeated sentence's first place wins 3 to 1.
The lead also kept,
open to veto:
the generation 16 shape,
which makes a pass refuse to resume into a directory of generation 15 artifacts
(the alternative,
an optional key in generation 15,
weakens the exact-key readers);
a separate outcome for the bench keeping the current English;
and the first round's strict majority for the run-off.

## B451 to B462: the independent review of the rebuild

Found on 2026-10-12 (UTC) by a reviewer that read every changed file,
ran twenty mutations and fuzzed six hundred benches;
it found no path where model output reaches the page without the bench and every floor,
and no failure that stops an entry.
Fixed on 2026-10-12 in `d9bcaf2787` (the cases) and `1a31a4a244` (the code);
`CONSISTENCY_CACHE_VERSION` moved to 3,
and generation 16 kept its number,
since no run has written it.
The fixer's replay of TianqiChen66623's ballots through the fixed build matched the replay before the fix byte for byte,
and every place that located before still locates at the same offset with word boundaries enforced.
Each finding:

- B451:
  two receiving places in one paragraph:
  a writer's rewrite of the paragraph replaced the first and dropped the second as `overlapped`,
  so the page shipped still inconsistent with no finding,
  and `overlapped` was defined as an earlier group's overlap.
  The writers now rewrite each block once for every receiving place in it.
- B452:
  quotes and slots matched as substrings
  ("so" inside "also"),
  so a slot could be written into the middle of a word;
  they now locate at word boundaries.
  A recurrence of "Words inside words".
- B453:
  a writer bench below its floor skipped the whole pass,
  under a line saying the entry stopped (M159);
  the carry now goes ahead without writers and records why.
- B454:
  the judges choosing among the writers' paragraphs were shown less than the writers,
  and neither was shown the chosen place's English slot words.
  A recurrence of "What each sheet is shown".
- B455:
  the fold was not idempotent,
  so a quote across a blank line located only when the caller had not folded it.
- B456:
  quotation marks and brackets at a quote's edges split the vote,
  as sentence punctuation no longer did.
- B457:
  a writer was given the whole top-level list,
  blockquote or `<details>` that held the span,
  and a span in no single block was not tried as a splice.
- B458:
  one catch charged every throw to the writer,
  including a floor's refusal of the source side.
  A recurrence of "What a catch charges".
- B459:
  the first-round ask copied the gather's cache protocol.
  A recurrence of "Copies of shared code".
- B460:
  sheets that forbid choosing by position named the first label in their reply example.
- B461:
  comments and TSDoc that still said the pass carries by code alone,
  defaults no read can reach,
  and no record of why a run-off was asked.
- B462:
  weak assertions in the new tests,
  and three mutations no unit test caught:
  touching spans unlocated,
  an overlap check that ignores the slice,
  and an abort swallowed in a group's settlement.
  Each mutation now fails a case of its own module's test.

The fixer left eight choices open:
a carry recorded under each label of a block rather than once with a list of labels;
words in no single block tried as a splice of the whole English rather than left uncarried;
edge marks taken off only when they enclose the quote or pair with nothing in it;
the writers' absence logged once as an error while the pass goes on;
a writer's block withheld when it loses another place's words;
a run-off's reason kept as a section finding;
a `<details>` paragraph given without its closing tag;
and blockquote markers kept in the block text.
An independent review of the fix weighs each before the next run.
The fixer also reported a semantic-bridge panic the lint prints for `consistency-edge-marks.ts`
and counts in no total;
its trigger was not found.
