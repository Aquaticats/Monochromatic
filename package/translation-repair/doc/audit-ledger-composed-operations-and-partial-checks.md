# Audit ledger: composed operations and checks on a part

One refusal inside a composed operation that stopped the whole,
or left no finding,
and checks that vouched for a whole text on one part of it.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B25: the attestation checked a reference quote against every reference at once

Status:
fixed,
2026-09-29.
Found while fixing the attestation's quote fold (B24):
`verifiedAttestations` looked for an item's reference quote anywhere in the reference block,
while the sheet told voices an entry is discarded unless the quote is found "in the named reference".
So an item kept the number it answered even when that page does not state it,
the `- attested:` line every sheet carries then credited the wrong page,
and a quote running from the end of one reference line into the next verified,
since `compacted` removes the line break between them.
Only verified items were logged,
so a lost vote left no trace in a run log.

Measured with `~/temp/agent/audit-glossary-fix/b25-named-reference.mjs`,
which rebuilds each entry's reference lines from the lookup cache with `referenceLineOf`
and reads every `ATTESTED item` line of every run log:
106 verified items are logged,
25 could not be rebuilt from the cache,
and all 81 of the rest quote the reference they name.
The positive control names the next reference instead
and moves all 49 of the checked items on entries with two or more references.
So no stored result moves.

The fix reads the block one line per reference.
`reference-line-head.ts` owns the head both ways:
`referenceLineHead` writes it at both lookup sites,
and `numberedReferenceLines` reads the number back
and throws `ReferenceLineHeadError` on a line without one,
naming where the line sits and never what it says.
`reference-attest-verdict.ts` gives each item a verdict:
verified under the named reference,
relabelled to the first reference whose line states the quote,
or dropped with the side not found;
the stage logs every verdict.
Relabelling rather than dropping is a quality call under the owner's standing directive,
open to veto:
the quote was found word for word,
a lost vote can cost the detail its quorum,
and a detail that misses its quorum reaches no sheet,
which is the Mio20 loss the cited-reference rule was written against;
the confirmation round still asks the bench about every candidate.
The lookup now folds the endpoint's failure tag onto its line too,
so the block stays one line per reference.
The refusal found a fixture drift on its first run:
`reference-attest-confirm.unit.test.ts` wrote its reference as `REFERENCE 1 <url>`,
a form the lookup never writes.
The sheet's consequence clause now says what is kept
(word for word,
spacing and quotation-mark style aside,
the reference quote within one reference's line),
and its instructions stay strict.
Red `44e02fcda`,
fix `5bf2388ff`,
sheet `f2ddc237c`.

Left open,
and closed by B104:
a reference quote is looked for in the whole line,
head included,
so a quote of a page's address or title verifies as if the page stated it.
The body cannot be cut from the line reliably while a title may itself contain the separator,
so this waits on a line format that marks where the body starts.
B104 reads the title as the page's own,
as B28 already did,
so only the address needs cutting,
and an address holds no space.

## B80: archive revisions checked one at a time and composed into one page

Found by T8's eighteenth batch,
which left it open.
`repairArchiveBlocks` (`corpus-run/archive-block-repair.ts`) reviews each unclaimed archive block,
latest first,
and splices every revision it accepts into one page,
but it handed each review the archive as it came,
so the footnote check on a revision (`revisionFootnoteFindings`,
the class eighty-four floor) never saw the revisions beside it.
The check's own comment calls a footnote a relation between blocks.
Two blocks each carrying the one marker of a note,
each revised to drop it,
passed alone,
since the other block's marker still referenced the note,
and together shipped the note with nothing referencing it;
the page guard downstream reads the reviewed page as its archive,
so it inherits the defect rather than finding it.
The red case (`ffe8132b1`) asserted both premises,
each revision alone leaving the note referenced,
and the page shipped both revisions with no refusal logged.

Fixed (`5faa6f689`):
each block's review now takes the page the revisions already applied leave,
and since later blocks go first,
the block under review stands in it at its archive offsets.
The reviewers' context,
the quote style a revision is restored to
and the footnote check all read that page.
`introducedFootnoteFindings` counts defects by kind,
convention and identifier,
so a revision passes only into a page whose defects are a sub-multiset of the page before it,
and the reviewed page carries none the archive lacked.
In the red case the later block's revision stands
and each revision of the earlier block is refused.
The refusal now says the revision "gives the page a footnote defect it does not carry as it stands",
and its tail says the defect's "other end" stands outside the block,
since the end outside may be a note rather than a marker.
The block-outside refusal now says "not in the page it is reviewed in".

No stored reply moves with the changed prompts:
preparation caches only its pairing,
section-pairing and page-title rounds,
each keyed by its full content (`corpus-run/pass-prepare.ts`),
and nothing under `src` keys an archive review reply.
The other callers of `introducedFootnoteFindings`
(`assembly-integrity.ts`,
`assembly-structural-withdrawal.ts`,
`corpus-run/page-footnote-integrity.ts`)
each read a whole spliced page,
so the archive review was the one reading a part.

A correction on the way.
The red case first expected one refusal from every roster seat,
but the gather asks a window of quorum plus one seats
(`stage-quorum.ts`,
`fanOut`),
and three of the four were asked,
so the case would have failed with the fix in place.
It now records the seats asked about the earlier block and expects one refusal from each.
Before the red case,
`308bbe206` wrapped the file's two awaited suites in one root suite,
so a failing case in the first no longer keeps the second from running.

Guard-off on the fixed tree:
the review handed the archive as it came again failed the new case alone in its file,
and `git diff` printed nothing after the restore.
The full suite on `5695cd5be`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,506 PASS lines and no FAIL line.

Calls made here are open to veto:

- the quote style a revision is restored to follows the page as it will ship,
  not the archive as written;
  the two differ only where an earlier accepted revision removed the archive's only quotes of a style;
- where two blocks' revisions conflict,
  the later block's stands,
  since it is reviewed first;
  a pair that is sound only together,
  such as a note and its only marker both removed,
  stays refusable as before,
  each half refused for the defect it alone would leave;
- the reviewers see the page as revised so far,
  which is the page their block would join.

Recurrence:
a check guarding a relation between parts of a composed text
reads each part's change in the text the changes already accepted leave;
`mistake-prevention.md`,
"Checks on one part of a composed text".

## B81: an apparatus check that vouched for a block on one of its lines

Found by T8's eighteenth batch,
which left it open.
`isVerifiableEditorialArchiveBlock` (`archive-block-evidence.ts`) corroborates a reviewer's claim
that an unclaimed archive block is translation-side apparatus:
a contributor line,
a citation,
a picture,
a comment.
It accepted a block when any line declared contributors,
or when the block opened with a label,
a picture
or a comment it closed somewhere,
so prose beside the apparatus passed with it;
and it matched the "translated by" and "translation by" labels as raw prefixes,
so "Translation byproducts" passed as a label (the B23 family).

Measured first,
as the batch asked
(`b81-editorial-census.mjs` in the audit's scratch folder,
which prints entry ids,
block indices,
counts and masked shapes only):
over the 92 pinned archives' 2,446 top-level blocks,
the check accepted 40,
38 by a contributor line and 2 bare `Translation:` labels,
39 of them a single line,
and none carried anything outside its apparatus.
So the loose arms reached nothing in the pinned corpus.
Its verdict also never changes the text that ships
(inference from `archive-block-review-stage.ts`):
it decides whether an "editorial-context" retention counts as an anchored voice,
which moves the block's record between unresolved and retained after a naturalness review,
and whether that review is bought;
revisions come only from "revise" voices,
and a block either way stands as the archive wrote it.
The tightening is the check meaning what its TSDoc says,
"narrow",
for any later pin and for the record each block keeps.

The red case (`060fb6bf4`) holds six blocks the check should refuse and five it should keep;
a probe over the build (`b81-red-probe.mjs`) read all six as accepted.
`d6d72cbcf` first moved the contributor reader's line logic into `contributorDeclarationLines`,
so the check and the reader of declared names read the same lines,
with a test of its own;
exporting it took `pipeline-barrel.ts` over its line budget,
and the identity exports moved to `identity-barrel.ts`.
Fixed (`3a1577c42`):
with the block's comments blanked (`maskHtmlComments`;
one left open refuses the block),
each line a reader sees must be a declaring line,
a line opening with a label at word edges,
or a line Markdown reads as pictures and nothing else visible;
closed comments alone are still apparatus.
The fix first bounded the label's end with a test of its own,
against "Words inside words",
which sends a phrase looked for in prose through `word-bounds.ts`;
`382763ca5` reads it with `wordStarts` instead.
On both forms the census accepted the same 40 blocks
(a diff of each id list against the one taken before the fix printed nothing).

What a label line says after its label is not read,
and the TSDoc says so:
whether `Translator: Mittens. The cat won an award` names a person or tells of one is the reviewers' to judge,
and a structural reading cannot tell a sentence break from "Ms." without guessing;
among the blocks the check accepted,
no label line outside the contributor lines carries text after its label.

Guard-offs on the fixed tree,
one per build,
each read with the stage test file and the build probe,
each restored with `git diff` printing nothing after:
the label's end left open failed the longer-word block alone,
on the first form and again on `382763ca5`'s;
the picture line's "nothing besides" off failed the picture-then-prose block alone;
every line turned into any line failed the three multi-line prose blocks;
the open-comment refusal off failed the earlier case's open comment;
comment-only acceptance off failed the comment-only blocks in both cases;
and the reader's continuation off failed the continuation cases in the reader's tests and the check's.
The full suite on `3788e8223`
(`mise run //package/module/translation-repair:buildAndTest` (moved on 2026-10-18 to
`//package/translation-repair/<package>:buildAndTest`),
with the worktree clean)
printed 1,508 PASS lines and no FAIL line.

Calls made here are open to veto:

- the check is tightened though the pinned corpus reaches none of its loose arms,
  since its contract says narrow and a later pin could reach them;
- the label line's tail is left to the reviewers rather than to a sentence heuristic;
- `rendersAsNothing` decides a line a reader sees,
  so a line of invisible characters asks nothing of the block.

Recurrence:
a check that vouches for a whole block reads every part a reader sees,
never one part or the block's edges;
`mistake-prevention.md`,
"Checks that vouch for a whole block".

## B104: an attestation quote verified against a reference line's head

Found 2026-10-01 (UTC) reading B25's open item again,
red in `2b413912f`,
fixed in the commit adding this entry.
`verdictOf` (`reference-attest-verdict.ts`) looked for an item's reference quote in the whole reference line,
so a quote of the page's address,
of the head's own "reference 1",
or of the lookup's note on a page it could not fetch or read
verified as if the page had stated it.
B25 left this open because a title may hold any separator,
so the body cannot be cut from the line;
B28's `referencePageTexts` had since read a page's statement as everything after the address,
title included,
and an address holds no space,
so the cut needs no body marker.

`referenceLinePageText` (`reference-line-head.ts`) now reads one numbered line's page text,
empty for an unfetched or unreadable page,
and both readers go through it:
the attestation and the archive block review's `referencePageTexts`.
The red case builds its block with the lookup's own `referenceLineOf`
and drops the address,
head and both failure notes while keeping the title and the text;
a case on `referenceLinePageText` covers a head with nothing after it,
which the lookup never writes.

Measured before the fix
(`b25-head-only.mjs` in the audit's scratch folder,
rebuilding each line from the lookup cache as B25 did):
of 106 verified items the run logs record,
25 could not be rebuilt and all 81 others quote their named page's text,
none its head alone,
so no stored result moves.
With each quote replaced by its own page's address,
the positive control,
all 81 moved to the head alone.
No cache version moves:
the translate slice key folds in the attested lines
and the repair slice key the reference context,
so an attestation that keeps different items changes the key.

Guard shown to fail:
with `verdictOf` reading the whole line again,
the red case failed alone in its file,
and passed once the page text was restored.

Calls made here are open to veto:

- a page's title is part of what it states,
  as B28 already read it.

Recurrence:
`mistake-prevention.md`,
"Checks on one part of a composed text".

## B142: a refiner's rewrite the resolver dropped left no finding

Red in `9c2eb03d9`,
fixed in `e0151b92c`.

Noted by the independent review (report 4,
weak assertions) behind the trial's case for a rewrite naming paragraph 99;
the behaviour predates the trial.
`resolveRefineRewrites` (`refine-wire.ts`) records `refine-unknown-paragraph (<n>)` for a rewrite naming a paragraph the sheet never showed,
`refine-duplicate-paragraph (<n>)` for a second rewrite of one paragraph,
and each invisible variant it folds.
`runRefineStage` read the resolution's operations alone,
so those findings reached neither the stage's findings nor a log line,
and the stage reported `refine-candidates (1/1 heard, 0 proposing)`,
which is what it reports for a refiner that proposed nothing.
A reply rewriting one paragraph twice and naming a missing one shipped its first rewrite with no trace of the other two.
The editor lane keeps its resolver's findings (`buildEditorCandidates`,
`editor-candidates.ts`).

The fix:
the stage resolves each heard reply once and carries the resolver's findings into its own,
between the gather's and `refine-candidates`,
each credited to its rewriter as the editor lane's are,
in roster order,
and logged.
The consolidation polish round calls the same stage and gains the same findings.
Two cases pin it:
the dropped rewrite with the stage's whole result,
and the dropped rewrites beside one that ships.

Left as it was:
a rewrite the atom gate refuses still reaches a log line only,
since the gate's detail quotes atom values and a finding needs wording that carries none.

Cache:
no version moved;
only findings are added.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
a caller of a function that returns operations beside findings reads both,
or says why it drops the findings.

## B148: an OCR reader that wrote nothing ended the entry, and an asset name chose where its copy was written

Red in `e300e6c33`,
fixed in `d550bf04f`.

Three more edges of `readImageWithOcr` (`image-ocr.ts`) were found writing its cases.
A reader that exited without an error and wrote no transcript made the reading reject with `ENOENT`,
which `readImagePair` does not contain,
so the entry ended;
it is now `unavailable` with reason `ocr-failed` and a warning.
The scratch copy took the whole text after an asset name's last dot as its extension:
`../../etc/passwd` and `tabby.png/../../paw` pointed the write into a directory nobody made and rejected,
and a 300-letter extension rejected with `ENAMETOOLONG`;
the extension is now taken only when it is at most 16 ASCII letters and digits,
and the picture is read either way.
A decoder that is not installed was logged as `refused by Error` under the scratch path;
the line now names the asset and says the decoder is not installed.
Each has its red case in `image-ocr.unit.test.ts`.

Recurrence:
`mistake-prevention.md`,
"File names derived from an input"
for the extension,
and "Refusals inside a composed operation"
for the missing transcript.

## B162: a rewrite the atom gate refused left no finding

Red in `779c90b3f`,
fixed in `86f7a2efa`.

Left open by the same fix.
`runRefineStage` (`refine-stage.ts`) logged an atom-gate refusal at info and recorded nothing,
so a rewriter whose every rewrite the gate refused read in the findings as heard and not proposing,
`refine-candidates (1/1 heard, 0 proposing)`,
which is what the stage reports for a rewriter that proposed nothing.
The gate's detail quotes the atoms it compared,
such as `protected atom 1 changed (number:17 became number:18)`,
so it could not enter the findings as it stood.

The fix:
each refused operation leaves `<model>: refine-atom-gate-refused (paragraph <n>, <kind>)`,
the paragraph numbered as the rewriter's sheet numbers it,
as the resolver's findings are,
and the kind cut from the detail at its parenthetical,
as the editor lane groups its rejections by kind (`repair-editor-stage.ts`).
These go into the stage's findings in roster order,
between the resolver's findings and `refine-candidates`.
The detail stays in the log line alone.
The consolidation polish round calls the same stage and gains the same findings.
Two cases in `refine-stage.unit.test.ts` pin the whole result
for a dropped number and for a changed number in a second paragraph,
the second asserting that neither value reaches the findings.

Cache:
no version moved;
only findings are added.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
a refusal is a finding on the stage that refused,
worded without the values it compared.

## B184: a deterministic picture reader that rejected ended the entry

Red in `7f99edb6a`,
fixed in `17646f215`.

Found on 2026-10-05 (UTC)
by the agent that fixed B148,
reading the reading pair's contract.
`readImagePair` (`image-reading-pair.ts`) says every reader failure but an abort is contained,
and awaited `readOcr` with no `try`;
`readDocumentPictures` (`document-readings.ts`) awaited the pair with no `try`.
A deterministic reader that rejects,
for a scratch directory that cannot be made or a scratch device with no space,
ended the pair,
the picture stage and the entry,
for a reading nothing downstream requires.
The red run printed the reader's own error,
`Error: no space left on the scratch device`,
out of the pair and out of `readDocumentPictures`,
and an abort arriving while that reader ran came out as an `Error` where an `AbortError` was expected.

The fix:
`askDeterministicReader` (`image-ocr.ts`) answers a rejection with `unavailable` and the reason `ocr-failed`,
with a warning that names the picture,
so the models are asked as they are where the tools are not installed;
where the signal has aborted it rethrows first,
since a stop is not a failure to absorb.
`readDocumentPictures` is unchanged:
its `@throws` is true once the pair contains the rejection.

The whole suite ran on `17646f215` in a side checkout:
1,604 `[PASS]` lines and no `[FAIL]` line,
the lint finding nothing,
and the 35 source scans passing.

Open to the owner's veto:
every rejection of that reader but an abort is contained,
a machine fault included,
where the other way rethrows an error carrying a system code such as `ENOSPC` or `EACCES`;
the fixing agent judged an entry lost to one full device worse than one picture read without its gate.
The helper lives in `image-ocr.ts`,
with a type-only import cycle to `image-reading-pair.ts`,
on the agent's reasons:
the pair's file passed its line cap with the helper in it,
and a module of its own would have needed a test file of its own.

Open:
the new warning logs the caught value with `String(error)`,
as the reader's own failure line in the same file does,
where the package's rule for a logged failure is `refusalText`.
A census of every place a caught value is turned into text whole was launched on 2026-10-05 (UTC)
and had not handed back when this was written.
It has since:
red in `afee9dcda`,
fixed in `4c26a48a0`,
and recorded as B232 to B237.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation".

## B216: a rewrite the patch refused as unchanged left no finding

Red in `1e444543d`,
fixed in `2b0bfad75`.

Found on 2026-10-05 (UTC)
by the refine-lane review,
and left by the fix of B162.
`runRefineStage` (`refine-stage.ts`) applied each rewriter's operations and read only the ones the patch applied.
A rewrite that came out as the paragraph it replaces was refused by the patch as `unchanged-region` and dropped,
and the stage reported its rewriter as heard and not proposing,
which is what it reports for a rewriter that proposed nothing.

The fix:
each such refusal leaves `<model>: refine-unchanged-rewrite (paragraph <n>)` in the stage's findings,
the paragraph numbered as the rewriter's sheet numbers it,
in roster order,
between the atom gate's findings and `refine-candidates`.
The consolidation polish round calls the same stage and gains the same finding.
Any other refusal by the patch throws `unreachable:`,
naming the reason cut at its parenthetical so that no page text reaches the message:
the lane binds its operations to its own envelopes,
applies them to the text those envelopes came from,
and skips preservation.
The agent's probe ran 4,000 generated documents,
3,807 envelopes,
through the lane's own derivation and patch:
the unchanged region was the only refusal reached,
2,031 times,
and 1,038 documents applied an operation,
which shows the probe reached the patch.
The steps that read one reply moved to `refine-stage-replies.ts` (`resolveReply`,
`applyReply`,
`inRosterOrder`),
for the file-length cap,
with no comment dropped.

Cases:
two in `refine-stage.unit.test.ts`,
one of them an older case whose expectation changed,
both failing on the lead's red tree;
and seven in `refine-stage-replies.unit.test.ts`,
two of which reach the throw through a reply built by hand with a wrong hash and an unknown envelope,
input no run produces.
One guard in the new module has no case:
the refusal of an operation naming no paragraph of the sheet,
which the patch refuses first under another reason.

The same commit gives `refineProbeVerdict` a test file of its own,
eleven cases,
written green with no mutation run,
and takes its line out of the list of modules without one.

Cache:
no version moved;
findings are added only.

Open to the owner's veto:
the finding's form,
where the other way is the editor lane's count of rejections by kind;
and the throw for every other refusal.

Open,
seen by the agent and outside these three entries:
a paragraph preceded by a byte order mark got an envelope whose text begins with the mark and ends one character short
(`parse-document.ts`),
and whether two runs of one input can order a checker round's findings differently was not settled.
A follow-up was launched for each and had not handed back when this was written.
Both have since:
the byte order mark is B257 (`68df23b05`),
and the order of a checker round's findings is B253 (`df807a8b5`).

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation":
a caller of a function that returns what it applied beside what it refused reads both,
or throws on the refusals its construction rules out.

## B353: the precedence of a read failure and a parse failure in the bench draw was unpinned

Cases added in `d1e6af41c`,
coverage only,
passing before and after `ce2654b2c`.

Found on 2026-10-06 (UTC) by the review of the input-order change's second pass;
no defect.
`sliceEntry` (`corpus-run/bench-sample.ts`) reports the original's failure,
read or parse,
ahead of the English page's,
and no case mixed a read failure with a parse failure.
Two cases in `bench-sample-draw.unit.test.ts` now pair an unparsable original with a missing English page,
and an unreadable original with an unparsable English page.
By the agent's control,
swapping the two checks in `sliceEntry` turns both red,
with three older cases.

The skip line of the pair step has no case:
by the agent's bounded probe over three fixed seeds of 2,000 generated page pairs,
5,773 pairs passed the pair step,
2,363 of them cut into 7,326 slices,
and none reached that line;
227 pages built with broken front matter each printed their parse line,
which shows the probe reads every line the draw prints,
and nothing controls for the pair line itself.
`PAIR_STEP_FAILED`'s TSDoc records the probe.

Open to the owner's veto:
Hoisting two expected lines into `ORIGINAL_MISSING_LINE` and `ORIGINAL_UNPARSABLE_LINE`
changed the bodies of existing cases.

Recurrence:
`mistake-prevention.md`,
"Refusals inside a composed operation".

## B803: the composite of disjoint archive revisions refused every revision that added or removed a line

Status:
fixed on 2026-10-21 (UTC),
red `b052d7f8dc`,
fix `5b2f83dde0`.
Found in the read of `zheermao22` (trace K4).
**This replaces the line-count refusal that B712 recorded as a decision open to the owner's veto**,
and the owner may keep it or veto it as B712 allowed.
No cache version and no artifact schema version moved.

What the run showed:
`compositeOfDisjointRevisions` matched a revision to the block line for line,
and built nothing when any revision that changed the block had another line count.
It was tried three times in the pass and built nothing three times.
At 21:10:56 UTC the email (`block/20`) drew five revisions,
and the log said `candidate 2 has 23 lines against the block's 24 lines`.
At 21:15:30 UTC the email's label (`block/19`) said `2 revisions overlap on line 1 (candidates 1 and 2), of the block's 1 line`.
At 21:21:58 UTC the letter (`block/14`) said `candidate 1 has 34 lines against the block's 36 lines`.

What the refusal did and did not decide:
a refused composite is not a fallback.
The composite is one more candidate beside the reviewers' whole revisions,
and a refusal only leaves it off the slate.
The shipped text was chosen afterwards by the judges' ballots over whole texts:
the letter by `candidate 1 from glm-5.3 won weight 3.5 (ballots: 5)` at 21:25:09 UTC,
the email by `candidate 5 from deepseek-v4.1-flash won weight 3 (ballots: 5)` at 21:14:09 UTC,
and the label by `candidate 2 from hf:Qwen/Qwen3.8-27B won weight 3 (ballots: 6)` at 21:15:51 UTC.
So a fix to the composite can put one more candidate before the judges and cannot make them choose it.
The letter that shipped was the 34-line revision of `glm-5.3` itself,
which fixed five hunks and left the rest.
The composite the refusal withheld was of that revision and the 36-line revision of `inception/mercury-2.5`,
and under the new rule those two collide (see the replay).

The fix:
`hunksBetween` (`archive/src/archive-composite-hunks.ts`) reads a revision as the hunks that turn the block into it,
by a longest-common-subsequence line diff;
`hunkCollisions` finds the block lines two revisions both replace and the gaps an insertion meets;
`spliceHunks` (`archive/src/archive-composite-splice.ts`) applies the hunks of several revisions together.
The package held no line diff to reuse.
`assembly-structural-withdrawal.ts` cuts a common prefix and suffix in characters on pages,
and `document-structure/src/align-blocks-walk.ts` is a Needleman-Wunsch walk over the parsed blocks of two languages
that pairs by a similarity score (`scorePairing`) and pays a gap penalty to leave a block unpartnered,
which is not a diff of equal lines.
Revisions whose hunks collide keep the outcome of B712 for the whole block:
no composite,
and the log line names the lines or the place and says that revisions that overlap are not combined.
The gates are unchanged:
shape,
footnotes,
and the combined text showing a reader nothing.
`CompositeContribution.changed` now numbers the composite's lines,
which the judges' sheet reads as `coversLines`,
and the note on that sheet (`COMPOSITE_ASSESSMENTS_NOTE`,
`archive-block-selection-evidence.ts`) and the TSDoc of
`coversLines` now say what a removal is credited with.
The red commit's case for a hunk that removes a line and adds one before the last line expected the log to say
"before line 4",
and the fix commit corrects it to "after line 3",
since an insertion is worded by the line before its gap (and "before line 1" at the start).

Bound:
after the common prefix and suffix are cut the table holds at most `ALIGNMENT_CELL_LIMIT` cells (4,000,000,
each 4 bytes),
so time and space are bounded by that figure whatever a reviewer writes.
A stretch that would need more is one hunk from the first differing line to the last,
which collides with more and never changes a revision's text.
A block of 36 lines against a revision of 36 lines fills at most 1,369 cells.
The test file aligns 1,500 lines all unlike their revision (inside the limit) and cuts 2,100 (outside it).

Replay over the three refused attempts,
keyless,
from the stored replies in the run directory of `zheermao22` and the pinned page
(each block's text hashes to the `textDigest` the log carries for it):
none of the three builds under the new rule.
- The email:
  five admissible revisions,
  and every pair collides.
  The least colliding pair,
  the revisions of `gemma-4-26b-a4b-it` and `minimax-m3`,
  share only line 15.
  The shipped email came from one whole revision and still would.
- The email's label:
  the same overlap on line 1 of 1.
- The letter:
  the slate of two revisions (the third was withheld by the shape floor,
  with the same finding the log printed),
  the 34-line revision with hunks at lines 3 to 4,
  7,
  11 to 13,
  23 and 26 to 28,
  and the 36-line revision with one hunk at lines 6 to 8.
  They collide on line 7,
  where the first writes "guilty" and the second "cat ...
  unforgivable".
  Their only collision is the one line,
  and the second revision has no other hunk,
  so even a region-level merge would add nothing to the first revision's text.

So the narrowness fixed here did not decide the letter of this run,
and the letter would have shipped the same text under the new rule.
The new rule decides blocks where the revisions that add or remove lines are disjoint from the others,
which the tests cover and the corpus has not yet shown.

Decisions taken in this session,
open to the owner's veto:
- Overlapping hunks keep the B712 outcome for the whole block.
  A composite from the hunks of the revisions that do not collide,
  leaving the colliding region to one of them,
  would need a rule for which one,
  and B712 refused to make the shipped text depend on who answered first.
- A removal of the whole block (an empty replacement) builds no composite.
  It is a verdict on the block and edits none of its lines.
- Hunks that only touch combine.
  A hunk that ends where another begins,
  and an insertion at the edge of a hunk,
  share no block line and no gap inside either.
  Two insertions at one gap,
  and an insertion inside lines another revision replaces or removes,
  collide,
  since the order of two added lines is undetermined and a gap inside a cut has no place.
- The alignment is deterministic and says which of two identical lines a deletion is attributed to:
  lines both texts start with are kept first,
  and a tie between deleting and inserting deletes first.
  A paragraph removed from between blank quote lines loses the blank line after it.
- A reviewer who only removed lines is credited,
  in `coversLines`,
  with the line that now follows the cut,
  or the last line when the cut is at the end.
  The note on the select sheet says so:
  it used to say the lines that reviewer changed,
  which would have been false for a line the reviewer did not write.
  Crediting a removal with nothing would have left its assessment covering no line.
  The wording change moves the prompt digest of the select rounds that hold a composite and no other,
  and no cache version.
- A line is still the text through its line ending,
  so a revision that adds a line after a last line with no ending replaces that last line,
  and a trailing newline is still a change to the last line.

Not measured:
whether the judges choose a composite of revisions that add and remove lines,
and whether such a composite reads coherently across lines;
both need a run.
The replay's slate order for the email follows what the log shows
(candidate 1 has 24 lines,
candidate 2 has 23,
candidate 5 is `deepseek-v4.1-flash`),
and the order of the middle three is a guess that changes no collision.
The `glm-5.3` and `hf:moonshotai/Kimi-K3` attribution on the label's identical pair is from the reply files' provider
and token counts and is a guess too.

Cache account:
none moves.
The archive block review is not cached and its result is stored under no key
(`page-check/src/corpus-run/archive-block-repair.ts` opens no cache),
as B714 recorded.
The stored decisions that read its replies are the prompt payloads,
keyed by the model-plus-message digest:
a review round's own prompt is unchanged,
and a composite that is newly built joins the selection sheet,
so the sheet's digest moves for exactly the blocks whose slate changed.
The archive text a pass ships is hashed by `repairSliceKey` as `targetText` and by the pairing question key as its target
blocks,
so those keys move for exactly the entries whose archive block changed.
`mise run //package/translation-repair/slice-cache:cache-account-audit` on the tree at `5b2f83dde0` read eight versions
(`CONSOLIDATE_CACHE_VERSION` 29,
`CONSISTENCY_CACHE_VERSION` 6,
`LANE_CONTEST_CACHE_VERSION` 14,
`REFINE_CACHE_VERSION` 12,
`SLICE_CACHE_VERSION` 41,
`PAGE_TITLE_CACHE_VERSION` 4,
`PAIRING_CACHE_VERSION` 3,
`TRANSLATE_SLICE_CACHE_VERSION` 23),
listed the fix commit as riding inside all eight,
and found no slice-cache record in the worktree's runs directory (0),
which is no evidence either way in a keyless worktree.
The reasons none moves are the ones from the source in this entry.

Recurrence:
`mistake-prevention-sheets-quorums-and-outcome-rules.md`,
"Refusals inside a composed operation".
