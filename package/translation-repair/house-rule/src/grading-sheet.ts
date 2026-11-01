import {
  type GradingCandidate,
  SIZE_BANDS,
} from './sample-grading.ts';
import { flattenSpace, } from './sheet-line-text.ts';

//region Grading sheet rendering
// Renders a drawn sample into a human grading sheet: a header stating the
// precision bar and how to grade, per-band sampled counts, then one block per
// candidate carrying its zh source and en target quotes with an unfilled grade
// box. The sheet quotes UNLICENSED corpus text, so callers write it OUTSIDE
// the repo.

/**
 Joins a candidate's quotes for display, or a placeholder when the issue
 anchors nothing on that side.

 @param quotes - distinct quotes for one side

 @returns Display line for the side

 @example
 ```ts
 const line = quoteLine(['猫猫',],);
 ```
 */
function quoteLine(quotes: readonly string[],): string {
  if (quotes.length === 0)
    return '(none)';
  return quotes
    .map(function quoted(text,) {
      return `“${flattenSpace({ text, },)}”`;
    },)
    .join(' · ',);
}

/**
 Renders one candidate as a grading-sheet block with an unfilled grade box.

 @param candidate - the sampled candidate

 @param index - 1-based position in the sheet

 @returns Markdown lines for the candidate

 @example
 ```ts
 const block = renderCandidate({ candidate, index: 1, },);
 ```
 */
function renderCandidate(
  {
    candidate,
    index,
  }: {
    readonly candidate: GradingCandidate;
    readonly index: number;
  },
): string {
  return [
    `### ${String(index,)}. grade: [ ]  (Y = real defect · N = false positive)`,
    `- entry: ${candidate.entryId} · band: ${candidate.band}`,
    `- category: ${candidate.category} · severity: ${candidate.severity}`,
    `- claim: ${flattenSpace({ text: candidate.summary, },)}`,
    `- zh source: ${sourceLine(candidate,)}`,
    `- en target: ${quoteLine(candidate.targetQuotes,)}`,
  ].join('\n',);
}

/**
 Renders the source line, naming why there is no quote when there is none.
 A bare `(none)` made an ungradable item look like a rendering glitch: an
 addition correctly anchors to an empty insertion point, whereas a claim
 anchoring nothing at all asserts an absence while pointing at nothing, which
 a grader cannot check and should be told about.

 @param candidate - the sampled candidate

 @returns Display line for the original side

 @example
 ```ts
 const line = sourceLine(candidate,);
 ```
 */
function sourceLine(candidate: GradingCandidate,): string {
  if (candidate.sourceQuotes
    .length
    > 0)
    return quoteLine(candidate.sourceQuotes,);
  return candidate.sourceAnchor === 'insertion-point'
    ? '(insertion point: the claim anchors a position in the original, which has no text at it)'
    : '(NO source anchor: the claim points at nothing in the original, so nothing here can confirm it)';
}

/**
 Renders the full grading sheet: a header stating the precision bar and how
 to grade, the per-band sampled counts, then one block per candidate. The
 sheet quotes UNLICENSED corpus text, so callers write it OUTSIDE the repo.

 @param sample - the drawn candidates, in draw order

 @param seed - the seed the sample was drawn under, recorded for reproduction

 @param bar - precision bar the graded sample must clear

 @param corpusSha - pinned corpus commit the artifacts were produced against

 @param drawDigest - fingerprint binding this sheet to one exact draw

 @returns The grading sheet as markdown text

 @example
 ```ts
 const sheet = formatGradingSheet({
   sample,
   seed: DEFAULT_SAMPLE_SEED,
   bar: DEFAULT_PRECISION_BAR,
   corpusSha: 'a41fc60',
   drawDigest,
 },);
 ```
 */
export function formatGradingSheet(
  {
    sample,
    seed,
    bar,
    corpusSha,
    drawDigest,
  }: {
    readonly sample: readonly GradingCandidate[];
    readonly seed: string;
    readonly bar: number;
    readonly corpusSha: string;
    readonly drawDigest: string;
  },
): string {
  /**
   Sampled count per band, for the header summary.
   */
  const sampledPerBand = SIZE_BANDS.map(function countBand(band,) {
    return `${band} ${
      String(
        sample
          .filter(function inBand(candidate,) {
            return candidate.band === band;
          },)
          .length,
      )
    }`;
  },)
    .join(' · ',);

  /**
   Header lines: task, how to grade, the bar, seed, corpus pin, and counts.
   */
  const header = [
    '# Milestone 3 precision grading sheet',
    '',
    'Grade each accepted issue below: replace `[ ]` with `Y` if it is a real',
    'translation defect, or `N` if it is a false positive (no real defect, or',
    'anchored to the wrong text). Leave ambiguous ones blank and note why.',
    '',
    `Precision bar: ${String(bar,)} (accepted-issue precision must clear this).`,
    `Draw seed: ${seed}`,
    `Corpus pin: ${corpusSha}`,
    `Draw digest: ${drawDigest}`,
    `Sample size: ${String(sample.length,)} (${sampledPerBand})`,
    '',
    '---',
    '',
  ].join('\n',);

  return `${header}${
    sample
      .map(function renderAt(
        candidate,
        position,
      ) {
        return renderCandidate({
          candidate,
          index: position + 1,
        },);
      },)
      .join('\n\n',)
  }\n`;
}

//endregion Grading sheet rendering
