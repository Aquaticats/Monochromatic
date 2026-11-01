import {
  FENCE_CHARACTER,
  PROMPT_FENCE_MIN,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Sheet evidence copied into a candidate
// MEASURED ON MIO27 (2026-09-17): of 68 translate slate candidates, four
// carried the writer sheet's own WHAT THE PICTURES HERE SAY block, copied
// under the picture component with the readers' transcript inside, and on
// both slices that had one the judges chose it (three of four on slice 11,
// praising it for "all transcribed text exactly as in the ORIGINAL"). The
// contest chose the poem slice's carrier five of five; the consolidation
// replaced it because it was 10.9 times the original's size, not because
// anyone named the block. A sheet's evidence blocks are shown to a writer and
// are never part of the passage, so a candidate carrying one of their labels
// is not a rendering of the original and never reaches a judge.
//
// LEDGER F-8 AND E7 (2026-09-27). `SHEET_LABELS` had fallen behind the
// sheets: six consolidation-writer and polish headers and the translate
// sheet's REJECTED CANDIDATE N were missing, so a candidate carrying one
// passed. A list kept by hand will fall behind again, so every fenced header
// line (a run of at least `PROMPT_FENCE_MIN` fence characters, a label, and
// another run) is refused whatever its label, unless the original or the page
// carries the same line; the list stays for a head copied without its fence.

/**
 Heads of the labels the sheets fence their evidence blocks with, none of
 which a page carries; matched anywhere, fenced or not. The last three are
 the editor sheet's (`edit-prompt.ts`): the marker it sets where a region
 stands in its context, and its two unfenced line heads, none of which any
 pinned original or archive carries (ledger B24).
 */
const SHEET_LABELS: readonly string[] = [
  'WHAT THE PICTURES HERE SAY',
  'EXISTING TRANSLATION',
  'ATTESTED DETAILS',
  'DECLARED NAMES',
  'CITED REFERENCES',
  'LATEST REJECTION',
  'ARCHIVE RENDERING',
  'REJECTED CANDIDATE',
  'WHAT THE JUDGES FOUND',
  'REQUIRED FINDINGS',
  'PRIOR CORRECTION STRATEGIES',
  'ORIGINAL (Chinese)',
  'SURROUNDING EXISTING TRANSLATION',
  'SURROUNDING ENGLISH',
  'PASSAGE BEING REPLACED',
  'EXISTING ENGLISH BEFORE REPAIR',
  '«REGION',
  'CURRENT TEXT:',
  'CONTEXT: ...',
];

/**
 How many fence characters a line opens with.

 @param line - one line, trimmed

 @returns Length of the leading run

 @example
 ```ts
 fenceRunAtStart({ line: '===== X =====', },); // 5
 ```
 */
function fenceRunAtStart({ line, }: { readonly line: string; },): number {
  for (let at = 0; at < line.length; at += 1) {
    if (line.charAt(at,) !== FENCE_CHARACTER)
      return at;
  }
  return line.length;
}

/**
 How many fence characters a line closes with.

 @param line - one line, trimmed

 @returns Length of the trailing run

 @example
 ```ts
 fenceRunAtEnd({ line: '===== X =====', },); // 5
 ```
 */
function fenceRunAtEnd({ line, }: { readonly line: string; },): number {
  /**
   Offset of the line's last character.
   */
  const last = line.length - 1;
  for (let at = last; at >= 0; at -= 1) {
    if (line.charAt(at,) !== FENCE_CHARACTER)
      return last - at;
  }
  return line.length;
}

/**
 The label of a fenced header line: a run of fence characters, a label, and
 another run, each run at least the sheets' shortest fence.

 A LABEL IS WORDS A READER SEES, asked of `rendersAsNothing` (ledger B40):
 asked of the trimmed text against the empty string, two fence runs around
 one zero-width space or Hangul filler were refused as a sheet block whose
 label the finding printed as nothing, where two runs around spaces passed.

 @param line - one line of a candidate

 @returns The label, or none for any other line and for a label that shows a
 reader nothing

 @example
 ```ts
 fencedLabel({ line: '===== ATTESTED DETAILS =====', },); // ['ATTESTED DETAILS']
 ```
 */
function fencedLabel({ line, }: { readonly line: string; },): readonly string[] {
  /**
   The line without surrounding space.
   */
  const trimmed = line.trim();
  /**
   The leading run.
   */
  const opening = fenceRunAtStart({ line: trimmed, },);
  /**
   The trailing run.
   */
  const closing = fenceRunAtEnd({ line: trimmed, },);
  /**
   Whether both runs are as long as the sheets' shortest fence.
   */
  const fencedBothSides = (opening >= PROMPT_FENCE_MIN) && (closing >= PROMPT_FENCE_MIN);
  if ((!fencedBothSides) || ((opening + closing) >= trimmed.length))
    return [];
  /**
   What stands between the runs.
   */
  const label = trimmed.slice(
    opening,
    trimmed.length - closing,
  )
    .trim();
  return rendersAsNothing({ text: label, },) ? [] : [label,];
}

/**
 Every line of a text, trimmed, for asking whether it carries a line.

 @param text - original or page

 @returns Its trimmed lines

 @example
 ```ts
 trimmedLines({ text: ' a \nb', },); // Set {'a', 'b'}
 ```
 */
function trimmedLines({ text, }: { readonly text: string; },): ReadonlySet<string> {
  return new Set(text
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },),);
}

/**
 Findings for a candidate that carries one of the sheet's evidence labels the
 original and the page do not.

 @param sourceText - original slice

 @param pageText - page slice the candidate would replace, empty where none

 @param candidateText - candidate under validation

 @returns One finding per listed head the candidate alone carries, and one
 per fenced header line under any other label

 @example
 ```ts
 const findings = sheetLeakFindings({ sourceText: '猫睡了。', pageText: '', candidateText: 'The cat slept.\n===== ATTESTED DETAILS =====', },);
 ```
 */
export function sheetLeakFindings(
  {
    sourceText,
    pageText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly pageText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  /**
   Listed heads the candidate alone carries.
   */
  const listed = SHEET_LABELS.filter(function leaked(label,): boolean {
    return candidateText.includes(label,)
      && (!sourceText.includes(label,))
      && (!pageText.includes(label,));
  },);
  /**
   Lines the original or the page already carries.
   */
  const carried = new Set([
    ...trimmedLines({ text: sourceText, },),
    ...trimmedLines({ text: pageText, },),
  ],);
  /**
   Labels of fenced header lines the candidate alone carries, where no listed
   head already names them.
   */
  const fenced = candidateText
    .split('\n',)
    .filter(function newLine(line,): boolean {
      return !carried.has(line.trim(),);
    },)
    .flatMap(function labelOf(line,): readonly string[] {
      return fencedLabel({ line, },);
    },)
    .filter(function unnamed(label,): boolean {
      return !listed.some(function covers(head,): boolean {
        return label.includes(head,);
      },);
    },);
  return [
    ...listed,
    ...fenced,
  ].map(function toFinding(label,): string {
    return `Your translation carries the sheet's own "${label}" block, which is evidence shown to you and `
      + 'never part of the passage. Render the ORIGINAL alone and leave every fenced block out.';
  },);
}

//endregion Sheet evidence copied into a candidate
