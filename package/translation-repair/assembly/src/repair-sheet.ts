import { fenceForMarkdown, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type RepairDisposition,
  SHIPPED_DISPOSITION,
} from './repair-record.ts';
import {
  flattenSpace,
  type GradableRepair,
  type GradableRepairRegion,
  type GradingCandidate,
} from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Repair grading sheet
// The SECOND sheet: whether the text the pipeline wrote actually fixes the
// defect, graded apart from whether the defect was real.
//
// A separate sheet, not a second box on the detection sheet, and that is a
// measurement decision rather than a layout one. Showing a grader the proposed
// correction makes the alleged defect more salient, which moves the answer to
// "is this a real defect". Round two's precision was measured by a sheet that
// showed no repair, so folding repair text into round three's sheet would
// compare two rounds through two different instruments and quietly credit the
// change of instrument to the pipeline. The detection sheet is therefore left
// exactly as it was, and this one is graded AFTER it.
//
// Grading order matters and the header says so. An item graded N for detection
// has no meaningful repair grade: there was no defect to fix, so "did it fix
// it" has no answer, and a harmful edit to a false positive is a different
// question that this sheet does not ask.
//
// The sheet also never shows the checker verdict. Checkers confirmed 98.1% of
// repairs in the round-two artifacts, so revealing that would anchor the human
// toward agreement on precisely the population the human is there to audit.


/**
 Plain-language reading of each disposition, so a grader is told why an item
 carries no grade box instead of finding one missing. A map, since the
 disposition is the artifact's text: a plain object answered one spelled
 `constructor` with the inherited function and printed it as the note
 (ledger B77).

 BUILT FROM A RECORD KEYED BY `RepairDisposition`, so a disposition the
 pipeline gains cannot reach a sheet unexplained: the compiler refuses the
 record until the new disposition has its note. The map keeps the lookup by
 the artifact's own text, which the record would answer for `constructor`.
 */
const DISPOSITION_NOTES: ReadonlyMap<string, string> = new Map(Object.entries({
  shipped: 'a targeted repair reached the returned translation',
  'not-selected': 'a repair was written but the unchanged text won its slice, so nothing reached the reader',
  withdrawn: 'a repair was written but the whole page was blocked as non-translation, so the original was returned',
  'no-region': 'no targeted repair exists: the issue anchored no editable region, or no proposed edit survived the apply gate',
} satisfies Readonly<Record<RepairDisposition, string>>,),);

/**
 Renders one replaced region, disclosing when the same edit serves other
 accepted issues so a shared replacement is never read as this issue's own.

 A replacement that shows a reader nothing is told as the deletion it is to
 the grader, asked of `rendersAsNothing` (ledger B40): asked of an
 empty-string comparison, an editor's replacement of spaces or of one Hangul
 filler was fenced as written text the grader would see as an empty box.

 @param region - replaced region

 @param issueId - issue this sheet item is about

 @returns Markdown lines for the region
 */
function renderRegion(
  {
    region,
    issueId,
    positionByIssueId,
  }: {
    readonly region: GradableRepairRegion;
    readonly issueId: string;
    readonly positionByIssueId: ReadonlyMap<string, number>;
  },
): string {
  /**
   Other accepted issues the same replacement was cut for.
   */
  const siblings = region.issueIds
    .filter(function isOther(candidate,) {
      return candidate !== issueId;
    },);

  /**
   Sheet positions of the siblings that were also drawn, ascending.

   Positions rather than issue ids, because an id is a 64-character hash a
   grader cannot look up, and five of them on one line is a third of a
   kilobyte of noise obscuring the one fact that matters: this same edit is
   about to be shown again under other items.
   */
  const onSheet = siblings.flatMap(function toPosition(candidate,) {
    /**
     Sibling's position, absent when it was not drawn into this sample.
     */
    const position = positionByIssueId.get(candidate,);
    return position === undefined ? [] : [position,];
  },)
    .toSorted(function ascending(
      left,
      right,
    ) {
      return left - right;
    },);

  return [
    ...(region.before === ''
      ? ['- before: (nothing; this is an insertion point, a real place in the translation with no text at it)',]
      : [
        '- before, the text that was replaced:',
        fenceForMarkdown({ text: region.before, },),
      ]),
    ...(rendersAsNothing({ text: region.editorAfter, },)
      ? ['- after: (nothing; the text above was DELETED rather than rewritten)',]
      : [
        '- after, what the pipeline wrote:',
        fenceForMarkdown({ text: region.editorAfter, },),
      ]),
    ...(siblings.length === 0
      ? []
      : [
        `- SHARED: this one edit was also written for ${
          String(siblings.length,)
        } other accepted issue(s)${
          onSheet.length === 0
            ? ', none of which were drawn into this sample'
            : `, of which ${
              onSheet.length === siblings.length ? 'all' : String(onSheet.length,)
            } appear here as item(s) ${onSheet.join(', ',)}`
        }. The same before and after text repeats under those items; judge it against THIS item's claim only.`,
      ]),
  ].join('\n',);
}

/**
 Renders the repair evidence and, when a repair actually shipped, its grade
 box.

 @param repair - repair provenance for this issue

 @param issueId - issue this sheet item is about

 @returns Markdown lines for the repair block
 */
function renderRepair(
  {
    repair,
    issueId,
    positionByIssueId,
  }: {
    readonly repair: GradableRepair;
    readonly issueId: string;
    readonly positionByIssueId: ReadonlyMap<string, number>;
  },
): string {
  /**
   Why this item does or does not carry a grade box.
   */
  const note = DISPOSITION_NOTES.get(repair.disposition,)
    ?? 'unrecognized disposition; report this rather than grading it';

  /**
   Whether this item is gradable at all, which also gates the refinement
   caveat: telling a grader to judge the final wording of a repair that never
   reached the reader, immediately before telling them not to grade it, is a
   contradiction they would have to resolve on their own.
   */
  const gradable = repair.disposition === SHIPPED_DISPOSITION;

  return [
    `- outcome: ${repair.disposition} (${note})`,
    ...repair.regions
      .map(function renderOne(region,) {
        return renderRegion({
          region,
          issueId,
          positionByIssueId,
        },);
      },),
    // The naturalness lane rewrites whole slices AFTER the accuracy stage, and
    // it does so whether or not that stage's patch was selected. So a rewrite
    // can reach the reader for an issue whose targeted repair did not, and
    // saying only "nothing reached the reader" would be true of the repair and
    // false of the text. The final wording is shown wherever the document
    // carries it; only the grade box depends on whether a targeted repair
    // shipped.
    //
    // Keyed on the wording being RECORDED rather than on refinement alone. A
    // rewritten slice can still be taken back whole, by the assembly guard
    // keeping a footnote relation intact or by a run returning its input, and
    // the record then carries no final wording because the document carries the
    // archive's. Showing an empty fence there would tell a grader the slice came
    // back blank.
    ...(repair.refined
      ? (repair.finalSliceText === undefined
        ? ['- NOTE: a later naturalness pass rewrote this slice, but that rewrite was taken back before the document was returned, so the text above is what the reader saw.',]
        : [
          gradable
            ? '- NOTE: a later naturalness pass rewrote this slice, so the wording above is not final.'
            : '- NOTE: no targeted repair shipped, but a later naturalness pass rewrote this slice anyway, so the returned text is not the original either.',
          '- the slice as actually returned:',
          fenceForMarkdown({ text: repair.finalSliceText, },),
          ...(gradable
            ? ['- grade the RETURNED wording, using the edit above only to see what was attempted.',]
            : []),
        ])
      : []),
    ...(gradable
      ? ['- repair grade: [ ]  (Y = fully fixes this defect and breaks nothing nearby · N = it does not)',]
      : ['- not graded: no targeted repair reached the reader, which counts against coverage, not against repair quality',]),
  ].join('\n',);
}

/**
 Joins quotes onto one line, or says why there are none.

 Quotes are short anchored spans rather than whole slices, so they stay inline
 rather than fenced; newlines are flattened so one quote cannot break the
 bullet it sits in.

 @param quotes - distinct quotes for one side

 @returns Display line

 @example
 ```ts
 const line = quoteList({ quotes: candidate.sourceQuotes, },);
 ```
 */
function quoteList(
  { quotes, }: { readonly quotes: readonly string[]; },
): string {
  if (quotes.length === 0)
    return '(nothing quoted on this side)';
  return quotes
    .map(function quoted(text,) {
      return `“${flattenSpace({ text, },)}”`;
    },)
    .join(' · ',);
}

/**
 Renders one candidate as a repair-sheet block.

 @param candidate - sampled candidate

 @param index - 1-based position, matching the detection sheet exactly

 @returns Markdown lines for the candidate
 */
function renderCandidate(
  {
    candidate,
    index,
    positionByIssueId,
  }: {
    readonly candidate: GradingCandidate;
    readonly index: number;
    readonly positionByIssueId: ReadonlyMap<string, number>;
  },
): string {
  /**
   Repair block, or the reason this run cannot answer the question at all.
   */
  const body = candidate.repair === undefined
    ? [
      '- NOT GRADABLE: this run predates repair recording, so nothing states what was written.',
      '- leave this item blank; it belongs in no repair denominator.',
    ]
      .join('\n',)
    : renderRepair({
      repair: candidate.repair,
      issueId: candidate.issueId,
      positionByIssueId,
    },);

  return [
    `### ${String(index,)}. ${candidate.entryId} · ${candidate.band}`,
    `- claim: ${flattenSpace({ text: candidate.summary, },)}`,
    // The original is what "does it fix it" is answered against, so the zh
    // evidence belongs on this sheet too rather than only on the detection
    // sheet the grader has by then set aside.
    `- zh original says: ${quoteList({ quotes: candidate.sourceQuotes, },)}`,
    `- original translation said: ${
      quoteList({ quotes: candidate.targetQuotes, },)
    }`,
    body,
  ].join('\n',);
}

/**
 Renders the repair grading sheet: the same sample in the same order as the
 detection sheet, graded on whether the pipeline's text fixes the defect. The
 sheet quotes UNLICENSED corpus text, so callers write it OUTSIDE the repo.

 @param sample - drawn candidates, in draw order

 @param seed - seed the sample was drawn under, recorded for reproduction

 @param corpusSha - pinned corpus commit artifacts were produced against

 @param drawDigest - fingerprint binding this sheet to one exact draw

 @returns Repair grading sheet as markdown text

 @example
 ```ts
 const sheet = formatRepairSheet({
   sample,
   seed: DEFAULT_SAMPLE_SEED,
   corpusSha: 'a41fc60',
   drawDigest,
 },);
 ```
 */
export function formatRepairSheet(
  {
    sample,
    seed,
    corpusSha,
    drawDigest,
  }: {
    readonly sample: readonly GradingCandidate[];
    readonly seed: string;
    readonly corpusSha: string;
    readonly drawDigest: string;
  },
): string {
  /**
   Header stating the grading order, what the grade means, and the pin.
   */
  const header = [
    '# Repair grading sheet',
    '',
    'GRADE THE DETECTION SHEET FIRST, and do not read this one until it is done.',
    'Seeing a correction makes an alleged defect look more real, which would move',
    'the detection grades and break comparison with earlier rounds.',
    '',
    'Then, for every item you graded `Y` there (a real defect), grade below whether',
    'the text the pipeline produced actually fixes it. `Y` means the returned wording',
    'fully resolves this defect and introduces no new error nearby; `N` means it does',
    'not. Leave items you graded `N` for detection blank: there was no defect to fix.',
    '',
    'Item numbers match the detection sheet exactly.',
    '',
    `Draw seed: ${seed}`,
    `Corpus pin: ${corpusSha}`,
    `Draw digest: ${drawDigest}`,
    `Sample size: ${String(sample.length,)}`,
    '',
    '---',
    '',
  ].join('\n',);

  /**
   Sheet position of every drawn issue, so a shared edit can name the other
   ITEMS a grader will meet it under instead of quoting hashes at them.
   */
  const positionByIssueId = new Map(sample.map(function toEntry(
    candidate,
    position,
  ) {
    return [
      candidate.issueId,
      position + 1,
    ] as const;
  },),);

  return `${header}${
    sample
      .map(function renderAt(
        candidate,
        position,
      ) {
        return renderCandidate({
          candidate,
          index: position + 1,
          positionByIssueId,
        },);
      },)
      .join('\n\n',)
  }\n`;
}

//endregion Repair grading sheet
