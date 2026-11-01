# Roster calibration: the stub marker strip and the damage probe's precision, 2026-09-03

The owner's three answers and the stub marker traced to archive text no slice covers,
then stripped;
the stub-fix XIEPT2 page that landed at 15:28 UTC;
the damage probe measured at six true claims in ten,
so the contest is shown them as evidence and nothing obeys them.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## The owner's three answers, and the stub marker traced and stripped

Asked three separable questions,
the owner answered:
the `(To-Do)` question was the wrong question ("The
pipeline's job is to give a good result even when the originals are bad"),
inline gloss on first use for
terms with no English equivalent,
and about two hours accepted for XIEPT2-scale entries on Hyper alone.
Records:
`doc/decision/translation-repair-good-result-over-bad-original.md`,
`doc/decision/translation-repair-inline-gloss.md` (house policy vocabulary bullet),
rule `FT2` in
`AGENTS.md` (`e04882b47`).

Traced in the postscript run's artifact:
slice 0 is the front matter and slice 2 an insertion anchor
whose incumbent is empty,
so `(To-Do)` and the hint comment sat in archive text no slice covered and
`spliceSlices` passed them through byte for byte,
as it should for archive text.
Measured against the
pinned corpus (92 English pages):
one stub marker,
XIEPT2's;
HTML comments in 22 pages,
all vocabulary
and voice notes (起床战争：Bed Wars,
WER:
World Educational Robot Contest,
"This section is first-person
narration"),
which `entry-notes.ts` already turns into "ARCHIVE editor comment" lines of the identity
block.
So the comments are evidence the pipeline reads and a reader never sees;
only the marker goes.

Fixed at `3d27f9b35`:
`corpus-run/archive-stub.ts` strips a paragraph that is nothing but a stub token
(to-do,
todo,
tbd,
wip;
one layer of brackets;
any case) with one adjacent blank line,
outside front
matter,
comments and fences,
in one linear pass with comments masked first;
`passArchiveText` composes
the invisible-variant fold and the strip,
logs each removal,
and now runs inside `preparePassEntry` so
preparation,
both lanes,
the stored archive and the page describe one text (and the entry driver stays
under its line budget).
`verify-published` weighs against the artifact's stored archive,
so the chain is
consistent.
Unit tests cover the XIEPT2 shape byte for byte and the negatives (inside a comment,
a
fence,
front matter,
a sentence;
a marker at the document's end;
a marker as the whole body),
and the
positive case was shown to fail with the membership check neutralised.
XIEPT2 launched on that tree at
13:01 UTC (`~/temp/agent/xiept2-stub-20260903`) and logged `archive: stripped stub marker "(To-Do)" at
line 8` at once;
the landing is recorded when it settles.

### The stub-fix XIEPT2 landed, 2026-09-03 15:28 UTC

Settled in 146 minutes (13:01:02 to 15:28,
`ms=8782669`),
against 219 for the postscript run:
35 slices,
repair issues 6 with 3 accepted and 3 resolved,
98 repair findings,
3 repair changes,
27 translate
changes,
28 documents differ,
27 pages changed,
0 silent,
10 alignment findings,
selection contested.
`verify-published` reads 1 of 1 pages carrying every promised wording at the implied length.
The page
opens with the front matter,
a blank line,
and the translator-hint comment,
then the translated body;
`To-Do` occurs 0 times.
The death lines stay vague ("having ended her life",
the quoted 安乐死 rendered as
"euthanasia" in quotation marks as the original quotes it) and the consolidation slate rejected the one
candidate that carried the method,
six judges naming the reader-protection rule.

FIRST LIVE WITNESS OF BOTH DRY WITHHOLDINGS.
Synthetic read wet at the lanes reading (13:02:26,
`wide=7 select=7 late=8 slate=8 hyper-slow seated=yes`),
went dry at 13:15:51 (`syntheticWeekly=0%`),
and both later readings withheld as designed:
lane contest 14:09:45 and consolidation 14:12:38,
each
`synthetic=dry wide=6 select=5 late=7 slate=6 hyper-slow seated=no`.
Qwen3.8-27B was abandoned 38 times
against 110 on the postscript run,
Kimi-K3 32 against 51,
minimax-m3 20 against 15;
HTTP 429 lines
570 against 1468.
Hyper served 1770 calls and Synthetic 254;
the Hyper balance moved 3154 to 2500,
so
the run cost 654 hypercredits against 724 for the postscript run.

WHAT THIS DOES NOT SHOW:
the contest evidence block still has no live draw with a corroborated damage
claim on a contested slice;
the unit tests remain its only witness.

## The probe measured: six true claims in ten, so the contest is shown them and nothing obeys them

Asked with four options (contest sees the claims;
measure first;
rule-cited revert;
probe every
candidate),
the owner answered that the options are not mutually exclusive,
that this is not a design
decision,
and to decide and measure.
Recorded as rule `QNX`.
Both compatible things were done.

Measured:
every region with a corroborated added-damage claim across the artifacts that carried the
probe (XIEPT2 rerun5:
3 probed regions,
none corroborated;
Carena0442 landed:
41 probed,
6
corroborated;
keyword233 wrap landing:
4 probed,
2;
keyword233 smoke:
4 probed,
2),
each claim read
against the BEFORE and AFTER text and the ORIGINAL it quotes:

- True (6):
  Carena "at last" added to 我来重新补上;
  Carena "world-weary" added to 藐视地;
  Carena "only"
  added to 会引来厌倦;
  keyword233 wrap "is a transgender woman ... she runs ... she shares" (tense);
  keyword233 smoke "is a transgender woman" (tense,
  two probers);
  keyword233 smoke "where she shares"
  (tense,
  two probers).
- False (3):
  Carena 十几二十分钟 rendered "ten or twenty minutes" is closer than the BEFORE's "fifteen";
  Carena 我方才知道 read by two probers as 我方 "our side" when it is 我 方才 "I only then learned",
  so
  the edit to "I learned" was right;
  Carena 担意外 "bearing the unexpected" is a defensible reading of
  担.
- Borderline (1):
  keyword233 wrap "which I think is great",
  a narrator's aside in the present tense
  beside a past-tense passage.

Six of ten,
and the two-prober claims two of three,
is the precision of an audit,
not of a gate:
the
probe stays in shadow mode as `introduced-defect-probe.ts` argues,
and the claims cited the tense rule
in every true tense case.
What changed:
the lane contest is now shown the corroborated claims against
the repair candidate as evidence lines after both candidates ("evidence to weigh,
not a verdict",
with
the four-in-ten error rate stated),
the cache key folds them in so ballots bought without them never
answer for a question asked with them,
and nothing acts on a claim.
`repair-damage-evidence.ts`
builds the lines from the repair lane's probe reports;
the wire test proves the block appears with
claims and not without,
the key test that claims separate the question and an empty list keys as
before,
the evidence test that only corroborated claims become lines.
