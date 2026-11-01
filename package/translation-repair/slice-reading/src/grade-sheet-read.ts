import {
  opensWithVerdict,
  trimLeadingDelimiters,
} from './verdict-letter.ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Graded sheet reading
// Reading a human's filled-in detection sheet back into verdicts.
//
// Built for calibration: the plan of record is to pre-grade a round BLIND, hand
// the untouched sheet to the human, and derive an agreement rate from the two
// sets of grades afterwards. The pre-grades deliberately never appear ON the
// sheet. Printing them there would anchor the human toward agreeing, and the
// precision number that same sheet produces is the milestone gate, so the
// calibration would be bought by corrupting the measurement it calibrates
// against. Nothing is lost by keeping them apart, because the agreed plan only
// starts filtering items a round LATER anyway.
//
// The parsing rules come from the two sheets already graded, not from a format
// anyone specified. They differ from each other and both must be read:
//
//   ### 3. grade: Y  (Y = real defect ...)          round one, bare
//   ### 2. grade: N. <rationale>  (Y = ...)         round one, verdict then prose
//   ### 4. grade: [Y]  (Y = ...)                    round two, bracketed
//   ### 7. grade: [Y, <rationale>]  (Y = ...)       round two, bracketed with prose
//   ### 10. grade: [Not enough context to grade]    neither verdict
//
// That last shape is why a verdict letter is only a verdict when a delimiter
// follows it. "Not enough context to grade" begins with N, and reading it as a
// false positive would silently move a real measurement into the denominator on
// the strength of one letter.

/**
 What a human decided about one sampled issue.

 @example
 ```ts
 const verdict: GradeVerdict = 'real-defect';
 ```
 */
export type GradeVerdict =
  /**
   Graded `Y`: a real translation defect.
   */
  | 'real-defect'
  /**
   Graded `N`: a false positive.
   */
    | 'false-positive'
  /**
   Left blank, or answered with something that is not a verdict. Kept as its
   own state rather than folded into either: an item the grader declined to
   score belongs in no precision denominator.
   */
    | 'unscored'
  /**
   Answered `Duplicate`: this item is the same underlying defect as an earlier
   item in the same sample.

   Separate from `unscored` because the two are declined for opposite reasons.
   An unscored item is one nobody could decide; a duplicate is one already
   decided, at another position. Round three drew seven of them, 14 percent of
   the sample, and counting them as false positives dragged strict precision
   from 0.740 to 0.680 while every other reading rose. That movement described
   the sampling instrument, not the detector.

   The pipeline emitting one defect as several accepted issues is a real
   defect of its own, tracked separately; it is simply not the thing precision
   measures.
   */
    | 'duplicate';

/**
 One item read back off a graded sheet.

 @example
 ```ts
 const item: GradedItem = { index: 1, verdict: 'real-defect', note: '', };
 ```
 */
export type GradedItem = {
  /**
   1-based position on the sheet, which is what a pre-grade is keyed by.
   */
  readonly index: number;

  /**
   Verdict the grader recorded.
   */
  readonly verdict: GradeVerdict;

  /**
   Free text the grader added, empty when they added none. Round one and round
   two both carry rationale that nothing else reproduces, so it is preserved
   rather than discarded once the verdict is extracted.
   */
  readonly note: string;
};

/**
 Marker beginning a sheet item.
 */
const ITEM_PREFIX = '### ';

/**
 Marker introducing the grade on a detection-sheet item.
 */
const GRADE_MARKER = 'grade:';

/**
 Legend that follows every grade, and therefore bounds the grader's answer.
 */
const LEGEND_MARKER = '(Y = ';

/**
 Reads the grader's answer out of one item heading, with the legend and any
 enclosing brackets removed.

 @param line - item heading line

 @returns Answer text, empty when the box was left unfilled

 @example
 ```ts
 const answer = extractAnswer({ line: '### 1. grade: [Y]  (Y = ...)', },);
 ```
 */
function extractAnswer({ line, }: { readonly line: string; },): string {
  /**
   Where the grade begins. The item filter keeps only lines carrying the
   marker, so this is always a real offset.
   */
  const start = line.indexOf(GRADE_MARKER,);

  // LOUD RATHER THAN SLICED FROM A MISS. Without this check a heading
  // lacking the marker would have its opening characters cut off as if they
  // were the marker, and whatever followed read as the grader's answer
  // (ledger M113).
  if (start === (-1))
    throw new Error(
      'unreachable: an item heading without its grade marker, though parseGradedSheet keeps only headings '
        + 'that carry it',
    );

  /**
   Everything after the marker, with the trailing legend cut off. The legend
   is last on the line, so the LAST occurrence bounds the answer even when a
   rationale quotes the legend's own wording.
   */
  const afterMarker = line.slice(start + GRADE_MARKER.length,);

  /**
   Where the legend starts within that, or the end when it is absent.
   */
  const legendAt = afterMarker.lastIndexOf(LEGEND_MARKER,);

  /**
   Grader's answer, still possibly bracketed.
   */
  const answer = (legendAt === (-1)
    ? afterMarker
    : afterMarker.slice(
      0,
      legendAt,
    ))
    .trim();

  if (answer.startsWith('[',) && answer.endsWith(']',))
    return answer.slice(
      1,
      -1,
    )
      .trim();
  return answer;
}

/**
 Answer marking an item as the same defect as an earlier one, lowercased.

 A word rather than a letter, because it is not a verdict about the item: the
 grader is saying the question was already answered elsewhere on the sheet.
 */
const DUPLICATE_ANSWER = 'duplicate';

/**
 Classifies one grader answer into a verdict and its remaining prose.

 @param answer - grader's answer, unbracketed

 @returns Verdict and the note that followed it

 @example
 ```ts
 const read = readAnswer({ answer: 'N, anchored to the wrong text', },);
 ```
 */
function readAnswer({ answer, }: { readonly answer: string; },): {
  readonly verdict: GradeVerdict;
  readonly note: string;
} {
  for (const [
    letter,
    verdict,
  ] of [
    [
      'Y',
      'real-defect',
    ],
    [
      'N',
      'false-positive',
    ],
  ] as const)
    if (opensWithVerdict({
      answer,
      letter,
    },))
      return {
        verdict,
        note: trimLeadingDelimiters({ text: answer.slice(letter.length,), },),
      };

  // Checked AFTER the letters, not before, so an answer opening with a real
  // verdict keeps it however the grader continued the sentence.
  if (answer
    .trimStart()
    .toLowerCase()
    .startsWith(DUPLICATE_ANSWER,))
    return {
      verdict: 'duplicate',
      note: answer,
    };

  // Everything else, including an untouched `[ ]` box and answers like
  // "Not enough context to grade", is deliberately not a verdict.
  return {
    verdict: 'unscored',
    note: answer,
  };
}

/**
 How a refusal names the number a sheet heading printed.

 @param printed - heading's number as written, spaces trimmed

 @returns Digits as written; anything else quoted, so a stray space or sign
 shows; or that the heading carries none, where `Number` said zero

 @example
 ```ts
 headedWords({ printed: '', },); // 'with no number'
 ```
 */
function headedWords({ printed, }: { readonly printed: string; },): string {
  if (printed === '')
    return 'with no number';
  return isWholeNumberText({ text: printed, },) ? printed : JSON.stringify(printed,);
}

/**
 Reads every graded item off a filled detection sheet.

 @param text - sheet as the grader left it

 @returns Items in sheet order

 @example
 ```ts
 const items = parseGradedSheet({ text: await readFile(path, 'utf8',), },);
 ```
 */
export function parseGradedSheet(
  { text, }: { readonly text: string; },
): readonly GradedItem[] {
  return text.split('\n',)
    .filter(function isItemHeading(line,) {
      return line.startsWith(ITEM_PREFIX,)
        && line.includes(GRADE_MARKER,);
    },)
    .map(function toItem(
      line,
      position,
    ): GradedItem {
      /**
       Number the heading prints, which the pre-grades are keyed by.

       READ AND CHECKED rather than trusted from the position: a heading a
       grader deleted, duplicated or added by hand renumbers every later item
       silently against pre-grades keyed by the printed number.
       */
      const [numberText = '',] = line
        .slice(ITEM_PREFIX.length,)
        .split('.',);
      /**
       That number as written, without the spaces a hand edit may leave.
       */
      const printed = numberText.trim();
      // DIGITS AS THE SHEET WRITES THEM, naming the position: `Number` read
      // a heading with no number as headed 0, and `1e0` as position 1
      // (ledger B73).
      if ((!isWholeNumberText({ text: printed, },)) || (Number(printed,) !== (position + 1)))
        throw new StatedRefusalError({
          says: `sheet item at position ${String(position + 1,)} is headed ${
            headedWords({ printed, },)
          }; a heading was added, deleted or duplicated by hand`,
        },);

      /**
       Verdict and note read off this heading.
       */
      const read = readAnswer({ answer: extractAnswer({ line, },), },);
      return {
        index: position + 1,
        verdict: read.verdict,
        note: read.note,
      };
    },);
}

//endregion Graded sheet reading
